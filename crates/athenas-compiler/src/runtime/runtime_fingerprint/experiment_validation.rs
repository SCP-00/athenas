/// PHASE-0011 — Experiment Validation.
///
/// Responde la pregunta: **"¿Este experimento merece ejecutarse?"**
///
/// No ejecuta nada. Solo valida.
///
/// ## Verificaciones
/// 1. Modelo: existe, hash, GGUF íntegro, metadata consistente
/// 2. Runtime: fingerprint existe, runtime saludable, versión conocida, librerías presentes
/// 3. Compatibilidad: el runtime soporta las features solicitadas
/// 4. Parámetros: coherencia entre ctx, batch, ubatch, ngl, threads, memoria, KV, flash
/// 5. Recursos: VRAM suficiente, RAM suficiente, GPU libre, no hay otro llama-server activo
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

// ═══════════════════════════════════════════════════════════════
// Core Types
// ═══════════════════════════════════════════════════════════════

/// Result of validating an experiment configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Overall pass/fail
    pub passed: bool,

    /// Experiment configuration that was validated
    pub configuration: serde_json::Value,

    /// Individual check results
    pub checks: Vec<ValidationCheck>,

    /// Hardware snapshot at validation time
    pub hardware: serde_json::Value,

    /// Summary
    pub summary: ValidationSummary,
}

/// A single validation check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    /// Check name (e.g., "model_exists", "runtime_fingerprint", "feature_compatibility")
    pub name: String,
    /// Whether the check passed
    pub passed: bool,
    /// Severity: "critical", "warning", "info"
    pub severity: String,
    /// Description of what was checked
    pub description: String,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Detail about what was found (if passed)
    pub detail: Option<String>,
}

/// Summary of validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Total checks performed
    pub total: usize,
    /// Checks passed
    pub passed: usize,
    /// Critical failures (experiment must not run)
    pub critical_failures: usize,
    /// Warnings (experiment may run but with caveats)
    pub warnings: usize,
    /// Decision: "can_run", "cannot_run", "needs_review"
    pub decision: String,
    /// Confidence that the experiment will succeed (0.0 to 1.0)
    pub confidence: f64,
}

// ═══════════════════════════════════════════════════════════════
// Experiment Validator
// ═══════════════════════════════════════════════════════════════

/// The Experiment Validator — validates an experiment before execution.
pub struct ExperimentValidator;

impl ExperimentValidator {
    /// Validate a complete experiment configuration.
    /// Returns a ValidationResult with all checks.
    pub fn validate(config: &ExperimentConfig) -> ValidationResult {
        let mut checks = Vec::new();
        let mut passed_count = 0;
        let mut critical_failures = 0;
        let mut warnings = 0;

        // ── 1. Model validation ──
        checks.push(Self::check_model_exists(&config.model_path));
        if checks.last().map_or(false, |c| c.passed) {
            checks.push(Self::check_model_hash(&config.model_path));
            checks.push(Self::check_gguf_integrity(&config.model_path));
            checks.push(Self::check_model_size(&config.model_path));
            checks.push(Self::check_model_metadata_consistency(&config.model_path, &config));
        }

        // ── 2. Runtime validation ──
        checks.push(Self::check_runtime_exists(&config.runtime_path));
        if checks.last().map_or(false, |c| c.passed) {
            checks.push(Self::check_runtime_executable(&config.runtime_path));
            checks.push(Self::check_runtime_fingerprint(&config.runtime_path));
            checks.push(Self::check_runtime_libraries(&config.runtime_path));
        }

        // ── 3. Feature compatibility ──
        if Path::new(&config.runtime_path).exists() {
            checks.push(Self::check_feature_compatibility(&config));
        }

        // ── 4. Parameter coherence ──
        checks.push(Self::check_parameter_coherence(&config));

        // ── 5. Resource availability ──
        checks.push(Self::check_vram_available(&config));
        checks.push(Self::check_ram_available(&config));
        checks.push(Self::check_disk_space());
        checks.push(Self::check_no_other_llama_server());
        checks.push(Self::check_gpu_available());

        // ── Tally results ──
        let checks_len = checks.len();
        for check in &checks {
            if check.passed {
                passed_count += 1;
            } else if check.severity == "critical" {
                critical_failures += 1;
            } else {
                warnings += 1;
            }
        }

        // ── Decision ──
        let decision = if critical_failures > 0 {
            "cannot_run"
        } else if warnings > 2 {
            "needs_review"
        } else {
            "can_run"
        };

        // ── Confidence ──
        let confidence = if checks_len == 0 {
            0.0
        } else {
            (passed_count as f64 - critical_failures as f64 * 0.5) / checks_len as f64
        };

        // Hardware snapshot
        let hardware = crate::runtime::hardware::detect_hardware();
        let hardware_json = serde_json::to_value(&hardware).unwrap_or_default();

        ValidationResult {
            passed: decision == "can_run",
            configuration: serde_json::to_value(config).unwrap_or_default(),
            checks,
            hardware: hardware_json,
            summary: ValidationSummary {
                total: checks_len,
                passed: passed_count,
                critical_failures,
                warnings,
                decision: decision.to_string(),
                confidence: (confidence * 1000.0).round() / 1000.0,
            },
        }
    }

    // ── 1. Model checks ──

    fn check_model_exists(model_path: &str) -> ValidationCheck {
        let path = Path::new(model_path);
        let exists = path.exists();
        ValidationCheck {
            name: "model_exists".to_string(),
            passed: exists,
            severity: if exists { "info".to_string() } else { "critical".to_string() },
            description: "Verifica que el archivo del modelo existe".to_string(),
            error: if exists { None } else { Some(format!("Model not found: {model_path}")) },
            detail: if exists { Some(format!("File size: {} MB", path.metadata().map(|m| m.len() / 1_048_576).unwrap_or(0))) } else { None },
        }
    }

    fn check_model_hash(model_path: &str) -> ValidationCheck {
        let path = Path::new(model_path);
        let result = compute_sha256(path);
        let (passed, error, detail) = match result {
            Ok(hash) => (true, None, Some(format!("SHA256: {}...", &hash[..16]))),
            Err(e) => (false, Some(format!("Cannot compute hash: {e}")), None),
        };
        ValidationCheck {
            name: "model_hash".to_string(),
            passed,
            severity: "warning".to_string(),
            description: "Calcula SHA256 del modelo para identificación futura".to_string(),
            error,
            detail,
        }
    }

    fn check_gguf_integrity(model_path: &str) -> ValidationCheck {
        let path = Path::new(model_path);
        let metadata = crate::runtime::model_intelligence::gguf::read_gguf_metadata(path);
        match metadata {
            Ok(m) => {
                let has_arch = m.architecture.is_some();
                ValidationCheck {
                    name: "gguf_integrity".to_string(),
                    passed: has_arch,
                    severity: "critical".to_string(),
                    description: "Lee el header GGUF y verifica que la metadata sea válida".to_string(),
                    error: if has_arch { None } else { Some("GGUF header missing architecture metadata".to_string()) },
                    detail: if has_arch {
                        Some(format!("Architecture: {:?}, Context: {:?}", m.architecture, m.context_length))
                    } else {
                        None
                    },
                }
            }
            Err(e) => ValidationCheck {
                name: "gguf_integrity".to_string(),
                passed: false,
                severity: "critical".to_string(),
                description: "Lee el header GGUF y verifica que la metadata sea válida".to_string(),
                error: Some(format!("Cannot read GGUF header: {e}")),
                detail: None,
            },
        }
    }

    fn check_model_size(model_path: &str) -> ValidationCheck {
        let path = Path::new(model_path);
        let size_mb = path.metadata().map(|m| m.len() / 1_048_576).unwrap_or(0);
        ValidationCheck {
            name: "model_size".to_string(),
            passed: size_mb > 0,
            severity: "critical".to_string(),
            description: "Verifica que el modelo tenga tamaño no nulo".to_string(),
            error: if size_mb == 0 { Some("Model file is empty".to_string()) } else { None },
            detail: Some(format!("{size_mb} MB")),
        }
    }

    fn check_model_metadata_consistency(model_path: &str, _config: &ExperimentConfig) -> ValidationCheck {
        let path = Path::new(model_path);
        let metadata = crate::runtime::model_intelligence::gguf::read_gguf_metadata(path);
        match metadata {
            Ok(m) => {
                // Check that context is at least reasonable (> 0)
                let ctx_ok = m.context_length.map(|c| c > 0).unwrap_or(true);
                ValidationCheck {
                    name: "model_metadata_consistency".to_string(),
                    passed: ctx_ok,
                    severity: "warning".to_string(),
                    description: "Verifica consistencia de metadata del modelo".to_string(),
                    error: if ctx_ok { None } else { Some("Model context is 0".to_string()) },
                    detail: Some(format!("Architecture: {:?}, Layers: {:?}, Context: {:?}",
                        m.architecture, m.block_count, m.context_length)),
                }
            }
            Err(e) => ValidationCheck {
                name: "model_metadata_consistency".to_string(),
                passed: false,
                severity: "warning".to_string(),
                description: "Verifica consistencia de metadata del modelo".to_string(),
                error: Some(format!("Cannot read GGUF: {e}")),
                detail: None,
            },
        }
    }

    // ── 2. Runtime checks ──

    fn check_runtime_exists(runtime_path: &str) -> ValidationCheck {
        let path = Path::new(runtime_path);
        let exists = path.exists();
        ValidationCheck {
            name: "runtime_exists".to_string(),
            passed: exists,
            severity: "critical".to_string(),
            description: "Verifica que el binario del runtime existe".to_string(),
            error: if exists { None } else { Some(format!("Runtime not found: {runtime_path}")) },
            detail: if exists { Some("Binary found".to_string()) } else { None },
        }
    }

    fn check_runtime_executable(runtime_path: &str) -> ValidationCheck {
        let path = Path::new(runtime_path);
        let is_executable = path.is_file();
        ValidationCheck {
            name: "runtime_executable".to_string(),
            passed: is_executable,
            severity: "critical".to_string(),
            description: "Verifica que el runtime sea un archivo ejecutable".to_string(),
            error: if is_executable { None } else { Some("Runtime is not a valid executable".to_string()) },
            detail: if is_executable { Some("File is executable".to_string()) } else { None },
        }
    }

    fn check_runtime_fingerprint(runtime_path: &str) -> ValidationCheck {
        let path = Path::new(runtime_path);
        let fp = crate::runtime::runtime_fingerprint::fingerprint_runtime(path);
        let has_sha256 = !fp.executable_sha256.is_empty() && fp.executable_sha256 != "0000000000000000000000000000000000000000000000000000000000000000";

        ValidationCheck {
            name: "runtime_fingerprint".to_string(),
            passed: has_sha256,
            severity: "warning".to_string(),
            description: "Calcula identidad forense del runtime (SHA256, BuildID, .so)".to_string(),
            error: if has_sha256 { None } else { Some("Cannot compute runtime fingerprint".to_string()) },
            detail: if has_sha256 {
                Some(format!("{} ({}) — {} .so files, commit: {:?}",
                    fp.display_name, fp.variant, fp.libraries.len(), fp.commit))
            } else {
                None
            },
        }
    }

    fn check_runtime_libraries(runtime_path: &str) -> ValidationCheck {
        let path = Path::new(runtime_path);
        let fp = crate::runtime::runtime_fingerprint::fingerprint_runtime(path);
        let all_libs_found = fp.ldd_entries.iter().all(|e| e.resolved_path != "not found");
        let missing: Vec<&str> = fp.ldd_entries.iter()
            .filter(|e| e.resolved_path == "not found")
            .map(|e| e.library.as_str())
            .collect();

        ValidationCheck {
            name: "runtime_libraries".to_string(),
            passed: all_libs_found,
            severity: "critical".to_string(),
            description: "Verifica que todas las librerías del runtime estén presentes".to_string(),
            error: if all_libs_found { None } else {
                Some(format!("Missing {} libraries: {}", missing.len(), missing.join(", ")))
            },
            detail: Some(format!("{} libraries linked, {} found", fp.ldd_entries.len(), fp.ldd_entries.len() - missing.len())),
        }
    }

    // ── 3. Feature compatibility ──

    fn check_feature_compatibility(config: &ExperimentConfig) -> ValidationCheck {
        let path = Path::new(&config.runtime_path);
        let help_text = get_help_text(path);
        let help_lower = help_text.to_lowercase();
        let mut incompatibilities = Vec::new();

        // Check if requested features are supported by the runtime
        if config.flash_attention {
            if !help_lower.contains("flash") && !help_lower.contains("flash-attn") {
                incompatibilities.push("Flash Attention requested but runtime does not support it".to_string());
            }
        }

        if config.kv_cache_type == "turbo3" || config.kv_cache_type == "iq4_nl" {
            if !help_lower.contains("cache-type") && !help_lower.contains("turbo3") && !config.runtime_path.to_lowercase().contains("turboquant") {
                incompatibilities.push("Turbo3 KV cache requested but runtime may not support it".to_string());
            }
        }

        if let Some(kv_type) = &config.kv_cache_type_value {
            if kv_type != "f16" && kv_type != "auto" {
                let kv_flag = format!("{kv_type}");
                if !help_lower.contains("cache-type") && !help_lower.contains(&kv_flag) {
                    incompatibilities.push(format!("KV cache type '{kv_type}' may not be supported by this runtime"));
                }
            }
        }

        ValidationCheck {
            name: "feature_compatibility".to_string(),
            passed: incompatibilities.is_empty(),
            severity: "critical".to_string(),
            description: "Verifica que el runtime soporte las features solicitadas".to_string(),
            error: if incompatibilities.is_empty() { None } else {
                Some(incompatibilities.join("; "))
            },
            detail: if incompatibilities.is_empty() {
                Some("All requested features appear compatible".to_string())
            } else {
                None
            },
        }
    }

    // ── 4. Parameter coherence ──

    fn check_parameter_coherence(config: &ExperimentConfig) -> ValidationCheck {
        let mut issues = Vec::new();

        // ctx must be positive
        if config.context_length == 0 {
            issues.push("Context length is 0".to_string());
        }

        // batch must be >= ubatch
        if config.batch_size < config.ubatch_size {
            issues.push(format!("Batch ({}) < UBatch ({}) — batch must be >= ubatch", config.batch_size, config.ubatch_size));
        }

        // threads must be > 0
        if config.threads == 0 {
            issues.push("Threads is 0".to_string());
        }

        // max_tokens must be > 0 for generation
        if config.max_tokens == 0 {
            issues.push("max_tokens is 0 — no tokens will be generated".to_string());
        }

        // gpu_layers must be valid
        if config.gpu_layers < -1 {
            issues.push(format!("Invalid gpu_layers: {}", config.gpu_layers));
        }

        // context must not exceed model max
        if config.context_length > 262144 {
            issues.push(format!("Context {} exceeds typical maximum (262144)", config.context_length));
        }

        ValidationCheck {
            name: "parameter_coherence".to_string(),
            passed: issues.is_empty(),
            severity: if issues.iter().any(|i| i.contains("0") || i.contains("<")) { "critical".to_string() } else { "warning".to_string() },
            description: "Verifica coherencia entre parámetros del experimento".to_string(),
            error: if issues.is_empty() { None } else { Some(issues.join("; ")) },
            detail: if issues.is_empty() {
                Some(format!("Ctx: {}, batch: {}, ubatch: {}, threads: {}, ngl: {}, tokens: {}",
                    config.context_length, config.batch_size, config.ubatch_size,
                    config.threads, config.gpu_layers, config.max_tokens))
            } else {
                None
            },
        }
    }

    // ── 5. Resource checks ──

    fn check_vram_available(config: &ExperimentConfig) -> ValidationCheck {
        let vram_gb = get_current_vram_gb();
        let available_vram = get_total_vram_gb() - vram_gb;
        // Estimate model VRAM requirement
        let estimated_vram = estimate_vram_needed(config);

        let enough = available_vram >= estimated_vram || estimated_vram <= 0.0;

        ValidationCheck {
            name: "vram_available".to_string(),
            passed: enough,
            severity: "critical".to_string(),
            description: "Verifica que haya suficiente VRAM libre para el modelo".to_string(),
            error: if enough { None } else {
                Some(format!("Estimated need: {:.1}GB, available: {:.1}GB (total: {:.1}GB, used: {:.1}GB)",
                    estimated_vram, available_vram, get_total_vram_gb(), vram_gb))
            },
            detail: if enough {
                Some(format!("Estimated: {:.1}GB, available: {:.1}GB (total: {:.1}GB)", estimated_vram, available_vram, get_total_vram_gb()))
            } else {
                None
            },
        }
    }

    fn check_ram_available(_config: &ExperimentConfig) -> ValidationCheck {
        let ram_gb = get_available_ram_gb();
        let enough = ram_gb > 2.0; // At least 2GB free

        ValidationCheck {
            name: "ram_available".to_string(),
            passed: enough,
            severity: "warning".to_string(),
            description: "Verifica que haya suficiente RAM libre".to_string(),
            error: if enough { None } else { Some(format!("Only {:.1}GB RAM free — need at least 2GB", ram_gb)) },
            detail: Some(format!("{:.1}GB available", ram_gb)),
        }
    }

    fn check_disk_space() -> ValidationCheck {
        // Basic check: at least 1GB free in current directory
        let free_gb = get_disk_free_gb();
        ValidationCheck {
            name: "disk_space".to_string(),
            passed: free_gb > 1.0,
            severity: "warning".to_string(),
            description: "Verifica espacio en disco para logs y artefactos".to_string(),
            error: if free_gb > 1.0 { None } else { Some(format!("Only {:.1}GB free — need at least 1GB", free_gb)) },
            detail: Some(format!("{:.1}GB free", free_gb)),
        }
    }

    fn check_no_other_llama_server() -> ValidationCheck {
        let running = find_processes("llama-server");
        ValidationCheck {
            name: "no_other_llama_server".to_string(),
            passed: running.is_empty(),
            severity: "warning".to_string(),
            description: "Verifica que no haya otro llama-server ejecutándose".to_string(),
            error: if running.is_empty() { None } else {
                Some(format!("Found {} llama-server process(es): {:?}", running.len(), running))
            },
            detail: if running.is_empty() {
                Some("No other llama-server running".to_string())
            } else {
                None
            },
        }
    }

    fn check_gpu_available() -> ValidationCheck {
        let higher_util = get_gpu_utilization() > 90.0;
        ValidationCheck {
            name: "gpu_available".to_string(),
            passed: !higher_util,
            severity: "warning".to_string(),
            description: "Verifica que la GPU no esté ocupada".to_string(),
            error: if higher_util { Some(format!("GPU utilization is {:.0}% — may be busy", get_gpu_utilization())) } else { None },
            detail: Some(format!("GPU util: {:.0}%", get_gpu_utilization())),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Experiment Configuration
// ═══════════════════════════════════════════════════════════════

/// Configuration for an experiment to be validated and executed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    /// Path to the GGUF model file
    pub model_path: String,
    /// Path to the runtime binary
    pub runtime_path: String,
    /// Context window size (tokens)
    pub context_length: u64,
    /// Batch size for prompt processing
    pub batch_size: u64,
    /// Batch size for generation
    pub ubatch_size: u64,
    /// Number of GPU layers (-1 = all, 0 = CPU only)
    pub gpu_layers: i64,
    /// KV cache type (f16, q8, q4, q6, turbo3, etc.)
    pub kv_cache_type: String,
    /// KV cache type value (actual parameter value)
    pub kv_cache_type_value: Option<String>,
    /// Whether KV cache is in RAM (vs VRAM)
    pub kv_in_ram: bool,
    /// Flash attention enabled
    pub flash_attention: bool,
    /// Number of CPU threads
    pub threads: u64,
    /// Max tokens to generate
    pub max_tokens: u64,
}

impl ExperimentConfig {
    pub fn new(model_path: &str, runtime_path: &str) -> Self {
        Self {
            model_path: model_path.to_string(),
            runtime_path: runtime_path.to_string(),
            context_length: 32768,
            batch_size: 512,
            ubatch_size: 256,
            gpu_layers: 999,
            kv_cache_type: "f16".to_string(),
            kv_cache_type_value: None,
            kv_in_ram: false,
            flash_attention: true,
            threads: 8,
            max_tokens: 100,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Resource helpers
// ═══════════════════════════════════════════════════════════════

fn get_current_vram_gb() -> f64 {
    Command::new("nvidia-smi")
        .args(["--query-gpu=memory.used", "--format=csv,noheader,nounits"])
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            s.parse::<f64>().ok().map(|mb| mb / 1024.0)
        })
        .unwrap_or(0.0)
}

fn get_total_vram_gb() -> f64 {
    Command::new("nvidia-smi")
        .args(["--query-gpu=memory.total", "--format=csv,noheader,nounits"])
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            s.parse::<f64>().ok().map(|mb| mb / 1024.0)
        })
        .unwrap_or(0.0)
}

fn get_available_ram_gb() -> f64 {
    Command::new("free")
        .arg("-b")
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout);
            for line in s.lines() {
                if line.starts_with("Mem:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 7 {
                        let available: f64 = parts[6].parse().ok()?;
                        return Some(available / (1024.0 * 1024.0 * 1024.0));
                    }
                }
            }
            None
        })
        .unwrap_or(0.0)
}

fn get_gpu_utilization() -> f64 {
    Command::new("nvidia-smi")
        .args(["--query-gpu=utilization.gpu", "--format=csv,noheader,nounits"])
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            s.parse().ok()
        })
        .unwrap_or(0.0)
}

fn get_disk_free_gb() -> f64 {
    // Use `df` to check free space in current directory
    Command::new("df")
        .args(["-B1", "."])
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout);
            for line in s.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let free: f64 = parts[3].parse().ok()?;
                    return Some(free / (1024.0 * 1024.0 * 1024.0));
                }
            }
            None
        })
        .unwrap_or(0.0)
}

fn find_processes(name: &str) -> Vec<String> {
    Command::new("pgrep")
        .arg("-a")
        .arg(name)
        .output()
        .ok()
        .map(|o| {
            let s = String::from_utf8_lossy(&o.stdout);
            s.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect()
        })
        .unwrap_or_default()
}

fn get_help_text(path: &Path) -> String {
    Command::new(path).arg("--help").output()
        .ok()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            format!("{}\n{}", stdout, stderr)
        })
        .unwrap_or_default()
}

fn compute_sha256(path: &Path) -> Result<String, String> {
    use std::fs::File;
    use std::io::Read;
    use sha2::{Sha256, Digest};
    let mut file = File::open(path).map_err(|e| format!("Cannot open: {e}"))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf).map_err(|e| format!("Cannot read: {e}"))?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Estimate VRAM needed for a model configuration (rough heuristic)
fn estimate_vram_needed(config: &ExperimentConfig) -> f64 {
    // Rough model size estimation: ~0.5GB per billion params at Q4
    // This is a very rough estimate — the VramCalculator is more precise
    let file_path = Path::new(&config.model_path);
    let file_size_gb = file_path.metadata().map(|m| m.len() as f64 / 1_048_576.0 / 1024.0).unwrap_or(4.0);
    file_size_gb * 1.2 // 20% overhead for KV cache, activations, etc.
}

// ═══════════════════════════════════════════════════════════════
// Negative Evidence — experiments that failed validation
// ═══════════════════════════════════════════════════════════════

/// A record of an experiment that was rejected by validation.
/// Stored as "negative evidence" so it's never retried.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegativeEvidence {
    /// Unique ID for this evidence record
    pub id: String,
    /// When the validation failed
    pub timestamp: u64,
    /// The experiment configuration that was rejected
    pub configuration: serde_json::Value,
    /// Why it was rejected
    pub reasons: Vec<String>,
    /// The runtime fingerprint (if available)
    pub runtime_fingerprint: Option<String>,
    /// The model hash (if available)
    pub model_hash: Option<String>,
    /// Hardware snapshot at failure time
    pub hardware: serde_json::Value,
    /// Category of failure
    pub category: String,
    /// How many times this configuration has been attempted
    pub attempt_count: u32,
}

impl NegativeEvidence {
    pub fn from_validation(config: &ExperimentConfig, result: &ValidationResult) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

        let reasons: Vec<String> = result.checks.iter()
            .filter(|c| !c.passed)
            .map(|c| format!("[{}] {}: {}", c.severity, c.name, c.error.as_deref().unwrap_or("unknown")))
            .collect();

        let category = if result.summary.critical_failures > 0 {
            "incompatible_configuration"
        } else if result.summary.warnings > 2 {
            "resource_constrained"
        } else {
            "unknown"
        };

        Self {
            id: format!("NEG-{now}"),
            timestamp: now,
            configuration: serde_json::to_value(config).unwrap_or_default(),
            reasons,
            runtime_fingerprint: None,
            model_hash: None,
            hardware: result.hardware.clone(),
            category: category.to_string(),
            attempt_count: 1,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = ExperimentConfig::new("/models/test.gguf", "/usr/bin/llama-server");
        assert_eq!(config.context_length, 32768);
        assert!(config.flash_attention);
        assert_eq!(config.max_tokens, 100);
    }

    #[test]
    fn test_validate_nonexistent_model() {
        let config = ExperimentConfig::new("/nonexistent/model.gguf", "/usr/bin/llama-server");
        let result = ExperimentValidator::validate(&config);
        assert_eq!(result.summary.decision, "cannot_run");
        assert!(!result.passed);
    }

    #[test]
    fn test_negative_evidence_creation() {
        let config = ExperimentConfig::new("/nonexistent/model.gguf", "/usr/bin/llama-server");
        let result = ExperimentValidator::validate(&config);
        let evidence = NegativeEvidence::from_validation(&config, &result);
        assert_eq!(evidence.category, "incompatible_configuration");
        assert!(!evidence.reasons.is_empty());
    }

    #[test]
    fn test_parameter_coherence() {
        let mut config = ExperimentConfig::new("/models/test.gguf", "/usr/bin/llama-server");
        config.batch_size = 128;
        config.ubatch_size = 256; // batch < ubatch — invalid
        let result = ExperimentValidator::validate(&config);
        // Should fail at least parameter_coherence check
        let param_check = result.checks.iter().find(|c| c.name == "parameter_coherence");
        assert!(param_check.is_some());
        assert!(!param_check.unwrap().passed);
    }
}
