use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Certification states
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertState {
    Valid,
    Stale,
    Superseded,
    Invalid,
}

impl CertState {
    pub fn name(&self) -> &str {
        match self {
            CertState::Valid => "VALID",
            CertState::Stale => "STALE",
            CertState::Superseded => "SUPERSEDED",
            CertState::Invalid => "INVALID",
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "VALID" => Some(CertState::Valid),
            "STALE" => Some(CertState::Stale),
            "SUPERSEDED" => Some(CertState::Superseded),
            "INVALID" => Some(CertState::Invalid),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// metadata.yaml — permanent model info
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_id: String,
    pub family: String,
    pub quantization: String,
    pub runtime: String,
    pub parameters_b: f64,
    pub created: String,
    pub last_seen: String,
    pub status: String,
}

// ---------------------------------------------------------------------------
// current.yaml — best known state (projected, never source of truth)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentCertification {
    pub capability_score: f64,
    pub engineering_score: f64,
    pub reliability_score: f64,
    pub confidence: f64,
    pub best_task: String,
    pub best_runtime: String,
    pub last_validation: String,
    pub state: String,
}

// ---------------------------------------------------------------------------
// History entry — one per certification execution
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: String,
    pub task: String,
    pub prompt_hash: String,
    pub runtime: String,
    pub model: String,
    pub engineering_score: f64,
    pub capability_score: f64,
    pub reliability_score: f64,
    pub signals: serde_json::Value,
    pub evidence: serde_json::Value,
    pub git_commit: String,
    pub athena_version: String,
    pub state: String,
    pub duration_ms: f64,
    pub total_tokens: usize,
    pub success: bool,
}

// ---------------------------------------------------------------------------
// ModelCertification — aggregates all data for one model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCertification {
    pub model_id: String,
    pub metadata: ModelMetadata,
    pub current: CurrentCertification,
    pub history: Vec<HistoryEntry>,
}

// ---------------------------------------------------------------------------
// CapabilityDatabase V2 — model-based storage with history
// ---------------------------------------------------------------------------

const MODELS_DIR: &str = ".state/models";

pub struct CapabilityDatabase {
    base_path: PathBuf,
}

impl CapabilityDatabase {
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from(MODELS_DIR),
        }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self { base_path: path }
    }

    /// Path to a model's directory
    fn model_dir(&self, model_id: &str) -> PathBuf {
        let safe_id = model_id.to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "-")
            .trim_matches('-')
            .to_string();
        self.base_path.join(&safe_id)
    }

    fn history_dir(&self, model_id: &str) -> PathBuf {
        self.model_dir(model_id).join("history")
    }

    fn metadata_path(&self, model_id: &str) -> PathBuf {
        self.model_dir(model_id).join("metadata.yaml")
    }

    fn current_path(&self, model_id: &str) -> PathBuf {
        self.model_dir(model_id).join("current.yaml")
    }

    // ── Metadata ──

    pub fn save_metadata(&self, metadata: &ModelMetadata) -> anyhow::Result<()> {
        let dir = self.model_dir(&metadata.model_id);
        std::fs::create_dir_all(&dir)?;
        let yaml = serde_yaml::to_string(metadata)?;
        std::fs::write(self.metadata_path(&metadata.model_id), yaml)?;
        Ok(())
    }

    pub fn load_metadata(&self, model_id: &str) -> Option<ModelMetadata> {
        let path = self.metadata_path(model_id);
        if !path.exists() { return None; }
        std::fs::read_to_string(&path).ok()
            .and_then(|s| serde_yaml::from_str(&s).ok())
    }

    // ── Current certification ──

    pub fn save_current(&self, model_id: &str, current: &CurrentCertification) -> anyhow::Result<()> {
        let dir = self.model_dir(model_id);
        std::fs::create_dir_all(&dir)?;
        let yaml = serde_yaml::to_string(current)?;
        std::fs::write(self.current_path(model_id), yaml)?;
        Ok(())
    }

    pub fn load_current(&self, model_id: &str) -> Option<CurrentCertification> {
        let path = self.current_path(model_id);
        if !path.exists() { return None; }
        std::fs::read_to_string(&path).ok()
            .and_then(|s| serde_yaml::from_str(&s).ok())
    }

    // ── History (append-only) ──

    pub fn append_history(&self, model_id: &str, entry: &HistoryEntry) -> anyhow::Result<()> {
        let dir = self.history_dir(model_id);
        std::fs::create_dir_all(&dir)?;

        let filename = format!("{}.yaml", entry.timestamp.replace(':', "-"));
        let path = dir.join(&filename);
        let yaml = serde_yaml::to_string(entry)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }

    pub fn list_history(&self, model_id: &str) -> Vec<HistoryEntry> {
        let dir = self.history_dir(model_id);
        if !dir.is_dir() { return vec![]; }

        let mut entries = Vec::new();
        for entry in std::fs::read_dir(&dir).ok().into_iter().flatten() {
            let entry = match entry { Ok(e) => e, _ => continue };
            let path = entry.path();
            if path.extension().map(|e| e == "yaml").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(he) = serde_yaml::from_str::<HistoryEntry>(&content) {
                        entries.push(he);
                    }
                }
            }
        }
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries
    }

    // ── Full model load ──

    pub fn load_model(&self, model_id: &str) -> Option<ModelCertification> {
        let metadata = self.load_metadata(model_id)?;
        let current = self.load_current(model_id).unwrap_or(CurrentCertification {
            capability_score: 0.0,
            engineering_score: 0.0,
            reliability_score: 0.0,
            confidence: 0.0,
            best_task: String::new(),
            best_runtime: String::new(),
            last_validation: String::new(),
            state: "UNKNOWN".to_string(),
        });
        let history = self.list_history(model_id);

        Some(ModelCertification {
            model_id: model_id.to_string(),
            metadata,
            current,
            history,
        })
    }

    // ── List all known models ──

    pub fn list_models(&self) -> Vec<String> {
        let dir = &self.base_path;
        if !dir.is_dir() { return vec![]; }

        let mut models = Vec::new();
        for entry in std::fs::read_dir(dir).ok().into_iter().flatten() {
            let entry = match entry { Ok(e) => e, _ => continue };
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    if !name.starts_with('.') {
                        models.push(name.to_string());
                    }
                }
            }
        }
        models.sort();
        models
    }

    // ── Best configuration query for scheduler ──

    pub fn best_configuration(&self, model_id: &str) -> Option<CurrentCertification> {
        let current = self.load_current(model_id)?;
        if current.state == "INVALID" {
            return None;
        }
        Some(current)
    }

    /// Transition a certification's state
    pub fn transition_state(&self, model_id: &str, new_state: CertState) -> anyhow::Result<()> {
        if let Some(mut current) = self.load_current(model_id) {
            current.state = new_state.name().to_string();
            current.last_validation = chrono::Utc::now().to_rfc3339();
            self.save_current(model_id, &current)?;
        }
        // Also update metadata status
        if let Some(mut meta) = self.load_metadata(model_id) {
            meta.status = new_state.name().to_string();
            meta.last_seen = chrono::Utc::now().to_rfc3339();
            self.save_metadata(&meta)?;
        }
        Ok(())
    }

    // ── Save a full certification from history entry ──

    pub fn save_certification(
        &self,
        model_id: &str,
        metadata: &ModelMetadata,
        entry: &HistoryEntry,
    ) -> anyhow::Result<()> {
        // Save/update metadata
        self.save_metadata(metadata)?;

        // Append to history
        self.append_history(model_id, entry)?;

        // Update current certification (best known)
        let current = CurrentCertification {
            capability_score: entry.capability_score,
            engineering_score: entry.engineering_score,
            reliability_score: entry.reliability_score,
            confidence: entry.capability_score.max(0.1), // baseline confidence
            best_task: entry.task.clone(),
            best_runtime: entry.runtime.clone(),
            last_validation: entry.timestamp.clone(),
            state: "VALID".to_string(),
        };
        self.save_current(model_id, &current)?;

        Ok(())
    }

    /// Prune: mark certifications as INVALID for models that no longer exist
    pub fn prune(&self, existing_model_ids: &[String]) -> anyhow::Result<usize> {
        let mut pruned = 0;
        for model_id in self.list_models() {
            if !existing_model_ids.contains(&model_id) {
                self.transition_state(&model_id, CertState::Invalid)?;
                pruned += 1;
            }
        }
        Ok(pruned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> (CapabilityDatabase, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db = CapabilityDatabase::with_path(dir.path().join("models"));
        (db, dir)
    }

    #[test]
    fn test_save_and_load_metadata() {
        let (db, _dir) = test_db();
        let meta = ModelMetadata {
            model_id: "qwen3.5-9b-q4_k_m".to_string(),
            family: "Qwen".to_string(),
            quantization: "Q4_K_M".to_string(),
            runtime: "llama.cpp".to_string(),
            parameters_b: 9.0,
            created: "2026-07-17".to_string(),
            last_seen: "2026-07-17".to_string(),
            status: "VALID".to_string(),
        };
        db.save_metadata(&meta).unwrap();
        let loaded = db.load_metadata("qwen3.5-9b-q4_k_m").unwrap();
        assert_eq!(loaded.model_id, "qwen3.5-9b-q4_k_m");
        assert_eq!(loaded.family, "Qwen");
    }

    #[test]
    fn test_save_and_list_history() {
        let (db, _dir) = test_db();
        let entry = HistoryEntry {
            timestamp: "2026-07-17T18-22-10".to_string(),
            task: "HumanEval".to_string(),
            prompt_hash: "abc123".to_string(),
            runtime: "llama.cpp".to_string(),
            model: "qwen3.5".to_string(),
            engineering_score: 0.85,
            capability_score: 0.92,
            reliability_score: 0.78,
            signals: serde_json::json!({}),
            evidence: serde_json::json!({}),
            git_commit: "abc".to_string(),
            athena_version: "0.1.0".to_string(),
            state: "VALID".to_string(),
            duration_ms: 1500.0,
            total_tokens: 500,
            success: true,
        };
        db.append_history("qwen3.5-9b-q4_k_m", &entry).unwrap();
        let history = db.list_history("qwen3.5-9b-q4_k_m");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].task, "HumanEval");
    }

    #[test]
    fn test_transition_state() {
        let (db, _dir) = test_db();
        let meta = ModelMetadata {
            model_id: "test-model".to_string(),
            family: "Test".to_string(),
            quantization: "Q4_0".to_string(),
            runtime: "mock".to_string(),
            parameters_b: 7.0,
            created: "now".to_string(),
            last_seen: "now".to_string(),
            status: "VALID".to_string(),
        };
        let entry = HistoryEntry {
            timestamp: "now".to_string(),
            task: "test".to_string(),
            prompt_hash: "hash".to_string(),
            runtime: "mock".to_string(),
            model: "test-model".to_string(),
            engineering_score: 0.8,
            capability_score: 0.8,
            reliability_score: 0.8,
            signals: serde_json::json!({}),
            evidence: serde_json::json!({}),
            git_commit: "abc".to_string(),
            athena_version: "0.1.0".to_string(),
            state: "VALID".to_string(),
            duration_ms: 100.0,
            total_tokens: 100,
            success: true,
        };
        db.save_certification("test-model", &meta, &entry).unwrap();

        // Transition to STALE
        db.transition_state("test-model", CertState::Stale).unwrap();
        let current = db.load_current("test-model").unwrap();
        assert_eq!(current.state, "STALE");
    }

    #[test]
    fn test_save_certification_creates_all_files() {
        let (db, _dir) = test_db();
        let meta = ModelMetadata {
            model_id: "full-test".to_string(),
            family: "Test".to_string(),
            quantization: "Q4_K_M".to_string(),
            runtime: "llama.cpp".to_string(),
            parameters_b: 7.0,
            created: "now".to_string(),
            last_seen: "now".to_string(),
            status: "VALID".to_string(),
        };
        let entry = HistoryEntry {
            timestamp: "2026-07-17T12-00-00".to_string(),
            task: "HumanEval".to_string(),
            prompt_hash: "def".to_string(),
            runtime: "llama.cpp".to_string(),
            model: "full-test".to_string(),
            engineering_score: 0.9,
            capability_score: 0.95,
            reliability_score: 0.85,
            signals: serde_json::json!({}),
            evidence: serde_json::json!({}),
            git_commit: "def".to_string(),
            athena_version: "0.1.0".to_string(),
            state: "VALID".to_string(),
            duration_ms: 2000.0,
            total_tokens: 1000,
            success: true,
        };
        db.save_certification("full-test", &meta, &entry).unwrap();

        // Verify all files exist
        assert!(db.metadata_path("full-test").exists());
        assert!(db.current_path("full-test").exists());
        assert!(db.history_dir("full-test").join("2026-07-17T12-00-00.yaml").exists());

        // Verify listing
        let models = db.list_models();
        assert!(models.contains(&"full-test".to_string()));
    }

    #[test]
    fn test_prune_marks_invalid() {
        let (db, dir) = test_db();

        // Create a model that doesn't exist anymore
        let meta = ModelMetadata {
            model_id: "deleted-model".to_string(),
            family: "Test".to_string(),
            quantization: "Q4_0".to_string(),
            runtime: "mock".to_string(),
            parameters_b: 3.0,
            created: "past".to_string(),
            last_seen: "past".to_string(),
            status: "VALID".to_string(),
        };
        let entry = HistoryEntry {
            timestamp: "past".to_string(),
            task: "test".to_string(),
            prompt_hash: "x".to_string(),
            runtime: "mock".to_string(),
            model: "deleted-model".to_string(),
            engineering_score: 0.5,
            capability_score: 0.5,
            reliability_score: 0.5,
            signals: serde_json::json!({}),
            evidence: serde_json::json!({}),
            git_commit: "x".to_string(),
            athena_version: "0.1.0".to_string(),
            state: "VALID".to_string(),
            duration_ms: 100.0,
            total_tokens: 100,
            success: true,
        };
        db.save_certification("deleted-model", &meta, &entry).unwrap();

        // Prune with empty existing list
        let pruned = db.prune(&[]).unwrap();
        assert_eq!(pruned, 1);

        let current = db.load_current("deleted-model").unwrap();
        assert_eq!(current.state, "INVALID");
        let meta = db.load_metadata("deleted-model").unwrap();
        assert_eq!(meta.status, "INVALID");

        // Clean up temp dir
        dir.close().ok();
    }
}
