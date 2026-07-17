use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

/// Unique phase identifier (e.g., "PHASE-0001-hardware")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseId(pub String);

impl PhaseId {
    pub fn new(id: &str) -> Self { Self(id.to_string()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

/// Status of a phase execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PhaseStatus {
    NotExecuted,
    Running,
    Success,
    Failure(String),
    Skipped(String),  // reason for skipping (e.g., "Cached")
}

/// A single event in the phase timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub timestamp_ms: u64,
    pub event: String,
    pub detail: String,
}

/// Metrics extracted from phase execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseMetrics {
    pub values: HashMap<String, f64>,
    pub units: HashMap<String, String>,
}

impl PhaseMetrics {
    pub fn new() -> Self {
        Self { values: HashMap::new(), units: HashMap::new() }
    }
    pub fn add(&mut self, key: &str, value: f64, unit: &str) {
        self.values.insert(key.to_string(), value);
        self.units.insert(key.to_string(), unit.to_string());
    }
}

/// Complete output from a phase execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseOutput {
    pub phase_id: PhaseId,
    pub experiment_id: String,

    /// Structured answer to the phase's question
    pub artifact: serde_json::Value,

    /// Status
    pub status: PhaseStatus,

    /// Timeline of events during execution
    pub timeline: Vec<TimelineEvent>,

    /// Extracted metrics
    pub metrics: PhaseMetrics,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// When the phase was executed
    pub executed_at: u64,  // unix timestamp

    /// Path to the evidence directory
    pub evidence_dir: PathBuf,

    /// Path to raw logs (stdout/stderr)
    pub raw_log_path: Option<PathBuf>,
}

impl PhaseOutput {
    pub fn new(phase_id: PhaseId, experiment_id: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            phase_id,
            experiment_id: experiment_id.to_string(),
            artifact: serde_json::Value::Null,
            status: PhaseStatus::Running,
            timeline: Vec::new(),
            metrics: PhaseMetrics::new(),
            duration_ms: 0,
            executed_at: now,
            evidence_dir: PathBuf::new(),
            raw_log_path: None,
        }
    }

    pub fn record_event(&mut self, event: &str, detail: &str) {
        let total_ms = self.timeline.iter().map(|e| e.timestamp_ms).sum::<u64>();
        self.timeline.push(TimelineEvent {
            timestamp_ms: total_ms,
            event: event.to_string(),
            detail: detail.to_string(),
        });
    }
}

/// Context passed to a phase during execution
pub struct PhaseContext {
    pub experiment_id: String,
    pub state_dir: PathBuf,
    pub hardware: serde_json::Value,
}

impl PhaseContext {
    pub fn new(experiment_id: &str, state_dir: &Path) -> Self {
        Self {
            experiment_id: experiment_id.to_string(),
            state_dir: state_dir.to_path_buf(),
            hardware: serde_json::Value::Null,
        }
    }
}

/// Result from executing a phase
#[derive(Debug, Clone, Serialize)]
pub struct PhaseResult {
    pub phase_id: String,
    pub status: PhaseStatus,
    pub artifact: serde_json::Value,
    pub timeline: Vec<TimelineEvent>,
    pub metrics: PhaseMetrics,
    pub duration_ms: u64,
}

// ═══════════════════════════════════════════════════════════════
// Phase Trait — The Core Abstraction
// ═══════════════════════════════════════════════════════════════

/// A single phase in the experiment pipeline.
/// Each phase answers ONE scientific question.
pub trait Phase: Send + Sync {
    /// Unique identifier (e.g., "PHASE-0001-hardware")
    fn id(&self) -> &str;

    /// The scientific question this phase answers
    fn question(&self) -> &str;

    /// Input dependencies (other phase IDs)
    fn inputs(&self) -> Vec<&str> {
        Vec::new()
    }

    /// Execute this phase and return structured output.
    /// `store` provides access to previous phase artifacts.
    fn execute(&self, ctx: &PhaseContext, store: &dyn ArtifactStoreRead) -> Result<PhaseOutput, String>;

    /// Human-readable name for display
    fn name(&self) -> &str {
        self.id()
    }

    /// Description of what this phase does
    fn description(&self) -> &str {
        ""
    }
}

/// Read-only view of the artifact store (for phase execution)
pub trait ArtifactStoreRead {
    fn load_artifact(&self, experiment_id: &str, phase_id: &str) -> Result<PhaseOutput, String>;
    fn phase_exists(&self, experiment_id: &str, phase_id: &str) -> bool;
    fn list_phase_ids(&self, experiment_id: &str) -> Result<Vec<String>, String>;
}
