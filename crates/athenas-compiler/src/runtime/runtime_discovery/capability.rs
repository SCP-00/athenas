use serde::Serialize;
use std::fmt;

// ── RuntimeCapabilities ──
// Describes what a runtime supports.
// Determined by probing binaries, NOT hardcoded.

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeCapabilities {
    // Identity
    pub binary_path: String,
    pub binary_name: String,
    pub display_name: String,
    pub version: Option<String>,

    // Special binaries found alongside this runtime
    pub special_binaries: Vec<String>,

    // Core capabilities
    pub has_server_mode: bool,
    pub has_cli_mode: bool,

    // Hardware acceleration
    pub supports_flash_attention: bool,
    pub supports_cuda: bool,
    pub supports_vulkan: bool,
    pub supports_metal: bool,

    // KV Cache
    pub kv_cache_types: Vec<String>,
    pub supports_kv_cache_quant: bool,
    pub supports_kv_in_ram: bool,

    // Memory architectures (Bonsai, etc.)
    pub supports_bonsai: bool,
    pub supports_iswa: bool,
    pub has_memory_recurrent: bool,
    pub has_memory_hybrid: bool,

    // Model support
    pub supports_embeddings: bool,
    pub supports_vision: bool,
    pub supports_grammar: bool,
    pub supports_tool_calling: bool,
    pub supports_speculative_decoding: bool,
    pub supports_continuous_batching: bool,
    pub supports_rope_scaling: bool,
    pub supports_sliding_window: bool,

    // Probing metadata
    pub probed_at: String,
    pub help_flags_found: Vec<String>,
}

impl RuntimeCapabilities {
    /// Name suitable for display (e.g., "llama.cpp (PrismML v9591)")
    pub fn display_name_short(&self) -> String {
        match &self.version {
            Some(v) => format!("{} v{}", self.display_name, v),
            None => self.display_name.clone(),
        }
    }

    /// Number of capabilities this runtime supports
    pub fn capability_count(&self) -> usize {
        let mut count = 0;
        if self.has_server_mode { count += 1; }
        if self.has_cli_mode { count += 1; }
        if self.supports_flash_attention { count += 1; }
        if self.supports_cuda { count += 1; }
        if self.supports_vulkan { count += 1; }
        if self.supports_metal { count += 1; }
        if self.supports_kv_cache_quant { count += 1; }
        if self.supports_bonsai { count += 1; }
        if self.supports_iswa { count += 1; }
        if self.supports_embeddings { count += 1; }
        if self.supports_vision { count += 1; }
        if self.supports_grammar { count += 1; }
        if self.supports_speculative_decoding { count += 1; }
        if self.supports_continuous_batching { count += 1; }
        if self.supports_rope_scaling { count += 1; }
        if self.supports_sliding_window { count += 1; }
        count
    }

    /// A score that can replace the old hardcoded runtime priority
    /// Based on actual probed capabilities, not assumptions.
    /// Designed to produce a spread from ~0.30 (minimal) to ~0.95 (full-featured).
    pub fn capability_score(&self) -> f64 {
        let mut score = 0.30; // baseline (lower to create more spread)

        // Hardware acceleration (max 0.14)
        if self.supports_flash_attention { score += 0.08; }
        if self.supports_cuda { score += 0.06; }  // NVidia GPU support
        if self.supports_vulkan { score += 0.04; }

        // Memory architectures — these are rare and valuable (max 0.26)
        if self.supports_bonsai { score += 0.12; }
        if self.supports_iswa { score += 0.08; }
        if self.has_memory_recurrent { score += 0.06; }
        if self.has_memory_hybrid { score += 0.06; }

        // Advanced features (max 0.16)
        if self.supports_speculative_decoding { score += 0.06; }
        if self.supports_grammar { score += 0.04; }
        if self.supports_continuous_batching { score += 0.04; }
        if self.supports_rope_scaling { score += 0.04; }
        if self.supports_sliding_window { score += 0.04; }

        // Model support (max 0.12)
        if self.supports_embeddings { score += 0.04; }
        if self.supports_vision { score += 0.06; }
        if self.supports_kv_cache_quant { score += 0.04; }

        // Operating modes (max 0.07)
        if self.has_server_mode { score += 0.04; }
        if self.has_cli_mode { score += 0.03; }

        // Bonus for special companion binaries (max 0.10)
        score += (self.special_binaries.len() as f64).min(5.0) * 0.02;

        // Cap at 0.95 to leave room for runtime-specific specializations
        score.min(0.95)
    }
}

impl fmt::Display for RuntimeCapabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Runtime: {} (Probed)", self.display_name_short())?;
        writeln!(f, "  Binary: {}", self.binary_path)?;
        if let Some(v) = &self.version {
            writeln!(f, "  Version: {}", v)?;
        }
        writeln!(f, "  Capabilities:")?;
        writeln!(f, "    Server mode:        {}", yesno(self.has_server_mode))?;
        writeln!(f, "    CLI mode:           {}", yesno(self.has_cli_mode))?;
        writeln!(f, "    Flash Attention:    {}", yesno(self.supports_flash_attention))?;
        writeln!(f, "    CUDA:               {}", yesno(self.supports_cuda))?;
        writeln!(f, "    KV Cache Quant:     {}", yesno(self.supports_kv_cache_quant))?;
        writeln!(f, "    Bonsai Memory:      {}", yesno(self.supports_bonsai))?;
        writeln!(f, "    ISWA:               {}", yesno(self.supports_iswa))?;
        writeln!(f, "    Embeddings:         {}", yesno(self.supports_embeddings))?;
        writeln!(f, "    Vision:             {}", yesno(self.supports_vision))?;
        writeln!(f, "    Grammar:            {}", yesno(self.supports_grammar))?;
        writeln!(f, "    Speculative:        {}", yesno(self.supports_speculative_decoding))?;
        writeln!(f, "    Rope Scaling:       {}", yesno(self.supports_rope_scaling))?;
        writeln!(f, "    Sliding Window:     {}", yesno(self.supports_sliding_window))?;

        if !self.kv_cache_types.is_empty() {
            writeln!(f, "  KV Cache Types: {:?}", self.kv_cache_types)?;
        }
        if !self.special_binaries.is_empty() {
            writeln!(f, "  Special binaries:")?;
            for b in &self.special_binaries {
                writeln!(f, "    - {}", b)?;
            }
        }
        if !self.help_flags_found.is_empty() {
            writeln!(f, "  Help flags found: {:?}", self.help_flags_found)?;
        }
        writeln!(f, "  Capability score: {:.3}", self.capability_score())?;
        Ok(())
    }
}

fn yesno(v: bool) -> &'static str {
    if v { "✅ yes" } else { "❌ no" }
}

// ── Discovering which KV cache types a runtime supports ──
// Different runtimes have different KV cache quant types available.

pub const KV_CACHE_TYPES_ALL: &[&str] = &[
    "FP16", "Q8", "Q6", "Q5", "Q4", "Q4_K", "Q8_K",
    "Turbo3", "Turbo2", "Turbo1",
    "IQ4", "Bonsai", "ISWA",
];

/// Returns true if the help text suggests a runtime supports a given KV cache type
pub fn help_text_suggests_kv_type(help: &str, kv_type: &str) -> bool {
    let lower = kv_type.to_lowercase();
    let help_lower = help.to_lowercase();
    // Check for cache-type flags
    if help_lower.contains(&format!("cache-type-{}", lower))
        || help_lower.contains(&format!("cache_type_{}", lower))
        || help_lower.contains(&format!("{}-kv", lower))
        || help_lower.contains(kv_type) && help_lower.contains("cache")
    {
        return true;
    }
    false
}
