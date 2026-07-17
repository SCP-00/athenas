/// Capability Discovery — Two-level capability detection.
///
/// **DeclaredCapabilities**: What the runtime SAYS it supports (from --help, --version, metadata).
/// **ObservedCapabilities**: What the runtime ACTUALLY does (verified during execution).
///
/// A capability can be "declared: true, observed: false" if --help says it supports
/// Flash Attention but actual execution shows it's not being used.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

// ═══════════════════════════════════════════════════════════════
// DeclaredCapabilities — from --help parsing
// ═══════════════════════════════════════════════════════════════

/// Capabilities declared by the runtime (from --help, --version, binary analysis).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclaredCapabilities {
    /// Flash Attention
    pub flash_attention: bool,
    /// CUDA GPU acceleration
    pub cuda: bool,
    /// Vulkan GPU acceleration
    pub vulkan: bool,
    /// Metal GPU acceleration (Apple)
    pub metal: bool,
    /// KV cache quantization (--cache-type-k)
    pub kv_cache_quant: bool,
    /// KV cache in RAM (--no-kv-offload)
    pub kv_in_ram: bool,
    /// Turbo3 / TurboQuant KV cache
    pub turbo3: bool,
    /// Bonsai memory architecture
    pub bonsai: bool,
    /// ISWA (Importance-based Sliding Window Attention)
    pub iswa: bool,
    /// Recurrent memory
    pub recurrent_memory: bool,
    /// Hybrid memory
    pub hybrid_memory: bool,
    /// Embeddings support
    pub embeddings: bool,
    /// Vision / multimodal support (mmproj)
    pub vision: bool,
    /// Grammar-constrained generation
    pub grammar: bool,
    /// Tool / function calling
    pub tool_calling: bool,
    /// Speculative decoding
    pub speculative_decoding: bool,
    /// Continuous batching
    pub continuous_batching: bool,
    /// Rope scaling (context extension)
    pub rope_scaling: bool,
    /// Sliding window attention
    pub sliding_window: bool,
    /// Server mode (HTTP API)
    pub server_mode: bool,
    /// CLI mode (stdin/stdout)
    pub cli_mode: bool,
    /// RPC support (distributed inference)
    pub rpc: bool,
    /// NUMA awareness
    pub numa: bool,
    /// Mlock (lock memory to avoid swapping)
    pub mlock: bool,
    /// Memory-mapped model loading
    pub mmap: bool,
    /// Speculative decoding with draft model
    pub speculative_draft: bool,
    /// Flash attention v2
    pub flash_attn_v2: bool,
    /// IQ (Integer Quantization) support
    pub iq_quant: bool,
    /// K-quant support
    pub k_quant: bool,
    /// Available KV cache quant types (from --help flags)
    pub kv_cache_types: Vec<String>,
    /// All discovered flags from --help
    pub all_flags: Vec<String>,
    /// Raw help text (first 2000 chars for reference)
    pub help_snippet: String,
}

impl DeclaredCapabilities {
    /// Detect declared capabilities from a binary path.
    /// Uses --help output and binary name analysis.
    pub fn detect(path: &Path) -> Self {
        let help_text = get_help_text(path);
        let help_lower = help_text.to_lowercase();
        let all_flags = extract_flags(&help_text);

        // Detect KV cache types from help
        let known_kv_types = ["f16", "q8_0", "q6_k", "q5_1", "q4_0", "q4_1", "iq4_nl", "q4_k"];
        let kv_cache_types: Vec<String> = known_kv_types.iter()
            .filter(|kt| help_lower.contains(&kt.to_lowercase()))
            .map(|s| s.to_string())
            .collect();

        // Detect KV cache quant more broadly
        let has_kv_quant = help_lower.contains("cache-type-k") || help_lower.contains("cache_type_k")
            || help_lower.contains("cache-type") || help_lower.contains("k-type");

        Self {
            flash_attention: help_lower.contains("flash-attn") || help_lower.contains("flash_attn")
                || help_lower.contains("flash_attention") || help_lower.contains("flash attention"),
            cuda: help_lower.contains("cuda") || help_lower.contains("n-gpu-layers") || help_lower.contains("ngl") || help_lower.contains("gpu-layers"),
            vulkan: help_lower.contains("vulkan"),
            metal: help_lower.contains("metal"),
            kv_cache_quant: has_kv_quant || !kv_cache_types.is_empty(),
            kv_in_ram: help_lower.contains("no-kv-offload") || help_lower.contains("cache-ram") || help_lower.contains("kv-ram"),
            turbo3: help_lower.contains("turbo3") || help_lower.contains("turbo_quant") || help_lower.contains("iq4_nl"),
            bonsai: help_lower.contains("bonsai") || help_lower.contains("memory-hybrid") || help_lower.contains("memory-recurrent") || help_lower.contains("hierarchical"),
            iswa: help_lower.contains("iswa") || help_lower.contains("importance-based"),
            recurrent_memory: help_lower.contains("memory-recurrent") || help_lower.contains("recurrent_memory"),
            hybrid_memory: help_lower.contains("memory-hybrid") || help_lower.contains("hybrid_memory"),
            embeddings: help_lower.contains("embed") || help_lower.contains("--embd") || help_lower.contains("embedding"),
            vision: help_lower.contains("mmproj") || help_lower.contains("multimodal") || help_lower.contains("llava") || help_lower.contains("qwen2vl") || help_lower.contains("vision"),
            grammar: help_lower.contains("grammar"),
            tool_calling: help_lower.contains("tool") || help_lower.contains("function-call") || help_lower.contains("function_call"),
            speculative_decoding: help_lower.contains("speculative") || help_lower.contains("draft") || help_lower.contains("--draft-model"),
            continuous_batching: help_lower.contains("continuous") || help_lower.contains("cont-batch"),
            rope_scaling: help_lower.contains("rope") || help_lower.contains("context-shift") || help_lower.contains("rope-freq"),
            sliding_window: help_lower.contains("sliding") || help_lower.contains("window-attention"),
            server_mode: help_lower.contains("--port") || help_lower.contains("server"),
            cli_mode: help_lower.contains("--prompt") || help_lower.contains("--n-predict") || help_lower.contains("--temp"),
            rpc: help_lower.contains("rpc") || help_lower.contains("--rpc"),
            numa: help_lower.contains("numa") || help_lower.contains("--numa"),
            mlock: help_lower.contains("mlock") || help_lower.contains("--mlock"),
            mmap: help_lower.contains("mmap") || help_lower.contains("--no-mmap"),
            speculative_draft: help_lower.contains("draft") && help_lower.contains("model"),
            flash_attn_v2: help_lower.contains("flash-attn") || help_lower.contains("flash_attn"),
            iq_quant: help_lower.contains("iq") || help_lower.contains("iq1") || help_lower.contains("iq2") || help_lower.contains("iq3") || help_lower.contains("iq4"),
            k_quant: help_lower.contains("q4_k") || help_lower.contains("q5_k") || help_lower.contains("q6_k") || help_lower.contains("q8_0"),
            kv_cache_types,
            all_flags,
            help_snippet: help_text.chars().take(2000).collect(),
        }
    }

    /// Count how many capabilities are declared
    pub fn count(&self) -> usize {
        let mut n = 0;
        if self.flash_attention { n += 1; }
        if self.cuda { n += 1; }
        if self.vulkan { n += 1; }
        if self.metal { n += 1; }
        if self.kv_cache_quant { n += 1; }
        if self.kv_in_ram { n += 1; }
        if self.turbo3 { n += 1; }
        if self.bonsai { n += 1; }
        if self.iswa { n += 1; }
        if self.recurrent_memory { n += 1; }
        if self.hybrid_memory { n += 1; }
        if self.embeddings { n += 1; }
        if self.vision { n += 1; }
        if self.grammar { n += 1; }
        if self.tool_calling { n += 1; }
        if self.speculative_decoding { n += 1; }
        if self.continuous_batching { n += 1; }
        if self.rope_scaling { n += 1; }
        if self.sliding_window { n += 1; }
        if self.server_mode { n += 1; }
        if self.cli_mode { n += 1; }
        if self.rpc { n += 1; }
        if self.numa { n += 1; }
        if self.mlock { n += 1; }
        if self.mmap { n += 1; }
        n
    }
}

// ═══════════════════════════════════════════════════════════════
// ObservedCapabilities — verified during actual execution
// ═══════════════════════════════════════════════════════════════

/// Capabilities actually observed during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservedCapabilities {
    /// Flash Attention was actually used (check stderr logs for flash attn messages)
    pub flash_attention_used: Option<bool>,
    /// Speculative decoding was active
    pub speculative_active: Option<bool>,
    /// KV cache was quantized (stderr reports quantization type)
    pub kv_quantized: Option<bool>,
    /// Turbo3 was actually active
    pub turbo3_active: Option<bool>,
    /// Bonsai memory was actually used
    pub bonsai_active: Option<bool>,
    /// ISWA was actually used
    pub iswa_active: Option<bool>,
    /// Embeddings endpoint works
    pub embeddings_working: Option<bool>,
    /// Completion endpoint works
    pub completion_working: Option<bool>,
    /// Health endpoint works
    pub health_working: Option<bool>,
    /// Server started successfully
    pub server_started: bool,
    /// Model loaded successfully
    pub model_loaded: bool,
    /// Generated at least one valid token
    pub generated_tokens: bool,
}

impl ObservedCapabilities {
    pub fn new() -> Self {
        Self {
            flash_attention_used: None,
            speculative_active: None,
            kv_quantized: None,
            turbo3_active: None,
            bonsai_active: None,
            iswa_active: None,
            embeddings_working: None,
            completion_working: None,
            health_working: None,
            server_started: false,
            model_loaded: false,
            generated_tokens: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Full Capability Report
// ═══════════════════════════════════════════════════════════════

/// Complete capability report combining declared and observed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityReport {
    /// Runtime binary path
    pub runtime_path: String,
    /// Display name
    pub display_name: String,
    /// Declared capabilities (from --help)
    pub declared: DeclaredCapabilities,
    /// Observed capabilities (from execution), None if not yet executed
    pub observed: Option<ObservedCapabilities>,
    /// Discrepancies between declared and observed
    pub discrepancies: Vec<String>,
}

impl CapabilityReport {
    /// Detect declared capabilities for a runtime
    pub fn from_declared(runtime_path: &Path, display_name: &str) -> Self {
        let declared = DeclaredCapabilities::detect(runtime_path);
        Self {
            runtime_path: runtime_path.to_string_lossy().to_string(),
            display_name: display_name.to_string(),
            declared,
            observed: None,
            discrepancies: Vec::new(),
        }
    }

    /// Merge observed capabilities and compute discrepancies
    pub fn with_observed(mut self, observed: ObservedCapabilities) -> Self {
        let mut discrepancies = Vec::new();

        // Flash Attention
        if self.declared.flash_attention && observed.flash_attention_used == Some(false) {
            discrepancies.push("Flash Attention: declared but not used during execution".to_string());
        }

        // Speculative
        if self.declared.speculative_decoding && observed.speculative_active == Some(false) {
            discrepancies.push("Speculative decoding: declared but not active".to_string());
        }

        // Turbo3
        if self.declared.turbo3 && observed.turbo3_active == Some(false) {
            discrepancies.push("Turbo3: declared but not active".to_string());
        }

        // Bonsai
        if self.declared.bonsai && observed.bonsai_active == Some(false) {
            discrepancies.push("Bonsai: declared but not active".to_string());
        }

        // Server didn't start
        if !observed.server_started {
            discrepancies.push("Server failed to start".to_string());
        }

        // Model didn't load
        if !observed.model_loaded {
            discrepancies.push("Model failed to load".to_string());
        }

        // No tokens generated
        if !observed.generated_tokens {
            discrepancies.push("No tokens were generated".to_string());
        }

        self.observed = Some(observed);
        self.discrepancies = discrepancies;
        self
    }
}

// ═══════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════

fn get_help_text(path: &Path) -> String {
    let output = Command::new(path).arg("--help").output().ok();
    match output {
        Some(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            format!("{}\n{}", stdout, stderr)
        }
        None => String::new(),
    }
}

fn extract_flags(help: &str) -> Vec<String> {
    let mut flags = Vec::new();
    for line in help.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('-') && !trimmed.starts_with("---") {
            let flag = trimmed.split_whitespace().next()
                .unwrap_or(trimmed)
                .trim_end_matches(|c: char| c == ',' || c == '=' || c == ' ')
                .to_string();
            if !flag.is_empty() && flag.len() > 2 {
                flags.push(flag);
            }
        }
    }
    flags.truncate(100);
    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declared_capabilities_defaults() {
        let caps = DeclaredCapabilities::detect(Path::new("/bin/echo"));
        // /bin/echo is not a runtime, should have few capabilities
        assert!(!caps.cuda);
    }

    #[test]
    fn test_observed_capabilities_new() {
        let obs = ObservedCapabilities::new();
        assert!(!obs.server_started);
        assert!(!obs.model_loaded);
    }

    #[test]
    fn test_capability_report_discrepancy_empty() {
        let report = CapabilityReport::from_declared(Path::new("/bin/echo"), "echo");
        assert!(report.discrepancies.is_empty());
    }
}
