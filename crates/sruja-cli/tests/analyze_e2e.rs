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
            out.contains("CTO Report") || out.contains("Health Score"),
            "out={}",
            out
        );
    }

    #[test]
    fn analyze_json_output_succeeds() {
        let repo = create_test_repo();
        write_file(repo.path(), "m.ts", r#"export const m = 1;"#);

        let (success, stdout, stderr) =
            run_sruja(&["analyze", "-r", repo.path().to_str().unwrap(), "-f", "json"]);

        assert!(success, "analyze -f json should succeed: stderr={}", stderr);

        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert!(json.get("summary").is_some() || json.get("health_score").is_some());
    }
}
