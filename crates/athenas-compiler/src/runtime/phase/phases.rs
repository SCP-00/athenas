use std::path::{Path, PathBuf};

use super::core::{ArtifactStoreRead, Phase, PhaseContext, PhaseId, PhaseOutput, PhaseStatus};

// ═══════════════════════════════════════════════════════════════
// PHASE-0001: Hardware Snapshot
// ═══════════════════════════════════════════════════════════════

pub struct HardwarePhase;

impl Phase for HardwarePhase {
    fn id(&self) -> &str { "PHASE-0001-hardware" }
    fn name(&self) -> &str { "Hardware Snapshot" }
    fn question(&self) -> &str { "¿Qué hardware existe?" }
    fn description(&self) -> &str { "Detecta GPU, VRAM, RAM, CPU, OS y kernel del sistema" }
    fn inputs(&self) -> Vec<&str> { vec![] }

    fn execute(&self, ctx: &PhaseContext, _store: &dyn ArtifactStoreRead) -> Result<PhaseOutput, String> {
        let mut output = PhaseOutput::new(PhaseId::new(self.id()), &ctx.experiment_id);
        output.record_event("Hardware detection started", "Reading /proc, lspci, nvidia-smi");

        let hw = crate::runtime::hardware::detect_hardware();
        let artifact = serde_json::to_value(&hw).map_err(|e| format!("Serialization failed: {e}"))?;

        output.artifact = artifact.clone();
        output.metrics.add("vram_gb", hw.gpu.first().map(|g| g.vram_gb).unwrap_or(0.0), "GB");
        output.metrics.add("ram_gb", hw.memory.total_gb, "GB");
        output.metrics.add("ram_available_gb", hw.memory.available_gb, "GB");
        output.metrics.add("cpu_cores", hw.cpu.cores as f64, "cores");
        output.metrics.add("cpu_threads", hw.cpu.threads as f64, "threads");
        output.status = PhaseStatus::Success;
        output.record_event("Hardware detection complete", &format!(
            "GPU: {} ({}GB), RAM: {:.0}GB available, CPU: {} ({} cores)",
            hw.gpu.first().map(|g| g.model.as_str()).unwrap_or("none"),
            hw.gpu.first().map(|g| g.vram_gb).unwrap_or(0.0),
            hw.memory.available_gb,
            hw.cpu.model, hw.cpu.cores,
        ));

        Ok(output)
    }
}

// ═══════════════════════════════════════════════════════════════
// PHASE-0002: Runtime Discovery
// ═══════════════════════════════════════════════════════════════

pub struct RuntimeDiscoveryPhase;

impl Phase for RuntimeDiscoveryPhase {
    fn id(&self) -> &str { "PHASE-0002-runtime-discovery" }
    fn name(&self) -> &str { "Runtime Discovery" }
    fn question(&self) -> &str { "¿Qué runtimes existen?" }
    fn description(&self) -> &str { "Busca binarios de runtime en PATH y directorios de compilación" }
    fn inputs(&self) -> Vec<&str> { vec!["PHASE-0001-hardware"] }

    fn execute(&self, ctx: &PhaseContext, _store: &dyn ArtifactStoreRead) -> Result<PhaseOutput, String> {
        let mut output = PhaseOutput::new(PhaseId::new(self.id()), &ctx.experiment_id);
        output.record_event("Runtime discovery started", "Searching PATH and build directories");

        let runtimes = crate::runtime::runtime_discovery::RuntimeProber::probe_all();
        let artifact = serde_json::to_value(&runtimes).map_err(|e| format!("Serialization failed: {e}"))?;

        output.artifact = artifact;
        output.metrics.add("runtimes_found", runtimes.len() as f64, "count");
        output.status = PhaseStatus::Success;

        let names: Vec<String> = runtimes.iter().map(|r| r.display_name.clone()).collect();
        output.record_event("Runtime discovery complete", &format!(
            "Found {} runtimes: {}", runtimes.len(), names.join(", ")
        ));

        Ok(output)
    }
}

// ═══════════════════════════════════════════════════════════════
// PHASE-0003: Runtime Capabilities
// ═══════════════════════════════════════════════════════════════

pub struct RuntimeCapabilitiesPhase;

impl Phase for RuntimeCapabilitiesPhase {
    fn id(&self) -> &str { "PHASE-0003-runtime-capabilities" }
    fn name(&self) -> &str { "Runtime Capabilities" }
    fn question(&self) -> &str { "¿Qué capacidades reales tiene cada runtime?" }
    fn description(&self) -> &str { "Prueba las capacidades de cada runtime mediante --help y probing" }
    fn inputs(&self) -> Vec<&str> { vec!["PHASE-0002-runtime-discovery"] }

    fn execute(&self, ctx: &PhaseContext, _store: &dyn ArtifactStoreRead) -> Result<PhaseOutput, String> {
        let mut output = PhaseOutput::new(PhaseId::new(self.id()), &ctx.experiment_id);
        output.record_event("Capability probing started", "Parsing --help for each runtime");

        let runtimes = crate::runtime::runtime_discovery::RuntimeProber::probe_all();
        let detailed: Vec<serde_json::Value> = runtimes.iter().map(|rt| {
            serde_json::json!({
                "name": rt.display_name,
                "path": rt.binary_path,
                "version": rt.version,
                "score": rt.capability_score(),
                "capabilities": {
                    "flash_attention": rt.supports_flash_attention,
                    "cuda": rt.supports_cuda,
                    "kv_cache_quant": rt.supports_kv_cache_quant,
                    "embeddings": rt.supports_embeddings,
                    "vision": rt.supports_vision,
                    "bonsai": rt.supports_bonsai,
                    "iswa": rt.supports_iswa,
                    "speculative": rt.supports_speculative_decoding,
                    "grammar": rt.supports_grammar,
                    "rope_scaling": rt.supports_rope_scaling,
                },
                "kv_cache_types": rt.kv_cache_types,
                "special_binaries": rt.special_binaries,
            })
        }).collect();

        output.artifact = serde_json::json!({ "runtimes": detailed });
        output.status = PhaseStatus::Success;
        output.record_event("Capability probing complete", &format!("Tested {} runtimes", runtimes.len()));

        Ok(output)
    }
}

// ═══════════════════════════════════════════════════════════════
// PHASE-0004: GGUF Inspection
// ═══════════════════════════════════════════════════════════════

pub struct GgufInspectionPhase {
    pub model_path: String,
}

impl GgufInspectionPhase {
    pub fn new(model_path: &str) -> Self { Self { model_path: model_path.to_string() } }
}

impl Phase for GgufInspectionPhase {
    fn id(&self) -> &str { "PHASE-0004-gguf-inspection" }
    fn name(&self) -> &str { "GGUF Inspection" }
    fn question(&self) -> &str { "¿Qué dice realmente este modelo?" }
    fn description(&self) -> &str { "Lee la metadata del header GGUF: arquitectura, contexto, tokenizer" }
    fn inputs(&self) -> Vec<&str> { vec![] }

    fn execute(&self, ctx: &PhaseContext, _store: &dyn ArtifactStoreRead) -> Result<PhaseOutput, String> {
        let mut output = PhaseOutput::new(PhaseId::new(self.id()), &ctx.experiment_id);
        output.record_event("GGUF inspection started", &format!("Reading {}", &self.model_path));

        let path = Path::new(&self.model_path);
        let metadata = crate::runtime::model_intelligence::gguf::read_gguf_metadata(path)?;
        let artifact = serde_json::to_value(&metadata)
            .map_err(|e| format!("Serialization failed: {e}"))?;

        if let Some(ctx_val) = metadata.context_length {
            output.metrics.add("context_declared", ctx_val as f64, "tokens");
        }
        if let Some(emb) = metadata.embedding_length {
            output.metrics.add("embedding_dim", emb as f64, "dim");
        }
        output.metrics.add("file_size_mb", path.metadata().map(|m| m.len() as f64 / 1_048_576.0).unwrap_or(0.0), "MB");

        output.artifact = artifact;
        output.status = PhaseStatus::Success;
        output.record_event("GGUF inspection complete", &format!(
            "Arch: {:?}, Context: {:?}, Embed: {:?}",
            metadata.architecture, metadata.context_length, metadata.embedding_length
        ));

        Ok(output)
    }
}

// ═══════════════════════════════════════════════════════════════
// PHASE-0005: Memory Hypothesis
// ═══════════════════════════════════════════════════════════════

pub struct MemoryHypothesisPhase {
    pub model_path: String,
}

impl MemoryHypothesisPhase {
    pub fn new(model_path: &str) -> Self { Self { model_path: model_path.to_string() } }
}

impl Phase for MemoryHypothesisPhase {
    fn id(&self) -> &str { "PHASE-0005-memory-hypothesis" }
    fn name(&self) -> &str { "Memory Hypothesis" }
    fn question(&self) -> &str { "¿Qué configuraciones parecen posibles?" }
    fn description(&self) -> &str { "Genera hipótesis de memoria usando VramCalculator. NO decide — solo predice." }
    fn inputs(&self) -> Vec<&str> { vec!["PHASE-0001-hardware", "PHASE-0004-gguf-inspection"] }

    fn execute(&self, ctx: &PhaseContext, store: &dyn ArtifactStoreRead) -> Result<PhaseOutput, String> {
        let mut output = PhaseOutput::new(PhaseId::new(self.id()), &ctx.experiment_id);
        output.record_event("Memory hypothesis started", "Reading hardware and model metadata");

        // Read hardware from phase 1
        let hw_output = store.load_artifact(&ctx.experiment_id, "PHASE-0001-hardware")?;
        let hw: crate::runtime::hardware::HardwareInfo = serde_json::from_value(hw_output.artifact)
            .map_err(|e| format!("Cannot parse hardware: {e}"))?;

        let vram_gb = hw.gpu.first().map(|g| g.vram_gb).unwrap_or(0.0);
        let ram_gb = hw.memory.available_gb;

        // Read model metadata from phase 4
        let model_output = store.load_artifact(&ctx.experiment_id, "PHASE-0004-gguf-inspection")?;
        let metadata: crate::runtime::model_intelligence::gguf::GgufMetadata =
            serde_json::from_value(model_output.artifact)
                .map_err(|e| format!("Cannot parse model metadata: {e}"))?;

        let context = metadata.context_length.unwrap_or(32768) as u64;
        let embed_dim = metadata.embedding_length.unwrap_or(2560) as u64;
        let heads = metadata.head_count.unwrap_or(32) as u64;
        let kv_heads = metadata.head_count_kv.unwrap_or(heads) as u64;
        let layers = metadata.block_count.unwrap_or(40) as u64;

        // Infer params and quantization from filename (centralized)
        let fname = Path::new(&self.model_path).file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let params_b = crate::runtime::model_intelligence::gguf::infer_params_from_filename(&fname);
        let quant_name = crate::runtime::model_intelligence::gguf::infer_quant_from_filename(&fname)
            .unwrap_or_else(|| "Q4_K_M".to_string());
        let quant_bits = if quant_name.contains("Q4") || quant_name.contains("Q5") { 4.0 }
            else if quant_name.contains("Q3") || quant_name.contains("IQ3") { 3.0 }
            else if quant_name.contains("Q2") || quant_name.contains("IQ2") { 2.0 }
            else if quant_name.contains("Q1") || quant_name.contains("IQ1") { 1.0 }
            else { 4.0 };

        output.record_event("Memory calculation", &format!(
            "Model: {:.0}B, Context: {context}, VRAM: {vram_gb}GB, RAM: {ram_gb:.0}GB",
            params_b
        ));

        use crate::runtime::model_intelligence::vram::{KvCacheType, VramCalculator};

        // Generate hypotheses for each memory strategy
        let strategies = vec![
            ("All VRAM FP16", KvCacheType::Fp16, false, layers),
            ("All VRAM Q8", KvCacheType::Q8, false, layers),
            ("Buendia Turbo3 RAM", KvCacheType::Turbo3, true, layers),
            ("All VRAM Q4", KvCacheType::Q4, false, layers),
        ];

        let mut hypotheses = Vec::new();
        for (name, kv_type, kv_ram, gpu_layers) in &strategies {
            let cfg = crate::runtime::model_intelligence::vram::MemoryConfig::new(params_b)
                .with_quant(quant_bits)
                .with_context(context as u64)
                .with_kv_type(*kv_type)
                .with_kv_in_ram(*kv_ram)
                .with_dims(embed_dim, heads, kv_heads)
                .with_layers(*gpu_layers, layers);

            let est = VramCalculator::estimate(&cfg, vram_gb, ram_gb);

            hypotheses.push(serde_json::json!({
                "strategy": name,
                "predicted": {
                    "fits": est.fits_in_vram,
                    "oom_risk": est.oom_risk,
                    "vram_gb": est.vram_total_gb,
                    "ram_gb": est.ram_total_gb,
                    "tokens_per_second": est.estimated_tok_s,
                },
                "config": {
                    "kv_cache_type": kv_type.name(),
                    "kv_in_ram": kv_ram,
                    "context": context,
                    "gpu_layers": gpu_layers,
                }
            }));

            output.record_event("Hypothesis generated", &format!(
                "{name}: predicted_fits={}, OOM_risk={:.0}%, VRAM={:.1}GB, RAM={:.1}GB",
                est.fits_in_vram, est.oom_risk * 100.0, est.vram_total_gb, est.ram_total_gb
            ));
        }

        output.artifact = serde_json::json!({
            "model_path": self.model_path,
            "hardware": { "vram_gb": vram_gb, "ram_gb": ram_gb },
            "model_metadata": {
                "context_declared": context,
                "embedding_dim": embed_dim,
                "layers": layers,
                "heads": heads,
                "kv_heads": kv_heads,
                "params_b": params_b,
                "quant_bits": quant_bits,
            },
            "hypotheses": hypotheses,
            "note": "These are HYPOTHESES only. ExecutionProbe (PHASE-0006) must verify them.",
        });

        output.status = PhaseStatus::Success;
        output.record_event("Memory hypothesis complete", &format!(
            "Generated {} hypotheses — ExecutionProbe needed to verify",
            hypotheses.len()
        ));

        Ok(output)
    }
}

// ═══════════════════════════════════════════════════════════════
// Phase Registry
// ═══════════════════════════════════════════════════════════════

use std::collections::HashMap;

pub struct PhaseRegistry {
    phases: HashMap<String, Box<dyn Phase>>,
}

impl PhaseRegistry {
    pub fn new() -> Self { Self { phases: HashMap::new() } }

    pub fn register(&mut self, phase: Box<dyn Phase>) {
        let id = phase.id().to_string();
        self.phases.insert(id, phase);
    }

    pub fn get(&self, id: &str) -> Option<&dyn Phase> {
        self.phases.get(id).map(|p| p.as_ref())
    }

    pub fn list(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.phases.keys().map(|k| k.as_str()).collect();
        ids.sort();
        ids
    }
}

/// Register all core phases.
/// PHASE-0004 and PHASE-0005 are registered as "needs-model" markers
/// and replaced with real implementations when --model is provided.
// ═══════════════════════════════════════════════════════════════
// PHASE-0006: Execution Laboratory
// ═══════════════════════════════════════════════════════════════

pub struct ExecutionLabPhase {
    pub runtime_path: String,
    pub model_path: String,
}

impl ExecutionLabPhase {
    pub fn new(runtime_path: &str, model_path: &str) -> Self {
        Self { runtime_path: runtime_path.to_string(), model_path: model_path.to_string() }
    }
}

impl Phase for ExecutionLabPhase {
    fn id(&self) -> &str { "PHASE-0006-execution-lab" }
    fn name(&self) -> &str { "Execution Laboratory" }
    fn question(&self) -> &str { "¿Funciona esta configuración realmente?" }
    fn description(&self) -> &str { "Ejecuta llama-server real con la configuración especificada y mide telemetría completa" }
    fn inputs(&self) -> Vec<&str> { vec!["PHASE-0001-hardware"] }

    fn execute(&self, ctx: &PhaseContext, _store: &dyn ArtifactStoreRead) -> Result<PhaseOutput, String> {
        let mut output = PhaseOutput::new(PhaseId::new(self.id()), &ctx.experiment_id);
        output.record_event("Execution started", &format!("Runtime: {}, Model: {}", self.runtime_path, self.model_path));

        let config = crate::runtime::execution_lab::ExecutionConfig::new(&self.runtime_path, &self.model_path);
        let report = match crate::runtime::execution_lab::ExecutionProbe::execute(&config, &ctx.experiment_id) {
            Ok(r) => r,
            Err(e) => return Err(format!("Execution failed: {e}")),
        };

        output.artifact = serde_json::to_value(&report)
            .map_err(|e| format!("Cannot serialize report: {e}"))?;
        output.metrics.add("load_time_s", report.telemetry.load_time_s, "s");
        output.metrics.add("first_token_ms", report.telemetry.first_token_ms, "ms");
        output.metrics.add("tokens_per_second", report.telemetry.tokens_per_second, "tok/s");
        output.metrics.add("vram_peak_gb", report.telemetry.vram_peak_gb, "GB");
        output.metrics.add("gpu_util_pct", report.telemetry.gpu_util_pct, "%");
        output.status = if report.success { PhaseStatus::Success } else { PhaseStatus::Failure("Execution failed".to_string()) };

        output.record_event("Execution complete", &format!(
            "Load: {:.1}s, Token: {:.1} tok/s, VRAM peak: {:.1}GB, GPU util: {:.0}%",
            report.telemetry.load_time_s,
            report.telemetry.tokens_per_second,
            report.telemetry.vram_peak_gb,
            report.telemetry.gpu_util_pct,
        ));

        Ok(output)
    }
}

// ═══════════════════════════════════════════════════════════════
// NeedsModelPhase — marker for model-dependent phases
// ═══════════════════════════════════════════════════════════════

/// Marker phase for phases that need a model path to execute.
/// Registered so `list_phases()` shows all available phases,
/// but execution fails with a clear error telling the user to use --model.
pub struct NeedsModelPhase {
    id: &'static str,
    name: &'static str,
    question: &'static str,
    description: &'static str,
}

impl NeedsModelPhase {
    pub fn new(id: &'static str, name: &'static str, question: &'static str, description: &'static str) -> Self {
        Self { id, name, question, description }
    }
}

impl Phase for NeedsModelPhase {
    fn id(&self) -> &str { self.id }
    fn name(&self) -> &str { self.name }
    fn question(&self) -> &str { self.question }
    fn description(&self) -> &str { self.description }
    fn execute(&self, _ctx: &PhaseContext, _store: &dyn ArtifactStoreRead) -> Result<PhaseOutput, String> {
        Err(format!("Phase {} requires --model <path> to execute.\nUse: ath phase run {} --model /path/to/model.gguf", self.id, self.id))
    }
}

// ═══════════════════════════════════════════════════════════════
// PHASE-0007: Runtime Fingerprint (Forensic Identity)
// ═══════════════════════════════════════════════════════════════

pub struct RuntimeFingerprintPhase {
    pub runtime_path: String,
}

impl RuntimeFingerprintPhase {
    pub fn new(runtime_path: &str) -> Self {
        Self { runtime_path: runtime_path.to_string() }
    }
}

impl Phase for RuntimeFingerprintPhase {
    fn id(&self) -> &str { "PHASE-0007-runtime-fingerprint" }
    fn name(&self) -> &str { "Runtime Fingerprint" }
    fn question(&self) -> &str { "¿Qué es realmente este runtime?" }
    fn description(&self) -> &str { "Identidad forense completa: SHA256, BuildID, .so hashes, ldd, dependencias, compilador" }
    fn inputs(&self) -> Vec<&str> { vec!["PHASE-0002-runtime-discovery"] }

    fn execute(&self, ctx: &PhaseContext, _store: &dyn ArtifactStoreRead) -> Result<PhaseOutput, String> {
        let mut output = PhaseOutput::new(PhaseId::new(self.id()), &ctx.experiment_id);
        output.record_event("Fingerprint started", &format!("Analyzing: {}", self.runtime_path));

        let path = std::path::Path::new(&self.runtime_path);
        let fingerprint = crate::runtime::runtime_fingerprint::fingerprint_runtime(path);

        output.artifact = serde_json::to_value(&fingerprint)
            .map_err(|e| format!("Serialization failed: {e}"))?;

        output.metrics.add("executable_size_mb", fingerprint.executable_size_bytes as f64 / 1_048_576.0, "MB");
        output.metrics.add("libraries_count", fingerprint.libraries.len() as f64, "count");
        output.metrics.add("ldd_entries_count", fingerprint.ldd_entries.len() as f64, "count");
        output.status = PhaseStatus::Success;

        output.record_event("Fingerprint complete", &format!(
            "{} ({}) — {} .so files, {} dependencies. SHA256: {}..",
            fingerprint.display_name,
            fingerprint.variant,
            fingerprint.libraries.len(),
            fingerprint.ldd_entries.len(),
            &fingerprint.executable_sha256[..16],
        ));

        Ok(output)
    }
}

// ═══════════════════════════════════════════════════════════════
// PHASE-0008: Capability Discovery (declared from --help)
// ═══════════════════════════════════════════════════════════════

pub struct CapabilityDiscoveryPhase {
    pub runtime_path: String,
}

impl CapabilityDiscoveryPhase {
    pub fn new(runtime_path: &str) -> Self {
        Self { runtime_path: runtime_path.to_string() }
    }
}

impl Phase for CapabilityDiscoveryPhase {
    fn id(&self) -> &str { "PHASE-0008-capability-discovery" }
    fn name(&self) -> &str { "Capability Discovery" }
    fn question(&self) -> &str { "¿Qué capacidades declara realmente este runtime?" }
    fn description(&self) -> &str { "Analiza --help para detectar capacidades declaradas (flash, cuda, bonsai, turbo3, etc.)" }
    fn inputs(&self) -> Vec<&str> { vec!["PHASE-0002-runtime-discovery"] }

    fn execute(&self, ctx: &PhaseContext, _store: &dyn ArtifactStoreRead) -> Result<PhaseOutput, String> {
        let mut output = PhaseOutput::new(PhaseId::new(self.id()), &ctx.experiment_id);
        output.record_event("Capability discovery started", &format!("Parsing --help for {}", self.runtime_path));

        let path = std::path::Path::new(&self.runtime_path);
        let display = path.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let report = crate::runtime::runtime_fingerprint::capability::CapabilityReport::from_declared(path, &display);

        output.artifact = serde_json::to_value(&report)
            .map_err(|e| format!("Serialization failed: {e}"))?;

        let count = report.declared.count();
        output.metrics.add("declared_capabilities", count as f64, "count");
        output.metrics.add("kv_cache_types", report.declared.kv_cache_types.len() as f64, "count");
        output.status = PhaseStatus::Success;

        output.record_event("Capability discovery complete", &format!(
            "Found {} declared capabilities, {} kv cache types — {} discrepancies",
            count,
            report.declared.kv_cache_types.len(),
            report.discrepancies.len(),
        ));

        Ok(output)
    }
}

// ═══════════════════════════════════════════════════════════════
// PHASE-0009: Parameter Normalization
// ═══════════════════════════════════════════════════════════════

pub struct ParameterNormalizationPhase;

impl Phase for ParameterNormalizationPhase {
    fn id(&self) -> &str { "PHASE-0009-parameter-normalization" }
    fn name(&self) -> &str { "Parameter Normalization" }
    fn question(&self) -> &str { "¿Cuál es el conjunto común de parámetros entre todos los runtimes?" }
    fn description(&self) -> &str { "Encuentra la intersección de parámetros compatibles entre todos los runtimes detectados" }
    fn inputs(&self) -> Vec<&str> { vec!["PHASE-0002-runtime-discovery"] }

    fn execute(&self, ctx: &PhaseContext, _store: &dyn ArtifactStoreRead) -> Result<PhaseOutput, String> {
        let mut output = PhaseOutput::new(PhaseId::new(self.id()), &ctx.experiment_id);
        output.record_event("Normalization started", "Analyzing all runtime parameter sets");

        // Discover runtimes
        let runtimes = crate::runtime::runtime_discovery::RuntimeProber::probe_all();

        // Detect parameter sets for each runtime
        use crate::runtime::runtime_fingerprint::normalization::RuntimeParameterSet;
        let mut runtime_sets = Vec::new();
        for rt in &runtimes {
            let path = std::path::Path::new(&rt.binary_path);
            if path.exists() {
                runtime_sets.push(RuntimeParameterSet::detect(path, &rt.display_name));
            }
        }

        // Detect hardware for context
        let hw = crate::runtime::hardware::detect_hardware();
        let hardware_json = serde_json::to_value(&hw).unwrap_or_default();

        // Compute normalization
        let normalized = crate::runtime::runtime_fingerprint::normalization::NormalizedParameterSet::compute(
            runtime_sets, hardware_json
        );

        output.artifact = serde_json::to_value(&normalized)
            .map_err(|e| format!("Serialization failed: {e}"))?;

        output.metrics.add("runtimes_analyzed", normalized.runtime_count() as f64, "count");
        output.metrics.add("common_parameters", normalized.common_count() as f64, "count");
        output.metrics.add("union_parameters", normalized.union_parameters.len() as f64, "count");

        if normalized.is_comparable() {
            output.status = PhaseStatus::Success;
            output.record_event("Normalization complete", &format!(
                "{} runtimes, {} common parameters — runtimes are COMPARABLE",
                normalized.runtime_count(),
                normalized.common_count(),
            ));
        } else {
            output.status = PhaseStatus::Success; // Still succeeds, just warns
            output.record_event("Normalization complete", &format!(
                "{} runtimes, {} common parameters — WARNING: runtimes may not be directly comparable",
                normalized.runtime_count(),
                normalized.common_count(),
            ));
        }

        Ok(output)
    }
}

// ═══════════════════════════════════════════════════════════════
// PHASE-0010: Output Validation (ProbeValidation)
// ═══════════════════════════════════════════════════════════════

pub struct OutputValidationPhase;

impl Phase for OutputValidationPhase {
    fn id(&self) -> &str { "PHASE-0010-output-validation" }
    fn name(&self) -> &str { "Output Validation" }
    fn question(&self) -> &str { "¿La salida obtenida es válida?" }
    fn description(&self) -> &str { "Verifica calidad de la salida: UTF-8 válido, no vacío, sin bucles, sin basura, diversidad" }
    fn inputs(&self) -> Vec<&str> { vec!["PHASE-0006-execution-lab"] }

    fn execute(&self, ctx: &PhaseContext, store: &dyn ArtifactStoreRead) -> Result<PhaseOutput, String> {
        let mut output = PhaseOutput::new(PhaseId::new(self.id()), &ctx.experiment_id);
        output.record_event("Validation started", "Reading execution output from PHASE-0006");

        // Try to load PHASE-0006 artifact to get the generated text
        let phase6 = match store.load_artifact(&ctx.experiment_id, "PHASE-0006-execution-lab") {
            Ok(p) => p,
            Err(_) => {
                // No execution lab data — validate this as a standalone instrument
                // with a dummy result
                let result = crate::runtime::runtime_fingerprint::validation::ValidationResult {
                    passed: true,
                    non_empty: true,
                    utf8_valid: true,
                    stopped_correctly: true,
                    repetition_score: 0.0,
                    infinite_loop_detected: false,
                    max_consecutive_repeats: 0,
                    diversity_score: 1.0,
                    token_count: 0,
                    line_count: 0,
                    corrupt_output: false,
                    contains_garbage: false,
                    warnings: vec!["No execution data to validate (standalone instrument)".to_string()],
                    output_snippet: "(no output)".to_string(),
                };
                let artifact = serde_json::to_value(&result)
                    .map_err(|e| format!("Serialization failed: {e}"))?;
                output.artifact = artifact;
                output.status = PhaseStatus::Skipped("No PHASE-0006 data available".to_string());
                return Ok(output);
            }
        };

        // Extract stdout_log or content from phase6 artifact
        let content = phase6.artifact["telemetry"]["stdout_log"]
            .as_str()
            .unwrap_or("(binary output)")
            .to_string();

        let success = matches!(phase6.status, PhaseStatus::Success);

        // Validate output
        let result = crate::runtime::runtime_fingerprint::validation::ValidationResult::validate(
            &content, 100, success
        );
        let score = crate::runtime::runtime_fingerprint::validation::quality_score(&result);

        // Also analyze stderr from the execution
        let stderr = phase6.artifact["telemetry"]["stderr_log"]
            .as_str()
            .unwrap_or("");
        let stderr_analysis = crate::runtime::runtime_fingerprint::validation::StderrAnalysis::analyze(stderr);

        // Build combined artifact
        let combined = serde_json::json!({
            "validation": result,
            "quality_score": (score * 1000.0).round() / 1000.0,
            "stderr_analysis": stderr_analysis,
        });

        output.artifact = combined;
        output.metrics.add("quality_score", score, "score");
        output.metrics.add("repetition_score", result.repetition_score, "score");
        output.metrics.add("diversity_score", result.diversity_score, "score");
        output.metrics.add("token_count", result.token_count as f64, "tokens");
        output.metrics.add("warnings_count", result.warnings.len() as f64, "count");

        if result.passed && score > 0.5 {
            output.status = PhaseStatus::Success;
        } else {
            let reasons: Vec<String> = result.warnings.iter().take(3).cloned().collect();
            output.status = PhaseStatus::Failure(
                format!("Validation failed — quality score {:.2}. Issues: {}", score, reasons.join("; "))
            );
        }

        output.record_event("Validation complete", &format!(
            "Quality score: {:.2}, tokens: {}, warnings: {}",
            score, result.token_count, result.warnings.len()
        ));

        Ok(output)
    }
}

// ═══════════════════════════════════════════════════════════════
// PHASE-0011: Experiment Validation
// ═══════════════════════════════════════════════════════════════

pub struct ExperimentValidationPhase;

impl Phase for ExperimentValidationPhase {
    fn id(&self) -> &str { "PHASE-0011-experiment-validation" }
    fn name(&self) -> &str { "Experiment Validation" }
    fn question(&self) -> &str { "¿Este experimento merece ejecutarse?" }
    fn description(&self) -> &str { "Valida modelo, runtime, compatibilidad, parámetros y recursos. NO ejecuta nada, solo valida." }
    fn inputs(&self) -> Vec<&str> { vec!["PHASE-0001-hardware", "PHASE-0002-runtime-discovery"] }

    fn execute(&self, ctx: &PhaseContext, _store: &dyn ArtifactStoreRead) -> Result<PhaseOutput, String> {
        let mut output = PhaseOutput::new(PhaseId::new(self.id()), &ctx.experiment_id);
        output.record_event("Validation started", "Checking experiment validity");

        // Use a default config — in practice, this would be passed via CLI
        // For standalone phase, validate the system state
        use crate::runtime::runtime_fingerprint::experiment_validation::{ExperimentConfig, ExperimentValidator};

        // Discover the first available runtime and model for a sanity check
        let runtimes = crate::runtime::runtime_discovery::RuntimeProber::probe_all();
        let runtime_path = runtimes.first().map(|r| r.binary_path.clone()).unwrap_or_else(|| "none".to_string());
        let model = crate::runtime::find_model(None).ok();
        let model_path = model.map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "none".to_string());

        let config = ExperimentConfig::new(&model_path, &runtime_path);
        let result = ExperimentValidator::validate(&config);

        // Store negative evidence if validation failed
        if !result.passed {
            let evidence = crate::runtime::runtime_fingerprint::experiment_validation::NegativeEvidence::from_validation(&config, &result);
            let state_dir = std::path::Path::new(".state");
            let mut store = crate::runtime::runtime_fingerprint::evidence_store::EvidenceStore::load(state_dir);
            store.store_negative(&evidence).ok();
            output.record_event("Negative evidence stored", &format!(
                "Stored {} — {}", evidence.id, evidence.reasons.join("; ")
            ));
        }

        output.artifact = serde_json::to_value(&result)
            .map_err(|e| format!("Serialization failed: {e}"))?;

        output.metrics.add("checks_total", result.summary.total as f64, "count");
        output.metrics.add("checks_passed", result.summary.passed as f64, "count");
        output.metrics.add("critical_failures", result.summary.critical_failures as f64, "count");
        output.metrics.add("warnings", result.summary.warnings as f64, "count");
        output.metrics.add("confidence", result.summary.confidence, "score");

        if result.passed {
            output.status = PhaseStatus::Success;
            output.record_event("Validation passed", &format!(
                "Experiment CAN run — {} checks passed, confidence: {:.1}%",
                result.summary.passed, result.summary.confidence * 100.0
            ));
        } else {
            output.status = PhaseStatus::Failure(format!(
                "Experiment CANNOT run — {} critical failures, {} warnings, confidence: {:.1}%",
                result.summary.critical_failures, result.summary.warnings, result.summary.confidence * 100.0
            ));
            output.record_event("Negative evidence generated", &format!(
                "Experiment rejected: {} critical, {} warning — reasons: {}",
                result.summary.critical_failures,
                result.summary.warnings,
                result.checks.iter()
                    .filter(|c| !c.passed)
                    .map(|c| format!("[{}] {}", c.severity, c.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        Ok(output)
    }
}

/// Register all core phases.
pub fn register_all_phases(registry: &mut PhaseRegistry) {
    registry.register(Box::new(HardwarePhase));
    registry.register(Box::new(RuntimeDiscoveryPhase));
    registry.register(Box::new(RuntimeCapabilitiesPhase));
    registry.register(Box::new(NeedsModelPhase::new(
        "PHASE-0004-gguf-inspection",
        "GGUF Inspection",
        "¿Qué dice realmente este modelo?",
        "Requiere --model <path> para leer metadata del archivo GGUF"
    )));
    registry.register(Box::new(NeedsModelPhase::new(
        "PHASE-0005-memory-hypothesis",
        "Memory Hypothesis",
        "¿Qué configuraciones parecen posibles?",
        "Requiere --model <path> para generar hipótesis de memoria"
    )));
    registry.register(Box::new(NeedsModelPhase::new(
        "PHASE-0006-execution-lab",
        "Execution Laboratory",
        "¿Funciona esta configuración realmente?",
        "Requiere --model <path> y --runtime <path> para ejecutar inferencia real y medir telemetría"
    )));
    registry.register(Box::new(NeedsModelPhase::new(
        "PHASE-0007-runtime-fingerprint",
        "Runtime Fingerprint",
        "¿Qué es realmente este runtime?",
        "Requiere --runtime <path> para identidad forense completa"
    )));
    registry.register(Box::new(NeedsModelPhase::new(
        "PHASE-0008-capability-discovery",
        "Capability Discovery",
        "¿Qué capacidades declara realmente este runtime?",
        "Requiere --runtime <path> para analizar --help"
    )));
    registry.register(Box::new(ParameterNormalizationPhase));
    registry.register(Box::new(OutputValidationPhase));
    registry.register(Box::new(ExperimentValidationPhase));
}
