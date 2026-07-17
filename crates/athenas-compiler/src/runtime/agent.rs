use std::path::PathBuf;

use serde::Serialize;

// ---------------------------------------------------------------------------
// AgentProvider — abstraction for agent implementations
// ---------------------------------------------------------------------------

/// Metadata describing an agent implementation
#[derive(Debug, Clone, Serialize)]
pub struct AgentMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub license: String,
    pub description: String,
    /// Whether this agent can run fully offline
    pub offline_capable: bool,
    /// Whether this agent requires a working directory
    pub requires_workspace: bool,
    /// Estimated context window needed (in tokens)
    pub context_window: usize,
}

/// Capabilities an agent can advertise
#[derive(Debug, Clone, Serialize)]
pub struct AgentCapabilities {
    /// Whether the agent can use tools (debugger, compiler, etc.)
    pub tool_use: bool,
    /// Whether the agent can browse the web
    pub web_search: bool,
    /// Whether the agent can execute code
    pub code_execution: bool,
    /// Whether the agent can read/write files
    pub file_operations: bool,
    /// Whether the agent can run multi-turn conversations
    pub multi_turn: bool,
    /// Whether the agent supports structured output (JSON mode)
    pub structured_output: bool,
    /// List of supported languages/capabilities
    pub supported_capabilities: Vec<String>,
}

/// Input to an agent execution
#[derive(Debug, Clone)]
pub struct AgentTask {
    /// The user's request
    pub prompt: String,
    /// Optional working directory
    pub working_dir: Option<PathBuf>,
    /// Maximum tokens to generate
    pub max_tokens: usize,
    /// Temperature for generation
    pub temperature: f64,
}

/// Result of an agent execution
#[derive(Debug, Clone, Serialize)]
pub struct AgentResult {
    /// The final response text
    pub response: String,
    /// Number of steps/turns the agent took
    pub steps: usize,
    /// Total tokens consumed
    pub total_tokens: usize,
    /// Total time in milliseconds
    pub total_duration_ms: f64,
    /// Any files that were created or modified
    pub files_changed: Vec<String>,
    /// Any commands that were executed
    pub commands_executed: Vec<String>,
}

/// Context provided to the agent from Athena's infrastructure
#[derive(Debug, Clone)]
pub struct AgentContext {
    /// The Engineering Graph (serialized as JSON for portability)
    pub engineering_graph: Option<serde_json::Value>,
    /// Available runtime information
    pub runtime_info: Option<String>,
    /// Available model information
    pub model_info: Option<String>,
    /// Knowledge packs context
    pub knowledge_context: Option<String>,
    /// Tool inventory
    pub tools: Vec<String>,
}

/// AgentProvider trait — every agent implementation must implement this.
/// Athena can support multiple agent implementations (native, OpenHands, Aider, etc.)
/// as long as they implement this interface.
pub trait AgentProvider: Send + Sync {
    /// Unique identifier for this agent type
    fn id(&self) -> &str;

    /// Human-readable metadata
    fn metadata(&self) -> AgentMetadata;

    /// What this agent can do
    fn capabilities(&self) -> AgentCapabilities;

    /// Execute a task with the given context.
    /// The provider is responsible for setting up runtime, loading knowledge,
    /// and managing the execution loop.
    fn execute(
        &self,
        task: AgentTask,
        context: AgentContext,
    ) -> anyhow::Result<AgentResult>;
}
