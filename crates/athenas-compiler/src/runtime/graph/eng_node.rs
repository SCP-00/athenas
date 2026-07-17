use serde::{Deserialize, Serialize};

/// Status of a runtime process
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuntimeStatus {
    Available,   // Binary exists in PATH
    Running,     // Health check passed
    NotRunning,  // Binary exists but not running
    NotFound,    // Not installed
}

/// All node types in the Engineering Graph.
/// Closed enum (not trait objects) for compile-time exhaustiveness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngNodeData {
    // Project
    Project { name: String, path: String },
    File { path: String, language: String },

    // Language / Toolchain
    Language { name: String, version: String },
    Compiler { name: String, version: String },
    Debugger { name: String, version: String },
    Formatter { name: String, version: String },
    Linter { name: String, version: String },
    TestRunner { name: String, version: String },
    PackageManager { name: String, version: String },

    // Runtime
    Runtime { name: String, status: RuntimeStatus, endpoint: Option<String> },

    // Model
    Model { name: String, path: String, params_b: f64, quant: String },

    // Hardware
    Cpu { model: String, cores: usize, threads: usize },
    Gpu { model: String, vram_gb: f64, driver: String },
    Ram { total_gb: f64, available_gb: f64 },
    Os { name: String, version: String, arch: String },

    // Knowledge
    KnowledgePack { id: String, name: String, version: String, language: String },
}

impl EngNodeData {
    pub fn label(&self) -> &str {
        match self {
            EngNodeData::Project { .. } => "Project",
            EngNodeData::File { .. } => "File",
            EngNodeData::Language { .. } => "Language",
            EngNodeData::Compiler { .. } => "Compiler",
            EngNodeData::Debugger { .. } => "Debugger",
            EngNodeData::Formatter { .. } => "Formatter",
            EngNodeData::Linter { .. } => "Linter",
            EngNodeData::TestRunner { .. } => "TestRunner",
            EngNodeData::PackageManager { .. } => "PackageManager",
            EngNodeData::Runtime { .. } => "Runtime",
            EngNodeData::Model { .. } => "Model",
            EngNodeData::Cpu { .. } => "CPU",
            EngNodeData::Gpu { .. } => "GPU",
            EngNodeData::Ram { .. } => "RAM",
            EngNodeData::Os { .. } => "OS",
            EngNodeData::KnowledgePack { .. } => "KnowledgePack",
        }
    }

    pub fn summary(&self) -> String {
        match self {
            EngNodeData::Project { name, .. } => name.clone(),
            EngNodeData::File { path, .. } => path.clone(),
            EngNodeData::Language { name, version } => format!("{name} {version}"),
            EngNodeData::Compiler { name, version } => format!("{name} {version}"),
            EngNodeData::Debugger { name, version } => format!("{name} {version}"),
            EngNodeData::Formatter { name, version } => format!("{name} {version}"),
            EngNodeData::Linter { name, version } => format!("{name} {version}"),
            EngNodeData::TestRunner { name, version } => format!("{name} {version}"),
            EngNodeData::PackageManager { name, version } => format!("{name} {version}"),
            EngNodeData::Runtime { name, status, .. } => {
                let s = match status {
                    RuntimeStatus::Running => "running",
                    RuntimeStatus::Available => "available",
                    RuntimeStatus::NotRunning => "not running",
                    RuntimeStatus::NotFound => "not found",
                };
                format!("{name} ({s})")
            }
            EngNodeData::Model { name, params_b, quant, .. } => format!("{name} ({params_b}B, {quant})"),
            EngNodeData::Cpu { model, cores, threads } => format!("{model} ({cores}c/{threads}t)"),
            EngNodeData::Gpu { model, vram_gb, .. } => format!("{model} ({vram_gb} GB)"),
            EngNodeData::Ram { total_gb, .. } => format!("{total_gb} GB"),
            EngNodeData::Os { name, version, arch } => format!("{name} {version} ({arch})"),
            EngNodeData::KnowledgePack { name, version, language, .. } => {
                format!("{name} v{version} ({language})")
            }
        }
    }
}

/// A node in the Engineering Graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngNode {
    pub id: String,
    pub data: EngNodeData,
}

impl EngNode {
    pub fn new(id: String, data: EngNodeData) -> Self {
        Self { id, data }
    }
}

/// Edge kinds
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EngEdgeKind {
    Contains,
    DependsOn,
    HasTool,
    RunsOn,
    UsesGpu,
    UsesMemory,
    HasKnowledge,
    DetectedBy,
    HasPackageManager,
    CompiledBy,
    DebuggedBy,
    FormattedBy,
    LintedBy,
    TestedBy,
}

/// An edge in the Engineering Graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngEdge {
    pub source: String,
    pub target: String,
    pub kind: EngEdgeKind,
}

impl EngEdge {
    pub fn new(source: String, target: String, kind: EngEdgeKind) -> Self {
        Self { source, target, kind }
    }
}
