//! Integration tests targeting CLI paths with low coverage.
//! Each test exercises a real command flow and asserts on structured output.

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

const VALID_STATE_MACHINE_SRUJA: &str = r#"
system = kind "System"
component = kind "Component"

App = system "App" {
  description "App"

  Svc = component "Service" {
    description "Service with lifecycle"
    state_machine "Lifecycle" {
      initial "Created"
      terminal ["Done"]
      "Created" -> "Done" on "finish"
    }
  }
}
"#;

fn write_minimal_cargo_repo(repo_root: &std::path::Path) {
    write_file(
        repo_root,
        "Cargo.toml",
        r#"[package]
name = "dummy"
version = "0.1.0"
edition = "2021"
"#,
    );
    write_file(repo_root, "src/lib.rs", "pub fn foo() {}");
}

#[test]
fn why_json_after_sync_includes_answer() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");
    run_sruja(&["sync", "-r", repo_str]);

    let (ok, stdout, stderr) = run_sruja(&[
        "why",
        "how do components depend?",
        "-r",
        repo_str,
        "-f",
        "json",
    ]);
    assert!(ok, "why should succeed: stderr={stderr}");
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(parsed.get("answer").is_some());
}

#[test]
fn index_registry_json_lists_discovered_artifacts() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(
        repo.path(),
        "docker-compose.yml",
        "services:\n  api:\n    image: nginx\n",
    );
    let repo_str = repo.path().to_str().expect("utf-8");

    let (ok, stdout, stderr) =
        run_sruja(&["index", "registry", "-r", repo_str, "--format", "json"]);
    assert!(ok, "index registry json should succeed: stderr={stderr}");
    let json_start = stdout.find('[').expect("JSON array in stdout");
    let parsed: serde_json::Value =
        serde_json::from_str(stdout[json_start..].trim()).expect("valid JSON");
    assert!(parsed.is_array(), "registry index returns artifact array");
    assert!(!parsed.as_array().unwrap().is_empty());
}

#[test]
fn compliance_text_format_includes_status_line() {
    let repo = create_test_repo();
    write_file(repo.path(), "repo.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");

    let (_ok, stdout, stderr) = run_sruja(&["compliance", "-r", repo_str, "-f", "text"]);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("Compliance")
            || combined.contains("compliance")
            || combined.contains("status"),
        "expected compliance text output: {combined}"
    );
}

#[test]
fn review_text_includes_truth_status() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(repo.path(), "repo.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");
    run_sruja(&["init", "--auto", "-r", repo_str]);

    let (ok, stdout, stderr) = run_sruja(&["review", "-r", repo_str, "-f", "text"]);
    assert!(ok, "review text should succeed: stderr={stderr}");
    assert!(
        stdout.contains("truth") || stdout.contains("Truth") || stdout.contains("drift"),
        "review text should mention truth/drift: {stdout}"
    );
}

#[test]
fn classify_writes_classification_json() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (ok, _stdout, stderr) = run_sruja(&["classify", "-r", repo_str]);
    assert!(ok, "classify should succeed: stderr={stderr}");
    let cls_path = repo.path().join(".sruja/classification.json");
    assert!(cls_path.exists(), "classification.json must be written");
    let content = std::fs::read_to_string(cls_path).expect("read classification");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    assert!(parsed.get("layers").is_some());
}

#[test]
fn intent_check_json_on_minimal_repo() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(repo.path(), "repo.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&["intent", "check", "-r", repo_str, "-f", "json"]);
    assert!(ok, "intent check should succeed: stderr={stderr}");
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(parsed.is_object());
}

#[test]
fn lint_valid_state_machine_is_clean() {
    let repo = create_test_repo();
    write_file(repo.path(), "sm.sruja", VALID_STATE_MACHINE_SRUJA);
    let path = repo
        .path()
        .join("sm.sruja")
        .to_str()
        .expect("utf-8")
        .to_string();

    let (ok, stdout, stderr) = run_sruja(&["lint", &path, "--format", "json"]);
    assert!(
        ok,
        "valid state machine should lint clean: stderr={stderr} stdout={stdout}"
    );
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    let err_count = parsed
        .get("error_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    assert_eq!(err_count, 0, "expected no errors: {parsed}");
}

#[test]
fn decision_new_and_list_roundtrip() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");
    run_sruja(&["init", "--auto", "-r", repo_str]);

    let (create_ok, create_out, create_err) = run_sruja(&[
        "decision",
        "new",
        "-r",
        repo_str,
        "--title",
        "Use Postgres",
        "--typ",
        "architecture",
    ]);
    assert!(create_ok, "decision new failed: {create_err}");
    let decisions_dir = repo.path().join(".sruja/decisions");
    assert!(
        decisions_dir.exists(),
        "decisions directory must exist after new: {create_out}{create_err}"
    );
    let count = std::fs::read_dir(&decisions_dir)
        .map(|d| d.count())
        .unwrap_or(0);
    assert!(count > 0, "expected a decision file in {decisions_dir:?}");

    let (list_ok, list_out, list_err) =
        run_sruja(&["decision", "list", "-r", repo_str, "-f", "json"]);
    assert!(list_ok, "decision list failed: {list_err}");
    let listed: serde_json::Value = serde_json::from_str(list_out.trim()).expect("list JSON");
    let records = listed.as_array().cloned().unwrap_or_default();
    assert!(!records.is_empty(), "expected at least one decision record");
}

#[test]
fn health_text_reports_score_after_init() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");
    run_sruja(&["init", "--auto", "-r", repo_str]);

    let (ok, stdout, stderr) = run_sruja(&["health", "-r", repo_str, "-f", "text"]);
    assert!(ok, "health text should succeed: stderr={stderr}");
    assert!(
        stdout.contains("Health") || stdout.contains("score") || stdout.contains("Score"),
        "health text should mention score: {stdout}"
    );
}

#[test]
fn drift_json_includes_remediation_playbook_fields() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(repo.path(), "repo.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&["drift", "-r", repo_str, "-f", "json"]);
    assert!(ok, "drift json should succeed: stderr={stderr}");
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(parsed.get("violations").is_some() || parsed.get("health_score").is_some());
}

#[test]
fn sync_ide_rules_check_after_sync() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (sync_ok, _, sync_err) = run_sruja(&["sync-ide-rules", "-r", repo_str]);
    assert!(sync_ok, "sync-ide-rules should succeed: {sync_err}");
    assert!(repo.path().join(".cursorrules").exists());

    let (check_ok, _, check_err) = run_sruja(&["sync-ide-rules", "-r", repo_str, "--check"]);
    assert!(
        check_ok,
        "sync-ide-rules --check should pass after sync: {check_err}"
    );
}
