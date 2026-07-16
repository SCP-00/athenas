use std::collections::HashMap;
use std::process::Command;
use std::time::Instant;

use crate::runtime::knowledge::KnowledgeProvider;
use crate::runtime::knowledge_ir::*;

/// Provider that extracts knowledge from Unix man pages via `man <query>`
pub struct ManProvider;

impl ManProvider {
    pub fn new() -> Self {
        Self
    }
}

impl KnowledgeProvider for ManProvider {
    fn id(&self) -> &str {
        "man"
    }

    /// Discover available man pages for a query
    fn discover(&self, query: &str) -> Result<Vec<KnowledgeSource>, String> {
        // Try man -w to find the man page path
        let output = Command::new("man")
            .args(["-w", query])
            .output()
            .map_err(|e| format!("man command not found: {e}"))?;

        if !output.status.success() {
            return Err(format!("No man page found for '{query}'"));
        }

        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let size = std::fs::metadata(&path).map(|m| m.len() as usize).unwrap_or(0);

        Ok(vec![KnowledgeSource {
            id: format!("man-{}", query.replace(' ', "-")),
            provider: "man".to_string(),
            query: query.to_string(),
            title: format!("man {}", query),
            language: None,
            tags: vec!["man".to_string(), query.to_string()],
            size_bytes: size,
        }])
    }

    /// Fetch raw man page content
    fn fetch(&self, source: &KnowledgeSource) -> Result<RawKnowledge, String> {
        let output = Command::new("man")
            .args(["--no-hyphenation", "--no-justification", &source.query])
            .output()
            .map_err(|e| format!("Failed to run man: {e}"))?;

        if !output.status.success() {
            return Err(format!("man {} returned non-zero exit", source.query));
        }

        let content = String::from_utf8_lossy(&output.stdout).to_string();

        Ok(RawKnowledge {
            source: source.clone(),
            content,
            content_type: "man".to_string(),
        })
    }

    /// Validate that the man page has meaningful content
    fn validate(&self, raw: &RawKnowledge) -> Result<ValidationReport, String> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut metadata = HashMap::new();

        let line_count = raw.content.lines().count();
        metadata.insert("lines".to_string(), line_count.to_string());
        metadata.insert("bytes".to_string(), raw.content.len().to_string());

        if raw.content.trim().is_empty() {
            errors.push(ValidationError {
                severity: "error".to_string(),
                message: "Empty man page".to_string(),
                location: None,
            });
        }

        if line_count < 5 {
            warnings.push(format!("Very short man page ({} lines)", line_count));
        }

        let has_name_section = raw.content.contains("NAME") || raw.content.contains("name");
        if !has_name_section {
            warnings.push("No NAME section found".to_string());
        }

        let has_description = raw.content.contains("DESCRIPTION") || raw.content.contains("description");
        if !has_description {
            warnings.push("No DESCRIPTION section found".to_string());
        }

        Ok(ValidationReport {
            source_id: raw.source.id.clone(),
            valid: errors.is_empty(),
            errors,
            warnings,
            metadata,
        })
    }

    /// Normalize man page content into structured KnowledgeItems
    fn normalize(&self, raw: RawKnowledge) -> Result<KnowledgeIR, String> {
        let start = Instant::now();
        let content = &raw.content;
        let mut items = Vec::new();

        // Extract NAME section
        if let Some(name_block) = extract_section(content, "NAME") {
            items.push(KnowledgeItem {
                id: generate_id("man", &format!("{}-name", raw.source.query.replace(' ', "-"))),
                title: format!("{} — {}", raw.source.query, name_block.lines().next().unwrap_or("")),
                content: name_block.clone(),
                kind: KnowledgeKind::Synopsis,
                language: None,
                tags: vec!["man".to_string(), "synopsis".to_string()],
                source: format!("man:{}", raw.source.query),
                confidence: 0.95,
            });
        }

        // Extract SYNOPSIS section
        if let Some(synopsis) = extract_section(content, "SYNOPSIS") {
            items.push(KnowledgeItem {
                id: generate_id("man", &format!("{}-synopsis", raw.source.query.replace(' ', "-"))),
                title: format!("{} synopsis", raw.source.query),
                content: synopsis,
                kind: KnowledgeKind::Synopsis,
                language: None,
                tags: vec!["man".to_string(), "syntax".to_string()],
                source: format!("man:{}", raw.source.query),
                confidence: 0.95,
            });
        }

        // Extract DESCRIPTION section
        if let Some(desc) = extract_section(content, "DESCRIPTION") {
            items.push(KnowledgeItem {
                id: generate_id("man", &format!("{}-description", raw.source.query.replace(' ', "-"))),
                title: format!("{} description", raw.source.query),
                content: desc,
                kind: KnowledgeKind::Description,
                language: None,
                tags: vec!["man".to_string(), "description".to_string()],
                source: format!("man:{}", raw.source.query),
                confidence: 0.9,
            });
        }

        // Extract OPTIONS section — parse each option flag
        if let Some(options) = extract_section(content, "OPTIONS") {
            // Split into individual option entries
            for line in options.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('-') && trimmed.len() > 1 {
                    let (flag, desc) = trimmed.split_once(|c| c == ' ' || c == '\t')
                        .map(|(f, d)| (f.trim().to_string(), d.trim().to_string()))
                        .unwrap_or_else(|| (trimmed.to_string(), String::new()));

                    let desc_clean = desc.trim_matches(|c| c == '-' || c == ' ').trim().to_string();
                    if !desc_clean.is_empty() {
                        items.push(KnowledgeItem {
                            id: generate_id("man", &format!("{}-option-{}", raw.source.query.replace(' ', "-"), flag.trim_start_matches('-'))),
                            title: flag.clone(),
                            content: desc_clean,
                            kind: KnowledgeKind::Option,
                            language: None,
                            tags: vec!["man".to_string(), "option".to_string(), flag.clone()],
                            source: format!("man:{}", raw.source.query),
                            confidence: 0.85,
                        });
                    }
                }
            }
        }

        // Extract EXAMPLES section
        if let Some(examples) = extract_section(content, "EXAMPLE") {
            items.push(KnowledgeItem {
                id: generate_id("man", &format!("{}-examples", raw.source.query.replace(' ', "-"))),
                title: format!("{} examples", raw.source.query),
                content: examples,
                kind: KnowledgeKind::Example,
                language: None,
                tags: vec!["man".to_string(), "example".to_string()],
                source: format!("man:{}", raw.source.query),
                confidence: 0.9,
            });
        }

        // Extract EXIT STATUS section
        if let Some(exit_codes) = extract_section(content, "EXIT STATUS") {
            items.push(KnowledgeItem {
                id: generate_id("man", &format!("{}-exit-codes", raw.source.query.replace(' ', "-"))),
                title: format!("{} exit codes", raw.source.query),
                content: exit_codes,
                kind: KnowledgeKind::ExitCode,
                language: None,
                tags: vec!["man".to_string(), "exit-code".to_string()],
                source: format!("man:{}", raw.source.query),
                confidence: 0.9,
            });
        }

        // Extract DIAGNOSTICS section
        if let Some(diag) = extract_section(content, "DIAGNOSTICS") {
            items.push(KnowledgeItem {
                id: generate_id("man", &format!("{}-diagnostics", raw.source.query.replace(' ', "-"))),
                title: format!("{} diagnostics", raw.source.query),
                content: diag,
                kind: KnowledgeKind::Diagnostic,
                language: None,
                tags: vec!["man".to_string(), "diagnostic".to_string()],
                source: format!("man:{}", raw.source.query),
                confidence: 0.85,
            });
        }

        // Extract SEE ALSO / REFERENCES section
        if let Some(see_also) = extract_section(content, "SEE ALSO") {
            items.push(KnowledgeItem {
                id: generate_id("man", &format!("{}-references", raw.source.query.replace(' ', "-"))),
                title: format!("{} references", raw.source.query),
                content: see_also,
                kind: KnowledgeKind::Reference,
                language: None,
                tags: vec!["man".to_string(), "reference".to_string()],
                source: format!("man:{}", raw.source.query),
                confidence: 0.9,
            });
        }

        // If no structured sections found, treat entire body as a concept
        if items.is_empty() && !content.trim().is_empty() {
            let preview: String = content.chars().take(1000).collect();
            items.push(KnowledgeItem {
                id: generate_id("man", &raw.source.query.replace(' ', "-")),
                title: format!("man {}", raw.source.query),
                content: preview,
                kind: KnowledgeKind::Concept,
                language: None,
                tags: vec!["man".to_string()],
                source: format!("man:{}", raw.source.query),
                confidence: 0.5,
            });
        }

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        let items_count = items.len();
        let raw_bytes = content.len();

        let validation = ValidationReport {
            source_id: raw.source.id.clone(),
            valid: true,
            errors: vec![],
            warnings: vec![],
            metadata: HashMap::new(),
        };

        Ok(KnowledgeIR {
            provider: "man".to_string(),
            source: raw.source,
            validation,
            items,
            extracted_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            metrics: CompileMetrics {
                raw_bytes,
                items_extracted: items_count,
                validation_errors: 0,
                validation_warnings: 0,
                compile_time_ms: elapsed,
                dedup_removed: 0,
            },
        })
    }
}

/// Extract a section from man page output by finding the section header and reading
/// until the next section header (all-caps word at start of line).
fn extract_section(content: &str, section_name: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();

    // Find the section start
    let start = lines.iter().position(|l| {
        let trimmed = l.trim();
        trimmed == section_name
            || trimmed == format!("{}:", section_name)
            || trimmed.starts_with(&format!("{} ", section_name))
            || trimmed.starts_with(&format!("{}\t", section_name))
    })?;

    // Collect lines until next section header (all-caps word)
    let mut section_lines = Vec::new();
    for line in lines.iter().skip(start + 1) {
        let trimmed = line.trim();
        // Next section header: an all-caps word (possibly with trailing colon) at start of line
        if !trimmed.is_empty()
            && trimmed.chars().all(|c| c.is_uppercase() || c == ' ' || c == ':')
            && trimmed.len() < 40
            && !section_lines.is_empty()
        {
            break;
        }
        section_lines.push(*line);
    }

    let result = section_lines.join("\n").trim().to_string();
    if result.is_empty() { None } else { Some(result) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_section_from_sample() {
        let sample = "NAME\n    go-build — compile packages\n\nSYNOPSIS\n    go build [-o output] [packages]\n\nDESCRIPTION\n    Build compiles packages.\n\nSEE ALSO\n    go-install(1)\n";
        let name = extract_section(sample, "NAME");
        assert!(name.is_some());
        assert!(name.unwrap().contains("go-build"));

        let synopsis = extract_section(sample, "SYNOPSIS");
        assert!(synopsis.is_some());
        assert!(synopsis.unwrap().contains("go build"));
    }

    #[test]
    fn test_man_provider_normalize() {
        let provider = ManProvider::new();
        let sample = "NAME\n    go-build — compile packages\n\nSYNOPSIS\n    go build [-o output] [packages]\n\nDESCRIPTION\n    Build compiles the packages named by the import paths.\n\nOPTIONS\n    -o file\n        Write output to file instead of default.\n    -v\n        Print package names.\n\nSEE ALSO\n    go-install(1)\n";

        let raw = RawKnowledge {
            source: KnowledgeSource {
                id: "man-go-build".to_string(),
                provider: "man".to_string(),
                query: "go-build".to_string(),
                title: "man go-build".to_string(),
                language: Some("go".to_string()),
                tags: vec!["go".to_string()],
                size_bytes: sample.len(),
            },
            content: sample.to_string(),
            content_type: "man".to_string(),
        };

        let ir = provider.normalize(raw).unwrap();
        assert!(!ir.items.is_empty(), "Should extract items");
        assert!(ir.items.iter().any(|i| matches!(i.kind, KnowledgeKind::Synopsis)));
        assert!(ir.items.iter().any(|i| matches!(i.kind, KnowledgeKind::Option)));
        assert!(ir.metrics.items_extracted > 0);
    }
}
