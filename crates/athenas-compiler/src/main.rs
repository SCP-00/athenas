mod generators;
mod graph;
mod parser;
mod validator;

use clap::Parser;
use colored::*;
use std::path::PathBuf;

/// Athenas Knowledge Compiler — compiles engineering documentation into structured knowledge artifacts
#[derive(Parser, Debug)]
#[command(
    name = "athenas",
    version = "0.1.0",
    about = "Engineering Compiler for Athenas"
)]
struct Cli {
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
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    println!();
    println!(
        "{}",
        "╔══════════════════════════════════════════╗".bright_blue()
    );
    println!(
        "{}",
        "║     Athenas Knowledge Compiler v0.1.0    ║".bright_blue()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════╝".bright_blue()
    );
    println!();

    let root = cli.project_root.canonicalize()?;
    let schemas_dir = cli.schemas;

    // Phase 0: Load JSON schemas
    println!("{} Loading schemas...", "📋".bold());
    let schemas = validator::load_schemas(&schemas_dir)?;
    println!(
        "  Loaded {} document type schemas",
        schemas.len().to_string().bright_green()
    );

    // Phase 1: Find and parse documents
    println!("{} Scanning project: {}", "🔍".bold(), root.display());

    let documents = parser::parse_all_documents(&root)?;

    println!(
        "  Found {} markdown documents with valid front-matter",
        documents.len().to_string().bright_green()
    );

    if documents.is_empty() {
        println!(
            "  {}",
            "⚠ No documents found — check project_root path".yellow()
        );
        return Ok(());
    }

    // Show what was found
    if cli.verbose {
        for doc in &documents {
            let doc_type = doc.id.split('-').next().unwrap_or("?");
            let status = doc
                .front_matter
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            println!(
                "    {} {} — {} ({}, {})",
                "📄".dimmed(),
                doc.id.bright_white(),
                doc.path.dimmed(),
                doc_type,
                status
            );
        }
        println!();
    }

    // Phase 1.5: Validate against schemas
    println!("{} Validating documents...", "✓".bold());
    let validation_errors = validator::validate_all_documents(&documents, &schemas);
    if validation_errors.is_empty() {
        println!(
            "  {} All documents pass schema validation",
            "✓".bright_green()
        );
    } else {
        for error in &validation_errors {
            println!("  {} {}", "✖".red(), error);
        }
    }

    // Phase 2: Compile knowledge graph
    println!("{} Compiling knowledge graph...", "⚙".bold());
    let output = generators::compile_all(&documents, &root);

    println!(
        "  Nodes: {} | Edges: {} | Types: {} | Diagnostics: {}",
        output.graph.metadata.total_nodes.to_string().bright_green(),
        output.graph.metadata.total_edges.to_string().bright_cyan(),
        output
            .graph
            .metadata
            .doc_types
            .len()
            .to_string()
            .bright_yellow(),
        output.diagnostics.len().to_string().bright_red()
    );

    // Show diagnostics
    if !output.diagnostics.is_empty() {
        println!();
        for diag in &output.diagnostics {
            let level = match diag.severity.as_str() {
                "error" => "✖".red(),
                "warning" => "⚠".yellow(),
                _ => "ℹ".dimmed(),
            };
            println!(
                "  {} [{}] {} — {}",
                level,
                diag.id,
                diag.message,
                diag.path.dimmed()
            );
        }
        println!();
    }

    // Show ontology status
    if output.ontology != serde_json::Value::Null {
        let terms = output.ontology.as_object().map(|o| o.len()).unwrap_or(0);
        println!(
            "  Ontology: {} terms defined",
            terms.to_string().bright_magenta()
        );
    } else {
        println!("  {} No ontology.yaml found", "ℹ".dimmed());
    }

    // Phase 3: Write output
    println!("{} Generating artifacts...", "📦".bold());
    generators::write_output(&output, &cli.output)?;

    println!();
    println!("{}", "✓ Compilation complete!".bright_green().bold());
    println!(
        "  Output: {}",
        cli.output.display().to_string().bright_white()
    );
    println!();

    Ok(())
}
