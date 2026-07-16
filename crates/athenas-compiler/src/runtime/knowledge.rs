use crate::runtime::knowledge_ir::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// A versioned, composable knowledge package for a language/tool/domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePack {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    pub depends_on: Vec<String>,
    pub languages: Vec<String>,
    pub tools: Vec<ToolDef>,
    pub prompts: HashMap<String, String>,
    pub knowledge: Vec<KnowledgeItem>,
    pub benchmarks: Vec<PackBenchmark>,
}

/// Tool definition within a knowledge pack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub command: String,
    pub description: String,
    pub check: Option<String>,
    #[serde(default)]
    pub installed: bool,
    #[serde(default)]
    pub version: Option<String>,
}

/// A single piece of knowledge (syntax, patterns, APIs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItem {
    pub title: String,
    pub content: String,
}

/// Benchmark prompt specific to this pack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackBenchmark {
    pub description: String,
    pub prompt: String,
    pub expected_output: String,
    pub max_tokens: usize,
}

/// Result of comparing model performance with and without a pack
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackBenchmarkResult {
    pub pack_id: String,
    pub raw_score: f64,
    pub packed_score: f64,
    pub improvement_pct: f64,
    pub raw_ttft_ms: f64,
    pub packed_ttft_ms: f64,
    pub raw_tokens_per_second: f64,
    pub packed_tokens_per_second: f64,
    pub model_path: String,
    pub hardware: crate::runtime::hardware::HardwareInfo,
    pub timestamp: String,
}

/// Discover all available knowledge packs
pub fn discover_packs(packs_dir: &Path) -> Vec<KnowledgePack> {
    let mut packs = Vec::new();
    if !packs_dir.exists() || !packs_dir.is_dir() {
        return packs;
    }

    if let Ok(entries) = std::fs::read_dir(packs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
                match load_pack(&path) {
                    Ok(pack) => packs.push(pack),
                    Err(e) => {
                        eprintln!("  ⚠ Failed to load pack {}: {e}", path.display());
                    }
                }
            }
        }
    }

    // Sort by name for consistent display
    packs.sort_by(|a, b| a.name.cmp(&b.name));
    packs
}

/// Load a single knowledge pack from a YAML file
pub fn load_pack(path: &Path) -> anyhow::Result<KnowledgePack> {
    let content = std::fs::read_to_string(path)?;
    let pack: KnowledgePack = serde_yaml::from_str(&content)?;
    Ok(pack)
}

/// Find a pack by ID
pub fn find_pack<'a>(packs: &'a [KnowledgePack], id: &str) -> Option<&'a KnowledgePack> {
    packs.iter().find(|p| p.id == id)
}

/// Check which tools in a pack are installed on the system
pub fn check_tools(pack: &KnowledgePack) -> Vec<ToolDef> {
    pack.tools
        .iter()
        .map(|tool| {
            let (installed, version) = if let Some(check_cmd) = &tool.check {
                check_tool_installed(check_cmd)
            } else {
                (false, None)
            };
            ToolDef {
                installed,
                version,
                ..tool.clone()
            }
        })
        .collect()
}

fn check_tool_installed(check: &str) -> (bool, Option<String>) {
    let parts: Vec<&str> = check.split_whitespace().collect();
    if parts.is_empty() {
        return (false, None);
    }
    let cmd = parts[0];
    let args = &parts[1..];

    match std::process::Command::new(cmd).args(args).output() {
        Ok(output) => {
            let version = if output.status.success() {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .map(|l| l.trim().to_string())
            } else {
                None
            };
            (true, version)
        }
        Err(_) => (false, None),
    }
}

/// Build the full system prompt for a pack
pub fn build_system_prompt(pack: &KnowledgePack, custom_instructions: Option<&str>) -> String {
    let mut lines = Vec::new();

    if let Some(system) = pack.prompts.get("system") {
        lines.push(system.clone());
    }

    if let Some(analysis) = pack.prompts.get("error_analysis") {
        lines.push("\n## Error Analysis Protocol".to_string());
        lines.push(analysis.clone());
    }

    if let Some(constraints) = pack.prompts.get("constraints") {
        lines.push("\n## Constraints".to_string());
        lines.push(constraints.clone());
    }

    // Add available tools
    if !pack.tools.is_empty() {
        lines.push("\n## Available Tools".to_string());
        for tool in &pack.tools {
            let check = if tool.installed {
                "✓".to_string()
            } else {
                "✗ (not detected)".to_string()
            };
            lines.push(format!("- {}: {} {check}", tool.name, tool.description));
        }
    }

    // Add knowledge items as context
    if !pack.knowledge.is_empty() {
        lines.push("\n## Domain Knowledge".to_string());
        for item in &pack.knowledge {
            lines.push(format!("\n### {}", item.title));
            lines.push(item.content.clone());
        }
    }

    if let Some(extra) = custom_instructions {
        lines.push("\n## Custom Instructions".to_string());
        lines.push(extra.to_string());
    }

    lines.join("\n")
}

// ---------------------------------------------------------------------------
// KnowledgeProvider trait — each provider implements this
// Notice: NO compile() method. Compilation belongs to the compiler.
// ---------------------------------------------------------------------------

pub trait KnowledgeProvider {
    fn id(&self) -> &str;
    fn discover(&self, query: &str) -> Result<Vec<KnowledgeSource>, String>;
    fn fetch(&self, source: &KnowledgeSource) -> Result<RawKnowledge, String>;
    fn validate(&self, raw: &RawKnowledge) -> Result<ValidationReport, String>;
    fn normalize(&self, raw: RawKnowledge) -> Result<KnowledgeIR, String>;
}

// ---------------------------------------------------------------------------
// Knowledge Compiler — transforms KnowledgeIR into compiled artifacts
// This is the ONLY place that creates KnowledgePack from IR.
// Providers never know about KnowledgePack.
// ---------------------------------------------------------------------------

/// Compile KnowledgeIR into a KnowledgePack
pub fn compile_ir_to_pack(ir: KnowledgeIR) -> KnowledgePack {
    let mut knowledge = Vec::new();
    let mut tools = Vec::new();
    let mut prompts = HashMap::new();

    for item in &ir.items {
        // Knowledge items
        knowledge.push(KnowledgeItem {
            title: item.title.clone(),
            content: item.content.clone(),
        });
    }

    // Build a system prompt from the IR
    let synopsis: Vec<&str> = ir.items.iter().filter(|i| matches!(i.kind, KnowledgeKind::Synopsis)).map(|i| i.content.as_str()).collect();
    let description: Vec<&str> = ir.items.iter().filter(|i| matches!(i.kind, KnowledgeKind::Description)).map(|i| i.content.as_str()).collect();
    let options: Vec<&str> = ir.items.iter().filter(|i| matches!(i.kind, KnowledgeKind::Option)).map(|i| i.content.as_str()).collect();
    let examples: Vec<&str> = ir.items.iter().filter(|i| matches!(i.kind, KnowledgeKind::Example)).map(|i| i.content.as_str()).collect();

    if !synopsis.is_empty() || !description.is_empty() {
        let mut system = String::new();
        system.push_str("You are an expert in ");
        system.push_str(&ir.source.query);
        system.push_str(". Here is the official reference documentation:\n\n");
        if !description.is_empty() {
            system.push_str(&description.join("\n\n"));
            system.push_str("\n\n");
        }
        if !options.is_empty() {
            system.push_str("Available options:\n");
            for opt in &options {
                system.push_str(&format!("- {opt}\n"));
            }
            system.push('\n');
        }
        if !examples.is_empty() {
            system.push_str("Examples:\n");
            for ex in &examples {
                system.push_str(&format!("{ex}\n"));
            }
        }
        prompts.insert("system".to_string(), system);
    }

    KnowledgePack {
        id: format!("pack-{}", ir.provider),
        name: format!("{} Knowledge", ir.source.title),
        version: "1.0.0".to_string(),
        description: format!("Generated from {}: {}", ir.provider, ir.source.query),
        tags: ir.source.tags.clone(),
        depends_on: vec![],
        languages: ir.source.language.clone().map(|l| vec![l]).unwrap_or_default(),
        tools,
        prompts,
        knowledge,
        benchmarks: vec![],
    }
}

/// Full pipeline: discover → fetch → validate → normalize → compile
pub fn build_knowledge<P: KnowledgeProvider>(
    provider: &P,
    query: &str,
) -> Result<(KnowledgeIR, KnowledgePack), String> {
    let start = Instant::now();

    // Phase 1: Discover
    println!("  🔍 Discovering sources for '{}'...", query);
    let sources = provider.discover(query)?;
    if sources.is_empty() {
        return Err(format!("No sources found for '{query}'"));
    }
    println!("     Found {} source(s)", sources.len());

    let mut combined_ir = None;

    for source in &sources {
        // Phase 2: Fetch
        println!("  📥 Fetching '{}'...", source.title);
        let raw = provider.fetch(source)?;
        println!("     {} bytes", raw.content.len());

        // Phase 3: Validate
        let validation = provider.validate(&raw)?;
        if !validation.valid {
            for err in &validation.errors {
                eprintln!("  ⚠ Validation error: {}", err.message);
            }
        }

        // Phase 4: Normalize to IR
        println!("  🔄 Normalizing...");
        let mut ir = provider.normalize(raw)?;
        ir.deduplicate();
        println!("     {} items extracted", ir.metrics.items_extracted);
        if ir.metrics.dedup_removed > 0 {
            println!("     {} duplicates removed", ir.metrics.dedup_removed);
        }

        combined_ir = Some(ir);
    }

    let ir = combined_ir.ok_or_else(|| "No IR produced".to_string())?;

    // Phase 5: Compile to pack
    println!("  📦 Compiling pack...");
    let pack = compile_ir_to_pack(ir.clone());

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    println!("  ✅ Build completed in {:.0}ms", elapsed);
    println!("     Pack ID: {}", pack.id);
    println!("     Knowledge items: {}", pack.knowledge.len());

    Ok((ir, pack))
}

/// Get the default packs directory
pub fn default_packs_dir() -> PathBuf {
    // Look relative to the binary first, then project root
    let candidates = [
        PathBuf::from("knowledge/packs"),
        PathBuf::from("../knowledge/packs"),
        PathBuf::from("../../knowledge/packs"),
    ];
    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }
    PathBuf::from("knowledge/packs")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_system_prompt() {
        let pack = KnowledgePack {
            id: "pack-test".to_string(),
            name: "Test Pack".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            tags: vec!["test".to_string()],
            depends_on: vec![],
            languages: vec!["go".to_string()],
            tools: vec![ToolDef {
                name: "go".to_string(),
                command: "go build".to_string(),
                description: "Go compiler".to_string(),
                check: Some("go version".to_string()),
                installed: false,
                version: None,
            }],
            prompts: {
                let mut m = HashMap::new();
                m.insert("system".to_string(), "You are a Go expert.".to_string());
                m
            },
            knowledge: vec![KnowledgeItem {
                title: "Error Handling".to_string(),
                content: "Always check errors.".to_string(),
            }],
            benchmarks: vec![PackBenchmark {
                description: "Write HTTP server".to_string(),
                prompt: "Write a Go HTTP server".to_string(),
                expected_output: "Compilable Go".to_string(),
                max_tokens: 300,
            }],
        };

        let prompt = build_system_prompt(&pack, None);
        assert!(prompt.contains("You are a Go expert"));
        assert!(prompt.contains("Always check errors"));
        assert!(prompt.contains("Go compiler"));
    }

    #[test]
    fn test_discover_packs_nonexistent() {
        let packs = discover_packs(Path::new("/nonexistent/packs"));
        assert!(packs.is_empty(), "Should return empty for nonexistent dir");
    }
}
