use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Certification Levels (L0-L5)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BenchmarkLevel {
    L0, L1, L2, L3, L4, L5,
}

impl BenchmarkLevel {
    pub fn all() -> &'static [BenchmarkLevel] {
        &[BenchmarkLevel::L0, BenchmarkLevel::L1, BenchmarkLevel::L2,
          BenchmarkLevel::L3, BenchmarkLevel::L4, BenchmarkLevel::L5]
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
            BenchmarkLevel::L0 => "L0", BenchmarkLevel::L1 => "L1",
            BenchmarkLevel::L2 => "L2", BenchmarkLevel::L3 => "L3",
            BenchmarkLevel::L4 => "L4", BenchmarkLevel::L5 => "L5",
        }
    }

    pub fn from_u8(v: u8) -> Option<BenchmarkLevel> {
        match v { 0 => Some(BenchmarkLevel::L0), 1 => Some(BenchmarkLevel::L1),
                  2 => Some(BenchmarkLevel::L2), 3 => Some(BenchmarkLevel::L3),
                  4 => Some(BenchmarkLevel::L4), 5 => Some(BenchmarkLevel::L5), _ => None }
    }
}

// ---------------------------------------------------------------------------
// Benchmark Metadata
// ---------------------------------------------------------------------------

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTask {
    pub id: String,
    pub description: String,
    pub prompt: String,
    /// Strings that MUST appear in the model output for the task to pass.
    pub required_elements: Vec<String>,
    /// Strings that MUST NOT appear in the model output.
    pub forbidden_elements: Vec<String>,
    /// How to validate: "exact_match" | "numeric_approximate" | "structural" | "execution" | "human_review"
    pub validation_type: String,
    /// Known correct answer (for exact_match, numeric_approximate, or human reference)
    pub known_answer: Option<String>,
    /// Tolerance for numeric_approximate validation (e.g., 0.01 for 1% error)
    pub tolerance: Option<f64>,
    /// Reference solution (for documentation, not evaluation)
    pub reference: Option<String>,
    /// Language hint for code tasks
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTaskResult {
    pub task_id: String,
    pub level: BenchmarkLevel,
    pub passed: bool,
    pub score: f64,
    pub model_output: String,
    pub match_details: Vec<String>,
    pub ttft_ms: f64,
    pub tokens_per_second: f64,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelResult {
    pub level: BenchmarkLevel,
    pub level_name: String,
    /// Overall pass rate (tasks with score >= 0.5)
    pub pass_rate: f64,
    /// Average score across all tasks (0.0 - 1.0)
    pub avg_score: f64,
    pub passed: usize,
    pub total: usize,
    pub avg_ttft_ms: f64,
    pub avg_tokens_per_second: f64,
    pub total_tokens: usize,
    pub total_duration_ms: f64,
}

// ---------------------------------------------------------------------------
// Validator Trait — Chatty's third abstraction
// ---------------------------------------------------------------------------

/// Result of a single validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    /// Score from 0.0 to 1.0 (allows partial credit)
    pub score: f64,
    pub details: Vec<String>,
    pub validator: String,
}

impl ValidationResult {
    pub fn pass(validator: &str) -> Self {
        Self { passed: true, score: 1.0, details: vec![], validator: validator.to_string() }
    }
    pub fn fail(validator: &str, reason: &str) -> Self {
        Self { passed: false, score: 0.0, details: vec![reason.to_string()], validator: validator.to_string() }
    }
    pub fn partial(validator: &str, score: f64, details: Vec<String>) -> Self {
        Self { passed: score >= 0.5, score, details, validator: validator.to_string() }
    }
}

/// Validator trait — separate from BenchmarkRunner.
/// Different validation strategies are independent from task definitions.
pub trait Validator: Send + Sync {
    fn id(&self) -> &str;
    fn validate(&self, task: &BenchmarkTask, model_output: &str) -> Result<ValidationResult, String>;
}

/// ExactMatchValidator — output must match known_answer exactly (after trimming whitespace)
pub struct ExactMatchValidator;

impl Validator for ExactMatchValidator {
    fn id(&self) -> &str { "exact_match" }

    fn validate(&self, task: &BenchmarkTask, model_output: &str) -> Result<ValidationResult, String> {
        let answer = task.known_answer.as_deref().ok_or_else(|| format!(
            "Task {} has validation_type 'exact_match' but no known_answer", task.id
        ))?;

        let cleaned = model_output.trim();
        let expected = answer.trim();

        if cleaned == expected {
            Ok(ValidationResult::pass("exact_match"))
        } else if cleaned.contains(expected) {
            // Output contains the answer as a substring — partial credit
            Ok(ValidationResult::partial("exact_match", 0.75, vec![
                format!("Expected exact match '{expected}', found as substring")
            ]))
        } else {
            Ok(ValidationResult::fail("exact_match",
                &format!("Expected '{expected}', got '{cleaned}'")))
        }
    }
}

/// StructuralValidator — check required elements present, forbidden elements absent.
/// This replaces the old runner-level validate() method.
pub struct StructuralValidator;

impl Validator for StructuralValidator {
    fn id(&self) -> &str { "structural" }

    fn validate(&self, task: &BenchmarkTask, model_output: &str) -> Result<ValidationResult, String> {
        let mut missing = Vec::new();
        for required in &task.required_elements {
            if !model_output.contains(required) {
                missing.push(required.clone());
            }
        }
        let mut found_forbidden = Vec::new();
        for forbidden in &task.forbidden_elements {
            if model_output.contains(forbidden) {
                found_forbidden.push(forbidden.clone());
            }
        }

        let mut details: Vec<String> = Vec::new();
        if !missing.is_empty() {
            details.push(format!("Missing required elements: {:?}", missing));
        }
        if !found_forbidden.is_empty() {
            details.push(format!("Contains forbidden elements: {:?}", found_forbidden));
        }

        // Pass only if ALL requirements met — no partial pass for missing elements
        let passed = missing.is_empty() && found_forbidden.is_empty();

        if passed {
            Ok(ValidationResult::pass("structural"))
        } else if !missing.is_empty() {
            // Partial credit based on ratio of present required elements
            let ratio = if task.required_elements.is_empty() { 1.0 }
                else { (task.required_elements.len() - missing.len()) as f64 / task.required_elements.len() as f64 };
            // If forbidden found, reduce score further
            let score = if found_forbidden.is_empty() { ratio * 0.7 } else { ratio * 0.4 };
            Ok(ValidationResult::partial("structural", score, details))
        } else {
            // Only forbidden elements found, no missing required
            Ok(ValidationResult::partial("structural", 0.3, details))
        }
    }
}

/// NumericApproximateValidator — extract number from output, compare within tolerance
pub struct NumericApproximateValidator;

impl NumericApproximateValidator {
    fn extract_number(text: &str) -> Option<f64> {
        // Simple number extraction: find first numeric sequence with optional decimal
        let mut chars = text.chars().peekable();
        let mut num_str = String::new();
        let mut found = false;
        
        // Skip leading non-numeric characters
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() || c == '-' && num_str.is_empty() {
                found = true;
                break;
            }
            chars.next();
        }
        
        if !found { return None; }
        
        // Collect the number
        if let Some(&c) = chars.peek() {
            if c == '-' { num_str.push(chars.next().unwrap()); }
        }
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() || c == '.' {
                num_str.push(chars.next().unwrap());
            } else {
                break;
            }
        }
        
        num_str.parse::<f64>().ok()
    }
}

impl Validator for NumericApproximateValidator {
    fn id(&self) -> &str { "numeric_approximate" }

    fn validate(&self, task: &BenchmarkTask, model_output: &str) -> Result<ValidationResult, String> {
        let answer = task.known_answer.as_deref().ok_or_else(|| format!(
            "Task {} has validation_type 'numeric_approximate' but no known_answer", task.id
        ))?;
        let expected: f64 = answer.parse().map_err(|_| format!(
            "known_answer '{}' is not a valid number", answer
        ))?;

        let tolerance = task.tolerance.unwrap_or(0.01); // default 1%

        match Self::extract_number(model_output) {
            Some(got) => {
                let error = (got - expected).abs() / expected.abs().max(1.0);
                if error <= tolerance {
                    Ok(ValidationResult::pass("numeric_approximate"))
                } else {
                    let score = (1.0 - (error / tolerance).min(1.0)).max(0.0);
                    Ok(ValidationResult::partial("numeric_approximate", score, vec![
                        format!("Expected ~{expected}, got {got} (error: {:.2}%)", error * 100.0)
                    ]))
                }
            }
            None => Ok(ValidationResult::fail("numeric_approximate",
                &format!("Could not extract a number from output for task {} (expected ~{expected})", task.id)))
        }
    }
}

/// HumanReviewValidator — defers validation to human review (always returns unvalidated)
pub struct HumanReviewValidator;

impl Validator for HumanReviewValidator {
    fn id(&self) -> &str { "human_review" }

    fn validate(&self, task: &BenchmarkTask, model_output: &str) -> Result<ValidationResult, String> {
        Ok(ValidationResult::partial("human_review", 0.49, vec![
            format!("Task '{}' requires human review. Output ({} chars) saved for inspection.",
                    task.id, model_output.len())
        ]))
    }
}

/// Resolve the appropriate validator based on a task's validation_type
pub fn resolve_validator(validation_type: &str) -> Box<dyn Validator> {
    match validation_type {
        "exact_match" => Box::new(ExactMatchValidator),
        "numeric_approximate" => Box::new(NumericApproximateValidator),
        "human_review" => Box::new(HumanReviewValidator),
        _ => Box::new(StructuralValidator), // default: structural (backward compatible)
    }
}

// ---------------------------------------------------------------------------
// Capability Database — persist certification runs
// ---------------------------------------------------------------------------

const CAPABILITIES_DIR: &str = ".state/capabilities";

/// A single persisted certification entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityEntry {
    pub id: String,
    pub timestamp: String,
    pub benchmark_id: String,
    pub benchmark_name: String,
    pub model_id: String,
    pub model_params: f64,
    pub hardware: serde_json::Value,
    pub level_results: Vec<LevelResult>,
    pub total_gain: f64,
    pub total_duration_ms: f64,
    pub config: serde_json::Value,
    pub ath_version: String,
}

/// Database of certification results
pub struct CapabilityDatabase {
    base_path: PathBuf,
}

impl CapabilityDatabase {
    pub fn new() -> Self {
        Self { base_path: PathBuf::from(CAPABILITIES_DIR) }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self { base_path: path }
    }

    /// Save a certification report as a capability entry
    pub fn save(&self, report: &CertificationReport) -> anyhow::Result<String> {
        std::fs::create_dir_all(&self.base_path)?;

        let model_id = report.model["id"].as_str().unwrap_or("unknown");
        let model_params = report.model["parameters_b"].as_f64().unwrap_or(0.0);
        let ts = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let entry_id = format!("{}-{}-{}", report.benchmark_id, model_id, ts);

        let entry = CapabilityEntry {
            id: entry_id.clone(),
            timestamp: report.timestamp.clone(),
            benchmark_id: report.benchmark_id.clone(),
            benchmark_name: report.benchmark_name.clone(),
            model_id: model_id.to_string(),
            model_params,
            hardware: report.hardware.clone(),
            level_results: report.levels.clone(),
            total_gain: report.total_gain,
            total_duration_ms: report.total_duration_ms,
            config: report.config.clone(),
            ath_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let path = self.base_path.join(format!("{entry_id}.json"));
        let json = serde_json::to_string_pretty(&entry)?;
        std::fs::write(&path, json)?;

        Ok(entry_id)
    }

    /// List all persisted certification entries
    pub fn list(&self) -> anyhow::Result<Vec<CapabilityEntry>> {
        let mut entries = Vec::new();
        if !self.base_path.is_dir() {
            return Ok(entries);
        }
        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(ce) = serde_json::from_str::<CapabilityEntry>(&content) {
                        entries.push(ce);
                    }
                }
            }
        }
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)); // newest first
        Ok(entries)
    }

    /// Query: average capability gain for a given benchmark
    pub fn avg_gain(&self, benchmark_id: &str) -> Option<f64> {
        let entries = self.list().ok()?;
        let relevant: Vec<&CapabilityEntry> = entries.iter()
            .filter(|e| e.benchmark_id == benchmark_id)
            .collect();
        if relevant.is_empty() { return None; }
        let sum: f64 = relevant.iter().map(|e| e.total_gain).sum();
        Some(sum / relevant.len() as f64)
    }

    /// Query: best model for a given benchmark
    pub fn best_model(&self, benchmark_id: &str) -> Option<CapabilityEntry> {
        self.list().ok()?.into_iter()
            .filter(|e| e.benchmark_id == benchmark_id)
            .max_by(|a, b| {
                a.total_gain.partial_cmp(&b.total_gain).unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}

// ---------------------------------------------------------------------------
// Certification Report — capability metrics first
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationReport {
    pub benchmark_id: String,
    pub benchmark_name: String,
    pub tier: String,
    pub model: serde_json::Value,
    pub hardware: serde_json::Value,
    pub levels: Vec<LevelResult>,
    pub total_gain: f64,
    pub total_duration_ms: f64,
    pub timestamp: String,
    pub config: serde_json::Value,
    // New fields for experiment tracking
    pub experiment: Option<ExperimentMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentMetadata {
    pub independent_variable: String,
    pub dependent_variable: String,
    pub hypothesis: String,
    pub conclusion: Option<String>,
}

impl CertificationReport {
    pub fn to_pretty_string(&self) -> String {
        let mut s = String::new();
        s.push_str("╔══════════════════════════════════════════════════════════╗\n");
        s.push_str(&format!(
            "║  {:<56}║\n",
            format!("Athenas Certification — {}", self.benchmark_name)
        ));
        s.push_str("╚══════════════════════════════════════════════════════════╝\n");
        s.push('\n');

        // === CAPABILITY METRICS FIRST (per Chatty) ===
        s.push_str("📊 CAPABILITY REPORT — Per-Layer Contribution\n");
        s.push_str(&"═".repeat(85));
        s.push('\n');
        s.push_str(&format!(
            "  {:<6} {:<20} {:>10} {:>12} {:>10} {:>12}\n",
            "Level", "Configuration", "Score", "Pass Rate", "Tasks", "Delta"
        ));
        s.push_str(&"─".repeat(85));
        s.push('\n');

        let mut prev_score: Option<f64> = None;
        for level in &self.levels {
            let delta = match prev_score {
                Some(prev) if prev >= 0.0 => {
                    let change = (level.avg_score - prev) * 100.0;
                    format!("{:>+7.1}%", change.round())
                }
                _ => "    base".to_string(),
            };
            s.push_str(&format!(
                "  {:<6} {:<20} {:>6.1}%  {:>8.1}%  {:>4}/{:<4}  {}\n",
                level.level.short(),
                level.level_name,
                level.avg_score * 100.0,
                level.pass_rate * 100.0,
                level.passed,
                level.total,
                delta
            ));
            prev_score = Some(level.avg_score);
        }
        s.push_str(&"═".repeat(85));
        s.push('\n');
        s.push('\n');

        // Summary
        let first = self.levels.first();
        let last = self.levels.last();
        if let (Some(first), Some(last)) = (first, last) {
            let gain = (last.avg_score - first.avg_score) * 100.0;
            s.push_str(&format!("  📈 Capability gain: {:+.1} points (L0 → {})\n", gain, last.level.short()));
        }
        s.push('\n');

        // === MODEL INFO ===
        s.push_str(&format!("  🎯  Benchmark: {}\n", self.benchmark_name));
        s.push_str(&format!("  📋  Model:     {}\n", self.model_path()));
        s.push_str(&format!("  📏  Params:    {:.0}B\n", self.model_params()));
        s.push_str(&format!("  🏷️   Tier:      {}\n", self.tier));
        s.push('\n');

        // === PERFORMANCE METRICS SECONDARY ===
        s.push_str("⚡ Performance Metrics (secondary)\n");
        s.push_str(&"─".repeat(60));
        s.push('\n');
        for level in &self.levels {
            s.push_str(&format!(
                "  {}: TTFT {:.1}ms | {:.1} tok/s | {} tokens\n",
                level.level.short(), level.avg_ttft_ms, level.avg_tokens_per_second, level.total_tokens
            ));
        }
        s.push('\n');

        // Experiment metadata
        if let Some(exp) = &self.experiment {
            s.push_str("🧪 Experiment\n");
            s.push_str(&"─".repeat(60));
            s.push('\n');
            s.push_str(&format!("  Variable:  {} → {}\n", exp.independent_variable, exp.dependent_variable));
            s.push_str(&format!("  Hypothesis: {}\n", exp.hypothesis));
            if let Some(conclusion) = &exp.conclusion {
                s.push_str(&format!("  Conclusion: {}\n", conclusion));
            }
            s.push('\n');
        }

        s.push_str(&format!("  ⏱   Total duration: {:.0}s\n", self.total_duration_ms / 1000.0));
        s.push_str(&format!("  📁  Persisted to: .state/capabilities/\n"));
        s.push('\n');
        s.push_str("  ✓ Certification complete!\n");

        s
    }

    fn model_path(&self) -> String {
        self.model["path"].as_str()
            .map(|s| Path::new(s).file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_else(|| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn model_params(&self) -> f64 {
        self.model["parameters_b"].as_f64().unwrap_or(0.0)
    }
}

// ---------------------------------------------------------------------------
// BenchmarkRunner Trait
// ---------------------------------------------------------------------------

pub trait BenchmarkRunner: Send + Sync {
    fn id(&self) -> &str;
    fn metadata(&self) -> BenchmarkMetadata;
    fn discover_tasks(&self) -> Result<Vec<BenchmarkTask>, String>;

    fn prepare_environment(&self, task: &BenchmarkTask) -> Result<(), String> {
        let _ = task; Ok(())
    }

    /// Old validate method — kept for backward compatibility.
    /// New code should use the Validator trait instead.
    fn validate(&self, task: &BenchmarkTask, model_output: &str) -> Result<bool, String> {
        // Default implementation uses StructuralValidator
        let validator = StructuralValidator;
        let result = validator.validate(task, model_output)?;
        Ok(result.passed)
    }

    fn teardown(&self, task: &BenchmarkTask) -> Result<(), String> {
        let _ = task; Ok(())
    }
}

// ---------------------------------------------------------------------------
// Benchmark Registry
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct BenchmarkRegistry {
    runners: Vec<Box<dyn BenchmarkRunner>>,
}

impl BenchmarkRegistry {
    pub fn new() -> Self { Self { runners: Vec::new() } }

    pub fn register(&mut self, runner: Box<dyn BenchmarkRunner>) {
        self.runners.push(runner);
    }

    pub fn get(&self, id: &str) -> Option<&dyn BenchmarkRunner> {
        self.runners.iter().find(|r| r.id() == id).map(|r| r.as_ref())
    }

    pub fn list(&self) -> Vec<&str> {
        self.runners.iter().map(|r| r.id()).collect()
    }

    pub fn list_with_metadata(&self) -> Vec<BenchmarkMetadata> {
        self.runners.iter().map(|r| r.metadata()).collect()
    }

    pub fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// Default validate helper (legacy, kept for backward compat)
// ---------------------------------------------------------------------------

pub fn default_validate(task: &BenchmarkTask, model_output: &str) -> Result<bool, String> {
    let validator = StructuralValidator;
    let result = validator.validate(task, model_output)?;
    Ok(result.passed)
}

// ---------------------------------------------------------------------------
// Certification Engine
// ---------------------------------------------------------------------------

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
    run_benchmark_certify_inner(
        runner, level, pack_id, workspace_path, max_tokens,
        model_path, server_path, json_output, false,
    )
}

/// Internal version with mock support.
/// When mock=true, uses MockRuntime (no GPU, no model file needed).
pub fn run_benchmark_certify_mock(
    runner: &dyn BenchmarkRunner,
    level: u8,
    pack_id: Option<&str>,
    max_tokens: usize,
    json_output: bool,
) -> anyhow::Result<CertificationReport> {
    let fake_path = std::path::Path::new("mock-model.gguf");
    run_benchmark_certify_inner(
        runner, level, pack_id, None, max_tokens,
        fake_path, None, json_output, true,
    )
}

fn run_benchmark_certify_inner(
    runner: &dyn BenchmarkRunner,
    level: u8,
    pack_id: Option<&str>,
    workspace_path: Option<&Path>,
    max_tokens: usize,
    model_path: &Path,
    server_path: Option<&Path>,
    json_output: bool,
    mock: bool,
) -> anyhow::Result<CertificationReport> {
    use crate::runtime::{knowledge, Runtime};

    let metadata = runner.metadata();
    let max_level = level.min(metadata.max_level);

    let tasks = runner.discover_tasks()
        .map_err(|e| anyhow::anyhow!("Failed to discover tasks: {e}"))?;

    let (model, hw, info) = if mock {
        // Mock mode: no real model, use placeholder hardware
        let hw = crate::runtime::hardware::detect_hardware();
        let info = crate::runtime::ModelInfo {
            id: "mock-model".to_string(),
            path: model_path.to_string_lossy().to_string(),
            parameters_b: 7.0,
            quantization: "Q4_K_M".to_string(),
            architecture: "Mock".to_string(),
            context_length: 32768,
            hardware: hw.clone(),
        };
        (model_path.to_path_buf(), hw, info)
    } else {
        let model = crate::runtime::find_model(Some(model_path))?;
        let hw = crate::runtime::hardware::detect_hardware();
        let info = crate::runtime::infer_model_info(&model, Some(&hw));
        (model, hw, info)
    };

    // Build level prompts
    let l1_system = pack_id.and_then(|pid| {
        let dir = knowledge::default_packs_dir();
        let packs = knowledge::discover_packs(&dir);
        knowledge::find_pack(&packs, pid).map(|p| knowledge::build_system_prompt(p, None))
    });

    let l2_system = workspace_path.and_then(|w| {
        let config_path = w.join("athenas.json");
        std::fs::read_to_string(config_path).ok().and_then(|content| {
            serde_json::from_str::<serde_json::Value>(&content).ok()
                .and_then(|v| v["system_prompt"].as_str().map(|s| s.to_string()))
        })
    });

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

    let build_level_prompt = |task_prompt: &str, lvl: BenchmarkLevel| -> String {
        match lvl {
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
                let base = l2_system.as_deref().or(l1_system.as_deref()).unwrap_or("");
                format!("{base}\n{l3_tools}\n\n### Task\n\n{task_prompt}")
            }
            _ => format!("### Task\n\n{task_prompt}"),
        }
    };

    let levels_to_run: Vec<BenchmarkLevel> = BenchmarkLevel::all().iter().copied()
        .take((max_level + 1) as usize).collect();

    // Build and start runtime (mock or real)
    let cert_start = std::time::Instant::now();

    if !json_output {
        println!("╔══════════════════════════════════════════╗");
        println!("║  Athenas Benchmark Engine v0.1.0        ║");
        println!("╚══════════════════════════════════════════╝");
        println!();
        println!("🎯 Benchmark: {}", metadata.name);
        println!("📋 Model:     {}", info.path);
        if mock { println!("📋 Mode:      Mock (deterministic, no GPU)"); }
        println!("📏 Tasks:     {} (running L0..L{})", tasks.len(), max_level);
        println!();
    }

    let mut rt: Box<dyn Runtime> = if mock {
        Box::new(crate::runtime::MockRuntime::new()
            .with_latency(5.0, 100.0)
            .with_default_response("Mock response: certification test completed."))
    } else {
        let mut builder = crate::runtime::LlamaServerRuntime::new();
        if let Some(sp) = server_path {
            builder = builder.with_server_path(sp.to_path_buf());
        }
        Box::new(builder)
    };

    rt.load_model(&model)?;

    if !json_output { println!("  ✅ Model loaded\n"); }

    let mut level_results: Vec<LevelResult> = Vec::new();

    for (idx, level) in levels_to_run.iter().enumerate() {
        if !json_output { println!("🧪 {}: {}", level.short(), level.name()); }

        let level_start = std::time::Instant::now();
        let mut task_results: Vec<BenchmarkTaskResult> = Vec::new();

        for task in &tasks {
            let prompt = build_level_prompt(&task.prompt, *level);

            let result = crate::runtime::run_benchmark(rt.as_ref(), &prompt, max_tokens);

            let task_result = match result {
                Ok(inference) => {
                    // Use the resolved validator
                    let validator = resolve_validator(&task.validation_type);
                    let v_result = validator.validate(task, &inference.text).unwrap_or_else(|e| {
                        ValidationResult::fail("error", &format!("Validation error: {e}"))
                    });

                    BenchmarkTaskResult {
                        task_id: task.id.clone(),
                        level: *level,
                        passed: v_result.passed,
                        score: v_result.score,
                        model_output: inference.text.clone(),
                        match_details: v_result.details,
                        ttft_ms: inference.ttft_ms,
                        tokens_per_second: inference.tokens_per_second,
                        total_tokens: inference.total_tokens,
                    }
                }
                Err(e) => BenchmarkTaskResult {
                    task_id: task.id.clone(),
                    level: *level,
                    passed: false,
                    score: 0.0,
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

        // Aggregate with scores
        let passed_count = task_results.iter().filter(|r| r.passed).count();
        let total_count = task_results.len();
        let pass_rate = if total_count > 0 { passed_count as f64 / total_count as f64 } else { 0.0 };
        let avg_score = if total_count > 0 {
            task_results.iter().map(|r| r.score).sum::<f64>() / total_count as f64
        } else { 0.0 };
        let avg_ttft = if total_count > 0 {
            task_results.iter().map(|r| r.ttft_ms).sum::<f64>() / total_count as f64
        } else { 0.0 };
        let avg_tps = if total_count > 0 {
            task_results.iter().map(|r| r.tokens_per_second).sum::<f64>() / total_count as f64
        } else { 0.0 };
        let total_tok: usize = task_results.iter().map(|r| r.total_tokens).sum();

        level_results.push(LevelResult {
            level: *level,
            level_name: level.name().to_string(),
            pass_rate,
            avg_score,
            passed: passed_count,
            total: total_count,
            avg_ttft_ms: avg_ttft,
            avg_tokens_per_second: avg_tps,
            total_tokens: total_tok,
            total_duration_ms: level_duration,
        });

        if !json_output {
            println!("     Score: {:.1}% | Pass rate: {:.1}% ({}/{})", avg_score * 100.0, pass_rate * 100.0, passed_count, total_count);
            println!("     Avg TTFT: {:.1}ms | TPS: {:.1}", avg_ttft, avg_tps);
            println!();
        }

        if idx + 1 < levels_to_run.len() {
            rt.unload()?;
            rt.load_model(&model)?;
        }
    }

    rt.unload()?;

    let total_duration = cert_start.elapsed().as_secs_f64() * 1000.0;

    let first_score = level_results.first().map(|l| l.avg_score).unwrap_or(0.0);
    let last_score = level_results.last().map(|l| l.avg_score).unwrap_or(0.0);
    let total_gain = (last_score - first_score) * 100.0;

    let experiment = Some(ExperimentMetadata {
        independent_variable: match max_level {
            0 => "None".to_string(),
            1 => "Knowledge Pack".to_string(),
            2 => "Workspace".to_string(),
            3 => "Tools".to_string(),
            _ => format!("L{max_level}"),
        },
        dependent_variable: format!("{} Score", metadata.name),
        hypothesis: format!("Adding {} improves model capability on {}", 
            match max_level {
                0 => "no augmentation".to_string(),
                1 => "a knowledge pack".to_string(),
                2 => "a workspace".to_string(),
                3 => "tools".to_string(),
                _ => format!("L{max_level}"),
            },
            metadata.name),            conclusion: Some(if total_gain.abs() < 0.1 {
                format!("No measurable change ({total_gain:+.1} points)")
            } else if total_gain > 0.0 {
                format!("Positive gain: {total_gain:+.1} points — layer added measurable capability")
            } else {
                format!("Negative gain: {total_gain:+.1} points — layer may not help this model/benchmark combination")
            }),
    });

    let report = CertificationReport {
        benchmark_id: metadata.id.clone(),
        benchmark_name: metadata.name.clone(),
        tier: metadata.tier.clone(),
        model: serde_json::json!({
            "id": info.id, "path": info.path, "parameters_b": info.parameters_b,
            "quantization": info.quantization, "architecture": info.architecture,
            "context_length": info.context_length,
        }),
        hardware: serde_json::to_value(&hw).unwrap_or(serde_json::Value::Null),
        levels: level_results,
        total_gain: (total_gain * 10.0).round() / 10.0,
        total_duration_ms: total_duration,
        timestamp: chrono::Utc::now().to_rfc3339(),
        config: serde_json::json!({
            "level": level, "max_tokens": max_tokens, "pack_id": pack_id,
            "workspace": workspace_path.map(|p| p.to_string_lossy().to_string()),
            "server_path": server_path.map(|p| p.to_string_lossy().to_string()),
        }),
        experiment,
    };

    // Persist to capability database
    let db = CapabilityDatabase::new();
    match db.save(&report) {
        Ok(entry_id) => {
            if !json_output {
                println!("  💾 Certification persisted: .state/capabilities/{entry_id}.json");
            }
        }
        Err(e) => {
            if !json_output {
                eprintln!("  ⚠ Failed to persist certification: {e}");
            }
        }
    }

    if json_output {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("{}", report.to_pretty_string());
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(validation_type: &str, known_answer: Option<&str>, tolerance: Option<f64>) -> BenchmarkTask {
        BenchmarkTask {
            id: "TEST-001".into(), description: "Test task".into(), prompt: "Test prompt".into(),
            required_elements: vec![], forbidden_elements: vec![],
            validation_type: validation_type.into(),
            known_answer: known_answer.map(|s| s.into()),
            tolerance,
            reference: None, language: None,
        }
    }

    // --- Validator Tests ---

    #[test]
    fn test_exact_match_validator_passes() {
        let task = make_task("exact_match", Some("42"), None);
        let validator = ExactMatchValidator;
        let result = validator.validate(&task, "42").unwrap();
        assert!(result.passed);
        assert!((result.score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_exact_match_validator_trims_whitespace() {
        let task = make_task("exact_match", Some("hello"), None);
        let validator = ExactMatchValidator;
        let result = validator.validate(&task, "  hello  ").unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_exact_match_validator_fails() {
        let task = make_task("exact_match", Some("42"), None);
        let validator = ExactMatchValidator;
        let result = validator.validate(&task, "43").unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn test_exact_match_partial_credit() {
        let task = make_task("exact_match", Some("hello world"), None);
        let validator = ExactMatchValidator;
        let result = validator.validate(&task, "the answer is hello world").unwrap();
        // Should get partial credit (0.75) since answer is a substring
        assert!((result.score - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_structural_validator_passes() {
        let task = BenchmarkTask {
            id: "TEST-002".into(), description: "Test".into(), prompt: "Test".into(),
            required_elements: vec!["fn hello".into(), "return".into()],
            forbidden_elements: vec![],
            validation_type: "structural".into(),
            known_answer: None, tolerance: None, reference: None, language: None,
        };
        let validator = StructuralValidator;
        let result = validator.validate(&task, "fn hello() { return 42; }").unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_structural_validator_fails_missing() {
        let task = BenchmarkTask {
            id: "TEST-003".into(), description: "Test".into(), prompt: "Test".into(),
            required_elements: vec!["fn hello".into(), "fn goodbye".into()],
            forbidden_elements: vec![],
            validation_type: "structural".into(),
            known_answer: None, tolerance: None, reference: None, language: None,
        };
        let validator = StructuralValidator;
        let result = validator.validate(&task, "fn hello() { }").unwrap();
        assert!(!result.passed);
        assert!(result.score < 0.5);
    }

    #[test]
    fn test_numeric_approximate_validator() {
        let task = make_task("numeric_approximate", Some("100"), Some(0.1));
        let validator = NumericApproximateValidator;
        // Within 10% tolerance
        let result = validator.validate(&task, "Answer: 105").unwrap();
        assert!(result.passed);
        // Outside tolerance
        let result = validator.validate(&task, "Answer: 150").unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn test_numeric_extraction_from_text() {
        let task = make_task("numeric_approximate", Some("42.5"), Some(0.01));
        let validator = NumericApproximateValidator;
        let result = validator.validate(&task, "The result is 42.5 tokens").unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_human_review_validator() {
        let task = make_task("human_review", None, None);
        let validator = HumanReviewValidator;
        let result = validator.validate(&task, "any output").unwrap();
        assert!(!result.passed); // human review always needs approval
        assert!((result.score - 0.49).abs() < 0.01); // always 0.49 (below pass threshold)
    }

    #[test]
    fn test_resolve_validator_defaults_to_structural() {
        let v = resolve_validator("unknown_type");
        assert_eq!(v.id(), "structural");
    }

    // --- Capability Database Tests ---

    #[test]
    fn test_capability_db_save_and_list() {
        use std::env;
        let tmp = env::temp_dir().join(format!("ath-test-capdb-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);

        let db = CapabilityDatabase::with_path(tmp.clone());

        let report = CertificationReport {
            benchmark_id: "test-bench".into(),
            benchmark_name: "Test Benchmark".into(),
            tier: "test".into(),
            model: serde_json::json!({"id": "test-model", "path": "/tmp/test.gguf", "parameters_b": 7.0}),
            hardware: serde_json::Value::Null,
            levels: vec![],
            total_gain: 15.0,
            total_duration_ms: 1000.0,
            timestamp: "2026-01-01T00:00:00Z".into(),
            config: serde_json::json!({}),
            experiment: None,
        };

        let id = db.save(&report).unwrap();
        assert!(id.contains("test-bench"));

        let entries = db.list().unwrap();
        assert!(!entries.is_empty());

        let avg = db.avg_gain("test-bench");
        assert!((avg.unwrap() - 15.0).abs() < 0.01);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // --- BenchmarkTask known_answer tests ---

    #[test]
    fn test_task_with_known_answer() {
        let task = make_task("exact_match", Some("hello world"), None);
        assert_eq!(task.known_answer.as_deref(), Some("hello world"));
    }

    #[test]
    fn test_task_with_tolerance() {
        let task = make_task("numeric_approximate", Some("100"), Some(0.05));
        assert_eq!(task.tolerance, Some(0.05));
    }

    #[test]
    fn test_task_default_known_answer_none() {
        let task = make_task("structural", None, None);
        assert!(task.known_answer.is_none());
    }
}
