//! Integration tests for extraction CLI: lint --format json, discover --context --format json.
//! Ensures machine-readable output contracts stay stable for DX and extraction quality.

mod common;
use common::{create_test_repo, run_sruja, write_file};

#[test]
fn lint_format_json_returns_valid_schema() {
    // Use repo's valid example (path relative to workspace root when run from crates/sruja-cli)
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let example = manifest_dir
        .join("..")
        .join("..")
        .join("book")
        .join("valid-examples")
        .join("getting-started.sruja");
    if !example.exists() {
        eprintln!("Skipping lint_format_json: {} not found", example.display());
        return;
    }
    let path_str = example.to_str().expect("path utf-8");
    let (success, stdout, stderr) = run_sruja(&["lint", "--format", "json", path_str]);

    assert!(
        success || !stdout.is_empty(),
        "lint should produce output: stderr={}",
        stderr
    );
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("lint --format json must be valid JSON");
    let obj = parsed.as_object().expect("root must be object");
    assert!(obj.contains_key("ok"), "must have 'ok'");
    assert!(obj.contains_key("error_count"), "must have 'error_count'");
    assert!(
        obj.contains_key("warning_count"),
        "must have 'warning_count'"
    );
    assert!(obj.contains_key("diagnostics"), "must have 'diagnostics'");
    let diags = obj
        .get("diagnostics")
        .and_then(|d| d.as_array())
        .expect("diagnostics must be array");
    for d in diags {
        let d = d.as_object().expect("each diagnostic is object");
        assert!(
            d.contains_key("code") || d.contains_key("message"),
            "diagnostic must have code or message"
        );
    }
}

#[test]
fn discover_context_format_json_returns_valid_schema() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo = manifest_dir.join("..").join("..");
    let repo_str = repo.to_str().expect("path utf-8");
    let (success, stdout, stderr) =
        run_sruja(&["discover", "--context", "-r", repo_str, "--format", "json"]);

    assert!(
        success,
        "discover --context --format json should succeed: stderr={}",
        stderr
    );
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("discover output must be valid JSON");
    let obj = parsed.as_object().expect("root must be object");
    assert!(obj.contains_key("repo"), "must have 'repo'");
    assert!(obj.contains_key("components"), "must have 'components'");
    assert!(obj.contains_key("edges"), "must have 'edges'");
    assert!(
        obj.contains_key("primary_language"),
        "must have 'primary_language'"
    );
    assert!(
        obj.contains_key("suggested_areas"),
        "must have 'suggested_areas'"
    );
}

#[test]
fn discover_explain_format_json_returns_valid_schema() {
    let temp = create_test_repo();
    write_file(
        temp.path(),
        "package.json",
        r#"{"dependencies":{"express":"4.18.0"}}"#,
    );
    write_file(
        temp.path(),
        "src/server.ts",
        r#"
import { query } from "./db";
export function start() { return query(); }
"#,
    );
    write_file(
        temp.path(),
        "src/db.ts",
        r#"export function query() { return []; }"#,
    );

    let repo_str = temp.path().to_str().expect("path utf-8");
    let (success, stdout, stderr) =
        run_sruja(&["discover", "--explain", "-r", repo_str, "--format", "json"]);

    assert!(
        success,
        "discover --explain --format json should succeed: stderr={}",
        stderr
    );
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("discover explanation output must be valid JSON");
    let obj = parsed.as_object().expect("root must be object");
    assert!(obj.contains_key("context"), "must have 'context'");
    assert!(obj.contains_key("reasoning"), "must have 'reasoning'");
    assert!(obj.contains_key("key_elements"), "must have 'key_elements'");
    assert!(obj.contains_key("confidence"), "must have 'confidence'");
    assert!(obj.contains_key("next_steps"), "must have 'next_steps'");
}

#[test]
fn context_format_json_returns_task_context_v1_schema_version() {
    let temp = create_test_repo();
    write_file(temp.path(), "src/main.rs", "fn main() {}\n");
    let repo_str = temp.path().to_str().expect("path utf-8");
    let (success, stdout, stderr) = run_sruja(&["ai-context", "-r", repo_str, "-f", "json"]);

    assert!(
        success,
        "context --format json should succeed: stderr={}",
        stderr
    );
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("context output must be valid JSON");
    let obj = parsed.as_object().expect("root must be object");
    assert_eq!(
        obj.get("schema_version").and_then(|v| v.as_str()),
        Some("task_context/v1")
    );
    assert!(
        obj.contains_key("selection_reason"),
        "must have 'selection_reason'"
    );
    assert!(
        obj.contains_key("focus_elements"),
        "must have 'focus_elements'"
    );
    assert!(
        obj.contains_key("impacted_systems"),
        "must have 'impacted_systems'"
    );
    assert!(
        obj.contains_key("impacted_containers"),
        "must have 'impacted_containers'"
    );
    assert!(
        obj.contains_key("impacted_components"),
        "must have 'impacted_components'"
    );
    assert!(
        obj.contains_key("hydrated_files"),
        "must have 'hydrated_files'"
    );
    assert!(obj.contains_key("truth_status"), "must have 'truth_status'");
    assert!(obj.contains_key("confidence"), "must have 'confidence'");
}

#[test]
fn lint_on_temp_sruja_file() {
    let temp = create_test_repo();
    let minimal = r#"person = kind "Person"
container = kind "Container"
system = kind "System"
User = person "User" {}
App = system "Temp" {
  A = container "A" { technology "Rust" description "A" }
  B = container "B" { technology "Rust" description "B" }
}
App.A -> App.B "calls"
"#;
    write_file(temp.path(), "arch.sruja", minimal);
    let sruja_path = temp.path().join("arch.sruja");
    let path_str = sruja_path.to_str().expect("path utf-8");
    let (success, _, stderr) = run_sruja(&["lint", path_str]);
    assert!(
        success,
        "lint on temp .sruja should succeed: stderr={}",
        stderr
    );
}

#[test]
fn test_scan_based_exports() {
    let temp = create_test_repo();
    write_file(temp.path(), "src/main.rs", "fn main() {}\n");
    let repo_str = temp.path().to_str().expect("path utf-8");

    // GraphML export
    let (success_g, stdout_g, stderr_g) = run_sruja(&[
        "export",
        "graphml",
        "dummy.sruja",
        "--from-scan",
        "-r",
        repo_str,
    ]);
    assert!(success_g, "GraphML export failed: {}", stderr_g);
    assert!(
        stdout_g.contains("<graphml"),
        "Output should be GraphML XML"
    );

    // Neo4j export
    let (success_n, stdout_n, stderr_n) = run_sruja(&[
        "export",
        "neo4j",
        "dummy.sruja",
        "--from-scan",
        "-r",
        repo_str,
    ]);
    assert!(success_n, "Neo4j export failed: {}", stderr_n);
    assert!(
        stdout_n.contains("CREATE (n:Component"),
        "Output should contain CREATE statements"
    );

    // Obsidian export
    let obsidian_dir = temp.path().join("obsidian-vault");
    let obsidian_dir_str = obsidian_dir.to_str().expect("path utf-8");
    let (success_o, _stdout_o, stderr_o) = run_sruja(&[
        "export",
        "obsidian",
        "dummy.sruja",
        "--from-scan",
        "-r",
        repo_str,
        "--output-dir",
        obsidian_dir_str,
    ]);
    assert!(success_o, "Obsidian export failed: {}", stderr_o);
    assert!(
        obsidian_dir.join("_Index.md").exists(),
        "Obsidian _Index.md should be created"
    );
}
