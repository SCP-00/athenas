use std::path::PathBuf;

use super::recovery::ExperimentConfig;
use crate::runtime::hardware::HardwareInfo;
use crate::runtime::model_intelligence::vram::{KvCacheType, MemoryConfig, VramCalculator};
use crate::runtime::runtime_discovery::RuntimeCapabilities;

// ── Experiment Description ──

#[derive(Debug, Clone)]
pub struct ExperimentDescription {
    /// Unique name for this experiment
    pub name: String,
    /// Runtime to use
    pub runtime: String,
    /// Runtime binary path
    pub runtime_path: String,
    /// Model path
    pub model_path: PathBuf,
    /// Experiment configuration to try
    pub config: ExperimentConfig,
    /// Rationale — why the planner chose this experiment
    pub rationale: String,
    /// Expected outcome (prediction before running)
    pub expected_outcome: String,
}

// ── Knowledge Entry (learned from experiments) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExperimentKnowledge {
    pub hardware_fingerprint: String,
    pub model_name: String,
    pub runtime_name: String,
    pub kv_cache_type: String,
    pub kv_in_ram: bool,
    pub context_size: u64,
    pub batch_size: u64,
    pub ubatch_size: u64,
    pub gpu_layers: u64,
    pub total_layers: u64,
    pub success: bool,
    pub oom: bool,
    pub tokens_per_second: f64,
    pub vram_used_gb: f64,
    pub ram_used_gb: f64,
    pub failure_reason: Option<String>,
    pub timestamp: String,
}

// ── Experiment Planner ──
// Uses heuristic search, not brute force.
// Learns from each experiment to narrow the search space.

pub struct ExperimentPlanner {
    /// Knowledge cache from previous experiments (avoids repeating failures)
    knowledge_cache: Vec<ExperimentKnowledge>,
    /// Hardware this planner is targeting
    hardware: HardwareInfo,
    /// Model path
    model_path: PathBuf,
    /// Model metadata
    model_params_b: f64,
    model_quant_bits: f64,
    model_context: u64,
    model_embed_dim: u64,
    model_heads: u64,
    model_kv_heads: u64,
    model_layers: u64,
    /// Available runtimes
    runtimes: Vec<RuntimeCapabilities>,
    /// OOM frontier — highest context that failed (per runtime+KV type)
    oom_frontier: Vec<(String, String, u64)>, // (runtime, kv_type, context that OOM'd)
    /// Success frontier — highest context that succeeded
    success_frontier: Vec<(String, String, u64)>,
}

impl ExperimentPlanner {
    pub fn new(
        hardware: HardwareInfo,
        model_path: PathBuf,
        model_params_b: f64,
        model_quant_bits: f64,
        model_context: u64,
        model_embed_dim: u64,
        model_heads: u64,
        model_kv_heads: u64,
        model_layers: u64,
        runtimes: Vec<RuntimeCapabilities>,
    ) -> Self {
        Self {
            knowledge_cache: Vec::new(),
            hardware,
            model_path,
            model_params_b,
            model_quant_bits,
            model_context,
            model_embed_dim,
            model_heads,
            model_kv_heads,
            model_layers,
            runtimes,
            oom_frontier: Vec::new(),
            success_frontier: Vec::new(),
        }
    }

    /// Load knowledge from previous experiments to avoid repeating failures.
    pub fn load_knowledge(&mut self, knowledge: Vec<ExperimentKnowledge>) {
        self.knowledge_cache = knowledge;

        // Rebuild frontiers from knowledge
        for k in &self.knowledge_cache {
            if k.oom {
                self.oom_frontier.push((
                    k.runtime_name.clone(),
                    k.kv_cache_type.clone(),
                    k.context_size,
                ));
            } else if k.success {
                self.success_frontier.push((
                    k.runtime_name.clone(),
                    k.kv_cache_type.clone(),
                    k.context_size,
                ));
            }
        }
    }

    /// Generate the experiment plan.
    /// This is NOT brute force — it uses heuristic search:
    /// 1. Start with the best-guess configuration (from VRAM Calculator)
    /// 2. Try increasing context to find the OOM frontier
    /// 3. Try improving KV cache quality
    /// 4. Learn from each result
    pub fn generate_plan(&self) -> Vec<ExperimentDescription> {
        let mut plan = Vec::new();
        let vram_avail = self.hardware.gpu.first().map(|g| g.vram_gb).unwrap_or(0.0);
        let ram_avail = self.hardware.memory.available_gb;

        // Early exit: no GPU or no RAM
        if vram_avail <= 0.0 || ram_avail <= 0.0 {
            return plan;
        }

        // Phase 1: Try the best-guess configurations from each runtime
        for rt in &self.runtimes {
            if rt.capability_score() < 0.35 {
                continue; // Skip runtimes with very low capability (Ollama, Pi, etc.)
            }

            let runtime_name = rt.display_name_short();

            // Use VRAM Calculator to find the best config for this runtime
            let best_configs = VramCalculator::find_best_config(
                self.model_params_b,
                self.model_quant_bits,
                self.model_context,
                self.model_embed_dim,
                self.model_heads,
                self.model_kv_heads,
                self.model_layers,
                vram_avail,
                ram_avail,
            );

            // Take the top configs and create experiments
            // Skip any config that we already know will OOM (from knowledge cache)
            let mut experiments_added = 0;
            for est in &best_configs {
                if experiments_added >= 3 {
                    break; // Max 3 experiments per runtime
                }

                // Check if this config is already in the OOM frontier
                let kv_name = est.config.kv_cache_type.name().to_string();
                let rt_name_oom = runtime_name.clone();
                let would_oom = self.oom_frontier.iter().any(|(r, k, c)| {
                    r == &rt_name_oom
                        && k == &kv_name
                        && est.config.context_length <= *c
                });

                if would_oom {
                    continue; // Skip — we already know this fails
                }

                // Check if this config is already in the success frontier
                let rt_name_success = runtime_name.clone();
                let already_succeeded = self.success_frontier.iter().any(|(r, k, c)| {
                    r == &rt_name_success
                        && k == &kv_name
                        && est.config.context_length == *c
                        && est.config.kv_in_ram == est.config.kv_in_ram
                });

                if already_succeeded {
                    continue; // Skip — we already know this works
                }

                let kv_type_str = if est.config.kv_in_ram {
                    format!("{} (RAM)", kv_name)
                } else {
                    kv_name.clone()
                };

                let rt_name_clone = runtime_name.clone();
                let name = format!(
                    "{}-{}-ctx{}",
                    rt_name_clone.replace(' ', "-"),
                    kv_type_str.replace(' ', "-"),
                    est.config.context_length / 1024
                );

                let fits_str = if est.fits_in_vram { "should fit" } else { "borderline" };
                let rationale = format!(
                    "VRAM estimate: {:.1}GB / {:.1}GB avail, KV={} ({}), OOM risk {:.0}%",
                    est.vram_total_gb, vram_avail, kv_name,
                    fits_str, est.oom_risk * 100.0
                );

                let expected = if est.fits_in_vram && est.oom_risk < 0.3 {
                    format!("Expected success (estimated ~{:.0} tok/s)", est.estimated_tok_s)
                } else {
                    format!("Expected OOM or instability (risk {:.0}%)", est.oom_risk * 100.0)
                };

                plan.push(ExperimentDescription {
                    name,
                    runtime: runtime_name.clone(),
                    runtime_path: rt.binary_path.clone(),
                    model_path: self.model_path.clone(),
                    config: ExperimentConfig {
                        context_length: est.config.context_length,
                        kv_cache_type: est.config.kv_cache_type,
                        kv_in_ram: est.config.kv_in_ram,
                        batch_size: est.config.batch_size,
                        ubatch_size: est.config.ubatch_size,
                        gpu_layers: est.config.gpu_layers,
                        total_layers: est.config.total_layers,
                        quantization_bits: est.config.quantization_bits,
                        model_params_b: est.config.model_params_b,
                        embedding_dim: est.config.embedding_dim,
                        num_heads: est.config.num_heads,
                        num_kv_heads: est.config.num_kv_heads,
                    },
                    rationale,
                    expected_outcome: expected,
                });

                experiments_added += 1;
            }
        }

        plan
    }

    /// After an experiment fails with OOM, record the frontier.
    pub fn record_oom(&mut self, runtime: &str, kv_type: &str, context: u64) {
        // Record that this config OOM'd — never try anything "worse" (more context, same KV) again
        self.oom_frontier.push((
            runtime.to_string(),
            kv_type.to_string(),
            context,
        ));
    }

    /// After an experiment succeeds, record the success.
    pub fn record_success(&mut self, runtime: &str, kv_type: &str, context: u64) {
        self.success_frontier.push((
            runtime.to_string(),
            kv_type.to_string(),
            context,
        ));
    }

    /// Suggest the next experiment to try, given results so far.
    /// After initial experiments, try to push boundaries:
    /// - If a config succeeded, try more context (125%)
    /// - If a config failed, try different KV or runtime
    pub fn suggest_next(
        &self,
        last_runtime: &str,
        last_config: &ExperimentConfig,
        last_succeeded: bool,
    ) -> Option<ExperimentDescription> {
        let rt = self.runtimes.iter().find(|r| r.display_name_short() == last_runtime)?;
        let vram_avail = self.hardware.gpu.first().map(|g| g.vram_gb).unwrap_or(0.0);
        let ram_avail = self.hardware.memory.available_gb;

        if last_succeeded {
            // Try 25% more context with the same configuration
            let new_ctx = (last_config.context_length as f64 * 1.25) as u64;
            let new_ctx = new_ctx.min(self.model_context);

            if new_ctx > last_config.context_length {
                let kv_name = last_config.kv_cache_type.name().to_string();
                let name = format!("{}-{}-ctx{}", last_runtime.replace(' ', "-"), kv_name, new_ctx / 1024);

                // Check frontiers
                let would_oom = self.oom_frontier.iter().any(|(r, k, c)| {
                    r == last_runtime && k == &kv_name && new_ctx <= *c
                });

                if !would_oom {
                    return Some(ExperimentDescription {
                        name,
                        runtime: last_runtime.to_string(),
                        runtime_path: rt.binary_path.clone(),
                        model_path: self.model_path.clone(),
                        config: ExperimentConfig {
                            context_length: new_ctx,
                            ..last_config.clone()
                        },
                        rationale: format!("Previous config succeeded at {}K — trying {}K", last_config.context_length / 1024, new_ctx / 1024),
                        expected_outcome: format!("Unknown — pushing context beyond validated point"),
                    });
                }
            }
        } else {
            // Failed — try a different approach
            // Strategy: Switch KV cache type (e.g., Q8 → Turbo3, or VRAM → RAM)
            let current_kv = last_config.kv_cache_type;
            let current_kv_ram = last_config.kv_in_ram;

            // Try a more memory-efficient KV type
            let better_kv = [
                KvCacheType::Fp16,
                KvCacheType::Q8,
                KvCacheType::Q4,
                KvCacheType::Turbo3,
                KvCacheType::Turbo2,
            ];

            for new_kv in &better_kv {
                if *new_kv == current_kv {
                    continue;
                }
                let kv_name = new_kv.name();
                let would_oom = self.oom_frontier.iter().any(|(r, k, c)| {
                    r == last_runtime && k == kv_name && last_config.context_length <= *c
                });

                if !would_oom {
                    let config = ExperimentConfig {
                        kv_cache_type: *new_kv,
                        kv_in_ram: !current_kv_ram, // Try moving to RAM (or back to VRAM)
                        ..last_config.clone()
                    };
                    return Some(ExperimentDescription {
                        name: format!("{}-{}-retry", last_runtime.replace(' ', "-"), kv_name),
                        runtime: last_runtime.to_string(),
                        runtime_path: rt.binary_path.clone(),
                        model_path: self.model_path.clone(),
                        config,
                        rationale: format!("Previous config OOM'd — switching KV to {} ({} RAM)",
                            kv_name, if !current_kv_ram { "in" } else { "not in" }),
                        expected_outcome: "Attempting recovery with different KV strategy".to_string(),
                    });
                }
            }
        }

        None // No more experiments to suggest
    }

    /// Check if a configuration is known to fail (from OOM frontier)
    pub fn is_known_failure(&self, runtime: &str, kv_type: &str, context: u64) -> bool {
        self.oom_frontier.iter().any(|(r, k, c)| {
            r == runtime && k == kv_type && context <= *c
        })
    }
}
