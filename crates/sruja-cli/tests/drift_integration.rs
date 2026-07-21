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
fn drift_emits_text_json_and_github_actions_reports() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(repo.path(), "repo.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");
    let arch_str = repo
        .path()
        .join("repo.sruja")
        .to_str()
        .expect("utf-8")
        .to_string();

    let (_ok_text, text_out, _text_err) =
        run_sruja(&["drift", "-r", repo_str, "-a", &arch_str, "-f", "text"]);
    assert!(
        text_out.contains("Baseline Drift"),
        "expected drift text report to include header: stdout={text_out}"
    );

    let (_ok_json, json_out, _json_err) =
        run_sruja(&["drift", "-r", repo_str, "-a", &arch_str, "-f", "json"]);
    let _parsed: serde_json::Value = serde_json::from_str(json_out.trim()).expect("valid JSON");

    let (_ok_ga, ga_out, _ga_err) = run_sruja(&[
        "drift",
        "-r",
        repo_str,
        "-a",
        &arch_str,
        "-f",
        "github-actions",
    ]);
    assert!(
        ga_out.contains("::error") || ga_out.contains("::warning") || ga_out.contains("::notice"),
        "expected github-actions report annotations: stdout={ga_out}"
    );
}

#[test]
fn baseline_and_drift_ci_json_succeeds() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let baseline_out = repo
        .path()
        .join(".sruja/violations.baseline.json")
        .to_str()
        .expect("utf-8")
        .to_string();

    let (baseline_success, baseline_stdout, baseline_stderr) =
        run_sruja(&["baseline", "-r", repo_str, "-o", &baseline_out]);
    assert!(
        baseline_success,
        "baseline should succeed: stderr={}",
        baseline_stderr
    );
    let baseline_path_printed = baseline_stdout.trim();
    assert!(!baseline_path_printed.is_empty());
    assert!(std::path::Path::new(baseline_path_printed).exists());

    let (check_success, check_stdout, check_stderr) = run_sruja(&[
        "drift",
        "-r",
        repo_str,
        "-f",
        "json",
        "--ci",
        "--baseline",
        baseline_path_printed,
    ]);
    assert!(
        check_success,
        "drift --ci should succeed: stderr={}",
        check_stderr
    );
    let parsed: serde_json::Value = serde_json::from_str(check_stdout.trim()).expect("valid JSON");
    assert!(parsed.get("truth_status").is_some());
    assert!(parsed.get("violations_baseline").is_some());
}

#[test]
fn compliance_report_json_succeeds() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");
    let arch_str = repo
        .path()
        .join("arch.sruja")
        .to_str()
        .expect("utf-8")
        .to_string();

    let (success, stdout, _stderr) =
        run_sruja(&["compliance", "-r", repo_str, "-a", &arch_str, "-f", "json"]);

    assert!(
        !success,
        "compliance should report non-compliance for empty repo vs architecture"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("valid JSON even on failure");
    assert!(parsed.get("status").is_some());
    assert!(parsed.get("health_score").is_some());
}
