use anyhow::{Context, Result};
use jsonschema::validator_for;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

use crate::parser::Document;

/// Schema collection keyed by document type prefix
pub type SchemaMap = HashMap<String, jsonschema::Validator>;

/// Load all JSON schemas from the schemas/ directory
pub fn load_schemas(schemas_dir: &Path) -> Result<SchemaMap> {
    let mut schemas = SchemaMap::new();

    let schema_files = [
        ("CONST", "const.schema.json"),
        ("REQ", "req.schema.json"),
        ("SPEC", "spec.schema.json"),
        ("ARCH", "arch.schema.json"),
        ("ADR", "adr.schema.json"),
        ("BENCH", "bench.schema.json"),
        ("DIRECTIVE", "directive.schema.json"),
        ("INDEX", "index.schema.json"),
        ("CONTEXT", "context.schema.json"),
        ("BOOTSTRAP", "bootstrap.schema.json"),
        ("DIRECTIVE", "directive.schema.json"),
        ("INDEX", "index.schema.json"),
        ("CONTEXT", "context.schema.json"),
        ("BOOTSTRAP", "bootstrap.schema.json"),
    ];

    for (doc_type, filename) in &schema_files {
        let path = schemas_dir.join(filename);
        if !path.exists() {
            println!("  ⚠ Schema file not found: {}", path.display());
            continue;
        }

        let schema_content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read schema: {}", filename))?;
        let schema_value: Value = serde_json::from_str(&schema_content)
            .with_context(|| format!("Failed to parse schema JSON: {}", filename))?;

        let compiled = validator_for(&schema_value)
            .with_context(|| format!("Failed to compile schema: {}", filename))?;

        schemas.insert(doc_type.to_string(), compiled);
    }

    Ok(schemas)
}

/// Validate a document's front-matter against its type's schema
pub fn validate_document(
    doc: &Document,
    schemas: &SchemaMap,
) -> Vec<String> {
    let doc_type = doc.id.split('-').next().unwrap_or("UNKNOWN");
    let mut errors = Vec::new();

    if let Some(schema) = schemas.get(doc_type) {
        // Convert YAML front-matter to JSON for validation
        let json_value = serde_json::to_value(&doc.front_matter)
            .unwrap_or(Value::Null);

        for error in schema.iter_errors(&json_value) {
            errors.push(format!(
                "{}: {} (at {})",
                doc.id,
                error,
                error.instance_path
            ));
        }
    } else {
        errors.push(format!(
            "{}: No schema registered for document type '{}'",
            doc.id, doc_type
        ));
    }

    errors
}

/// Validate all documents and return a list of all validation errors
pub fn validate_all_documents(
    documents: &[Document],
    schemas: &SchemaMap,
) -> Vec<String> {
    let mut all_errors = Vec::new();

    for doc in documents {
        let errors = validate_document(doc, schemas);
        all_errors.extend(errors);
    }

    all_errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_schemas() {
        let schemas_dir = Path::new("schemas");
        if schemas_dir.exists() {
            let schemas = load_schemas(schemas_dir).unwrap();
            assert!(!schemas.is_empty(), "Should load at least some schemas");
        }
    }
}
