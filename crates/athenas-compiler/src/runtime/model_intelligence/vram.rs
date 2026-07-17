use serde::Serialize;
use std::collections::HashSet;

// ── KV Cache Type ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
pub enum KvCacheType {
    Fp16,
    Q8,
    Q8K,   // K-quant Q8
    Q6,    // Residual Q6
    Q5,    // Residual Q5
    Q4,    // Residual Q4
    Turbo3,
    Turbo2,
    Turbo1,
    Iq4,
    Bonsai,
    IsWa,  // Importance-based Sparse Window Attention
}

impl KvCacheType {
    pub fn name(&self) -> &str {
        match self {
            KvCacheType::Fp16 => "FP16",
            KvCacheType::Q8 => "Q8",
            KvCacheType::Q8K => "Q8_K",
            KvCacheType::Q6 => "Q6",
            KvCacheType::Q5 => "Q5",
            KvCacheType::Q4 => "Q4",
            KvCacheType::Turbo3 => "Turbo3",
            KvCacheType::Turbo2 => "Turbo2",
            KvCacheType::Turbo1 => "Turbo1",
            KvCacheType::Iq4 => "IQ4",
            KvCacheType::Bonsai => "Bonsai",
            KvCacheType::IsWa => "ISWA",
        }
    }

    /// Bytes per element for this KV cache type
    pub fn bytes_per_element(&self) -> f64 {
        match self {
            KvCacheType::Fp16 => 2.0,
            KvCacheType::Q8 => 1.0,
            KvCacheType::Q8K => 1.0,
            KvCacheType::Q6 => 0.75,
            KvCacheType::Q5 => 0.625,
            KvCacheType::Q4 => 0.5,
            KvCacheType::Turbo3 => 0.5,
            KvCacheType::Turbo2 => 0.375,
            KvCacheType::Turbo1 => 0.25,
            KvCacheType::Iq4 => 0.5,
            KvCacheType::Bonsai => 0.375,
            KvCacheType::IsWa => 0.75,
        }
    }

    /// Quality degradation compared to FP16 (0.0 = none, 1.0 = unusable)
    pub fn quality_loss(&self) -> f64 {
        match self {
            KvCacheType::Fp16 => 0.0,
            KvCacheType::Q8 => 0.02,
            KvCacheType::Q8K => 0.015,
            KvCacheType::Q6 => 0.04,
            KvCacheType::Q5 => 0.07,
            KvCacheType::Q4 => 0.12,
            KvCacheType::Turbo3 => 0.05,
            KvCacheType::Turbo2 => 0.10,
            KvCacheType::Turbo1 => 0.18,
            KvCacheType::Iq4 => 0.08,
            KvCacheType::Bonsai => 0.03,
            KvCacheType::IsWa => 0.01,
        }
    }

    pub fn all() -> Vec<KvCacheType> {
        vec![
            KvCacheType::Fp16,
            KvCacheType::Q8,
            KvCacheType::Q8K,
            KvCacheType::Q6,
            KvCacheType::Q5,
            KvCacheType::Q4,
            KvCacheType::Turbo3,
            KvCacheType::Turbo2,
            KvCacheType::Turbo1,
            KvCacheType::Iq4,
            KvCacheType::Bonsai,
            KvCacheType::IsWa,
        ]
    }
}

// ── Memory Config ──

#[derive(Debug, Clone, Serialize)]
pub struct MemoryConfig {
    pub model_params_b: f64,
    pub quantization_bits: f64,
    pub context_length: u64,
    pub kv_cache_type: KvCacheType,
    pub kv_in_ram: bool,
    pub gpu_layers: u64,
    pub total_layers: u64,
    pub embedding_dim: u64,
    pub num_heads: u64,
    pub num_kv_heads: u64,
    pub batch_size: u64,
    pub ubatch_size: u64,
}

impl MemoryConfig {
    pub fn new(model_params_b: f64) -> Self {
        Self {
            model_params_b,
            quantization_bits: 4.0,
            context_length: 8192,
            kv_cache_type: KvCacheType::Fp16,
            kv_in_ram: false,
            gpu_layers: 0,
            total_layers: 40,
            embedding_dim: 2560,
            num_heads: 32,
            num_kv_heads: 8,
            batch_size: 512,
            ubatch_size: 64,
        }
    }

    pub fn with_quant(mut self, bits: f64) -> Self { self.quantization_bits = bits; self }
    pub fn with_context(mut self, ctx: u64) -> Self { self.context_length = ctx; self }
    pub fn with_kv_type(mut self, kv: KvCacheType) -> Self { self.kv_cache_type = kv; self }
    pub fn with_kv_in_ram(mut self, ram: bool) -> Self { self.kv_in_ram = ram; self }
    pub fn with_layers(mut self, gpu: u64, total: u64) -> Self { self.gpu_layers = gpu; self.total_layers = total; self }
    pub fn with_dims(mut self, embed: u64, heads: u64, kv_heads: u64) -> Self { self.embedding_dim = embed; self.num_heads = heads; self.num_kv_heads = kv_heads; self }
    pub fn with_batch(mut self, batch: u64, ubatch: u64) -> Self { self.batch_size = batch; self.ubatch_size = ubatch; self }
}

// ── Memory Estimate ──

#[derive(Debug, Clone, Serialize)]
pub struct MemoryEstimate {
    pub config: MemoryConfig,

    // VRAM
    pub vram_weights_gb: f64,
    pub vram_kv_cache_gb: f64,
    pub vram_activations_gb: f64,
    pub vram_total_gb: f64,

    // RAM
    pub ram_weights_gb: f64,
    pub ram_kv_cache_gb: f64,
    pub ram_total_gb: f64,

    // Status
    pub fits_in_vram: bool,
    pub oom_risk: f64, // 0.0 = none, 1.0 = certain
    pub estimated_tok_s: f64,
}

impl MemoryEstimate {
    pub fn display(&self, vram_available_gb: f64, ram_available_gb: f64) -> String {
        let mut s = String::new();
        let kv_name = self.config.kv_cache_type.name();
        s.push_str("┌── Memory Analysis ───────────────────────┐\n");

        // VRAM
        s.push_str(&format!("│ VRAM ({:.1} GB available)               │\n", vram_available_gb));
        s.push_str(&format!("│   Weights:    {:>6.2} GB               │\n", self.vram_weights_gb));
        s.push_str(&format!("│   KV Cache:   {:>6.2} GB ({})      │\n",
            self.vram_kv_cache_gb, if self.config.kv_in_ram { "in RAM" } else { kv_name }));
        s.push_str(&format!("│   Activations:{:>6.2} GB               │\n", self.vram_activations_gb));
        s.push_str(&format!("│   Total:      {:>6.2} GB               │\n", self.vram_total_gb));

        let vram_icon = if self.fits_in_vram { "✅" } else { "❌" };
        s.push_str(&format!("│   {} Fits: {}                     │\n", vram_icon,
            if self.fits_in_vram { "Yes" } else { "No — would OOM" }));

        // RAM
        s.push_str(&format!("│ RAM ({:.1} GB available)                │\n", ram_available_gb));
        s.push_str(&format!("│   Weights:    {:>6.2} GB               │\n", self.ram_weights_gb));
        s.push_str(&format!("│   KV Cache:   {:>6.2} GB               │\n", self.ram_kv_cache_gb));
        s.push_str(&format!("│   Total:      {:>6.2} GB               │\n", self.ram_total_gb));

        // KV Cache quality
        s.push_str(&format!("│ KV Cache: {} (quality loss ~{:.1}%)    │\n",
            kv_name,
            self.config.kv_cache_type.quality_loss() * 100.0));

        // OOM risk
        let risk_str = if self.oom_risk < 0.1 { "🟢 Low" }
            else if self.oom_risk < 0.5 { "🟡 Medium" }
            else { "🔴 High" };
        s.push_str(&format!("│ OOM Risk: {} ({:.0}%)             │\n", risk_str, self.oom_risk * 100.0));

        // Speed estimate
        s.push_str(&format!("│ Estimated:  {:.0} tok/s               │\n", self.estimated_tok_s));

        s.push_str("└──────────────────────────────────────────┘\n");
        s
    }
}

// ── VRAM Calculator ──

pub struct VramCalculator;

impl VramCalculator {
    /// Calculate memory usage for a given configuration.
    /// Formula from llama.cpp + empirical measurements.
    pub fn estimate(config: &MemoryConfig, vram_available_gb: f64, ram_available_gb: f64) -> MemoryEstimate {
        // Model weights in VRAM
        // formula: params * bits_per_weight / 8 / 1e9 = GB
        let bits_per_weight = config.quantization_bits;
        let total_weight_bytes = config.model_params_b * 1e9 * bits_per_weight / 8.0;
        let weight_gb = total_weight_bytes / 1e9;
        let gpu_ratio = config.gpu_layers as f64 / config.total_layers.max(1) as f64;
        let vram_weights = weight_gb * gpu_ratio;
        let ram_weights = weight_gb * (1.0 - gpu_ratio);

        // KV Cache: 2 * n_layers * (n_kv_heads * head_dim + n_kv_heads * head_dim) * context * bytes_per_element
        // Simplified: 2 * n_layers * n_kv_heads * embedding_dim / n_heads * 2 * context * bytes_per_element
        // Actually: key_cache + value_cache = 2 * n_layers * n_kv_heads * head_dim * context
        // where head_dim = embedding_dim / n_heads
        let head_dim = config.embedding_dim as f64 / config.num_heads.max(1) as f64;
        let kv_elements = 2.0 * config.total_layers as f64 * config.num_kv_heads as f64 * head_dim * config.context_length as f64;
        let kv_bytes = kv_elements * config.kv_cache_type.bytes_per_element();
        let kv_gb = kv_bytes / 1e9;

        let vram_kv = if config.kv_in_ram { 0.0 } else { kv_gb };
        let ram_kv = if config.kv_in_ram { kv_gb } else { 0.0 };

        // Activations (batch-dependent)
        // Rough estimate: batch * embedding_dim * 4 bytes * 2 (for K and V)
        let act_bytes = config.batch_size as f64 * config.embedding_dim as f64 * 4.0 * 2.0;
        let act_gb = act_bytes / 1e9;
        // Plus workspace for scratch (empirical: ~10% of model size)
        let workspace_gb = vram_weights * 0.1;
        let vram_activations = act_gb + workspace_gb;

        // Totals
        let vram_total = vram_weights + vram_kv + vram_activations;
        let ram_total = ram_weights + ram_kv;

        // Safety margin (fragmentation, runtime buffers)
        let safety_margin = 0.5; // 500 MB safety margin
        let vram_needed = vram_total + safety_margin;
        let fits = vram_needed <= vram_available_gb;

        // OOM risk
        let oom_risk = if fits {
            let headroom = (vram_available_gb - vram_needed) / vram_available_gb;
            if headroom > 0.2 { 0.05 }
            else if headroom > 0.1 { 0.15 }
            else { 0.30 }
        } else {
            let deficit = (vram_needed - vram_available_gb) / vram_needed;
            (deficit * 2.0).min(1.0)
        };

        // Speed estimate (very rough)
        let base_tok_s = match config.model_params_b {
            p if p <= 4.0 => 60.0,
            p if p <= 7.0 => 42.0,
            p if p <= 9.0 => 30.0,
            p if p <= 14.0 => 20.0,
            p if p <= 27.0 => 12.0,
            _ => 8.0,
        };
        let kv_penalty = config.kv_cache_type.quality_loss();
        let context_penalty = (config.context_length as f64 / 131072.0).min(1.0) * 0.2;
        let vram_speedup = gpu_ratio * 0.5;
        let estimated_tok_s = base_tok_s * (1.0 + vram_speedup - kv_penalty - context_penalty);

        MemoryEstimate {
            config: config.clone(),
            vram_weights_gb: round2(vram_weights),
            vram_kv_cache_gb: round2(vram_kv),
            vram_activations_gb: round2(vram_activations),
            vram_total_gb: round2(vram_total),
            ram_weights_gb: round2(ram_weights),
            ram_kv_cache_gb: round2(ram_kv),
            ram_total_gb: round2(ram_total),
            fits_in_vram: fits,
            oom_risk: round2(oom_risk),
            estimated_tok_s: round2(estimated_tok_s),
        }
    }

    /// Try multiple KV cache strategies AND context scales, return best viable ones.
    /// Scales context down (100%, 75%, 50%, 25%) to find configurations that fit.
    pub fn find_best_config(
        model_params_b: f64,
        quantization_bits: f64,
        context_length: u64,
        embedding_dim: u64,
        num_heads: u64,
        num_kv_heads: u64,
        total_layers: u64,
        vram_available_gb: f64,
        ram_available_gb: f64,
    ) -> Vec<MemoryEstimate> {
        let mut results = Vec::new();

        // Context scales to try
        let ctx_scales: [(u64, &str); 4] = [
            (100, "Full"),
            (75, "75%"),
            (50, "50%"),
            (25, "25%"),
        ];

        // Strategies to try for each context scale
        let strategies: Vec<(&str, KvCacheType, bool, u64)> = vec![
            ("FP16 VRAM", KvCacheType::Fp16, false, total_layers),
            ("Q8 VRAM", KvCacheType::Q8, false, total_layers),
            ("Turbo3 RAM", KvCacheType::Turbo3, true, total_layers),
            ("Q4 RAM", KvCacheType::Q4, true, total_layers),
            ("Q4 VRAM", KvCacheType::Q4, false, total_layers),
        ];

        for (scale_pct, scale_name) in &ctx_scales {
            let scaled_ctx = ((context_length as f64 * *scale_pct as f64) / 100.0) as u64;
            let scaled_ctx = scaled_ctx.max(4096).min(context_length); // between 4K and max

            for (strat_name, kv_type, kv_ram, layers) in &strategies {
                let cfg = MemoryConfig::new(model_params_b)
                    .with_quant(quantization_bits)
                    .with_context(scaled_ctx)
                    .with_kv_type(*kv_type)
                    .with_kv_in_ram(*kv_ram)
                    .with_dims(embedding_dim, num_heads, num_kv_heads)
                    .with_layers(*layers, total_layers);
                results.push(Self::estimate(&cfg, vram_available_gb, ram_available_gb));
            }
        }

        // Remove duplicates (same context + same kv type + same kv location)
        results.sort_by(|a, b| {
            // First: fits in VRAM
            let a_fits = a.fits_in_vram as i32;
            let b_fits = b.fits_in_vram as i32;
            b_fits.cmp(&a_fits)
                // Then: lower OOM risk
                .then(a.oom_risk.partial_cmp(&b.oom_risk).unwrap_or(std::cmp::Ordering::Equal))
                // Then: higher speed
                .then(b.estimated_tok_s.partial_cmp(&a.estimated_tok_s).unwrap_or(std::cmp::Ordering::Equal))
                // Then: larger context
                .then(b.config.context_length.cmp(&a.config.context_length))
        });

        // Deduplicate by (context, kv_type, kv_in_ram)
        let mut seen = std::collections::HashSet::new();
        results.retain(|r| {
            let key = (r.config.context_length, r.config.kv_cache_type, r.config.kv_in_ram);
            seen.insert(key)
        });

        // Return top configurations
        results.truncate(12);
        results
    }
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kv_cache_types() {
        assert!(KvCacheType::Fp16.bytes_per_element() > KvCacheType::Q8.bytes_per_element());
        assert!(KvCacheType::Fp16.quality_loss() < KvCacheType::Q4.quality_loss());
        assert_eq!(KvCacheType::all().len(), 12);
    }

    #[test]
    fn test_small_model_fits() {
        // Qwen 4B Q4_K_M, 32K context, FP16 KV
        let cfg = MemoryConfig::new(4.0)
            .with_quant(4.0)
            .with_context(32768)
            .with_kv_type(KvCacheType::Q8)
            .with_dims(2560, 32, 8)
            .with_layers(40, 40);
        let est = VramCalculator::estimate(&cfg, 6.0, 16.0);
        // Should fit in 6GB VRAM
        assert!(est.fits_in_vram, "Qwen 4B should fit in 6GB VRAM");
    }

    #[test]
    fn test_large_model_oom() {
        // Bonsai 27B Q1_0, 256K context, FP16 KV → OOM
        let cfg = MemoryConfig::new(27.0)
            .with_quant(2.0)  // Q1_0 ≈ 2 bits
            .with_context(262144)
            .with_kv_type(KvCacheType::Fp16)
            .with_dims(5120, 40, 10)
            .with_layers(80, 80);
        let est = VramCalculator::estimate(&cfg, 6.0, 16.0);
        // KV cache alone is huge at 256K
        assert!(!est.fits_in_vram || est.vram_kv_cache_gb > 6.0,
            "27B with 256K context in FP16 should need >6GB for KV alone");
    }

    #[test]
    fn test_kv_in_ram_reduces_vram() {
        let cfg_vram = MemoryConfig::new(9.0)
            .with_quant(3.0)
            .with_context(262144)
            .with_kv_type(KvCacheType::Turbo3)
            .with_kv_in_ram(false)
            .with_dims(3584, 32, 8)
            .with_layers(60, 60);
        let cfg_ram = MemoryConfig::new(9.0)
            .with_quant(3.0)
            .with_context(262144)
            .with_kv_type(KvCacheType::Turbo3)
            .with_kv_in_ram(true)
            .with_dims(3584, 32, 8)
            .with_layers(60, 60);

        let est_vram = VramCalculator::estimate(&cfg_vram, 6.0, 16.0);
        let est_ram = VramCalculator::estimate(&cfg_ram, 6.0, 16.0);

        assert!(est_ram.vram_total_gb < est_vram.vram_total_gb,
            "KV in RAM should use less VRAM");
        assert!(est_ram.ram_total_gb > est_vram.ram_total_gb,
            "KV in RAM should use more RAM");
    }

    #[test]
    fn test_find_best_config() {
        let results = VramCalculator::find_best_config(
            9.0, 3.0, 131072,
            3584, 32, 8, 60,
            6.0, 16.0,
        );
        assert!(!results.is_empty());
        // First result should have lowest OOM risk
        assert!(results[0].oom_risk <= results.last().unwrap().oom_risk);
    }
}
