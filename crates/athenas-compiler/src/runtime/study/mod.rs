/// Study System — Declarative scientific programs for Athena.
///
/// Un "Study" es un programa científico completo, declarativo en YAML.
/// Athena lee el estudio, descubre automáticamente las fases necesarias,
/// construye el DAG de dependencias y ejecuta todo el programa.
///
/// ## Comandos
/// ```bash
/// ath study list          # List all available studies
/// ath study SP-005        # Run Runtime Health Check
/// ath study PC-001        # Run Runtime Comparison
/// ```
///
/// ## Diferencia con Experiment Queue
/// - Experiment Queue: cola de experimentos individuales
/// - Study Queue: estudios completos conteniendo campañas, experimentos y ejecuciones
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::SystemTime;

use crate::runtime::phase::core::Phase;

// ═══════════════════════════════════════════════════════════════
// Study Definition
// ═══════════════════════════════════════════════════════════════

/// A complete scientific program definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Study {
    /// Unique study ID (e.g., "SP-005", "PC-001")
    pub id: String,

    /// Scientific question this study answers
    pub question: String,

    /// Human-readable name
    pub name: String,

    /// Description
    pub description: String,

    /// List of phases required for this study (in order)
    /// Athena discovers dependencies automatically.
    pub phase_ids: Vec<String>,

    /// For each runtime, how many repetitions
    pub repetitions: u32,

    /// Default context to use
    pub default_context: u64,

    /// Default max tokens for generation
    pub default_max_tokens: u64,

    /// Model path (optional — auto-discovered if omitted)
    pub model_path: Option<String>,

    /// Runtime paths to test (optional — all discovered if omitted)
    pub runtimes: Vec<String>,

    /// Optimization objective
    pub objective: String,

    /// Success criteria
    pub success_criteria: Vec<String>,

    /// Whether to validate experiments before running (PHASE-0011)
    pub validate_before_execution: bool,

    /// Whether to store evidence (negative and positive)
    pub store_evidence: bool,

    /// Status of this study
    pub status: StudyStatus,
}

/// Status of a study execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StudyStatus {
    /// Not started yet
    Pending,
    /// Currently running
    Running,
    /// Completed successfully
    Completed,
    /// Completed with failures
    Failed(String),
    /// Cancelled
    Cancelled,
}

impl Study {
    /// Create a new study definition
    pub fn new(id: &str, question: &str, phase_ids: Vec<&str>) -> Self {
        Self {
            id: id.to_string(),
            question: question.to_string(),
            name: id.to_string(),
            description: String::new(),
            phase_ids: phase_ids.iter().map(|s| s.to_string()).collect(),
            repetitions: 1,
            default_context: 32768,
            default_max_tokens: 100,
            model_path: None,
            runtimes: Vec::new(),
            objective: "maximum_quality".to_string(),
            success_criteria: Vec::new(),
            validate_before_execution: true,
            store_evidence: true,
            status: StudyStatus::Pending,
        }
    }

    /// Build the DAG of phases (topological sort based on dependencies)
    pub fn build_dag(&self, registry: &crate::runtime::phase::phases::PhaseRegistry) -> Vec<String> {
        let mut ordered = Vec::new();
        let mut visited = HashSet::new();

        for phase_id in &self.phase_ids {
            let deps = self.collect_dependencies(phase_id, registry, &mut visited);
            for dep in deps {
                if !ordered.contains(&dep) {
                    ordered.push(dep);
                }
            }
            if !ordered.contains(phase_id) {
                ordered.push(phase_id.clone());
            }
        }

        ordered
    }

    /// Collect all dependencies for a phase (recursive)
    fn collect_dependencies(
        &self,
        phase_id: &str,
        registry: &crate::runtime::phase::phases::PhaseRegistry,
        visited: &mut HashSet<String>,
    ) -> Vec<String> {
        let mut deps = Vec::new();
        if visited.contains(phase_id) {
            return deps;
        }
        visited.insert(phase_id.to_string());

        if let Some(phase) = registry.get(phase_id) {
            for input in phase.inputs() {
                let input_deps = self.collect_dependencies(input, registry, visited);
                for d in input_deps {
                    if !deps.contains(&d) {
                        deps.push(d);
                    }
                }
                if !deps.contains(&input.to_string()) {
                    deps.push(input.to_string());
                }
            }
        }

        deps
    }

    /// Run the study — executes all phases in order, auto-discovers runtimes and models
    pub fn execute(&self, registry: &crate::runtime::phase::phases::PhaseRegistry) -> Result<StudyReport, String> {
        let dag = self.build_dag(registry);
        let start = SystemTime::now();
        let mut report = StudyReport::new(&self.id);

        let state_dir = Path::new(".state");
        let exp_id = format!("STUDY-{}-{}", self.id, 
            start.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs());

        eprintln!("\n  📖 Study: {}", self.id);
        eprintln!("  ❓ Question: {}", self.question);
        eprintln!("  📋 Phases: {} (DAG: {} total including deps)", self.phase_ids.len(), dag.len());
        eprintln!();

        // Auto-discover runtimes and model if not specified
        let model = self.model_path.clone().or_else(|| {
            crate::runtime::find_model(None).ok().map(|p| p.to_string_lossy().to_string())
        });
        let runtimes = if self.runtimes.is_empty() {
            crate::runtime::runtime_discovery::RuntimeProber::probe_all()
                .into_iter()
                .map(|r| r.binary_path.clone())
                .collect()
        } else {
            self.runtimes.clone()
        };

        eprintln!("  🔍 Auto-discovered: {} runtime(s), model: {:?}", runtimes.len(), model);
        eprintln!();

        // Create artifact store
        let mut store = crate::runtime::phase::ArtifactStore::new(state_dir);
        let ctx = crate::runtime::phase::PhaseContext::new(&exp_id, state_dir);

        // Execute each phase in DAG order
        for phase_id in &dag {
            // Check if already executed
            if store.phase_exists(&exp_id, phase_id) {
                eprintln!("  ⏭  {phase_id} (cached)");
                report.phases_completed += 1;
                continue;
            }

            // Create specialized phase if needed
            let phase: Box<dyn Phase> = match phase_id.as_str() {
                "PHASE-0001-hardware" => {
                    Box::new(crate::runtime::phase::phases::HardwarePhase)
                }
                "PHASE-0002-runtime-discovery" => {
                    Box::new(crate::runtime::phase::phases::RuntimeDiscoveryPhase)
                }
                "PHASE-0003-runtime-capabilities" => {
                    Box::new(crate::runtime::phase::phases::RuntimeCapabilitiesPhase)
                }
                "PHASE-0004-gguf-inspection" => {
                    let mp = model.clone().ok_or_else(|| format!("Model required for {phase_id}"))?;
                    Box::new(crate::runtime::phase::phases::GgufInspectionPhase::new(&mp))
                }
                "PHASE-0005-memory-hypothesis" => {
                    let mp = model.clone().ok_or_else(|| format!("Model required for {phase_id}"))?;
                    Box::new(crate::runtime::phase::phases::MemoryHypothesisPhase::new(&mp))
                }
                "PHASE-0006-execution-lab" => {
                    let rt = runtimes.first().ok_or_else(|| format!("Runtime required for {phase_id}"))?;
                    let mp = model.clone().ok_or_else(|| format!("Model required for {phase_id}"))?;
                    Box::new(crate::runtime::phase::phases::ExecutionLabPhase::new(rt, &mp))
                }
                "PHASE-0007-runtime-fingerprint" => {
                    let rt = runtimes.first().ok_or_else(|| format!("Runtime required for {phase_id}"))?;
                    Box::new(crate::runtime::phase::phases::RuntimeFingerprintPhase::new(rt))
                }
                "PHASE-0008-capability-discovery" => {
                    let rt = runtimes.first().ok_or_else(|| format!("Runtime required for {phase_id}"))?;
                    Box::new(crate::runtime::phase::phases::CapabilityDiscoveryPhase::new(rt))
                }
                "PHASE-0009-parameter-normalization" => {
                    Box::new(crate::runtime::phase::phases::ParameterNormalizationPhase)
                }
                "PHASE-0010-output-validation" => {
                    Box::new(crate::runtime::phase::phases::OutputValidationPhase)
                }
                "PHASE-0011-experiment-validation" => {
                    Box::new(crate::runtime::phase::phases::ExperimentValidationPhase)
                }
                _ => {
                    eprintln!("  ⚠ Unknown or unsupported phase in study: {phase_id} — skipping");
                    report.errors.push(format!("Unknown phase in study: {phase_id}"));
                    continue;
                }
            };

            // Run validation before execution if enabled
            if self.validate_before_execution && phase_id == "PHASE-0006-execution-lab" {
                if store.phase_exists(&exp_id, "PHASE-0011-experiment-validation") {
                    eprintln!("  ✅ Experiment already validated");
                } else if registry.get("PHASE-0011-experiment-validation").is_some() {
                    // Create validation phase directly
                    let val_phase = crate::runtime::phase::phases::ExperimentValidationPhase;
                    let val_ctx = crate::runtime::phase::PhaseContext::new(&exp_id, state_dir);
                    match val_phase.execute(&val_ctx, &store) {
                        Ok(val_output) => {
                            if matches!(val_output.status, crate::runtime::phase::core::PhaseStatus::Success) {
                                eprintln!("  ✅ Experiment validation passed");
                            } else {
                                eprintln!("  ⚠ Experiment validation had issues");
                            }
                            store.save_phase(&exp_id, &val_output).ok();
                        }
                        Err(e) => {
                            eprintln!("  ⚠ Validation error (non-fatal): {e}");
                        }
                    }
                }
            }

            // Execute the phase
            eprint!("  🔬 {phase_id}...");
            let phase_start = std::time::Instant::now();
            match phase.execute(&ctx, &store) {
                Ok(mut output) => {
                    output.duration_ms = phase_start.elapsed().as_millis() as u64;
                    store.save_phase(&exp_id, &output)
                        .map_err(|e| format!("Cannot save phase {phase_id}: {e}"))?;

                    let status_str = format!("{:?}", output.status);
                    if status_str.contains("Success") {
                        eprintln!(" ✅ ({}ms)", output.duration_ms);
                        report.phases_completed += 1;
                    } else if status_str.contains("Skipped") {
                        eprintln!(" ⏭ (skipped: {})", status_str);
                        report.phases_skipped += 1;
                    } else {
                        eprintln!(" ❌ ({})", status_str);
                        report.errors.push(format!("{phase_id}: {status_str}"));
                    }
                }
                Err(e) => {
                    eprintln!(" ❌ ({e})");
                    report.errors.push(format!("{phase_id}: {e}"));
                }
            }
        }

        // Complete report
        if let Ok(duration) = start.elapsed() {
            report.duration_seconds = duration.as_secs_f64();
        }
        report.total_phases = dag.len();
        report.runtimes_discovered = runtimes.len();
        report.model_used = model;

        if report.errors.is_empty() {
            report.status = "completed".to_string();
            eprintln!("\n  ✅ Study {} completed successfully!", self.id);
        } else {
            report.status = "completed_with_errors".to_string();
            eprintln!("\n  ⚠ Study {} completed with {} error(s)", self.id, report.errors.len());
        }

        eprintln!("  📊 Phases: {}/{} completed, {} skipped, {} errors",
            report.phases_completed, report.total_phases, report.phases_skipped, report.errors.len());
        eprintln!("  ⏱ Duration: {:.0}s", report.duration_seconds);
        eprintln!("  📁 Artifacts: .state/experiments/{exp_id}/");

        Ok(report)
    }
}

// ═══════════════════════════════════════════════════════════════
// Study Report
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyReport {
    pub study_id: String,
    pub status: String,
    pub total_phases: usize,
    pub phases_completed: usize,
    pub phases_skipped: usize,
    pub errors: Vec<String>,
    pub duration_seconds: f64,
    pub runtimes_discovered: usize,
    pub model_used: Option<String>,
}

impl StudyReport {
    pub fn new(study_id: &str) -> Self {
        Self {
            study_id: study_id.to_string(),
            status: "pending".to_string(),
            total_phases: 0,
            phases_completed: 0,
            phases_skipped: 0,
            errors: Vec::new(),
            duration_seconds: 0.0,
            runtimes_discovered: 0,
            model_used: None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Study Registry — Built-in studies
// ═══════════════════════════════════════════════════════════════

/// Returns the set of built-in studies.
pub fn built_in_studies() -> HashMap<String, Study> {
    let mut studies = HashMap::new();

    // SC-000: Laboratory Validation
    studies.insert("SC-000".to_string(), Study {
        id: "SC-000".to_string(),
        question: "¿El laboratorio de Athena funciona correctamente de extremo a extremo?".to_string(),
        name: "Laboratory Validation".to_string(),
        description: "Valida que Athena puede ejecutar inferencia real de extremo a extremo: carga, generacion, descarga, artefactos, telemetria y quality score. Usa el primer runtime y modelo auto-descubiertos.".to_string(),
        phase_ids: vec![
            "PHASE-0001-hardware".to_string(),
            "PHASE-0002-runtime-discovery".to_string(),
            "PHASE-0007-runtime-fingerprint".to_string(),
            "PHASE-0008-capability-discovery".to_string(),
            "PHASE-0004-gguf-inspection".to_string(),
            "PHASE-0005-memory-hypothesis".to_string(),
            "PHASE-0011-experiment-validation".to_string(),
            "PHASE-0006-execution-lab".to_string(),
            "PHASE-0010-output-validation".to_string(),
        ],
        repetitions: 1,
        default_context: 32768,
        default_max_tokens: 100,
        model_path: None,
        runtimes: Vec::new(),
        objective: "validation".to_string(),
        success_criteria: vec![
            "no_crash".to_string(),
            "no_oom".to_string(),
            "telemetry_complete".to_string(),
            "output_valid".to_string(),
        ],
        validate_before_execution: true,
        store_evidence: true,
        status: StudyStatus::Pending,
    });

    // SP-005: Runtime Health Check
    studies.insert("SP-005".to_string(), Study {
        id: "SP-005".to_string(),
        question: "¿Los runtimes están sanos?".to_string(),
        name: "Runtime Health Check".to_string(),
        description: "Verifica que todos los runtimes detectados carguen correctamente, generen tokens y finalicen sin errores. Es la puerta de entrada del laboratorio.".to_string(),
        phase_ids: vec![
            "PHASE-0001-hardware".to_string(),
            "PHASE-0002-runtime-discovery".to_string(),
            "PHASE-0007-runtime-fingerprint".to_string(),
            "PHASE-0008-capability-discovery".to_string(),
            "PHASE-0011-experiment-validation".to_string(),
        ],
        repetitions: 1,
        default_context: 32768,
        default_max_tokens: 30,
        model_path: None,
        runtimes: Vec::new(),
        objective: "health_check".to_string(),
        success_criteria: vec![
            "no_crash".to_string(),
            "no_oom".to_string(),
            "output_valid".to_string(),
        ],
        validate_before_execution: true,
        store_evidence: true,
        status: StudyStatus::Pending,
    });

    // PC-001: Runtime Comparison
    studies.insert("PC-001".to_string(), Study {
        id: "PC-001".to_string(),
        question: "¿Cuál implementación de runtime ofrece el mejor comportamiento bajo las mismas condiciones experimentales?".to_string(),
        name: "Runtime Comparison".to_string(),
        description: "Compara todos los runtimes detectados (Official, TurboQuant, PrismML) bajo exactamente los mismos parametros. Cada runtime ejecuta 5 repeticiones. Mide load time, first token, TPS, VRAM, RAM y estabilidad.".to_string(),
        phase_ids: vec![
            "PHASE-0001-hardware".to_string(),
            "PHASE-0002-runtime-discovery".to_string(),
            "PHASE-0003-runtime-capabilities".to_string(),
            "PHASE-0004-gguf-inspection".to_string(),
            "PHASE-0007-runtime-fingerprint".to_string(),
            "PHASE-0008-capability-discovery".to_string(),
            "PHASE-0009-parameter-normalization".to_string(),
            "PHASE-0011-experiment-validation".to_string(),
            "PHASE-0006-execution-lab".to_string(),
            "PHASE-0010-output-validation".to_string(),
        ],
        repetitions: 5,
        default_context: 32768,
        default_max_tokens: 100,
        model_path: None,
        runtimes: Vec::new(),
        objective: "maximum_quality".to_string(),
        success_criteria: vec![
            "no_crash".to_string(),
            "no_oom".to_string(),
            "output_valid".to_string(),
            "confidence > 0.95".to_string(),
        ],
        validate_before_execution: true,
        store_evidence: true,
        status: StudyStatus::Pending,
    });

    studies
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_built_in_studies_exist() {
        let studies = built_in_studies();
        assert!(studies.contains_key("SP-005"));
        assert!(studies.contains_key("PC-001"));
    }

    #[test]
    fn test_study_dag_building() {
        use crate::runtime::phase::phases::PhaseRegistry;
        use crate::runtime::phase::phases::register_all_phases;
        let mut registry = PhaseRegistry::new();
        register_all_phases(&mut registry);

        let studies = built_in_studies();
        let study = studies.get("SP-005").unwrap();
        let dag = study.build_dag(&registry);
        assert!(!dag.is_empty());
        // PHASE-0001 should come before PHASE-0002 since 0002 depends on 0001
        let pos_0001 = dag.iter().position(|p| p == "PHASE-0001-hardware");
        let pos_0002 = dag.iter().position(|p| p == "PHASE-0002-runtime-discovery");
        if let (Some(p1), Some(p2)) = (pos_0001, pos_0002) {
            assert!(p1 < p2, "PHASE-0001 must come before PHASE-0002 in DAG");
        }
    }

    #[test]
    fn test_study_new() {
        let study = Study::new("TEST-001", "Test question", vec!["PHASE-0001", "PHASE-0002"]);
        assert_eq!(study.id, "TEST-001");
        assert_eq!(study.phase_ids.len(), 2);
        assert!(study.validate_before_execution);
    }
}
