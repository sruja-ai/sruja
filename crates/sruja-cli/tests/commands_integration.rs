//! Integration tests for CLI commands that lack dedicated E2E coverage.
//! Exercises: export, fmt, list, tree, validate, scan using temp repos and run_sruja.

mod common;
use common::{create_test_repo, run_sruja, write_file};

const MINIMAL_VALID_SRUJA: &str = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

User = person "User" { description "End user" }
App = system "My App" {
  description "Main application"
  Web = container "Web" { technology "React"; description "UI" }
}
User -> App "uses"
"#;

#[test]
fn export_json_succeeds_on_valid_file() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", MINIMAL_VALID_SRUJA);
    let path = repo.path().join("arch.sruja");
    let path_str = path.to_str().expect("utf-8");

    let (success, stdout, stderr) =
        run_sruja(&["export", "json", path_str]);

    assert!(success, "export json should succeed: stderr={}", stderr);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(parsed.get("elements").is_some(), "export should have elements");
}

#[test]
fn export_mermaid_succeeds_on_valid_file() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", MINIMAL_VALID_SRUJA);
    let path_str = repo.path().join("arch.sruja").to_str().expect("utf-8").to_string();

    let (success, stdout, stderr) =
        run_sruja(&["export", "mermaid", &path_str]);

    assert!(success, "export mermaid should succeed: stderr={}", stderr);
    assert!(
        stdout.contains("flowchart") || stdout.contains("graph"),
        "mermaid output should contain diagram"
    );
}

#[test]
fn fmt_succeeds_on_valid_file() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", MINIMAL_VALID_SRUJA);
    let path_str = repo.path().join("arch.sruja").to_str().expect("utf-8").to_string();

    let (success, _stdout, stderr) = run_sruja(&["fmt", &path_str]);

    assert!(success, "fmt should succeed: stderr={}", stderr);
}

#[test]
fn fmt_check_exits_nonzero_when_changes_needed() {
    let repo = create_test_repo();
    // Intentionally badly formatted (extra spaces, no newline at end)
    write_file(repo.path(), "arch.sruja", "S   =   system \"S\" { description \"x\" }   ");
    let path_str = repo.path().join("arch.sruja").to_str().expect("utf-8").to_string();

    let (success, _stdout, _stderr) = run_sruja(&["fmt", "--check", &path_str]);

    // --check exits with error if file would be reformatted
    assert!(!success, "fmt --check should fail when changes are needed");
}

#[test]
fn list_succeeds_on_valid_file() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", MINIMAL_VALID_SRUJA);
    let path_str = repo.path().join("arch.sruja").to_str().expect("utf-8").to_string();

    let (success, stdout, stderr) = run_sruja(&["list", &path_str]);

    assert!(success, "list should succeed: stderr={}", stderr);
    assert!(stdout.contains("User") || stdout.contains("App"), "list should show elements");
}

#[test]
fn tree_succeeds_on_valid_file() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", MINIMAL_VALID_SRUJA);
    let path_str = repo.path().join("arch.sruja").to_str().expect("utf-8").to_string();

    let (success, stdout, stderr) = run_sruja(&["tree", &path_str]);

    assert!(success, "tree should succeed: stderr={}", stderr);
    assert!(!stdout.is_empty(), "tree should produce output");
}

#[test]
fn validate_succeeds_on_valid_file() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", MINIMAL_VALID_SRUJA);
    let path_str = repo.path().join("arch.sruja").to_str().expect("utf-8").to_string();

    let (success, _stdout, stderr) = run_sruja(&["validate", &path_str]);

    assert!(success, "validate should succeed: stderr={}", stderr);
}

#[test]
fn scan_succeeds_on_repo_with_cargo_toml() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "Cargo.toml",
        r#"[package]
name = "dummy"
version = "0.1.0"
edition = "2021"
"#,
    );
    write_file(repo.path(), "src/lib.rs", "pub fn foo() {}");
    let path_str = repo.path().to_str().expect("utf-8");

    let (success, stdout, stderr) =
        run_sruja(&["scan", path_str, "--output", "-"]);

    assert!(success, "scan should succeed on Cargo repo: stderr={}", stderr);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("scan outputs JSON");
    assert!(
        parsed.get("nodes").is_some() || parsed.get("elements").is_some()
            || stdout.contains("\"nodes\""),
        "scan output should contain graph structure"
    );
}

#[test]
fn version_prints_version() {
    let (success, stdout, stderr) = run_sruja(&["version"]);

    assert!(success, "version should succeed: stderr={}", stderr);
    assert!(!stdout.is_empty(), "version should print something");
}
