use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_yaml::Value as YamlValue;
use std::collections::HashMap;
use std::path::Path;

use crate::graph::{KnowledgeGraph, build_knowledge_graph};
use crate::parser::Document;

/// Search index entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEntry {
    pub id: String,
    pub title: String,
    pub doc_type: String,
    pub status: String,
    pub path: String,
    pub snippet: String,
}

/// Timeline entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub date: String,
    pub id: String,
    pub title: String,
    pub event: String,
}

/// Decision log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEntry {
    pub date: String,
    pub id: String,
    pub title: String,
    pub decision: String,
    pub rationale: String,
}

/// Diagnostic message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: String,
    pub id: String,
    pub message: String,
    pub path: String,
}

/// Cross-reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReference {
    pub source_id: String,
    pub source_title: String,
    pub target_id: String,
    pub target_title: String,
    pub relationship: String,
}

/// Complete compiler output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerOutput {
    pub graph: KnowledgeGraph,
    pub index: Vec<SearchEntry>,
    pub timeline: Vec<TimelineEntry>,
    pub decisions: Vec<DecisionEntry>,
    pub diagnostics: Vec<Diagnostic>,
    pub references: Vec<CrossReference>,
    pub ontology: serde_json::Value,
}

/// Generate the search index from documents
pub fn generate_search_index(documents: &[Document]) -> Vec<SearchEntry> {
    documents
        .iter()
        .map(|doc| {
            let title = doc
                .front_matter
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled");

            let doc_type = doc.id.split('-').next().unwrap_or("UNKNOWN");
            let status = doc
                .front_matter
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");

            // Take first 200 chars of body as snippet
            let snippet = doc.body.chars().take(200).collect::<String>();

            SearchEntry {
                id: doc.id.clone(),
                title: title.to_string(),
                doc_type: doc_type.to_string(),
                status: status.to_string(),
                path: doc.path.clone(),
                snippet,
            }
        })
        .collect()
}

/// Generate timeline from document dates
pub fn generate_timeline(documents: &[Document]) -> Vec<TimelineEntry> {
    let mut entries: Vec<TimelineEntry> = documents
        .iter()
        .filter_map(|doc| {
            let date = doc.front_matter.get("date")?.as_str()?;
            let title = doc
                .front_matter
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled");

            Some(TimelineEntry {
                date: date.to_string(),
                id: doc.id.clone(),
                title: title.to_string(),
                event: format!("{} created/updated", doc.id),
            })
        })
        .collect();

    entries.sort_by(|a, b| a.date.cmp(&b.date));
    entries
}

/// Generate decision log entries from documents
pub fn generate_decisions(documents: &[Document]) -> Vec<DecisionEntry> {
    let mut entries = Vec::new();

    for doc in documents {
        // Try to extract Decision Log from markdown body
        if let Some(log_section) = doc.body.split("## Decision Log").nth(1) {
            for line in log_section.lines() {
                if line.starts_with('|') && line.contains("|") {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 3 {
                        let date = parts[1].trim();
                        let decision = parts.get(2).map(|s| s.trim()).unwrap_or("");
                        let rationale = parts.get(3).map(|s| s.trim()).unwrap_or("");
                        let title = doc
                            .front_matter
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");

                        // Skip header/separator rows
                        if !date.is_empty() && !date.contains("Date") && !date.contains("---") {
                            entries.push(DecisionEntry {
                                date: date.to_string(),
                                id: doc.id.clone(),
                                title: title.to_string(),
                                decision: decision.to_string(),
                                rationale: rationale.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    entries.sort_by(|a, b| a.date.cmp(&b.date));
    entries
}

/// Generate cross-references between documents
pub fn generate_references(documents: &[Document], graph: &KnowledgeGraph) -> Vec<CrossReference> {
    let title_map: HashMap<String, String> = documents
        .iter()
        .map(|d| {
            let title = d
                .front_matter
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled")
                .to_string();
            (d.id.clone(), title)
        })
        .collect();

    graph
        .edges
        .iter()
        .map(|edge| CrossReference {
            source_id: edge.source.clone(),
            source_title: title_map.get(&edge.source).cloned().unwrap_or_default(),
            target_id: edge.target.clone(),
            target_title: title_map.get(&edge.target).cloned().unwrap_or_default(),
            relationship: edge.relationship.clone(),
        })
        .collect()
}

/// Generate diagnostics: missing references, validation errors, etc.
pub fn generate_diagnostics(documents: &[Document], graph: &KnowledgeGraph) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let known_ids: HashMap<String, &Document> =
        documents.iter().map(|d| (d.id.clone(), d)).collect();

    // Check all documents have required fields
    for doc in documents {
        // Check for missing date
        if doc
            .front_matter
            .get("date")
            .and_then(|v| v.as_str())
            .is_none()
        {
            diagnostics.push(Diagnostic {
                severity: "warning".to_string(),
                id: doc.id.clone(),
                message: "Missing required 'date' field in front-matter".to_string(),
                path: doc.path.clone(),
            });
        }

        // Check for missing title
        if doc
            .front_matter
            .get("title")
            .and_then(|v| v.as_str())
            .is_none()
        {
            diagnostics.push(Diagnostic {
                severity: "warning".to_string(),
                id: doc.id.clone(),
                message: "Missing optional 'title' field in front-matter".to_string(),
                path: doc.path.clone(),
            });
        }

        // Check for missing status
        if doc
            .front_matter
            .get("status")
            .and_then(|v| v.as_str())
            .is_none()
        {
            diagnostics.push(Diagnostic {
                severity: "warning".to_string(),
                id: doc.id.clone(),
                message: "Missing required 'status' field in front-matter".to_string(),
                path: doc.path.clone(),
            });
        }
    }

    // Check for broken references in the graph
    for edge in &graph.edges {
        // Check if source exists
        if !known_ids.contains_key(&edge.source) {
            diagnostics.push(Diagnostic {
                severity: "error".to_string(),
                id: edge.source.clone(),
                message: format!("Document '{}' referenced but not found", edge.source),
                path: "unknown".to_string(),
            });
        }
    }

    diagnostics
}

/// Load the ontology from the ontology.yaml file
pub fn load_ontology(project_root: &Path) -> Result<serde_json::Value> {
    let ontology_path = project_root.join("knowledge").join("ontology.yaml");
    if !ontology_path.exists() {
        return Ok(serde_json::Value::Null);
    }

    let content = std::fs::read_to_string(&ontology_path)
        .with_context(|| format!("Failed to read ontology: {}", ontology_path.display()))?;

    let yaml_value: YamlValue = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse ontology YAML: {}", ontology_path.display()))?;

    let json_value =
        serde_json::to_value(&yaml_value).context("Failed to convert ontology to JSON")?;

    Ok(json_value)
}

/// Run the full compilation pipeline and return the output
pub fn compile_all(documents: &[Document], project_root: &Path) -> CompilerOutput {
    let graph = build_knowledge_graph(documents);
    let index = generate_search_index(documents);
    let timeline = generate_timeline(documents);
    let decisions = generate_decisions(documents);
    let references = generate_references(documents, &graph);
    let diagnostics = generate_diagnostics(documents, &graph);
    let ontology = load_ontology(project_root).unwrap_or(serde_json::Value::Null);

    CompilerOutput {
        graph,
        index,
        timeline,
        decisions,
        diagnostics,
        references,
        ontology,
    }
}

/// Write a JSON file
fn write_json_file<T: serde::Serialize>(path: &Path, data: &T, filename: &str) -> Result<()> {
    let file_path = path.join(filename);
    let json = serde_json::to_string_pretty(data)?;
    std::fs::write(&file_path, json)?;
    println!("  ✓ Generated {}", filename);
    Ok(())
}

pub fn write_output(output: &CompilerOutput, output_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(output_dir)?;

    write_json_file(output_dir, &output.graph, "graph.json")?;
    write_json_file(output_dir, &output.index, "index.json")?;
    write_json_file(output_dir, &output.timeline, "timeline.json")?;
    write_json_file(output_dir, &output.decisions, "decisions.json")?;
    write_json_file(output_dir, &output.diagnostics, "diagnostics.json")?;
    write_json_file(output_dir, &output.references, "references.json")?;

    // Only write ontology if it's not null
    if output.ontology != serde_json::Value::Null {
        write_json_file(output_dir, &output.ontology, "ontology.json")?;
    }

    // Write a summary file
    let summary = serde_json::json!({
        "generated_at": output.graph.metadata.generated_at,
        "total_documents": output.graph.metadata.total_nodes,
        "total_relationships": output.graph.metadata.total_edges,
        "document_types": output.graph.metadata.doc_types,
        "status_counts": output.graph.metadata.status_counts,
        "warnings": output.diagnostics.iter().filter(|d| d.severity == "warning").count(),
        "errors": output.diagnostics.iter().filter(|d| d.severity == "error").count(),
    });
    write_json_file(output_dir, &summary, "summary.json")?;

    Ok(())
}
