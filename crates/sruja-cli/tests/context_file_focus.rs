mod common;
use common::{create_test_repo, run_sruja, write_file};

#[test]
fn context_json_with_file_focus_selects_file_match() {
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
    let (ok, stdout, stderr) = run_sruja(&[
        "context",
        "-r",
        repo_str,
        "-f",
        "json",
        "--file",
        "src/app.ts",
        "--depth",
        "2",
        "--max-tokens",
        "500",
    ]);
    assert!(ok, "context should succeed: stderr={}", stderr);

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(
        parsed
            .get("schema_version")
            .and_then(|v| v.as_str())
            .unwrap_or_default(),
        "task_context/v1"
    );
    assert_eq!(
        parsed
            .get("selection_reason")
            .and_then(|v| v.get("primary"))
            .and_then(|v| v.as_str())
            .unwrap_or_default(),
        "file"
    );

    let focus_elements = parsed
        .get("focus_elements")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(!focus_elements.is_empty(), "expected focus elements");

    let first_evidence_kind = focus_elements[0]
        .get("evidence")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.get("kind"))
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    assert_eq!(first_evidence_kind, "file_match");

    let locator_path = focus_elements[0]
        .get("evidence")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.get("locator"))
        .and_then(|v| v.get("path"))
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    assert_eq!(locator_path, "src/app.ts");

    let grounding_trace = parsed
        .get("grounding_trace")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        !grounding_trace.is_empty(),
        "expected non-empty grounding_trace"
    );
    assert_eq!(
        grounding_trace[0]
            .get("phase")
            .and_then(|v| v.as_str())
            .unwrap_or_default(),
        "input"
    );
    assert_eq!(
        grounding_trace
            .iter()
            .any(|s| s.get("phase").and_then(|v| v.as_str()) == Some("focus_resolution")),
        true
    );
}

#[test]
fn context_json_with_query_includes_semantic_candidates_in_grounding_trace() {
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
    let (ok, stdout, stderr) = run_sruja(&[
        "context",
        "-r",
        repo_str,
        "-f",
        "json",
        "--query",
        "helper function",
        "--depth",
        "1",
        "--max-tokens",
        "500",
    ]);
    assert!(ok, "context should succeed: stderr={}", stderr);

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(
        parsed
            .get("schema_version")
            .and_then(|v| v.as_str())
            .unwrap_or_default(),
        "task_context/v1"
    );
    assert_eq!(
        parsed
            .get("selection_reason")
            .and_then(|v| v.get("primary"))
            .and_then(|v| v.as_str())
            .unwrap_or_default(),
        "query"
    );

    let grounding_trace = parsed
        .get("grounding_trace")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        !grounding_trace.is_empty(),
        "expected non-empty grounding_trace"
    );

    let mut saw_candidates = false;
    for step in &grounding_trace {
        let Some(details) = step.get("details") else {
            continue;
        };
        let Some(candidates) = details.get("candidates").and_then(|v| v.as_array()) else {
            continue;
        };
        if !candidates.is_empty() {
            saw_candidates = true;
            break;
        }
    }
    assert!(
        saw_candidates,
        "expected grounding_trace to include semantic candidates details"
    );
}
