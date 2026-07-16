mod generators;
mod graph;
mod parser;
mod validator;

use clap::{Parser, Subcommand};
use colored::*;
use std::path::{Path, PathBuf};

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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    print_banner();

    match &cli.command {
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
