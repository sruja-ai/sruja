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
fn export_json_handles_missing_file() {
    let (success, _stdout, _stderr) = run_sruja(&["export", "json", "/nonexistent/file.sruja"]);

    assert!(!success, "export should fail for missing file");
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
