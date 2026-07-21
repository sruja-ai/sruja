mod common;
use common::{create_test_repo, run_sruja, write_file};

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
