use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use super::capability::{help_text_suggests_kv_type, RuntimeCapabilities, KV_CACHE_TYPES_ALL};

// ── Runtime Prober ──
// Probes a runtime binary to discover its capabilities.
// All probing is deterministic — no LLM, no heuristics.
// Uses three methods:
//   1. --version output → version string
//   2. --help output → parse known flags
//   3. Adjacent binary discovery → special binaries (llama-memory-recurrent, etc.)

pub struct RuntimeProber;

impl RuntimeProber {
    /// Probe ALL available runtimes on the system.
    /// Searches PATH and common build directories.
    pub fn probe_all() -> Vec<RuntimeCapabilities> {
        let mut results = Vec::new();

        // Standard PATH detection — these are in $PATH
        let path_binaries = Self::discover_path_binaries();
        results.extend(path_binaries);

        // Common build directories — these are NOT in PATH but contain compiled binaries
        let build_dirs = Self::discover_build_directories();
        results.extend(build_dirs);

        // Sort by capability score descending
        results.sort_by(|a, b| {
            b.capability_score()
                .partial_cmp(&a.capability_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Deduplicate by binary path
        let mut seen = std::collections::HashSet::new();
        results.retain(|r| seen.insert(r.binary_path.clone()));

        results
    }

    /// Probe a single binary by its filesystem path.
    /// Returns a default/empty entry if the binary doesn't exist.
    pub fn probe_binary(path: &Path) -> RuntimeCapabilities {
        if !path.exists() || !path.is_file() {
            let name = path.file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            return RuntimeCapabilities {
                binary_path: path.to_string_lossy().to_string(),
                binary_name: name.clone(),
                display_name: name,
                version: None,
                special_binaries: Vec::new(),
                has_server_mode: false,
                has_cli_mode: false,
                supports_flash_attention: false,
                supports_cuda: false,
                supports_vulkan: false,
                supports_metal: false,
                kv_cache_types: Vec::new(),
                supports_kv_cache_quant: false,
                supports_kv_in_ram: false,
                supports_bonsai: false,
                supports_iswa: false,
                has_memory_recurrent: false,
                has_memory_hybrid: false,
                supports_embeddings: false,
                supports_vision: false,
                supports_grammar: false,
                supports_tool_calling: false,
                supports_speculative_decoding: false,
                supports_continuous_batching: false,
                supports_rope_scaling: false,
                supports_sliding_window: false,
                probed_at: String::new(),
                help_flags_found: Vec::new(),
            };
        }

        let fname = path.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let probed_at = {
            use std::time::{SystemTime, UNIX_EPOCH};
            let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
            format!("{}", d.as_secs())
        };

        // Phase 1: Get version
        let version = Self::get_version(path);

        // Phase 2: Get help and parse
        let help_text = Self::get_help(path);
        let help_flags = Self::extract_flags(&help_text);

        // Phase 3: Parse capabilities from help text and binary name
        let is_llama = fname.contains("llama");
        let is_ollama = fname.contains("ollama") || fname.contains("ollam");
        let is_lm_studio = fname.contains("lms");
        let is_grok = fname.contains("grok") || fname.contains("pi");
        let help_lower = help_text.to_lowercase();

        let display_name = if is_ollama {
            "Ollama".to_string()
        } else if is_lm_studio {
            "LM Studio".to_string()
        } else if is_grok {
            "Grok CLI".to_string()
        } else if path.to_string_lossy().contains("prism") || path.to_string_lossy().contains("bonsai") {
            "PrismML llama.cpp".to_string()
        } else if path.to_string_lossy().contains("turboquant") {
            "TurboQuant llama.cpp".to_string()
        } else if is_llama {
            "llama.cpp".to_string()
        } else {
            fname.clone()
        };

        let has_server_mode = fname.contains("server") || help_lower.contains("--port")
            || help_lower.contains("server");
        let has_cli_mode = fname.contains("cli") || fname.contains("main")
            || help_lower.contains("--prompt") || help_lower.contains("--n-predict");

        let supports_flash_attention = help_lower.contains("flash")
            || help_lower.contains("flash-attn");
        let supports_cuda = help_lower.contains("cuda") || help_lower.contains("--gpu-layers")
            || help_lower.contains("ngl") || help_lower.contains("n-gpu-layers");
        let supports_vulkan = help_lower.contains("vulkan");
        let supports_metal = help_lower.contains("metal");

        // Detect KV cache quant types
        let kv_cache_types: Vec<String> = KV_CACHE_TYPES_ALL.iter()
            .filter(|kt| help_text_suggests_kv_type(&help_lower, kt))
            .map(|s| s.to_string())
            .collect();
        let supports_kv_cache_quant = help_lower.contains("cache-type")
            || help_lower.contains("cache_type")
            || help_lower.contains("k-type")
            || help_lower.contains("v-type")
            || !kv_cache_types.is_empty();
        let supports_kv_in_ram = help_lower.contains("cache-ram")
            || help_lower.contains("cache_in_ram")
            || help_lower.contains("kv-ram")
            || help_lower.contains("--memory-f32")
            || help_lower.contains("memory-type");

        // Bonsai / ISWA / memory architectures
        let supports_bonsai = path.to_string_lossy().to_lowercase().contains("prism")
            || help_lower.contains("bonsai")
            || help_lower.contains("memory-hybrid")
            || help_lower.contains("memory-recurrent")
            || help_lower.contains("hierarchical");
        let supports_iswa = help_lower.contains("iswa")
            || help_lower.contains("kv-cache-iswa")
            || help_lower.contains("importance-based");

        // Embeddings
        let supports_embeddings = help_lower.contains("embed")
            || help_lower.contains("--embd")
            || fname.contains("embed");

        // Vision
        let supports_vision = help_lower.contains("mmproj")
            || help_lower.contains("multimodal")
            || help_lower.contains("vision")
            || help_lower.contains("llava")
            || help_lower.contains("qwen2vl")
            || fname.contains("llava") || fname.contains("qwen2vl");

        // Grammar
        let supports_grammar = help_lower.contains("grammar")
            || help_lower.contains("--grammar");

        // Tool calling
        let supports_tool_calling = help_lower.contains("tool")
            || help_lower.contains("function-call")
            || help_lower.contains("function_call");

        // Speculative decoding
        let supports_speculative_decoding = help_lower.contains("speculative")
            || help_lower.contains("draft")
            || help_lower.contains("--draft");

        // Continuous batching
        let supports_continuous_batching = help_lower.contains("batch")
            && (help_lower.contains("continuous") || help_lower.contains("cont-batch"));

        // Rope scaling
        let supports_rope_scaling = help_lower.contains("rope")
            || help_lower.contains("context-shift")
            || help_lower.contains("rope-freq");

        // Sliding window
        let supports_sliding_window = help_lower.contains("sliding")
            || help_lower.contains("window-attention");

        // Phase 4: Discover special binary companions
        let parent_dir = path.parent().unwrap_or(Path::new(""));
        let special_binaries = Self::discover_special_binaries(parent_dir, &fname);

        // Separate memory binaries check
        let has_memory_recurrent = special_binaries.iter().any(|b| b.contains("memory-recurrent"));
        let has_memory_hybrid = special_binaries.iter().any(|b| b.contains("memory-hybrid"));

        RuntimeCapabilities {
            binary_path: path.to_string_lossy().to_string(),
            binary_name: fname,
            display_name,
            version,
            special_binaries,
            has_server_mode,
            has_cli_mode,
            supports_flash_attention,
            supports_cuda,
            supports_vulkan,
            supports_metal,
            kv_cache_types,
            supports_kv_cache_quant,
            supports_kv_in_ram,
            supports_bonsai,
            supports_iswa,
            has_memory_recurrent,
            has_memory_hybrid,
            supports_embeddings,
            supports_vision,
            supports_grammar,
            supports_tool_calling,
            supports_speculative_decoding,
            supports_continuous_batching,
            supports_rope_scaling,
            supports_sliding_window,
            probed_at,
            help_flags_found: help_flags,
        }
    }

    // ── Private helpers ──

    /// Find runtime binaries on PATH
    fn discover_path_binaries() -> Vec<RuntimeCapabilities> {
        let mut results = Vec::new();

        // Known runtime binary names
        let known: &[(&str, &str)] = &[
            ("llama-server", "llama.cpp (server)"),
            ("llama-cli", "llama.cpp (CLI)"),
            ("ollama", "Ollama"),
            ("lms", "LM Studio"),
            ("grok", "Grok CLI"),
            ("pi", "Pi Agent"),
        ];

        for (name, display) in known {
            if let Some(path) = Self::which(name) {
                let caps = Self::probe_binary(&path);
                // Preserve the friendly display name
                let mut caps = caps;
                caps.display_name = display.to_string();
                results.push(caps);
            }
        }

        results
    }

    /// Find runtime binaries in common build directories
    fn discover_build_directories() -> Vec<RuntimeCapabilities> {
        let mut results = Vec::new();
        let home = std::env::var("HOME").unwrap_or_default();

        let candidate_dirs = vec![
            "~/prism-llama.cpp/build/bin",
            "~/llama.cpp/build/bin",
            "~/llama.cpp-turboquant/build/bin",
            "~/.lmstudio/bin",
        ];

        for dir_str in &candidate_dirs {
            let dir = PathBuf::from(dir_str.replace('~', &home));
            if !dir.is_dir() { continue; }

            // Look for runtime binaries in this directory
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_file() { continue; }
                    let fname = path.file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Skip object files, libraries, headers
                    if fname.ends_with(".o") || fname.ends_with(".a")
                        || fname.ends_with(".so") || fname.ends_with(".h") {
                        continue;
                    }

                    // Only probe inference runtime binaries (not utilities like bench, quantize, etc.)
                    let is_inference_runtime = match fname.as_str() {
                        "llama-server" | "llama-cli" | "llama-main" | "main" | "server"
                        | "llama-memory-recurrent" | "llama-memory-hybrid"
                        | "llama-kv-cache-iswa" | "llama-qwen2vl-cli" | "llama-llava-cli"
                        | "lms" | "lms-cli" => true,
                        _ if fname.starts_with("llama") => {
                            // Additional inference-related binaries
                            fname.contains("server") || fname.contains("cli")
                                || fname.contains("memory") || fname.contains("kv-cache")
                                || fname.contains("llava") || fname.contains("qwen2vl")
                                || fname.contains("embedding")
                        }
                        _ => false,
                    };

                    if is_inference_runtime {
                        let caps = Self::probe_binary(&path);
                        results.push(caps);
                    }
                }
            }
        }

        results
    }

    /// Discover special companion binaries in the same directory
    fn discover_special_binaries(dir: &Path, exclude: &str) -> Vec<String> {
        let mut found = Vec::new();
        if !dir.is_dir() { return found; }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() { continue; }
                let fname = path.file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Skip the binary itself
                if fname == exclude { continue; }

                // Known special binaries
                let special_patterns = [
                    "memory-recurrent", "memory-hybrid",
                    "kv-cache-iswa", "qwen2vl", "llava",
                    "embedding", "perplexity",
                ];

                if special_patterns.iter().any(|p| fname.contains(p)) {
                    found.push(fname);
                }
            }
        }

        found.sort();
        found
    }

    /// Run `--version` on a binary and return a clean version string.
    /// Strips common prefixes like "version:" to avoid double-prefixing in display.
    fn get_version(path: &Path) -> Option<String> {
        let output = Command::new(path)
            .arg("--version")
            .output()
            .ok()?;

        let stdout = String::from_utf8(output.stdout).ok()?;
        let stderr = String::from_utf8(output.stderr).ok()?;
        let combined = stdout + &stderr;
        let first_line = combined.lines().next()?.trim().to_string();

        // Strip common prefixes so display_name_short() doesn't show "vversion:"
        let cleaned = first_line
            .strip_prefix("version: ")
            .or_else(|| first_line.strip_prefix("version "))
            .or_else(|| first_line.strip_prefix("v"))
            .unwrap_or(&first_line)
            .to_string();

        Some(cleaned)
    }

    /// Run `--help` on a binary and return the output text
    fn get_help(path: &Path) -> String {
        let output = Command::new(path)
            .arg("--help")
            .output()
            .ok();
        match output {
            Some(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                format!("{}\n{}", stdout, stderr)
            }
            None => String::new(),
        }
    }

    /// Extract flag-like tokens from help text for debugging
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
        flags.truncate(100); // Limit to avoid huge lists
        flags
    }

    /// Like `which` command: find a binary on PATH
    fn which(name: &str) -> Option<PathBuf> {
        let output = Command::new("which").arg(name).output().ok()?;
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
        None
    }
}
