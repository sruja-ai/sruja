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
fn focus_json_provides_briefing_for_file() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

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
