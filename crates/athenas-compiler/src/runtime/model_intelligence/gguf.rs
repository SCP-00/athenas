use serde::Serialize;
use std::path::Path;

// ── GGUF Header ──
// Specification: https://github.com/ggml-org/ggml/blob/master/docs/gguf.md

const GGUF_MAGIC: &[u8; 4] = b"GGUF";

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct GgufMetadata {
    // File info
    pub file_path: String,
    pub file_size_bytes: u64,
    pub gguf_version: u32,
    pub tensor_count: u64,
    pub metadata_kv_count: u64,

    // Standard keys
    pub architecture: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub file_type: Option<u32>,

    // Model architecture
    pub context_length: Option<u64>,
    pub embedding_length: Option<u64>,
    pub block_count: Option<u64>,
    pub head_count: Option<u64>,
    pub head_count_kv: Option<u64>,
    pub feed_forward_length: Option<u64>,

    // Quantization
    pub quantization: Option<String>,

    // Tokenizer
    pub chat_template: Option<String>,
    pub tokenizer_model: Option<String>,

    // Training
    pub training_context: Option<u64>,

    // Raw metadata (all extracted key-value pairs)
    pub raw_metadata: Vec<(String, String)>,
}

impl GgufMetadata {
    pub fn display(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("╔══════════════════════════════════════════╗\n"));
        s.push_str(&format!("║     Model Intelligence — GGUF Reader     ║\n"));
        s.push_str(&format!("╚══════════════════════════════════════════╝\n\n"));

        // File info
        let path = Path::new(&self.file_path);
        let fname = path.file_name().map(|f| f.to_string_lossy()).unwrap_or(std::borrow::Cow::Borrowed(&self.file_path));
        let size_mb = self.file_size_bytes as f64 / (1024.0 * 1024.0);
        let size_gb = size_mb / 1024.0;
        let size_str = if size_gb >= 1.0 {
            format!("{:.1} GB", size_gb)
        } else {
            format!("{:.0} MB", size_mb)
        };

        s.push_str(&format!("📄 File:       {}\n", fname));
        s.push_str(&format!("📏 Size:       {}\n", size_str));
        s.push_str(&format!("📋 GGUF v{}     Tensors: {}  Metadata: {}\n",
            self.gguf_version, self.tensor_count, self.metadata_kv_count));
        s.push('\n');

        // Architecture
        s.push_str(&format!("🧠 Architecture\n"));
        s.push_str(&format!("{}\n", "─".repeat(40)));
        if let Some(arch) = &self.architecture {
            s.push_str(&format!("  Type:    {}\n", arch));
        }
        if let Some(name) = &self.name {
            s.push_str(&format!("  Name:    {}\n", name));
        }
        if let Some(ctx) = self.context_length.or(self.training_context) {
            s.push_str(&format!("  Context: {} ({}K tokens)\n", ctx, ctx / 1024));
        }
        if let Some(emb) = self.embedding_length {
            s.push_str(&format!("  Embed:   {}\n", emb));
        }
        if let Some(blocks) = self.block_count {
            s.push_str(&format!("  Layers:  {}\n", blocks));
        }
        if let Some(heads) = self.head_count {
            s.push_str(&format!("  Heads:   {} (KV: {})\n", heads,
                self.head_count_kv.unwrap_or(heads)));
        }
        if let Some(ffn) = self.feed_forward_length {
            s.push_str(&format!("  FFN:     {}\n", ffn));
        }
        if let Some(ft) = self.file_type {
            s.push_str(&format!("  Type:    {}\n", file_type_name(ft)));
        }
        if let Some(q) = &self.quantization {
            s.push_str(&format!("  Quant:   {}\n", q));
        }
        s.push('\n');

        // Tokenizer
        if self.chat_template.is_some() || self.tokenizer_model.is_some() {
            s.push_str(&format!("🔤 Tokenizer\n"));
            s.push_str(&format!("{}\n", "─".repeat(40)));
            if let Some(tm) = &self.tokenizer_model {
                s.push_str(&format!("  Model: {}\n", tm));
            }
            if let Some(ct) = &self.chat_template {
                let preview: String = ct.chars().take(120).collect();
                s.push_str(&format!("  Chat:  {}...\n", preview));
            }
            s.push('\n');
        }

        // Raw metadata (top extra items)
        let extra: Vec<_> = self.raw_metadata.iter()
            .filter(|(k, _)| {
                !k.starts_with("general.") && !k.starts_with("llama.")
                    && !k.starts_with("tokenizer.")
            })
            .collect();
        if !extra.is_empty() {
            s.push_str(&format!("📎 Extra Metadata ({})\n", extra.len()));
            s.push_str(&format!("{}\n", "─".repeat(40)));
            for (k, v) in extra.iter().take(10) {
                let val_preview: String = v.chars().take(80).collect();
                s.push_str(&format!("  {} = {}\n", k, val_preview));
            }
            if extra.len() > 10 {
                s.push_str(&format!("  ... and {} more\n", extra.len() - 10));
            }
            s.push('\n');
        }

        s
    }
}

// ── Architecture Database ──
// When a GGUF header doesn't include metadata like context_length, block_count, etc.
// (common with newer llama.cpp conversions), we infer them from the architecture + file name.

#[derive(Debug, Clone, Serialize)]
pub struct ArchitectureInfo {
    pub context_length: u64,
    pub embedding_length: u64,
    pub block_count: u64,
    pub head_count: u64,
    pub head_count_kv: u64,
    pub feed_forward_length: u64,
}

/// Known model architectures with verified dimensions.
/// Used as fallback when GGUF header lacks metadata keys.
/// Keys are "arch_name:param_count" e.g. "Qwen2:4B"
pub fn lookup_architecture(arch: &str, params_b: f64) -> Option<ArchitectureInfo> {
    match arch {
        "Qwen2" | "6" => match params_b as u64 {
            1 => Some(ArchitectureInfo { context_length: 32768, embedding_length: 2048, block_count: 24, head_count: 16, head_count_kv: 16, feed_forward_length: 5504 }),
            4 => Some(ArchitectureInfo { context_length: 32768, embedding_length: 2560, block_count: 40, head_count: 32, head_count_kv: 8, feed_forward_length: 10240 }),
            7 => Some(ArchitectureInfo { context_length: 32768, embedding_length: 4096, block_count: 28, head_count: 32, head_count_kv: 32, feed_forward_length: 11008 }),
            9 => Some(ArchitectureInfo { context_length: 32768, embedding_length: 4096, block_count: 48, head_count: 32, head_count_kv: 8, feed_forward_length: 11008 }),
            14 => Some(ArchitectureInfo { context_length: 32768, embedding_length: 5120, block_count: 48, head_count: 40, head_count_kv: 10, feed_forward_length: 13696 }),
            32 => Some(ArchitectureInfo { context_length: 32768, embedding_length: 5120, block_count: 64, head_count: 40, head_count_kv: 10, feed_forward_length: 13696 }),
            72 => Some(ArchitectureInfo { context_length: 32768, embedding_length: 8192, block_count: 80, head_count: 64, head_count_kv: 16, feed_forward_length: 22016 }),
            _ => None,
        },
        "llama" | "2" => match params_b as u64 {
            1 => Some(ArchitectureInfo { context_length: 2048, embedding_length: 2048, block_count: 22, head_count: 16, head_count_kv: 16, feed_forward_length: 5504 }),
            3 => Some(ArchitectureInfo { context_length: 8192, embedding_length: 3200, block_count: 26, head_count: 32, head_count_kv: 32, feed_forward_length: 8640 }),
            7 => Some(ArchitectureInfo { context_length: 8192, embedding_length: 4096, block_count: 32, head_count: 32, head_count_kv: 32, feed_forward_length: 11008 }),
            8 => Some(ArchitectureInfo { context_length: 8192, embedding_length: 4096, block_count: 32, head_count: 32, head_count_kv: 32, feed_forward_length: 11008 }),
            13 => Some(ArchitectureInfo { context_length: 8192, embedding_length: 5120, block_count: 40, head_count: 40, head_count_kv: 40, feed_forward_length: 13824 }),
            70 => Some(ArchitectureInfo { context_length: 8192, embedding_length: 8192, block_count: 80, head_count: 64, head_count_kv: 8, feed_forward_length: 22016 }),
            _ => None,
        },
        // Bonsai: ternary models, dimensions depend on specific variant
        "Bonsai" | "ternary" => None, // Unknown — must be probed at runtime
        _ => None,
    }
}

/// Infer parameter count from filename
pub fn infer_params_from_filename(fname: &str) -> f64 {
    let fname = fname.to_lowercase();
    if fname.contains("70b") || fname.contains("72b") { 72.0 }
    else if fname.contains("27b") { 27.0 }
    else if fname.contains("14b") { 14.0 }
    else if fname.contains("13b") { 13.0 }
    else if fname.contains("9b") { 9.0 }
    else if fname.contains("8b") { 8.0 }
    else if fname.contains("7b") { 7.0 }
    else if fname.contains("4b") { 4.0 }
    else if fname.contains("3b") { 3.0 }
    else if fname.contains("1.5b") { 1.5 }
    else if fname.contains("1b") { 1.0 }
    else if fname.contains("0.5b") { 0.5 }
    else { 0.0 }
}

/// Infer quantization from filename
pub fn infer_quant_from_filename(fname: &str) -> Option<String> {
    let fname = fname.to_lowercase();
    if fname.contains("q4_k_m") { Some("Q4_K_M".to_string()) }
    else if fname.contains("q4_0") { Some("Q4_0".to_string()) }
    else if fname.contains("q4_1") { Some("Q4_1".to_string()) }
    else if fname.contains("q8_0") { Some("Q8_0".to_string()) }
    else if fname.contains("q2_k") { Some("Q2_K".to_string()) }
    else if fname.contains("q3_k") { Some("Q3_K".to_string()) }
    else if fname.contains("q5_k") { Some("Q5_K".to_string()) }
    else if fname.contains("q6_k") { Some("Q6_K".to_string()) }
    else if fname.contains("iq3_xxs") { Some("IQ3_XXS".to_string()) }
    else if fname.contains("iq3_s") { Some("IQ3_S".to_string()) }
    else if fname.contains("iq3_m") { Some("IQ3_M".to_string()) }
    else if fname.contains("iq4") { Some("IQ4".to_string()) }
    else if fname.contains("iq2") { Some("IQ2".to_string()) }
    else if fname.contains("iq1") { Some("IQ1".to_string()) }
    else if fname.contains("q1_0") { Some("Q1_0".to_string()) }
    else { None }
}

fn file_type_name(ft: u32) -> &'static str {
    match ft {
        0 => "ALL_F32",
        1 => "MOSTLY_F16",
        2 => "MOSTLY_Q4_0",
        3 => "MOSTLY_Q4_1",
        6 => "MOSTLY_Q5_0",
        7 => "MOSTLY_Q5_1",
        8 => "MOSTLY_Q8_0",
        9 => "MOSTLY_Q8_1",
        10 => "MOSTLY_Q2_K",
        11 => "MOSTLY_Q3_K_S",
        12 => "MOSTLY_Q3_K_M",
        13 => "MOSTLY_Q3_K_L",
        14 => "MOSTLY_Q4_K_S",
        15 => "MOSTLY_Q4_K_M",
        16 => "MOSTLY_Q5_K_S",
        17 => "MOSTLY_Q5_K_M",
        18 => "MOSTLY_Q6_K",
        19 => "MOSTLY_Q8_K",
        20 => "MOSTLY_IQ2_XXS",
        21 => "MOSTLY_IQ2_XS",
        22 => "MOSTLY_IQ3_XXS",
        23 => "MOSTLY_IQ3_S",
        24 => "MOSTLY_IQ3_M",
        25 => "MOSTLY_IQ1_S",
        26 => "MOSTLY_IQ1_M",
        27 => "MOSTLY_IQ4_NL",
        28 => "MOSTLY_IQ4_XS",
        29 => "MOSTLY_Q4_K_M_O",
        _ => "UNKNOWN",
    }
}

/// Parse a GGUF file and extract all metadata.
/// Completely deterministic — no LLM, no external dependencies.
pub fn read_gguf_metadata(path: &Path) -> Result<GgufMetadata, String> {
    let data = std::fs::read(path).map_err(|e| format!("Cannot read file: {e}"))?;
    let file_size = data.len() as u64;

    if data.len() < 16 {
        return Err("File too small to be a valid GGUF".to_string());
    }

    // Magic bytes
    if &data[0..4] != GGUF_MAGIC {
        return Err("Not a valid GGUF file (wrong magic)".to_string());
    }

    // Version (uint32 LE)
    let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);

    // Tensor count (uint64 LE)
    let tensor_count = u64::from_le_bytes([
        data[8], data[9], data[10], data[11],
        data[12], data[13], data[14], data[15],
    ]);

    // Metadata KV count (uint64 LE)
    let metadata_kv_count = u64::from_le_bytes([
        data[16], data[17], data[18], data[19],
        data[20], data[21], data[22], data[23],
    ]);

    let mut pos = 24usize;
    let mut raw_metadata = Vec::new();
    let mut architecture = None;
    let mut name = None;
    let mut description = None;
    let mut file_type = None;
    let mut context_length = None;
    let mut embedding_length = None;
    let mut block_count = None;
    let mut head_count = None;
    let mut head_count_kv = None;
    let mut feed_forward_length = None;
    let mut chat_template = None;
    let mut tokenizer_model = None;
    let mut training_context = None;
    let mut quantization = None;

    for _ in 0..metadata_kv_count {
        if pos + 8 > data.len() { break; }

        // Key string length (uint64 LE)
        let key_len = u64::from_le_bytes([
            data[pos], data[pos+1], data[pos+2], data[pos+3],
            data[pos+4], data[pos+5], data[pos+6], data[pos+7],
        ]) as usize;
        pos += 8;

        if pos + key_len > data.len() { break; }
        let key = String::from_utf8_lossy(&data[pos..pos + key_len]).to_string();
        pos += key_len;

        if pos + 4 > data.len() { break; }

        // Value type (uint32 LE)
        let val_type = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]);
        pos += 4;

        let value_str = match val_type {
            0 => { // uint8
                if pos + 1 > data.len() { break; }
                let v = data[pos];
                pos += 1;
                format!("{}", v)
            }
            1 => { // int8
                if pos + 1 > data.len() { break; }
                let v = data[pos] as i8;
                pos += 1;
                format!("{}", v)
            }
            2 => { // int16
                if pos + 2 > data.len() { break; }
                let v = i16::from_le_bytes([data[pos], data[pos+1]]);
                pos += 2;
                format!("{}", v)
            }
            3 => { // int32
                if pos + 4 > data.len() { break; }
                let v = i32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]);
                pos += 4;
                format!("{}", v)
            }
            8 => { // uint64 — used for architecture enum values in some GGUF files
                if pos + 8 > data.len() { break; }
                let v = u64::from_le_bytes([
                    data[pos], data[pos+1], data[pos+2], data[pos+3],
                    data[pos+4], data[pos+5], data[pos+6], data[pos+7],
                ]);
                pos += 8;
                format!("{}", v)
            }
            4 => { // float32
                if pos + 4 > data.len() { break; }
                let v = f32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]);
                pos += 4;
                format!("{}", v)
            }
            5 => { // bool
                if pos + 1 > data.len() { break; }
                let v = data[pos] != 0;
                pos += 1;
                format!("{}", v)
            }
            6 => { // string
                if pos + 8 > data.len() { break; }
                let s_len = u64::from_le_bytes([
                    data[pos], data[pos+1], data[pos+2], data[pos+3],
                    data[pos+4], data[pos+5], data[pos+6], data[pos+7],
                ]) as usize;
                pos += 8;
                let end = std::cmp::min(pos + s_len, data.len());
                let s = String::from_utf8_lossy(&data[pos..end]).to_string();
                pos += s_len;
                s
            }
            7 => { // array
                if pos + 4 > data.len() { break; }
                let arr_type = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]);
                pos += 4;
                if pos + 8 > data.len() { break; }
                let arr_len = u64::from_le_bytes([
                    data[pos], data[pos+1], data[pos+2], data[pos+3],
                    data[pos+4], data[pos+5], data[pos+6], data[pos+7],
                ]) as usize;
                pos += 8;
                format!("array[{}] type={}", arr_len, arr_type)
            }
            _ => {
                format!("<unknown_type_{}>", val_type)
            }
        };

        // Map known keys
        match key.as_str() {
            "general.architecture" => architecture = Some(value_str.clone()),
            "general.name" => name = Some(value_str.clone()),
            "general.description" => description = Some(value_str.clone()),
            "general.file_type" => file_type = value_str.parse::<u32>().ok(),
            "llama.context_length" | "qwen2.context_length" | "qwen2moe.context_length" | "starcoder2.context_length" |
            "command-r.context_length" | "dbrx.context_length" | "bert.context_length" | "nomic-bert.context_length" |
            "grok-1.context_length" | "phi2.context_length" | "phi3.context_length" | "phi3small.context_length" |
            "mixtral.context_length" | "mistral.context_length" | "yi.context_length" | "falcon.context_length" |
            "baichuan.context_length" | "internlm2.context_length" | "gemma.context_length" | "gemma2.context_length" |
            "starcoder.context_length" | "mpt.context_length" | "bloom.context_length" | "chatglm.context_length" |
            "deepseek2.context_length" | "exaone.context_length" | "olmo.context_length" | "opt.context_length" |
            "phi.context_length" | "plamo.context_length" | "gptneox.context_length" | "gpt2.context_length" => {
                context_length = value_str.parse::<u64>().ok();
            }
            "llama.embedding_length" | "qwen2.embedding_length" | "mistral.embedding_length" => {
                embedding_length = value_str.parse::<u64>().ok();
            }
            "llama.block_count" | "qwen2.block_count" | "mistral.block_count" => {
                block_count = value_str.parse::<u64>().ok();
            }
            "llama.attention.head_count" | "qwen2.attention.head_count" | "mistral.attention.head_count" => {
                head_count = value_str.parse::<u64>().ok();
            }
            "llama.attention.head_count_kv" | "qwen2.attention.head_count_kv" | "mistral.attention.head_count_kv" => {
                head_count_kv = value_str.parse::<u64>().ok();
            }
            "llama.feed_forward_length" | "qwen2.feed_forward_length" | "mistral.feed_forward_length" => {
                feed_forward_length = value_str.parse::<u64>().ok();
            }
            "tokenizer.chat_template" => chat_template = Some(value_str.clone()),
            "tokenizer.ggml.model" => tokenizer_model = Some(value_str.clone()),
            "general.quantization_version" => quantization = Some(format!("v{}", value_str)),
            "general.training.context" | "llama.training.context" => {
                training_context = value_str.parse::<u64>().ok();
            }
            _ if key.contains(".context_length") => {
                if context_length.is_none() {
                    context_length = value_str.parse::<u64>().ok();
                }
            }
            _ => {}
        }

        raw_metadata.push((key, value_str));
    }

    // Infer quantization from filename if not in metadata
    let quantization = quantization.or_else(|| {
        let fname = path.file_name()?.to_string_lossy();
        infer_quant_from_filename(&fname)
    });

    // ── Architecture Database Fallback ──
    // If the GGUF header doesn't include metadata keys (common with newer llama.cpp),
    // infer dimensions from the architecture + filename.
    let fname = path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let params_b = infer_params_from_filename(&fname);

    if context_length.is_none() || block_count.is_none() {
        if let Some(arch_str) = &architecture {
            if let Some(arch_info) = lookup_architecture(arch_str, params_b) {
                if context_length.is_none() {
                    context_length = Some(arch_info.context_length);
                }
                if embedding_length.is_none() {
                    embedding_length = Some(arch_info.embedding_length);
                }
                if block_count.is_none() {
                    block_count = Some(arch_info.block_count);
                }
                if head_count.is_none() {
                    head_count = Some(arch_info.head_count);
                }
                if head_count_kv.is_none() {
                    head_count_kv = Some(arch_info.head_count_kv);
                }
                if feed_forward_length.is_none() {
                    feed_forward_length = Some(arch_info.feed_forward_length);
                }
            }
        }
    }

    // Infer training context from context_length if not present
    if training_context.is_none() {
        training_context = context_length;
    }

    Ok(GgufMetadata {
        file_path: path.to_string_lossy().to_string(),
        file_size_bytes: file_size,
        gguf_version: version,
        tensor_count,
        metadata_kv_count,
        architecture,
        name,
        description,
        file_type,
        context_length,
        embedding_length,
        block_count,
        head_count,
        head_count_kv,
        feed_forward_length,
        chat_template,
        tokenizer_model,
        training_context,
        quantization,
        raw_metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_nonexistent_file() {
        let result = read_gguf_metadata(Path::new("/nonexistent/model.gguf"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot read file"));
    }

    #[test]
    fn test_invalid_magic() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fake.gguf");
        // Write a file with >= 16 bytes but wrong magic
        let mut data = vec![0x41u8; 24]; // 24 bytes of 'AAAA...' instead of 'GGUF'
        std::fs::write(&path, &data).unwrap();
        let result = read_gguf_metadata(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("wrong magic"));
    }

    #[test]
    fn test_too_small_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("small.gguf");
        std::fs::write(&path, &[0u8; 4]).unwrap();
        let result = read_gguf_metadata(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_minimal_valid_header() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.gguf");

        // Build minimal GGUF header
        let mut data = Vec::new();
        data.extend_from_slice(b"GGUF");        // magic
        data.extend_from_slice(&3u32.to_le_bytes()); // version
        data.extend_from_slice(&0u64.to_le_bytes()); // tensor_count = 0
        data.extend_from_slice(&1u64.to_le_bytes()); // metadata_kv_count = 1

        // Key: "general.architecture" (string)
        let key = "general.architecture";
        data.extend_from_slice(&(key.len() as u64).to_le_bytes());
        data.extend_from_slice(key.as_bytes());

        // Value type: string (6)
        data.extend_from_slice(&6u32.to_le_bytes());

        // Value: "TestArch"
        let val = "TestArch";
        data.extend_from_slice(&(val.len() as u64).to_le_bytes());
        data.extend_from_slice(val.as_bytes());

        std::fs::write(&path, &data).unwrap();

        let result = read_gguf_metadata(&path).unwrap();
        assert_eq!(result.gguf_version, 3);
        assert_eq!(result.architecture.as_deref(), Some("TestArch"));
        assert_eq!(result.file_size_bytes, data.len() as u64);
    }

    #[test]
    fn test_display_format() {
        let meta = GgufMetadata {
            file_path: "/path/to/qwen3.5-4b-q4_k_m.gguf".to_string(),
            file_size_bytes: 3_000_000_000,
            gguf_version: 3,
            tensor_count: 290,
            metadata_kv_count: 15,
            architecture: Some("Qwen2".to_string()),
            name: Some("Qwen3.5-4B".to_string()),
            description: None,
            file_type: Some(15),
            context_length: Some(32768),
            embedding_length: Some(2560),
            block_count: Some(40),
            head_count: Some(32),
            head_count_kv: Some(8),
            feed_forward_length: Some(10240),
            chat_template: Some("{{ messages }}".to_string()),
            tokenizer_model: Some("SentencePiece".to_string()),
            training_context: None,
            quantization: Some("Q4_K_M".to_string()),
            raw_metadata: vec![("custom.key".to_string(), "custom_value".to_string())],
        };
        let display = meta.display();
        assert!(display.contains("Qwen2"));
        assert!(display.contains("2.8 GB"));
        assert!(display.contains("Q4_K_M"));
        assert!(display.contains("32768"));
    }
}
