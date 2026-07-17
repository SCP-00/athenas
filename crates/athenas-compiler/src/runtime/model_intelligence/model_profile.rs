use std::path::{Path, PathBuf};

use super::gguf::{read_gguf_metadata, GgufMetadata};
use super::vram::{KvCacheType, MemoryConfig, MemoryEstimate, VramCalculator};

/// Complete profile of a model combining GGUF metadata + memory analysis
#[derive(serde::Serialize)]
pub struct ModelProfile {
    pub file_path: PathBuf,
    pub metadata: GgufMetadata,
    pub hw_gpu_vram_gb: f64,
    pub hw_ram_gb: f64,
    pub configs: Vec<MemoryEstimate>,
}

impl ModelProfile {
    /// Full analysis of a model file against current hardware
    pub fn analyze(path: &Path) -> Result<Self, String> {
        let metadata = read_gguf_metadata(path)?;

        // Detect hardware
        let hw = crate::runtime::hardware::detect_hardware();
        let gpu_vram = hw.gpu.first().map(|g| g.vram_gb).unwrap_or(0.0);
        let ram_gb = hw.memory.available_gb;

        // Determine model characteristics
        let params_b = infer_params_from_metadata(&metadata);
        let quant_bits = infer_quant_bits(&metadata);
        let ctx = metadata.context_length.unwrap_or(8192);
        let embed = metadata.embedding_length.unwrap_or(2560);
        let heads = metadata.head_count.unwrap_or(32);
        let kv_heads = metadata.head_count_kv.unwrap_or(heads);
        let layers = metadata.block_count.unwrap_or(40);

        // Try multiple context sizes and KV cache strategies
        let mut configs = Vec::new();

        // For maximum context: try the declared context with KV in RAM (Turbo3)
        let mut results = VramCalculator::find_best_config(
            params_b, quant_bits, ctx,
            embed, heads, kv_heads, layers,
            gpu_vram, ram_gb,
        );
        configs.append(&mut results);

        // For practical context (50% of max): try VRAM+FP16
        let practical_ctx = (ctx / 2).max(8192);
        let cfg = MemoryConfig::new(params_b)
            .with_quant(quant_bits)
            .with_context(practical_ctx)
            .with_kv_type(KvCacheType::Q8)
            .with_kv_in_ram(false)
            .with_dims(embed, heads, kv_heads)
            .with_layers(layers.min(99), layers);
        configs.push(VramCalculator::estimate(&cfg, gpu_vram, ram_gb));

        // For chat context (8K): all in VRAM, FP16
        let cfg = MemoryConfig::new(params_b)
            .with_quant(quant_bits)
            .with_context(8192)
            .with_kv_type(KvCacheType::Fp16)
            .with_kv_in_ram(false)
            .with_dims(embed, heads, kv_heads)
            .with_layers(layers.min(99), layers);
        configs.push(VramCalculator::estimate(&cfg, gpu_vram, ram_gb));

        // Deduplicate and sort by OOM risk
        configs.sort_by(|a, b| {
            a.oom_risk.partial_cmp(&b.oom_risk).unwrap_or(std::cmp::Ordering::Equal)
                .then(b.estimated_tok_s.partial_cmp(&a.estimated_tok_s).unwrap_or(std::cmp::Ordering::Equal))
        });

        Ok(Self {
            file_path: path.to_path_buf(),
            metadata,
            hw_gpu_vram_gb: gpu_vram,
            hw_ram_gb: ram_gb,
            configs,
        })
    }

    pub fn display(&self) -> String {
        let mut s = String::new();

        // Header
        s.push_str(&format!(
            "╔══════════════════════════════════════════╗\n\
             ║     Athena Model Intelligence             ║\n\
             ╚══════════════════════════════════════════╝\n\n"
        ));

        // GGUF Metadata
        s.push_str(&self.metadata.display());

        // Hardware
        s.push_str(&format!("🖥️  Hardware Context\n"));
        s.push_str(&format!("{}\n", "─".repeat(40)));
        let gpu_name = crate::runtime::hardware::detect_hardware()
            .gpu.first()
            .map(|g| g.model.clone())
            .unwrap_or_else(|| "No GPU".to_string());
        s.push_str(&format!("  GPU: {} ({:.1} GB VRAM)\n", gpu_name, self.hw_gpu_vram_gb));
        s.push_str(&format!("  RAM: {:.1} GB available\n", self.hw_ram_gb));
        s.push('\n');

        // Memory Analysis (configs)
        s.push_str(&format!("📊 Configurations\n"));
        s.push_str(&format!("{}\n", "─".repeat(40)));
        for (i, config) in self.configs.iter().enumerate() {
            let label = match i {
                0 => "✅ Recommended",
                1 => "⚡ Faster",
                2 => "🎯 Maximum Context",
                _ => "🔧 Alternative",
            };
            s.push_str(&format!("{} — Configuration {}\n", label, i + 1));
            s.push_str(&config.display(self.hw_gpu_vram_gb, self.hw_ram_gb));
            s.push('\n');
        }

        // Summary
        let best = &self.configs[0];
        s.push_str(&format!("🏆 Best Configuration\n"));
        s.push_str(&format!("{}\n", "─".repeat(40)));
        s.push_str(&format!(
            "  Context: {} ({}K tokens)\n",
            best.config.context_length,
            best.config.context_length / 1024
        ));
        s.push_str(&format!(
            "  KV Cache: {} ({})\n",
            best.config.kv_cache_type.name(),
            if best.config.kv_in_ram { "RAM" } else { "VRAM" }
        ));
        s.push_str(&format!("  Estimated: {:.0} tok/s\n", best.estimated_tok_s));
        s.push_str(&format!("  OOM Risk:  {:.0}%\n", best.oom_risk * 100.0));
        if !best.fits_in_vram {
            s.push_str(&format!(
                "  ⚠ This model may not fit in VRAM. Consider:\n\
                  - Reducing context to {}K\n  \
                  - Using KV cache in RAM ({})\n  \
                  - Using fewer GPU layers (-ngl)\n",
                (self.configs.iter()
                    .find(|c| c.fits_in_vram)
                    .map(|c| c.config.context_length / 1024)
                    .unwrap_or(8)),
                self.configs.iter()
                    .find(|c| c.config.kv_in_ram)
                    .map(|c| c.config.kv_cache_type.name())
                    .unwrap_or("Turbo3")
            ));
        }
        s.push('\n');
        s.push_str("✅ Analysis complete.\n");

        s
    }
}

fn infer_params_from_metadata(meta: &GgufMetadata) -> f64 {
    // Try to infer from block_count * embedding_length * head_count
    if let (Some(blocks), Some(embed)) = (meta.block_count, meta.embedding_length) {
        let estimated = (blocks as f64 * embed as f64 * 4.0) / 1e9; // rough
        if estimated > 0.5 {
            return (estimated * 2.0).round() / 2.0;
        }
    }
    // Fallback: infer from filename
    let lower = meta.file_path.to_lowercase();
    if lower.contains("27b") { 27.0 }
    else if lower.contains("9b") { 9.0 }
    else if lower.contains("7b") { 7.0 }
    else if lower.contains("4b") { 4.0 }
    else if lower.contains("8b") { 8.0 }
    else if lower.contains("3b") { 3.0 }
    else if lower.contains("1b") { 1.0 }
    else { 7.0 }
}

fn infer_quant_bits(meta: &GgufMetadata) -> f64 {
    if let Some(q) = &meta.quantization {
        let lower = q.to_lowercase();
        if lower.contains("q8") { 8.0 }
        else if lower.contains("q6") { 6.0 }
        else if lower.contains("q5") { 5.0 }
        else if lower.contains("q4") { 4.0 }
        else if lower.contains("q3") || lower.contains("iq3") { 3.0 }
        else if lower.contains("q2") || lower.contains("iq2") { 2.0 }
        else if lower.contains("q1") || lower.contains("iq1") { 2.0 }
        else if lower.contains("q0") { 1.0 }
        else { 4.0 } // default
    } else {
        // Infer from file type
        if let Some(ft) = meta.file_type {
            match ft {
                2 | 3 => 4.0,  // Q4_0, Q4_1
                6 | 7 => 5.0,  // Q5_0, Q5_1
                8 | 9 => 8.0,  // Q8_0, Q8_1
                10..=19 => 4.0, // K-quants
                20 | 21 => 2.0, // IQ2
                22..=24 => 3.0, // IQ3
                25 | 26 => 1.0, // IQ1
                27 | 28 => 4.0, // IQ4
                _ => 4.0,
            }
        } else {
            4.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_nonexistent() {
        let result = ModelProfile::analyze(Path::new("/nonexistent.gguf"));
        assert!(result.is_err());
    }

    #[test]
    fn test_infer_params() {
        let meta = GgufMetadata {
            file_path: "qwen3.5-9b-q4_k_m.gguf".to_string(),
            file_size_bytes: 0,
            gguf_version: 3,
            tensor_count: 0,
            metadata_kv_count: 0,
            architecture: None,
            name: None,
            description: None,
            file_type: None,
            context_length: None,
            embedding_length: None,
            block_count: None,
            head_count: None,
            head_count_kv: None,
            feed_forward_length: None,
            chat_template: None,
            tokenizer_model: None,
            training_context: None,
            quantization: None,
            raw_metadata: vec![],
        };
        // Should infer 9B from filename
        let params = infer_params_from_metadata(&meta);
        assert_eq!(params, 9.0);
    }
}
