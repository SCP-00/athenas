use serde::Serialize;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Signal — a single measurable fact about an execution
// ---------------------------------------------------------------------------

/// Kinds of signals that can be measured
#[derive(Debug, Clone, Serialize)]
pub enum SignalValue {
    Boolean(bool),
    Numeric(f64),
    String(String),
    Structured(serde_json::Value),
}

/// A single signal definition
#[derive(Debug, Clone, Serialize)]
pub struct Signal {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Which domains this signal applies to (empty = universal)
    pub domains: Vec<String>,
}

/// Result of evaluating a single signal
#[derive(Debug, Clone, Serialize)]
pub struct SignalResult {
    pub signal_id: String,
    pub value: SignalValue,
    /// How reliable is this measurement (0.0 - 1.0)
    pub confidence: f64,
    /// Raw output from the tool (for debugging)
    pub raw_output: Option<String>,
    /// Tool exit code (if applicable)
    pub exit_code: Option<i32>,
    /// How long the measurement took (ms)
    pub duration_ms: f64,
}

// ---------------------------------------------------------------------------
// SignalProvider — plugin that produces signals from tool execution
// ---------------------------------------------------------------------------

/// Context passed to a SignalProvider for evaluation
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Working directory of the project
    pub working_dir: PathBuf,
    /// Any files that were created or modified
    pub files_changed: Vec<String>,
    /// Any commands that were executed
    pub commands_executed: Vec<String>,
    /// The inference result text
    pub model_output: Option<String>,
}

/// SignalProvider trait — produces signals by executing tools and parsing output.
/// Follows the same plugin pattern as KnowledgeProvider, BenchmarkRunner, etc.
pub trait SignalProvider: Send + Sync {
    /// Unique identifier for this provider
    fn id(&self) -> &str;

    /// Which signals this provider can produce
    fn produces(&self) -> Vec<Signal>;

    /// Evaluate all applicable signals for the given context.
    /// Returns a SignalResult for each signal this provider can measure.
    fn evaluate(&self, ctx: &EvaluationContext) -> Vec<SignalResult>;
}

// ---------------------------------------------------------------------------
// Evidence Source Hierarchy
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum EvidenceSource {
    Academic,
    Synthetic,
    Execution,
    RealProject,
    Repeated,
    Production,
    HumanValidation,
}

impl EvidenceSource {
    /// Strength derived from the hierarchy position
    pub fn strength(&self) -> f64 {
        match self {
            EvidenceSource::Academic => 0.1,
            EvidenceSource::Synthetic => 0.2,
            EvidenceSource::Execution => 0.3,
            EvidenceSource::RealProject => 0.5,
            EvidenceSource::Repeated => 0.7,
            EvidenceSource::Production => 0.85,
            EvidenceSource::HumanValidation => 1.0,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            EvidenceSource::Academic => "academic",
            EvidenceSource::Synthetic => "synthetic",
            EvidenceSource::Execution => "execution",
            EvidenceSource::RealProject => "real-project",
            EvidenceSource::Repeated => "repeated",
            EvidenceSource::Production => "production",
            EvidenceSource::HumanValidation => "human-validation",
        }
    }
}

// ---------------------------------------------------------------------------
// Evidence — a collection of signals from a single execution
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct Evidence {
    pub source: EvidenceSource,
    pub strength: f64,
    pub confidence: f64,
    pub timestamp: String,
    pub signals: Vec<SignalResult>,
    pub domain: String,
    pub task_type: String,
}

// ---------------------------------------------------------------------------
// Reliability Metrics
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct ReliabilityMetrics {
    pub success_rate: f64,
    pub variance: f64,
    pub timeout_rate: f64,
    pub crash_rate: f64,
    pub tool_failure_rate: f64,
    pub regression_rate: f64,
    pub total_executions: usize,
}

impl ReliabilityMetrics {
    pub fn new() -> Self {
        Self {
            success_rate: 0.0,
            variance: 0.0,
            timeout_rate: 0.0,
            crash_rate: 0.0,
            tool_failure_rate: 0.0,
            regression_rate: 0.0,
            total_executions: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Coverage Metrics
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct CoverageMetrics {
    pub domains_tested: Vec<String>,
    pub domains_total: usize,
    pub coverage_pct: f64,
    pub signals_per_domain: Vec<(String, usize)>,
    pub tasks_per_domain: Vec<(String, usize)>,
}

impl CoverageMetrics {
    pub fn new() -> Self {
        Self {
            domains_tested: Vec::new(),
            domains_total: 0,
            coverage_pct: 0.0,
            signals_per_domain: Vec::new(),
            tasks_per_domain: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// EngineeringReport — the output of evaluation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct EngineeringReport {
    /// Did the task succeed? (0.0 - 1.0)
    pub outcome_score: f64,
    /// How well was it solved? (0.0 - 1.0)
    pub quality_score: f64,
    /// Overall capability from benchmarks (0.0 - 1.0)
    pub capability_score: f64,
    /// Confidence in this evaluation (0.0 - 1.0)
    pub confidence: f64,
    /// Reliability over multiple executions
    pub reliability: ReliabilityMetrics,
    /// Coverage across domains
    pub coverage: CoverageMetrics,
    /// All evidence collected
    pub evidence: Vec<Evidence>,
    /// Timestamp
    pub timestamp: String,
    /// Duration of evaluation (ms)
    pub duration_ms: f64,
}

impl EngineeringReport {
    pub fn new() -> Self {
        Self {
            outcome_score: 0.0,
            quality_score: 0.0,
            capability_score: 0.0,
            confidence: 0.0,
            reliability: ReliabilityMetrics::new(),
            coverage: CoverageMetrics::new(),
            evidence: Vec::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms: 0.0,
        }
    }

    /// Compute the aggregate engineering score (outcome + quality weighted average)
    pub fn engineering_score(&self) -> f64 {
        // Outcome is weighted more heavily than quality
        self.outcome_score * 0.6 + self.quality_score * 0.4
    }

    /// Display the report in human-readable format
    pub fn display(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "╔══════════════════════════════════════════╗\n\
             ║     Engineering Evaluation Report         ║\n\
             ╚══════════════════════════════════════════╝\n\n"
        ));
        s.push_str(&format!("  📊 Engineering Score: {:.1}%\n", self.engineering_score() * 100.0));
        s.push_str(&format!("  🎯 Outcome:          {:.1}%\n", self.outcome_score * 100.0));
        s.push_str(&format!("  ✨ Quality:          {:.1}%\n", self.quality_score * 100.0));
        s.push_str(&format!("  🏆 Capability:       {:.1}%\n", self.capability_score * 100.0));
        s.push_str(&format!("  🔒 Confidence:       {:.1}%\n", self.confidence * 100.0));
        s.push('\n');

        s.push_str(&format!("  📈 Reliability:\n"));
        s.push_str(&format!("     Success rate: {:.1}%\n", self.reliability.success_rate * 100.0));
        s.push_str(&format!("     Executions:   {}\n", self.reliability.total_executions));

        if !self.coverage.domains_tested.is_empty() {
            s.push_str(&format!("\n  🌐 Coverage: {:.0}%\n", self.coverage.coverage_pct * 100.0));
            for (domain, count) in &self.coverage.tasks_per_domain {
                s.push_str(&format!("     {}: {} tasks\n", domain, count));
            }
        }

        if !self.evidence.is_empty() {
            s.push_str(&format!("\n  📋 Evidence ({} sources)\n", self.evidence.len()));
            for ev in &self.evidence {
                s.push_str(&format!(
                    "     • {} ({}, strength {:.1}%)\n",
                    ev.source.name(),
                    ev.domain,
                    ev.strength * 100.0
                ));
            }
        }

        s.push('\n');
        s.push_str("  ✓ Evaluation complete.\n");
        s
    }
}

// ---------------------------------------------------------------------------
// EngineeringEvaluator trait — aggregates signals into an EngineeringReport
// ---------------------------------------------------------------------------

pub trait EngineeringEvaluator: Send + Sync {
    fn id(&self) -> &str;

    /// Register a signal provider
    fn register(&mut self, provider: Box<dyn SignalProvider>);

    /// Evaluate a task execution and produce an EngineeringReport
    fn evaluate(&self, ctx: &EvaluationContext) -> EngineeringReport;
}

// ---------------------------------------------------------------------------
// DefaultEngineeringEvaluator — concrete implementation
// ---------------------------------------------------------------------------

pub struct DefaultEngineeringEvaluator {
    providers: Vec<Box<dyn SignalProvider>>,
}

impl DefaultEngineeringEvaluator {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }
}

impl EngineeringEvaluator for DefaultEngineeringEvaluator {
    fn id(&self) -> &str {
        "default"
    }

    fn register(&mut self, provider: Box<dyn SignalProvider>) {
        self.providers.push(provider);
    }

    fn evaluate(&self, ctx: &EvaluationContext) -> EngineeringReport {
        let start = std::time::Instant::now();
        let mut all_signals: Vec<SignalResult> = Vec::new();

        for provider in &self.providers {
            let results = provider.evaluate(ctx);
            all_signals.extend(results);
        }

        // Compute outcome score from boolean signals
        let bool_results: Vec<&SignalResult> = all_signals.iter()
            .filter(|s| matches!(s.value, SignalValue::Boolean(_)))
            .collect();

        let outcome_score = if bool_results.is_empty() {
            0.0
        } else {
            let passed = bool_results.iter()
                .filter(|s| matches!(s.value, SignalValue::Boolean(true)))
                .count();
            passed as f64 / bool_results.len() as f64
        };

        // Compute quality score from numeric signals
        let numeric_signals: Vec<&SignalResult> = all_signals.iter()
            .filter(|s| matches!(s.value, SignalValue::Numeric(_)))
            .collect();

        let quality_score = if numeric_signals.is_empty() {
            // Without numeric signals, quality defaults to outcome
            outcome_score
        } else {
            let avg: f64 = numeric_signals.iter()
                .filter_map(|s| {
                    if let SignalValue::Numeric(v) = s.value {
                        Some(v * s.confidence)
                    } else {
                        None
                    }
                })
                .sum();
            let weight_sum: f64 = numeric_signals.iter()
                .map(|s| s.confidence)
                .sum();

            if weight_sum > 0.0 {
                (avg / weight_sum).min(1.0).max(0.0)
            } else {
                outcome_score
            }
        };

        // Compute aggregate confidence
        let confidence = if all_signals.is_empty() {
            0.0
        } else {
            let avg_confidence: f64 = all_signals.iter()
                .map(|s| s.confidence)
                .sum::<f64>() / all_signals.len() as f64;
            // Penalize for few signals
            let signal_count_factor = (all_signals.len() as f64 / 5.0).min(1.0);
            avg_confidence * signal_count_factor
        };

        // Collect evidence
        let evidence = vec![Evidence {
            source: EvidenceSource::Execution,
            strength: EvidenceSource::Execution.strength(),
            confidence,
            timestamp: chrono::Utc::now().to_rfc3339(),
            signals: all_signals,
            domain: "general".to_string(),
            task_type: "engineering".to_string(),
        }];

        EngineeringReport {
            outcome_score,
            quality_score,
            capability_score: outcome_score, // Will be overridden when capability data exists
            confidence,
            reliability: ReliabilityMetrics::new(),
            coverage: CoverageMetrics::new(),
            evidence,
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms: start.elapsed().as_secs_f64() * 1000.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Built-in SignalProviders
// ---------------------------------------------------------------------------

/// ExitCodeProvider — evaluates commands by their exit codes
pub struct ExitCodeSignalProvider;

impl SignalProvider for ExitCodeSignalProvider {
    fn id(&self) -> &str {
        "exit-code"
    }

    fn produces(&self) -> Vec<Signal> {
        vec![
            Signal {
                id: "exit-code:success".to_string(),
                name: "ExitCodeSuccess".to_string(),
                description: "Command exited with code 0".to_string(),
                domains: vec![],
            },
        ]
    }

    fn evaluate(&self, ctx: &EvaluationContext) -> Vec<SignalResult> {
        // Success if commands were executed (the task ran without crashing)
        let success = !ctx.commands_executed.is_empty();
        vec![SignalResult {
            signal_id: "exit-code:success".to_string(),
            value: SignalValue::Boolean(success),
            confidence: 0.95,
            raw_output: None,
            exit_code: if success { Some(0) } else { None },
            duration_ms: 0.0,
        }]
    }
}

/// FileChangeSignalProvider — evaluates the quality of file changes
pub struct FileChangeSignalProvider;

impl SignalProvider for FileChangeSignalProvider {
    fn id(&self) -> &str {
        "file-changes"
    }

    fn produces(&self) -> Vec<Signal> {
        vec![
            Signal {
                id: "file-changes:count".to_string(),
                name: "FilesModified".to_string(),
                description: "Number of files modified".to_string(),
                domains: vec![],
            },
        ]
    }

    fn evaluate(&self, ctx: &EvaluationContext) -> Vec<SignalResult> {
        let count = ctx.files_changed.len() as f64;
        // Fewer files is generally better (higher quality score)
        let quality = if count == 0.0 {
            0.0
        } else if count == 1.0 {
            1.0
        } else if count <= 5.0 {
            0.7
        } else {
            0.3
        };

        vec![SignalResult {
            signal_id: "file-changes:count".to_string(),
            value: SignalValue::Numeric(quality),
            confidence: 0.8,
            raw_output: Some(format!("{} files modified", ctx.files_changed.len())),
            exit_code: None,
            duration_ms: 0.0,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_provider_trait() {
        let provider = ExitCodeSignalProvider;
        assert_eq!(provider.id(), "exit-code");
        assert!(!provider.produces().is_empty());
    }

    #[test]
    fn test_engineering_evaluator_empty() {
        let evaluator = DefaultEngineeringEvaluator::new();
        let ctx = EvaluationContext {
            working_dir: PathBuf::from("."),
            files_changed: vec![],
            commands_executed: vec![],
            model_output: None,
        };
        let report = evaluator.evaluate(&ctx);
        // With no providers, outcome is 0.0
        assert_eq!(report.outcome_score, 0.0);
        assert_eq!(report.confidence, 0.0);
    }

    #[test]
    fn test_engineering_evaluator_with_providers() {
        let mut evaluator = DefaultEngineeringEvaluator::new();
        evaluator.register(Box::new(ExitCodeSignalProvider));
        evaluator.register(Box::new(FileChangeSignalProvider));

        let ctx = EvaluationContext {
            working_dir: PathBuf::from("."),
            files_changed: vec!["src/main.rs".to_string()],
            commands_executed: vec!["cargo build".to_string()],
            model_output: Some("fixed the bug".to_string()),
        };
        let report = evaluator.evaluate(&ctx);
        // ExitCodeSignalProvider gives Boolean(true)
        assert_eq!(report.outcome_score, 1.0);
        // FileChangeSignalProvider gives quality 1.0 for 1 file
        assert_eq!(report.quality_score, 1.0);
        // Confidence should be > 0
        assert!(report.confidence > 0.0);
        // Evidence should be present
        assert!(!report.evidence.is_empty());
    }

    #[test]
    fn test_engineering_score() {
        let report = EngineeringReport {
            outcome_score: 1.0,
            quality_score: 0.5,
            capability_score: 0.8,
            confidence: 0.9,
            reliability: ReliabilityMetrics::new(),
            coverage: CoverageMetrics::new(),
            evidence: vec![],
            timestamp: "now".to_string(),
            duration_ms: 0.0,
        };
        // 0.6 * 1.0 + 0.4 * 0.5 = 0.6 + 0.2 = 0.8
        assert!((report.engineering_score() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_evidence_source_strength() {
        assert!(EvidenceSource::Academic.strength() < EvidenceSource::RealProject.strength());
        assert!(EvidenceSource::RealProject.strength() < EvidenceSource::Production.strength());
        assert!(EvidenceSource::Production.strength() < EvidenceSource::HumanValidation.strength());
    }

    #[test]
    fn test_file_change_quality() {
        let provider = FileChangeSignalProvider;
        let ctx = EvaluationContext {
            working_dir: PathBuf::from("."),
            files_changed: vec!["src/main.rs".to_string(), "src/lib.rs".to_string()],
            commands_executed: vec![],
            model_output: None,
        };
        let results = provider.evaluate(&ctx);
        assert_eq!(results.len(), 1);
        // 2 files → quality 0.7
        if let SignalValue::Numeric(v) = results[0].value {
            assert!((v - 0.7).abs() < 0.01);
        } else {
            panic!("Expected Numeric value");
        }
    }
}
