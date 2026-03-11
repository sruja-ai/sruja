//! E2E tests for `sruja analyze` command (semantic + optional runtime)

mod common;
use common::*;

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
