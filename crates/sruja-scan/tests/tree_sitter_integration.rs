//! Integration tests for tree-sitter scanning: temp repo with minimal source files.
//! Exercises detector and language parsers (e.g. Rust) without relying on the full workspace.

use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn write_file(dir: &Path, path: &str, content: &str) {
    let full = dir.join(path);
    if let Some(p) = full.parent() {
        fs::create_dir_all(p).expect("create dir");
    }
    fs::write(&full, content).expect("write file");
}

#[test]
fn scan_minimal_rust_repo_produces_graph() {
    let repo = TempDir::new().expect("temp dir");
    write_file(
        repo.path(),
        "Cargo.toml",
        r#"[package]
name = "fixture"
version = "0.1.0"
edition = "2021"
"#,
    );
    write_file(
        repo.path(),
        "src/lib.rs",
        r#"pub fn foo() -> i32 { 42 }
pub mod bar { pub fn baz() {} }
"#,
    );

    let graph = sruja_scan::scan_repo(repo.path()).expect("scan should succeed");

    // Tree-sitter or manifest-based scan should yield at least one node for a Cargo repo
    assert!(
        !graph.nodes.is_empty(),
        "minimal Rust repo should produce at least one node"
    );
}
