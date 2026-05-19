//! E2E tests for grounded architecture authoring (`sruja author`).

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn sruja_bin() -> Command {
    Command::cargo_bin("sruja").unwrap()
}

#[test]
fn author_evidence_writes_capped_bundle_with_communities() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path();

    fs::write(
        repo.join("Cargo.toml"),
        r#"
[package]
name = "demo"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1"
"#,
    )
    .unwrap();
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::write(
        repo.join("src/main.rs"),
        r#"
fn main() {
    println!("hello");
}
"#,
    )
    .unwrap();

    sruja_bin()
        .args(["author", "evidence", "-r"])
        .arg(repo)
        .assert()
        .success();

    let evidence_path = repo.join(".sruja/author_evidence.json");
    assert!(evidence_path.exists(), "author_evidence.json should exist");

    let bytes = fs::read(&evidence_path).unwrap();
    assert!(
        bytes.len() <= 80 * 1024,
        "evidence should stay under 80KB cap, got {} bytes",
        bytes.len()
    );

    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["schema_version"], "author_evidence/v1");

    let empty: Vec<serde_json::Value> = vec![];
    let communities = json["communities"].as_array().unwrap_or(&empty);
    if !communities.is_empty() {
        assert_eq!(communities[0]["edge_basis"], "import_graph");
    }
}

#[test]
fn author_propose_does_not_write_repo_sruja() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path();

    fs::write(
        repo.join("Cargo.toml"),
        r#"
[package]
name = "demo"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::write(repo.join("src/lib.rs"), "pub fn demo() {}\n").unwrap();

    let repo_sruja = repo.join("repo.sruja");
    fs::write(
        &repo_sruja,
        r#"
import { * } from 'sruja.ai/stdlib'

Demo = system "Demo" {
  Lib = container "Lib" {
    technology "Rust"
    description "Demo lib"
  }
}
"#,
    )
    .unwrap();

    let before = fs::read_to_string(&repo_sruja).unwrap();

    let proposal_json = r#"{
  "id": "test-prop",
  "title": "Test",
  "description": "Synthetic proposal from test",
  "created_at": "2026-01-01T00:00:00Z",
  "status": "draft",
  "changes": [],
  "open_questions": ["Is this boundary correct?"],
  "evidence_refs": []
}"#;

    let proposal_fixture = repo.join("_proposal_stdout.json");
    fs::write(&proposal_fixture, proposal_json).unwrap();
    let enrich_cmd = format!("cat {}", proposal_fixture.display());

    sruja_bin()
        .args(["author", "propose", "-r"])
        .arg(repo)
        .arg("--enrich-cmd")
        .arg(&enrich_cmd)
        .assert()
        .success();

    let after = fs::read_to_string(&repo_sruja).unwrap();
    assert_eq!(
        before, after,
        "repo.sruja must not be modified by author propose"
    );

    assert!(
        repo.join(".sruja/proposals/test-prop.json").exists(),
        "proposal should be written under .sruja/proposals/"
    );
}
