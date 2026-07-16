use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------------------------------------------------------------------------
// Certification Levels (L0-L5)
// ---------------------------------------------------------------------------

/// Certification level corresponding to Athena's architectural layers.
/// Each level adds exactly one thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BenchmarkLevel {
    /// Raw model — no augmentation
    L0,
    /// + Knowledge Pack (structured domain knowledge in system prompt)
    L1,
    /// + Workspace (engineering environment config + system prompt)
    L2,
    /// + Tools (tool descriptions and availability)
    L3,
    /// + Agent Loop (iterative plan → execute → observe → repair)
    L4,
    /// + Experience Cache (accumulated procedural memory)
    L5,
}

impl BenchmarkLevel {
    pub fn all() -> &'static [BenchmarkLevel] {
        &[
            BenchmarkLevel::L0,
            BenchmarkLevel::L1,
            BenchmarkLevel::L2,
            BenchmarkLevel::L3,
            BenchmarkLevel::L4,
            BenchmarkLevel::L5,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            BenchmarkLevel::L0 => "Raw",
            BenchmarkLevel::L1 => "+ Knowledge",
            BenchmarkLevel::L2 => "+ Workspace",
            BenchmarkLevel::L3 => "+ Tools",
            BenchmarkLevel::L4 => "+ Agent Loop",
            BenchmarkLevel::L5 => "+ Experience",
        }
    }

    pub fn short(&self) -> &'static str {
        match self {
            BenchmarkLevel::L0 => "L0",
            BenchmarkLevel::L1 => "L1",
            BenchmarkLevel::L2 => "L2",
            BenchmarkLevel::L3 => "L3",
            BenchmarkLevel::L4 => "L4",
            BenchmarkLevel::L5 => "L5",
        }
    }

    pub fn from_u8(v: u8) -> Option<BenchmarkLevel> {
        match v {
            0 => Some(BenchmarkLevel::L0),
            1 => Some(BenchmarkLevel::L1),
            2 => Some(BenchmarkLevel::L2),
            3 => Some(BenchmarkLevel::L3),
            4 => Some(BenchmarkLevel::L4),
            5 => Some(BenchmarkLevel::L5),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Benchmark Metadata
// ---------------------------------------------------------------------------

/// Human-readable metadata about a benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub task_count: usize,
    pub max_level: u8,
    pub languages: Vec<String>,
    pub requires_docker: bool,
    pub requires_network: bool,
    pub estimated_duration_minutes: u32,
    pub tier: String,
}

// ---------------------------------------------------------------------------
// Task and result types
// ---------------------------------------------------------------------------

/// A single task within a benchmark.
/// The runner only defines the task and how to validate it.
/// The certification engine builds the actual prompts at each level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTask {
    pub id: String,
    pub description: String,
    /// The raw task prompt (no level augmentation).
    /// Certify will wrap this with L0..L3 context.
    pub prompt: String,
    /// Strings that MUST appear in the model output for the task to pass.
    pub required_elements: Vec<String>,
    /// Strings that MUST NOT appear in the model output.
    pub forbidden_elements: Vec<String>,
    /// How to validate: "code" | "contains" | "exact_match" | "function_defined"
    pub validation_type: String,
    /// Reference solution (for documentation, not evaluation)
    pub reference: Option<String>,
    /// Language hint for code tasks
    pub language: Option<String>,
}

/// Result of executing a single task at a single certification level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTaskResult {
    pub task_id: String,
    pub level: BenchmarkLevel,
    pub passed: bool,
    pub model_output: String,
    pub match_details: Vec<String>,
    pub ttft_ms: f64,
    pub tokens_per_second: f64,
    pub total_tokens: usize,
}

/// Aggregated metrics for one certification level across all tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelResult {
    pub level: BenchmarkLevel,
    pub level_name: String,
    pub pass_rate: f64,
    pub passed: usize,
    pub total: usize,
    pub avg_ttft_ms: f64,
    pub avg_tokens_per_second: f64,
    pub total_tokens: usize,
    pub total_duration_ms: f64,
}

// ---------------------------------------------------------------------------
// Certification Report
// ---------------------------------------------------------------------------

/// Complete certification report for a benchmark.
/// This is the ONLY place reports are generated — runners never write reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationReport {
    pub benchmark_id: String,
    pub benchmark_name: String,
    pub tier: String,
    pub model: serde_json::Value,
    pub hardware: serde_json::Value,
    pub levels: Vec<LevelResult>,
    /// Capability gain from L0 to the highest executed level (percentage points)
    pub total_gain: f64,
    pub total_duration_ms: f64,
    pub timestamp: String,
    pub config: serde_json::Value,
}

impl CertificationReport {
    /// Generate a pretty-printed human-readable report
    pub fn to_pretty_string(&self) -> String {
        let mut s = String::new();
        s.push_str("╔══════════════════════════════════════════════════════════╗\n");
        s.push_str(&format!(
            "║  {:<56}║\n",
            format!("Athenas Certification — {}", self.benchmark_name)
        ));
        s.push_str("╚══════════════════════════════════════════════════════════╝\n");
        s.push('\n');
        s.push_str(&format!("  🎯  Benchmark: {}\n", self.benchmark_name));
        s.push_str(&format!("  📋  Model:     {}\n", self.model_path()));
        s.push_str(&format!("  📏  Params:    {:.0}B\n", self.model_params()));
        s.push_str(&format!("  🏷️   Tier:      {}\n", self.tier));
        s.push('\n');

        // Level results table
        s.push_str("📊 CAPABILITY REPORT — Per-Layer Contribution\n");
        s.push_str(&"═".repeat(75));
        s.push('\n');
        s.push_str(&format!(
            "  {:<6} {:<20} {:>14} {:>12} {:>12}\n",
            "Level", "Configuration", "Pass Rate", "Tasks", "TTFT (ms)"
        ));
        s.push_str(&"─".repeat(75));
        s.push('\n');

        let mut prev_pass_rate: Option<f64> = None;
        for level in &self.levels {
            let delta = match prev_pass_rate {
                Some(prev) if prev >= 0.0 => {
                    let change = ((level.pass_rate - prev) * 100.0).round();
                    format!("{:>+7.1}%", change)
                }
                _ => "    base".to_string(),
            };
            s.push_str(&format!(
                "  {:<6} {:<20} {:>8.1}%  {:>4}/{:<4} {:>8.1}  {}\n",
                level.level.short(),
                level.level_name,
                level.pass_rate * 100.0,
                level.passed,
                level.total,
                level.avg_ttft_ms,
                delta
            ));
            prev_pass_rate = Some(level.pass_rate);
        }
        s.push_str(&"═".repeat(75));
        s.push('\n');
        s.push('\n');

        // Summary
        let first = self.levels.first();
        let last = self.levels.last();
        if let (Some(first), Some(last)) = (first, last) {
            let gain = (last.pass_rate - first.pass_rate) * 100.0;
            s.push_str(&format!(
                "  📈 Capability gain:  {:+.1} percentage points (L0 → {})\n",
                gain,
                last.level.short()
            ));
        }
        s.push_str(&format!(
            "  ⏱   Total duration: {:.0}s\n",
            self.total_duration_ms / 1000.0
        ));
        s.push('\n');
        s.push_str(&format!("  {} Certification complete!\n", "✓"));

        s
    }

    fn model_path(&self) -> String {
        self.model["path"]
            .as_str()
            .map(|s| {
                Path::new(s)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| s.to_string())
            })
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn model_params(&self) -> f64 {
        self.model["parameters_b"].as_f64().unwrap_or(0.0)
    }
}

// ---------------------------------------------------------------------------
// BenchmarkRunner Trait
// ---------------------------------------------------------------------------

/// Every benchmark implements this trait.
///
/// The runner describes HOW to execute a benchmark:
/// - What tasks exist
/// - How to prepare/setup each task
/// - How to validate the model's output
/// - How to clean up
///
/// The runner NEVER:
/// - Writes certification reports
/// - Computes per-level comparisons
/// - Interacts with the model directly
///
/// Those belong to the certification engine.
pub trait BenchmarkRunner: Send + Sync {
    /// Unique identifier (e.g., "human-eval", "swe-bench")
    fn id(&self) -> &str;

    /// Human-readable metadata about this benchmark
    fn metadata(&self) -> BenchmarkMetadata;

    /// Discover all tasks in this benchmark
    fn discover_tasks(&self) -> Result<Vec<BenchmarkTask>, String>;

    /// Prepare the environment for a specific task (e.g., Docker, git checkout)
    /// Returns an error if setup fails
    fn prepare_environment(&self, task: &BenchmarkTask) -> Result<(), String> {
        let _ = task; // default: no-op
        Ok(())
    }

    /// Validate a model's output against a task.
    /// Returns true if the output passes the task requirements.
    fn validate(&self, task: &BenchmarkTask, model_output: &str) -> Result<bool, String>;

    /// Clean up after a task (e.g., stop containers, remove temp files)
    fn teardown(&self, task: &BenchmarkTask) -> Result<(), String> {
        let _ = task; // default: no-op
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Benchmark Registry
// ---------------------------------------------------------------------------

/// Registry of all available benchmark runners.
/// Adding a new benchmark means registering a new runner here — never modifying certify.
#[derive(Default)]
pub struct BenchmarkRegistry {
    runners: Vec<Box<dyn BenchmarkRunner>>,
}

impl BenchmarkRegistry {
    pub fn new() -> Self {
        Self {
            runners: Vec::new(),
        }
    }

    /// Register a benchmark runner
    pub fn register(&mut self, runner: Box<dyn BenchmarkRunner>) {
        self.runners.push(runner);
    }

    /// Find a runner by ID
    pub fn get(&self, id: &str) -> Option<&dyn BenchmarkRunner> {
        self.runners.iter().find(|r| r.id() == id).map(|r| r.as_ref())
    }

    /// List all registered benchmark IDs
    pub fn list(&self) -> Vec<&str> {
        self.runners.iter().map(|r| r.id()).collect()
    }

    /// List all registered benchmarks with metadata
    pub fn list_with_metadata(&self) -> Vec<BenchmarkMetadata> {
        self.runners.iter().map(|r| r.metadata()).collect()
    }

    /// Build a default registry with built-in benchmarks.
    /// Note: callers should register runners explicitly since
    /// module visibility varies. This provides an empty default.
    pub fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Default validate helper (used by simpler runners)
// ---------------------------------------------------------------------------

/// Default validation: check if required elements are present and forbidden
/// elements are absent in the model output.
pub fn default_validate(task: &BenchmarkTask, model_output: &str) -> Result<bool, String> {
    let mut passed = true;
    let mut missing = Vec::new();

    for required in &task.required_elements {
        if !model_output.contains(required) {
            passed = false;
            missing.push(required.clone());
        }
    }

    for forbidden in &task.forbidden_elements {
        if model_output.contains(forbidden) {
            passed = false;
        }
    }

    if !passed && !missing.is_empty() {
        return Err(format!(
            "Missing required elements: {:?}",
            missing
        ));
    }

    Ok(passed)
}

// ---------------------------------------------------------------------------
// Certification Engine
// ---------------------------------------------------------------------------

/// Run a full multi-level certification for a given benchmark and model.
///
/// This is the core pipeline:
/// 1. Build prompts at each level (L0..L3)
/// 2. Execute each task at each level
/// 3. Aggregate results per level
/// 4. Generate the certification report
///
/// The runner provides tasks and validation. The engine handles everything else.
pub fn run_benchmark_certify(
    runner: &dyn BenchmarkRunner,
    level: u8,
    pack_id: Option<&str>,
    workspace_path: Option<&Path>,
    max_tokens: usize,
    model_path: &Path,
    server_path: Option<&Path>,
    json_output: bool,
) -> anyhow::Result<CertificationReport> {
    use crate::runtime::{knowledge, Runtime};

    let metadata = runner.metadata();
    let max_level = level.min(metadata.max_level);

    // Get the benchmark tasks
    let tasks = runner
        .discover_tasks()
        .map_err(|e| anyhow::anyhow!("Failed to discover tasks: {e}"))?;

    // Resolve model + hardware
    let model = crate::runtime::find_model(Some(model_path))?;
    let hw = crate::runtime::hardware::detect_hardware();
    let info = crate::runtime::infer_model_info(&model, Some(&hw));

    // Load knowledge pack + workspace for L1/L2/L3 prompts
    let l1_system = pack_id.and_then(|pid| {
        let dir = knowledge::default_packs_dir();
        let packs = knowledge::discover_packs(&dir);
        knowledge::find_pack(&packs, pid).map(|p| knowledge::build_system_prompt(p, None))
    });

    let l2_system = workspace_path.and_then(|w| {
        let config_path = w.join("athenas.json");
        std::fs::read_to_string(config_path).ok().and_then(|content| {
            serde_json::from_str::<serde_json::Value>(&content)
                .ok()
                .and_then(|v| v["system_prompt"].as_str().map(|s| s.to_string()))
        })
    });

    // L3 tools string
    let l3_tools = pack_id.and_then(|pid| {
        let dir = knowledge::default_packs_dir();
        let packs = knowledge::discover_packs(&dir);
        knowledge::find_pack(&packs, pid).map(|p| {
            let tools = knowledge::check_tools(p);
            let mut desc = String::from("\n## Available Tools\n");
            for t in &tools {
                let status = if t.installed { "✓" } else { "✗" };
                desc.push_str(&format!("- {}: {} ({status})\n", t.name, t.description));
            }
            desc
        })
    }).unwrap_or_default();

    // Build prompts for each level from the base task prompt
    let build_level_prompt = |task_prompt: &str, level: BenchmarkLevel| -> String {
        match level {
            BenchmarkLevel::L0 => format!("### Task\n\n{task_prompt}"),
            BenchmarkLevel::L1 => match &l1_system {
                Some(sp) => format!("{sp}\n\n### Task\n\n{task_prompt}"),
                None => format!("### Task\n\n{task_prompt}"),
            },
            BenchmarkLevel::L2 => match &l2_system {
                Some(wp) => format!("{wp}\n\n### Task\n\n{task_prompt}"),
                None => match &l1_system {
                    Some(sp) => format!("{sp}\n\n### Task\n\n{task_prompt}"),
                    None => format!("### Task\n\n{task_prompt}"),
                },
            },
            BenchmarkLevel::L3 => {
                let base = l2_system
                    .as_deref()
                    .or(l1_system.as_deref())
                    .unwrap_or("");
                format!("{base}\n{l3_tools}\n\n### Task\n\n{task_prompt}")
            }
            _ => format!("### Task\n\n{task_prompt}"), // L4+ not implemented yet
        }
    };

    // Determine which levels to run
    let levels_to_run: Vec<BenchmarkLevel> = BenchmarkLevel::all()
        .iter()
        .copied()
        .take((max_level + 1) as usize)
        .collect();

    // Build and start runtime
    let mut rt_builder = crate::runtime::LlamaServerRuntime::new();
    if let Some(sp) = server_path {
        rt_builder = rt_builder.with_server_path(sp.to_path_buf());
    }
    let mut rt = rt_builder;

    let cert_start = std::time::Instant::now();

    if !json_output {
        println!("╔══════════════════════════════════════════╗");
        println!("║  Athenas Benchmark Engine v0.1.0        ║");
        println!("╚══════════════════════════════════════════╝");
        println!();
        println!("🎯 Benchmark: {}", metadata.name);
        println!("📋 Model:     {}", info.path);
        println!("📏 Tasks:     {} (running L0..L{})", tasks.len(), max_level);
        println!();
    }

    rt.load_model(&model)?;

    if !json_output {
        println!("  ✅ Model loaded");
        println!();
    }

    // For each level, run all tasks
    let mut level_results: Vec<LevelResult> = Vec::new();

    for (idx, level) in levels_to_run.iter().enumerate() {
        if !json_output {
            println!("🧪 {}: {}", level.short(), level.name());
        }

        let level_start = std::time::Instant::now();
        let mut task_results: Vec<BenchmarkTaskResult> = Vec::new();

        for task in &tasks {
            let prompt = build_level_prompt(&task.prompt, *level);

            // Send to model
            let rt_start = std::time::Instant::now();
            let result = crate::runtime::run_benchmark(&rt, &prompt, max_tokens);
            let rt_duration = rt_start.elapsed().as_secs_f64() * 1000.0;

            let task_result = match result {
                Ok(inference) => {
                    // Validate
                    let passed = runner
                        .validate(task, &inference.text)
                        .unwrap_or(false);

                    BenchmarkTaskResult {
                        task_id: task.id.clone(),
                        level: *level,
                        passed,
                        model_output: inference.text.clone(),
                        match_details: vec![],
                        ttft_ms: inference.ttft_ms,
                        tokens_per_second: inference.tokens_per_second,
                        total_tokens: inference.total_tokens,
                    }
                }
                Err(e) => BenchmarkTaskResult {
                    task_id: task.id.clone(),
                    level: *level,
                    passed: false,
                    model_output: format!("Error: {e}"),
                    match_details: vec![],
                    ttft_ms: 0.0,
                    tokens_per_second: 0.0,
                    total_tokens: 0,
                },
            };

            task_results.push(task_result);
        }

        let level_duration = level_start.elapsed().as_secs_f64() * 1000.0;

        // Aggregate results for this level
        let passed_count = task_results.iter().filter(|r| r.passed).count();
        let total_count = task_results.len();
        let pass_rate = if total_count > 0 {
            passed_count as f64 / total_count as f64
        } else {
            0.0
        };
        let avg_ttft = if total_count > 0 {
            task_results.iter().map(|r| r.ttft_ms).sum::<f64>() / total_count as f64
        } else {
            0.0
        };
        let avg_tps = if total_count > 0 {
            task_results
                .iter()
                .map(|r| r.tokens_per_second)
                .sum::<f64>()
                / total_count as f64
        } else {
            0.0
        };
        let total_tok: usize = task_results.iter().map(|r| r.total_tokens).sum();

        level_results.push(LevelResult {
            level: *level,
            level_name: level.name().to_string(),
            pass_rate,
            passed: passed_count,
            total: total_count,
            avg_ttft_ms: avg_ttft,
            avg_tokens_per_second: avg_tps,
            total_tokens: total_tok,
            total_duration_ms: level_duration,
        });

        if !json_output {
            println!("     Pass rate: {:.1}% ({}/{})", pass_rate * 100.0, passed_count, total_count);
            println!("     Avg TTFT: {:.1}ms | TPS: {:.1}", avg_ttft, avg_tps);
            println!();
        }

        // Reload model between levels (except last)
        if idx + 1 < levels_to_run.len() {
            rt.unload()?;
            rt.load_model(&model)?;
        }
    }

    rt.unload()?;

    let total_duration = cert_start.elapsed().as_secs_f64() * 1000.0;

    // Compute total gain
    let first_pass = level_results.first().map(|l| l.pass_rate).unwrap_or(0.0);
    let last_pass = level_results.last().map(|l| l.pass_rate).unwrap_or(0.0);
    let total_gain = (last_pass - first_pass) * 100.0;

    let report = CertificationReport {
        benchmark_id: metadata.id.clone(),
        benchmark_name: metadata.name.clone(),
        tier: metadata.tier.clone(),
        model: serde_json::json!({
            "id": info.id,
            "path": info.path,
            "parameters_b": info.parameters_b,
            "quantization": info.quantization,
            "architecture": info.architecture,
            "context_length": info.context_length,
        }),
        hardware: serde_json::to_value(&hw).unwrap_or(serde_json::Value::Null),
        levels: level_results,
        total_gain: (total_gain * 10.0).round() / 10.0,
        total_duration_ms: total_duration,
        timestamp: chrono::Utc::now().to_rfc3339(),
        config: serde_json::json!({
            "level": level,
            "max_tokens": max_tokens,
            "pack_id": pack_id,
            "workspace": workspace_path.map(|p| p.to_string_lossy().to_string()),
            "server_path": server_path.map(|p| p.to_string_lossy().to_string()),
        }),
    };

    if json_output {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("{}", report.to_pretty_string());
    }

    Ok(report)
}
