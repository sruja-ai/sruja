mod common;
use common::{create_test_repo, run_sruja, write_file};

#[test]
fn intent_check_json_smoke() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "Cargo.toml",
        r#"[package]
name = "dummy"
version = "0.1.0"
edition = "2021"
"#,
    );
    write_file(repo.path(), "src/lib.rs", "pub fn foo() {}");

    let repo_str = repo.path().to_str().expect("utf-8");
    let (ok, stdout, stderr) = run_sruja(&["intent", "check", "-r", repo_str, "-f", "json"]);
    assert!(ok, "intent check should succeed: stderr={}", stderr);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(parsed.get("intent_source").is_some());
    assert!(parsed.get("reality_source").is_some());
    assert!(parsed.get("drift_score").is_some());
}

#[test]
fn intent_propose_prints_adr_header() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "Cargo.toml",
        r#"[package]
name = "dummy"
version = "0.1.0"
edition = "2021"
"#,
    );
    write_file(repo.path(), "src/lib.rs", "pub fn bar() {}");

    let repo_str = repo.path().to_str().expect("utf-8");
    let (ok, stdout, stderr) = run_sruja(&["intent", "propose", "-r", repo_str]);
    assert!(ok, "intent propose should succeed: stderr={}", stderr);
    assert!(
        stdout.contains("# ADR-XXXX: Document Current Reality"),
        "should print ADR header"
    );
}
