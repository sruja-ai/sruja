//! E2E tests for `sruja check` (CI-focused drift check, always exit 0).

mod common;
use common::*;

#[test]
fn check_exits_zero_json() {
    let repo = create_test_repo();
    write_file(repo.path(), "lib.js", "export function x() {}");
    let (ok, stdout, _stderr) =
        run_sruja(&["check", "-r", repo.path().to_str().unwrap(), "-f", "json"]);
    assert!(ok, "sruja check must exit 0");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("JSON output");
    assert!(parsed.get("truth_status").is_some());
    assert!(parsed.get("violations_count").is_some());
}

#[test]
fn check_exits_zero_text() {
    let repo = create_test_repo();
    write_file(repo.path(), "lib.js", "export function x() {}");
    let (ok, stdout, _stderr) =
        run_sruja(&["check", "-r", repo.path().to_str().unwrap(), "-f", "text"]);
    assert!(ok, "sruja check must exit 0");
    assert!(
        stdout.contains("Truth:"),
        "text output should contain Truth:"
    );
}

#[test]
fn check_exits_zero_github_actions() {
    let repo = create_test_repo();
    write_file(repo.path(), "lib.js", "export function x() {}");
    let (ok, stdout, _stderr) = run_sruja(&[
        "check",
        "-r",
        repo.path().to_str().unwrap(),
        "-f",
        "github-actions",
    ]);
    assert!(ok, "sruja check must exit 0");
    assert!(
        stdout.contains("::notice") || stdout.contains("Sruja"),
        "github-actions output should contain annotation or Sruja"
    );
}
