use anyhow::{Context, Result};
use serde_yaml::Value;
use sha2::{Digest, Sha256};
use std::path::Path;
use walkdir::WalkDir;

/// A parsed document with front-matter, body, and computed hash
#[derive(Debug, Clone)]
pub struct Document {
    /// Document ID (e.g., "CONST-0001")
    pub id: String,
    /// File path relative to project root
    pub path: String,
    /// Raw YAML front-matter as a serde Value
    pub front_matter: Value,
    /// Markdown body (everything after the front-matter)
    pub body: String,
    /// Full raw content including front-matter
    pub raw: String,
    /// SHA256 hash of the raw content
    pub hash: String,
}

/// Find all markdown files in the given directory, excluding templates and node_modules
pub fn find_markdown_files(root: &Path) -> Vec<String> {
    let mut files = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let path = e.path();
            // Skip hidden directories, node_modules, .git, templates
            if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                return !name.starts_with('.')
                    && name != "node_modules"
                    && name != "templates"
                    && name != "target";
            }
            true
        })
    {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "md")
            && !path.to_string_lossy().contains("templates")
        {
            if let Ok(relative) = path.strip_prefix(root) {
                files.push(relative.to_string_lossy().to_string());
            }
        }
    }

    files.sort();
    files
}

/// Parse a markdown file, extracting YAML front-matter and body
pub fn parse_document(file_path: &Path, root: &Path) -> Result<Option<Document>> {
    let raw = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read {}", file_path.display()))?;

    if raw.is_empty() {
        return Ok(None);
    }

    // Compute SHA256 hash of raw content
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    let hash = hex::encode(hasher.finalize());

    // Extract front-matter between --- delimiters
    let trimmed = raw.trim_start();
    if !trimmed.starts_with("---") {
        return Ok(None);
    }

    let after_first = &trimmed[3..];
    let end = after_first.find("---").unwrap_or(0);

    if end == 0 {
        return Ok(None);
    }

    let yaml_str = &after_first[..end].trim();
    let body_start = &after_first[end + 3..];

    // Parse YAML front-matter
    let front_matter: Value = serde_yaml::from_str(yaml_str)
        .with_context(|| format!("Failed to parse YAML in {}", file_path.display()))?;

    // Extract document ID
    let id = front_matter
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("UNKNOWN")
        .to_string();

    if id == "UNKNOWN" {
        return Ok(None);
    }

    let relative_path = file_path
        .strip_prefix(root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .to_string();

    Ok(Some(Document {
        id,
        path: relative_path,
        front_matter,
        body: body_start.trim().to_string(),
        raw: trimmed.to_string(),
        hash,
    }))
}

/// Parse all documents in the project
pub fn parse_all_documents(root: &Path) -> Result<Vec<Document>> {
    let files = find_markdown_files(root);
    let mut documents = Vec::new();

    for file in files {
        let full_path = root.join(&file);
        if let Ok(Some(doc)) = parse_document(&full_path, root) {
            documents.push(doc);
        }
    }

    Ok(documents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_markdown_files() {
        let files = find_markdown_files(Path::new("."));
        assert!(!files.is_empty(), "Should find at least some markdown files");
    }
}
