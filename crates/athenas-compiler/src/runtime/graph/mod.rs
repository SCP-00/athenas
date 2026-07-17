pub mod eng_node;

use eng_node::{EngEdge, EngEdgeKind, EngNode, EngNodeData, RuntimeStatus};
use serde::{Deserialize, Serialize};

/// The Engineering Graph — single source of truth for all project knowledge.
/// Runtime data structure: Vec<Node> + Vec<Edge>. No database, no indexes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineeringGraph {
    pub nodes: Vec<EngNode>,
    pub edges: Vec<EngEdge>,
}

impl EngineeringGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: EngNode) -> String {
        let id = node.id.clone();
        // Don't add duplicates (same id)
        if !self.nodes.iter().any(|n| n.id == id) {
            self.nodes.push(node);
        }
        id
    }

    pub fn add_edge(&mut self, source: String, target: String, kind: EngEdgeKind) {
        // Don't add duplicate edges
        if !self.edges.iter().any(|e| e.source == source && e.target == target && e.kind == kind) {
            self.edges.push(EngEdge::new(source, target, kind));
        }
    }

    pub fn add_edge_checked(&mut self, source: &str, target: &str, kind: EngEdgeKind) {
        if self.nodes.iter().any(|n| n.id == source) && self.nodes.iter().any(|n| n.id == target) {
            self.add_edge(source.to_string(), target.to_string(), kind);
        }
    }

    /// Find all nodes matching a type predicate
    pub fn nodes_by_type(&self, pred: impl Fn(&EngNodeData) -> bool) -> Vec<&EngNode> {
        self.nodes.iter().filter(|n| pred(&n.data)).collect()
    }

    /// Find outgoing edges from a node
    pub fn outgoing(&self, node_id: &str, kind: Option<EngEdgeKind>) -> Vec<&EngEdge> {
        self.edges.iter().filter(|e| {
            e.source == node_id && kind.as_ref().map_or(true, |k| e.kind == *k)
        }).collect()
    }

    /// Find incoming edges to a node
    pub fn incoming(&self, node_id: &str, kind: Option<EngEdgeKind>) -> Vec<&EngEdge> {
        self.edges.iter().filter(|e| {
            e.target == node_id && kind.as_ref().map_or(true, |k| e.kind == *k)
        }).collect()
    }

    /// Traverse from starting nodes following a path of edge kinds.
    /// Returns all reachable target nodes.
    pub fn traverse(
        &self,
        start_pred: impl Fn(&EngNodeData) -> bool,
        path: &[EngEdgeKind],
    ) -> Vec<&EngNode> {
        let mut current: Vec<String> = self.nodes.iter()
            .filter(|n| start_pred(&n.data))
            .map(|n| n.id.clone())
            .collect();

        for edge_kind in path {
            let mut next = Vec::new();
            for node_id in &current {
                for edge in &self.edges {
                    if edge.source == *node_id && edge.kind == *edge_kind {
                        if !next.contains(&edge.target) {
                            next.push(edge.target.clone());
                        }
                    }
                }
            }
            current = next;
        }

        self.nodes.iter().filter(|n| current.contains(&n.id)).collect()
    }

    /// Count nodes by label
    pub fn count_by_label(&self) -> Vec<(&str, usize)> {
        let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for node in &self.nodes {
            *counts.entry(node.data.label()).or_insert(0) += 1;
        }
        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }

    /// Find broken edges — edges whose source or target doesn't exist
    pub fn broken_edges(&self) -> Vec<&EngEdge> {
        self.edges.iter().filter(|e| {
            !self.nodes.iter().any(|n| n.id == e.source)
                || !self.nodes.iter().any(|n| n.id == e.target)
        }).collect()
    }

    /// Create a standard ID from a name
    pub fn make_id(prefix: &str, name: &str) -> String {
        let clean: String = name.chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        format!("{prefix}::{}", clean.to_lowercase())
    }

    /// Detect if a command binary is available
    pub fn check_command(cmd: &str) -> Option<String> {
        let output = std::process::Command::new("which").arg(cmd).output().ok()?;
        if output.status.success() {
            String::from_utf8(output.stdout).ok()
                .map(|s| s.trim().to_string())
        } else {
            None
        }
    }

    /// Detect runtime process via health check
    pub fn check_runtime_endpoint(endpoint: &str) -> bool {
        use std::io::{Read, Write};
        use std::net::TcpStream;
        use std::time::Duration;

        if let Ok(mut stream) = TcpStream::connect_timeout(
            &endpoint.parse().unwrap_or_else(|_| "127.0.0.1:11434".parse().unwrap()),
            Duration::from_millis(200),
        ) {
            let request = format!(
                "GET /health HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
                endpoint
            );
            stream.write_all(request.as_bytes()).ok();
            let mut buf = [0u8; 256];
            stream.read(&mut buf).ok();
            return true;
        }
        false
    }
}

impl Default for EngineeringGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Save the graph to disk
pub fn save_graph(graph: &EngineeringGraph, path: &std::path::Path) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(graph)?;
    std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new(".")))?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Load the graph from disk
pub fn load_graph(path: &std::path::Path) -> anyhow::Result<EngineeringGraph> {
    let content = std::fs::read_to_string(path)?;
    let graph: EngineeringGraph = serde_json::from_str(&content)?;
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let g = EngineeringGraph::new();
        assert_eq!(g.nodes.len(), 0);
        assert_eq!(g.edges.len(), 0);
    }

    #[test]
    fn test_add_node_and_edge() {
        let mut g = EngineeringGraph::new();
        let n1 = EngNode::new("proj::test".into(), EngNodeData::Project {
            name: "test".into(), path: "/tmp/test".into(),
        });
        let n2 = EngNode::new("lang::rust".into(), EngNodeData::Language {
            name: "Rust".into(), version: "1.89.0".into(),
        });
        g.add_node(n1);
        g.add_node(n2);
        g.add_edge("proj::test".into(), "lang::rust".into(), EngEdgeKind::HasTool);
        assert_eq!(g.nodes.len(), 2);
        assert_eq!(g.edges.len(), 1);
    }

    #[test]
    fn test_traverse() {
        let mut g = EngineeringGraph::new();
        let proj = EngNode::new("proj".into(), EngNodeData::Project {
            name: "test".into(), path: "/test".into(),
        });
        let lang = EngNode::new("lang".into(), EngNodeData::Language {
            name: "Rust".into(), version: "1.0".into(),
        });
        let tool = EngNode::new("tool".into(), EngNodeData::Compiler {
            name: "rustc".into(), version: "1.0".into(),
        });
        g.add_node(proj);
        g.add_node(lang);
        g.add_node(tool);
        g.add_edge("proj".into(), "lang".into(), EngEdgeKind::HasTool);
        g.add_edge("lang".into(), "tool".into(), EngEdgeKind::HasTool);
        let result = g.traverse(|d| matches!(d, EngNodeData::Project { .. }), &[EngEdgeKind::HasTool]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "lang");
    }

    #[test]
    fn test_broken_edges() {
        let mut g = EngineeringGraph::new();
        g.add_edge("missing".into(), "nowhere".into(), EngEdgeKind::DependsOn);
        assert_eq!(g.broken_edges().len(), 1);
    }
}
