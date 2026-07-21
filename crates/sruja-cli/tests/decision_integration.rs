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
