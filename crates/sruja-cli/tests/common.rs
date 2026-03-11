//! Shared helpers for sruja-cli e2e tests.
//!
//! Include in a test file with: `mod common; use common::*;`

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Create a temporary directory for a test repo.
pub fn create_test_repo() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

/// Write a file at `dir/name` with `content`; creates parent dirs if needed.
pub fn write_file(dir: &Path, name: &str, content: &str) {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(path, content).expect("Failed to write file");
}

/// Run the sruja binary with the given args. Returns (success, stdout, stderr).
pub fn run_sruja(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_sruja"))
        .args(args)
        .output()
        .expect("Failed to run sruja");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.success(), stdout, stderr)
}
