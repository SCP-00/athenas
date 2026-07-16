mod generators;
mod graph;
mod parser;
mod runtime;
mod validator;

use clap::{Parser, Subcommand};
use colored::*;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

use runtime::{InferenceParams, Runtime};

/// Subcommands for `ath pack`
#[derive(Subcommand, Debug)]
enum PackAction {
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
enum WorkspaceAction {
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

    /// Benchmark a model against a capability and generate certification report
    Certify {
        /// Path to the GGUF model file (auto-detect if omitted)
        #[arg(short, long)]
        model: Option<PathBuf>,

        /// Capability to benchmark (default: text-generation)
        #[arg(short, long, default_value = "text-generation")]
        capability: String,

        /// Knowledge pack ID to benchmark WITH (compares raw vs packed)
        #[arg(short = 'P', long)]
        pack: Option<String>,

        /// Maximum tokens to generate
        #[arg(short = 'n', long, default_value = "100")]
        max_tokens: usize,

        /// Path to llama-server binary
        #[arg(long)]
        server_path: Option<PathBuf>,

        /// Output structured JSON instead of human-readable format
        #[arg(short, long)]
        json: bool,
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
}

fn print_banner() {
    println!();
    println!("{}", "╔══════════════════════════════════════════╗".bright_blue());
    println!("{}", "║     Athenas Knowledge Compiler v0.1.0    ║".bright_blue());
    println!("{}", "╚══════════════════════════════════════════╝".bright_blue());
    println!();
}

fn validate_documents(documents: &[crate::parser::Document], schemas: &crate::validator::SchemaMap) -> i32 {
    let mut exit_code = 0;

    // Schema validation
    println!("{} Validating documents...", "✓".bold());
    let validation_errors = crate::validator::validate_all_documents(documents, schemas);
    if validation_errors.is_empty() {
        println!("  {} All documents pass schema validation", "✓".bright_green());
    } else {
        for error in &validation_errors {
            println!("  {} {}", "✖".red(), error);
            exit_code = 1;
        }
    }

    // ID uniqueness check
    let mut id_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for doc in documents {
        *id_counts.entry(doc.id.clone()).or_insert(0) += 1;
    }
    let duplicate_ids: Vec<&String> = id_counts.iter().filter(|&(_, &c)| c > 1).map(|(id, _)| id).collect();
    if !duplicate_ids.is_empty() {
        for id in &duplicate_ids {
            println!("  {} Duplicate ID: {}", "✖".red(), id.bright_red());
            exit_code = 1;
        }
    } else {
        println!("  ✓ All document IDs are unique");
    }

    // Reference integrity (scan ALL relationship fields in ALL documents)
    let known_ids: std::collections::HashSet<String> = documents.iter().map(|d| d.id.clone()).collect();
    let relationship_keys = ["implements", "depends_on", "validated_by", "derived_from", "supersedes", "related", "validates"];
    let mut broken_refs = 0;
    let mut total_refs = 0;
    for doc in documents {
        for key in &relationship_keys {
            if let Some(values) = doc.front_matter.get(*key).and_then(|v| v.as_sequence()) {
                for value in values {
                    if let Some(target) = value.as_str() {
                        total_refs += 1;
                        if !known_ids.contains(target) {
                            println!("  {} Broken reference in {}: '{}' '{}' not found", "✖".red(), doc.id.bright_white(), key, target.bright_red());
                            broken_refs += 1;
                            exit_code = 1;
                        }
                    }
                }
            }
        }
    }
    if broken_refs == 0 {
        println!("  ✓ All {} references resolve correctly", total_refs.to_string().bright_cyan());
    }

    exit_code
}

fn run_validate(root: &Path, schemas_dir: &Path, verbose: bool) -> anyhow::Result<i32> {
    // Load schemas
    println!("{} Loading schemas...", "📋".bold());
    let schemas = validator::load_schemas(schemas_dir)?;
    println!("  Loaded {} document type schemas", schemas.len().to_string().bright_green());

    // Find and parse documents
    println!("{} Scanning project: {}", "🔍".bold(), root.display());
    let documents = parser::parse_all_documents(root)?;

    if documents.is_empty() {
        println!("  {}", "ℹ No documents found".dimmed());
        return Ok(0);
    }

    println!("  Found {} markdown documents with valid front-matter", documents.len().to_string().bright_green());

    if verbose {
        for doc in &documents {
            let doc_type = doc.id.split('-').next().unwrap_or("?");
            let status = doc.front_matter.get("status").and_then(|v| v.as_str()).unwrap_or("?");
            println!("    {} {} — {} ({}, {})", "📄".dimmed(), doc.id.bright_white(), doc.path.dimmed(), doc_type, status);
        }
        println!();
    }

    let mut exit_code = validate_documents(&documents, &schemas);

    // Build graph for diagnostics
    let graph = graph::build_knowledge_graph(&documents);
    let diagnostics = generators::generate_diagnostics(&documents, &graph);
    if !diagnostics.is_empty() {
        println!("  {} Diagnostics:", "ℹ".bold());
        for diag in &diagnostics {
            let level = match diag.severity.as_str() {
                "error" => "✖".red(),
                "warning" => "⚠".yellow(),
                _ => "ℹ".dimmed(),
            };
            println!("    {} [{}] {} — {}", level, diag.id, diag.message, diag.path.dimmed());
            if diag.severity == "error" {
                exit_code = 1;
            }
        }
    }

    // Summary
    println!();
    let status = if exit_code == 0 { "✓ Validation PASSED".bright_green() } else { "✖ Validation FAILED".bright_red() };
    println!("{}", status);
    println!("  Documents: {} | Edges: {} | Errors: {} | Warnings: {}",
        graph.metadata.total_nodes.to_string().bright_white(),
        graph.metadata.total_edges.to_string().bright_cyan(),
        diagnostics.iter().filter(|d| d.severity == "error").count().to_string().bright_red(),
        diagnostics.iter().filter(|d| d.severity == "warning").count().to_string().yellow(),
    );

    Ok(exit_code)
}

fn run_graph(root: &Path, output_dir: &Path, verbose: bool) -> anyhow::Result<i32> {
    println!("{} Scanning project: {}", "🔍".bold(), root.display());
    let documents = parser::parse_all_documents(root)?;

    if documents.is_empty() {
        println!("  {}", "ℹ No documents found".dimmed());
        return Ok(0);
    }

    println!("  Found {} documents", documents.len().to_string().bright_green());

    if verbose {
        for doc in &documents {
            println!("    {} {} — {}", "📄".dimmed(), doc.id.bright_white(), doc.path.dimmed());
        }
        println!();
    }

    println!("{} Building knowledge graph...", "⚙".bold());
    let graph = graph::build_knowledge_graph(&documents);

    println!("  Nodes: {} | Edges: {} | Types: {}",
        graph.metadata.total_nodes.to_string().bright_green(),
        graph.metadata.total_edges.to_string().bright_cyan(),
        graph.metadata.doc_types.len().to_string().bright_yellow(),
    );

    // Output only graph.json — focused, per Chatty's spec
    std::fs::create_dir_all(output_dir)?;
    let graph_json = serde_json::to_string_pretty(&graph)?;
    let output_path = output_dir.join("graph.json");
    std::fs::write(&output_path, graph_json)?;
    println!("  ✓ Graph written to {}", output_path.display().to_string().bright_white());

    println!();
    println!("{}", "✓ Graph complete!".bright_green().bold());

    Ok(0)
}

fn run_build(root: &Path, output_dir: &Path, schemas_dir: &Path, _verbose: bool) -> anyhow::Result<i32> {
    println!("{} Phase 1: Load schemas", "1️⃣".bold());
    let schemas = validator::load_schemas(schemas_dir)?;
    println!("  Loaded {} document type schemas", schemas.len().to_string().bright_green());

    // Parse once, reuse everywhere
    println!("{} Phase 2: Parse documents", "2️⃣".bold());
    let documents = parser::parse_all_documents(root)?;
    println!("  Found {} documents", documents.len().to_string().bright_green());

    // Validate
    println!("{} Phase 3: Validate", "3️⃣".bold());
    let exit_code = validate_documents(&documents, &schemas);
    if exit_code != 0 {
        println!("  {} Build aborted — validation errors found", "⚠".yellow());
        return Ok(exit_code);
    }

    // Compile knowledge graph
    println!();
    println!("{} Phase 4: Compile knowledge graph", "4️⃣".bold());
    let output = generators::compile_all(&documents, root);
    println!("  Nodes: {} | Edges: {} | Types: {} | Diagnostics: {}",
        output.graph.metadata.total_nodes.to_string().bright_green(),
        output.graph.metadata.total_edges.to_string().bright_cyan(),
        output.graph.metadata.doc_types.len().to_string().bright_yellow(),
        output.diagnostics.len().to_string().bright_red(),
    );

    // Generate artifacts
    println!();
    println!("{} Phase 5: Generate artifacts", "5️⃣".bold());
    generators::write_output(&output, output_dir)?;

    println!();
    println!("{}", "✓ Build complete!".bright_green().bold());
    println!("  Output: {}", output_dir.display().to_string().bright_white());

    Ok(0)
}

fn run_pack(action: &PackAction, packs_dir: Option<&Path>, json_output: bool) -> anyhow::Result<i32> {
    let dir = packs_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(runtime::knowledge::default_packs_dir);

    let packs = runtime::knowledge::discover_packs(&dir);

    match action {
        PackAction::List => {
            if json_output {
                println!("{}", serde_json::to_string_pretty(&packs)?);
            } else {
                println!("╔══════════════════════════════════════════╗");
                println!("║        Athenas Knowledge Packs           ║");
                println!("╚══════════════════════════════════════════╝");
                println!();
                if packs.is_empty() {
                    println!("  No knowledge packs found in: {}", dir.display());
                    println!("  Create YAML files in knowledge/packs/ to add packs.");
                } else {
                    for pack in &packs {
                        let tools_count = pack.tools.len();
                        let knowledge_count = pack.knowledge.len();
                        println!("  📦 {} (v{})", pack.name, pack.version);
                        println!("     ID: {}", pack.id);
                        println!("     {}", pack.description);
                        println!("     Languages: {} | Tools: {} | Knowledge items: {}",
                            pack.languages.join(", "), tools_count, knowledge_count);
                        if !pack.depends_on.is_empty() {
                            println!("     Depends on: {}", pack.depends_on.join(", "));
                        }
                        println!();
                    }
                }
            }
        }
        PackAction::Show { id } => {
            let pack = runtime::knowledge::find_pack(&packs, id)
                .ok_or_else(|| anyhow::anyhow!("Pack '{id}' not found"))?;

            let tools = runtime::knowledge::check_tools(pack);
            let installed_count = tools.iter().filter(|t| t.installed).count();
            let total_tools = tools.len();

            if json_output {
                let output = serde_json::json!({
                    "pack": pack,
                    "tools_status": tools,
                    "installed": installed_count,
                    "total_tools": total_tools,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("╔══════════════════════════════════════════╗");
                println!("║        Pack Details                       ║");
                println!("╚══════════════════════════════════════════╝");
                println!();
                println!("  📦 {} (v{})", pack.name, pack.version);
                println!("     ID: {}", pack.id);
                println!("     {}", pack.description);
                println!("     Languages: {}", pack.languages.join(", "));
                if !pack.depends_on.is_empty() {
                    println!("     Depends on: {}", pack.depends_on.join(", "));
                }
                println!();

                // Tools
                println!("  🔧 Tools ({installed_count}/{total_tools} installed):");
                for tool in &tools {
                    let status = if tool.installed {
                        format!("✓ {}", tool.version.as_deref().unwrap_or("installed"))
                    } else {
                        "✗ not found".to_string()
                    };
                    println!("     {} — {} ({status})", tool.name, tool.description);
                }
                println!();

                // Knowledge items
                println!("  📚 Knowledge items:");
                for item in &pack.knowledge {
                    println!("     - {}", item.title);
                }
                println!();

                // Benchmarks
                if !pack.benchmarks.is_empty() {
                    println!("  🏋️  Benchmarks:");
                    for b in &pack.benchmarks {
                        println!("     - {} ({})", b.description, b.max_tokens);
                    }
                    println!();
                }

                // System prompt preview
                if let Some(system) = pack.prompts.get("system") {
                    let preview: String = system.chars().take(200).collect();
                    println!("  🧠 System prompt (preview):");
                    println!("     {preview}...");
                    println!();
                }
            }
        }
    }

    Ok(0)
}

fn run_workspace(action: &WorkspaceAction) -> anyhow::Result<i32> {
    match action {
        WorkspaceAction::List => {
            let packs_dir = runtime::knowledge::default_packs_dir();
            let packs = runtime::knowledge::discover_packs(&packs_dir);
            println!("╔══════════════════════════════════════════╗");
            println!("║      Athenas Workspaces                   ║");
            println!("╚══════════════════════════════════════════╝");
            println!();
            println!("  Available workspaces (from knowledge packs):");
            println!();
            for pack in &packs {
                println!("  🏗️  {} ({})", pack.name, pack.id);
                println!("     $ ath workspace create {}", pack.id);
                println!();
            }
            println!("  Create a workspace: ath workspace create <pack-id>");
        }
        WorkspaceAction::Create { pack_id, output } => {
            let packs_dir = runtime::knowledge::default_packs_dir();
            let packs = runtime::knowledge::discover_packs(&packs_dir);
            let pack = runtime::knowledge::find_pack(&packs, pack_id)
                .ok_or_else(|| anyhow::anyhow!("Pack '{pack_id}' not found"))?;

            let workspace_dir = output.join(format!("workspace-{}", pack_id));
            std::fs::create_dir_all(&workspace_dir)?;

            // Generate workspace config
            let tools = runtime::knowledge::check_tools(pack);
            let config = serde_json::json!({
                "workspace": {
                    "name": format!("{} Workspace", pack.name),
                    "pack": pack.id,
                    "pack_version": pack.version,
                    "languages": pack.languages,
                },
                "tools": tools.iter().map(|t| serde_json::json!({
                    "name": t.name,
                    "command": t.command,
                    "installed": t.installed,
                    "version": t.version,
                })).collect::<Vec<_>>(),
                "model_config": {
                    "temperature": 0.7,
                    "max_tokens": 2048,
                },
                "system_prompt": runtime::knowledge::build_system_prompt(pack, None),
            });

            let config_path = workspace_dir.join("athenas.json");
            std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;

            println!("╔══════════════════════════════════════════╗");
            println!("║        Workspace Created                  ║");
            println!("╚══════════════════════════════════════════╝");
            println!();
            println!("  🏗️  Workspace: {}", workspace_dir.display());
            println!("  📦 Pack: {} (v{})", pack.name, pack.version);
            println!("  🔧 Tools available: {}", tools.iter().filter(|t| t.installed).count());
            println!();
            println!("  Config: {}", config_path.display());
            println!("  To use: cd {} && ath run", workspace_dir.display());
            println!();
            println!("  The workspace contains the full system prompt with");
            println!("  domain knowledge for {}. Edit athenas.json to customize.", pack.languages.join(", "));
        }
    }
    Ok(0)
}

fn run_knowledge_build(provider_name: &str, query: &str, output_dir: &Path, json_output: bool) -> anyhow::Result<i32> {
    use runtime::knowledge_ir;

    println!("╔══════════════════════════════════════════╗");
    println!("║     Athenas Knowledge Builder             ║");
    println!("╚══════════════════════════════════════════╝");
    println!();
    println!("  Provider: {provider_name}");
    println!("  Query:    {query}");
    println!();

    match provider_name {
        "man" => {
            let provider = runtime::providers::man_provider::ManProvider::new();
            let (ir, pack) = runtime::knowledge::build_knowledge(&provider, query)
                .map_err(|e| anyhow::anyhow!("Build failed: {e}"))?;

            // Write IR as JSON
            std::fs::create_dir_all(output_dir)?;
            let ir_path = output_dir.join(format!("{}-ir.json", query.replace(' ', "-")));
            std::fs::write(&ir_path, serde_json::to_string_pretty(&ir)?)?;

            // Write pack as YAML
            let pack_path = output_dir.join(format!("{}.yaml", query.replace(' ', "-")));
            std::fs::write(&pack_path, serde_yaml::to_string(&pack)?)?;

            if json_output {
                let report = serde_json::json!({
                    "provider": provider_name,
                    "query": query,
                    "ir": ir,
                    "pack": pack,
                });
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!();
                println!("📊 Build Report");
                println!("{}", "─".repeat(50));
                println!("  Raw bytes:       {}", ir.metrics.raw_bytes);
                println!("  Items extracted: {}", ir.metrics.items_extracted);
                println!("  Dedup removed:   {}", ir.metrics.dedup_removed);
                println!("  Validation:      {}", if ir.validation.valid { "✓ PASS" } else { "✖ FAIL" });
                println!("  Compile time:    {:.0} ms", ir.metrics.compile_time_ms);
                println!();
                println!("  Knowledge items by type:");
                let mut kind_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                for item in &ir.items {
                    *kind_counts.entry(item.kind.to_string()).or_insert(0) += 1;
                }
                let mut sorted: Vec<_> = kind_counts.into_iter().collect();
                sorted.sort_by(|a, b| b.1.cmp(&a.1));
                for (kind, count) in &sorted {
                    println!("    {kind:20}: {count}");
                }
                println!();
                println!("  📁 IR:     {}", ir_path.display());
                println!("  📦 Pack:   {}", pack_path.display());
            }

            Ok(0)
        }
        _ => {
            anyhow::bail!(
                "Unknown provider '{provider_name}'. Available providers: man"
            );
        }
    }
}

fn run_doctor(json_output: bool) -> anyhow::Result<i32> {
    use runtime::hardware;
    use runtime::Capability;

    println!("╔══════════════════════════════════════════╗");
    println!("║        Athenas System Doctor             ║");
    println!("╚══════════════════════════════════════════╝");
    println!();

    // Hardware detection
    println!("{} Detecting hardware...", "🔍".bold());
    let hw = hardware::detect_hardware();

    // Model discovery
    println!("{} Discovering models...", "📦".bold());
    let models = runtime::find_all_models();

    if json_output {
        let output = serde_json::json!({
            "hardware": hw,
            "models": models,
            "capabilities": Capability::all().iter().map(|c| c.name()).collect::<Vec<_>>(),
            "platform": std::env::consts::ARCH,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        // CPU
        println!("  🖥  CPU: {} ({} cores, {} threads)", hw.cpu.model, hw.cpu.cores, hw.cpu.threads);

        // GPU
        for gpu in &hw.gpu {
            println!("  🎮 GPU: {} ({} GB VRAM, driver {})", gpu.model, gpu.vram_gb, gpu.driver_version);
        }

        // Memory
        println!("  💾 RAM: {:.0} GB total ({:.0} GB available)", hw.memory.total_gb, hw.memory.available_gb);

        // OS
        println!("  💻 OS: {} {} ({})", hw.os.name, hw.os.version, hw.os.arch);
        println!("  🐧 Kernel: {}", hw.kernel);
        println!();

        // Models
        println!("📦 Models discovered:");
        if models.is_empty() {
            println!("  No GGUF models found.");
        } else {
            for m in &models {
                println!("  {} ({:.0}B, {})", m.id, m.parameters_b, m.quantization);
                println!("     Path: {}", m.path);
            }
        }
        println!();

        // Capabilities
        println!("🎯 Available capabilities:");
        for c in Capability::all() {
            println!("  - {}", c.name());
        }
        println!();
    }

    Ok(0)
}

fn run_benchmark(rt: &impl Runtime, prompt: &str, max_tokens: usize) -> anyhow::Result<runtime::InferenceResult> {
    let params = InferenceParams {
        max_tokens,
        ..Default::default()
    };
    rt.complete(prompt, &params)
}

fn run_certify(
    model_path: Option<&Path>,
    capability_name: &str,
    pack_id: Option<&str>,
    max_tokens: usize,
    server_path: Option<&Path>,
    json_output: bool,
) -> anyhow::Result<i32> {
    use runtime::hardware;
    use runtime::Capability;
    use runtime::knowledge;

    // Resolve capability
    let capability = Capability::from_name(capability_name)
        .ok_or_else(|| anyhow::anyhow!(
            "Unknown capability: '{capability_name}'. Available: {:?}",
            Capability::all().iter().map(|c| c.name()).collect::<Vec<_>>()
        ))?;

    // Resolve model
    let model = runtime::find_model(model_path)?;
    let hw = hardware::detect_hardware();
    let info = runtime::infer_model_info(&model, Some(&hw));

    let prompt = match capability {
        Capability::TextGeneration => "Write a concise summary of what an AI engineering platform is in 3 sentences.",
        Capability::Coding => "Write a Python function that implements a binary search tree with insert and search methods.",
        Capability::ToolCalling => "Given the functions: get_weather(city: str) and send_email(to: str, body: str), respond with a JSON function call to check the weather in Tokyo.",
        Capability::Translation => "Translate this to Spanish: 'The quick brown fox jumps over the lazy dog.'",
        Capability::Reasoning => "If a bat and a ball cost $1.10 in total, and the bat costs $1.00 more than the ball, how much does the ball cost? Explain step by step.",
        Capability::InstructionFollowing => "Reply ONLY with the word 'compliant'. Do not add any other text.",
        Capability::RAG => "Based on the context: 'Athenas is an engineering platform for local AI. It compiles knowledge into structured artifacts.' Answer: What does Athenas compile?",
        Capability::LongContext => "Repeat the following sentence: The certification process validates model capabilities and generates structured evidence for engineering decisions. The certification process validates model capabilities and generates structured evidence for engineering decisions."
    };

    // Load knowledge pack if specified
    let system_prompt = pack_id.and_then(|pid| {
        let dir = knowledge::default_packs_dir();
        let packs = knowledge::discover_packs(&dir);
        knowledge::find_pack(&packs, pid).map(|p| knowledge::build_system_prompt(p, None))
    });
    let packed_prompt = match &system_prompt {
        Some(sp) => format!("{}\n\n### Task\n\n{prompt}", sp),
        None => prompt.to_string(),
    };

    if !json_output {
        println!("╔══════════════════════════════════════════╗");
        println!("║     Athenas Certification v0.1.0         ║");
        println!("╚══════════════════════════════════════════╝");
        println!();
        println!("🎯 Capability: {}", capability.name());
        println!("📋 Model: {}", info.path);
        println!("📏 Parameters: {:.0}B | Quant: {}", info.parameters_b, info.quantization);
        if let Some(pid) = pack_id {
            println!("📦 Pack: {pid}");
        }
        println!();
    }

    // Build and start runtime
    let mut rt_builder = runtime::LlamaServerRuntime::new();
    if let Some(sp) = server_path {
        rt_builder = rt_builder.with_server_path(sp.to_path_buf());
    }
    let mut rt = rt_builder;

    let load_start = Instant::now();
    rt.load_model(&model)?;
    if !json_output {
        println!("⏱  Model loaded in {:.1}s", load_start.elapsed().as_secs_f64());
        println!();
    }

    if let Some(pid) = pack_id {
        // Pack mode: benchmark raw first, then packed
        println!("🧪 PHASE 1: Raw (without knowledge pack)");
        let raw_result = run_benchmark(&rt, prompt, max_tokens)?;

        println!("🧪 PHASE 2: Packed (with knowledge pack '{pid}')");
        // Reset context by unloading and reloading
        rt.unload()?;
        rt.load_model(&model)?;
        // The first borrow was released after raw_result, so rt is free to borrow mutably
        let packed_result = run_benchmark(&rt, &packed_prompt, max_tokens)?;
        rt.unload()?;

        // Calculate improvement
        let ttft_improvement = if raw_result.ttft_ms > 0.0 {
            ((raw_result.ttft_ms - packed_result.ttft_ms) / raw_result.ttft_ms * 100.0).round()
        } else {
            0.0
        };

        if json_output {
            let output = serde_json::json!({
                "model": info,
                "hardware": hw,
                "capability": capability.name(),
                "pack": pid,
                "raw": {
                    "ttft_ms": raw_result.ttft_ms,
                    "tokens_per_second": raw_result.tokens_per_second,
                    "total_tokens": raw_result.total_tokens,
                },
                "packed": {
                    "ttft_ms": packed_result.ttft_ms,
                    "tokens_per_second": packed_result.tokens_per_second,
                    "total_tokens": packed_result.total_tokens,
                },
                "improvement": {
                    "ttft_ms_pct": ttft_improvement,
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!();
            println!("📊 COMPARISON: Raw vs Packed");
            println!("{}", "─".repeat(60));
            println!("  Metric               Raw           Packed        Δ");
            println!("{}", "─".repeat(60));
            println!("  TTFT (ms)           {:8.1}      {:8.1}      {:>+6}%",
                raw_result.ttft_ms, packed_result.ttft_ms, ttft_improvement);
            println!("  Tokens/sec          {:8.1}      {:8.1}",
                raw_result.tokens_per_second, packed_result.tokens_per_second);
            println!("  Generated tokens    {:8}       {:8}",
                raw_result.total_tokens, packed_result.total_tokens);
            println!("{}", "─".repeat(60));
            println!();
            println!("📝 Raw response:");
            println!("{}", "─".repeat(60));
            println!("{}", raw_result.text);
            println!("{}", "─".repeat(60));
            println!();
            println!("📝 Packed response:");
            println!("{}", "─".repeat(60));
            println!("{}", packed_result.text);
            println!("{}", "─".repeat(60));
            println!();
            println!("{} Certification with pack complete!", "✓".bright_green());
        }
    } else {
        // Standard mode: single benchmark
        if !json_output {
            println!("⚡ Benchmarking...");
            println!();
        }
        let result = run_benchmark(&rt, prompt, max_tokens)?;
        rt.unload()?;

        let execution = runtime::ExecutionResult {
            inference: result,
            hardware: hw,
            capability,
            model_path: model.to_string_lossy().to_string(),
            model_info: info,
            warnings: Vec::new(),
            evidence_ref: None,
        };

        if json_output {
            println!("{}", serde_json::to_string_pretty(&execution)?);
        } else {
            println!("📊 Certification Results:");
            println!("  🎯 Capability:  {}", execution.capability.name());
            println!("  ⏱  TTFT:       {:.1} ms", execution.inference.ttft_ms);
            println!("  🚀 Throughput:  {:.1} tok/s", execution.inference.tokens_per_second);
            println!("  📊 Generated:   {} tokens", execution.inference.total_tokens);
            println!("  💾 Hardware:    {} | {} GB RAM", execution.hardware.cpu.model, execution.hardware.memory.total_gb);
            if let Some(gpu) = execution.hardware.gpu.first() {
                println!("  🎮 GPU:         {}", gpu.model);
            }
            println!();
            println!("📝 Response:");
            println!("{}", "─".repeat(60));
            println!("{}", execution.inference.text);
            println!("{}", "─".repeat(60));
            println!();
            println!("{} Certification complete!", "✓".bright_green());
        }
    }

    Ok(0)
}

fn run_inference(
    model_path: Option<&Path>,
    prompt: Option<&str>,
    workspace: Option<&Path>,
    max_tokens: usize,
    temperature: f64,
    json_output: bool,
    server_path: Option<&Path>,
    _verbose: bool,
) -> anyhow::Result<i32> {

    // Resolve model
    let model = runtime::find_model(model_path)?;
    let info = runtime::infer_model_info(&model, None);

    // Check for workspace config first
    let workspace_prompt = workspace.and_then(|w| {
        let config_path = w.join("athenas.json");
        if config_path.exists() {
            std::fs::read_to_string(&config_path).ok().and_then(|content| {
                serde_json::from_str::<serde_json::Value>(&content).ok()
                    .and_then(|v| v["system_prompt"].as_str().map(|s| s.to_string()))
            })
        } else {
            None
        }
    });

    // Read prompt: workspace > cli arg > stdin
    let prompt_text = match (workspace_prompt, prompt) {
        (Some(wp), None) => {
            if !json_output {
                println!("  📖 Using system prompt from workspace ({} chars)", wp.len());
            }
            wp
        }
        (Some(wp), Some(user_prompt)) => {
            // Combine workspace system prompt with user prompt
            format!("{}\n\n### User Request\n\n{user_prompt}", wp)
        }
        (None, Some(p)) => p.to_string(),
        (None, None) => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    if prompt_text.trim().is_empty() {
        anyhow::bail!("No prompt provided. Use --prompt or pipe input via stdin.");
    }

    // Build runtime with optional custom server path
    let mut rt_builder = runtime::LlamaServerRuntime::new();
    if let Some(sp) = server_path {
        rt_builder = rt_builder.with_server_path(sp.to_path_buf());
    }

    if !json_output {
        println!("╔══════════════════════════════════════════╗");
        println!("║     Athenas Runtime v0.1.0 — Spike       ║");
        println!("╚══════════════════════════════════════════╝");
        println!();
        println!("📋 Model: {}", info.path);
        println!("📏 Parameters: {:.0}B | Quant: {} | Context: {}",
            info.parameters_b, info.quantization, info.context_length);
        println!("💬 Prompt: {} chars", prompt_text.len());
        println!("🎯 Max tokens: {} | Temperature: {}", max_tokens, temperature);
        println!();
    }

    // Start runtime
    let mut rt = rt_builder;
    let load_start = Instant::now();
    rt.load_model(&model)?;
    let load_time = load_start.elapsed();

    if !json_output {
        println!();
        println!("⏱  Model loaded in {:.1}s", load_time.as_secs_f64());
        println!();
        println!("⚡ Generating...");
    }

    // Run inference
    let params = InferenceParams {
        max_tokens,
        temperature,
        ..Default::default()
    };

    let result = rt.complete(&prompt_text, &params)?;
    rt.unload()?;

    if json_output {
        // Pure JSON output — machine-readable
        let output = serde_json::json!({
            "runtime": rt.name(),
            "model": info,
            "prompt": {
                "text": prompt_text,
                "chars": prompt_text.len()
            },
            "inference": {
                "text": result.text,
                "performance": {
                    "ttft_ms": result.ttft_ms,
                    "tokens_per_second": result.tokens_per_second,
                    "total_tokens": result.total_tokens,
                    "prompt_tokens": result.prompt_tokens,
                    "total_duration_ms": result.total_duration_ms
                }
            }
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!();
        println!("📝 Response:");
        println!("{}", "─".repeat(60));
        println!("{}", result.text);
        println!("{}", "─".repeat(60));
        println!();

        println!("📊 Performance:");
        println!("  ⏱  TTFT (Time to First Token):  {:.1} ms", result.ttft_ms);
        println!("  🚀 Tokens/sec:                  {:.1}", result.tokens_per_second);
        println!("  📊 Total tokens generated:      {}", result.total_tokens);
        println!("  📊 Prompt tokens processed:     {}", result.prompt_tokens);
        println!("  ⏱  Total duration:              {:.1} ms", result.total_duration_ms);
        println!();
        println!("{} Spike complete!", "✓".bright_green());
    }

    Ok(0)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    print_banner();

    match &cli.command {
        Some(Commands::Run {
            model,
            prompt,
            workspace,
            max_tokens,
            temperature,
            json,
            server_path,
            verbose,
        }) => {
            let exit_code = run_inference(
                model.as_deref(),
                prompt.as_deref(),
                workspace.as_deref(),
                *max_tokens,
                *temperature,
                *json,
                server_path.as_deref(),
                *verbose,
            )?;
            std::process::exit(exit_code);
        }

        Some(Commands::Validate { project_root, schemas, verbose }) => {
            let root = project_root.canonicalize()?;
            let exit_code = run_validate(&root, schemas, *verbose)?;
            std::process::exit(exit_code);
        }

        Some(Commands::Graph { project_root, output, verbose }) => {
            let root = project_root.canonicalize()?;
            let exit_code = run_graph(&root, output, *verbose)?;
            std::process::exit(exit_code);
        }

        Some(Commands::Doctor { json }) => {
            let exit_code = run_doctor(*json)?;
            std::process::exit(exit_code);
        }

        Some(Commands::Pack {
            action,
            packs_dir,
            json,
        }) => {
            let exit_code = run_pack(action, packs_dir.as_deref(), *json)?;
            std::process::exit(exit_code);
        }

        Some(Commands::Workspace { action }) => {
            let exit_code = run_workspace(action)?;
            std::process::exit(exit_code);
        }

        Some(Commands::Knowledge {
            provider,
            query,
            output,
            json,
        }) => {
            let exit_code = run_knowledge_build(provider, query, output, *json)?;
            std::process::exit(exit_code);
        }

        Some(Commands::Certify {
            model,
            capability,
            pack,
            max_tokens,
            server_path,
            json,
        }) => {
            let exit_code = run_certify(
                model.as_deref(),
                capability,
                pack.as_deref(),
                *max_tokens,
                server_path.as_deref(),
                *json,
            )?;
            std::process::exit(exit_code);
        }

        Some(Commands::Build { project_root, output, schemas, verbose }) => {
            let root = project_root.canonicalize()?;
            let exit_code = run_build(&root, output, schemas, *verbose)?;
            std::process::exit(exit_code);
        }

        None => {
            // Default: run full pipeline (backwards compatible)
            let root = std::env::current_dir()?;
            let schemas_dir = PathBuf::from("schemas");
            let output_dir = PathBuf::from(".knowledge");
            let exit_code = run_build(&root, &output_dir, &schemas_dir, false)?;
            std::process::exit(exit_code);
        }
    }
}
