//! E2E tests for `sruja drift --ci` (CI-focused drift check).

mod common;
use common::*;

#[test]
fn drift_ci_exits_zero_json() {
    let repo = create_test_repo();
    write_file(repo.path(), "lib.js", "export function x() {}");
    let (ok, stdout, _stderr) = run_sruja(&[
        "drift",
        "-r",
        repo.path().to_str().unwrap(),
        "-f",
        "json",
        "--ci",
    ]);
    assert!(ok, "sruja drift --ci must exit 0");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("JSON output");
    assert!(parsed.get("truth_status").is_some());
    assert!(parsed.get("violations_count").is_some());
}

#[test]
fn drift_ci_exits_zero_text() {
    let repo = create_test_repo();
    write_file(repo.path(), "lib.js", "export function x() {}");
    // Note: --ci overrides text format to github-actions
    let (ok, stdout, _stderr) = run_sruja(&[
        "drift",
        "-r",
        repo.path().to_str().unwrap(),
        "-f",
        "text",
        "--ci",
    ]);
    assert!(ok, "sruja drift --ci must exit 0");
    assert!(
        stdout.contains("::notice") || stdout.contains("Truth status:"),
        "output should contain CI annotation or truth status"
    );
}

#[test]
fn drift_ci_exits_zero_github_actions() {
    let repo = create_test_repo();
    write_file(repo.path(), "lib.js", "export function x() {}");
    let (ok, stdout, _stderr) = run_sruja(&[
        "drift",
        "-r",
        repo.path().to_str().unwrap(),
        "-f",
        "github-actions",
        "--ci",
    ]);
    assert!(ok, "sruja drift --ci must exit 0");
    assert!(
        stdout.contains("::notice") || stdout.contains("Sruja"),
        "github-actions output should contain annotation or Sruja"
    );
}
