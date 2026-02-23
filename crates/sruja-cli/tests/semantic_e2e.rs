//! E2E tests for `sruja semantic` command

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

mod semantic_command {
    use super::*;

    #[test]
    fn semantic_runs_successfully() {
        let repo = create_test_repo();

        write_file(
            repo.path(),
            "app.ts",
            r#"
import { service } from './service';
export function app() { return service(); }
"#,
        );
        write_file(repo.path(), "service.ts", r#"export function service() { return 'hello'; }"#);

        let (success, stdout, stderr) =
            run_sruja(&["semantic", "-r", repo.path().to_str().unwrap(), "-f", "text"]);

        assert!(success, "semantic should succeed: stderr={}", stderr);
        let out = format!("{} {}", stdout, stderr);
        assert!(
            out.contains("component") || out.contains("context") || out.contains("Semantic"),
            "Output should mention semantic analysis. out={}",
            out
        );
    }

    #[test]
    fn semantic_json_output_structure() {
        let repo = create_test_repo();

        write_file(
            repo.path(),
            "main.ts",
            r#"
import { util } from './util';
export function main() { return util(); }
"#,
        );
        write_file(repo.path(), "util.ts", r#"export function util() { return 42; }"#);

        let (success, stdout, stderr) = run_sruja(&[
            "semantic",
            "-r",
            repo.path().to_str().unwrap(),
            "-f",
            "json",
        ]);

        assert!(success, "semantic -f json should succeed: stderr={}", stderr);

        let json: serde_json::Value =
            serde_json::from_str(&stdout).expect("Output should be valid JSON");
        assert!(json.get("component_count").is_some(), "JSON should have component_count");
        assert!(json.get("context_count").is_some(), "JSON should have context_count");
        assert!(
            json.get("hidden_coupling_count").is_some(),
            "JSON should have hidden_coupling_count"
        );
    }

    #[test]
    fn semantic_handles_empty_repo() {
        let repo = create_test_repo();

        let (success, _stdout, stderr) =
            run_sruja(&["semantic", "-r", repo.path().to_str().unwrap(), "-f", "text"]);

        assert!(success, "semantic on empty repo should succeed: stderr={}", stderr);
    }
}
