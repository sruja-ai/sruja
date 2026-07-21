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
fn workflow_init_creates_manifest_and_dirs() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (success, _stdout, stderr) = run_sruja(&[
        "workflow",
        "init",
        "-r",
        repo_str,
        "--title",
        "Demo Workflow",
        "--id",
        "wf-test",
    ]);

    assert!(success, "workflow init should succeed: stderr={}", stderr);
    assert!(
        repo.path()
            .join(".sruja/workflows/wf-test/manifest.json")
            .exists(),
        "manifest.json must be created"
    );
    assert!(
        repo.path()
            .join(".sruja/workflows/wf-test/inception")
            .exists(),
        "inception dir must be created"
    );
    assert!(
        repo.path()
            .join(".sruja/workflows/wf-test/construction")
            .exists(),
        "construction dir must be created"
    );
    assert!(
        repo.path()
            .join(".sruja/workflows/wf-test/operations")
            .exists(),
        "operations dir must be created"
    );
}

#[test]
fn workflow_strict_gate_blocks_advance_without_approval() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (success, _stdout, stderr) = run_sruja(&[
        "workflow",
        "init",
        "-r",
        repo_str,
        "--title",
        "Demo Workflow",
        "--id",
        "wf-test",
    ]);
    assert!(success, "workflow init should succeed: stderr={}", stderr);

    let (success, _stdout, _stderr) =
        run_sruja(&["workflow", "advance", "-r", repo_str, "--id", "wf-test"]);
    assert!(
        !success,
        "advance should fail in strict mode without approval"
    );
}

#[test]
fn workflow_init_with_aidlc_creates_manifest_v2_and_docs_dir() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (success, stdout, stderr) = run_sruja(&[
        "workflow",
        "init",
        "-r",
        repo_str,
        "--title",
        "AIDLC Demo",
        "--id",
        "wf-aidlc",
        "--with-aidlc",
        "--aidlc-profile",
        "minimal",
    ]);
    assert!(success, "workflow init --with-aidlc failed: {stderr}");
    assert!(
        stdout.contains("workflow/v2") || stdout.contains("\"aidlc\""),
        "stdout should mention v2 or aidlc: {stdout}"
    );
    assert!(repo
        .path()
        .join(".sruja/workflows/wf-aidlc/inception/aidlc-docs")
        .exists());
}

#[test]
fn workflow_status_includes_aidlc_block_when_enabled() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");
    run_sruja(&[
        "workflow",
        "init",
        "-r",
        repo_str,
        "--title",
        "AIDLC",
        "--id",
        "wf-aidlc2",
        "--with-aidlc",
    ]);
    let (success, stdout, stderr) =
        run_sruja(&["workflow", "status", "-r", repo_str, "--id", "wf-aidlc2"]);
    assert!(success, "status failed: {stderr}");
    assert!(
        stdout.contains("\"aidlc\"") && stdout.contains("workflow_status/v2"),
        "expected aidlc in status json: {stdout}"
    );
}

#[test]
fn workflow_audit_appends_jsonl() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");
    run_sruja(&[
        "workflow", "init", "-r", repo_str, "--title", "Audit", "--id", "wf-audit",
    ]);
    let (success, _, stderr) = run_sruja(&[
        "workflow",
        "audit",
        "-r",
        repo_str,
        "--id",
        "wf-audit",
        "--event",
        "test event",
    ]);
    assert!(success, "audit failed: {stderr}");
    let audit_path = repo.path().join(".sruja/workflows/wf-audit/audit.jsonl");
    assert!(audit_path.exists());
    let text = std::fs::read_to_string(audit_path).expect("audit.jsonl");
    assert!(text.contains("test event"));
}

#[test]
fn workflow_full_lifecycle_init_to_gate_check() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    // 1. Init workflow with AIDLC
    let (success, stdout, stderr) = run_sruja(&[
        "workflow",
        "init",
        "-r",
        repo_str,
        "--title",
        "Lifecycle Test",
        "--id",
        "wf-lifecycle",
        "--with-aidlc",
        "--aidlc-profile",
        "minimal",
    ]);
    assert!(success, "workflow init failed: {stderr}");
    assert!(
        stdout.contains("workflow/v2") || stdout.contains("\"aidlc\""),
        "stdout should mention v2 or aidlc: {stdout}"
    );

    // 2. Check initial status (should be in inception phase)
    let (success, stdout, stderr) =
        run_sruja(&["workflow", "status", "-r", repo_str, "--id", "wf-lifecycle"]);
    assert!(success, "workflow status failed: {stderr}");
    let status: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON status");
    let phase = status["workflow"]["phase"].as_str().unwrap_or("");
    assert_eq!(
        phase, "inception",
        "should start in inception phase, got: {phase}"
    );

    // 3. Create inception artifacts (required for approval)
    let inception_dir = repo.path().join(".sruja/workflows/wf-lifecycle/inception");
    std::fs::create_dir_all(&inception_dir).expect("create inception dir");
    write_file(
        &inception_dir,
        "scope.md",
        "# Scope\nTest workflow for lifecycle integration.",
    );
    write_file(
        &inception_dir,
        "impact.json",
        r#"{"schema_version": "impact/v0", "target_id": "test", "depth": 1, "upstream": [], "downstream": []}"#,
    );

    // 4. Advance to construction (simulate inception approval)
    let (success, _, stderr) = run_sruja(&[
        "workflow",
        "approve",
        "-r",
        repo_str,
        "--id",
        "wf-lifecycle",
        "--phase",
        "inception",
    ]);
    assert!(success, "approve inception failed: {stderr}");

    // Advance to next phase
    let (success, _, stderr) = run_sruja(&[
        "workflow",
        "advance",
        "-r",
        repo_str,
        "--id",
        "wf-lifecycle",
    ]);
    assert!(success, "advance to construction failed: {stderr}");

    // 5. Verify construction artifacts exist
    let (success, stdout, stderr) =
        run_sruja(&["workflow", "status", "-r", repo_str, "--id", "wf-lifecycle"]);
    assert!(success, "status after approve failed: {stderr}");
    let status: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON status");
    let phase = status["workflow"]["phase"].as_str().unwrap_or("");
    assert_eq!(
        phase, "construction",
        "should be in construction phase after inception approval, got: {phase}"
    );

    // Create construction artifacts (required for validation)
    let construction_dir = repo
        .path()
        .join(".sruja/workflows/wf-lifecycle/construction");
    std::fs::create_dir_all(&construction_dir).expect("create construction dir");
    write_file(
        &construction_dir,
        "linked_proposal_ids.json",
        r#"["proposal-001"]"#,
    );
    write_file(
        &construction_dir,
        "task-plan.md",
        "# Task Plan\nImplementation plan for the workflow.",
    );

    // 6. Status should show construction phase
    let (success, stdout, stderr) =
        run_sruja(&["workflow", "status", "-r", repo_str, "--id", "wf-lifecycle"]);
    assert!(success, "workflow status failed: {stderr}");
    let status: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON status");
    let phase = status["workflow"]["phase"].as_str().unwrap_or("");
    assert_eq!(
        phase, "construction",
        "should be in construction phase, got: {phase}"
    );

    // 7. Audit trail - append an event
    let (success, _, stderr) = run_sruja(&[
        "workflow",
        "audit",
        "-r",
        repo_str,
        "--id",
        "wf-lifecycle",
        "--event",
        "lifecycle test event",
    ]);
    assert!(success, "audit failed: {stderr}");

    let audit_path = repo
        .path()
        .join(".sruja/workflows/wf-lifecycle/audit.jsonl");
    assert!(audit_path.exists(), "audit.jsonl should exist");
    let audit_text = std::fs::read_to_string(&audit_path).expect("read audit");
    assert!(
        audit_text.contains("lifecycle test event"),
        "audit should contain our event: {audit_text}"
    );
}

#[test]
fn workflow_e2e_lifecycle_and_mcp_tools() {
    let repo = create_test_repo();
    write_file(repo.path(), "repo.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");

    // 1. Init workflow with E2E template
    let (success, stdout, stderr) = run_sruja(&[
        "workflow",
        "init",
        "-r",
        repo_str,
        "--title",
        "E2E Test Workflow",
        "--id",
        "wf-e2e",
        "--template",
        "e2e",
    ]);
    assert!(success, "workflow init failed: {stderr}");
    assert!(stdout.contains("wf-e2e"), "should mention workflow ID");

    // 2. Validate manifest and required files exist
    let manifest_path = repo.path().join(".sruja/workflows/wf-e2e/manifest.json");
    assert!(manifest_path.exists(), "manifest should be created");

    let manifest_str = std::fs::read_to_string(&manifest_path).unwrap();
    let manifest_val: serde_json::Value = serde_json::from_str(&manifest_str).unwrap();
    assert_eq!(manifest_val["profile"].as_str(), Some("e2e"));

    // 3. Capture requirements
    let (req_ok, _req_out, req_err) = run_sruja(&[
        "workflow",
        "capture-requirements",
        "-r",
        repo_str,
        "--id",
        "wf-e2e",
    ]);
    assert!(req_ok, "capture requirements failed: {req_err}");
    let req_file = repo
        .path()
        .join(".sruja/workflows/wf-e2e/inception/requirements.md");
    assert!(req_file.exists(), "requirements.md should be created");

    // 4. Record test results
    let test_results_path = repo.path().join("my-test-results.json");
    write_file(
        repo.path(),
        "my-test-results.json",
        r#"{"all_passed": true}"#,
    );
    let (tr_ok, _tr_out, tr_err) = run_sruja(&[
        "workflow",
        "record-test-results",
        "-r",
        repo_str,
        "--id",
        "wf-e2e",
        "--from-file",
        test_results_path.to_str().unwrap(),
    ]);
    assert!(tr_ok, "record test results failed: {tr_err}");

    // 5. Record readiness
    let (readiness_ok, _readiness_out, readiness_err) = run_sruja(&[
        "workflow",
        "record-readiness",
        "-r",
        repo_str,
        "--id",
        "wf-e2e",
    ]);
    assert!(readiness_ok, "record readiness failed: {readiness_err}");

    // 6. Get workflow summary in JSON format
    let (sum_ok, sum_out, sum_err) = run_sruja(&[
        "workflow", "summary", "-r", repo_str, "--id", "wf-e2e", "--format", "json",
    ]);
    assert!(sum_ok, "workflow summary failed: {sum_err}");
    let summary: serde_json::Value = serde_json::from_str(sum_out.trim()).unwrap();
    assert_eq!(summary["workflow_id"].as_str(), Some("wf-e2e"));
    assert_eq!(summary["profile"].as_str(), Some("e2e"));
    assert!(summary["health_score"].as_u64().is_some());
}

#[test]
fn init_creates_dot_sruja_dir() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (success, _stdout, stderr) = run_sruja(&["init", "-r", repo_str]);

    assert!(success, "init should succeed: stderr={}", stderr);
    assert!(repo.path().join(".sruja").exists());
}

#[test]
fn init_auto_generates_baseline() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (success, _stdout, stderr) = run_sruja(&["init", "--auto", "-r", repo_str]);

    assert!(success, "init --auto should succeed: stderr={}", stderr);
    assert!(repo.path().join(".sruja").exists());
    assert!(
        repo.path().join("repo.sruja.draft").exists(),
        "init --auto should generate repo.sruja.draft structural evidence"
    );
}

#[test]
fn init_generates_prompt_file() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let skill_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("skills/sruja-architecture/SKILL.md")
        .canonicalize()
        .expect("skill file exists");

    std::env::set_var("SRUJA_SKILL_PATH", skill_path);

    let (success, _stdout, stderr) = run_sruja(&["init", "--prompt", "-r", repo_str]);

    assert!(success, "init --prompt should succeed: stderr={}", stderr);
    assert!(repo.path().join(".sruja").exists());
    assert!(
        repo.path().join(".sruja/init_prompt.txt").exists(),
        "init --prompt should generate init_prompt.txt"
    );
}

#[test]
fn init_fails_when_repo_not_found() {
    let (success, _stdout, stderr) =
        run_sruja(&["init", "-r", "/nonexistent/path/that/does/not/exist"]);

    assert!(!success, "init should fail for nonexistent repo");
    assert!(stderr.contains("Repository not found") || stderr.contains("not found"));
}

#[test]
fn watch_fails_when_repo_not_found() {
    let (success, _stdout, stderr) =
        run_sruja(&["watch", "-r", "/nonexistent/path/that/does/not/exist"]);

    assert!(!success, "watch should fail for nonexistent repo");
    assert!(stderr.contains("Repository not found") || stderr.contains("not found"));
}
