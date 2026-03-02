//! E2E tests for `sruja analyze` command (semantic + optional runtime)

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn create_test_repo() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

fn write_file(dir: &Path, name: &str, content: &str) {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(path, content).expect("Failed to write file");
}

fn run_sruja(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_sruja"))
        .args(args)
        .output()
        .expect("Failed to run sruja");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.success(), stdout, stderr)
}

const SAMPLE_TRACES: &str = r#"[{"id":"1","name":"a","start":"2025-01-15T10:00:00Z","end":"2025-01-15T10:00:01Z","attributes":[],"children":[]}]"#;

mod analyze_command {
    use super::*;

    #[test]
    fn analyze_semantic_only_succeeds() {
        let repo = create_test_repo();
        write_file(repo.path(), "app.ts", r#"export const x = 1;"#);

        let (success, stdout, stderr) =
            run_sruja(&["analyze", "-r", repo.path().to_str().unwrap(), "-f", "text"]);

        assert!(success, "analyze should succeed: stderr={}", stderr);
        let out = format!("{} {}", stdout, stderr);
        assert!(
            out.contains("Semantic") || out.contains("Architecture Intelligence"),
            "out={}",
            out
        );
    }

    #[test]
    fn analyze_json_has_semantic_and_runtime() {
        let repo = create_test_repo();
        write_file(repo.path(), "m.ts", r#"export const m = 1;"#);
        let traces_path = repo.path().join("traces.json");
        fs::write(&traces_path, SAMPLE_TRACES).expect("write traces");

        let (success, stdout, stderr) = run_sruja(&[
            "analyze",
            "-r",
            repo.path().to_str().unwrap(),
            "-t",
            traces_path.to_str().unwrap(),
            "-f",
            "json",
        ]);

        assert!(success, "analyze -f json should succeed: stderr={}", stderr);

        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert!(json.get("semantic").is_some());
        assert!(json.get("runtime").is_some());
        let runtime = json.get("runtime").unwrap();
        assert!(!runtime.is_null());
        assert!(runtime.get("trace_count").is_some());
    }
}
