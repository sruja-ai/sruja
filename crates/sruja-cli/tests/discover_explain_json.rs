mod common;
use common::{create_test_repo, run_sruja, write_file};

#[test]
fn discover_explain_json_has_expected_sections() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "package.json",
        r#"{"dependencies":{"express":"4.18.0"}}"#,
    );
    write_file(
        repo.path(),
        "src/server.ts",
        r#"
import { query } from "./db";
export function start() { return query(); }
"#,
    );
    write_file(
        repo.path(),
        "src/db.ts",
        r#"export function query() { return []; }"#,
    );

    let repo_str = repo.path().to_str().expect("utf-8");
    let (ok, stdout, stderr) = run_sruja(&[
        "discover",
        "--explain",
        "-r",
        repo_str,
        "--format",
        "json",
    ]);
    assert!(ok, "discover --explain json should succeed: stderr={}", stderr);

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(parsed.get("context").is_some());
    assert!(parsed.get("kind_counts").is_some());
    assert!(parsed.get("reasoning").is_some());
    assert!(parsed.get("top_directories").is_some());
    assert!(parsed.get("key_elements").is_some());
    assert!(parsed.get("key_relationships").is_some());
    assert!(parsed.get("confidence").is_some());
    assert!(parsed.get("next_steps").is_some());
}
