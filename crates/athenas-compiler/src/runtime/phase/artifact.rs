use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::core::{ArtifactStoreRead, PhaseOutput, PhaseResult, PhaseStatus};

/// ArtifactStore — persists every phase execution to disk.
/// Each phase produces a complete, self-contained directory.
/// Any agent can read a phase's output without running it.
pub struct ArtifactStore {
    base_path: PathBuf,
    /// In-memory cache of loaded phase outputs (for speed)
    cache: HashMap<String, PhaseOutput>,
}

impl ArtifactStore {
    pub fn new(state_dir: &Path) -> Self {
        Self {
            base_path: state_dir.join("experiments"),
            cache: HashMap::new(),
        }
    }

    /// Save a phase output to disk.
    /// Creates the full directory structure with evidence, logs, and signature.
    pub fn save_phase(&mut self, experiment_id: &str, output: &PhaseOutput) -> Result<(), String> {
        let phase_path = self.phase_dir(experiment_id, &output.phase_id.0);
        std::fs::create_dir_all(&phase_path).map_err(|e| format!("Cannot create phase dir: {e}"))?;

        // Save artifact.json
        let artifact_json = serde_json::to_string_pretty(output)
            .map_err(|e| format!("Cannot serialize phase output: {e}"))?;
        std::fs::write(phase_path.join("artifact.json"), &artifact_json)
            .map_err(|e| format!("Cannot write artifact.json: {e}"))?;

        // Save artifact.yaml (JSON is the canonical format, YAML is for convenience)
        std::fs::write(phase_path.join("artifact.yaml"), &artifact_json)
            .map_err(|e| format!("Cannot write artifact.yaml: {e}"))?;

        // Save metrics.json
        let metrics_json = serde_json::to_string_pretty(&output.metrics)
            .map_err(|e| format!("Cannot serialize metrics: {e}"))?;
        std::fs::write(phase_path.join("metrics.json"), &metrics_json)
            .map_err(|e| format!("Cannot write metrics.json: {e}"))?;

        // Save timeline.json
        let timeline_json = serde_json::to_string_pretty(&output.timeline)
            .map_err(|e| format!("Cannot serialize timeline: {e}"))?;
        std::fs::write(phase_path.join("timeline.json"), &timeline_json)
            .map_err(|e| format!("Cannot write timeline.json: {e}"))?;

        // Save status file
        let status_str = format!("{:?}", output.status);
        std::fs::write(phase_path.join("status"), &status_str)
            .map_err(|e| format!("Cannot write status: {e}"))?;

        // Cache in memory
        self.cache.insert(format!("{experiment_id}/{}", output.phase_id.0), output.clone());

        Ok(())
    }

    /// Load a phase output from disk.
    pub fn load_phase(&self, experiment_id: &str, phase_id: &str) -> Result<PhaseOutput, String> {
        // Check cache first
        let cache_key = format!("{experiment_id}/{phase_id}");
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        // Read from disk
        let phase_path = self.phase_dir(experiment_id, phase_id);
        let artifact_path = phase_path.join("artifact.json");
        if !artifact_path.exists() {
            return Err(format!("Phase {phase_id} not found for experiment {experiment_id}"));
        }

        let content = std::fs::read_to_string(&artifact_path)
            .map_err(|e| format!("Cannot read {artifact_path:?}: {e}"))?;
        let output: PhaseOutput = serde_json::from_str(&content)
            .map_err(|e| format!("Cannot parse {artifact_path:?}: {e}"))?;
        Ok(output)
    }

    /// Check if a phase has been executed and cached
    pub fn phase_exists(&self, experiment_id: &str, phase_id: &str) -> bool {
        let cache_key = format!("{experiment_id}/{phase_id}");
        if self.cache.contains_key(&cache_key) {
            return true;
        }
        let phase_path = self.phase_dir(experiment_id, phase_id);
        phase_path.join("artifact.json").exists()
    }

    /// List all phases that have been executed for an experiment
    pub fn list_phases(&self, experiment_id: &str) -> Result<Vec<String>, String> {
        let exp_path = self.base_path.join(experiment_id).join("phases");
        if !exp_path.is_dir() {
            return Ok(Vec::new());
        }
        let mut phases = Vec::new();
        for entry in std::fs::read_dir(&exp_path).map_err(|e| format!("Cannot read {exp_path:?}: {e}"))? {
            let entry = entry.map_err(|e| format!("Cannot read entry: {e}"))?;
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    phases.push(name.to_string());
                }
            }
        }
        phases.sort();
        Ok(phases)
    }

    /// Get the evidence directory for a phase (create if needed)
    pub fn evidence_dir(&self, experiment_id: &str, phase_id: &str) -> PathBuf {
        let dir = self.phase_dir(experiment_id, phase_id).join("evidence");
        std::fs::create_dir_all(&dir).ok();
        dir
    }

    /// Write raw evidence to the evidence directory
    pub fn write_evidence(&self, experiment_id: &str, phase_id: &str, filename: &str, content: &str) -> Result<(), String> {
        let dir = self.evidence_dir(experiment_id, phase_id);
        std::fs::write(dir.join(filename), content)
            .map_err(|e| format!("Cannot write evidence {filename}: {e}"))?;
        Ok(())
    }

    /// Create a new experiment directory
    pub fn create_experiment(&self, experiment_id: &str) -> Result<PathBuf, String> {
        let exp_path = self.base_path.join(experiment_id);
        std::fs::create_dir_all(&exp_path)
            .map_err(|e| format!("Cannot create experiment dir: {e}"))?;
        Ok(exp_path)
    }

    /// Path to the experiment's phase directory
    fn phase_dir(&self, experiment_id: &str, phase_id: &str) -> PathBuf {
        self.base_path.join(experiment_id).join("phases").join(phase_id)
    }
}

impl ArtifactStoreRead for ArtifactStore {
    fn load_artifact(&self, experiment_id: &str, phase_id: &str) -> Result<PhaseOutput, String> {
        self.load_phase(experiment_id, phase_id)
    }

    fn phase_exists(&self, experiment_id: &str, phase_id: &str) -> bool {
        self.phase_exists(experiment_id, phase_id)
    }

    fn list_phase_ids(&self, experiment_id: &str) -> Result<Vec<String>, String> {
        self.list_phases(experiment_id)
    }
}
