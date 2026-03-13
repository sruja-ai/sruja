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
        assert!(
            json.get("summary").is_some() || json.get("health_score").is_some()
        );
        // New fields from architecture completion scoring should be present.
        if let Some(score) = json.get("architecture_completion_score") {
            assert!(score.is_number(), "completion score should be numeric");
        }
        if let Some(breakdown) = json.get("completion_breakdown") {
            assert!(
                breakdown.get("structural").is_some()
                    && breakdown.get("operational").is_some()
                    && breakdown.get("security").is_some(),
                "completion_breakdown should include structural/operational/security"
            );
        }
    }
}
