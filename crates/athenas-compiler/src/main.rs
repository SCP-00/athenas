mod cli;
mod generators;
mod graph;
mod parser;
mod runtime;
mod validator;

use clap::{Parser, Subcommand};
use colored::*;
use std::path::{Path, PathBuf};

use runtime::benchmark::BenchmarkRegistry;
use runtime::graph::EngineeringGraph;
use runtime::environment::EnvironmentBuilder;

/// Subcommands for `ath phase`
#[derive(Subcommand, Debug)]
pub enum PhaseAction {
    /// Run a single phase and persist the result
    Run {
        /// Phase ID to execute (e.g., "PHASE-0001-hardware")
        phase_id: String,

        /// Experiment ID (auto-generated if omitted)
        #[arg(long)]
        experiment: Option<String>,

        /// Path to GGUF model (required for phases like PHASE-0004, PHASE-0005, PHASE-0006)
        #[arg(short, long)]
        model: Option<PathBuf>,

        /// Path to llama-server binary (required for PHASE-0006 execution-lab)
        #[arg(long)]
        runtime: Option<PathBuf>,

        /// Output structured JSON
        #[arg(short, long)]
        json: bool,
    },
    /// List all registered phases
    List,
}

/// Subcommands for `ath pack`
#[derive(Subcommand, Debug)]
pub enum PackAction {
    /// List all available knowledge packs
    List,
    /// Show details of a specific pack
    Show {
        /// Pack ID to show
        id: String,
    },
}

/// Subcommands for `ath workspace`
#[derive(Subcommand, Debug)]
pub enum WorkspaceAction {
    /// List all available workspaces
    List,
    /// Create a new workspace from a pack
    Create {
        /// Pack ID to create workspace from
        pack_id: String,

        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },
}

/// Athenas Knowledge Compiler — compiles engineering documentation into structured knowledge artifacts
#[derive(Parser, Debug)]
#[command(
    name = "ath",
    version = "0.1.0",
    about = "Engineering Compiler for Athenas"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Parse, validate schemas, check IDs and references, print diagnostics, exit with code
    Validate {
        /// Path to the project root (defaults to current directory)
        #[arg(default_value = ".")]
        project_root: PathBuf,

        /// Path to schemas directory (defaults to schemas/)
        #[arg(short, long, default_value = "schemas")]
        schemas: PathBuf,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Parse documents and build the knowledge graph (outputs graph.json)
    Graph {
        /// Path to the project root (defaults to current directory)
        #[arg(default_value = ".")]
        project_root: PathBuf,

        /// Output directory for generated JSON files (defaults to .knowledge/)
        #[arg(short, long, default_value = ".knowledge")]
        output: PathBuf,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Run inference against a local model via llama.cpp
    Run {
        /// Path to the GGUF model file (auto-detect if omitted)
        #[arg(short, long)]
        model: Option<PathBuf>,

        /// Input prompt (reads from stdin if omitted)
        #[arg(short, long)]
        prompt: Option<String>,

        /// Path to a workspace directory (loads prompt from athenas.json)
        #[arg(short = 'w', long)]
        workspace: Option<PathBuf>,

        /// Maximum tokens to generate (default: 512)
        #[arg(short = 'n', long, default_value = "512")]
        max_tokens: usize,

        /// Temperature (default: 0.7)
        #[arg(short, long, default_value = "0.7")]
        temperature: f64,

        /// Output structured JSON instead of human-readable format
        #[arg(short, long)]
        json: bool,

        /// Path to llama-server binary (default: llama-server in PATH)
        #[arg(long)]
        server_path: Option<PathBuf>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Detect hardware, discover models, list capabilities
    Doctor {
        /// Output structured JSON instead of human-readable format
        #[arg(short, long)]
        json: bool,
    },

    /// Full autonomous certification — discovers hardware, plans experiments, recovers from OOM, generates knowledge report
    CertifyModel {
        /// Path to the GGUF model file (auto-detect if omitted)
        #[arg(short, long)]
        model: Option<PathBuf>,

        /// Skip configurations known to fail from previous experiments
        #[arg(long, default_value = "true")]
        skip_known: bool,

        /// Output structured JSON
        #[arg(short, long)]
        json: bool,
    },

    /// Benchmark a model against a capability and generate certification report
    Certify {
        /// Path to the GGUF model file (auto-detect if omitted)
        #[arg(short, long)]
        model: Option<PathBuf>,

        /// Capability to benchmark (default: text-generation)
        #[arg(short, long, default_value = "text-generation")]
        capability: String,

        /// Benchmark runner ID (e.g., "human-eval"). Uses capability mode if omitted.
        #[arg(short = 'B', long)]
        benchmark: Option<String>,

        /// Knowledge pack ID to benchmark WITH (compares raw vs packed)
        #[arg(short = 'P', long)]
        pack: Option<String>,

        /// Certification level (default: 1)
        /// L0=raw, L1=+knowledge, L2=+workspace, L3=+tools
        #[arg(short = 'L', long, default_value = "1")]
        level: u8,

        /// Path to workspace directory (for level L2+)
        #[arg(long)]
        workspace: Option<PathBuf>,

        /// Maximum tokens to generate
        #[arg(short = 'n', long, default_value = "100")]
        max_tokens: usize,

        /// Path to llama-server binary
        #[arg(long)]
        server_path: Option<PathBuf>,

        /// Output structured JSON instead of human-readable format
        #[arg(short, long)]
        json: bool,

        /// Use MockRuntime (deterministic, no GPU needed, for CI testing)
        #[arg(short = 'M', long)]
        mock: bool,
    },

    /// List and inspect knowledge packs
    Pack {
        /// Subcommand: list, show
        #[command(subcommand)]
        action: PackAction,

        /// Path to packs directory (defaults to knowledge/packs/)
        #[arg(short, long)]
        packs_dir: Option<PathBuf>,

        /// Output JSON
        #[arg(short, long)]
        json: bool,
    },

    /// Create or manage engineering workspaces
    Workspace {
        /// Action: create, list, show
        #[command(subcommand)]
        action: WorkspaceAction,
    },

    /// Build knowledge packs from providers (man, help, etc.)
    Knowledge {
        /// Provider to use (e.g. "man")
        provider: String,

        /// Query to pass to the provider (e.g. "go-build")
        query: String,

        /// Output directory for the generated pack YAML
        #[arg(short, long, default_value = ".knowledge/packs")]
        output: PathBuf,

        /// Output structured JSON report
        #[arg(short, long)]
        json: bool,
    },

    /// Full pipeline: validate + graph + all artifacts (project.yaml, index, search, crossrefs)
    Build {
        /// Path to the project root (defaults to current directory)
        #[arg(default_value = ".")]
        project_root: PathBuf,

        /// Output directory for generated JSON files (defaults to .knowledge/)
        #[arg(short, long, default_value = ".knowledge")]
        output: PathBuf,

        /// Path to schemas directory (defaults to schemas/)
        #[arg(short, long, default_value = "schemas")]
        schemas: PathBuf,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Inspect the Engineering Graph — display project knowledge, debug broken edges
    Inspect {
        /// Section to inspect: models, runtime, knowledge, hardware, tools, broken
        #[arg(short, long)]
        section: Option<String>,

        /// Output structured JSON
        #[arg(short, long)]
        json: bool,
    },

    /// Initialize an Engineering Environment from a project directory
    EnvInit {
        /// Path to the project root (defaults to current directory)
        #[arg(default_value = ".")]
        project_root: PathBuf,

        /// Output structured JSON
        #[arg(short, long)]
        json: bool,
    },

    /// Run a single phase in the experiment pipeline
    Phase {
        /// Subcommand: run, list
        #[command(subcommand)]
        action: PhaseAction,
    },

    /// Analyze a GGUF model — read metadata, calculate memory, recommend configs
    Analyze {
        /// Path to the GGUF model file
        model: PathBuf,

        /// Output structured JSON
        #[arg(short, long)]
        json: bool,
    },

    /// Show the Athena laboratory dashboard
    Tui,

    /// Knowledge Base: list, show, mark outdated
    KnowledgeBase {
        /// Action: list, show, outdated
        action: String,

        /// Question ID (for show action)
        #[arg(short, long)]
        question: Option<String>,

        /// Output structured JSON
        #[arg(short, long)]
        json: bool,
    },

    /// Run a campaign (autonomous study with repetitions)
    Campaign {
        /// Study ID to campaign (e.g., "PC-001")
        study_id: String,

        /// Path to GGUF model
        #[arg(short, long)]
        model: PathBuf,

        /// Number of repetitions (default: from study)
        #[arg(short, long)]
        repetitions: Option<u32>,

        /// Output structured JSON
        #[arg(short, long)]
        json: bool,
    },

    /// Run a scientific study (declarative, auto-discovers phases and dependencies)
    Study {
        /// Study ID (e.g., "SP-005", "PC-001")
        study_id: String,

        /// Output structured JSON
        #[arg(short, long)]
        json: bool,
    },

    /// List all available scientific studies
    StudyList,

    /// Manage the experiment queue (persistent, autonomous)
    Queue {
        /// Action: add, list, process, clean, show
        action: String,

        /// Model path (required for add)
        #[arg(short, long)]
        model: Option<PathBuf>,

        /// Experiment ID (for show action)
        #[arg(short, long)]
        experiment: Option<String>,

        /// Days threshold for clean action
        #[arg(short = 'd', long, default_value = "7")]
        days: u64,

        /// Output structured JSON
        #[arg(short, long)]
        json: bool,
    },

    /// Recommend the best configuration for an engineering task
    Recommend {
        /// Task name (e.g., "Rust Development", "Web Pentest")
        /// If omitted, lists all available task profiles
        task: Option<String>,

        /// Optimization objective
        /// Options: MaximumCapability, MaximumThroughput, MinimumLatency,
        ///          MinimumVram, OfflineOnly, Coding, Research, Default
        #[arg(short, long)]
        objective: Option<String>,

        /// Path to the Engineering Graph file (.athena/graph.json)
        #[arg(long)]
        graph: Option<PathBuf>,

        /// Output structured JSON
        #[arg(short, long)]
        json: bool,
    },
}

fn print_banner() {
    println!();
    println!("{}", "╔══════════════════════════════════════════╗".bright_blue());
    println!("{}", "║     Athenas Knowledge Compiler v0.1.0    ║".bright_blue());
    println!("{}", "╚══════════════════════════════════════════╝".bright_blue());
    println!();
}



fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    print_banner();

    match &cli.command {
        Some(Commands::Run { model, prompt, workspace, max_tokens, temperature, json, server_path, verbose }) => {
            let exit_code = cli::run_inference(
                model.as_deref(), prompt.as_deref(), workspace.as_deref(),
                *max_tokens, *temperature, *json, server_path.as_deref(), *verbose,
            )?;
            std::process::exit(exit_code);
        }

        Some(Commands::Validate { project_root, schemas, verbose }) => {
            let root = project_root.canonicalize()?;
            let exit_code = cli::run_validate(&root, schemas, *verbose)?;
            std::process::exit(exit_code);
        }

        Some(Commands::Graph { project_root, output, verbose }) => {
            let root = project_root.canonicalize()?;
            let exit_code = cli::run_graph_cmd(&root, output, *verbose)?;
            std::process::exit(exit_code);
        }

        Some(Commands::Doctor { json }) => {
            let exit_code = cli::run_doctor(*json)?;
            std::process::exit(exit_code);
        }

        Some(Commands::Pack { action, packs_dir, json }) => {
            let exit_code = cli::run_pack(action, packs_dir.as_deref(), *json)?;
            std::process::exit(exit_code);
        }

        Some(Commands::Workspace { action }) => {
            let exit_code = cli::run_workspace(action)?;
            std::process::exit(exit_code);
        }

        Some(Commands::Knowledge { provider, query, output, json }) => {
            let exit_code = cli::run_knowledge_build(provider, query, output, *json)?;
            std::process::exit(exit_code);
        }

        Some(Commands::CertifyModel { model, skip_known, json }) => {
            let model_path = model.clone().unwrap_or_else(|| {
                runtime::find_model(None)
                    .map(|p| {
                        eprintln!("  🔄 Auto-detected model: {}", p.display());
                        p
                    })
                    .unwrap_or_else(|_| {
                        eprintln!("  ⚠ No model found. Use --model <path>");
                        std::process::exit(1);
                    })
            });
            let exit_code = cli::run_certify_model(&model_path, *skip_known, *json)?;
            std::process::exit(exit_code);
        }

        Some(Commands::Certify { model, capability, benchmark, pack, level, workspace, max_tokens, server_path, json, mock }) => {
            let mut registry = BenchmarkRegistry::new();
            registry.register(Box::new(runtime::benchmarks::human_eval::HumanEvalRunner::new()));
            registry.register(Box::new(runtime::benchmarks::aider_polyglot::AiderPolyglotRunner::new()));
            let exit_code = cli::run_certify(
                &registry, model.as_deref(), benchmark.as_deref(), capability,
                pack.as_deref(), *level, workspace.as_deref(), *max_tokens,
                server_path.as_deref(), *json, *mock,
            )?;
            std::process::exit(exit_code);
        }

        Some(Commands::Build { project_root, output, schemas, verbose }) => {
            let root = project_root.canonicalize()?;
            let exit_code = cli::run_build(&root, output, schemas, *verbose)?;
            std::process::exit(exit_code);
        }

        Some(Commands::Inspect { section, json }) => {
            let graph_path = PathBuf::from(".athena/graph.json");
            let exit_code = cli::run_inspect(&graph_path, section.as_deref(), *json)?;
            std::process::exit(exit_code);
        }

        Some(Commands::Phase { action }) => {
            match action {
                PhaseAction::Run { phase_id, experiment, model, runtime, json } => {
                    let exit_code = cli::run_phase(phase_id, experiment.as_deref(), model.as_deref(), runtime.as_deref(), *json)?;
                    std::process::exit(exit_code);
                }
                PhaseAction::List => {
                    let exit_code = cli::list_phases()?;
                    std::process::exit(exit_code);
                }
            }
        }

        Some(Commands::Tui) => {
            // Render the laboratory dashboard
            runtime::tui::render_dashboard();
            std::process::exit(0);
        }

        Some(Commands::KnowledgeBase { action, question, json }) => {
            let state_dir = std::path::Path::new(".state");
            let mut kb = runtime::knowledge_base::KnowledgeBase::load(state_dir);
            match action.as_str() {
                "list" => {
                    let questions = kb.questions();
                    if *json {
                        println!("{}", serde_json::json!({
                            "total_revisions": kb.total_revisions(),
                            "questions": questions,
                        }));
                    } else {
                        println!("╔══════════════════════════════════════════╗");
                        println!("║     Knowledge Base                       ║");
                        println!("╚══════════════════════════════════════════╝");
                        println!();
                        println!("  📚 {} AnswerRevisions", kb.total_revisions());
                        println!();
                        for q in &questions {
                            if let Some(latest) = kb.latest(q) {
                                println!("{}", latest.display());
                            }
                        }
                    }
                }
                "show" => {
                    let q = question.as_deref().ok_or_else(|| anyhow::anyhow!("--question required for show"))?;
                    if let Some(latest) = kb.latest(q) {
                        if *json {
                            println!("{}", serde_json::to_string_pretty(latest)?);
                        } else {
                            println!("{}", latest.display());
                        }
                    } else {
                        anyhow::bail!("No answer found for question: {q}");
                    }
                }
                "outdated" => {
                    let count = kb.mark_all_outdated("Manual: user request");
                    println!("  Marked {count} answers as outdated");
                }
                _ => anyhow::bail!("Unknown action. Use: list, show, outdated"),
            }
            std::process::exit(0);
        }

        Some(Commands::Campaign { study_id, model, repetitions, json }) => {
            let studies = runtime::study::built_in_studies();
            let study = studies.get(study_id)
                .ok_or_else(|| {
                    let available: Vec<&String> = studies.keys().collect();
                    anyhow::anyhow!("Unknown study '{study_id}'. Available: {:?}", available)
                })?;

            let mut campaign = runtime::campaign::Campaign::from_study(study, &model.to_string_lossy());
            if let Some(rep) = repetitions {
                campaign.repetitions = *rep;
            }

            if !json {
                println!("╔══════════════════════════════════════════╗");
                println!("║     Athena Campaign Engine v0.1.0         ║");
                println!("╚══════════════════════════════════════════╝");
                println!();
            }

            let mut registry = runtime::phase::phases::PhaseRegistry::new();
            runtime::phase::phases::register_all_phases(&mut registry);

            let report = campaign.execute(&registry)
                .map_err(|e| anyhow::anyhow!("Campaign '{}' failed: {e}", study_id))?;

            if *json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
            std::process::exit(if report.errors.is_empty() { 0 } else { 1 });
        }

        Some(Commands::Study { study_id, json }) => {
            // Run a scientific study
            let studies = runtime::study::built_in_studies();
            let study = studies.get(study_id)
                .ok_or_else(|| {
                    let available: Vec<&String> = studies.keys().collect();
                    anyhow::anyhow!("Unknown study '{study_id}'. Available: {:?}", available)
                })?;

            let mut registry = crate::runtime::phase::phases::PhaseRegistry::new();
            crate::runtime::phase::phases::register_all_phases(&mut registry);

            if !json {
                println!("╔══════════════════════════════════════════╗");
                println!("║     Athena Study System v0.1.0            ║");
                println!("╚══════════════════════════════════════════╝");
                println!();
            }

            let report = study.execute(&registry)
                .map_err(|e| anyhow::anyhow!("Study '{study_id}' failed: {e}"))?;

            if *json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!();
                println!("  📊 Study Report: {}", report.study_id);
                println!("  Status: {}", report.status);
                println!("  Phases: {}/{} completed", report.phases_completed, report.total_phases);
                println!("  Errors: {}", report.errors.len());
                if !report.errors.is_empty() {
                    for err in &report.errors {
                        println!("    ❌ {err}");
                    }
                }
                println!();
            }
            std::process::exit(if report.errors.is_empty() { 0 } else { 1 });
        }

        Some(Commands::StudyList) => {
            let studies = runtime::study::built_in_studies();
            println!("╔══════════════════════════════════════════╗");
            println!("║     Athena Study System — Studies       ║");
            println!("╚══════════════════════════════════════════╝");
            println!();
            let mut ids: Vec<&String> = studies.keys().collect();
            ids.sort();
            for id in ids {
                if let Some(study) = studies.get(id) {
                    println!("  📖 {} — {}", id, study.name);
                    println!("     ❓ {}", study.question);
                    println!("     📋 {} phases, {} rep(s)", study.phase_ids.len(), study.repetitions);
                    println!();
                }
            }
            println!("  Total: {} studies", studies.len());
            std::process::exit(0);
        }

        Some(Commands::Queue { action, model, experiment, days, json }) => {
            let exit_code = cli::run_queue(
                action, model.as_deref(), experiment.as_deref(), *days, *json,
            )?;
            std::process::exit(exit_code);
        }

        Some(Commands::Analyze { model, json }) => {
            let exit_code = cli::run_analyze(model, *json)?;
            std::process::exit(exit_code);
        }

        Some(Commands::Recommend { task, objective, graph, json }) => {
            let exit_code = cli::run_recommend(
                task.as_deref(), objective.as_deref(), graph.as_deref(), *json,
            )?;
            std::process::exit(exit_code);
        }

        Some(Commands::EnvInit { project_root, json }) => {
            let root = project_root.canonicalize()?;
            println!("╔══════════════════════════════════════════╗");
            println!("║     Environment Builder v0.1.0            ║");
            println!("╚══════════════════════════════════════════╝");
            println!();
            println!("🔍 Detecting project: {}", root.display());
            println!();

            let builder = EnvironmentBuilder::new();
            let graph = builder.build(&root);

            let graph_dir = root.join(".athena");
            std::fs::create_dir_all(&graph_dir)?;
            let graph_path = graph_dir.join("graph.json");
            crate::runtime::graph::save_graph(&graph, &graph_path)?;

            if *json {
                println!("{}", serde_json::to_string_pretty(&graph)?);
            } else {
                println!("📊 Engineering Graph Summary");
                println!("{}", "─".repeat(50));
                let counts = graph.count_by_label();
                for (label, count) in &counts {
                    println!("  {:<15}: {}", label, count);
                }
                println!();
                println!("  Nodes: {} | Edges: {}", graph.nodes.len(), graph.edges.len());
                println!();
                println!("  📁 Graph saved to: {}", graph_path.display());
                println!();
                println!("  Use: ath inspect   — to explore the graph");
                println!("  Use: ath run       — to run inference");
            }
            println!();
            println!("✅ Environment ready!");
            std::process::exit(0);
        }

        None => {
            let root = std::env::current_dir()?;
            let schemas_dir = PathBuf::from("schemas");
            let output_dir = PathBuf::from(".knowledge");
            let exit_code = cli::run_build(&root, &output_dir, &schemas_dir, false)?;
            std::process::exit(exit_code);
        }
    }
}
