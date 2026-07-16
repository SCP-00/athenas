---
id: COMPILER-0001
title: Athenas Compiler Architecture
author: Chief Software Architect
date: 2026-07-16
status: Draft
version: 0.1.0
authority: Level 3 — Specification
tags:
  - compiler
  - architecture
  - pipeline
  - core
---

# COMPILER-0001 — Athenas Compiler Architecture

> **Authority:** Level 3 (Specification)
> **Status:** Draft

---

## Overview

The Athenas Compiler (`ath`) is the heart of the project. It does not compile code — it **compiles knowledge**. Markdown documents with YAML front-matter are the source language. The compiler transforms them through a series of passes into structured knowledge artifacts: a Knowledge Graph, Search Index, Cross-References, Timeline, Ontology, and Diagnostic Reports.

This document specifies the compiler's internal architecture: its pipeline, passes, intermediate representation, and output model.

---

## Pipeline Architecture

```
Markdown Files
      │
      ▼
   Pass 1: Reader
      │
      ▼
   RawDocument[]
      │
      ▼
   Pass 2: Parser
      │
      ▼
   DocumentAST[]
      │
      ▼
   Pass 3: Schema Validator
      │
      ▼
   ValidatedDocument[]
      │
      ▼
   Pass 4: ID Resolver
      │
      ▼
   ResolvedGraph
      │
      ▼
   Pass 5: Reference Resolver
      │
      ▼
   ConnectedGraph
      │
      ▼
   Pass 6: Semantic Validator
      │
      ▼
   KnowledgeGraph
      │
      ▼
   Pass 7: Artifact Generator
      │
      ▼
   graph.json, index.json, references.json,
   timeline.json, decisions.json,
   diagnostics.json, ontology.json, summary.json
```

---

## Intermediate Representation (IR)

The compiler uses a multi-stage IR that evolves through the pipeline:

### Stage 1: RawDocument
```rust
struct RawDocument {
    path: PathBuf,       // File path relative to root
    raw_content: String,  // Raw file content
    hash: Sha256,        // Content hash for change detection
}
```

### Stage 2: DocumentAST
```rust
struct DocumentAST {
    id: String,                    // Document ID (e.g., "SPEC-0008")
    front_matter: YamlValue,       // Parsed YAML front-matter
    body_ast: MarkdownAst,         // Parsed markdown body (headings, paragraphs, tables, links)
    metadata: DocumentMetadata,     // Extracted metadata
    source: RawDocument,            // Link to source
}
```

### Stage 3: ValidatedDocument
```rust
struct ValidatedDocument {
    ast: DocumentAST,              // The parsed document
    schema_errors: Vec<SchemaError>, // Schema validation errors
    is_valid: bool,                 // Whether the document passes schema validation
}
```

### Stage 4: GraphNode
```rust
struct GraphNode {
    id: String,                     // Document ID
    node_type: NodeType,            // CONST, REQ, SPEC, ARCH, etc.
    title: String,
    status: DocumentStatus,         // Draft, Review, Approved, etc.
    authority_level: u8,            // 0-8 per the Constitution
    hash: String,                   // Content hash
    tags: Vec<String>,
    relationships: Vec<Relationship>, // Outgoing edges
}
```

### Stage 5: KnowledgeGraph
```rust
struct KnowledgeGraph {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    metadata: GraphMetadata,  // Counts, types, generated_at
}
```

---

## Pass Specifications

### Pass 1: Reader
**Input:** File system
**Output:** `Vec<RawDocument>`
**Behavior:** Walks the repository directory tree, finds all `.md` files, reads them into memory. Skips `templates/`, `node_modules/`, `.git/`, `.knowledge/`, `target/`.
**Configuration:** Accepts include/exclude path patterns.

### Pass 2: Parser
**Input:** `Vec<RawDocument>`
**Output:** `Vec<DocumentAST>`
**Behavior:** 
- Splits each document at the `---` front-matter delimiters
- Parses YAML front-matter into a structured value
- Parses markdown body into an AST (headings, paragraphs, tables, code blocks, lists, links)
- Extracts inline references (`(see REQ-0001)`, `[BENCH-0007](link)`)
- Computes the content SHA256 hash
- **Error handling:** Malformed YAML → error with file path and line number. Missing front-matter → warning, document skipped.

### Pass 3: Schema Validator
**Input:** `Vec<DocumentAST>`
**Output:** `Vec<ValidatedDocument>`
**Behavior:**
- Loads JSON Schema files from `schemas/` directory
- Matches each document by type prefix (CONST → const.schema.json, REQ → req.schema.json)
- Validates YAML front-matter against the corresponding schema
- Collects all validation errors per document
- **Error handling:** Unknown document type → warning. Schema not found → warning. Validation failure → error with field path.

### Pass 4: ID Resolver
**Input:** `Vec<ValidatedDocument>`
**Output:** `ResolvedGraph`
**Behavior:**
- Collects all document IDs into a global registry
- Detects duplicate IDs → error
- Detects ID pattern violations (e.g., `ADR-NNNN` instead of `ADR-0000`) → warning
- Builds an initial node list with authority levels

### Pass 5: Reference Resolver
**Input:** `ResolvedGraph`
**Output:** `ConnectedGraph`
**Behavior:**
- Iterates all documents' `implements`, `depends_on`, `validated_by`, `derived_from`, `supersedes`, `related` fields
- Resolves each reference against the global ID registry
- Creates directed edges between nodes
- Detects broken references (referenced ID not found) → error
- Detects circular references → warning

### Pass 6: Semantic Validator
**Input:** `ConnectedGraph`
**Output:** `KnowledgeGraph`
**Behavior:**
- Validates authority level consistency (lower level documents cannot depend on higher level ones)
- Validates status lifecycle transitions (a SPEC cannot be Approved before its REQ is Approved)
- Detects orphan documents (no relationships at all) → warning
- Computes document health metrics
- **Error handling:** Authority violation → error. Status violation → warning.

### Pass 7: Artifact Generator
**Input:** `KnowledgeGraph`
**Output:** JSON files
**Behavior:**
- Generates `graph.json` — complete knowledge graph
- Generates `index.json` — search index with snippets
- Generates `references.json` — cross-reference table
- Generates `timeline.json` — chronological document history
- Generates `decisions.json` — extracted decision log entries
- Generates `diagnostics.json` — all errors and warnings
- Generates `ontology.json` — parsed ontology from `knowledge/ontology.yaml`
- Generates `summary.json` — aggregate statistics

---

## Output Schema

All generated JSON files follow consistent schemas for consumption by the Astro dashboard, CLI queries, and external tools.

### graph.json
```json
{
  "nodes": [{ "id": "SPEC-0008", "doc_type": "SPEC", "title": "...", ... }],
  "edges": [{ "source": "SPEC-0008", "target": "REQ-0003", "relationship": "implements" }],
  "metadata": { "total_nodes": 42, "total_edges": 87, ... }
}
```

### summary.json
```json
{
  "total_documents": 42,
  "total_relationships": 87,
  "document_types": { "CONST": 1, "REQ": 3, "SPEC": 5 },
  "warnings": 2,
  "errors": 0,
  "health_score": 94.2
}
```

---

## CLI Interface

```bash
ath build         # Run full pipeline (Pass 1-7)
ath validate      # Run Pass 1-3 only (fast validation)
ath graph         # Run Pass 1-6, output knowledge graph
ath doctor        # Run all passes, show diagnostics
ath search        # Run Pass 1-7, search the index
ath stats         # Show aggregate statistics
ath check         # Run Pass 1-6, exit with error code if issues found
```

---

## Future Passes

These passes are planned but not yet implemented:

- **Pass 8: HTML Renderer** — Generate HTML pages from the knowledge graph
- **Pass 9: Search Engine** — Build a full-text search index with BM25 or embeddings
- **Pass 10: PDF Generator** — Generate printable documentation
- **Pass 11: LLM Context Pack** — Package knowledge for LLM consumption
- **Pass 12: Graph Database Export** — Export to Neo4j or similar

---

## Design Principles

1. **Each pass is independent.** Passes communicate through well-defined IR types. Any pass can be replaced or modified without affecting others.
2. **Errors are data.** The compiler never crashes on malformed input. Validation errors become diagnostic entries in the output.
3. **Everything is a graph.** The central data structure is the Knowledge Graph. All outputs are derived from it.
4. **Reproducibility.** The same input always produces the same output. Content hashes enable change detection.
5. **Graceful degradation.** The compiler works with partial or incomplete data. Missing schemas, broken references, and invalid documents produce diagnostics but never block the pipeline.

---

## Decision Log

| Date | Decision | Rationale | Alternatives |
|------|----------|-----------|--------------|
| 2026-07-16 | 7-pass pipeline architecture | Modular, testable, extensible | Monolithic parser | 
| 2026-07-16 | JSON as primary output | Universal interoperability | HTML-first |
| 2026-07-16 | Error-as-data philosophy | Compiler never crashes on bad input | Fail-fast |
