/// Campaign Engine — Ejecución autónoma de estudios científicos.
///
/// Un Campaign es un estudio en ejecución:
/// - Itera sobre cada runtime
/// - Ejecuta cada runtime N repeticiones
/// - Escribe automáticamente a la Knowledge Base
/// - Produce un CampaignReport al finalizar
///
/// ## Jerarquía
/// Study → Campaign → Execution → Evidence
///
/// Un Study define el protocolo.
/// Un Campaign ejecuta el protocolo.
/// Una Execution es una iteración del protocolo.
/// Evidence es lo que queda después.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;

use crate::runtime::knowledge_base::{AnswerRevision, KnowledgeBase};
use crate::runtime::phase::core::ArtifactStoreRead;
use crate::runtime::study::{Study, StudyReport};
use crate::runtime::runtime_fingerprint::validation::{ValidationResult, quality_score as compute_quality_score};

// ═══════════════════════════════════════════════════════════════
// Campaign
// ═══════════════════════════════════════════════════════════════

/// Una campaña científica en ejecución.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    /// ID único (e.g., "CAMP-PC-001-20260718")
    pub id: String,

    /// El estudio que esta campaña ejecuta
    pub study_id: String,

    /// Runtimes a evaluar
    pub runtimes: Vec<String>,

    /// Repeticiones por runtime
    pub repetitions: u32,

    /// Modelo a usar
    pub model_path: String,

    /// Resultados por runtime (runtime_path → lista de reportes)
    pub results: HashMap<String, Vec<StudyReport>>,

    /// Estado actual
    pub status: CampaignStatus,

    /// Cuando empezó
    pub started_at: u64,

    /// IDs de experimentos en la cola
    pub experiment_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CampaignStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}

/// Reporte final de una campaña
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignReport {
    pub campaign_id: String,
    pub study_id: String,
    pub status: String,
    pub total_runtimes: usize,
    pub total_repetitions: u32,
    pub completed_executions: u32,
    pub failed_executions: u32,
    pub duration_seconds: f64,
    pub runtimes_tested: Vec<String>,
    pub model_used: String,
    pub answer_revision: Option<AnswerRevision>,
    pub errors: Vec<String>,
}

impl Campaign {
    /// Crear una nueva campaña desde un estudio
    pub fn from_study(study: &Study, model_path: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let runtimes = if study.runtimes.is_empty() {
            crate::runtime::runtime_discovery::RuntimeProber::probe_all()
                .into_iter()
                .filter(|r| r.binary_path.contains("llama-server"))
                .map(|r| r.binary_path.clone())
                .collect()
        } else {
            study.runtimes.clone()
        };

        Self {
            id: format!("CAMP-{}-{}", study.id, now),
            study_id: study.id.clone(),
            runtimes,
            repetitions: study.repetitions,
            model_path: model_path.to_string(),
            results: HashMap::new(),
            status: CampaignStatus::Pending,
            started_at: now,
            experiment_ids: Vec::new(),
        }
    }

    /// Ejecutar la campaña completa (todos los runtimes × repeticiones)
    pub fn execute(&mut self, registry: &crate::runtime::phase::phases::PhaseRegistry) -> Result<CampaignReport, String> {
        self.status = CampaignStatus::Running;
        let start = SystemTime::now();
        let mut completed = 0u32;
        let mut failed = 0u32;
        let mut all_errors = Vec::new();
        let mut all_metrics: HashMap<String, Vec<HashMap<String, f64>>> = HashMap::new();

        eprintln!("\n╔══════════════════════════════════════════╗");
        eprintln!("║     Athena Campaign Engine v0.1.0        ║");
        eprintln!("╚══════════════════════════════════════════╝");
        eprintln!();
        eprintln!("  📖 Study: {}", self.study_id);
        eprintln!("  📦 Model: {}", self.model_path);
        eprintln!("  🏃 Runtimes: {} × {} reps = {} executions",
            self.runtimes.len(), self.repetitions, self.runtimes.len() * self.repetitions as usize);
        eprintln!();

        // Obtener el estudio
        let studies = crate::runtime::study::built_in_studies();
        let study = studies.get(&self.study_id)
            .ok_or_else(|| format!("Study '{}' not found", self.study_id))?;

        // Modificar estudio para usar modelo y runtimes específicos
        let mut campaign_study = study.clone();
        campaign_study.model_path = Some(self.model_path.clone());
        campaign_study.store_evidence = true;

        // Para cada runtime
        for (rt_idx, runtime_path) in self.runtimes.iter().enumerate() {
            eprintln!("\n  ── Runtime {}/{}: {} ──", rt_idx + 1, self.runtimes.len(), runtime_path);
            eprintln!();

            // Por ahora solo usamos el primer runtime configurado
            // En el futuro iteraremos sobre todos
            campaign_study.runtimes = vec![runtime_path.clone()];

            let mut runtime_metrics = Vec::new();
            let mut saturated = false;

            for rep in 0..self.repetitions {
                if saturated {
                    eprintln!("    ⏭ Repetición {}/{} (saturated — CV < 5%)", rep + 1, self.repetitions);
                    break;
                }

                eprintln!("    Repetición {}/{}", rep + 1, self.repetitions);
                let rep_report = match campaign_study.execute(registry) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("    ❌ Campaign execution error: {e}");
                        failed += 1;
                        all_errors.push(format!("{runtime_path} rep {rep}: {e}"));
                        continue;
                    }
                };

                completed += 1;

                // Extraer métricas del reporte
                let mut metrics = HashMap::new();
                let state_dir = Path::new(".state");
                let store = crate::runtime::phase::ArtifactStore::new(state_dir);

                // Buscar el experiment ID más reciente
                let exp_dir = state_dir.join("experiments");
                if let Ok(entries) = std::fs::read_dir(&exp_dir) {
                    let mut exp_ids: Vec<String> = entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().is_dir())
                        .filter_map(|e| e.file_name().into_string().ok())
                        .filter(|name| name.starts_with("STUDY-"))
                        .collect();
                    exp_ids.sort();
                    exp_ids.reverse();

                    if let Some(latest_exp) = exp_ids.first() {
                        if let Ok(phase6) = store.load_artifact(latest_exp, "PHASE-0006-execution-lab") {
                            if let Some(tps) = phase6.artifact["telemetry"]["tokens_per_second"].as_f64() {
                                metrics.insert("tokens_per_second".to_string(), tps);
                            }
                            if let Some(vram) = phase6.artifact["telemetry"]["vram_peak_gb"].as_f64() {
                                metrics.insert("vram_peak_gb".to_string(), vram);
                            }
                            if let Some(load) = phase6.artifact["telemetry"]["load_time_s"].as_f64() {
                                metrics.insert("load_time_s".to_string(), load);
                            }
                            if let Some(first) = phase6.artifact["telemetry"]["first_token_ms"].as_f64() {
                                metrics.insert("first_token_ms".to_string(), first);
                            }
                            if let Some(gpu) = phase6.artifact["telemetry"]["gpu_util_pct"].as_f64() {
                                metrics.insert("gpu_util_pct".to_string(), gpu);
                            }

                            // ── Quality metrics from output validation ──
                            // stdout_log contiene la respuesta JSON completa de llama-server.
                            // Extraemos el campo "content" antes de validar.
                            let expected_tokens = phase6.artifact["config"]["max_tokens"]
                                .as_u64().unwrap_or(100);
                            let stopped_ok = phase6.artifact["telemetry"]["exit_reason"]
                                .as_str().map(|r| r == "Completed").unwrap_or(false);

                            let quality_metrics = phase6.artifact["telemetry"]["stdout_log"]
                                .as_str()
                                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                                .and_then(|v| v["content"].as_str().map(|s| s.to_string()));

                            if let Some(ref output_text) = quality_metrics {
                                let vresult = ValidationResult::validate(output_text, expected_tokens, stopped_ok);
                                let qscore = compute_quality_score(&vresult);
                                metrics.insert("quality_score".to_string(), qscore);
                                metrics.insert("repetition_score".to_string(), vresult.repetition_score);
                                metrics.insert("diversity_score".to_string(), vresult.diversity_score);
                                if vresult.infinite_loop_detected {
                                    metrics.insert("infinite_loop".to_string(), 1.0);
                                }
                            }
                        }
                    }
                }

                runtime_metrics.push(metrics);

                // ── Adaptive evidence saturation ──
                // After at least 3 reps, compute CV. If CV < 5%, mark as saturated.
                if runtime_metrics.len() >= 3 {
                    let tps_values: Vec<f64> = runtime_metrics.iter()
                        .filter_map(|m| m.get("tokens_per_second").copied())
                        .filter(|v| *v > 0.0)
                        .collect();

                    if tps_values.len() >= 3 {
                        let avg = tps_values.iter().sum::<f64>() / tps_values.len() as f64;
                        let variance = tps_values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / tps_values.len() as f64;
                        let std_dev = variance.sqrt();
                        let cv = std_dev / avg;

                        if cv < 0.05 {
                            saturated = true;
                            eprintln!("    ✅ Evidence saturated — CV = {:.1}%", cv * 100.0);
                        }
                    }
                }
            }

            all_metrics.insert(runtime_path.clone(), runtime_metrics);
        }

        // ── Generar AnswerRevision ──
        let answer_revision = self.generate_answer_revision(&all_metrics, &mut all_errors);

        // Almacenar en Knowledge Base
        if let Some(ref revision) = answer_revision {
            let state_dir = Path::new(".state");
            let mut kb = KnowledgeBase::load(state_dir);
            kb.store(revision.clone()).ok();
            eprintln!("\n  ✅ AnswerRevision stored in Knowledge Base");
            eprintln!("     Question: {}", revision.question);
            eprintln!("     Confidence: {:.1}%", revision.confidence * 100.0);
        }

        // Reporte final
        let duration = start.elapsed().unwrap_or_default().as_secs_f64();
        self.status = CampaignStatus::Completed;
        let report = CampaignReport {
            campaign_id: self.id.clone(),
            study_id: self.study_id.clone(),
            status: if all_errors.is_empty() { "completed".to_string() } else { "completed_with_errors".to_string() },
            total_runtimes: self.runtimes.len(),
            total_repetitions: self.repetitions,
            completed_executions: completed,
            failed_executions: failed,
            duration_seconds: duration,
            runtimes_tested: self.runtimes.clone(),
            model_used: self.model_path.clone(),
            answer_revision,
            errors: all_errors,
        };

        eprintln!("\n  ╔══════════════════════════════════════════╗");
        eprintln!("  ║     Campaign Complete                     ║");
        eprintln!("  ╚══════════════════════════════════════════╝");
        eprintln!();
        eprintln!("  ✅ {} executions completed", completed);
        eprintln!("  ❌ {} failed", failed);
        eprintln!("  ⏱ Duration: {:.0}s", duration);
        eprintln!();

        Ok(report)
    }

    /// Generar una AnswerRevision a partir de las métricas de la campaña.
    /// Busca el runtime ganador y produce una respuesta científica.
    fn generate_answer_revision(
        &self,
        all_metrics: &HashMap<String, Vec<HashMap<String, f64>>>,
        errors: &mut Vec<String>,
    ) -> Option<AnswerRevision> {
        if all_metrics.is_empty() {
            return None;
        }

        // Calcular promedio de TPS por runtime
        let mut runtime_avg_tps: Vec<(&str, f64, f64)> = Vec::new(); // (name, avg_tps, std_dev)
        for (runtime, metrics_list) in all_metrics {
            if metrics_list.is_empty() {
                continue;
            }
            let tps_values: Vec<f64> = metrics_list.iter()
                .filter_map(|m| m.get("tokens_per_second").copied())
                .filter(|v| *v > 0.0)
                .collect();

            if tps_values.is_empty() {
                continue;
            }

            let avg = tps_values.iter().sum::<f64>() / tps_values.len() as f64;
            let variance = tps_values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / tps_values.len() as f64;
            let std_dev = variance.sqrt();

            runtime_avg_tps.push((runtime, avg, std_dev));
        }

        if runtime_avg_tps.is_empty() {
            errors.push("No valid TPS measurements for any runtime".to_string());
            return None;
        }

        // Encontrar el mejor runtime
        runtime_avg_tps.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let best = &runtime_avg_tps[0];

        // Construir la respuesta
        let mut answer = String::new();
        answer.push_str(&format!("Runtime: {}", best.0));
        if runtime_avg_tps.len() > 1 {
            answer.push_str(&format!("\nTPS: {:.1} ± {:.1}", best.1, best.2));
            for (name, avg, std) in &runtime_avg_tps[1..] {
                let diff_pct = ((best.1 - avg) / avg) * 100.0;
                answer.push_str(&format!("\nvs {}: {:.1} ± {:.1} ({:+.1}%)", name, avg, std, diff_pct));
            }
        } else {
            answer.push_str(&format!("\nTPS: {:.1} ± {:.1}", best.1, best.2));
        }

        // Calcular confianza basada en varianza
        let cv = best.2 / best.1; // Coefficient of variation
        let confidence = (1.0 - cv).clamp(0.0, 1.0);

        // Obtener hardware fingerprint
        let hw = crate::runtime::hardware::detect_hardware();
        let hw_fingerprint = format!("{} {}GB", 
            hw.gpu.first().map(|g| g.model.as_str()).unwrap_or("unknown"),
            hw.gpu.first().map(|g| g.vram_gb).unwrap_or(0.0),
        );

        // Obtener driver/CUDA
        let driver = std::process::Command::new("nvidia-smi")
            .args(["--query-gpu=driver_version", "--format=csv,noheader"])
            .output().ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string());

        let cuda = std::process::Command::new("nvcc")
            .arg("--version")
            .output().ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.lines().find(|l| l.contains("release")).map(|l| l.trim().to_string()));

        // Construir key_metrics
        let mut key_metrics = HashMap::new();
        key_metrics.insert("best_tps".to_string(), best.1);
        key_metrics.insert("best_std_dev".to_string(), best.2);
        key_metrics.insert("confidence".to_string(), confidence);
        key_metrics.insert("runtimes_compared".to_string(), runtime_avg_tps.len() as f64);

        let model_name = Path::new(&self.model_path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let mut revision = AnswerRevision::new(
            &format!("¿Cuál runtime ofrece mejor rendimiento para {} en {}?", model_name, hw_fingerprint),
            &answer,
            confidence,
        );
        revision.runtime_variant = Some(best.0.to_string());
        revision.model = Some(model_name);
        revision.key_metrics = key_metrics;
        revision.hardware_fingerprint = hw_fingerprint;
        revision.driver_version = driver;
        revision.cuda_version = cuda;
        revision.total_executions = all_metrics.values().map(|v| v.len() as u32).sum();

        Some(revision)
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_campaign_creation() {
        let study = crate::runtime::study::Study::new("TEST", "Test question", vec!["PHASE-0001"]);
        let campaign = Campaign::from_study(&study, "/models/test.gguf");
        assert_eq!(campaign.study_id, "TEST");
        assert_eq!(campaign.model_path, "/models/test.gguf");
    }
}
