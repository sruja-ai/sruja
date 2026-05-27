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
fn discover_subcommands_cover_context_repomap_questions_and_enrichment() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "package.json",
        r#"{"name":"fixture","version":"0.1.0","dependencies":{"express":"4.18.0"}}"#,
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

    let (ctx_ok, ctx_out, ctx_err) =
        run_sruja(&["discover", "-r", repo_str, "--format", "json", "context"]);
    assert!(ctx_ok, "discover context should succeed: stderr={ctx_err}");
    assert!(
        !ctx_out.trim().is_empty(),
        "discover context should produce output"
    );
    let ctx_trim = ctx_out.trim_start();
    if ctx_trim.starts_with('{') || ctx_trim.starts_with('[') {
        let _parsed: serde_json::Value =
            serde_json::from_str(ctx_out.trim()).expect("discover context JSON");
    } else {
        assert!(
            ctx_out.contains("Context") || ctx_out.contains("Repo") || ctx_out.contains("Sruja"),
            "discover context should look like a summary: stdout={ctx_out}"
        );
    }

    let (rm_ok, rm_out, rm_err) = run_sruja(&[
        "discover",
        "-r",
        repo_str,
        "--max-files",
        "25",
        "--max-tokens",
        "1200",
        "--update",
        "repomap",
    ]);
    assert!(rm_ok, "discover repomap should succeed: stderr={rm_err}");
    assert!(
        rm_out.contains("# Sruja Repomap")
            || rm_out.contains("# Repository Map")
            || rm_out.contains("Repomap"),
        "expected repomap header: stdout={rm_out}"
    );

    let (q_ok, q_out, q_err) =
        run_sruja(&["discover", "-r", repo_str, "--format", "json", "questions"]);
    assert!(q_ok, "discover questions should succeed: stderr={q_err}");
    assert!(
        !q_out.trim().is_empty(),
        "discover questions should produce output"
    );
    let q_trim = q_out.trim_start();
    if q_trim.starts_with('{') || q_trim.starts_with('[') {
        let questions: serde_json::Value =
            serde_json::from_str(q_out.trim()).expect("discover questions JSON");
        assert!(
            questions.as_array().is_some() || questions.get("questions").is_some(),
            "expected questions payload: {q_out}"
        );
    } else {
        assert!(
            q_out.to_lowercase().contains("question"),
            "discover questions should include a header: stdout={q_out}"
        );
    }

    let report_path = repo.path().join("GRAPH_REPORT.md");
    let report_str = report_path.to_str().expect("utf-8");
    let (ex_ok, ex_out, ex_err) = run_sruja(&[
        "discover",
        "-r",
        repo_str,
        "--format",
        "json",
        "--enrich",
        "--enrich-provider",
        "cmd",
        "--enrich-cmd",
        "cat",
        "--export-report",
        report_str,
        "explain",
    ]);
    assert!(ex_ok, "discover explain should succeed: stderr={ex_err}");
    assert!(report_path.exists(), "expected report file to be written");
    let combined = format!("{}{}", ex_out, ex_err);
    if !combined.trim().is_empty() {
        let trim = combined.trim_start();
        if trim.starts_with('{') || trim.starts_with('[') {
            let _parsed: serde_json::Value =
                serde_json::from_str(trim).expect("discover explain JSON");
        }
    }
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
        repo.path().join(".sruja/cache/scan.json").exists(),
        "daily should refresh .sruja/cache/scan.json"
    );
}

#[test]
fn quickstart_generates_baseline_and_emits_json() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(
        repo.path(),
        "src/main.rs",
        "fn main() { println!(\"hi\"); }\n",
    );
    let repo_str = repo.path().to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&[
        "quickstart",
        "-r",
        repo_str,
        "-f",
        "json",
        "--generate-baseline",
        "--advisory",
    ]);
    assert!(ok, "quickstart should succeed: stderr={stderr}");
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(
        parsed.get("repo").is_some()
            && parsed.get("health_score").is_some()
            && parsed.get("inventory").is_some(),
        "expected quickstart json payload: stdout={stdout}"
    );
    assert!(
        repo.path().join("repo.sruja.draft").exists(),
        "quickstart --generate-baseline should write repo.sruja.draft"
    );
}

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
    assert!(repo.path().join(".sruja/cache/scan.json").exists());
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
fn focus_json_includes_run_id_and_resolved_target() {
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
        r#"
export function helper() { return "ok"; }
"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");
    let (sync_ok, _, sync_err) = run_sruja(&["sync", "-r", repo_str, "-f", "json"]);
    assert!(sync_ok, "sync should succeed: stderr={sync_err}");

    let (ok, stdout, stderr) = run_sruja(&[
        "focus",
        "-r",
        repo_str,
        "--file",
        "src/app.ts",
        "-f",
        "json",
    ]);
    assert!(ok, "focus should succeed: stderr={stderr}");

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    let run_id = parsed
        .get("run_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    assert!(!run_id.is_empty(), "expected run_id to be set");
    let resolved = parsed
        .get("target")
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    assert!(
        !resolved.is_empty(),
        "expected target.id to be set: stdout={stdout}"
    );
}

#[test]
fn focus_for_ai_outputs_schema_version() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "src/app.ts",
        r#"
export function app() { return 1; }
"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");
    let (ok, stdout, stderr) = run_sruja(&[
        "focus",
        "-r",
        repo_str,
        "--file",
        "src/app.ts",
        "-f",
        "for-ai",
    ]);
    assert!(ok, "focus -f for-ai should succeed: stderr={stderr}");

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(
        parsed
            .get("schema_version")
            .and_then(|v| v.as_str())
            .unwrap_or_default(),
        "focus_for_ai/v1"
    );
}

#[test]
fn ai_output_includes_task_context_section() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "src/app.ts",
        r#"
export function app() { return 1; }
"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&[
        "ai",
        "-r",
        repo_str,
        "--file",
        "src/app.ts",
        "--max-tokens",
        "500",
    ]);
    assert!(ok, "ai should succeed: stderr={stderr}");
    assert!(
        stdout.contains("## Task Context"),
        "expected task context section: stdout={stdout}"
    );
}

#[test]
fn ai_context_cursor_rules_includes_header() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&[
        "ai-context",
        "-r",
        repo_str,
        "-f",
        "cursor-rules",
        "--max-tokens",
        "2000",
    ]);
    assert!(
        ok,
        "ai-context -f cursor-rules should succeed: stderr={stderr}"
    );
    assert!(
        stdout.contains("# Sruja Architecture Context"),
        "expected cursor-rules header: stdout={stdout}"
    );
}

#[test]
fn ai_context_multi_repo_cursor_rules_includes_combined_summary() {
    let repo_a = create_test_repo();
    write_minimal_cargo_repo(repo_a.path());
    let repo_a_str = repo_a.path().to_str().expect("utf-8");

    let repo_b = create_test_repo();
    write_minimal_cargo_repo(repo_b.path());
    let repo_b_str = repo_b.path().to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&[
        "ai-context",
        "-r",
        repo_a_str,
        "-r",
        repo_b_str,
        "-f",
        "cursor-rules",
        "--max-tokens",
        "2500",
    ]);
    assert!(
        ok,
        "ai-context multi -f cursor-rules should succeed: stderr={stderr}"
    );
    assert!(
        stdout.contains("Multi-Repo") && stdout.contains("Combined Summary"),
        "expected multi-repo cursor-rules output: stdout={stdout}"
    );
}

#[test]
fn ai_context_multi_repo_copilot_instructions_includes_header() {
    let repo_a = create_test_repo();
    write_minimal_cargo_repo(repo_a.path());
    let repo_a_str = repo_a.path().to_str().expect("utf-8");

    let repo_b = create_test_repo();
    write_minimal_cargo_repo(repo_b.path());
    let repo_b_str = repo_b.path().to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&[
        "ai-context",
        "-r",
        repo_a_str,
        "-r",
        repo_b_str,
        "-f",
        "copilot-instructions",
        "--max-tokens",
        "2500",
    ]);
    assert!(
        ok,
        "ai-context multi -f copilot-instructions should succeed: stderr={stderr}"
    );
    assert!(
        stdout.contains("GitHub Copilot") && stdout.contains("Combined Summary"),
        "expected copilot instructions output: stdout={stdout}"
    );
}

#[test]
fn ai_context_markdown_and_repomap_formats_succeed() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "src/index.ts",
        r#"
import { helper } from "./helper";
export function main() { return helper(); }
"#,
    );
    write_file(
        repo.path(),
        "src/helper.ts",
        r#"
export function helper() { return 1; }
"#,
    );
    write_file(
        repo.path(),
        "package.json",
        r#"{"name":"fixture","version":"0.1.0"}"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");

    let (md_ok, md_out, md_err) = run_sruja(&[
        "ai-context",
        "-r",
        repo_str,
        "-f",
        "markdown",
        "--max-tokens",
        "2500",
    ]);
    assert!(
        md_ok,
        "ai-context -f markdown should succeed: stderr={md_err}"
    );
    assert!(
        md_out.contains("# Architecture Context"),
        "expected markdown header: stdout={md_out}"
    );

    let (rm_ok, rm_out, rm_err) = run_sruja(&[
        "ai-context",
        "-r",
        repo_str,
        "-f",
        "repomap",
        "--max-tokens",
        "1500",
    ]);
    assert!(
        rm_ok,
        "ai-context -f repomap should succeed: stderr={rm_err}"
    );
    assert!(
        rm_out.contains("# Sruja Repomap"),
        "expected repomap header: stdout={rm_out}"
    );
}

#[test]
fn dsl_import_from_exported_json_succeeds() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", MINIMAL_VALID_SRUJA);
    let arch_str = repo
        .path()
        .join("arch.sruja")
        .to_str()
        .expect("utf-8")
        .to_string();

    let (export_ok, export_out, export_err) = run_sruja(&["export", "json", &arch_str]);
    assert!(export_ok, "export json should succeed: stderr={export_err}");

    let json_path = repo.path().join("arch.json");
    std::fs::write(&json_path, export_out).expect("write arch.json");
    let json_str = json_path.to_str().expect("utf-8");

    let (import_ok, import_out, import_err) = run_sruja(&["dsl", "import", "json", json_str]);
    assert!(
        import_ok,
        "dsl import json should succeed: stderr={import_err}"
    );
    assert!(
        import_out.contains("App = system") && import_out.contains("User -> App"),
        "expected imported DSL to include elements and relations: stdout={import_out}"
    );
}

#[test]
fn dsl_generate_prompt_only_writes_file() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");
    let out = repo
        .path()
        .join("dsl_prompt.txt")
        .to_str()
        .expect("utf-8")
        .to_string();

    let skill_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("skills/sruja-architecture/SKILL.md")
        .canonicalize()
        .expect("skill file exists");
    let skill_path_str = skill_path.to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&[
        "dsl",
        "generate",
        "-r",
        repo_str,
        "--skill-path",
        skill_path_str,
        "--prompt-only",
        "-o",
        &out,
    ]);
    assert!(ok, "dsl generate should succeed: stderr={stderr}");
    assert!(stdout.contains("Wrote prompt") || stdout.contains("prompt"));
    let content = std::fs::read_to_string(&out).expect("dsl_prompt.txt exists");
    assert!(content.contains("SKILL"));
}

#[test]
fn agent_run_plan_json_succeeds() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&[
        "agent",
        "run",
        "-r",
        repo_str,
        "--goal",
        "test goal",
        "--file",
        "src/lib.rs",
        "--mode",
        "plan",
        "-f",
        "json",
    ]);
    assert!(ok, "agent run --mode plan should succeed: stderr={stderr}");
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(
        parsed.get("plan").is_some() || stdout.contains("agent_plan_output/v1"),
        "expected agent run output to include plan: stdout={stdout}"
    );
}

#[test]
fn agent_run_apply_emits_json_and_records_compression_and_matts_note() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(repo.path(), "repo.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");

    let (init_ok, _init_out, init_err) = run_sruja(&["init", "--auto", "-r", repo_str]);
    assert!(init_ok, "init --auto should succeed: stderr={init_err}");

    write_file(
        repo.path(),
        ".sruja/config.toml",
        r#"
[context_engineering]
compression_token_threshold = 1
compression_keep_recent = 1
compression_suppress_recompress_turns = 1

[sandbox]
policy = "warn_and_degrade"
cleanup_on_success = true
keep_on_failure = false
"#,
    );

    let (ok, stdout, stderr) = run_sruja(&[
        "agent",
        "run",
        "-r",
        repo_str,
        "--goal",
        "test apply",
        "--file",
        "src/lib.rs",
        "--mode",
        "apply",
        "--continue-on-error",
        "--trajectories",
        "2",
        "--enrich",
        "--enrich-provider",
        "cmd",
        "--enrich-cmd",
        "cat",
        "-f",
        "json",
    ]);
    assert!(ok, "agent run --mode apply should succeed: stderr={stderr}");

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(
        parsed.get("schema_version").and_then(|v| v.as_str()),
        Some("agent_apply_output/v1")
    );

    let compression = parsed
        .get("observation_compression")
        .expect("expected observation_compression to exist");
    assert!(
        compression
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        "expected compression to be enabled: {compression}"
    );

    let notes = parsed
        .get("memory_recorded")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        notes.iter().any(|n| {
            n.as_str()
                .is_some_and(|s| s.contains("MaTTS requested") || s.contains("sandboxing"))
        }),
        "expected MaTTS degrade note in memory_recorded: {notes:?}"
    );
}

#[test]
fn propose_approve_writes_updated_repo_sruja() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(
        repo.path(),
        "repo.sruja",
        r#"
system = kind "System"

App = system "App" {
  description "App"
}
"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");

    let (create_ok, create_stdout, create_err) = run_sruja(&[
        "propose",
        "create",
        "-r",
        repo_str,
        "-d",
        "Add worker system",
        "-e",
        "Worker:system:Worker",
        "-l",
        "App->Worker:calls",
        "-f",
        "json",
    ]);
    assert!(
        create_ok,
        "propose create should succeed: stderr={create_err}"
    );
    let created: serde_json::Value =
        serde_json::from_str(create_stdout.trim()).expect("valid JSON");
    let proposal_id = created
        .get("proposal")
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    assert!(!proposal_id.is_empty());

    let (approve_ok, approve_out, approve_err) = run_sruja(&[
        "propose",
        "approve",
        &proposal_id,
        "-r",
        repo_str,
        "-f",
        "json",
    ]);
    assert!(
        approve_ok,
        "propose approve should succeed: stderr={approve_err} stdout={approve_out}"
    );

    let dsl = std::fs::read_to_string(repo.path().join("repo.sruja")).expect("repo.sruja");
    assert!(
        dsl.contains("Worker") && dsl.contains("system"),
        "expected approved proposal to update repo.sruja: {dsl}"
    );
}

#[test]
fn focus_text_output_includes_blast_radius() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "src/app.ts",
        r#"
export function app() { return 1; }
"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");
    let (ok, stdout, stderr) = run_sruja(&["focus", "-r", repo_str, "--file", "src/app.ts"]);
    assert!(ok, "focus (text) should succeed: stderr={stderr}");
    assert!(
        stdout.contains("Blast Radius"),
        "expected blast radius section: stdout={stdout}"
    );
}

#[test]
fn focus_json_includes_temporal_context_when_git_range_is_provided() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "src/app.ts",
        r#"
export function app() { return 1; }
"#,
    );
    let repo_path = repo.path();
    std::process::Command::new("git")
        .current_dir(repo_path)
        .args(["init"])
        .output()
        .expect("git init");
    std::process::Command::new("git")
        .current_dir(repo_path)
        .args(["config", "user.email", "test@example.com"])
        .output()
        .expect("git config email");
    std::process::Command::new("git")
        .current_dir(repo_path)
        .args(["config", "user.name", "Test"])
        .output()
        .expect("git config name");
    std::process::Command::new("git")
        .current_dir(repo_path)
        .args(["add", "."])
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .current_dir(repo_path)
        .args(["commit", "-m", "c1"])
        .output()
        .expect("git commit c1");

    write_file(
        repo.path(),
        "src/app.ts",
        r#"
export function app() { return 2; }
"#,
    );
    std::process::Command::new("git")
        .current_dir(repo_path)
        .args(["add", "."])
        .output()
        .expect("git add 2");
    std::process::Command::new("git")
        .current_dir(repo_path)
        .args(["commit", "-m", "c2"])
        .output()
        .expect("git commit c2");

    let repo_str = repo.path().to_str().expect("utf-8");
    let (ok, stdout, stderr) = run_sruja(&[
        "focus",
        "-r",
        repo_str,
        "--file",
        "src/app.ts",
        "--base-ref",
        "HEAD~1",
        "--head-ref",
        "HEAD",
        "-f",
        "json",
    ]);
    assert!(
        ok,
        "focus with temporal context should succeed: stderr={stderr}"
    );
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(
        parsed.get("temporal").is_some(),
        "expected temporal context in focus json: stdout={stdout}"
    );
}

#[test]
fn ai_writes_output_file() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "src/app.ts",
        r#"
export function app() { return 1; }
"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");
    let out_path = repo.path().join("ai.txt");
    let out_str = out_path.to_str().expect("utf-8");

    let (ok, _stdout, stderr) = run_sruja(&[
        "ai",
        "-r",
        repo_str,
        "--file",
        "src/app.ts",
        "--max-tokens",
        "500",
        "-o",
        out_str,
    ]);
    assert!(ok, "ai -o should succeed: stderr={stderr}");
    assert!(out_path.exists(), "expected output file to be written");
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
fn agent_record_and_history_roundtrip_json() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (record_ok, _, record_err) = run_sruja(&[
        "agent",
        "record",
        "-r",
        repo_str,
        "-c",
        "test context",
        "-H",
        "test hypothesis",
        "-o",
        "failed",
        "-g",
        "test guardrail",
        "-s",
        "test reason",
    ]);
    assert!(
        record_ok,
        "agent record should succeed: stderr={record_err}"
    );

    let (history_ok, stdout, history_err) =
        run_sruja(&["agent", "history", "-r", repo_str, "-f", "json"]);
    assert!(
        history_ok,
        "agent history -f json should succeed: stderr={history_err}"
    );
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    let entries = parsed.as_array().cloned().unwrap_or_default();
    assert!(!entries.is_empty(), "expected at least one entry");
}

#[test]
fn agent_crud_commands_succeed() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    run_sruja(&[
        "agent",
        "record",
        "-r",
        repo_str,
        "-c",
        "ctx 1",
        "-H",
        "hyp 1",
        "-o",
        "failed",
        "-g",
        "guardrail 1",
    ]);
    run_sruja(&[
        "agent",
        "record",
        "-r",
        repo_str,
        "-c",
        "ctx 2",
        "-H",
        "hyp 2",
        "-o",
        "failed",
        "-g",
        "guardrail 2",
    ]);

    let (history_ok, history_out, history_err) =
        run_sruja(&["agent", "history", "-r", repo_str, "-f", "json"]);
    assert!(
        history_ok,
        "agent history should succeed: stderr={history_err}"
    );
    let parsed: serde_json::Value = serde_json::from_str(history_out.trim()).expect("valid JSON");
    let entries = parsed.as_array().cloned().unwrap_or_default();
    assert!(entries.len() >= 2, "expected >=2 entries: {history_out}");
    let id1 = entries[0]
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let id2 = entries[1]
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    assert!(!id1.is_empty() && !id2.is_empty());

    let (update_ok, _, update_err) = run_sruja(&[
        "agent",
        "update",
        "-r",
        repo_str,
        "-i",
        id1,
        "-c",
        "updated ctx",
    ]);
    assert!(
        update_ok,
        "agent update should succeed: stderr={update_err}"
    );

    let (curate_ok, curate_out, curate_err) =
        run_sruja(&["agent", "curate", "-r", repo_str, "-f", "json"]);
    assert!(
        curate_ok,
        "agent curate should succeed: stderr={curate_err}"
    );
    let _parsed: serde_json::Value = serde_json::from_str(curate_out.trim()).expect("valid JSON");

    let (clusters_ok, clusters_out, clusters_err) =
        run_sruja(&["agent", "clusters", "-r", repo_str, "-f", "json"]);
    assert!(
        clusters_ok,
        "agent clusters should succeed: stderr={clusters_err}"
    );
    let _parsed: serde_json::Value = serde_json::from_str(clusters_out.trim()).expect("valid JSON");

    let (delete_ok, _, delete_err) =
        run_sruja(&["agent", "delete", "-r", repo_str, "-i", id2, "-y"]);
    assert!(
        delete_ok,
        "agent delete should succeed: stderr={delete_err}"
    );

    let (clear_ok, _, clear_err) = run_sruja(&["agent", "clear", "-r", repo_str, "-y"]);
    assert!(clear_ok, "agent clear should succeed: stderr={clear_err}");
}

#[test]
fn agent_plan_prints_plan_json() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&[
        "agent",
        "plan",
        "-r",
        repo_str,
        "--goal",
        "test goal",
        "--file",
        "src/lib.rs",
        "--print",
    ]);
    assert!(ok, "agent plan should succeed: stderr={stderr}");

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(
        parsed
            .get("schema_version")
            .and_then(|v| v.as_str())
            .unwrap_or_default(),
        "agent_plan_output/v1"
    );
}

#[test]
fn propose_create_list_and_approve_dry_run_work() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(
        repo.path(),
        "repo.sruja",
        r#"
system = kind "System"

App = system "App" {
  description "App"
}
"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");

    let (create_ok, create_stdout, create_err) = run_sruja(&[
        "propose",
        "create",
        "-r",
        repo_str,
        "-d",
        "Add worker container",
        "-e",
        "Worker:system:Worker",
        "-l",
        "App->Worker:calls",
        "-f",
        "json",
    ]);
    assert!(
        create_ok,
        "propose create should succeed: stderr={create_err} stdout={create_stdout}"
    );
    let created: serde_json::Value =
        serde_json::from_str(create_stdout.trim()).expect("valid JSON");
    assert_eq!(
        created
            .get("schema_version")
            .and_then(|v| v.as_str())
            .unwrap_or_default(),
        "proposal_create/v1"
    );
    let proposal_id = created
        .get("proposal")
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    assert!(!proposal_id.is_empty(), "expected proposal_id in output");

    let (list_ok, list_stdout, list_err) =
        run_sruja(&["propose", "list", "-r", repo_str, "-f", "json"]);
    assert!(list_ok, "propose list should succeed: stderr={list_err}");
    let listed: serde_json::Value = serde_json::from_str(list_stdout.trim()).expect("valid JSON");
    let proposals = listed.as_array().cloned().unwrap_or_default();
    assert!(
        proposals
            .iter()
            .any(|p| p.get("id").and_then(|v| v.as_str()) == Some(&proposal_id)),
        "expected proposal_id to appear in list: id={proposal_id} stdout={list_stdout}"
    );

    let (approve_ok, approve_stdout, approve_err) = run_sruja(&[
        "propose",
        "approve",
        &proposal_id,
        "-r",
        repo_str,
        "--dry-run",
        "-f",
        "json",
    ]);
    assert!(
        approve_ok,
        "propose approve --dry-run should succeed: stderr={approve_err}"
    );
    let approved: serde_json::Value =
        serde_json::from_str(approve_stdout.trim()).expect("valid JSON");
    assert_eq!(
        approved
            .get("schema_version")
            .and_then(|v| v.as_str())
            .unwrap_or_default(),
        "proposal_approve_dry_run/v1"
    );
}

#[test]
fn dsl_subcommands_smoke() {
    let repo = create_test_repo();
    write_file(repo.path(), "a.sruja", MINIMAL_VALID_SRUJA);
    write_file(
        repo.path(),
        "b.sruja",
        r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

User = person "User" { description "End user" }

App = system "My App" {
  description "Main application"
  Web = container "Web" { technology "React" description "UI" }
  Api = container "API" { technology "Go" description "API" }
  Web -> Api "calls"
}
User -> App "uses"
"#,
    );
    let a = repo
        .path()
        .join("a.sruja")
        .to_str()
        .expect("utf-8")
        .to_string();
    let b = repo
        .path()
        .join("b.sruja")
        .to_str()
        .expect("utf-8")
        .to_string();

    let (list_ok, list_out, list_err) = run_sruja(&["dsl", "list", &a]);
    assert!(list_ok, "dsl list should succeed: stderr={list_err}");
    assert!(!list_out.is_empty());

    let (tree_ok, tree_out, tree_err) = run_sruja(&["dsl", "tree", &a]);
    assert!(tree_ok, "dsl tree should succeed: stderr={tree_err}");
    assert!(!tree_out.is_empty());

    let (diff_ok, diff_out, diff_err) = run_sruja(&["dsl", "diff", &a, &b, "--format", "json"]);
    assert!(diff_ok, "dsl diff should succeed: stderr={diff_err}");
    let _parsed: serde_json::Value = serde_json::from_str(diff_out.trim()).expect("valid JSON");

    let (compile_ok, _, compile_err) = run_sruja(&["dsl", "compile", &a]);
    assert!(
        compile_ok,
        "dsl compile should succeed: stderr={compile_err}"
    );

    let (validate_ok, validate_out, validate_err) =
        run_sruja(&["dsl", "validate", &a, "--format-json"]);
    assert!(
        validate_ok,
        "dsl validate should succeed: stderr={validate_err}"
    );
    let _parsed: serde_json::Value = serde_json::from_str(validate_out.trim()).expect("valid JSON");

    let (explain_ok, explain_out, explain_err) =
        run_sruja(&["dsl", "explain", "App", "--file", &a, "--json"]);
    assert!(
        explain_ok,
        "dsl explain should succeed: stderr={explain_err}"
    );
    let parsed: serde_json::Value = serde_json::from_str(explain_out.trim()).expect("valid JSON");
    assert!(
        parsed.get("element").is_some() || parsed.get("id").is_some(),
        "expected explain json to include element info: stdout={explain_out}"
    );
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

#[test]
fn run_show_json_includes_snapshots() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "src/app.ts",
        r#"
export function app() { return 1; }
"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");

    let (focus_ok, focus_out, focus_err) = run_sruja(&[
        "focus",
        "-r",
        repo_str,
        "--file",
        "src/app.ts",
        "-f",
        "json",
    ]);
    assert!(focus_ok, "focus should succeed: stderr={focus_err}");
    let focus_json: serde_json::Value = serde_json::from_str(focus_out.trim()).expect("valid JSON");
    let run_id = focus_json
        .get("run_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    assert!(!run_id.is_empty(), "expected run_id: stdout={focus_out}");

    let (show_ok, show_out, show_err) = run_sruja(&[
        "run", "show", "-r", repo_str, "--run-id", &run_id, "-f", "json",
    ]);
    assert!(show_ok, "run show should succeed: stderr={show_err}");
    let show_json: serde_json::Value = serde_json::from_str(show_out.trim()).expect("valid JSON");
    assert_eq!(
        show_json.get("schema_version").and_then(|v| v.as_str()),
        Some("run_show/v1")
    );
    assert_eq!(
        show_json.get("run_id").and_then(|v| v.as_str()),
        Some(run_id.as_str())
    );
    assert!(show_json.get("files").is_some());
    assert!(show_json.get("snapshots").is_some());
}

#[test]
fn run_export_writes_manifest() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "src/app.ts",
        r#"
export function app() { return 1; }
"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");

    let (focus_ok, focus_out, focus_err) = run_sruja(&[
        "focus",
        "-r",
        repo_str,
        "--file",
        "src/app.ts",
        "-f",
        "json",
    ]);
    assert!(focus_ok, "focus should succeed: stderr={focus_err}");
    let focus_json: serde_json::Value = serde_json::from_str(focus_out.trim()).expect("valid JSON");
    let run_id = focus_json
        .get("run_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    assert!(!run_id.is_empty(), "expected run_id: stdout={focus_out}");

    let out_dir = repo.path().join("exported_run");
    let out_dir_str = out_dir.to_str().expect("utf-8");
    let (export_ok, export_out, export_err) = run_sruja(&[
        "run",
        "export",
        "-r",
        repo_str,
        "--run-id",
        &run_id,
        "--out",
        out_dir_str,
        "--events-limit",
        "10",
    ]);
    assert!(export_ok, "run export should succeed: stderr={export_err}");
    assert!(
        export_out.contains(out_dir_str),
        "expected out_dir to be printed: stdout={export_out}"
    );

    let manifest_path = out_dir.join("manifest.json");
    assert!(
        manifest_path.exists(),
        "manifest.json must exist at {}",
        manifest_path.display()
    );
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest_path).expect("read manifest"))
            .expect("manifest must be valid JSON");
    assert_eq!(
        manifest.get("schema_version").and_then(|v| v.as_str()),
        Some("run_export/v1")
    );
    assert_eq!(
        manifest.get("run_id").and_then(|v| v.as_str()),
        Some(run_id.as_str())
    );
    assert!(out_dir.join("context_events.json").exists());
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

fn mcp_send(stdin: &mut std::process::ChildStdin, value: serde_json::Value) -> std::io::Result<()> {
    use std::io::Write;
    writeln!(
        stdin,
        "{}",
        serde_json::to_string(&value).unwrap_or_default()
    )?;
    stdin.flush()
}

fn mcp_recv_matching_id(
    rx: &std::sync::mpsc::Receiver<serde_json::Value>,
    id: serde_json::Value,
) -> serde_json::Value {
    use std::time::{Duration, Instant};
    let deadline = Instant::now() + Duration::from_secs(5);

    loop {
        let now = Instant::now();
        let wait = deadline.saturating_duration_since(now);
        let msg = rx
            .recv_timeout(wait)
            .unwrap_or_else(|_| panic!("timed out waiting for MCP response id={id}"));
        if msg.get("id") == Some(&id) {
            return msg;
        }
    }
}

fn mcp_recv_notification_method(
    rx: &std::sync::mpsc::Receiver<serde_json::Value>,
    method: &str,
) -> serde_json::Value {
    use std::time::{Duration, Instant};
    let deadline = Instant::now() + Duration::from_secs(5);

    loop {
        let now = Instant::now();
        let wait = deadline.saturating_duration_since(now);
        let msg = rx
            .recv_timeout(wait)
            .unwrap_or_else(|_| panic!("timed out waiting for MCP notification {method}"));
        if msg.get("method").and_then(|v| v.as_str()) == Some(method) {
            return msg;
        }
    }
}

fn mcp_extract_text_result(response: &serde_json::Value) -> String {
    response
        .get("result")
        .and_then(|r| r.get("content"))
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|c0| c0.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or_default()
        .to_string()
}

#[test]
fn mcp_server_roundtrip_tools_resources_prompts_and_tools_call() {
    use std::io::BufRead;
    use std::process::Stdio;

    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(repo.path(), "repo.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");

    let (init_ok, _init_out, init_err) = run_sruja(&["init", "--auto", "-r", repo_str]);
    assert!(init_ok, "init --auto should succeed: stderr={init_err}");

    let (wf_ok, _wf_out, wf_err) = run_sruja(&[
        "workflow", "init", "-r", repo_str, "--title", "MCP Test", "--id", "wf-mcp",
    ]);
    assert!(wf_ok, "workflow init should succeed: stderr={wf_err}");

    let bundles_dir = repo.path().join("bundles");
    std::fs::create_dir_all(&bundles_dir).expect("create bundles dir");
    let bundle_path = bundles_dir.join("a.repo.bundle.json");
    let bundle_str = bundle_path.to_str().expect("utf-8");
    let (pub_ok, _pub_out, pub_err) = run_sruja(&[
        "publish",
        "-r",
        repo_str,
        "--repo-id",
        "repo-a",
        "-o",
        bundle_str,
    ]);
    assert!(pub_ok, "publish should succeed: stderr={pub_err}");
    assert!(bundle_path.exists(), "bundle must exist");

    let system_index_path = repo.path().join("system.index.json");
    let system_index_str = system_index_path.to_str().expect("utf-8");
    let (compose_ok, _compose_out, compose_err) =
        run_sruja(&["compose", "-i", bundle_str, "-o", system_index_str]);
    assert!(compose_ok, "compose should succeed: stderr={compose_err}");
    assert!(system_index_path.exists(), "system.index.json must exist");

    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_sruja"))
        .args(["mcp", "-r", repo_str])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn mcp server");

    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");

    let (tx, rx) = std::sync::mpsc::channel::<serde_json::Value>();
    let reader_handle = std::thread::spawn(move || {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            let line = line.trim().to_string();
            if line.is_empty() {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                let _ = tx.send(v);
            }
        }
    });

    let init = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "initializationOptions": {
                "tool_profile": "full",
                "watch_drift": true
            }
        }
    });
    mcp_send(&mut stdin, init).expect("send initialize");
    let init_resp = mcp_recv_matching_id(&rx, serde_json::json!(1));
    assert_eq!(
        init_resp["result"]["protocolVersion"]
            .as_str()
            .unwrap_or(""),
        "2025-06-18"
    );

    let initialized = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    });
    mcp_send(&mut stdin, initialized).expect("send notifications/initialized");
    let drift_note = mcp_recv_notification_method(&rx, "notifications/drift_state");
    assert!(
        drift_note.get("params").is_some(),
        "expected drift_state params: {drift_note}"
    );

    mcp_send(
        &mut stdin,
        serde_json::json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {} }),
    )
    .expect("send tools/list");
    let tools_list = mcp_recv_matching_id(&rx, serde_json::json!(2));
    let tools = tools_list["result"]["tools"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(!tools.is_empty(), "expected non-empty tools list");

    mcp_send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "resources/list",
            "params": { "path": repo_str }
        }),
    )
    .expect("send resources/list");
    let resources_list = mcp_recv_matching_id(&rx, serde_json::json!(3));
    let resources = resources_list["result"]["resources"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        !resources.is_empty(),
        "expected resources list to be non-empty"
    );
    let first_uri = resources[0]["uri"].as_str().unwrap_or_default().to_string();
    assert!(!first_uri.is_empty());

    mcp_send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "resources/read",
            "params": { "path": repo_str, "uri": first_uri }
        }),
    )
    .expect("send resources/read");
    let resources_read = mcp_recv_matching_id(&rx, serde_json::json!(4));
    assert!(
        resources_read.get("result").is_some() || resources_read.get("error").is_some(),
        "expected resources/read response"
    );

    mcp_send(
        &mut stdin,
        serde_json::json!({ "jsonrpc": "2.0", "id": 5, "method": "prompts/list", "params": {} }),
    )
    .expect("send prompts/list");
    let prompts_list = mcp_recv_matching_id(&rx, serde_json::json!(5));
    let prompts = prompts_list["result"]["prompts"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(!prompts.is_empty(), "expected prompts list to be non-empty");
    let prompt_name = prompts[0]["name"].as_str().unwrap_or_default().to_string();
    assert!(!prompt_name.is_empty());

    mcp_send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "prompts/get",
            "params": { "name": prompt_name }
        }),
    )
    .expect("send prompts/get");
    let prompts_get = mcp_recv_matching_id(&rx, serde_json::json!(6));
    assert!(
        prompts_get.get("result").is_some() || prompts_get.get("error").is_some(),
        "expected prompts/get response"
    );

    for (id, name, arguments) in [
        (
            10,
            "sruja_get_repomap",
            serde_json::json!({ "path": repo_str }),
        ),
        (
            11,
            "sruja_list_architecture_index",
            serde_json::json!({ "path": repo_str, "max_tokens": 500 }),
        ),
        (
            12,
            "sruja_get_topology",
            serde_json::json!({ "path": repo_str, "id": "App", "depth": 1, "max_tokens": 1200 }),
        ),
        (
            13,
            "sruja_get_elements",
            serde_json::json!({ "path": repo_str, "ids": ["App"], "max_tokens": 1500 }),
        ),
        (
            14,
            "sruja_get_drift_state",
            serde_json::json!({ "path": repo_str }),
        ),
        (
            15,
            "sruja_reindex_memory",
            serde_json::json!({ "path": repo_str }),
        ),
        (
            16,
            "sruja_search_memory",
            serde_json::json!({ "path": repo_str, "query": "App", "limit": 5 }),
        ),
        (
            17,
            "sruja_get_memory_timeline",
            serde_json::json!({ "path": repo_str, "before": 1, "after": 1 }),
        ),
        (
            18,
            "sruja_get_context_score",
            serde_json::json!({ "path": repo_str, "format": "json" }),
        ),
        (
            19,
            "sruja_commit_evolution",
            serde_json::json!({ "path": repo_str, "id": "fitness-1", "target": "test", "result": "PASS", "detail": "ok" }),
        ),
        (
            20,
            "sruja_check_drift",
            serde_json::json!({ "path": repo_str, "architecture": "repo.sruja" }),
        ),
        (
            21,
            "sruja_propose_topology_change",
            serde_json::json!({
                "path": repo_str,
                "description": "Add worker system",
                "add_elements": ["Worker:system:Worker"],
                "add_relationships": ["App->Worker:calls"]
            }),
        ),
        (
            22,
            "sruja_get_workflow",
            serde_json::json!({ "path": repo_str, "workflow_id": "wf-mcp" }),
        ),
        (
            33,
            "sruja_workflow_summary",
            serde_json::json!({ "path": repo_str, "workflow_id": "wf-mcp" }),
        ),
        (
            34,
            "sruja_workflow_next_steps",
            serde_json::json!({ "path": repo_str, "workflow_id": "wf-mcp" }),
        ),
        (
            23,
            "sruja_record_decision_event",
            serde_json::json!({ "path": repo_str, "kind": "guardrail", "summary": "test decision event", "outcome": "ok" }),
        ),
        (
            24,
            "sruja_get_context_events",
            serde_json::json!({ "path": repo_str, "limit": 5 }),
        ),
        (
            25,
            "sruja_get_task_context",
            serde_json::json!({ "path": repo_str, "file": "src/lib.rs", "max_tokens": 2000 }),
        ),
        (
            26,
            "sruja_get_operational_context",
            serde_json::json!({ "path": repo_str }),
        ),
        (
            27,
            "sruja_get_system_context",
            serde_json::json!({ "path": repo_str }),
        ),
        (
            28,
            "sruja_list_elements",
            serde_json::json!({ "path": repo_str, "kind": "system" }),
        ),
        (
            29,
            "sruja_add_element",
            serde_json::json!({
                "path": repo_str,
                "id": "Worker2",
                "kind": "system",
                "title": "Worker 2",
                "description": "Background worker system"
            }),
        ),
        (
            30,
            "sruja_add_relationship",
            serde_json::json!({
                "path": repo_str,
                "source": "App",
                "target": "Worker2",
                "label": "calls"
            }),
        ),
        (
            31,
            "sruja_propose_change",
            serde_json::json!({
                "path": repo_str,
                "description": "Propose element via structured args",
                "add_elements": [{ "id": "Billing", "kind": "system", "label": "Billing", "technology": "Rust" }],
                "add_relationships": [{ "source": "App", "target": "Billing", "label": "calls" }]
            }),
        ),
        (
            32,
            "sruja_unknown_tool_for_coverage",
            serde_json::json!({ "path": repo_str }),
        ),
    ] {
        mcp_send(
            &mut stdin,
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": "tools/call",
                "params": { "name": name, "arguments": arguments }
            }),
        )
        .unwrap_or_else(|e| panic!("send tools/call {name} failed: {e}"));
        let resp = mcp_recv_matching_id(&rx, serde_json::json!(id));
        assert_eq!(resp.get("jsonrpc").and_then(|v| v.as_str()), Some("2.0"));
        let text = mcp_extract_text_result(&resp);
        assert!(
            !text.trim().is_empty(),
            "expected tools/call to return text: tool={name} resp={resp}"
        );
    }

    mcp_send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 40,
            "method": "tools/call",
            "params": {
                "name": "sruja_create_decision_record",
                "arguments": {
                    "path": repo_str,
                    "title": "MCP Decision",
                    "record_type": "guardrail",
                    "scope": "repo"
                }
            }
        }),
    )
    .expect("send create decision record");
    let created = mcp_recv_matching_id(&rx, serde_json::json!(40));
    let created_text = mcp_extract_text_result(&created);
    let created_json: serde_json::Value =
        serde_json::from_str(created_text.trim()).expect("create_decision_record must return JSON");
    let decision_id = created_json["id"].as_str().unwrap_or_default().to_string();
    assert!(
        !decision_id.is_empty(),
        "expected decision id: {created_text}"
    );

    mcp_send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 41,
            "method": "tools/call",
            "params": {
                "name": "sruja_link_decision_to_element",
                "arguments": {
                    "path": repo_str,
                    "decision_id": decision_id,
                    "element_id": "App"
                }
            }
        }),
    )
    .expect("send link decision");
    let link_resp = mcp_recv_matching_id(&rx, serde_json::json!(41));
    assert!(!mcp_extract_text_result(&link_resp).trim().is_empty());

    mcp_send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 42,
            "method": "tools/call",
            "params": {
                "name": "sruja_get_decisions",
                "arguments": { "path": repo_str }
            }
        }),
    )
    .expect("send get decisions");
    let decisions_resp = mcp_recv_matching_id(&rx, serde_json::json!(42));
    assert!(!mcp_extract_text_result(&decisions_resp).trim().is_empty());

    drop(stdin);
    let _ = child.wait();
    let _ = reader_handle.join();
}

#[test]
fn explore_outputs_valid_explorer_model_json() {
    let repo = create_test_repo();
    write_file(repo.path(), "repo.sruja", MINIMAL_VALID_SRUJA);
    write_file(
        repo.path(),
        "src/index.js",
        "const x = require('./helper');",
    );
    write_file(repo.path(), "src/helper.js", "module.exports = {};");

    let repo_str = repo.path().to_str().expect("utf-8");
    let (success, stdout, stderr) = run_sruja(&["explore", "-r", repo_str]);

    assert!(success, "explore should succeed: stderr={}", stderr);

    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("explore output should be valid JSON");

    assert_eq!(
        parsed["schema_version"].as_str(),
        Some("explorer/v1"),
        "schema_version mismatch"
    );
    assert!(parsed["nodes"].is_array(), "nodes should be an array");
    assert!(parsed["edges"].is_array(), "edges should be an array");
    assert!(parsed["summary"].is_object(), "summary should be an object");
    assert!(
        parsed["summary"]["total_nodes"].as_u64().unwrap_or(0) > 0,
        "should have at least one node"
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
