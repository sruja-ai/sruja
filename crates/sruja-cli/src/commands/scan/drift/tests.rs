use std::process::Command;
use std::sync::Mutex;

static DRIFT_PR_LOCK: Mutex<()> = Mutex::new(());

fn git(repo: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(repo)
        .status()
        .expect("git command failed to spawn");
    assert!(status.success(), "git {:?} failed", args);
}

fn write_file(repo: &std::path::Path, rel: &str, content: &str) {
    let path = repo.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create dir");
    }
    std::fs::write(&path, content).expect("write file");
}

fn init_git_repo(repo: &std::path::Path) {
    git(repo, &["init"]);
    git(repo, &["config", "user.email", "tests@sruja.local"]);
    git(repo, &["config", "user.name", "Sruja Tests"]);
}

#[tokio::test]
async fn drift_pr_runs_with_cache_and_worktree_paths() {
    let _guard = DRIFT_PR_LOCK.lock().unwrap();

    let dir = tempfile::tempdir().expect("tempdir");
    let repo = dir.path();
    init_git_repo(repo);
    write_file(
        repo,
        "Cargo.toml",
        r#"[package]
name = "fixture"
version = "0.1.0"
edition = "2021"
"#,
    );
    write_file(repo, "src/lib.rs", "pub fn a() -> u32 { 1 }\n");
    git(repo, &["add", "."]);
    git(repo, &["commit", "-m", "initial"]);

    write_file(repo, "src/lib.rs", "pub fn a() -> u32 { 1 }\n\n");
    git(repo, &["add", "."]);
    git(repo, &["commit", "-m", "touch"]);

    let repo_str = repo.to_str().expect("utf-8");
    super::drift_pr(repo_str, Some("HEAD~1"), Some("HEAD"), "json")
        .await
        .expect("drift_pr json");
    super::drift_pr(repo_str, Some("HEAD~1"), Some("HEAD"), "github-actions")
        .await
        .expect("drift_pr github-actions");
    super::drift_pr(repo_str, Some("HEAD~1"), Some("HEAD"), "text")
        .await
        .expect("drift_pr text");
    super::drift_pr(repo_str, Some("HEAD"), Some("HEAD"), "json")
        .await
        .expect("drift_pr no changes");
}

#[tokio::test]
async fn drift_json_string_violations_only_has_expected_keys() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = dir.path();
    write_file(
        repo,
        "Cargo.toml",
        r#"[package]
name = "fixture"
version = "0.1.0"
edition = "2021"
"#,
    );
    write_file(repo, "src/lib.rs", "pub fn a() -> u32 { 1 }\n");

    let repo_str = repo.to_str().expect("utf-8");
    let json = super::drift_json_string(repo_str, None, true)
        .await
        .expect("drift_json_string");
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.get("truth_status").is_some());
    assert!(v.get("violations").is_some());
    assert!(v.get("health_score").is_some());
}

#[tokio::test]
async fn status_result_includes_baseline_and_context_when_present() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = dir.path();
    init_git_repo(repo);
    write_file(
        repo,
        "Cargo.toml",
        r#"[package]
name = "fixture"
version = "0.1.0"
edition = "2021"
"#,
    );
    write_file(repo, "src/lib.rs", "pub fn a() -> u32 { 1 }\n");
    write_file(
        repo,
        "repo.sruja",
        r#"
system = kind "System"
container = kind "Container"

App = system "App" {
  description "App"
  Api = container "API" {
    technology "Rust"
    description "API"
  }
}
"#,
    );
    write_file(
        repo,
        ".sruja/context.json",
        r#"{ "updated_at": "2026-05-25T00:00:00Z" }"#,
    );
    write_file(
        repo,
        ".sruja/health_history.json",
        r#"{ "scores": [ { "score": 99 }, { "score": 97 } ] }"#,
    );
    git(repo, &["add", "."]);
    git(repo, &["commit", "-m", "baseline"]);

    let repo_str = repo.to_str().expect("utf-8");
    let status = super::status_result(repo_str).await.expect("status_result");
    assert!(status.baseline.is_some());
    assert_eq!(
        status.context_updated_at.as_deref(),
        Some("2026-05-25T00:00:00Z")
    );
    assert!(!status.truth_status.is_empty());
    assert!(status.health_score.is_some());
    assert_eq!(status.health_history, vec![99, 97]);
}

#[tokio::test]
async fn should_fail_on_violations_matches_expected_criteria() {
    use sruja_diff::{Severity, Violation, ViolationKind};

    let violations = vec![
        Violation {
            kind: ViolationKind::CircularDependency,
            severity: Severity::Error,
            message: "cycle".to_string(),
            location: Some("a -> b".to_string()),
            suggestion: None,
            sources: vec![],
            confidence: None,
            evidence_count: Some(0),
            production_relevant: None,
            baseline_delta: None,
            suppressed: None,
            rule_id: None,
            rationale: None,
        },
        Violation {
            kind: ViolationKind::OrphanComponent,
            severity: Severity::Warning,
            message: "orphan".to_string(),
            location: Some("mod_x".to_string()),
            suggestion: None,
            sources: vec![],
            confidence: None,
            evidence_count: Some(0),
            production_relevant: None,
            baseline_delta: None,
            suppressed: None,
            rule_id: None,
            rationale: None,
        },
        Violation {
            kind: ViolationKind::LayerViolation,
            severity: Severity::Warning,
            message: "layer warning".to_string(),
            location: Some("a -> b".to_string()),
            suggestion: None,
            sources: vec![],
            confidence: None,
            evidence_count: Some(0),
            production_relevant: None,
            baseline_delta: None,
            suppressed: None,
            rule_id: None,
            rationale: None,
        },
        Violation {
            kind: ViolationKind::LayerViolation,
            severity: Severity::Error,
            message: "layer error".to_string(),
            location: Some("core -> infra".to_string()),
            suggestion: None,
            sources: vec![],
            confidence: None,
            evidence_count: Some(0),
            production_relevant: None,
            baseline_delta: None,
            suppressed: None,
            rule_id: None,
            rationale: None,
        },
        Violation {
            kind: ViolationKind::GodModule,
            severity: Severity::Warning,
            message: "god warning".to_string(),
            location: Some("big_module".to_string()),
            suggestion: None,
            sources: vec![],
            confidence: None,
            evidence_count: Some(0),
            production_relevant: None,
            baseline_delta: None,
            suppressed: None,
            rule_id: None,
            rationale: None,
        },
    ];

    // "all" matches Error-severity violations only.
    assert!(super::should_fail_on_violations(Some("all"), &violations));
    // cycles has an Error-severity entry — should match.
    assert!(super::should_fail_on_violations(
        Some("cycles"),
        &violations
    ));
    // layer-violations has a Warning entry (should NOT match) and an Error entry (should match).
    assert!(super::should_fail_on_violations(
        Some("layer-violations"),
        &violations
    ));
    // orphans has only a Warning-severity entry — should NOT match.
    assert!(!super::should_fail_on_violations(
        Some("orphans"),
        &violations
    ));
    // god-modules has only a Warning-severity entry — should NOT match.
    assert!(!super::should_fail_on_violations(
        Some("god-modules"),
        &violations
    ));
    // None should never fail.
    assert!(!super::should_fail_on_violations(None, &violations));
}

#[tokio::test]
async fn status_result_without_baseline_uses_drift_detection() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = dir.path();
    write_file(
        repo,
        "Cargo.toml",
        r#"[package]
name = "fixture"
version = "0.1.0"
edition = "2021"
"#,
    );
    write_file(repo, "src/lib.rs", "pub fn a() -> u32 { 1 }\n");
    write_file(
        repo,
        ".sruja/context.json",
        r#"{ "updated_at": "2026-05-25T00:00:00Z" }"#,
    );

    let repo_str = repo.to_str().expect("utf-8");
    let status = super::status_result(repo_str).await.expect("status_result");
    assert!(status.baseline.is_none());
    assert_eq!(
        status.context_updated_at.as_deref(),
        Some("2026-05-25T00:00:00Z")
    );
    assert!(!status.truth_status.is_empty());
    assert!(status.health_score.is_some());
}

#[tokio::test]
async fn drift_json_string_with_architecture_violations_only_has_expected_keys() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = dir.path();
    write_file(
        repo,
        "Cargo.toml",
        r#"[package]
name = "fixture"
version = "0.1.0"
edition = "2021"
"#,
    );
    write_file(repo, "src/lib.rs", "pub fn a() -> u32 { 1 }\n");
    write_file(
        repo,
        "repo.sruja",
        r#"
system = kind "System"
App = system "App" { description "App" }
"#,
    );

    let repo_str = repo.to_str().expect("utf-8");
    let arch_path = repo.join("repo.sruja");
    let arch_str = arch_path.to_str().expect("utf-8");
    let json = super::drift_json_string(repo_str, Some(arch_str), true)
        .await
        .expect("drift_json_string");
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.get("truth_status").is_some());
    assert!(v.get("summary").is_some());
    assert!(v.get("violations").is_some());
}
