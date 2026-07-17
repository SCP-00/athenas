use std::path::{Path, PathBuf};
use std::time::Instant;

use super::planner::ExperimentKnowledge;
use super::recovery::ExperimentResult;

/// Checkpoint — captures the full state of an in-progress certification
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CertificationCheckpoint {
    pub experiment_id: String,
    pub model_path: String,
    pub model_name: String,
    pub created_at: String,
    pub updated_at: String,
    pub total_experiments_planned: usize,
    pub completed_experiments: usize,
    pub failed_experiments: usize,
    pub current_experiment_index: usize,
    pub knowledge: Vec<ExperimentKnowledge>,
    pub results: Vec<ExperimentResult>,
    pub start_time_epoch: u64,
}

impl CertificationCheckpoint {
    pub fn new(experiment_id: String, model_path: String, model_name: String, total_planned: usize) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            experiment_id,
            model_path,
            model_name,
            created_at: format!("{}", now),
            updated_at: format!("{}", now),
            total_experiments_planned: total_planned,
            completed_experiments: 0,
            failed_experiments: 0,
            current_experiment_index: 0,
            knowledge: Vec::new(),
            results: Vec::new(),
            start_time_epoch: now,
        }
    }

    pub fn elapsed_seconds(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.start_time_epoch)
    }

    pub fn progress_pct(&self) -> f64 {
        if self.total_experiments_planned == 0 {
            return 0.0;
        }
        (self.completed_experiments + self.failed_experiments) as f64 / self.total_experiments_planned as f64 * 100.0
    }
}

// ── Checkpoint Manager ──

pub struct CheckpointManager {
    base_path: PathBuf,
    current_checkpoint: Option<CertificationCheckpoint>,
    start_time: Instant,
}

impl CheckpointManager {
    pub fn new(state_dir: &Path) -> Self {
        let base_path = state_dir.join("experiments");
        Self {
            base_path,
            current_checkpoint: None,
            start_time: Instant::now(),
        }
    }

    /// Begin a new certification run with a checkpoint.
    pub fn begin(
        &mut self,
        experiment_id: &str,
        model_path: &str,
        model_name: &str,
        total_planned: usize,
    ) -> anyhow::Result<()> {
        let cp = CertificationCheckpoint::new(
            experiment_id.to_string(),
            model_path.to_string(),
            model_name.to_string(),
            total_planned,
        );
        self.current_checkpoint = Some(cp);
        self.start_time = Instant::now();
        self.save()?;
        Ok(())
    }

    /// Record a completed experiment result.
    pub fn record_experiment(&mut self, knowledge: ExperimentKnowledge, result: ExperimentResult) -> anyhow::Result<()> {
        if let Some(ref mut cp) = self.current_checkpoint {
            cp.knowledge.push(knowledge);
            cp.results.push(result);
            if cp.results.last().map(|r| r.success).unwrap_or(false) {
                cp.completed_experiments += 1;
            } else {
                cp.failed_experiments += 1;
            }
            cp.current_experiment_index += 1;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            cp.updated_at = format!("{}", now);
            self.save()?;
        }
        Ok(())
    }

    /// Load a checkpoint from a file.
    pub fn load(&mut self, experiment_id: &str) -> anyhow::Result<CertificationCheckpoint> {
        let path = self.base_path.join(format!("{experiment_id}.json"));
        let content = std::fs::read_to_string(&path)?;
        let cp: CertificationCheckpoint = serde_json::from_str(&content)?;
        self.current_checkpoint = Some(cp.clone());
        self.start_time = Instant::now();
        Ok(cp)
    }

    /// List all saved checkpoints.
    pub fn list_checkpoints(&self) -> anyhow::Result<Vec<String>> {
        let mut ids = Vec::new();
        if !self.base_path.is_dir() {
            return Ok(ids);
        }
        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Some(stem) = path.file_stem() {
                    ids.push(stem.to_string_lossy().to_string());
                }
            }
        }
        ids.sort();
        ids.reverse();
        Ok(ids)
    }

    /// Get the current checkpoint (if any)
    pub fn current(&self) -> Option<&CertificationCheckpoint> {
        self.current_checkpoint.as_ref()
    }

    /// Get elapsed time since the certification started
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Save the checkpoint to disk
    fn save(&self) -> anyhow::Result<()> {
        if let Some(ref cp) = self.current_checkpoint {
            std::fs::create_dir_all(&self.base_path)?;
            let path = self.base_path.join(format!("{}.json", cp.experiment_id));
            let json = serde_json::to_string_pretty(cp)?;
            std::fs::write(&path, json)?;
        }
        Ok(())
    }
}
