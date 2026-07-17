/// Parameter Normalization — Find the common parameter set across runtimes.
///
/// When comparing runtimes (e.g., Official vs TurboQuant vs Bonsai),
/// they must be compared using the SAME parameters — otherwise the
/// comparison is scientifically invalid.
///
/// This module discovers:
/// 1. Which parameters each runtime supports (from --help)
/// 2. The intersection of all supported parameters
/// 3. The union of parameters for runtime-specific testing
///
/// ## Parameters considered
/// | Parameter | Official | TurboQuant | Bonsai |
/// |-----------|----------|------------|--------|
/// | context   | ✓        | ✓          | ✓      |
/// | batch     | ✓        | ✓          | ✓      |
/// | ubatch    | ✓        | ✓          | ✓      |
/// | ngl       | ✓        | ✓          | ✓      |
/// | threads   | ✓        | ✓          | ✓      |
/// | flash-attn | ✓       | ✓          | ✓      |
/// | cache-type-k | ✓     | ✓          | ✓      |
/// | kv-offload | ✓       | ✓          | ✓      |
/// | cache-type-k turbo3 | ✗ | ✓        | ✗      |
/// | bonsai    | ✗        | ✗          | ✓      |
/// | iswa      | ✗        | ✗          | ✓      |
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

// ═══════════════════════════════════════════════════════════════
// Parameter Definition
// ═══════════════════════════════════════════════════════════════

/// A parameter that can be passed to a runtime.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Parameter {
    /// Parameter name (e.g., "context", "batch", "flash_attention")
    pub name: String,
    /// CLI flag (e.g., "-c", "--flash-attn")
    pub flag: String,
    /// Category of the parameter
    pub category: ParameterCategory,
    /// Description
    pub description: String,
}

/// Category of a parameter
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ParameterCategory {
    /// Memory / context parameters
    Memory,
    /// Performance / throughput parameters
    Performance,
    /// Memory strategy (KV, offload)
    MemoryStrategy,
    /// Feature flags (flash, speculative, etc.)
    Feature,
    /// CPU / thread parameters
    Cpu,
    /// Inference parameters (temperature, etc.)
    Inference,
    /// Runtime-specific features (Bonsai, ISWA, etc.)
    RuntimeSpecific,
}

// ═══════════════════════════════════════════════════════════════
/// A detected parameter set for a single runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeParameterSet {
    /// Runtime display name
    pub runtime_name: String,
    /// Runtime binary path
    pub runtime_path: String,
    /// Set of supported parameters
    pub parameters: HashSet<String>,  // parameter names
    /// All detected flags from --help
    pub detected_flags: Vec<String>,
}

impl RuntimeParameterSet {
    /// Detect supported parameters from a runtime binary.
    /// Uses --help output to determine which flags are available.
    pub fn detect(path: &Path, name: &str) -> Self {
        let help_text = get_help_text(path);
        let help_lower = help_text.to_lowercase();
        let mut parameters = HashSet::new();

        // Standard llama.cpp parameters — check if the runtime supports each
        if help_lower.contains("-c ") || help_lower.contains("--ctx-size ") || help_lower.contains("context-size") {
            parameters.insert("context".to_string());
        }
        if help_lower.contains("-b ") || help_lower.contains("--batch-size ") {
            parameters.insert("batch".to_string());
        }
        if help_lower.contains("-ub ") || help_lower.contains("--ubatch-size ") {
            parameters.insert("ubatch".to_string());
        }
        if help_lower.contains("-ngl ") || help_lower.contains("--n-gpu-layers ") || help_lower.contains("gpu-layers") {
            parameters.insert("gpu_layers".to_string());
        }
        if help_lower.contains("-t ") || help_lower.contains("--threads ") {
            parameters.insert("threads".to_string());
        }
        if help_lower.contains("flash") || help_lower.contains("flash-attn") {
            parameters.insert("flash_attention".to_string());
        }
        if help_lower.contains("cache-type") || help_lower.contains("cache-type-k") {
            parameters.insert("kv_cache_type".to_string());
        }
        if help_lower.contains("no-kv-offload") || help_lower.contains("cache-ram") || help_lower.contains("kv-ram") {
            parameters.insert("kv_in_ram".to_string());
        }
        if help_lower.contains("--temp") || help_lower.contains("temperature") {
            parameters.insert("temperature".to_string());
        }
        if help_lower.contains("--seed") || help_lower.contains("--sampling-seed") {
            parameters.insert("seed".to_string());
        }
        if help_lower.contains("--repeat-penalty") || help_lower.contains("repeat-penalty") {
            parameters.insert("repeat_penalty".to_string());
        }
        if help_lower.contains("--top-k") {
            parameters.insert("top_k".to_string());
        }
        if help_lower.contains("--top-p") {
            parameters.insert("top_p".to_string());
        }
        if help_lower.contains("mlock") || help_lower.contains("--mlock") {
            parameters.insert("mlock".to_string());
        }
        if help_lower.contains("no-mmap") || help_lower.contains("mmap") {
            parameters.insert("mmap".to_string());
        }
        if help_lower.contains("numa") || help_lower.contains("--numa") {
            parameters.insert("numa".to_string());
        }
        // Bonsai-specific
        if help_lower.contains("bonsai") || help_lower.contains("memory-hybrid") || help_lower.contains("memory-recurrent") {
            parameters.insert("bonsai_mode".to_string());
        }
        // ISWA-specific
        if help_lower.contains("iswa") || help_lower.contains("importance-based") {
            parameters.insert("iswa_mode".to_string());
        }
        // TurboQuant-specific
        if help_lower.contains("turbo3") || help_lower.contains("iq4_nl") {
            parameters.insert("turbo3_mode".to_string());
        }

        Self {
            runtime_name: name.to_string(),
            runtime_path: path.to_string_lossy().to_string(),
            parameters,
            detected_flags: extract_flags(&help_text),
        }
    }

    /// Does this runtime support a specific parameter?
    pub fn supports(&self, param: &str) -> bool {
        self.parameters.contains(param)
    }
}

// ═══════════════════════════════════════════════════════════════
/// Normalized parameter set — intersection and union
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedParameterSet {
    /// All runtimes analyzed
    pub runtimes: Vec<RuntimeParameterSet>,
    /// Parameters supported by ALL runtimes (safe for comparison)
    pub common_parameters: Vec<String>,
    /// Parameters supported by ANY runtime
    pub union_parameters: Vec<String>,
    /// Runtime-specific parameters (only one runtime supports them)
    pub runtime_specific: HashMap<String, Vec<String>>,
    /// For each common parameter, the default value to use
    pub common_values: HashMap<String, serde_json::Value>,
    /// Hardware constraints that affect parameters
    pub hardware: serde_json::Value,
}

impl NormalizedParameterSet {
    /// Compute the intersection of parameters across runtimes.
    pub fn compute(runtime_sets: Vec<RuntimeParameterSet>, hardware: serde_json::Value) -> Self {
        // If no runtimes, return empty
        if runtime_sets.is_empty() {
            return Self {
                runtimes: runtime_sets,
                common_parameters: Vec::new(),
                union_parameters: Vec::new(),
                runtime_specific: HashMap::new(),
                common_values: HashMap::new(),
                hardware,
            };
        }

        // Start with all parameters from first runtime
        let mut common: HashSet<String> = runtime_sets[0].parameters.clone();
        let mut union: HashSet<String> = runtime_sets[0].parameters.clone();

        for rt in &runtime_sets[1..] {
            common = common.intersection(&rt.parameters).cloned().collect();
            union = union.union(&rt.parameters).cloned().collect();
        }

        // Compute runtime-specific parameters
        let mut runtime_specific = HashMap::new();
        for rt in &runtime_sets {
            let mut specific = union.difference(&common)
                .filter(|p| rt.parameters.contains(p.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            specific.sort();
            runtime_specific.insert(rt.runtime_name.clone(), specific);
        }

        // Default values for common parameters
        let mut common_values = HashMap::new();
        common_values.insert("context".to_string(), serde_json::json!(32768));
        common_values.insert("batch".to_string(), serde_json::json!(512));
        common_values.insert("ubatch".to_string(), serde_json::json!(256));
        common_values.insert("gpu_layers".to_string(), serde_json::json!(999));
        common_values.insert("threads".to_string(), serde_json::json!(4));
        common_values.insert("flash_attention".to_string(), serde_json::json!(true));
        common_values.insert("temperature".to_string(), serde_json::json!(0.0));  // deterministic
        common_values.insert("seed".to_string(), serde_json::json!(42));

        let mut common_list: Vec<String> = common.into_iter().collect();
        common_list.sort();
        let mut union_list: Vec<String> = union.into_iter().collect();
        union_list.sort();

        Self {
            runtimes: runtime_sets,
            common_parameters: common_list,
            union_parameters: union_list,
            runtime_specific,
            common_values,
            hardware,
        }
    }

    /// Number of runtimes analyzed
    pub fn runtime_count(&self) -> usize {
        self.runtimes.len()
    }

    /// Number of common parameters
    pub fn common_count(&self) -> usize {
        self.common_parameters.len()
    }

    /// True if all runtimes support the same set of core parameters
    pub fn is_comparable(&self) -> bool {
        self.common_count() >= 5 // At least 5 common parameters for a valid comparison
    }
}

// ═══════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════

fn get_help_text(path: &Path) -> String {
    let output = std::process::Command::new(path).arg("--help").output().ok();
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

// ═══════════════════════════════════════════════════════════════
// Default Parameters for Scientific Experiments
// ═══════════════════════════════════════════════════════════════

/// Default set of parameters for a scientific experiment.
/// These are the parameters that all llama.cpp variants should support.
pub fn default_experiment_parameters() -> HashMap<String, serde_json::Value> {
    let mut params = HashMap::new();
    params.insert("context".to_string(), serde_json::json!(32768));
    params.insert("batch".to_string(), serde_json::json!(512));
    params.insert("ubatch".to_string(), serde_json::json!(256));
    params.insert("gpu_layers".to_string(), serde_json::json!(999));
    params.insert("threads".to_string(), serde_json::json!(4));
    params.insert("flash_attention".to_string(), serde_json::json!(true));
    params.insert("temperature".to_string(), serde_json::json!(0.0));
    params.insert("seed".to_string(), serde_json::json!(42));
    params.insert("max_tokens".to_string(), serde_json::json!(100));
    params.insert("prompt".to_string(), serde_json::json!("Hello, explain what you are."));
    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_runtime_self_intersection() {
        let set = RuntimeParameterSet {
            runtime_name: "test".to_string(),
            runtime_path: "/bin/echo".to_string(),
            parameters: ["context", "batch", "threads"].iter().map(|s| s.to_string()).collect(),
            detected_flags: vec![],
        };
        let normalized = NormalizedParameterSet::compute(vec![set], serde_json::json!({}));
        assert_eq!(normalized.common_count(), 3);
    }

    #[test]
    fn test_two_runtimes_intersection() {
        let set1 = RuntimeParameterSet {
            runtime_name: "a".to_string(),
            runtime_path: "/bin/echo".to_string(),
            parameters: ["context", "batch", "threads"].iter().map(|s| s.to_string()).collect(),
            detected_flags: vec![],
        };
        let set2 = RuntimeParameterSet {
            runtime_name: "b".to_string(),
            runtime_path: "/bin/echo".to_string(),
            parameters: ["context", "batch", "flash_attention"].iter().map(|s| s.to_string()).collect(),
            detected_flags: vec![],
        };
        let normalized = NormalizedParameterSet::compute(vec![set1, set2], serde_json::json!({}));
        assert_eq!(normalized.common_parameters, vec!["batch", "context"]);
        assert!(normalized.union_parameters.contains(&"threads".to_string()));
        assert!(normalized.union_parameters.contains(&"flash_attention".to_string()));
    }

    #[test]
    fn test_default_experiment_parameters() {
        let params = default_experiment_parameters();
        assert!(params.contains_key("context"));
        assert!(params.contains_key("temperature"));
        assert_eq!(params["seed"], serde_json::json!(42));
    }
}
