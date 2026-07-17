use serde::Serialize;
use std::path::Path;

use super::benchmark::ResearchDatabase;
use super::graph::eng_node::RuntimeStatus;
use super::graph::{EngineeringGraph, load_graph};
use super::hardware;
use super::knowledge;
use super::model_intelligence::vram::{KvCacheType, MemoryConfig, VramCalculator};

// ---------------------------------------------------------------------------
// Objective — what the user wants to optimize for
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum Objective {
    MaximumCapability,
    MaximumThroughput,
    MinimumLatency,
    MinimumVram,
    OfflineOnly,
    Coding,
    Research,
    Default,
}

impl Objective {
    pub fn all() -> &'static [Objective] {
        &[
            Objective::MaximumCapability,
            Objective::MaximumThroughput,
            Objective::MinimumLatency,
            Objective::MinimumVram,
            Objective::OfflineOnly,
            Objective::Coding,
            Objective::Research,
            Objective::Default,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Objective::MaximumCapability => "Maximum Capability",
            Objective::MaximumThroughput => "Maximum Throughput",
            Objective::MinimumLatency => "Minimum Latency",
            Objective::MinimumVram => "Minimum VRAM",
            Objective::OfflineOnly => "Offline Only",
            Objective::Coding => "Coding",
            Objective::Research => "Research",
            Objective::Default => "Default (balanced)",
        }
    }

    pub fn from_name(s: &str) -> Option<Objective> {
        Objective::all().iter().find(|o| o.name().eq_ignore_ascii_case(s)
            || format!("{:?}", o).eq_ignore_ascii_case(s)).copied()
    }
}

// ---------------------------------------------------------------------------
// TaskDescriptor — describes what the user wants to accomplish
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TaskDescriptor {
    /// Human-readable task name (e.g., "Rust Development", "Web Pentest")
    pub name: String,
    /// Required capabilities (e.g., "coding", "tool-calling", "reasoning")
    pub capabilities: Vec<String>,
    /// Relevant programming languages
    pub languages: Vec<String>,
    /// Optimization objective
    pub objective: Objective,
}

impl TaskDescriptor {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            capabilities: Vec::new(),
            languages: Vec::new(),
            objective: Objective::Default,
        }
    }

    pub fn with_capabilities(mut self, caps: Vec<&str>) -> Self {
        self.capabilities = caps.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_languages(mut self, langs: Vec<&str>) -> Self {
        self.languages = langs.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_objective(mut self, obj: Objective) -> Self {
        self.objective = obj;
        self
    }

    /// Built-in task profiles
    pub fn rust_development() -> Self {
        Self::new("Rust Development")
            .with_capabilities(vec!["coding", "reasoning", "tool-calling"])
            .with_languages(vec!["Rust"])
            .with_objective(Objective::Coding)
    }

    pub fn web_pentest() -> Self {
        Self::new("Web Pentest")
            .with_capabilities(vec!["tool-calling", "reasoning", "coding"])
            .with_languages(vec!["Python", "JavaScript", "Bash"])
            .with_objective(Objective::MaximumCapability)
    }

    pub fn python_debugging() -> Self {
        Self::new("Python Debugging")
            .with_capabilities(vec!["coding", "reasoning", "instruction-following"])
            .with_languages(vec!["Python"])
            .with_objective(Objective::Coding)
    }

    pub fn reverse_engineering() -> Self {
        Self::new("Reverse Engineering")
            .with_capabilities(vec!["reasoning", "tool-calling"])
            .with_languages(vec!["C", "C++", "Assembly"])
            .with_objective(Objective::MaximumCapability)
    }

    pub fn all_profiles() -> Vec<TaskDescriptor> {
        vec![
            Self::rust_development(),
            Self::web_pentest(),
            Self::python_debugging(),
            Self::reverse_engineering(),
            Self::new("Research Assistant")
                .with_capabilities(vec!["reasoning", "text-generation", "instruction-following"])
                .with_objective(Objective::Research),
            Self::new("DevOps Automation")
                .with_capabilities(vec!["tool-calling", "coding"])
                .with_languages(vec!["Bash", "Python", "Docker"])
                .with_objective(Objective::MaximumThroughput),
        ]
    }
}

// ---------------------------------------------------------------------------
// CertifiedConfiguration — the scheduler's output with evidence
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct CertifiedConfiguration {
    pub task: String,
    pub objective: String,
    pub runtime: String,
    pub model: String,
    pub model_params_b: f64,
    pub model_quant: String,
    pub tools: Vec<String>,
    pub knowledge_packs: Vec<String>,
    pub capability_score: f64,
    pub confidence: f64,
    pub expected_ttft_ms: f64,
    pub expected_tokens_per_second: f64,
    pub evidence: Vec<String>,
}

impl CertifiedConfiguration {
    pub fn display(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "╔══════════════════════════════════════════╗\n\
             ║     Capacidad Scheduler                   ║\n\
             ╚══════════════════════════════════════════╝\n\n"
        ));

        s.push_str(&format!("🎯  Task:       {}\n", self.task));
        s.push_str(&format!("⚙️   Objective:  {}\n", self.objective));
        s.push_str("\n┌── Recommended Configuration ──────────────┐\n");

        s.push_str(&format!(
            "│ Runtime:    {}                            │\n",
            self.runtime
        ));
        s.push_str(&format!(
            "│ Model:      {} ({:.0}B, {})  │\n",
            self.model, self.model_params_b, self.model_quant
        ));
        s.push_str(&format!(
            "│ Capability: {:.1}%                         │\n",
            self.capability_score * 100.0
        ));
        s.push_str(&format!(
            "│ Confidence: {:.1}%                         │\n",
            self.confidence * 100.0
        ));
        s.push_str(&format!(
            "│ Latency:    {:.0} ms  TTFT                │\n",
            self.expected_ttft_ms
        ));
        s.push_str(&format!(
            "│ Speed:      {:.1} tok/s                    │\n",
            self.expected_tokens_per_second
        ));
        s.push_str("└──────────────────────────────────────────┘\n");

        if !self.tools.is_empty() {
            s.push_str(&format!("\n🔧 Tools ({})", self.tools.len()));
            for t in &self.tools {
                s.push_str(&format!("\n   ✓ {}", t));
            }
        }

        if !self.knowledge_packs.is_empty() {
            s.push_str(&format!("\n\n📚 Knowledge Packs ({})", self.knowledge_packs.len()));
            for kp in &self.knowledge_packs {
                s.push_str(&format!("\n   📦 {}", kp));
            }
        }

        if !self.evidence.is_empty() {
            s.push_str(&format!("\n\n📊 Evidence ({})", self.evidence.len()));
            for e in &self.evidence {
                s.push_str(&format!("\n   • {}", e));
            }
        }

        s.push_str("\n\n✅ Recommendation ready.\n");
        s
    }
}

// ---------------------------------------------------------------------------
// CapabilityScheduler trait
// ---------------------------------------------------------------------------

pub trait CapabilityScheduler: Send + Sync {
    fn select_configuration(&self, task: &TaskDescriptor) -> anyhow::Result<CertifiedConfiguration>;

    fn available_tasks(&self) -> Vec<String>;

    fn explain(&self, task: &TaskDescriptor) -> anyhow::Result<String>;
}

// ---------------------------------------------------------------------------
// SchedulerEngine — the concrete implementation
// ---------------------------------------------------------------------------

pub struct SchedulerEngine {
    graph: Option<EngineeringGraph>,
}

impl SchedulerEngine {
    pub fn new() -> Self {
        Self { graph: None }
    }

    pub fn with_graph(graph: EngineeringGraph) -> Self {
        Self { graph: Some(graph) }
    }

    pub fn load_graph(path: &Path) -> Self {
        let graph = load_graph(path).ok();
        Self { graph }
    }

    /// Read graph from .athena/graph.json in the project root
    pub fn load_default() -> Self {
        let graph = load_graph(&Path::new(".athena/graph.json")).ok();
        Self { graph }
    }

    fn detect_runtimes(&self) -> Vec<(String, f64, RuntimeStatus)> {
        // Use RuntimeProber to discover actual capabilities instead of hardcoded scores
        use super::runtime_discovery::RuntimeProber;

        let discovered = RuntimeProber::probe_all();
        let mut runtimes: Vec<(String, f64, RuntimeStatus)> = discovered.iter()
            .map(|rt| {
                // Determine endpoint for status check
                let endpoint = if rt.binary_name.contains("server") {
                    Some("http://127.0.0.1:18080")
                } else if rt.binary_name.contains("ollama") {
                    Some("http://127.0.0.1:11434")
                } else if rt.binary_name.contains("lms") {
                    Some("http://127.0.0.1:1234")
                } else {
                    None
                };

                let running = endpoint
                    .map(|ep| EngineeringGraph::check_runtime_endpoint(ep))
                    .unwrap_or(false);

                let status = if running {
                    RuntimeStatus::Running
                } else {
                    // Probed binary may be available even if not running
                    RuntimeStatus::Available
                };

                // Use the probed capability score instead of hardcoded priority
                let score = rt.capability_score();
                (rt.display_name_short(), score, status)
            })
            .collect();

        // Sort: running first, then by capability score
        runtimes.sort_by(|a, b| {
            let a_priority = match a.2 {
                RuntimeStatus::Running => 3,
                RuntimeStatus::Available => 2,
                RuntimeStatus::NotRunning => 1,
                RuntimeStatus::NotFound => 0,
            };
            let b_priority = match b.2 {
                RuntimeStatus::Running => 3,
                RuntimeStatus::Available => 2,
                RuntimeStatus::NotRunning => 1,
                RuntimeStatus::NotFound => 0,
            };
            b_priority.cmp(&a_priority).then(
                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            )
        });

        runtimes
    }

    fn detect_models(&self) -> Vec<(String, f64, String)> {
        use super::find_all_models;
        let models = find_all_models();
        models.into_iter()
            .map(|m| (m.id, m.parameters_b, m.quantization))
            .collect()
    }

    fn detect_tools(&self, task: &TaskDescriptor) -> Vec<String> {
        let mut tools = Vec::new();

        // Map from languages → likely tools
        for lang in &task.languages {
            let toolset: Vec<&str> = match lang.as_str() {
                "Rust" => vec!["cargo", "rustc", "clippy", "rustfmt", "gdb"],
                "Python" => vec!["python3", "pip", "uv", "ruff", "pytest", "gdb"],
                "JavaScript" | "TypeScript" => vec!["node", "npm", "pnpm", "eslint", "prettier"],
                "Go" => vec!["go", "gofmt", "golangci-lint", "delve"],
                "C" | "C++" => vec!["gcc", "clang", "gdb", "lldb", "cmake", "make"],
                "Bash" | "Shell" => vec!["bash", "sh"],
                "Docker" => vec!["docker", "podman"],
                _ => vec![],
            };
            for tool in toolset {
                let installed = EngineeringGraph::check_command(tool).is_some();
                if installed && !tools.contains(&tool.to_string()) {
                    tools.push(tool.to_string());
                }
            }
        }

        // Add general tools
        let general = ["curl", "wget", "git", "jq", "rg", "fd"];
        for tool in &general {
            let installed = EngineeringGraph::check_command(tool).is_some();
            if installed && !tools.contains(&tool.to_string()) {
                tools.push(tool.to_string());
            }
        }

        tools.sort();
        tools
    }

    fn detect_knowledge(&self, task: &TaskDescriptor) -> Vec<(String, String)> {
        let dir = knowledge::default_packs_dir();
        let packs = knowledge::discover_packs(&dir);
        let mut relevant = Vec::new();

        for pack in &packs {
            // Match by language
            let lang_match = task.languages.is_empty()
                || pack.languages.iter().any(|l| {
                    task.languages.iter().any(|tl| tl.eq_ignore_ascii_case(l))
                });
            // Match by name/description
            let name_match = task.capabilities.iter().any(|c| {
                pack.name.to_lowercase().contains(&c.to_lowercase())
                    || pack.description.to_lowercase().contains(&c.to_lowercase())
            });

            if lang_match || name_match {
                relevant.push((pack.id.clone(), pack.name.clone()));
            }
        }

        relevant.sort_by(|a, b| a.1.cmp(&b.1));
        relevant.dedup();
        relevant
    }

    fn query_capability_database(
        &self,
        task: &TaskDescriptor,
        model_params: f64,
        runtime_name: &str,
    ) -> Option<(f64, f64, f64)> {
        let db = ResearchDatabase::new();

        // Find experiments that match this task's language/capability
        let experiments = db.list().ok()?;
        let relevant: Vec<_> = experiments.iter()
            .filter(|e| {
                // Match by model size (within 3B)
                let model_match = (e.model_params - model_params).abs() <= 3.0;
                // Match by runtime
                let runtime_match = e.runtime_id.eq_ignore_ascii_case(runtime_name);
                model_match && runtime_match
            })
            .collect();

        if relevant.is_empty() {
            return None;
        }

        // Average capability score from matching experiments
        let avg_score = relevant.iter()
            .map(|e| {
                e.level_results.last()
                    .map(|l| l.avg_score)
                    .unwrap_or(0.0)
            })
            .sum::<f64>() / relevant.len() as f64;

        let avg_ttft = relevant.iter()
            .map(|e| {
                e.level_results.first()
                    .map(|l| l.avg_ttft_ms)
                    .unwrap_or(100.0)
            })
            .sum::<f64>() / relevant.len() as f64;

        let avg_tps = relevant.iter()
            .map(|e| {
                e.level_results.first()
                    .map(|l| l.avg_tokens_per_second)
                    .unwrap_or(10.0)
            })
            .sum::<f64>() / relevant.len() as f64;

        Some((avg_score, avg_ttft, avg_tps))
    }

    /// Compute a capability score based on model params and objective
    fn compute_capability_score(
        &self,
        model_params: f64,
        quantization: &str,
        runtime_score: f64,
        tools_count: usize,
        knowledge_count: usize,
        objective: Objective,
    ) -> f64 {
        // Base score from model size (diminishing returns)
        let base = (model_params / 10.0).min(1.0) * 0.4;

        // Quantization bonus (more precision = better capability)
        let quant_bonus = match quantization {
            "Q8_0" => 0.15,
            "Q6_K" => 0.12,
            "Q5_K_M" | "Q5_K" => 0.10,
            "Q4_K_M" => 0.08,
            "Q4_0" => 0.05,
            "IQ3_XXS" | "Q3_K" => 0.03,
            "Q2_K" => 0.01,
            _ => 0.05,
        };

        // Runtime quality
        let runtime_factor = runtime_score * 0.15;

        // Tool availability
        let tools_factor = (tools_count as f64).min(10.0) / 10.0 * 0.15;

        // Knowledge
        let knowledge_factor = (knowledge_count as f64).min(5.0) / 5.0 * 0.15;

        let total = base + quant_bonus + runtime_factor + tools_factor + knowledge_factor;

        // Adjust for objective
        match objective {
            Objective::MaximumCapability => (total * 1.1).min(1.0),
            Objective::MaximumThroughput => {
                // Penalize large models (they're slower)
                let throughput_bonus = if model_params <= 7.0 { 0.05 } else { -0.05 };
                (total + throughput_bonus).min(1.0)
            }
            Objective::MinimumLatency => {
                // Penalize large models heavily
                let latency_bonus = if model_params <= 4.0 { 0.15 }
                    else if model_params <= 7.0 { 0.05 }
                    else { -0.10 };
                (total + latency_bonus).min(1.0)
            }
            Objective::MinimumVram => {
                // Prefer smaller quantizations
                let vram_bonus = match quantization {
                    "Q2_K" | "IQ3_XXS" => 0.10,
                    "Q3_K" => 0.05,
                    "Q4_0" => 0.02,
                    _ => 0.0,
                };
                let size_penalty = if model_params > 9.0 { -0.10 }
                    else if model_params > 4.0 { 0.0 }
                    else { 0.05 };
                (total + vram_bonus + size_penalty).min(1.0)
            }
            Objective::Coding => {
                // Coding benefits from medium-sized models with good quantization
                let coding_bonus = if (4.0..=9.0).contains(&model_params) { 0.08 }
                    else if model_params > 9.0 { 0.05 }
                    else { -0.02 };
                (total + coding_bonus).min(1.0)
            }
            _ => total.min(1.0),
        }
    }
}

impl CapabilityScheduler for SchedulerEngine {
    fn select_configuration(&self, task: &TaskDescriptor) -> anyhow::Result<CertifiedConfiguration> {
        // Phase 1: Detect what's available
        let runtimes = self.detect_runtimes();
        let models = self.detect_models();
        let tools = self.detect_tools(task);
        let knowledge = self.detect_knowledge(task);

        // Phase 2: Select the best runtime (first available, else first in priority)
        let (runtime_name, runtime_score, runtime_status) = runtimes.first()
            .map(|(n, s, st)| (n.clone(), *s, st.clone()))
            .unwrap_or_else(|| ("llama-server".to_string(), 0.5, RuntimeStatus::NotFound));

        let runtime_desc = match runtime_status {
            RuntimeStatus::Running => format!("{runtime_name} (running)"),
            RuntimeStatus::Available => format!("{runtime_name} (available)"),
            RuntimeStatus::NotRunning => format!("{runtime_name} (can be started)"),
            RuntimeStatus::NotFound => format!("{runtime_name} (not found — needs installation)"),
        };

        // Phase 3: VRAM-aware model selection
        let hw = hardware::detect_hardware();
        let vram_gb = hw.gpu.first().map(|g| g.vram_gb).unwrap_or(0.0);
        let ram_gb = hw.memory.available_gb;

        let (model_name, model_params, model_quant) = if models.is_empty() {
            ("No model found".to_string(), 0.0, "N/A".to_string())
        } else {
            // Filter models by VRAM availability using VramCalculator from Model Intelligence
            let vram_fitted: Vec<(String, f64, String)> = models.iter()
                .filter(|(name, params, quant)| {
                    if vram_gb <= 0.0 { return true; } // No GPU → don't filter
                    if *params <= 0.0 { return true; }  // Unknown size → don't filter
                    // Use VramCalculator for accurate estimation
                    let bits = match quant.as_str() {
                        "Q8_0" | "Q8_K" => 8.0,
                        "Q6_K" => 6.0,
                        "Q5_K_M" | "Q5_K" | "Q5_0" | "Q5_1" => 5.0,
                        "Q4_K_M" | "Q4_K" | "Q4_0" | "Q4_1" => 4.0,
                        "Q3_K" | "IQ3_XXS" | "IQ3_S" | "IQ3_M" => 3.0,
                        "Q2_K" | "IQ2_XXS" | "IQ2_XS" | "Q2_0" => 2.0,
                        "Q1_0" | "IQ1_S" | "IQ1_M" => 1.5,
                        _ => 4.0,
                    };
                    // Use VramCalculator with a practical context (16K) and Q8 KV
                    let cfg = MemoryConfig::new(*params)
                        .with_quant(bits)
                        .with_context(16384)
                        .with_kv_type(KvCacheType::Q8)
                        .with_kv_in_ram(false);
                    let est = VramCalculator::estimate(&cfg, vram_gb, ram_gb);
                    est.fits_in_vram
                })
                .map(|(name, params, quant)| (name.clone(), *params, quant.clone()))
                .collect();

            // If filtering removed everything, use all models (but flag in evidence)
            let candidates = if vram_fitted.is_empty() { &models } else { &vram_fitted };
            let was_filtered = !vram_fitted.is_empty() && vram_fitted.len() < models.len();

            // Rank models by objective score
            let mut scored: Vec<(String, f64, f64, String)> = candidates.iter()
                .map(|(name, params, quant)| {
                    let score = self.compute_capability_score(
                        *params, quant, runtime_score,
                        tools.len(), knowledge.len(), task.objective,
                    );
                    (name.clone(), score, *params, quant.clone())
                })
                .collect();

            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            if scored.is_empty() {
                ("No model fits in VRAM".to_string(), 0.0, "N/A".to_string())
            } else {
                let best = &scored[0];
                (best.0.clone(), best.2, best.3.clone())
            }
        };

        // Phase 4: Query capability database for evidence
        let db_result = self.query_capability_database(task, model_params, &runtime_name);

        let (avg_score, avg_ttft, avg_tps) = db_result.unwrap_or_else(|| {
            // Estimate based on model size and objective
            let est_score = self.compute_capability_score(
                model_params, &model_quant, runtime_score,
                tools.len(), knowledge.len(), task.objective,
            );
            let est_ttft = match model_params {
                p if p > 20.0 => 200.0,
                p if p > 9.0 => 120.0,
                p if p > 4.0 => 70.0,
                _ => 40.0,
            };
            let est_tps = match model_params {
                p if p > 20.0 => 15.0,
                p if p > 9.0 => 25.0,
                p if p > 4.0 => 42.0,
                _ => 60.0,
            };
            (est_score, est_ttft, est_tps)
        });

        // Phase 5: Build evidence
        let mut evidence = Vec::new();

        if db_result.is_some() {
            evidence.push(format!(
                "Capability database has {} matching experiments for {}B models on {}",
                "previous", model_params, runtime_name
            ));
        } else {
            evidence.push("No prior benchmark data — using model-based estimation".to_string());
        }

        evidence.push(format!(
            "Runtime '{}' selected: {} ({:.0} priority score)",
            runtime_name, runtime_status_str(&runtime_status), runtime_score * 100.0
        ));

        evidence.push(format!(
            "Model '{}' ({:.0}B, {}) — {:.1}% capability score under {:?} objective",
            model_name, model_params, model_quant,
            avg_score * 100.0, task.objective
        ));

        if !tools.is_empty() {
            evidence.push(format!("{} tools detected and available", tools.len()));
        }
        if !knowledge.is_empty() {
            evidence.push(format!("{} relevant knowledge packs available", knowledge.len()));
        }

        let confidence = if db_result.is_some() { 0.85 } else { 0.65 };

        Ok(CertifiedConfiguration {
            task: task.name.clone(),
            objective: format!("{:?}", task.objective),
            runtime: runtime_desc,
            model: model_name,
            model_params_b: model_params,
            model_quant,
            tools,
            knowledge_packs: knowledge.into_iter().map(|(_, name)| name).collect(),
            capability_score: avg_score,
            confidence,
            expected_ttft_ms: avg_ttft,
            expected_tokens_per_second: avg_tps,
            evidence,
        })
    }

    fn available_tasks(&self) -> Vec<String> {
        TaskDescriptor::all_profiles().into_iter().map(|t| t.name).collect()
    }

    fn explain(&self, task: &TaskDescriptor) -> anyhow::Result<String> {
        let config = self.select_configuration(task)?;
        Ok(config.display())
    }
}

fn runtime_status_str(status: &RuntimeStatus) -> &str {
    match status {
        RuntimeStatus::Running => "currently running",
        RuntimeStatus::Available => "binary found in PATH",
        RuntimeStatus::NotRunning => "binary found, not running",
        RuntimeStatus::NotFound => "not installed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_detection() {
        let engine = SchedulerEngine::new();
        let runtimes = engine.detect_runtimes();
        // Should at least find something (even if NotFound)
        assert!(!runtimes.is_empty(), "Should detect at least one runtime entry");
        // First entry should contain 'llama' (highest priority)
        assert!(runtimes[0].0.to_lowercase().contains("llama"),
            "First runtime should contain 'llama', got: {}", runtimes[0].0);
    }

    #[test]
    fn test_basic_recommendation() {
        let engine = SchedulerEngine::new();
        let task = TaskDescriptor::rust_development();
        let config = engine.select_configuration(&task);
        assert!(config.is_ok(), "Should produce a configuration");
        let config = config.unwrap();
        assert_eq!(config.task, "Rust Development");
        assert!(!config.evidence.is_empty(), "Should have evidence");
    }

    #[test]
    fn test_available_tasks() {
        let engine = SchedulerEngine::new();
        let tasks = engine.available_tasks();
        assert!(tasks.len() >= 6, "Should have at least 6 task profiles");
        assert!(tasks.contains(&"Rust Development".to_string()));
        assert!(tasks.contains(&"Web Pentest".to_string()));
    }

    #[test]
    fn test_objective_parsing() {
        assert_eq!(Objective::from_name("MaximumCapability"), Some(Objective::MaximumCapability));
        assert_eq!(Objective::from_name("Maximum Throughput"), Some(Objective::MaximumThroughput));
        assert!(Objective::from_name("Unknown").is_none());
    }

    #[test]
    fn test_capability_score() {
        let engine = SchedulerEngine::new();
        // 9B Q4_K_M on Coding objective should give decent score
        let score = engine.compute_capability_score(
            9.0, "Q4_K_M", 0.95, 5, 2, Objective::Coding,
        );
        assert!(score > 0.3, "Score should be at least 0.3");
        assert!(score <= 1.0, "Score should not exceed 1.0");

        // 4B Q2_K on MinimumVram objective should also work
        let score_vram = engine.compute_capability_score(
            4.0, "Q2_K", 0.80, 2, 0, Objective::MinimumVram,
        );
        assert!(score_vram > 0.2, "VRAM-optimized score should be at least 0.2");
    }

    #[test]
    fn test_detect_tools_for_task() {
        let engine = SchedulerEngine::new();
        let task = TaskDescriptor::rust_development();
        let tools = engine.detect_tools(&task);
        // If cargo is installed, it should appear
        let _has_cargo = tools.iter().any(|t| t == "cargo");
        // This test will pass regardless of whether cargo is actually installed
        // But the tool list should at least have some entries from general tools
        assert!(!tools.is_empty() || EngineeringGraph::check_command("rustc").is_none(),
            "Should detect tools or have none installed");
    }
}
