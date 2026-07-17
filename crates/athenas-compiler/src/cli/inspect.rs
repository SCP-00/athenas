use colored::*;
use std::path::Path;

use crate::runtime::environment::EnvironmentBuilder;
use crate::runtime::graph::eng_node::{EngNodeData, RuntimeStatus};
use crate::runtime::graph::EngineeringGraph;

/// Run `ath inspect` — display the Engineering Graph state
pub fn run_inspect(graph_path: &Path, section: Option<&str>, json_output: bool) -> anyhow::Result<i32> {
    let graph = if graph_path.exists() {
        crate::runtime::graph::load_graph(graph_path)?
    } else {
        // Build graph on the fly if no cached version
        let builder = EnvironmentBuilder::new();
        builder.build(std::env::current_dir().as_deref().unwrap_or(Path::new(".")))
    };

    match section {
        None => display_overview(&graph, json_output),
        Some("models") => display_subgraph(&graph, "Models", |d| matches!(d, EngNodeData::Model { .. })),
        Some("runtime") => display_subgraph(&graph, "Runtimes", |d| matches!(d, EngNodeData::Runtime { .. })),
        Some("knowledge") => display_subgraph(&graph, "Knowledge", |d| matches!(d, EngNodeData::KnowledgePack { .. })),
        Some("hardware") => display_subgraph(&graph, "Hardware", |d| matches!(d, EngNodeData::Cpu { .. } | EngNodeData::Gpu { .. } | EngNodeData::Ram { .. } | EngNodeData::Os { .. })),
        Some("tools") => display_subgraph(&graph, "Tools", |d| matches!(d, EngNodeData::Compiler { .. } | EngNodeData::Debugger { .. } | EngNodeData::Formatter { .. } | EngNodeData::Linter { .. } | EngNodeData::TestRunner { .. })),
        Some("broken") => display_broken_edges(&graph),
        Some(s) => anyhow::bail!("Unknown section: '{s}'. Available: models, runtime, knowledge, hardware, tools, broken"),
    }
}

fn display_overview(graph: &EngineeringGraph, json: bool) -> anyhow::Result<i32> {
    if json {
        println!("{}", serde_json::to_string_pretty(graph)?);
        return Ok(0);
    }

    println!("╔══════════════════════════════════════════╗");
    println!("║     Engineering Graph Inspector          ║");
    println!("╚══════════════════════════════════════════╝");
    println!();

    // Counts by type
    let counts = graph.count_by_label();
    println!("📊 Graph Summary");
    println!("{}", "─".repeat(50));
    println!("  Nodes: {} | Edges: {}", 
        graph.nodes.len().to_string().bright_white(),
        graph.edges.len().to_string().bright_white());
    for (label, count) in &counts {
        println!("  {:<15}: {}", label, count);
    }
    println!();

    // Languages
    let langs = graph.nodes_by_type(|d| matches!(d, EngNodeData::Language { .. }));
    if !langs.is_empty() {
        println!("🔤 Languages");
        println!("{}", "─".repeat(50));
        for lang in langs {
            println!("  {} {} {}", "∙".dimmed(), lang.data.summary(), "✓".bright_green());
        }
        println!();
    }

    // Runtime
    let runtimes = graph.nodes_by_type(|d| matches!(d, EngNodeData::Runtime { .. }));
    if !runtimes.is_empty() {
        println!("🖥️  Runtimes");
        println!("{}", "─".repeat(50));
        for rt in runtimes {
            let (icon, status_str) = match rt.data {
                EngNodeData::Runtime { status: RuntimeStatus::Running, .. } => 
                    ("🟢".green().to_string(), "running".to_string()),
                EngNodeData::Runtime { status: RuntimeStatus::Available, .. } => 
                    ("🟡".yellow().to_string(), "available".to_string()),
                EngNodeData::Runtime { status: RuntimeStatus::NotRunning, .. } => 
                    ("🔴".red().to_string(), "not running".to_string()),
                _ => ("⚪".to_string(), "not found".to_string()),
            };
            println!("  {} {} — {} {}", "∙".dimmed(), rt.data.summary(), icon, status_str);
        }
        println!();
    }

    // Models
    let models = graph.nodes_by_type(|d| matches!(d, EngNodeData::Model { .. }));
    if !models.is_empty() {
        println!("📦 Models");
        println!("{}", "─".repeat(50));
        for m in models {
            let is_ollama = if let EngNodeData::Model { path, .. } = &m.data {
                path.starts_with("ollama://")
            } else { false };
            let runtime = if is_ollama { " (Ollama)" } else { " (GGUF)" };
            println!("  {} {} {}", "∙".dimmed(), m.data.summary(), runtime.dimmed());
        }
        println!();
    }

    // Hardware
    let hw_nodes = graph.nodes_by_type(|d| matches!(d, EngNodeData::Cpu { .. } | EngNodeData::Gpu { .. } | EngNodeData::Ram { .. }));
    if !hw_nodes.is_empty() {
        println!("🖥️  Hardware");
        println!("{}", "─".repeat(50));
        for hw in hw_nodes {
            let icon = match hw.data {
                EngNodeData::Cpu { .. } => "🖥️",
                EngNodeData::Gpu { .. } => "🎮",
                EngNodeData::Ram { .. } => "💾",
                _ => "∙",
            };
            println!("  {} {}", icon, hw.data.summary());
        }
        println!();
    }

    // Knowledge
    let kps = graph.nodes_by_type(|d| matches!(d, EngNodeData::KnowledgePack { .. }));
    if !kps.is_empty() {
        println!("📚 Knowledge");
        println!("{}", "─".repeat(50));
        for kp in kps {
            println!("  {} {}", "∙".dimmed(), kp.data.summary());
        }
        println!();
    }

    println!("✅ Inspect complete. {} nodes, {} edges.", 
        graph.nodes.len().to_string().bright_white(),
        graph.edges.len().to_string().bright_white());

    Ok(0)
}

fn display_subgraph<F>(graph: &EngineeringGraph, title: &str, pred: F) -> anyhow::Result<i32>
where
    F: Fn(&EngNodeData) -> bool,
{
    let nodes: Vec<&crate::runtime::graph::eng_node::EngNode> = graph.nodes_by_type(pred);
    let count = nodes.len();
    println!("╔══════════════════════════════════════════╗");
    println!("║  {:<38}║", format!(" {} — Graph Subset", title));
    println!("╚══════════════════════════════════════════╝");
    println!();
    if nodes.is_empty() {
        println!("  ℹ No {} nodes found.", title);
    } else {
        // Pre-compute edge info to avoid borrowing graph while iterating nodes
        let edge_info: Vec<(String, String)> = graph.edges.iter()
            .filter(|e| nodes.iter().any(|n| n.id == e.source))
            .map(|e| {
                let target_summary = graph.nodes.iter()
                    .find(|n| n.id == e.target)
                    .map(|n| n.data.summary())
                    .unwrap_or_else(|| "?".to_string());
                (e.source.clone(), format!("{:?} → {}", e.kind, target_summary))
            })
            .collect();

        for node in &nodes {
            println!("  📍 {}", node.data.summary());
            for (src, desc) in &edge_info {
                if src == &node.id {
                    println!("     └─ {desc}");
                }
            }
        }
    }
    println!();
    println!("  {} nodes total", count.to_string().bright_white());
    Ok(0)
}

fn display_broken_edges(graph: &EngineeringGraph) -> anyhow::Result<i32> {
    let broken = graph.broken_edges();
    println!("╔══════════════════════════════════════════╗");
    println!("║     Broken Edge Detection                ║");
    println!("╚══════════════════════════════════════════╝");
    println!();
    if broken.is_empty() {
        println!("  ✓ All {} edges resolve correctly.", 
            graph.edges.len().to_string().bright_green());
    } else {
        for edge in &broken {
            let src_exists = graph.nodes.iter().any(|n| n.id == edge.source);
            let tgt_exists = graph.nodes.iter().any(|n| n.id == edge.target);
            println!("  ✖ Broken edge: {:?}", edge.kind);
            if !src_exists {
                println!("     Source '{}' does not exist", edge.source);
            }
            if !tgt_exists {
                println!("     Target '{}' does not exist", edge.target);
            }
        }
        println!();
        println!("  Warning: {} broken edge(s) found.", 
            broken.len().to_string().bright_red());
    }
    Ok(0)
}
