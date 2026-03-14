//! E2E test that locks in the demo "happy path" intent drift story.
//!
//! The goal is not to validate every line of output, but to ensure the demo
//! continues to surface the key drift: direct frontend → database access.

mod common;
use common::*;

use std::path::PathBuf;

fn demo_dir() -> PathBuf {
    // crates/sruja-cli/tests -> crates/sruja-cli -> crates -> repo root -> demo
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("demo")
}

#[test]
fn demo_intent_check_highlights_frontend_db_access() {
    let demo = demo_dir();
    if !demo.exists() {
        eprintln!(
            "Skipping demo_intel_e2e: demo dir not found at {}",
            demo.display()
        );
        return;
    }

    let demo_str = demo.to_str().expect("demo path should be utf-8");
    let (success, stdout, stderr) = run_sruja(&[
        "intent", "check", "-r", demo_str, "-i", demo_str, "-f", "markdown",
    ]);

    assert!(success, "intent check should succeed: stderr={}", stderr);
    assert!(
        stdout.contains("frontend_py' -> 'database"),
        "Output should contain the demo's key drift (frontend -> database).\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
}
