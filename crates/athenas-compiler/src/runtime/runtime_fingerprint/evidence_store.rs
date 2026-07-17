/// Evidence Store — Persistent storage for scientific evidence.
///
/// ## Negative Evidence
/// Experiments that failed validation are stored as "negative evidence".
/// Athena learns from discarded experiments — configurations that should
/// never be retried on this hardware with this runtime.
///
/// ## Structure
/// .state/evidence/
///   negative/        ← Failed validations (never retry)
///     NEG-174321/    ← Individual evidence record
///       evidence.yaml
///   positive/        ← Successful experiments
///     POS-174322/
///       evidence.yaml
///   index.json       ← Quick lookup index
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::experiment_validation::NegativeEvidence;

/// Category of evidence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EvidenceCategory {
    /// Experiment rejected by PHASE-0011 validation
    Negative,
    /// Successful experiment with valid results
    Positive,
}

/// An evidence record in the store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRecord {
    /// Unique ID
    pub id: String,
    /// Category
    pub category: EvidenceCategory,
    /// Timestamp
    pub timestamp: u64,
    /// The negative evidence (if category is Negative)
    pub negative_evidence: Option<NegativeEvidence>,
    /// Configuration hash for deduplication
    pub config_hash: String,
    /// Runtime fingerprint hash (for grouping by runtime)
    pub runtime_hash: Option<String>,
    /// Model hash (for grouping by model)
    pub model_hash: Option<String>,
    /// Hardware fingerprint
    pub hardware_snapshot: serde_json::Value,
}

/// Quick-lookup index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceIndex {
    /// Maps config_hash → list of evidence IDs
    pub by_config: HashMap<String, Vec<String>>,
    /// Maps runtime_hash → list of evidence IDs
    pub by_runtime: HashMap<String, Vec<String>>,
    /// Maps model_hash → list of evidence IDs
    pub by_model: HashMap<String, Vec<String>>,
    /// Categories with counts
    pub counts: HashMap<String, usize>,
}

impl EvidenceIndex {
    pub fn new() -> Self {
        Self {
            by_config: HashMap::new(),
            by_runtime: HashMap::new(),
            by_model: HashMap::new(),
            counts: HashMap::new(),
        }
    }
}

/// Persistent evidence store.
/// Manages both negative and positive evidence on disk.
#[derive(Debug, Clone)]
pub struct EvidenceStore {
    /// Directory where evidence is stored
    state_dir: PathBuf,
    /// In-memory index
    index: EvidenceIndex,
}

impl EvidenceStore {
    /// Load or create the evidence store
    pub fn load(state_dir: &Path) -> Self {
        let evidence_dir = state_dir.join("evidence");
        std::fs::create_dir_all(evidence_dir.join("negative")).ok();
        std::fs::create_dir_all(evidence_dir.join("positive")).ok();

        let index = Self::load_index(&evidence_dir);

        Self { state_dir: state_dir.to_path_buf(), index }
    }

    /// Store negative evidence (experiment rejected by validation)
    pub fn store_negative(&mut self, evidence: &NegativeEvidence) -> Result<(), String> {
        let dir = self.state_dir.join("evidence").join("negative").join(&evidence.id);
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Cannot create evidence dir: {e}"))?;

        let record = EvidenceRecord {
            id: evidence.id.clone(),
            category: EvidenceCategory::Negative,
            timestamp: evidence.timestamp,
            negative_evidence: Some(evidence.clone()),
            config_hash: simple_hash(&serde_json::to_string(&evidence.configuration).unwrap_or_default()),
            runtime_hash: evidence.runtime_fingerprint.clone(),
            model_hash: evidence.model_hash.clone(),
            hardware_snapshot: evidence.hardware.clone(),
        };

        // Save record
        let path = dir.join("evidence.json");
        let content = serde_json::to_string_pretty(&record)
            .map_err(|e| format!("Cannot serialize evidence: {e}"))?;
        std::fs::write(&path, &content)
            .map_err(|e| format!("Cannot write evidence: {e}"))?;

        // Update index
        self.index.by_config.entry(record.config_hash.clone())
            .or_default().push(evidence.id.clone());
        if let Some(ref rh) = record.runtime_hash {
            self.index.by_runtime.entry(rh.clone())
                .or_default().push(evidence.id.clone());
        }
        if let Some(ref mh) = record.model_hash {
            self.index.by_model.entry(mh.clone())
                .or_default().push(evidence.id.clone());
        }
        *self.index.counts.entry("negative".to_string()).or_insert(0) += 1;

        // Save index
        self.save_index()
    }

    /// Check if a configuration has been rejected before
    pub fn is_known_invalid(&self, config: &serde_json::Value) -> bool {
        let hash = simple_hash(&serde_json::to_string(config).unwrap_or_default());
        self.index.by_config.contains_key(&hash)
    }

    /// Look up evidence by runtime hash
    pub fn get_by_runtime(&self, runtime_hash: &str) -> Vec<&str> {
        self.index.by_runtime.get(runtime_hash)
            .map(|ids| ids.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Look up evidence by model hash
    pub fn get_by_model(&self, model_hash: &str) -> Vec<&str> {
        self.index.by_model.get(model_hash)
            .map(|ids| ids.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get evidence counts by category
    pub fn counts(&self) -> &HashMap<String, usize> {
        &self.index.counts
    }

    /// Get total amount of evidence stored
    pub fn total_count(&self) -> usize {
        self.index.counts.values().sum()
    }

    // ── Private helpers ──

    fn load_index(evidence_dir: &Path) -> EvidenceIndex {
        let index_path = evidence_dir.join("index.json");
        if index_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&index_path) {
                if let Ok(index) = serde_json::from_str(&content) {
                    return index;
                }
            }
        }
        EvidenceIndex::new()
    }

    fn save_index(&self) -> Result<(), String> {
        let index_path = self.state_dir.join("evidence").join("index.json");
        let content = serde_json::to_string_pretty(&self.index)
            .map_err(|e| format!("Cannot serialize index: {e}"))?;
        std::fs::write(&index_path, &content)
            .map_err(|e| format!("Cannot write index: {e}"))?;
        Ok(())
    }
}

/// Simple hash of a string for deduplication
fn simple_hash(s: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_store_load() {
        let dir = tempdir().unwrap();
        let mut store = EvidenceStore::load(dir.path());

        let neg = NegativeEvidence {
            id: "NEG-001".to_string(),
            timestamp: 1000,
            configuration: serde_json::json!({"test": true}),
            reasons: vec!["Test failure".to_string()],
            runtime_fingerprint: None,
            model_hash: None,
            hardware: serde_json::json!({}),
            category: "test".to_string(),
            attempt_count: 1,
        };

        store.store_negative(&neg).unwrap();
        assert_eq!(store.total_count(), 1);
        assert_eq!(*store.counts().get("negative").unwrap_or(&0), 1);
    }

    #[test]
    fn test_known_invalid() {
        let dir = tempdir().unwrap();
        let mut store = EvidenceStore::load(dir.path());

        let config = serde_json::json!({"ctx": 32768, "runtime": "test"});
        assert!(!store.is_known_invalid(&config));

        let neg = NegativeEvidence {
            id: "NEG-002".to_string(),
            timestamp: 1000,
            configuration: config.clone(),
            reasons: vec!["VRAM insufficient".to_string()],
            runtime_fingerprint: None,
            model_hash: None,
            hardware: serde_json::json!({}),
            category: "incompatible_configuration".to_string(),
            attempt_count: 1,
        };

        store.store_negative(&neg).unwrap();
        assert!(store.is_known_invalid(&config));
    }

    #[test]
    fn test_persistence() {
        let dir = tempdir().unwrap();

        // Store evidence
        {
            let mut store = EvidenceStore::load(dir.path());
            let neg = NegativeEvidence {
                id: "NEG-003".to_string(),
                timestamp: 1000,
                configuration: serde_json::json!({"ctx": 65536}),
                reasons: vec!["OOM".to_string()],
                runtime_fingerprint: None,
                model_hash: Some("abc123".to_string()),
                hardware: serde_json::json!({}),
                category: "memory".to_string(),
                attempt_count: 1,
            };
            store.store_negative(&neg).unwrap();
        }

        // Reload and verify
        {
            let store = EvidenceStore::load(dir.path());
            assert_eq!(store.total_count(), 1);
            assert!(!store.get_by_model("abc123").is_empty());
        }
    }
}
