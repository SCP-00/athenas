use crate::runtime::model_intelligence::vram::KvCacheType;
use serde::Serialize;

// ── Experiment Configuration ──

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct ExperimentConfig {
    pub context_length: u64,
    pub kv_cache_type: KvCacheType,
    pub kv_in_ram: bool,
    pub batch_size: u64,
    pub ubatch_size: u64,
    pub gpu_layers: u64,
    pub total_layers: u64,
    pub quantization_bits: f64,
    pub model_params_b: f64,
    pub embedding_dim: u64,
    pub num_heads: u64,
    pub num_kv_heads: u64,
}

// ── Experiment Result ──

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct ExperimentResult {
    pub config: ExperimentConfig,
    pub success: bool,
    pub oom: bool,
    pub runtime_error: Option<String>,
    pub execution_time_ms: f64,
    pub tokens_per_second: f64,
    pub vram_used_gb: f64,
    pub ram_used_gb: f64,
    pub recovery_attempts: usize,
    pub recovery_path: Vec<String>,
}

// ── Recovery Strategy ──
// Each strategy is tried in order until one works.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecoveryStrategy {
    /// Move KV cache from VRAM to RAM
    KvToRam,
    /// Reduce batch size by half
    ReduceBatch,
    /// Reduce context by 25%
    ReduceContext25,
    /// Reduce GPU layers by 25%
    ReduceGpuLayers,
    /// Switch to a more memory-efficient KV cache type
    KvCacheFp16ToQ8,
    KvCacheQ8ToQ4,
    KvCacheQ4ToTurbo3,
    /// Abandon this configuration branch entirely
    AbandonBranch,
}

impl RecoveryStrategy {
    pub fn name(&self) -> &str {
        match self {
            RecoveryStrategy::KvToRam => "Move KV cache to RAM",
            RecoveryStrategy::ReduceBatch => "Reduce batch size 50%",
            RecoveryStrategy::ReduceContext25 => "Reduce context 25%",
            RecoveryStrategy::ReduceGpuLayers => "Reduce GPU layers 25%",
            RecoveryStrategy::KvCacheFp16ToQ8 => "KV FP16 → Q8",
            RecoveryStrategy::KvCacheQ8ToQ4 => "KV Q8 → Q4",
            RecoveryStrategy::KvCacheQ4ToTurbo3 => "KV Q4 → Turbo3",
            RecoveryStrategy::AbandonBranch => "Abandon configuration branch",
        }
    }
}

// ── Recovery Engine ──

pub struct RecoveryEngine {
    strategies_attempted: Vec<RecoveryStrategy>,
    recovery_path: Vec<String>,
    initial_config: ExperimentConfig,
    current_config: ExperimentConfig,
}

impl RecoveryEngine {
    pub fn new(config: ExperimentConfig) -> Self {
        Self {
            strategies_attempted: Vec::new(),
            recovery_path: Vec::new(),
            initial_config: config.clone(),
            current_config: config,
        }
    }

    /// Get the current configuration (may have been modified by recovery)
    pub fn current_config(&self) -> &ExperimentConfig {
        &self.current_config
    }

    /// Try the next recovery strategy.
    /// Returns Some(new_config) if a strategy was applied, or None if all strategies exhausted.
    pub fn next_strategy(&mut self) -> Option<ExperimentConfig> {
        // Try strategies in order of least impact first
        let strategies = [
            RecoveryStrategy::KvToRam,
            RecoveryStrategy::ReduceBatch,
            RecoveryStrategy::ReduceContext25,
            RecoveryStrategy::ReduceGpuLayers,
            RecoveryStrategy::KvCacheQ8ToQ4,
            RecoveryStrategy::KvCacheQ4ToTurbo3,
            RecoveryStrategy::AbandonBranch,
        ];

        for strategy in &strategies {
            if self.strategies_attempted.contains(strategy) {
                continue; // Already tried this one
            }

            self.strategies_attempted.push(*strategy);
            self.recovery_path.push(strategy.name().to_string());

            match strategy {
                RecoveryStrategy::KvToRam => {
                    if !self.current_config.kv_in_ram {
                        self.current_config.kv_in_ram = true;
                        return Some(self.current_config.clone());
                    }
                }
                RecoveryStrategy::ReduceBatch => {
                    if self.current_config.batch_size > 64 {
                        self.current_config.batch_size /= 2;
                        return Some(self.current_config.clone());
                    }
                }
                RecoveryStrategy::ReduceContext25 => {
                    let new_ctx = (self.current_config.context_length as f64 * 0.75) as u64;
                    if new_ctx >= 4096 && new_ctx < self.current_config.context_length {
                        self.current_config.context_length = new_ctx;
                        return Some(self.current_config.clone());
                    }
                }
                RecoveryStrategy::ReduceGpuLayers => {
                    if self.current_config.gpu_layers > 10 {
                        self.current_config.gpu_layers = (self.current_config.gpu_layers as f64 * 0.75) as u64;
                        return Some(self.current_config.clone());
                    }
                }
                RecoveryStrategy::KvCacheFp16ToQ8 => {
                    if matches!(self.current_config.kv_cache_type, KvCacheType::Fp16) {
                        self.current_config.kv_cache_type = KvCacheType::Q8;
                        return Some(self.current_config.clone());
                    }
                }
                RecoveryStrategy::KvCacheQ8ToQ4 => {
                    if matches!(self.current_config.kv_cache_type, KvCacheType::Q8) {
                        self.current_config.kv_cache_type = KvCacheType::Q4;
                        return Some(self.current_config.clone());
                    }
                }
                RecoveryStrategy::KvCacheQ4ToTurbo3 => {
                    if matches!(self.current_config.kv_cache_type, KvCacheType::Q4) {
                        self.current_config.kv_cache_type = KvCacheType::Turbo3;
                        return Some(self.current_config.clone());
                    }
                }
                RecoveryStrategy::AbandonBranch => {
                    // Mark as abandoned — caller should stop
                    return None;
                }
            }
        }

        None
    }

    /// Get the full recovery path as a readable string
    pub fn recovery_path_string(&self) -> String {
        if self.recovery_path.is_empty() {
            "No recovery needed".to_string()
        } else {
            self.recovery_path.join(" → ")
        }
    }

    /// Number of recovery attempts made
    pub fn attempts(&self) -> usize {
        self.strategies_attempted.len()
    }

    /// Reset the engine with a new initial config
    pub fn reset(&mut self, config: ExperimentConfig) {
        self.strategies_attempted.clear();
        self.recovery_path.clear();
        self.initial_config = config.clone();
        self.current_config = config;
    }
}
