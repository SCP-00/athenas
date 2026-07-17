use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Status of an experiment in the queue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExperimentStatus {
    Queued,
    Running,
    Completed,
    Failed(String),
    Blocked(String),
    Cancelled,
}

/// An experiment represents a complete certification run for a model.
/// It is processed autonomously by Athena — each phase produces artifacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    /// Unique experiment ID (e.g., "EXP-20260718-001")
    pub id: String,

    /// Path to the GGUF model file
    pub model_path: String,

    /// Current status
    pub status: ExperimentStatus,

    /// When the experiment was created (unix timestamp)
    pub created_at: u64,

    /// When the experiment started processing (unix timestamp)
    pub started_at: Option<u64>,

    /// When the experiment completed (unix timestamp)
    pub completed_at: Option<u64>,

    /// Number of retry attempts (auto-retry on failure)
    pub retry_count: u8,

    /// Maximum retry attempts before giving up
    pub max_retries: u8,

    /// IDs of phases that have been completed
    pub completed_phases: Vec<String>,

    /// IDs of phases that failed
    pub failed_phases: Vec<String>,

    /// Results summary (populated on completion)
    pub result: Option<ExperimentResult>,

    /// Error message if failed
    pub error: Option<String>,

    /// Tags for filtering/grouping
    pub tags: Vec<String>,
}

/// Summary of experiment results after completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    pub duration_seconds: f64,
    pub total_phases: usize,
    pub completed_phases: usize,
    pub failed_phases: usize,
    pub best_config: Option<serde_json::Value>,
    pub hardware_snapshot: Option<serde_json::Value>,
    pub runtimes_found: usize,
    pub models_discovered: Vec<String>,
}

impl Experiment {
    pub fn new(id: &str, model_path: &str) -> Self {
        use std::time::SystemTime;
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            id: id.to_string(),
            model_path: model_path.to_string(),
            status: ExperimentStatus::Queued,
            created_at: now,
            started_at: None,
            completed_at: None,
            retry_count: 0,
            max_retries: 3,
            completed_phases: Vec::new(),
            failed_phases: Vec::new(),
            result: None,
            error: None,
            tags: Vec::new(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Experiment Queue — Persistent Queue Manager
// ═══════════════════════════════════════════════════════════════

/// The Experiment Queue manages a persistent queue of experiments.
/// Buffy feeds experiments into the queue; Athena processes them autonomously.
pub struct ExperimentQueue {
    /// Directory where the queue is stored
    queue_dir: PathBuf,

    /// In-memory cache of all experiments (keyed by experiment ID)
    experiments: HashMap<String, Experiment>,
}

impl ExperimentQueue {
    /// Load or create the experiment queue from disk
    pub fn load(state_dir: &Path) -> Self {
        let queue_dir = state_dir.join("queue");
        std::fs::create_dir_all(&queue_dir).ok();

        let mut queue = Self {
            queue_dir,
            experiments: HashMap::new(),
        };
        queue.load_from_disk();
        queue
    }

    /// Enqueue a new experiment
    pub fn enqueue(&mut self, experiment: Experiment) -> Result<(), String> {
        let id = experiment.id.clone();
        if self.experiments.contains_key(&id) {
            return Err(format!("Experiment {id} already exists"));
        }
        self.experiments.insert(id.clone(), experiment);
        self.save_to_disk()
    }

    /// Dequeue the next experiment to process (QUEUED → RUNNING)
    pub fn dequeue(&mut self) -> Option<Experiment> {
        let next_id = self.experiments.iter()
            .filter(|(_, e)| e.status == ExperimentStatus::Queued)
            .min_by_key(|(_, e)| e.created_at)
            .map(|(id, _)| id.clone());

        if let Some(id) = next_id {
            if let Some(experiment) = self.experiments.get_mut(&id) {
                use std::time::SystemTime;
                experiment.status = ExperimentStatus::Running;
                experiment.started_at = Some(
                    SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                );
                self.save_to_disk().ok();
                return self.experiments.get(&id).cloned();
            }
        }
        None
    }

    /// Mark an experiment as completed
    pub fn complete(&mut self, id: &str, result: ExperimentResult) -> Result<(), String> {
        if let Some(experiment) = self.experiments.get_mut(id) {
            use std::time::SystemTime;
            experiment.status = ExperimentStatus::Completed;
            experiment.completed_at = Some(
                SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            );
            experiment.result = Some(result);
            self.save_to_disk()
        } else {
            Err(format!("Experiment {id} not found"))
        }
    }

    /// Mark an experiment as blocked (e.g., waiting for a dependency)
    pub fn block(&mut self, id: &str, reason: &str) -> Result<(), String> {
        if let Some(experiment) = self.experiments.get_mut(id) {
            experiment.status = ExperimentStatus::Blocked(reason.to_string());
            self.save_to_disk()
        } else {
            Err(format!("Experiment {id} not found"))
        }
    }

    /// Unblock a previously blocked experiment (re-queues it)
    pub fn unblock(&mut self, id: &str) -> Result<(), String> {
        if let Some(experiment) = self.experiments.get_mut(id) {
            experiment.status = ExperimentStatus::Queued;
            self.save_to_disk()
        } else {
            Err(format!("Experiment {id} not found"))
        }
    }

    /// Mark an experiment as failed (re-queues on retry, fails permanently after max_retries)
    pub fn fail(&mut self, id: &str, error: &str) -> Result<(), String> {
        if let Some(experiment) = self.experiments.get_mut(id) {
            if experiment.retry_count < experiment.max_retries {
                experiment.retry_count += 1;
                experiment.status = ExperimentStatus::Queued; // Re-queue for retry
                experiment.error = Some(format!("Attempt {}: {error}", experiment.retry_count));
            } else {
                experiment.status = ExperimentStatus::Failed(error.to_string());
                experiment.error = Some(format!("Max retries ({}) exceeded: {error}", experiment.max_retries));
            }
            self.save_to_disk()
        } else {
            Err(format!("Experiment {id} not found"))
        }
    }

    /// Record that a phase was completed for an experiment
    pub fn record_phase_complete(&mut self, experiment_id: &str, phase_id: &str) -> Result<(), String> {
        if let Some(exp) = self.experiments.get_mut(experiment_id) {
            if !exp.completed_phases.contains(&phase_id.to_string()) {
                exp.completed_phases.push(phase_id.to_string());
            }
            self.save_to_disk()
        } else {
            Err(format!("Experiment {experiment_id} not found"))
        }
    }

    /// Record that a phase failed
    pub fn record_phase_failed(&mut self, experiment_id: &str, phase_id: &str) -> Result<(), String> {
        if let Some(exp) = self.experiments.get_mut(experiment_id) {
            if !exp.failed_phases.contains(&phase_id.to_string()) {
                exp.failed_phases.push(phase_id.to_string());
            }
            self.save_to_disk()
        } else {
            Err(format!("Experiment {experiment_id} not found"))
        }
    }

    /// Get an experiment by ID
    pub fn get(&self, id: &str) -> Option<&Experiment> {
        self.experiments.get(id)
    }

    /// List all experiments, optionally filtered by status
    pub fn list(&self, status_filter: Option<ExperimentStatus>) -> Vec<&Experiment> {
        let mut all: Vec<&Experiment> = self.experiments.values()
            .filter(|e| {
                if let Some(ref filter) = status_filter {
                    e.status == *filter
                } else {
                    true
                }
            })
            .collect();
        all.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        all
    }

    /// Count experiments by status
    pub fn count_by_status(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for exp in self.experiments.values() {
            let status_str = format!("{:?}", exp.status);
            *counts.entry(status_str).or_insert(0) += 1;
        }
        counts
    }

    /// Process the next experiment in the queue autonomously.
    /// Runs all phases (PHASE-0001 through PHASE-0005 at minimum)
    /// and tracks progress. Returns the experiment ID processed, or None if empty.
    pub fn process_next(&mut self, state_dir: &Path) -> Result<Option<String>, String> {
        let experiment = match self.dequeue() {
            Some(e) => e,
            None => return Ok(None),
        };

        let exp_id = experiment.id.clone();
        let model_path = experiment.model_path.clone();
        eprintln!("  🎯 Processing experiment {exp_id}");
        eprintln!("  📦 Model: {model_path}");

        // Create artifact store
        let mut store = crate::runtime::phase::ArtifactStore::new(state_dir);
        let ctx = crate::runtime::phase::PhaseContext::new(&exp_id, state_dir);

        // Run phases in order (hardcoded — dependencies resolved by ordering)
        let phase_order = vec![
            "PHASE-0001-hardware",
            "PHASE-0002-runtime-discovery",
            "PHASE-0003-runtime-capabilities",
            "PHASE-0004-gguf-inspection",
            "PHASE-0005-memory-hypothesis",
        ];

        let mut success_count = 0;
        let mut fail_count = 0;

        for phase_id in &phase_order {
            // Get the phase
            let has_model = *phase_id == "PHASE-0004-gguf-inspection"
                || *phase_id == "PHASE-0005-memory-hypothesis";

            let phase: Box<dyn crate::runtime::phase::Phase> = if has_model {
                if *phase_id == "PHASE-0004-gguf-inspection" {
                    Box::new(crate::runtime::phase::phases::GgufInspectionPhase::new(&model_path))
                } else {
                    Box::new(crate::runtime::phase::phases::MemoryHypothesisPhase::new(&model_path))
                }
            } else {
                // Create fresh instances for standalone phases
                match *phase_id {
                    "PHASE-0001-hardware" => Box::new(crate::runtime::phase::phases::HardwarePhase),
                    "PHASE-0002-runtime-discovery" => Box::new(crate::runtime::phase::phases::RuntimeDiscoveryPhase),
                    "PHASE-0003-runtime-capabilities" => Box::new(crate::runtime::phase::phases::RuntimeCapabilitiesPhase),
                    _ => {
                        self.record_phase_failed(&exp_id, phase_id).ok();
                        fail_count += 1;
                        continue;
                    }
                }
            };

            // Execute the phase (dependencies resolved by phase ordering above)
            eprintln!("    🔬 Running {phase_id}...");
            let phase_start = std::time::Instant::now();
            match phase.execute(&ctx, &store) {
                Ok(mut output) => {
                    output.duration_ms = phase_start.elapsed().as_millis() as u64;
                    store.save_phase(&exp_id, &output).ok();
                    self.record_phase_complete(&exp_id, phase_id).ok();
                    success_count += 1;
                    eprintln!("    ✅ {phase_id} complete");
                }
                Err(e) => {
                    eprintln!("    ❌ {phase_id} failed: {e}");
                    self.record_phase_failed(&exp_id, phase_id).ok();
                    fail_count += 1;
                }
            }
        }

        // Build result
        let result = ExperimentResult {
            duration_seconds: 0.0, // Will be calculated from timestamps
            total_phases: phase_order.len(),
            completed_phases: success_count,
            failed_phases: fail_count,
            best_config: None, // Will be populated from PHASE-0005
            hardware_snapshot: None, // Will be populated from PHASE-0001
            runtimes_found: 0,
            models_discovered: vec![model_path.clone()],
        };

        if fail_count == 0 {
            self.complete(&exp_id, result).ok();
            eprintln!("  ✅ Experiment {exp_id} completed successfully ({success_count}/{success_count} phases)");
        } else {
            let err_msg = format!("{fail_count}/{success_count} phases failed");
            self.fail(&exp_id, &err_msg).ok();
            eprintln!("  ⚠ Experiment {exp_id} completed with {fail_count} failed phases");
        }

        Ok(Some(exp_id))
    }

    /// Remove completed/failed experiments older than `days`
    pub fn clean(&mut self, days: u64) -> usize {
        let cutoff_secs = days * 86400;
        use std::time::SystemTime;
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let to_remove: Vec<String> = self.experiments.iter()
            .filter(|(_, e)| {
                match e.status {
                    ExperimentStatus::Completed | ExperimentStatus::Failed(_) | ExperimentStatus::Cancelled => {
                        e.completed_at.map(|t| now - t > cutoff_secs).unwrap_or(true)
                    }
                    _ => false,
                }
            })
            .map(|(id, _)| id.clone())
            .collect();

        let count = to_remove.len();
        for id in to_remove {
            self.experiments.remove(&id);
            // Also remove the experiment directory
            let exp_dir = self.queue_dir.join("experiments").join(&id);
            if exp_dir.exists() {
                std::fs::remove_dir_all(&exp_dir).ok();
            }
        }
        if count > 0 {
            self.save_to_disk().ok();
        }
        count
    }

    // ── Persistence ──

    fn queue_file(&self) -> PathBuf {
        self.queue_dir.join("queue.json")
    }

    fn load_from_disk(&mut self) {
        let path = self.queue_file();
        if !path.exists() {
            return;
        }
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(experiments) = serde_json::from_str::<HashMap<String, Experiment>>(&content) {
                self.experiments = experiments;
            }
        }
    }

    fn save_to_disk(&self) -> Result<(), String> {
        let path = self.queue_file();
        let content = serde_json::to_string_pretty(&self.experiments)
            .map_err(|e| format!("Cannot serialize queue: {e}"))?;
        std::fs::write(&path, &content)
            .map_err(|e| format!("Cannot write queue: {e}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_enqueue_dequeue() {
        let dir = tempdir().unwrap();
        let mut queue = ExperimentQueue::load(dir.path());

        let exp = Experiment::new("EXP-001", "/models/test.gguf");
        queue.enqueue(exp).unwrap();

        assert_eq!(queue.list(None).len(), 1);

        let next = queue.dequeue();
        assert!(next.is_some());
        assert_eq!(next.unwrap().status, ExperimentStatus::Running);
    }

    #[test]
    fn test_persistence() {
        let dir = tempdir().unwrap();

        // Create and add experiments
        {
            let mut queue = ExperimentQueue::load(dir.path());
            queue.enqueue(Experiment::new("EXP-001", "/models/a.gguf")).unwrap();
            queue.enqueue(Experiment::new("EXP-002", "/models/b.gguf")).unwrap();
        }

        // Reload and verify persistence
        {
            let queue = ExperimentQueue::load(dir.path());
            assert_eq!(queue.list(None).len(), 2);
            assert!(queue.get("EXP-001").is_some());
            assert!(queue.get("EXP-002").is_some());
        }
    }

    #[test]
    fn test_complete_experiment() {
        let dir = tempdir().unwrap();
        let mut queue = ExperimentQueue::load(dir.path());

        queue.enqueue(Experiment::new("EXP-001", "/models/test.gguf")).unwrap();
        let exp = queue.dequeue().unwrap();
        assert_eq!(exp.status, ExperimentStatus::Running);

        let result = ExperimentResult {
            duration_seconds: 42.0,
            total_phases: 5,
            completed_phases: 5,
            failed_phases: 0,
            best_config: None,
            hardware_snapshot: None,
            runtimes_found: 3,
            models_discovered: vec!["qwen.gguf".to_string()],
        };
        queue.complete("EXP-001", result).unwrap();

        let completed = queue.get("EXP-001").unwrap();
        assert_eq!(completed.status, ExperimentStatus::Completed);
        assert!(completed.result.is_some());
    }

    #[test]
    fn test_retry_logic() {
        let dir = tempdir().unwrap();
        let mut queue = ExperimentQueue::load(dir.path());

        let mut exp = Experiment::new("EXP-001", "/models/test.gguf");
        exp.max_retries = 2;
        queue.enqueue(exp).unwrap();

        // Fail once — should be re-queued
        queue.fail("EXP-001", "OOM error").unwrap();
        assert_eq!(queue.get("EXP-001").unwrap().retry_count, 1);

        // Fail twice — should be re-queued
        queue.fail("EXP-001", "OOM error").unwrap();
        assert_eq!(queue.get("EXP-001").unwrap().retry_count, 2);

        // Fail third time — max retries exceeded, should be Failed
        queue.fail("EXP-001", "OOM error").unwrap();
        assert_eq!(queue.get("EXP-001").unwrap().status, ExperimentStatus::Failed("OOM error".to_string()));
    }

    #[test]
    fn test_clean_old_experiments() {
        let dir = tempdir().unwrap();
        let mut queue = ExperimentQueue::load(dir.path());

        queue.enqueue(Experiment::new("EXP-001", "/models/a.gguf")).unwrap();

        // Manually set completed_at to a very old time
        let old_time = 1000u64; // Way in the past
        if let Some(exp) = queue.experiments.get_mut("EXP-001") {
            exp.status = ExperimentStatus::Completed;
            exp.completed_at = Some(old_time);
            exp.result = Some(ExperimentResult {
                duration_seconds: 0.0,
                total_phases: 0, completed_phases: 0, failed_phases: 0,
                best_config: None, hardware_snapshot: None,
                runtimes_found: 0, models_discovered: vec![],
            });
        }

        // Clean experiments older than 0 days (essentially all)
        let cleaned = queue.clean(0);
        assert_eq!(cleaned, 1);
        assert!(queue.get("EXP-001").is_none());
    }
}
