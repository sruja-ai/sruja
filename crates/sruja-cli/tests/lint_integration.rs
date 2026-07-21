mod common;
use common::{create_test_repo, run_sruja, write_file};

const MINIMAL_VALID_SRUJA: &str = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

User = person "User" {
  description "End user"
}

App = system "My App" {
  description "Main application"

  Web = container "Web" {
    technology "React"
    description "UI"
  }
}
User -> App "uses"
"#;

#[test]
fn lint_json_write_baseline_then_filters_existing_diagnostics() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "arch.sruja",
        r#"
system = kind "System"
container = kind "Container"

App = system "App" {
  description "App"
  Api = container "API" {
    technology "Rust"
  }
}
"#,
    );
    let file_path = repo.path().join("arch.sruja");
    let file_str = file_path.to_str().expect("utf-8");
    let baseline_path = repo.path().join("lint.baseline.json");
    let baseline_str = baseline_path.to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&[
        "lint",
        file_str,
        "--format",
        "json",
        "--write-baseline",
        baseline_str,
    ]);
    assert!(ok, "lint --write-baseline should succeed: stderr={stderr}");
    assert!(baseline_path.exists(), "baseline file must be written");
    let out: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(
        out.get("baseline").and_then(|v| v.as_str()),
        Some(baseline_str),
        "expected baseline path to be echoed: stdout={stdout}"
    );
    assert_eq!(
        out.get("error_count").and_then(|v| v.as_u64()),
        Some(0),
        "expected error_count to be reset to 0 when baseline is written: stdout={stdout}"
    );
    assert!(
        out.get("total_error_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 0
            || out
                .get("total_warning_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                > 0,
        "expected totals to be reported when baseline is written: stdout={stdout}"
    );

    let (ok, stdout, stderr) = run_sruja(&[
        "lint",
        file_str,
        "--format",
        "json",
        "--baseline",
        baseline_str,
    ]);
    assert!(ok, "lint --baseline should succeed: stderr={stderr}");
    let out: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(
        out.get("baseline").and_then(|v| v.as_str()),
        Some(baseline_str),
        "expected baseline to be echoed: stdout={stdout}"
    );
    assert_eq!(
        out.get("error_count").and_then(|v| v.as_u64()),
        Some(0),
        "expected errors to be filtered by baseline: stdout={stdout}"
    );
    assert_eq!(
        out.get("diagnostics")
            .and_then(|v| v.as_array())
            .map(|a| a.len()),
        Some(0),
        "expected diagnostics to be filtered by baseline: stdout={stdout}"
    );
}

#[test]
fn lint_json_parse_error_emits_json_and_exits_nonzero() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "broken.sruja",
        r#"
system = kind "System"
App = system "App" { description "App"
"#,
    );
    let file_path = repo.path().join("broken.sruja");
    let file_str = file_path.to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&["lint", file_str, "--format", "json"]);
    assert!(!ok, "lint should fail on parse error: stderr={stderr}");
    let out: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(
        out.get("ok").and_then(|v| v.as_bool()),
        Some(false),
        "expected ok=false on parse error: stdout={stdout}"
    );
    assert!(
        out.get("diagnostics")
            .and_then(|v| v.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false),
        "expected diagnostics on parse error: stdout={stdout}"
    );
}

#[test]
fn fmt_succeeds_on_valid_file() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", MINIMAL_VALID_SRUJA);
    let path_str = repo
        .path()
        .join("arch.sruja")
        .to_str()
        .expect("utf-8")
        .to_string();

    let (success, _stdout, stderr) = run_sruja(&["fmt", &path_str]);

    assert!(success, "fmt should succeed: stderr={}", stderr);
}

#[test]
fn fmt_check_exits_nonzero_when_changes_needed() {
    let repo = create_test_repo();
    // Intentionally badly formatted (extra spaces, no newline at end)
    write_file(
        repo.path(),
        "arch.sruja",
        "S   =   system \"S\" { description \"x\" }   ",
    );
    let path_str = repo
        .path()
        .join("arch.sruja")
        .to_str()
        .expect("utf-8")
        .to_string();

    let (success, _stdout, _stderr) = run_sruja(&["fmt", "--check", &path_str]);

    // --check exits with error if file would be reformatted
    assert!(!success, "fmt --check should fail when changes are needed");
}

#[test]
fn fmt_check_exits_zero_on_already_formatted_file() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "arch.sruja",
        r#"system = kind "System"
App = system "App" {
  description "Test"
}
"#,
    );
    let path_str = repo
        .path()
        .join("arch.sruja")
        .to_str()
        .expect("utf-8")
        .to_string();

    let (success, _stdout, _stderr) = run_sruja(&["fmt", "--check", &path_str]);

    assert!(
        success,
        "fmt --check should succeed on already formatted file"
    );
}

#[test]
fn validate_succeeds_on_valid_file() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", MINIMAL_VALID_SRUJA);
    let path_str = repo
        .path()
        .join("arch.sruja")
        .to_str()
        .expect("utf-8")
        .to_string();

    let (success, _stdout, stderr) = run_sruja(&["validate", &path_str]);

    assert!(success, "validate should succeed: stderr={}", stderr);
}

#[test]
fn validate_fails_on_invalid_dsl() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "arch.sruja",
        "This is not valid DSL syntax !!!",
    );
    let path_str = repo
        .path()
        .join("arch.sruja")
        .to_str()
        .expect("utf-8")
        .to_string();

    let (success, _stdout, _stderr) = run_sruja(&["validate", &path_str]);

    assert!(!success, "validate should fail on invalid DSL");
}
