use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture_root() -> PathBuf {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    crate_dir.join("tests").join("fixtures")
}

fn ath_binary() -> PathBuf {
    if let Ok(path) = std::env::var("ATH_BIN") {
        return PathBuf::from(path);
    }
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let debug_path = crate_dir.join("target").join("debug").join("ath");
    if debug_path.exists() {
        return debug_path;
    }
    let release_path = crate_dir.join("target").join("release").join("ath");
    if release_path.exists() {
        return release_path;
    }
    let status = std::process::Command::new("cargo")
        .args(["build", "--manifest-path", &crate_dir.join("Cargo.toml").to_string_lossy()])
        .status()
        .expect("Failed to run cargo build");
    assert!(status.success(), "cargo build should succeed");
    assert!(debug_path.exists(), "Binary should exist after build: {:?}", debug_path);
    debug_path
}

#[test]
fn test_validate_passes_on_fixtures() {
    let root = fixture_root();
    let ath = ath_binary();
    let schemas = root.join("schemas");

    let output = Command::new(&ath)
        .arg("validate").arg(&root).arg("--schemas").arg(&schemas)
        .output()
        .expect("Failed to run ath validate");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("stdout:\n{}", stdout);

    assert!(output.status.success(), "ath validate should pass on fixtures");
    assert!(stdout.contains("Validation PASSED"), "Should report PASSED");
    assert!(stdout.contains("Errors: 0"), "Should have 0 errors");
}

#[test]
fn test_graph_output() {
    let root = fixture_root();
    let ath = ath_binary();
    let output_dir = root.join("output");
    let _ = std::fs::remove_dir_all(&output_dir);

    let output = Command::new(&ath)
        .arg("graph").arg(&root).arg("--output").arg(&output_dir)
        .output()
        .expect("Failed to run ath graph");
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("stdout:\n{}", stdout);

    assert!(output.status.success(), "ath graph should pass");

    let graph_path = output_dir.join("graph.json");
    assert!(graph_path.exists(), "graph.json should exist");

    let content = std::fs::read_to_string(&graph_path).unwrap();
    let graph: serde_json::Value = serde_json::from_str(&content).unwrap();
    let nodes = graph.get("nodes").and_then(|v| v.as_array()).unwrap();
    assert!(nodes.len() >= 3, "At least 3 nodes");
    let edges = graph.get("edges").and_then(|v| v.as_array()).unwrap();
    assert!(!edges.is_empty(), "Should have edges");

    let _ = std::fs::remove_dir_all(&output_dir);
}

#[test]
fn test_build_output() {
    let root = fixture_root();
    let ath = ath_binary();
    let output_dir = root.join("output");
    let _ = std::fs::remove_dir_all(&output_dir);

    let output = Command::new(&ath)
        .arg("build").arg(&root).arg("--output").arg(&output_dir)
        .arg("--schemas").arg(root.join("schemas"))
        .output()
        .expect("Failed to run ath build");
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("stdout:\n{}", stdout);

    assert!(output.status.success(), "ath build should pass");

    for filename in &["graph.json", "index.json", "timeline.json", "decisions.json",
                       "diagnostics.json", "references.json", "summary.json"] {
        let fp = output_dir.join(filename);
        assert!(fp.exists(), "{} should exist", filename);
        assert!(fp.metadata().unwrap().len() > 0, "{} should be non-empty", filename);
    }

    let summary: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(output_dir.join("summary.json")).unwrap()
    ).unwrap();
    assert!(summary.get("total_documents").is_some());
    assert!(summary.get("total_relationships").is_some());

    let _ = std::fs::remove_dir_all(&output_dir);
}

#[test]
fn test_validate_detects_broken_refs() {
    let tmp = std::env::temp_dir().join("ath-test-broken-refs");
    let _ = std::fs::create_dir_all(&tmp);
    let _ = std::fs::create_dir_all(tmp.join("schemas"));
    let _ = std::fs::create_dir_all(tmp.join("documents"));

    // Write schema with properly escaped regex pattern (digits class \d)
    let schema = std::fs::read_to_string(fixture_root().join("schemas").join("const.schema.json"))
        .expect("Should read const.schema.json");
    std::fs::write(tmp.join("schemas").join("const.schema.json"), &schema)
        .expect("Should write schema");

    // Create document with broken references
    let doc = "---
id: CONST-0001
title: Broken Ref Test
author: Test
date: 2026-01-01
status: Draft
version: 1.0.0
authority: Level 0 - Constitution
depends_on:
  - NONEXIST-0001
related:
  - ANOTHER-MISSING-0002
---
# Broken Ref Test
";
    std::fs::write(tmp.join("documents").join("CONST-0001.md"), doc)
        .expect("Should write doc");

    let ath = ath_binary();
    let output = Command::new(&ath)
        .arg("validate").arg(&tmp).arg("--schemas").arg(tmp.join("schemas"))
        .output()
        .expect("Failed to run ath validate");
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("stdout:\n{}", stdout);

    assert!(!output.status.success(), "Validation should fail on broken refs");
    assert!(stdout.contains("NONEXIST-0001"), "Should detect NONEXIST-0001");
    assert!(stdout.contains("ANOTHER-MISSING-0002"), "Should detect ANOTHER-MISSING-0002");

    let _ = std::fs::remove_dir_all(&tmp);
}
