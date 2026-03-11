//! E2E tests for `sruja lint` command.

mod common;
use common::*;

#[test]
fn lint_valid_file_succeeds() {
    let repo = create_test_repo();
    // Minimal valid .sruja: person, system, containers, descriptions, relationships
    write_file(
        repo.path(),
        "architecture.sruja",
        r#"person = kind "Person"
system = kind "System"
container = kind "Container"
database = kind "Database"

User = person "User" { description "End user" }
App = system "My App" {
  description "Main application"
  Web = container "Web" { technology "React"; description "UI" }
  Api = container "API" { technology "Node.js"; description "REST API" }
  Web -> Api "HTTPS"
}
User -> App "uses"
"#,
    );

    let path = repo.path().join("architecture.sruja");
    let path_str = path.to_str().expect("temp path is UTF-8");
    let (success, _stdout, stderr) = run_sruja(&["lint", path_str]);

    assert!(
        success,
        "lint should succeed on valid file: stderr={}",
        stderr
    );
}

#[test]
fn lint_invalid_file_fails() {
    let repo = create_test_repo();
    // Invalid: missing description on person, and broken relationship (no such target)
    write_file(
        repo.path(),
        "bad.sruja",
        r#"person = kind "Person"
system = kind "System"

User = person "User"
App = system "App" { description "App" }
User -> NonExistent "uses"
"#,
    );

    let path = repo.path().join("bad.sruja");
    let path_str = path.to_str().expect("temp path is UTF-8");
    let (success, _stdout, stderr) = run_sruja(&["lint", path_str]);

    assert!(
        !success,
        "lint should fail on invalid file: stderr={}",
        stderr
    );
    assert!(
        stderr.contains("error") || stderr.contains("Error") || stderr.contains("E"),
        "stderr should mention an error: stderr={}",
        stderr
    );
}

#[test]
fn lint_nonexistent_file_fails() {
    let repo = create_test_repo();
    let path = repo.path().join("does_not_exist.sruja");
    let path_str = path.to_str().expect("temp path is UTF-8");

    let (success, _stdout, stderr) = run_sruja(&["lint", path_str]);

    assert!(
        !success,
        "lint should fail when file does not exist: stderr={}",
        stderr
    );
}
