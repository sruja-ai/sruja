//! Integration tests for CLI commands that lack dedicated E2E coverage.
//! Exercises: export, fmt, list, tree, validate, scan using temp repos and run_sruja.

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
fn export_json_succeeds_on_valid_file() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", MINIMAL_VALID_SRUJA);
    let path = repo.path().join("arch.sruja");
    let path_str = path.to_str().expect("utf-8");

    let (success, stdout, stderr) = run_sruja(&["export", "json", path_str]);

    assert!(success, "export json should succeed: stderr={}", stderr);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(
        parsed.get("elements").is_some(),
        "export should have elements"
    );
}

#[test]
fn export_mermaid_succeeds_on_valid_file() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", MINIMAL_VALID_SRUJA);
    let path_str = repo
        .path()
        .join("arch.sruja")
        .to_str()
        .expect("utf-8")
        .to_string();

    let (success, stdout, stderr) = run_sruja(&["export", "mermaid", &path_str]);

    assert!(success, "export mermaid should succeed: stderr={}", stderr);
    assert!(
        stdout.contains("flowchart") || stdout.contains("graph"),
        "mermaid output should contain diagram"
    );
}

#[test]
fn discover_explain_text_highlights_reasoning() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "package.json",
        r#"{"dependencies":{"express":"4.18.0"}}"#,
    );
    write_file(
        repo.path(),
        "src/server.ts",
        r#"
import { query } from "./db";
export function start() { return query(); }
"#,
    );
    write_file(
        repo.path(),
        "src/db.ts",
        r#"export function query() { return []; }"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");

    let (success, stdout, stderr) = run_sruja(&["discover", "--explain", "-r", repo_str]);

    assert!(
        success,
        "discover --explain should succeed: stderr={}",
        stderr
    );
    assert!(stdout.contains("# Sruja Discovery Explanation"));
    assert!(stdout.contains("Why Sruja Thinks That"));
    assert!(stdout.contains("Next Steps"));
}

#[test]
fn daily_alias_refreshes_context_and_prints_next_steps() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "src/app.ts",
        r#"
import { helper } from "./helper";
export function app() { return helper(); }
"#,
    );
    write_file(
        repo.path(),
        "src/helper.ts",
        r#"export function helper() { return "ok"; }"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");

    let (success, stdout, stderr) = run_sruja(&["daily", "-r", repo_str]);

    assert!(success, "daily should succeed: stderr={}", stderr);

    let out = format!("{} {}", stdout, stderr);
    assert!(
        out.contains("Top Actions:") || out.contains("Recommended Actions:"),
        "daily output should include next actions. stdout={} stderr={}",
        stdout,
        stderr
    );
    assert!(
        out.contains("sruja start -r") || out.contains("sruja watch -r"),
        "daily output should include workflow guidance. stdout={} stderr={}",
        stdout,
        stderr
    );

    assert!(
        repo.path().join(".sruja/context.json").exists(),
        "daily should refresh .sruja/context.json"
    );
    assert!(
        repo.path().join(".sruja/graph.json").exists(),
        "daily should refresh .sruja/graph.json"
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
fn list_succeeds_on_valid_file() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", MINIMAL_VALID_SRUJA);
    let path_str = repo
        .path()
        .join("arch.sruja")
        .to_str()
        .expect("utf-8")
        .to_string();

    let (success, stdout, stderr) = run_sruja(&["list", &path_str]);

    assert!(success, "list should succeed: stderr={}", stderr);
    assert!(
        stdout.contains("User") || stdout.contains("App"),
        "list should show elements"
    );
}

#[test]
fn tree_succeeds_on_valid_file() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", MINIMAL_VALID_SRUJA);
    let path_str = repo
        .path()
        .join("arch.sruja")
        .to_str()
        .expect("utf-8")
        .to_string();

    let (success, stdout, stderr) = run_sruja(&["tree", &path_str]);

    assert!(success, "tree should succeed: stderr={}", stderr);
    assert!(!stdout.is_empty(), "tree should produce output");
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
fn diff_json_reports_elements_and_relations() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "a.sruja",
        r#"system = kind "System"
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
        repo.path(),
        "b.sruja",
        r#"system = kind "System"
container = kind "Container"

App = system "App" {
  description "App"
  Api = container "API" {
    technology "Go"
    description "API"
  }
  Worker = container "Worker" {
    technology "Go"
    description "Jobs"
  }
  Api -> Worker "calls"
}
"#,
    );

    let a = repo.path().join("a.sruja");
    let b = repo.path().join("b.sruja");
    let a_str = a.to_str().expect("utf-8");
    let b_str = b.to_str().expect("utf-8");

    let (success, stdout, stderr) = run_sruja(&["diff", a_str, b_str, "--format", "json"]);
    assert!(success, "diff should succeed: stderr={}", stderr);

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(parsed.get("added_elements").is_some());
    assert!(parsed.get("removed_elements").is_some());
    assert!(parsed.get("changed_elements").is_some());
    assert!(parsed.get("added_relations").is_some());
    assert!(parsed.get("removed_relations").is_some());

    let added_elements = parsed
        .get("added_elements")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        added_elements
            .iter()
            .any(|e| e.as_str() == Some("App.Worker")),
        "should report added element App.Worker: added_elements={:?} stdout={}",
        added_elements,
        stdout
    );

    let changed_elements = parsed
        .get("changed_elements")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        changed_elements
            .iter()
            .any(|c| c.get("id").and_then(|v| v.as_str()) == Some("App.Api")),
        "should report changed element App.Api: changed_elements={:?} stdout={}",
        changed_elements,
        stdout
    );

    let added_relations = parsed
        .get("added_relations")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        added_relations.iter().any(|r| {
            r.get("from").and_then(|v| v.as_str()) == Some("App.Api")
                && r.get("to").and_then(|v| v.as_str()) == Some("App.Worker")
        }),
        "should report added relation Api -> Worker: added_relations={:?} stdout={}",
        added_relations,
        stdout
    );
}

#[test]
fn scan_succeeds_on_repo_with_cargo_toml() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let path_str = repo.path().to_str().expect("utf-8");

    let (success, stdout, stderr) = run_sruja(&["scan", path_str, "--output", "-"]);

    assert!(
        success,
        "scan should succeed on Cargo repo: stderr={}",
        stderr
    );
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("scan outputs JSON");
    assert!(
        parsed.get("nodes").is_some()
            || parsed.get("elements").is_some()
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

#[test]
fn impact_json_includes_direct_dependency() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "src/index.js",
        r#"
const helper = require("./helper");
module.exports = { main: () => helper.help() };
"#,
    );
    write_file(
        repo.path(),
        "src/helper.js",
        r#"module.exports = { help: () => "ok" };"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");

    let (success, stdout, stderr) = run_sruja(&[
        "impact",
        "src_index_js",
        "-r",
        repo_str,
        "--depth",
        "1",
        "-f",
        "json",
    ]);

    assert!(success, "impact should succeed: stderr={}", stderr);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    let downstream = parsed
        .get("downstream")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        downstream.iter().any(|hit| {
            hit.get("node")
                .and_then(|n| n.get("id"))
                .and_then(|id| id.as_str())
                == Some("src_helper_js")
        }),
        "impact downstream should include src_helper_js"
    );
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
fn learn_json_labels_hypothesis_artifact() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (init_ok, _, init_stderr) = run_sruja(&["init", "-r", repo_str]);
    assert!(init_ok, "init should succeed: stderr={}", init_stderr);

    let (success, stdout, stderr) =
        run_sruja(&["learn", "-r", repo_str, "-f", "json", "--skip-proposals"]);
    assert!(success, "learn should succeed: stderr={}", stderr);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(
        parsed.get("artifact_kind").and_then(|v| v.as_str()),
        Some("learned_hypothesis")
    );
    assert!(parsed.get("metric_description").is_some());
    assert!(parsed.get("fact_count").is_some());
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
fn sync_writes_context_and_graph() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (success, stdout, stderr) = run_sruja(&["sync", "-r", repo_str, "-f", "json"]);

    assert!(success, "sync should succeed: stderr={}", stderr);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    let context_path = parsed
        .get("context_path")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    assert!(context_path.ends_with(".sruja/context.json"));
    assert!(repo.path().join(".sruja/context.json").exists());
    assert!(repo.path().join(".sruja/graph.json").exists());
}

#[test]
fn context_multi_repo_json_includes_combined_summary_and_repos() {
    let repo_a = create_test_repo();
    write_minimal_cargo_repo(repo_a.path());
    let repo_a_str = repo_a.path().to_str().expect("utf-8");

    let repo_b = create_test_repo();
    write_minimal_cargo_repo(repo_b.path());
    let repo_b_str = repo_b.path().to_str().expect("utf-8");

    let (success, stdout, stderr) =
        run_sruja(&["context", "-r", repo_a_str, "-r", repo_b_str, "-f", "json"]);

    assert!(
        success,
        "context multi-repo should succeed: stderr={}",
        stderr
    );
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");

    let repos = parsed
        .get("repos")
        .and_then(|v| v.as_array())
        .expect("repos must be array");
    assert_eq!(repos.len(), 2, "expected 2 repos in context output");

    let combined = parsed
        .get("combined_summary")
        .expect("combined_summary must exist");
    assert!(
        combined
            .get("total_modules")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 0,
        "combined summary should include modules"
    );
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
fn baseline_and_check_json_succeeds() {
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
        "check",
        "-r",
        repo_str,
        "-f",
        "json",
        "--baseline",
        baseline_path_printed,
    ]);
    assert!(
        check_success,
        "check should succeed: stderr={}",
        check_stderr
    );
    let parsed: serde_json::Value = serde_json::from_str(check_stdout.trim()).expect("valid JSON");
    assert!(parsed.get("truth_status").is_some());
    assert!(parsed.get("violations_baseline").is_some());
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
fn generate_prompt_only_writes_file() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");
    let out = repo
        .path()
        .join("prompt.txt")
        .to_str()
        .expect("utf-8")
        .to_string();

    let skill_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("skills/sruja-architecture/SKILL.md")
        .canonicalize()
        .expect("skill file exists");
    let skill_path_str = skill_path.to_str().expect("utf-8");

    let (success, stdout, stderr) = run_sruja(&[
        "generate",
        "-r",
        repo_str,
        "--skill-path",
        skill_path_str,
        "--prompt-only",
        "-o",
        &out,
    ]);

    assert!(success, "generate should succeed: stderr={}", stderr);
    assert!(stdout.contains("Wrote prompt"));
    let content = std::fs::read_to_string(&out).expect("prompt exists");
    assert!(content.contains("SKILL (follow these rules):"));
    assert!(content.contains("REPO CONTEXT"));
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

#[test]
fn impact_fails_when_node_not_found() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (success, _stdout, stderr) = run_sruja(&[
        "impact",
        "nonexistent_node_xyz",
        "-r",
        repo_str,
        "--depth",
        "1",
        "-f",
        "json",
    ]);

    assert!(!success, "impact should fail for nonexistent node");
    assert!(stderr.contains("No node found") || stderr.contains("not found"));
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

#[test]
fn export_json_handles_missing_file() {
    let (success, _stdout, _stderr) = run_sruja(&["export", "json", "/nonexistent/file.sruja"]);

    assert!(!success, "export should fail for missing file");
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

    // compliance returns non-zero exit code if NonCompliant, which is expected here
    // because the empty repo doesn't match the architecture.
    // We want to verify it still produces a valid JSON report.
    assert!(
        !success,
        "compliance should report non-compliance for empty repo vs architecture"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("valid JSON even on failure");
    assert!(parsed.get("status").is_some());
    assert!(parsed.get("health_score").is_some());
}

#[test]
fn explain_element_json_succeeds() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", MINIMAL_VALID_SRUJA);
    let arch_str = repo
        .path()
        .join("arch.sruja")
        .to_str()
        .expect("utf-8")
        .to_string();

    let (success, stdout, stderr) = run_sruja(&["explain", "App", "--file", &arch_str, "--json"]);

    assert!(success, "explain should succeed: stderr={}", stderr);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(parsed.get("id").and_then(|v| v.as_str()), Some("App"));
    assert!(parsed.get("incoming_relations").is_some());
}

#[test]
fn why_question_succeeds() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    // First sync to build the graph
    run_sruja(&["sync", "-r", repo_str]);

    let (success, stdout, stderr) = run_sruja(&[
        "why",
        "what does this repo do?",
        "-r",
        repo_str,
        "-f",
        "json",
    ]);

    assert!(success, "why should succeed: stderr={}", stderr);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(parsed.get("answer").is_some());
}

#[test]
fn index_embeddings_succeeds() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");
    let arch_str = repo
        .path()
        .join("arch.sruja")
        .to_str()
        .expect("utf-8")
        .to_string();
    let out_path = repo.path().join("vectors.json");
    let out_str = out_path.to_str().expect("utf-8");

    let (success, _stdout, stderr) = run_sruja(&[
        "index", "semantic", "-r", repo_str, "-a", &arch_str, "-o", out_str,
    ]);

    assert!(success, "index should succeed: stderr={}", stderr);
    assert!(out_path.exists(), "vectors.json should be created");
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
fn context_score_json_reports_breakdown() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    // Sync to create the graph
    run_sruja(&["sync", "-r", repo_str]);

    let (success, stdout, stderr) = run_sruja(&["context-score", "-r", repo_str, "-f", "json"]);

    assert!(success, "context-score should succeed: stderr={}", stderr);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(parsed.get("score").is_some());
    assert!(parsed.get("architecture_coverage").is_some());
    assert!(parsed.get("quick_wins").is_some());
    assert_eq!(
        parsed.get("metric_type").and_then(|v| v.as_str()),
        Some("ai_readiness")
    );
    assert!(parsed.get("metric_description").is_some());
}

#[test]
fn focus_json_provides_briefing_for_file() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    // Sync to create the graph
    run_sruja(&["sync", "-r", repo_str]);

    let (success, stdout, stderr) = run_sruja(&[
        "focus",
        "-r",
        repo_str,
        "--file",
        "src/lib.rs",
        "-f",
        "json",
    ]);

    assert!(success, "focus should succeed: stderr={}", stderr);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(parsed.get("target").is_some());
    assert!(parsed.get("blast_radius").is_some());
    assert!(parsed.get("ai_instructions").is_some());
}

#[test]
fn ingest_copies_file_to_context_dir() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    write_file(repo.path(), "adr.md", "# ADR 001\nDecision goes here.");
    let adr_path = repo
        .path()
        .join("adr.md")
        .to_str()
        .expect("utf-8")
        .to_string();

    let (success, _stdout, stderr) =
        run_sruja(&["ingest", "-r", repo_str, &adr_path, "--category", "adr"]);

    assert!(success, "ingest should succeed: stderr={}", stderr);

    let dest_path = repo.path().join(".sruja/context/adr.md");
    assert!(
        dest_path.exists(),
        "ingested file should exist in .sruja/context/"
    );

    let content = std::fs::read_to_string(dest_path).expect("read ingested file");
    assert!(
        content.contains("category: adr"),
        "ingested file should have front-matter"
    );
    assert!(
        content.contains("# ADR 001"),
        "ingested file should keep original content"
    );
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
fn verify_task_coding_profile_succeeds() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(repo.path(), "repo.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");

    // Sync to create graph
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
        "verify_task/v1"
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
    // Should have focus, check, and intent steps
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
    // Should have review, intent, and drift steps
    let steps = parsed["steps"].as_array().unwrap();
    assert!(
        steps.len() >= 3,
        "review should have at least 3 steps, got {}",
        steps.len()
    );
}
