/// Knowledge Base — Almacén de respuestas científicas versionadas.
///
/// El laboratorio produce evidencia. La evidencia produce conocimiento.
/// El conocimiento produce AnswerRevisions — respuestas científicas versionadas,
/// con confianza, referencias a experimentos, y condiciones de validez.
///
/// ## Conceptos
/// - **AnswerRevision**: Una respuesta a una pregunta científica, con versión y confianza.
/// - **KnowledgeBase**: Almacén persistente de AnswerRevisions, consultable.
/// - **Cada respuesta es válida para un hardware/runtime/modelo específico**.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

// ═══════════════════════════════════════════════════════════════
// AnswerRevision
// ═══════════════════════════════════════════════════════════════

/// Una respuesta científica versionada a una pregunta de investigación.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerRevision {
    /// Pregunta científica (e.g., "¿Cuál runtime ofrece mejor rendimiento para Qwen 4B?")
    pub question: String,

    /// Número de revisión (incrementa con cada nueva evidencia)
    pub revision: u32,

    /// La respuesta en lenguaje natural
    pub answer: String,

    /// Confianza (0.0 a 1.0)
    pub confidence: f64,

    /// IDs de los experimentos que respaldan esta respuesta
    pub evidence_ids: Vec<String>,

    /// Número total de ejecuciones que respaldan la respuesta
    pub total_executions: u32,

    /// Hardware para el cual es válida esta respuesta
    pub hardware_fingerprint: String,

    /// Runtime evaluado
    pub runtime_variant: Option<String>,

    /// Modelo evaluado
    pub model: Option<String>,

    /// Métricas clave que resumen la respuesta
    pub key_metrics: HashMap<String, f64>,

    /// Versión del driver NVIDIA cuando se generó
    pub driver_version: Option<String>,

    /// Versión de CUDA cuando se generó
    pub cuda_version: Option<String>,

    /// Timestamp de creación
    pub created_at: u64,

    /// Timestamp de última actualización
    pub updated_at: u64,

    /// Estado: "active", "superseded", "outdated"
    pub status: String,

    /// IDs de revisiones anteriores que esta reemplaza
    pub supersedes: Vec<u32>,
}

impl AnswerRevision {
    /// Crear una nueva AnswerRevision
    pub fn new(question: &str, answer: &str, confidence: f64) -> Self {
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            question: question.to_string(),
            revision: 1,
            answer: answer.to_string(),
            confidence,
            evidence_ids: Vec::new(),
            total_executions: 0,
            hardware_fingerprint: String::new(),
            runtime_variant: None,
            model: None,
            key_metrics: HashMap::new(),
            driver_version: None,
            cuda_version: None,
            created_at: now,
            updated_at: now,
            status: "active".to_string(),
            supersedes: Vec::new(),
        }
    }

    /// Formato legible para mostrar en CLI/TUI
    pub fn display(&self) -> String {
        let status_icon = match self.status.as_str() {
            "active" => "🟢",
            "superseded" => "🟡",
            "outdated" => "🔴",
            _ => "⚪",
        };
        let mut s = format!(
            "{status_icon} Rev.{} — {}\n",
            self.revision, self.question
        );
        s.push_str(&format!("   Respuesta: {}\n", self.answer));
        s.push_str(&format!(
            "   Confianza: {:.1}% | Ejecuciones: {} | Evidencia: {}\n",
            self.confidence * 100.0,
            self.total_executions,
            self.evidence_ids.len()
        ));
        if let Some(ref rt) = self.runtime_variant {
            s.push_str(&format!("   Runtime: {rt}\n"));
        }
        if let Some(ref m) = self.model {
            s.push_str(&format!("   Modelo: {m}\n"));
        }
        if !self.key_metrics.is_empty() {
            s.push_str("   Métricas:\n");
            for (k, v) in &self.key_metrics {
                s.push_str(&format!("     {k}: {v}\n"));
            }
        }
        s
    }
}

// ═══════════════════════════════════════════════════════════════
// Knowledge Base
// ═══════════════════════════════════════════════════════════════

/// Almacén persistente de conocimiento científico versionado.
pub struct KnowledgeBase {
    state_dir: PathBuf,
    /// question → lista de revisiones (ordenadas por revision number descendente)
    revisions: HashMap<String, Vec<AnswerRevision>>,
    /// Índice por runtime_variant
    by_runtime: HashMap<String, Vec<String>>,
    /// Índice por modelo
    by_model: HashMap<String, Vec<String>>,
}

impl KnowledgeBase {
    /// Cargar o crear la Knowledge Base
    pub fn load(state_dir: &Path) -> Self {
        let kb_dir = state_dir.join("knowledge");
        std::fs::create_dir_all(&kb_dir).ok();

        let mut kb = Self {
            state_dir: state_dir.to_path_buf(),
            revisions: HashMap::new(),
            by_runtime: HashMap::new(),
            by_model: HashMap::new(),
        };
        kb.load_from_disk();
        kb
    }

    /// Almacenar una nueva AnswerRevision (o actualizar si existe)
    pub fn store(&mut self, revision: AnswerRevision) -> Result<(), String> {
        let question = revision.question.clone();

        // Obtener o crear la lista de revisiones para esta pregunta
        let entries = self.revisions.entry(question.clone()).or_default();

        // Calcular el siguiente número de revisión
        let next_revision = entries.iter().map(|r| r.revision).max().unwrap_or(0) + 1;

        // Si ya existe una revisión activa, marcarla como superseded
        for existing in entries.iter_mut() {
            if existing.status == "active" {
                existing.status = "superseded".to_string();
            }
        }

        // Añadir la nueva revisión con el número correcto
        let mut revision = revision;
        revision.revision = next_revision;
        entries.push(revision.clone());

        // Ordenar por revision number descendente
        entries.sort_by(|a, b| b.revision.cmp(&a.revision));

        // Actualizar índices
        if let Some(ref rt) = revision.runtime_variant {
            self.by_runtime.entry(rt.clone()).or_default().push(question.clone());
        }
        if let Some(ref m) = revision.model {
            self.by_model.entry(m.clone()).or_default().push(question.clone());
        }

        self.save_to_disk()
    }

    /// Obtener la última revisión activa para una pregunta
    pub fn latest(&self, question: &str) -> Option<&AnswerRevision> {
        self.revisions.get(question)
            .and_then(|entries| entries.iter().find(|r| r.status == "active"))
    }

    /// Obtener todas las revisiones para una pregunta
    pub fn all_revisions(&self, question: &str) -> Vec<&AnswerRevision> {
        self.revisions.get(question)
            .map(|entries| entries.iter().collect())
            .unwrap_or_default()
    }

    /// Obtener respuestas filtradas por runtime
    pub fn by_runtime(&self, runtime: &str) -> Vec<&AnswerRevision> {
        self.by_runtime.get(runtime)
            .map(|questions| {
                questions.iter()
                    .filter_map(|q| self.latest(q))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Obtener respuestas filtradas por modelo
    pub fn by_model(&self, model: &str) -> Vec<&AnswerRevision> {
        self.by_model.get(model)
            .map(|questions| {
                questions.iter()
                    .filter_map(|q| self.latest(q))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Listar todas las preguntas con respuestas activas
    pub fn questions(&self) -> Vec<&str> {
        let mut questions: Vec<&str> = self.revisions.keys().map(|s| s.as_str()).collect();
        questions.sort();
        questions
    }

    /// Obtener el número total de revisiones almacenadas
    pub fn total_revisions(&self) -> usize {
        self.revisions.values().map(|v| v.len()).sum()
    }

    /// Marcar todas las respuestas como outdated (ej: cuando cambia el driver)
    pub fn mark_all_outdated(&mut self, reason: &str) -> usize {
        let mut count = 0;
        for entries in self.revisions.values_mut() {
            for entry in entries.iter_mut() {
                if entry.status == "active" {
                    entry.status = "outdated".to_string();
                    count += 1;
                }
            }
        }
        if count > 0 {
            eprintln!("  ⚠ Marked {count} answer(s) as outdated: {reason}");
            self.save_to_disk().ok();
        }
        count
    }

    // ── Persistencia ──

    fn kb_file(&self) -> PathBuf {
        self.state_dir.join("knowledge").join("knowledge_base.json")
    }

    fn load_from_disk(&mut self) {
        let path = self.kb_file();
        if !path.exists() {
            return;
        }
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(data) = serde_json::from_str::<KnowledgeBaseData>(&content) {
                self.revisions = data.revisions;
                self.by_runtime = data.by_runtime;
                self.by_model = data.by_model;
            }
        }
    }

    fn save_to_disk(&self) -> Result<(), String> {
        let path = self.kb_file();
        let data = KnowledgeBaseData {
            revisions: self.revisions.clone(),
            by_runtime: self.by_runtime.clone(),
            by_model: self.by_model.clone(),
        };
        let content = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Cannot serialize knowledge base: {e}"))?;
        std::fs::write(&path, &content)
            .map_err(|e| format!("Cannot write knowledge base: {e}"))?;
        Ok(())
    }
}

/// Serializable data for KnowledgeBase persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct KnowledgeBaseData {
    revisions: HashMap<String, Vec<AnswerRevision>>,
    by_runtime: HashMap<String, Vec<String>>,
    by_model: HashMap<String, Vec<String>>,
}

// ═══════════════════════════════════════════════════════════════
// Recomendation Engine
// ═══════════════════════════════════════════════════════════════

/// Motor de recomendaciones que transforma evidencia en decisiones.
pub struct RecommendationEngine;

impl RecommendationEngine {
    /// Obtener la mejor configuración conocida para un hardware/modelo específico.
    /// Busca en la Knowledge Base la respuesta con mayor confianza.
    pub fn best_config<'a>(kb: &'a KnowledgeBase, _hardware: &str, model: &str) -> Option<&'a AnswerRevision> {
        // Primero buscar respuestas para este modelo específico
        let model_answers = kb.by_model(model);
        if !model_answers.is_empty() {
            return model_answers.into_iter()
                .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal));
        }
        None
    }

    /// Verificar si una respuesta necesita actualización (driver desactualizado, etc.)
    pub fn needs_update(revision: &AnswerRevision, current_driver: &str) -> bool {
        if let Some(ref rev_driver) = revision.driver_version {
            rev_driver != current_driver
        } else {
            false
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_answer_revision_new() {
        let rev = AnswerRevision::new("Test question?", "Test answer", 0.95);
        assert_eq!(rev.revision, 1);
        assert_eq!(rev.status, "active");
        assert!((rev.confidence - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_knowledge_base_store_and_latest() {
        let dir = tempdir().unwrap();
        let mut kb = KnowledgeBase::load(dir.path());

        let rev = AnswerRevision::new("¿Mejor runtime?", "TurboQuant", 0.92);
        kb.store(rev).unwrap();

        let latest = kb.latest("¿Mejor runtime?");
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().answer, "TurboQuant");
    }

    #[test]
    fn test_revision_superseding() {
        let dir = tempdir().unwrap();
        let mut kb = KnowledgeBase::load(dir.path());

        let rev1 = AnswerRevision::new("¿Mejor runtime?", "Official", 0.80);
        kb.store(rev1).unwrap();

        let rev2 = AnswerRevision::new("¿Mejor runtime?", "TurboQuant", 0.95);
        kb.store(rev2).unwrap();

        let latest = kb.latest("¿Mejor runtime?").unwrap();
        assert_eq!(latest.answer, "TurboQuant");
        assert_eq!(latest.revision, 2); // Auto-incremented: revision 2 supersedes revision 1
        // Actually, we need to check that the first one was superseded
        let all = kb.all_revisions("¿Mejor runtime?");
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].status, "active"); // Latest is first (sorted by revision desc)
        assert_eq!(all[0].answer, "TurboQuant");
    }

    #[test]
    fn test_persistence() {
        let dir = tempdir().unwrap();

        {
            let mut kb = KnowledgeBase::load(dir.path());
            let rev = AnswerRevision::new("¿Mejor runtime?", "TurboQuant", 0.92);
            kb.store(rev).unwrap();
        }

        {
            let kb = KnowledgeBase::load(dir.path());
            assert_eq!(kb.total_revisions(), 1);
            assert_eq!(kb.questions().len(), 1);
        }
    }

    #[test]
    fn test_mark_all_outdated() {
        let dir = tempdir().unwrap();
        let mut kb = KnowledgeBase::load(dir.path());

        kb.store(AnswerRevision::new("Q1", "A1", 0.9)).unwrap();
        kb.store(AnswerRevision::new("Q2", "A2", 0.8)).unwrap();

        assert_eq!(kb.mark_all_outdated("Driver updated"), 2);
        assert!(kb.latest("Q1").is_none()); // No active answers
    }
}
