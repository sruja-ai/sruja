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

    let (success, stdout, stderr) = run_sruja(&["status", "-r", repo_str, "-f", "json"]);

    assert!(success, "status should succeed: stderr={}", stderr);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(parsed.get("truth_status").is_some());
    assert!(parsed.get("baseline").is_some());
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
    let repo_str = repo.path().to_str().expect("utf-8");

    let bundle = repo
        .path()
        .join("repo.bundle.json")
        .to_str()
        .expect("utf-8")
        .to_string();

    let (pub_success, _pub_stdout, pub_stderr) =
        run_sruja(&["publish", "-r", repo_str, "-o", &bundle]);
    assert!(pub_success, "publish should succeed: stderr={}", pub_stderr);
    assert!(std::path::Path::new(&bundle).exists());

    let index = repo
        .path()
        .join("system.index.json")
        .to_str()
        .expect("utf-8")
        .to_string();
    let (compose_success, _compose_stdout, compose_stderr) =
        run_sruja(&["compose", "-i", &bundle, "-o", &index]);
    assert!(
        compose_success,
        "compose should succeed: stderr={}",
        compose_stderr
    );
    assert!(std::path::Path::new(&index).exists());
}

#[test]
fn sources_and_knowledge_commands_work() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(
        repo.path(),
        "repo.sruja",
        r#"
system = kind "System"

App = system "App" {
  description "Main"
  doc ".sruja/knowledge/App.md"
  source openapi "api/openapi.yaml"
}

Worker = system "Worker" { description "Background jobs" }

App -> Worker "queues jobs"
"#,
    );
    write_file(
        repo.path(),
        "api/openapi.yaml",
        "openapi: 3.0.0\ninfo:\n  title: API\n  version: 1.0.0\n",
    );
    write_file(repo.path(), ".sruja/knowledge/App.md", "App knowledge\n");

    let repo_str = repo.path().to_str().expect("utf-8");

    let (sources_success, sources_stdout, sources_stderr) = run_sruja(&[
        "sources",
        "-r",
        repo_str,
        "-a",
        "repo.sruja",
        "--format",
        "json",
    ]);
    assert!(
        sources_success,
        "sources should succeed: stderr={}",
        sources_stderr
    );
    let sources_json: serde_json::Value =
        serde_json::from_str(sources_stdout.trim()).expect("valid JSON");
    assert!(sources_json.as_array().is_some());

    let (knowledge_list_success, knowledge_list_stdout, knowledge_list_stderr) =
        run_sruja(&["knowledge", "list", "-r", repo_str, "-a", "repo.sruja"]);
    assert!(
        knowledge_list_success,
        "knowledge list should succeed: stderr={}",
        knowledge_list_stderr
    );
    assert!(knowledge_list_stdout.contains("App"));

    let (knowledge_show_success, knowledge_show_stdout, knowledge_show_stderr) = run_sruja(&[
        "knowledge",
        "show",
        "App",
        "-r",
        repo_str,
        "-a",
        "repo.sruja",
    ]);
    assert!(
        knowledge_show_success,
        "knowledge show should succeed: stderr={}",
        knowledge_show_stderr
    );
    assert!(knowledge_show_stdout.contains("App knowledge"));

    let (knowledge_gaps_success, knowledge_gaps_stdout, knowledge_gaps_stderr) =
        run_sruja(&["knowledge", "gaps", "-r", repo_str, "-a", "repo.sruja"]);
    assert!(
        knowledge_gaps_success,
        "knowledge gaps should succeed: stderr={}",
        knowledge_gaps_stderr
    );
    assert!(knowledge_gaps_stdout.contains("Worker"));
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
