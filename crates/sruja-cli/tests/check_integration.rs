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
fn version_prints_version() {
    let (success, stdout, stderr) = run_sruja(&["version"]);

    assert!(success, "version should succeed: stderr={}", stderr);
    assert!(!stdout.is_empty(), "version should print something");
}

#[test]
fn status_json_includes_truth_and_baseline() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (init_ok, _, init_stderr) = run_sruja(&["init", "-r", repo_str]);
    assert!(init_ok, "init should succeed: stderr={}", init_stderr);

    let (success, stdout, stderr) = run_sruja(&["status", "-r", repo_str, "-f", "json"]);

    assert!(success, "status should succeed: stderr={}", stderr);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(parsed.get("truth_status").is_some());
    assert!(parsed.get("baseline").is_some());
    assert_eq!(
        parsed.get("metric_type").and_then(|v| v.as_str()),
        Some("truth_freshness")
    );
    assert!(parsed.get("metric_description").is_some());
}

#[test]
fn health_json_includes_metric_descriptors() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (init_ok, _, init_stderr) = run_sruja(&["init", "--auto", "-r", repo_str]);
    assert!(
        init_ok,
        "init --auto should succeed: stderr={}",
        init_stderr
    );

    let (success, stdout, stderr) = run_sruja(&["health", "-r", repo_str, "-f", "json"]);
    assert!(success, "health should succeed: stderr={}", stderr);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(parsed.get("score").is_some());
    assert_eq!(
        parsed.get("metric_type").and_then(|v| v.as_str()),
        Some("structural_health")
    );
    assert!(parsed.get("metric_description").is_some());
}

#[test]
fn review_json_succeeds_without_baseline() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (success, stdout, stderr) = run_sruja(&["review", "-r", repo_str, "-f", "json"]);

    assert!(success, "review should succeed: stderr={}", stderr);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(parsed.get("truth_status").is_some());
    assert!(parsed.get("suggestions").is_some());
}

#[test]
fn critique_json_smoke_on_sruja_file() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(repo.path(), "repo.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&[
        "critique",
        "-r",
        repo_str,
        "-f",
        "repo.sruja",
        "--format",
        "json",
        "-d",
        "Change architecture",
    ]);
    assert!(ok, "critique should succeed: stderr={stderr}");
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(
        parsed.get("findings").is_some() || parsed.get("summary").is_some(),
        "expected critique json to include findings or summary: stdout={stdout}"
    );
}

#[test]
fn publish_and_compose_succeeds() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(repo.path(), "repo.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");

    let bundles_dir = repo.path().join("bundles");
    let nested_dir = bundles_dir.join("nested");
    std::fs::create_dir_all(&nested_dir).expect("create bundles/nested");

    let bundle_a_path = bundles_dir.join("a.repo.bundle.json");
    let bundle_a = bundle_a_path.to_str().expect("utf-8").to_string();
    let (pub_success, _pub_stdout, pub_stderr) = run_sruja(&[
        "publish",
        "-r",
        repo_str,
        "--repo-id",
        "repo-a",
        "-o",
        &bundle_a,
    ]);
    assert!(pub_success, "publish should succeed: stderr={}", pub_stderr);
    assert!(bundle_a_path.exists());

    let bundle_a_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&bundle_a_path).expect("read bundle"))
            .expect("bundle must be valid JSON");
    assert_eq!(
        bundle_a_json
            .get("repo_id")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
        "repo-a",
        "repo_id should be overridden"
    );

    let bundle_b_path = nested_dir.join("b.repo.bundle.json");
    let bundle_b = bundle_b_path.to_str().expect("utf-8").to_string();
    let (pub_success_b, _pub_stdout_b, pub_stderr_b) = run_sruja(&[
        "publish",
        "-r",
        repo_str,
        "--repo-id",
        "repo-b",
        "-o",
        &bundle_b,
    ]);
    assert!(
        pub_success_b,
        "publish (2) should succeed: stderr={}",
        pub_stderr_b
    );
    assert!(bundle_b_path.exists());

    let index = repo
        .path()
        .join("system.index.json")
        .to_str()
        .expect("utf-8")
        .to_string();
    let bundles_dir_str = bundles_dir.to_str().expect("utf-8");
    let (compose_success, _compose_stdout, compose_stderr) = run_sruja(&[
        "compose",
        "-i",
        bundles_dir_str,
        "--recursive",
        "-o",
        &index,
    ]);
    assert!(
        compose_success,
        "compose should succeed: stderr={}",
        compose_stderr
    );
    assert!(std::path::Path::new(&index).exists());

    let index_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&index).expect("read index"))
            .expect("index must be valid JSON");
    let repos = index_json
        .get("repos")
        .and_then(|v| v.as_array())
        .expect("repos must be array");
    assert!(
        repos
            .iter()
            .any(|r| r.get("repo_id").and_then(|v| v.as_str()) == Some("repo-a")),
        "system index must include repo-a"
    );
    assert!(
        repos
            .iter()
            .any(|r| r.get("repo_id").and_then(|v| v.as_str()) == Some("repo-b")),
        "system index must include repo-b"
    );
    let nodes = index_json
        .get("nodes")
        .and_then(|v| v.as_array())
        .expect("nodes must be array");
    assert!(
        nodes.iter().any(|n| {
            n.get("canonical_id")
                .and_then(|v| v.as_str())
                .is_some_and(|id| id.starts_with("repo-a::"))
        }),
        "nodes should include canonical ids for repo-a"
    );
    assert!(
        nodes.iter().any(|n| {
            n.get("canonical_id")
                .and_then(|v| v.as_str())
                .is_some_and(|id| id.starts_with("repo-b::"))
        }),
        "nodes should include canonical ids for repo-b"
    );

    let index2 = repo
        .path()
        .join("system2.index.json")
        .to_str()
        .expect("utf-8")
        .to_string();
    let nested_dir_str = nested_dir.to_str().expect("utf-8");
    let (compose2_success, _compose2_stdout, compose2_stderr) = run_sruja(&[
        "compose",
        "-i",
        &bundle_a,
        "-i",
        nested_dir_str,
        "--recursive",
        "-o",
        &index2,
    ]);
    assert!(
        compose2_success,
        "compose (multi -i) should succeed: stderr={}",
        compose2_stderr
    );
    assert!(std::path::Path::new(&index2).exists());
}

#[test]
fn completions_generates_script() {
    let (ok, stdout, stderr) = run_sruja(&["completions", "bash"]);
    assert!(ok, "completions should succeed: stderr={stderr}");
    assert!(
        !stdout.trim().is_empty(),
        "completions output should not be empty"
    );
    assert!(
        stdout.contains("sruja") || stdout.contains("_sruja"),
        "expected completions output to mention the command: stdout={stdout}"
    );
}

#[test]
fn verify_task_coding_profile_succeeds() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(repo.path(), "repo.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");

    run_sruja(&["sync", "-r", repo_str]);

    let (success, stdout, stderr) = run_sruja(&[
        "verify-task",
        "-r",
        repo_str,
        "--profile",
        "coding",
        "-f",
        "json",
        "--max-runtime-ms",
        "60000",
    ]);
    assert!(success, "verify-task coding failed: {stderr}");
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(
        parsed["schema_version"].as_str().unwrap_or(""),
        "verify_task/v2"
    );
    assert_eq!(parsed["profile"].as_str().unwrap_or(""), "coding");
    assert!(parsed.get("steps").is_some(), "should have steps array");
    assert!(parsed.get("all_passed").is_some(), "should have all_passed");
}

#[test]
fn verify_task_arch_profile_succeeds() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(repo.path(), "repo.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");

    run_sruja(&["sync", "-r", repo_str]);

    let (success, stdout, stderr) = run_sruja(&[
        "verify-task",
        "-r",
        repo_str,
        "--profile",
        "arch",
        "-f",
        "json",
        "--max-runtime-ms",
        "60000",
    ]);
    assert!(success, "verify-task arch failed: {stderr}");
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(parsed["profile"].as_str().unwrap_or(""), "arch");
    assert!(parsed.get("steps").is_some());
}

#[test]
fn verify_task_bugfix_profile_succeeds() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(repo.path(), "repo.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");

    run_sruja(&["sync", "-r", repo_str]);

    let (success, stdout, stderr) = run_sruja(&[
        "verify-task",
        "-r",
        repo_str,
        "--profile",
        "bugfix",
        "--file",
        "src/lib.rs",
        "-f",
        "json",
        "--max-runtime-ms",
        "60000",
    ]);
    assert!(success, "verify-task bugfix failed: {stderr}");
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(parsed["profile"].as_str().unwrap_or(""), "bugfix");
    assert!(parsed.get("steps").is_some());
    let steps = parsed["steps"].as_array().unwrap();
    assert!(
        steps.len() >= 3,
        "bugfix should have at least 3 steps, got {}",
        steps.len()
    );
}

#[test]
fn verify_task_review_profile_succeeds() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(repo.path(), "repo.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");

    run_sruja(&["sync", "-r", repo_str]);

    let (success, stdout, stderr) = run_sruja(&[
        "verify-task",
        "-r",
        repo_str,
        "--profile",
        "review",
        "-f",
        "json",
        "--max-runtime-ms",
        "60000",
    ]);
    assert!(success, "verify-task review failed: {stderr}");
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(parsed["profile"].as_str().unwrap_or(""), "review");
    assert!(parsed.get("steps").is_some());
    let steps = parsed["steps"].as_array().unwrap();
    assert!(
        steps.len() >= 3,
        "review should have at least 3 steps, got {}",
        steps.len()
    );
}
