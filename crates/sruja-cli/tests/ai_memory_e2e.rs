//! E2E tests for `sruja ai` memory and feedback loop.
//!
//! Seeds .sruja/memory with a fact and interaction, runs feedback, then verifies fact updated.

use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn run_sruja(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_sruja"))
        .args(args)
        .output()
        .expect("Failed to run sruja");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.success(), stdout, stderr)
}

/// Run sruja with LLM env vars cleared so provider resolution fails (for fallback tests).
fn run_sruja_no_llm(args: &[&str]) -> (bool, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sruja"));
    cmd.args(args);
    for key in [
        "OPENAI_API_KEY",
        "OPENROUTER_API_KEY",
        "ANTHROPIC_API_KEY",
        "GEMINI_API_KEY",
        "GOOGLE_API_KEY",
        "SRUJA_LLM_API_KEY",
    ] {
        cmd.env_remove(key);
    }
    let output = cmd.output().expect("Failed to run sruja");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.success(), stdout, stderr)
}

/// Create a temp dir with .sruja/memory/facts.jsonl and interactions.jsonl seeded.
fn seed_memory_dir() -> TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path();
    let memory_dir = root.join(".sruja").join("memory");
    fs::create_dir_all(&memory_dir).expect("create .sruja/memory");

    // One fact: confidence 0.7, candidate
    let fact_line = r#"{"fact_id":"fact_demo1","statement":"Requests enter via API gateway.","fact_type":"flow","status":"candidate","confidence":0.7,"source":"llm","repo":"/tmp","commit_sha":"abc1234","evidence":[],"tags":[],"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}"#;
    fs::write(memory_dir.join("facts.jsonl"), format!("{}\n", fact_line)).expect("write facts");

    // One interaction referencing that fact
    let interaction_line = r#"{"answer_id":"ans_demo1","question":"How does request flow work?","response_markdown":"...","used_fact_ids":["fact_demo1"],"new_fact_ids":[],"confidence":0.7,"commit_sha":"abc1234","created_at":"2026-01-01T00:00:00Z"}"#;
    fs::write(
        memory_dir.join("interactions.jsonl"),
        format!("{}\n", interaction_line),
    )
    .expect("write interactions");

    dir
}

#[test]
fn feedback_updates_fact_confidence_and_status() {
    let repo = seed_memory_dir();
    let root = repo.path();
    let root_str = root.to_str().expect("path");

    // Run: sruja ai feedback -r <root> --answer-id ans_demo1 --fact-id fact_demo1 --verdict wrong
    let (ok, stdout, stderr) = run_sruja(&[
        "ai",
        "feedback",
        "-r",
        root_str,
        "--answer-id",
        "ans_demo1",
        "--fact-id",
        "fact_demo1",
        "--verdict",
        "wrong",
    ]);
    assert!(
        ok,
        "feedback should succeed: stdout={} stderr={}",
        stdout, stderr
    );

    // Read facts.jsonl and assert confidence dropped (0.7 - 0.35 = 0.35), status = disputed
    let facts_path = root.join(".sruja").join("memory").join("facts.jsonl");
    let content = fs::read_to_string(&facts_path).expect("read facts");
    let line = content.lines().next().expect("one line");
    let fact: serde_json::Value = serde_json::from_str(line).expect("parse fact");
    let conf = fact["confidence"].as_f64().expect("confidence");
    let status = fact["status"].as_str().expect("status");
    assert!(
        (conf - 0.35).abs() < 0.01,
        "confidence should be ~0.35 after wrong verdict, got {}",
        conf
    );
    assert_eq!(status, "disputed", "status should be disputed");
}

/// With no LLM API key, explain must succeed (exit 0) and output fallback message + evidence preview.
#[test]
fn explain_fallback_when_no_llm() {
    let repo = tempfile::tempdir().expect("temp dir");
    let root = repo.path();
    let root_str = root.to_str().expect("path");
    // Minimal repo so scan runs: one source file
    std::fs::create_dir_all(root.join("src")).ok();
    std::fs::write(root.join("src/lib.rs"), "pub fn main() {}").ok();

    let (ok, stdout, stderr) =
        run_sruja_no_llm(&["ai", "explain", "-r", root_str, "--topic", "request flow"]);
    assert!(
        ok,
        "explain without LLM should exit 0 (fallback): stderr={}",
        stderr
    );
    let out = stdout.to_lowercase();
    assert!(
        out.contains("llm unavailable")
            || out.contains("evidence from scan")
            || out.contains("evidence (from repository"),
        "output should contain fallback or evidence: {}",
        stdout
    );
}

/// Feedback with invalid fact_id must fail (exit != 0), report error, and not modify facts.jsonl.
#[test]
fn feedback_invalid_fact_id_fails_and_does_not_modify_facts() {
    let repo = seed_memory_dir();
    let root = repo.path();
    let root_str = root.to_str().expect("path");
    let facts_path = root.join(".sruja").join("memory").join("facts.jsonl");
    let before = fs::read_to_string(&facts_path).expect("read facts before");

    let (ok, stdout, stderr) = run_sruja(&[
        "ai",
        "feedback",
        "-r",
        root_str,
        "--answer-id",
        "ans_demo1",
        "--fact-id",
        "fact_nonexistent",
        "--verdict",
        "wrong",
    ]);
    assert!(!ok, "feedback with invalid fact_id should fail");
    let err = format!("{}{}", stdout, stderr);
    assert!(
        err.to_lowercase().contains("fact not found") || err.contains("fact_nonexistent"),
        "error should mention fact not found: {}",
        err
    );

    let after = fs::read_to_string(&facts_path).expect("read facts after");
    assert_eq!(before, after, "facts.jsonl must be unchanged");
}

/// Feedback with invalid verdict must fail (exit != 0) and report error.
#[test]
fn feedback_invalid_verdict_fails() {
    let repo = seed_memory_dir();
    let root = repo.path();
    let root_str = root.to_str().expect("path");

    let (ok, stdout, stderr) = run_sruja(&[
        "ai",
        "feedback",
        "-r",
        root_str,
        "--answer-id",
        "ans_demo1",
        "--fact-id",
        "fact_demo1",
        "--verdict",
        "invalid",
    ]);
    assert!(!ok, "feedback with invalid verdict should fail");
    let err = format!("{}{}", stdout, stderr);
    assert!(
        err.to_lowercase().contains("verdict")
            && (err.to_lowercase().contains("invalid") || err.to_lowercase().contains("correct")),
        "error should mention verdict: {}",
        err
    );
}

/// Memory with no .sruja/memory must succeed and report 0 facts (no crash on missing dir).
#[test]
fn memory_command_empty_when_no_memory_dir() {
    let repo = tempfile::tempdir().expect("temp dir");
    let root = repo.path();
    let root_str = root.to_str().expect("path");
    // Do not create .sruja/memory

    let (ok, stdout, stderr) = run_sruja(&["ai", "memory", "-r", root_str, "-f", "json"]);
    assert!(
        ok,
        "ai memory with no memory dir should succeed: stderr={}",
        stderr
    );
    let out: serde_json::Value = serde_json::from_str(stdout.trim()).expect("parse memory json");
    assert_eq!(out["facts_count"].as_u64(), Some(0));
    assert_eq!(out["interactions_count"].as_u64(), Some(0));
}

#[test]
fn memory_command_lists_seeded_facts() {
    let repo = seed_memory_dir();
    let root = repo.path();
    let root_str = root.to_str().expect("path");

    let (ok, stdout, _) = run_sruja(&["ai", "memory", "-r", root_str, "-f", "json"]);
    assert!(ok, "ai memory should succeed");

    let out: serde_json::Value = serde_json::from_str(stdout.trim()).expect("parse memory json");
    assert_eq!(out["facts_count"].as_u64(), Some(1), "one fact");
    let quality = out["quality"].as_object().expect("quality object");
    assert_eq!(quality["candidate"].as_u64(), Some(1), "one candidate fact");
    assert_eq!(quality["confirmed"].as_u64(), Some(0));
    let facts = out["facts"].as_array().expect("facts array");
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0]["fact_id"].as_str(), Some("fact_demo1"));
}
