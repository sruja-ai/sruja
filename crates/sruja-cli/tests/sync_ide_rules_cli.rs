//! Integration tests for `sruja sync-ide-rules`.

mod common;
use common::{create_test_repo, run_sruja, write_file};

fn write_minimal_cargo_repo(repo_root: &std::path::Path) {
    write_file(
        repo_root,
        "Cargo.toml",
        r#"[package]
name = "fixture"
version = "0.1.0"
edition = "2021"
"#,
    );
    write_file(repo_root, "src/lib.rs", "pub fn api() -> i32 { 42 }");
}

#[test]
fn sync_ide_rules_writes_cursor_and_llms_files() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (ok, _stdout, stderr) = run_sruja(&["sync-ide-rules", "-r", repo_str]);
    assert!(ok, "sync-ide-rules should succeed: {stderr}");

    let cursor_rules = repo.path().join(".cursorrules");
    let llms = repo.path().join("llms-architecture.txt");
    assert!(cursor_rules.exists(), ".cursorrules should be created");
    assert!(llms.exists(), "llms-architecture.txt should be created");

    let rules = std::fs::read_to_string(&cursor_rules).expect("read .cursorrules");
    assert!(
        rules.contains("Sruja Architecture Context") || rules.contains("Architecture"),
        "cursor rules should include architecture header"
    );
}

#[test]
fn sync_ide_rules_check_passes_after_sync() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (sync_ok, _, sync_err) = run_sruja(&["sync-ide-rules", "-r", repo_str]);
    assert!(sync_ok, "initial sync failed: {sync_err}");

    let (check_ok, _, check_err) = run_sruja(&["sync-ide-rules", "-r", repo_str, "--check"]);
    assert!(
        check_ok,
        "sync-ide-rules --check should pass when files match: {check_err}"
    );
}
