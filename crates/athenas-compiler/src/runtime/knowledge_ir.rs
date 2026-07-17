use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A reference to a knowledge source (e.g. a man page, a doc page, a CLI --help output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSource {
    pub id: String,
    pub provider: String,
    pub query: String,
    pub title: String,
    pub language: Option<String>,
    pub tags: Vec<String>,
    pub size_bytes: usize,
}

/// Raw knowledge fetched from a provider (before normalization)
#[derive(Debug, Clone)]
pub struct RawKnowledge {
    pub source: KnowledgeSource,
    pub content: String,
    #[allow(dead_code)]
    pub content_type: String, // "man", "help", "lsp", "html", "json"
}

/// Validation result from a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub source_id: String,
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub severity: String, // "error", "warning"
    pub message: String,
    pub location: Option<String>,
}

/// A single normalized piece of knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub kind: KnowledgeKind,
    pub language: Option<String>,
    pub tags: Vec<String>,
    pub source: String, // provider:query
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeKind {
    Synopsis,
    Description,
    Option,
    Example,
    ExitCode,
    Diagnostic,
    Reference,
    Concept,
    Pattern,
    Api,
    Configuration,
    Troubleshooting,
}

impl std::fmt::Display for KnowledgeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KnowledgeKind::Synopsis => write!(f, "synopsis"),
            KnowledgeKind::Description => write!(f, "description"),
            KnowledgeKind::Option => write!(f, "option"),
            KnowledgeKind::Example => write!(f, "example"),
            KnowledgeKind::ExitCode => write!(f, "exit-code"),
            KnowledgeKind::Diagnostic => write!(f, "diagnostic"),
            KnowledgeKind::Reference => write!(f, "reference"),
            KnowledgeKind::Concept => write!(f, "concept"),
            KnowledgeKind::Pattern => write!(f, "pattern"),
            KnowledgeKind::Api => write!(f, "api"),
            KnowledgeKind::Configuration => write!(f, "configuration"),
            KnowledgeKind::Troubleshooting => write!(f, "troubleshooting"),
        }
    }
}

/// The complete intermediate representation emitted by a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeIR {
    pub provider: String,
    pub source: KnowledgeSource,
    pub validation: ValidationReport,
    pub items: Vec<KnowledgeItem>,
    pub extracted_at: String,
    pub metrics: CompileMetrics,
}

/// Metrics collected during compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileMetrics {
    pub raw_bytes: usize,
    pub items_extracted: usize,
    pub validation_errors: usize,
    pub validation_warnings: usize,
    pub compile_time_ms: f64,
    pub dedup_removed: usize,
}

#[allow(dead_code)]
impl KnowledgeIR {
    /// Filter items by kind
    pub fn by_kind(&self, kind: &KnowledgeKind) -> Vec<&KnowledgeItem> {
        self.items.iter().filter(|i| matches!(i.kind, _) && std::mem::discriminant(&i.kind) == std::mem::discriminant(kind)).collect()
    }

    #[allow(dead_code)]
    /// Filter items by language
    pub fn by_language(&self, lang: &str) -> Vec<&KnowledgeItem> {
        self.items.iter().filter(|i| i.language.as_deref() == Some(lang)).collect()
    }

    /// Deduplicate items by content
    pub fn deduplicate(&mut self) {
        let mut seen = std::collections::HashSet::new();
        let before = self.items.len();
        self.items.retain(|item| {
            seen.insert(item.content.clone())
        });
        self.metrics.dedup_removed = before - self.items.len();
    }
}

/// Generate an ID from a string
pub fn generate_id(provider: &str, title: &str) -> String {
    let safe = title
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "-")
        .trim_matches('-')
        .to_string();
    format!("{}-{}", provider, &safe[..safe.len().min(40)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id = generate_id("man", "go build");
        assert_eq!(id, "man-go-build");
    }

    #[test]
    fn test_kind_display() {
        assert_eq!(KnowledgeKind::Synopsis.to_string(), "synopsis");
        assert_eq!(KnowledgeKind::Example.to_string(), "example");
    }

    #[test]
    fn test_deduplicate_removes_duplicates() {
        let mut ir = KnowledgeIR {
            provider: "test".to_string(),
            source: KnowledgeSource {
                id: "test".to_string(),
                provider: "test".to_string(),
                query: "test".to_string(),
                title: "Test".to_string(),
                language: None,
                tags: vec![],
                size_bytes: 0,
            },
            validation: ValidationReport {
                source_id: "test".to_string(),
                valid: true,
                errors: vec![],
                warnings: vec![],
                metadata: HashMap::new(),
            },
            items: vec![
                KnowledgeItem {
                    id: "a".to_string(),
                    title: "A".to_string(),
                    content: "same content".to_string(),
                    kind: KnowledgeKind::Concept,
                    language: None,
                    tags: vec![],
                    source: "test:a".to_string(),
                    confidence: 1.0,
                },
                KnowledgeItem {
                    id: "b".to_string(),
                    title: "B".to_string(),
                    content: "same content".to_string(),
                    kind: KnowledgeKind::Concept,
                    language: None,
                    tags: vec![],
                    source: "test:b".to_string(),
                    confidence: 1.0,
                },
            ],
            extracted_at: "now".to_string(),
            metrics: CompileMetrics {
                raw_bytes: 0,
                items_extracted: 2,
                validation_errors: 0,
                validation_warnings: 0,
                compile_time_ms: 0.0,
                dedup_removed: 0,
            },
        };
        ir.deduplicate();
        assert_eq!(ir.items.len(), 1);
        assert_eq!(ir.metrics.dedup_removed, 1);
    }
}
