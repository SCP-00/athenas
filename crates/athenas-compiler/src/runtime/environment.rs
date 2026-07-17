use std::path::Path;
use std::process::Command;

use super::graph::eng_node::{EngEdgeKind, EngNode, EngNodeData, RuntimeStatus};
use super::graph::EngineeringGraph;

/// Environment Builder — discovers reality and populates the Engineering Graph.
/// Does NOT recommend, decide, or optimize. Only observes.
pub struct EnvironmentBuilder;

impl EnvironmentBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Build the complete Engineering Graph for a project directory.
    pub fn build(&self, project_root: &Path) -> EngineeringGraph {
        let mut graph = EngineeringGraph::new();
        let project_id = self.detect_project(&mut graph, project_root);
        self.detect_languages(&mut graph, project_root);
        self.detect_tools(&mut graph, &project_id);
        self.detect_runtimes(&mut graph, &project_id);
        self.detect_models(&mut graph);
        self.detect_hardware(&mut graph, &project_id);
        self.detect_knowledge(&mut graph);
        graph
    }

    // ── Project Detection ──

    fn detect_project(&self, graph: &mut EngineeringGraph, root: &Path) -> String {
        let name = root.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let canonical = root.canonicalize().ok()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| root.to_string_lossy().to_string());

        let id = EngineeringGraph::make_id("project", &name);
        graph.add_node(EngNode::new(id.clone(), EngNodeData::Project {
            name: name.clone(), path: canonical,
        }));

        // Detect config files → add as File nodes
        let configs = [
            ("Cargo.toml", "Rust"),
            ("package.json", "JavaScript"),
            ("pyproject.toml", "Python"),
            ("setup.py", "Python"),
            ("requirements.txt", "Python"),
            ("go.mod", "Go"),
            ("CMakeLists.txt", "C"),
            ("Makefile", "C"),
            ("Dockerfile", "Docker"),
            ("docker-compose.yml", "Docker"),
        ];

        for (filename, lang) in &configs {
            let path = root.join(filename);
            if path.exists() {
                let file_id = EngineeringGraph::make_id("file", filename);
                graph.add_node(EngNode::new(file_id.clone(), EngNodeData::File {
                    path: path.to_string_lossy().to_string(),
                    language: lang.to_string(),
                }));
                graph.add_edge(id.clone(), file_id, EngEdgeKind::Contains);

                // Also add a Language node
                let lang_id = EngineeringGraph::make_id("lang", lang);
                graph.add_node(EngNode::new(lang_id.clone(), EngNodeData::Language {
                    name: lang.to_string(),
                    version: "detected".to_string(),
                }));
                graph.add_edge(id.clone(), lang_id, EngEdgeKind::HasTool);
            }
        }
        id
    }

    // ── Language & Toolchain Detection ──

    fn detect_languages(&self, graph: &mut EngineeringGraph, root: &Path) {
        // For each detected config file, check actual tool versions
        let checks = [
            ("Cargo.toml", "rustc", "--version", "Language"),
            ("package.json", "node", "--version", "Language"),
            ("pyproject.toml", "python3", "--version", "Language"),
            ("go.mod", "go", "version", "Language"),
        ];

        for (config, cmd, arg, _) in &checks {
            if root.join(config).exists() {
                let version = Command::new(cmd).arg(arg).output().ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "detected".to_string());

                // Update the Language node with actual version
                let lang_name = match *cmd {
                    "rustc" => "Rust",
                    "node" => "JavaScript",
                    "python3" => "Python",
                    "go" => "Go",
                    _ => cmd,
                };
                let lang_id = EngineeringGraph::make_id("lang", lang_name);
                graph.add_node(EngNode::new(lang_id, EngNodeData::Language {
                    name: lang_name.to_string(),
                    version,
                }));
            }
        }
    }

    // ── Tool Detection ──

    fn detect_tools(&self, graph: &mut EngineeringGraph, project_id: &str) {
        // Compilers & Interpreters
        let compilers = ["rustc", "gcc", "clang", "go", "javac", "python3", "node"];
        for cmd in &compilers {
            if EngineeringGraph::check_command(cmd).is_some() {
                let id = EngineeringGraph::make_id("compiler", cmd);
                graph.add_node(EngNode::new(id.clone(), EngNodeData::Compiler {
                    name: cmd.to_string(),
                    version: get_version(cmd).unwrap_or_default(),
                }));
                graph.add_edge(project_id.to_string(), id, EngEdgeKind::CompiledBy);
            }
        }

        // Debuggers
        let debuggers = ["gdb", "lldb", "delve", "rust-gdb"];
        for cmd in &debuggers {
            if EngineeringGraph::check_command(cmd).is_some() {
                let id = EngineeringGraph::make_id("debugger", cmd);
                graph.add_node(EngNode::new(id.clone(), EngNodeData::Debugger {
                    name: cmd.to_string(),
                    version: get_version(cmd).unwrap_or_default(),
                }));
                graph.add_edge(project_id.to_string(), id, EngEdgeKind::DebuggedBy);
            }
        }

        // Formatters
        let formatters = ["rustfmt", "prettier", "black", "gofmt"];
        for cmd in &formatters {
            if EngineeringGraph::check_command(cmd).is_some() {
                let id = EngineeringGraph::make_id("formatter", cmd);
                graph.add_node(EngNode::new(id.clone(), EngNodeData::Formatter {
                    name: cmd.to_string(),
                    version: get_version(cmd).unwrap_or_default(),
                }));
                graph.add_edge(project_id.to_string(), id, EngEdgeKind::FormattedBy);
            }
        }

        // Linters
        let linters = ["clippy-driver", "eslint", "ruff", "golangci-lint"];
        for cmd in &linters {
            if EngineeringGraph::check_command(cmd).is_some() {
                let id = EngineeringGraph::make_id("linter", cmd);
                graph.add_node(EngNode::new(id.clone(), EngNodeData::Linter {
                    name: cmd.to_string(),
                    version: get_version(cmd).unwrap_or_default(),
                }));
                graph.add_edge(project_id.to_string(), id, EngEdgeKind::LintedBy);
            }
        }

        // Test runners
        let testers = ["cargo", "pytest", "jest", "go"];
        for cmd in &testers {
            if EngineeringGraph::check_command(cmd).is_some() {
                let id = EngineeringGraph::make_id("test", cmd);
                graph.add_node(EngNode::new(id.clone(), EngNodeData::TestRunner {
                    name: cmd.to_string(),
                    version: get_version(cmd).unwrap_or_default(),
                }));
            }
        }
    }

    // ── Runtime Detection ──
    // Uses RuntimeProber to discover actual capabilities instead of hardcoded lists.

    fn detect_runtimes(&self, graph: &mut EngineeringGraph, project_id: &str) {
        use super::runtime_discovery::RuntimeProber;

        let discovered = RuntimeProber::probe_all();
        for rt in &discovered {
            // Determine status: check if endpoint is reachable
            let endpoint = if rt.binary_name.contains("server") {
                Some(format!("http://127.0.0.1:18080"))
            } else if rt.binary_name.contains("ollama") {
                Some(format!("http://127.0.0.1:11434"))
            } else if rt.binary_name.contains("lms") {
                Some(format!("http://127.0.0.1:1234"))
            } else {
                None
            };

            let running = endpoint.as_ref()
                .map(|ep| EngineeringGraph::check_runtime_endpoint(ep))
                .unwrap_or(false);

            let status = if running {
                RuntimeStatus::Running
            } else {
                RuntimeStatus::NotRunning
            };

            let id = EngineeringGraph::make_id("runtime", &rt.display_name);
            graph.add_node(EngNode::new(id.clone(), EngNodeData::Runtime {
                name: rt.display_name.clone(),
                status,
                endpoint: endpoint.clone(),
            }));
            graph.add_edge(project_id.to_string(), id, EngEdgeKind::DependsOn);
        }
    }

    // ── Model Detection ──

    fn detect_models(&self, graph: &mut EngineeringGraph) {
        // Use the canonical find_all_models() from runtime/mod.rs
        let all_models = super::find_all_models();
        for model in &all_models {
            let fname = Path::new(&model.path)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let id = EngineeringGraph::make_id("model", &fname);
            graph.add_node(EngNode::new(id.clone(), EngNodeData::Model {
                name: model.id.clone(),
                path: model.path.clone(),
                params_b: model.parameters_b,
                quant: model.quantization.clone(),
            }));
        }

        // Also detect Ollama models
        if let Ok(output) = Command::new("ollama").args(["list"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                if line.trim().is_empty() { continue; }
                let name = line.split_whitespace().next().unwrap_or("unknown");
                let id = EngineeringGraph::make_id("model", name);
                graph.add_node(EngNode::new(id.clone(), EngNodeData::Model {
                    name: name.to_string(),
                    path: format!("ollama://{name}"),
                    params_b: 0.0,
                    quant: "unknown".to_string(),
                }));
                // Connect to Ollama runtime
                graph.add_edge_checked("runtime::ollama", &id, EngEdgeKind::RunsOn);
            }
        }
    }

    // ── Hardware Detection ──

    fn detect_hardware(&self, graph: &mut EngineeringGraph, project_id: &str) {
        use super::hardware;
        let hw = hardware::detect_hardware();

        // CPU
        let cpu_id = EngineeringGraph::make_id("cpu", &hw.cpu.model);
        graph.add_node(EngNode::new(cpu_id.clone(), EngNodeData::Cpu {
            model: hw.cpu.model,
            cores: hw.cpu.cores,
            threads: hw.cpu.threads,
        }));
        graph.add_edge(project_id.to_string(), cpu_id, EngEdgeKind::DetectedBy);

        // GPU(s)
        for gpu in &hw.gpu {
            let gpu_id = EngineeringGraph::make_id("gpu", &gpu.model);
            graph.add_node(EngNode::new(gpu_id.clone(), EngNodeData::Gpu {
                model: gpu.model.clone(),
                vram_gb: gpu.vram_gb,
                driver: gpu.driver_version.clone(),
            }));
            graph.add_edge(project_id.to_string(), gpu_id, EngEdgeKind::DetectedBy);
        }

        // RAM
        let ram_id = "hw::ram".to_string();
        graph.add_node(EngNode::new(ram_id.clone(), EngNodeData::Ram {
            total_gb: hw.memory.total_gb,
            available_gb: hw.memory.available_gb,
        }));
        graph.add_edge(project_id.to_string(), ram_id, EngEdgeKind::DetectedBy);

        // OS
        let os_id = EngineeringGraph::make_id("os", &hw.os.name);
        graph.add_node(EngNode::new(os_id.clone(), EngNodeData::Os {
            name: hw.os.name,
            version: hw.os.version,
            arch: hw.os.arch,
        }));
        graph.add_edge(project_id.to_string(), os_id, EngEdgeKind::DetectedBy);
    }

    // ── Knowledge Detection ──

    fn detect_knowledge(&self, graph: &mut EngineeringGraph) {
        use super::knowledge;
        let dir = knowledge::default_packs_dir();
        let packs = knowledge::discover_packs(&dir);
        for pack in &packs {
            let id = EngineeringGraph::make_id("kp", &pack.id);
            let lang = pack.languages.first().cloned().unwrap_or_default();
            let lang_for_edge = lang.clone();
            graph.add_node(EngNode::new(id.clone(), EngNodeData::KnowledgePack {
                id: pack.id.clone(),
                name: pack.name.clone(),
                version: pack.version.clone(),
                language: lang,
            }));
            // Connect to language
            let lang_id = EngineeringGraph::make_id("lang", &lang_for_edge);
            graph.add_edge_checked(&id, &lang_id, EngEdgeKind::HasKnowledge);
            graph.add_edge_checked(&lang_id, &id, EngEdgeKind::HasKnowledge);
        }
    }
}

fn get_version(cmd: &str) -> Option<String> {
    let output = Command::new(cmd).arg("--version").output().ok()?;
    if output.status.success() {
        String::from_utf8(output.stdout).ok()
            .or_else(|| String::from_utf8(output.stderr).ok())
            .map(|s| s.lines().next().unwrap_or("").trim().to_string())
            .filter(|s| !s.is_empty())
    } else {
        None
    }
}

fn infer_params(path: &str) -> f64 {
    let lower = path.to_lowercase();
    if lower.contains("27b") { 27.0 }
    else if lower.contains("9b") { 9.0 }
    else if lower.contains("7b") { 7.0 }
    else if lower.contains("4b") { 4.0 }
    else if lower.contains("8b") { 8.0 }
    else if lower.contains("3b") { 3.0 }
    else if lower.contains("1b") { 1.0 }
    else { 0.0 }
}

fn infer_quant(path: &str) -> String {
    let lower = path.to_lowercase();
    if lower.contains("q4_k_m") { "Q4_K_M".into() }
    else if lower.contains("q4_0") { "Q4_0".into() }
    else if lower.contains("q8_0") { "Q8_0".into() }
    else if lower.contains("q2_k") { "Q2_K".into() }
    else if lower.contains("q3_k") { "Q3_K".into() }
    else if lower.contains("q5_k") { "Q5_K".into() }
    else if lower.contains("iq3_xxs") { "IQ3_XXS".into() }
    else if lower.contains("iq4") { "IQ4".into() }
    else { "unknown".into() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_builder_creates_graph() {
        let builder = EnvironmentBuilder::new();
        let graph = builder.build(Path::new("."));
        // Should always detect hardware and OS
        assert!(graph.nodes_by_type(|d| matches!(d, EngNodeData::Os { .. })).len() >= 1);
        assert!(graph.nodes_by_type(|d| matches!(d, EngNodeData::Cpu { .. })).len() >= 1);
        assert!(graph.nodes_by_type(|d| matches!(d, EngNodeData::Ram { .. })).len() >= 1);
    }

    #[test]
    fn test_detect_project_cargo() {
        let builder = EnvironmentBuilder::new();
        let graph = builder.build(Path::new("../..")); // Project root
        let projects = graph.nodes_by_type(|d| matches!(d, EngNodeData::Project { .. }));
        assert!(!projects.is_empty());
    }
}
