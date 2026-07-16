use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::parser::Document;

/// A node in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub title: String,
    pub doc_type: String,
    pub status: String,
    pub authority_level: u8,
    pub path: String,
    pub hash: String,
    pub tags: Vec<String>,
}

/// An edge (relationship) between two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relationship: String,
}

/// The complete knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub metadata: GraphMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub doc_types: HashMap<String, usize>,
    pub status_counts: HashMap<String, usize>,
    pub generated_at: String,
}

/// Extract document type prefix from ID (e.g., "CONST-0001" -> "CONST")
pub fn doc_type_from_id(id: &str) -> String {
    id.split('-').next().unwrap_or("UNKNOWN").to_string()
}

/// Get authority level from document type prefix
pub fn authority_level(doc_type: &str) -> u8 {
    match doc_type {
        "CONST" => 0,
        "VISION" => 1,
        "REQ" => 2,
        "SPEC" => 3,
        "ARCH" => 4,
        "DIRECTIVE" => 3,
        "ADR" => 4,
        "BENCH" => 6,
        "EXP" => 7,
        "MODEL" => 5,
        "PROFILE" => 5,
        "EVID" => 7,
        "INDEX" => 8,
        "CONTEXT" => 8,
        "BOOTSTRAP" => 8,
        _ => 8,
    }
}

/// Extract tags from front-matter
fn extract_tags(doc: &Document) -> Vec<String> {
    doc.front_matter
        .get("tags")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Extract relationship fields from front-matter
fn extract_relationships(doc: &Document) -> Vec<(String, String)> {
    let relationship_keys = [
        "implements",
        "depends_on",
        "validated_by",
        "derived_from",
        "supersedes",
        "related",
        "validates",
    ];

    let mut edges = Vec::new();

    for key in &relationship_keys {
        if let Some(values) = doc.front_matter.get(*key).and_then(|v| v.as_sequence()) {
            for value in values {
                if let Some(target) = value.as_str() {
                    let rel = match *key {
                        "implements" => "implements".to_string(),
                        "depends_on" => "depends_on".to_string(),
                        "validated_by" => "validated_by".to_string(),
                        "derived_from" => "derived_from".to_string(),
                        "supersedes" => "supersedes".to_string(),
                        "related" => "related_to".to_string(),
                        "validates" => "validates".to_string(),
                        _ => "related_to".to_string(),
                    };
                    edges.push((target.to_string(), rel));
                }
            }
        }
    }

    edges
}

/// Build the complete knowledge graph from parsed documents
pub fn build_knowledge_graph(documents: &[Document]) -> KnowledgeGraph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut doc_types: HashMap<String, usize> = HashMap::new();
    let mut status_counts: HashMap<String, usize> = HashMap::new();

    let known_ids: HashSet<String> = documents.iter().map(|d| d.id.clone()).collect();

    for doc in documents {
        let doc_type = doc_type_from_id(&doc.id);
        let level = authority_level(&doc_type);
        let status = doc
            .front_matter
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let title = doc
            .front_matter
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled")
            .to_string();

        let tags = extract_tags(doc);

        // Count types and statuses
        let dt = doc_type.clone();
        *doc_types.entry(dt).or_insert(0) += 1;

        // Add node
        nodes.push(GraphNode {
            id: doc.id.clone(),
            title,
            doc_type,
            status: status.clone(),
            authority_level: level,
            path: doc.path.clone(),
            hash: doc.hash.clone(),
            tags,
        });
        *status_counts.entry(status).or_insert(0) += 1;

        // Extract relationships
        let rels = extract_relationships(doc);
        for (target, relationship) in rels {
            // Only add edge if target exists in our document set
            if known_ids.contains(&target) {
                edges.push(GraphEdge {
                    source: doc.id.clone(),
                    target,
                    relationship,
                });
            }
        }
    }

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    KnowledgeGraph {
        metadata: GraphMetadata {
            total_nodes: nodes.len(),
            total_edges: edges.len(),
            doc_types,
            status_counts,
            generated_at: now,
        },
        nodes,
        edges,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_type_extraction() {
        assert_eq!(doc_type_from_id("CONST-0001"), "CONST");
        assert_eq!(doc_type_from_id("REQ-0042"), "REQ");
        assert_eq!(doc_type_from_id("BENCH-9999"), "BENCH");
    }

    #[test]
    fn test_authority_levels() {
        assert_eq!(authority_level("CONST"), 0);
        assert_eq!(authority_level("ARCH"), 4);
        assert_eq!(authority_level("BENCH"), 6);
        assert_eq!(authority_level("UNKNOWN"), 8);
    }
}
