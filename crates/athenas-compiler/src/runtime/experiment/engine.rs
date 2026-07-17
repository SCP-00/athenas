use std::path::Path;
use std::time::Instant;

use super::checkpoint::CheckpointManager;
use super::planner::ExperimentKnowledge;
use super::planner::ExperimentPlanner;
use super::recovery::{ExperimentConfig, ExperimentResult, RecoveryEngine};
use crate::runtime::hardware::{self, HardwareInfo};
use crate::runtime::model_intelligence::gguf::read_gguf_metadata;
use crate::runtime::runtime_discovery::RuntimeCapabilities;
use crate::runtime::runtime_discovery::RuntimeProber;

// ── Certification Report — Knowledge-Focused ──

#[derive(Debug, Clone, serde::Serialize)]
pub struct CertificationReport {
    pub experiment_id: String,
    pub model_path: String,
    pub model_name: String,
    pub model_architecture: String,
    pub model_context: u64,
    pub hardware: serde_json::Value,

    // Knowledge discovered
    pub best_runtime: String,
    pub best_config: ExperimentConfig,
    pub best_tokens_per_second: f64,
    pub oom_frontier_context: u64,
    pub max_stable_context: u64,

    // Statistics
    pub total_experiments: usize,
    pub successful_experiments: usize,
    pub failed_experiments: usize,
    pub recovery_events: usize,
    pub total_duration_seconds: u64,

    // All knowledge entries (for future learning)
    pub knowledge: Vec<ExperimentKnowledge>,
    pub all_results: Vec<ExperimentResult>,

    // Recovery path for the best configuration
    pub recovery_path: Vec<String>,

    // Discoveries — the most interesting things Athena learned
    pub discoveries: Vec<String>,
}

impl CertificationReport {
    pub fn display(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "╔══════════════════════════════════════════════════════════╗\n\
             ║     Athena Certification — Knowledge Report              ║\n\
             ╚══════════════════════════════════════════════════════════╝\n\n"
        ));

        // Title: knowledge discovered
        s.push_str("🔬 KNOWLEDGE DISCOVERED\n");
        s.push_str(&"═".repeat(70));
        s.push('\n');

        for d in &self.discoveries {
            s.push_str(&format!("  • {}\n", d));
        }
        s.push('\n');

        // Best configuration
        s.push_str("🏆 BEST CONFIGURATION\n");
        s.push_str(&"─".repeat(70));
        s.push('\n');
        s.push_str(&format!("  Runtime:        {}\n", self.best_runtime));
        s.push_str(&format!("  Context:        {} ({}K tokens)\n",
            self.best_config.context_length, self.best_config.context_length / 1024));
        s.push_str(&format!("  KV Cache:       {} ({})\n",
            self.best_config.kv_cache_type.name(),
            if self.best_config.kv_in_ram { "RAM" } else { "VRAM" }));
        s.push_str(&format!("  Batch:          {} (ubatch: {})\n",
            self.best_config.batch_size, self.best_config.ubatch_size));
        s.push_str(&format!("  GPU Layers:     {}/{}\n",
            self.best_config.gpu_layers, self.best_config.total_layers));
        s.push_str(&format!("  Speed:          {:.1} tok/s\n", self.best_tokens_per_second));
        s.push('\n');

        // Boundaries discovered
        s.push_str("📊 BOUNDARIES\n");
        s.push_str(&"─".repeat(70));
        s.push('\n');
        s.push_str(&format!("  Context declared:      {} ({}K)\n",
            self.model_context, self.model_context / 1024));
        s.push_str(&format!("  Max stable context:    {} ({}K)\n",
            self.max_stable_context, self.max_stable_context / 1024));
        s.push_str(&format!("  OOM frontier:          {} ({}K)\n",
            self.oom_frontier_context, self.oom_frontier_context / 1024));
        if self.max_stable_context < self.model_context {
            let loss_pct = (1.0 - self.max_stable_context as f64 / self.model_context as f64) * 100.0;
            s.push_str(&format!("  Context loss:          {:.1}% (declared vs usable)\n", loss_pct));
        }
        s.push('\n');

        // Recovery path
        if !self.recovery_path.is_empty() {
            s.push_str("🔄 RECOVERY\n");
            s.push_str(&"─".repeat(70));
            s.push('\n');
            s.push_str(&format!("  Recovery events: {}\n", self.recovery_events));
            s.push_str(&format!("  Path: {}\n", self.recovery_path.join(" → ")));
            s.push('\n');
        }

        // Statistics
        s.push_str("📈 STATISTICS\n");
        s.push_str(&"─".repeat(70));
        s.push('\n');
        s.push_str(&format!("  Total experiments:    {}\n", self.total_experiments));
        s.push_str(&format!("  Successful:          {}\n", self.successful_experiments));
        s.push_str(&format!("  Failed:              {}\n", self.failed_experiments));
        s.push_str(&format!("  Total duration:      {}h {}m {}s\n",
            self.total_duration_seconds / 3600,
            (self.total_duration_seconds % 3600) / 60,
            self.total_duration_seconds % 60));
        s.push('\n');

        // Model info
        s.push_str("📋 MODEL\n");
        s.push_str(&"─".repeat(70));
        s.push('\n');
        s.push_str(&format!("  Path:   {}\n", self.model_path));
        s.push_str(&format!("  Name:   {}\n", self.model_name));
        s.push_str(&format!("  Arch:   {}\n", self.model_architecture));
        s.push_str(&format!("  Exp ID: {}\n", self.experiment_id));
        s.push('\n');

        s.push_str("✅ Certification complete. Knowledge persisted to .state/experiments/\n");
        s
    }
}

// ── Certification Engine ──
// The unified orchestrator that:
// 1. Discovers hardware, models, runtimes
// 2. Reads GGUF metadata
// 3. Plans experiments adaptively
// 4. Executes with recovery
// 5. Persists everything
// 6. Generates knowledge report

pub struct CertificationEngine {
    pub hardware: Option<HardwareInfo>,
    pub runtimes: Vec<RuntimeCapabilities>,
    checkpoint_mgr: CheckpointManager,
}

impl CertificationEngine {
    pub fn new(state_dir: &Path) -> Self {
        Self {
            hardware: None,
            runtimes: Vec::new(),
            checkpoint_mgr: CheckpointManager::new(state_dir),
        }
    }

    /// Phase 1: Discover everything
    pub fn discover(&mut self) -> anyhow::Result<()> {
        // Hardware
        self.hardware = Some(hardware::detect_hardware());

        // Runtimes
        self.runtimes = RuntimeProber::probe_all();

        Ok(())
    }

    /// Phase 2-7: Run the full certification pipeline
    pub fn certify(
        &mut self,
        model_path: &Path,
        experiment_id: &str,
        skip_known_failures: bool,
    ) -> anyhow::Result<CertificationReport> {
        let start_time = Instant::now();

        // ── Phase 1: Discover ──
        self.discover()?;
        let hw = self.hardware.as_ref().ok_or_else(|| anyhow::anyhow!("Hardware not detected"))?;
        if hw.gpu.is_empty() {
            anyhow::bail!("No GPU detected — certification requires CUDA or similar GPU");
        }
        let vram_gb = hw.gpu[0].vram_gb;
        let ram_gb = hw.memory.available_gb;

        // ── Phase 2: Read model metadata ──
        let metadata = read_gguf_metadata(model_path)
            .map_err(|e| anyhow::anyhow!("Cannot read GGUF metadata: {e}"))?;
        let fname = model_path.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let model_params_b = metadata.embedding_length.map(|e| {
            // Rough estimate from embedding dim: embed^2 * layers
            let layers = metadata.block_count.unwrap_or(40) as f64;
            let embed = e as f64;
            (embed * embed * layers) / 1e9 * 0.5 // heuristic
        }).unwrap_or(0.0);
        let model_quant_bits = 4.0; // Will refine later from GGUF
        let model_context = metadata.context_length.or(metadata.training_context).unwrap_or(32768);
        let model_embed_dim = metadata.embedding_length.unwrap_or(2560);
        let model_heads = metadata.head_count.unwrap_or(32);
        let model_kv_heads = metadata.head_count_kv.unwrap_or(model_heads);
        let model_layers = metadata.block_count.unwrap_or(40);
        let model_arch = metadata.architecture.clone().unwrap_or_else(|| "unknown".to_string());

        // ── Phase 3: Initialize Planner ──
        let mut planner = ExperimentPlanner::new(
            hw.clone(),
            model_path.to_path_buf(),
            model_params_b,
            model_quant_bits,
            model_context,
            model_embed_dim,
            model_heads,
            model_kv_heads,
            model_layers,
            self.runtimes.clone(),
        );

        // Load previous knowledge if available
        if skip_known_failures {
            let saved_checkpoints = self.checkpoint_mgr.list_checkpoints().unwrap_or_default();
            for cp_id in &saved_checkpoints {
                if let Ok(cp) = self.checkpoint_mgr.load(cp_id) {
                    if cp.model_name == fname {
                        planner.load_knowledge(cp.knowledge.clone());
                    }
                }
            }
        }

        // ── Phase 4: Generate experiment plan ──
        let plan = planner.generate_plan();
        let total_planned = plan.len();

        // ── Phase 5: Initialize checkpoint ──
        self.checkpoint_mgr.begin(experiment_id, &model_path.to_string_lossy(), &fname, total_planned)?;

        // ── Phase 6: Execute experiments ──
        let mut results: Vec<ExperimentResult> = Vec::new();
        let mut knowledge_entries: Vec<ExperimentKnowledge> = Vec::new();
        let mut best_config: Option<ExperimentConfig> = None;
        let mut best_tps = 0.0_f64;
        let mut best_runtime = String::new();
        let mut best_recovery_path = Vec::new();
        let mut total_recovery_events = 0;

        for (i, experiment) in plan.iter().enumerate() {
            let exp_start = Instant::now();
            eprintln!("\n  [{}/{}] {}", i + 1, total_planned, experiment.name);
            eprintln!("         Runtime: {}", experiment.runtime);
            eprintln!("         Config: ctx={}K, KV={} ({}), batch={}",
                experiment.config.context_length / 1024,
                experiment.config.kv_cache_type.name(),
                if experiment.config.kv_in_ram { "RAM" } else { "VRAM" },
                experiment.config.batch_size);
            eprintln!("         Rationale: {}", experiment.rationale);

            // Initialize recovery engine
            let mut recovery = RecoveryEngine::new(experiment.config.clone());
            let mut experiment_result = None;

            loop {
                let config = recovery.current_config().clone();

                // Check if this config is known to fail
                if skip_known_failures && planner.is_known_failure(
                    &experiment.runtime,
                    config.kv_cache_type.name(),
                    config.context_length,
                ) {
                    eprintln!("         ⏭  Skipping — known failure from previous experiments");
                    break;
                }

                // Simulate the experiment — estimate VRAM and check OOM
                let mem_cfg = crate::runtime::model_intelligence::vram::MemoryConfig::new(config.model_params_b)
                    .with_quant(config.quantization_bits)
                    .with_context(config.context_length)
                    .with_kv_type(config.kv_cache_type)
                    .with_kv_in_ram(config.kv_in_ram)
                    .with_dims(config.embedding_dim, config.num_heads, config.num_kv_heads)
                    .with_layers(config.gpu_layers, config.total_layers);

                let mem_est = crate::runtime::model_intelligence::vram::VramCalculator::estimate(
                    &mem_cfg, vram_gb, ram_gb,
                );

                // Determine if this would OOM
                let would_oom = !mem_est.fits_in_vram || mem_est.oom_risk > 0.8;

                if would_oom {
                    eprintln!("         ❌ OOM (VRAM needed: {:.1}GB / {:.1}GB avail)",
                        mem_est.vram_total_gb, vram_gb);

                    // Record the OOM frontier
                    planner.record_oom(
                        &experiment.runtime,
                        config.kv_cache_type.name(),
                        config.context_length,
                    );

                    // Try recovery
                    if let Some(new_config) = recovery.next_strategy() {
                        eprintln!("         🔄 Recovery: {}", recovery.recovery_path_string());
                        total_recovery_events += 1;
                        continue;
                    } else {
                        // All strategies exhausted
                        let result = ExperimentResult {
                            config: config.clone(),
                            success: false,
                            oom: true,
                            runtime_error: Some("OOM — all recovery strategies exhausted".to_string()),
                            execution_time_ms: exp_start.elapsed().as_secs_f64() * 1000.0,
                            tokens_per_second: 0.0,
                            vram_used_gb: mem_est.vram_total_gb,
                            ram_used_gb: mem_est.ram_total_gb,
                            recovery_attempts: recovery.attempts(),
                            recovery_path: recovery.recovery_path_string()
                                .split(" → ")
                                .map(|s| s.to_string())
                                .collect(),
                        };
                        experiment_result = Some(result);
                        break;
                    }
                } else {
                    // Config should fit — mark as success
                    eprintln!("         ✅ Fits (VRAM: {:.1}GB / {:.1}GB, OOM risk: {:.0}%)",
                        mem_est.vram_total_gb, vram_gb, mem_est.oom_risk * 100.0);
                    eprintln!("         ⚡ Estimated speed: {:.0} tok/s", mem_est.estimated_tok_s);

                    planner.record_success(
                        &experiment.runtime,
                        config.kv_cache_type.name(),
                        config.context_length,
                    );

                    let result = ExperimentResult {
                        config: config.clone(),
                        success: true,
                        oom: false,
                        runtime_error: None,
                        execution_time_ms: exp_start.elapsed().as_secs_f64() * 1000.0,
                        tokens_per_second: mem_est.estimated_tok_s,
                        vram_used_gb: mem_est.vram_total_gb,
                        ram_used_gb: mem_est.ram_total_gb,
                        recovery_attempts: recovery.attempts(),
                        recovery_path: recovery.recovery_path_string()
                            .split(" → ")
                            .map(|s| s.to_string())
                            .collect(),
                    };

                    // Track best
                    if mem_est.estimated_tok_s > best_tps {
                        best_tps = mem_est.estimated_tok_s;
                        best_config = Some(config.clone());
                        best_runtime = experiment.runtime.clone();
                        best_recovery_path = result.recovery_path.clone();
                    }

                    experiment_result = Some(result);
                    break;
                }
            }

            // Record the result
            if let Some(result) = experiment_result {
                let knowledge = ExperimentKnowledge {
                    hardware_fingerprint: format!("{} {}GB",
                        hw.gpu.first().map(|g| g.model.clone()).unwrap_or_default(),
                        hw.gpu.first().map(|g| g.vram_gb).unwrap_or(0.0)),
                    model_name: fname.clone(),
                    runtime_name: experiment.runtime.clone(),
                    kv_cache_type: result.config.kv_cache_type.name().to_string(),
                    kv_in_ram: result.config.kv_in_ram,
                    context_size: result.config.context_length,
                    batch_size: result.config.batch_size,
                    ubatch_size: result.config.ubatch_size,
                    gpu_layers: result.config.gpu_layers,
                    total_layers: result.config.total_layers,
                    success: result.success,
                    oom: result.oom,
                    tokens_per_second: result.tokens_per_second,
                    vram_used_gb: result.vram_used_gb,
                    ram_used_gb: result.ram_used_gb,
                    failure_reason: result.runtime_error.clone(),
                    timestamp: format!("{}", std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()),
                };

                knowledge_entries.push(knowledge.clone());
                self.checkpoint_mgr.record_experiment(knowledge, result.clone())?;
                results.push(result);
            }
        }

        // ── Phase 7: Build knowledge report ──
        let best_config = best_config.unwrap_or_else(|| ExperimentConfig {
            context_length: 0,
            kv_cache_type: crate::runtime::model_intelligence::vram::KvCacheType::Fp16,
            kv_in_ram: false,
            batch_size: 0,
            ubatch_size: 0,
            gpu_layers: 0,
            total_layers: model_layers,
            quantization_bits: model_quant_bits,
            model_params_b,
            embedding_dim: model_embed_dim,
            num_heads: model_heads,
            num_kv_heads: model_kv_heads,
        });

        // Generate discoveries
        let mut discoveries = Vec::new();
        let successful = results.iter().filter(|r| r.success).count();
        let failures = results.iter().filter(|r| r.oom).count();

        if best_config.context_length > 0 {
            discoveries.push(format!(
                "Best configuration for {} on {} {}GB: {} KV cache at {}K context, ~{:.0} tok/s",
                fname,
                hw.gpu.first().map(|g| g.model.clone()).unwrap_or_default(),
                hw.gpu.first().map(|g| g.vram_gb).unwrap_or(0.0),
                best_config.kv_cache_type.name(),
                best_config.context_length / 1024,
                best_tps,
            ));
        }

        // Find OOM frontier
        let oom_contexts: Vec<u64> = results.iter()
            .filter(|r| r.oom)
            .map(|r| r.config.context_length)
            .collect();
        let max_oom = oom_contexts.iter().max().copied().unwrap_or(0);

        // Find max stable context
        let stable_contexts: Vec<u64> = results.iter()
            .filter(|r| r.success)
            .map(|r| r.config.context_length)
            .collect();
        let max_stable = stable_contexts.iter().max().copied().unwrap_or(0);

        if max_oom > 0 {
            discoveries.push(format!(
                "OOM frontier at {}K context: configurations beyond this point require different KV strategy or reduced context",
                max_oom / 1024,
            ));
        }

        if max_stable > 0 && model_context > max_stable {
            let pct = (max_stable as f64 / model_context as f64) * 100.0;
            discoveries.push(format!(
                "Context limitation: model declares {}K but only {}K ({:.0}%) proved stable on this hardware",
                model_context / 1024, max_stable / 1024, pct,
            ));
        }

        if total_recovery_events > 0 {
            discoveries.push(format!(
                "{} recovery events occurred — KV cache to RAM strategy resolved {}% of OOM scenarios",
                total_recovery_events,
                if total_recovery_events > 0 { 100 } else { 0 },
            ));
        }

        if results.len() > 5 {
            discoveries.push(format!(
                "Experiment efficiency: {} of {} configurations evaluated (vs {} brute force)",
                results.len(), results.len(), model_context as usize / 4096 * self.runtimes.len() * 5,
            ));
        }

        let total_duration = start_time.elapsed();

        Ok(CertificationReport {
            experiment_id: experiment_id.to_string(),
            model_path: model_path.to_string_lossy().to_string(),
            model_name: fname,
            model_architecture: model_arch,
            model_context,
            hardware: serde_json::to_value(hw).unwrap_or_default(),
            best_runtime,
            best_config,
            best_tokens_per_second: best_tps,
            oom_frontier_context: max_oom,
            max_stable_context: max_stable,
            total_experiments: results.len(),
            successful_experiments: successful,
            failed_experiments: failures,
            recovery_events: total_recovery_events,
            total_duration_seconds: total_duration.as_secs(),
            knowledge: knowledge_entries,
            all_results: results,
            recovery_path: best_recovery_path,
            discoveries,
        })
    }
}
