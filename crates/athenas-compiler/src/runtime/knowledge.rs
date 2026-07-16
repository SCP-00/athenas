use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
