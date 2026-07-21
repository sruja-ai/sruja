//! Integration tests for the `compress_stats` command handler.

use serde_json::json;
use std::fs;

/// Create a temporary repo fixture with a `loop.json` file containing known
/// token usage data, then invoke `compress_stats` and return the result.
async fn run_compress_stats_with_fixture(format: &str) -> Result<(), String> {
    // Create a unique temp directory to avoid test interference.
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let repo_root = tmp.path();

    // Build the fixture path: <repo>/.sruja/runs/test-run/loop.json
    let run_dir = repo_root.join(".sruja").join("runs").join("test-run");
    fs::create_dir_all(&run_dir).expect("failed to create run dir");

    // Known token usage data for deterministic assertions.
    let loop_json = json!({
        "run_id": "test-run",
        "iterations": [
            {
                "status": "ok",
                "usage": {
                    "prompt_tokens": 1000,
                    "completion_tokens": 500,
                    "cache_read_tokens": 200,
                    "cache_write_tokens": 300
                }
            },
            {
                "status": "ok",
                "usage": {
                    "prompt_tokens": 2000,
                    "completion_tokens": 800,
                    "cache_read_tokens": 400,
                    "cache_write_tokens": 600
                }
            }
        ]
    });

    let loop_path = run_dir.join("loop.json");
    fs::write(
        &loop_path,
        serde_json::to_string_pretty(&loop_json).unwrap(),
    )
    .expect("failed to write loop.json fixture");

    let repo_str = repo_root.to_str().unwrap();
    sruja_cli::commands::compress_stats::compress_stats(repo_str, Some("test-run"), format)
        .await
        .map_err(|e| format!("compress_stats returned error: {e}"))
}

// ---------- Tests ----------

#[tokio::test]
async fn compress_stats_text_format_succeeds() {
    let result = run_compress_stats_with_fixture("text").await;
    assert!(result.is_ok(), "text format should succeed: {result:?}");
}

#[tokio::test]
async fn compress_stats_json_format_succeeds() {
    let result = run_compress_stats_with_fixture("json").await;
    assert!(result.is_ok(), "json format should succeed: {result:?}");
}

#[tokio::test]
async fn compress_stats_handles_empty_iterations() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let repo_root = tmp.path();
    let run_dir = repo_root.join(".sruja").join("runs").join("empty-run");
    fs::create_dir_all(&run_dir).expect("failed to create run dir");

    let loop_json = json!({
        "run_id": "empty-run",
        "iterations": []
    });
    fs::write(
        run_dir.join("loop.json"),
        serde_json::to_string_pretty(&loop_json).unwrap(),
    )
    .expect("failed to write fixture");

    let repo_str = repo_root.to_str().unwrap();
    let result =
        sruja_cli::commands::compress_stats::compress_stats(repo_str, Some("empty-run"), "text")
            .await;

    assert!(
        result.is_ok(),
        "compress_stats failed on empty iterations: {result:?}"
    );

    tmp.close().ok();
}

#[tokio::test]
async fn compress_stats_handles_missing_usage_fields() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let repo_root = tmp.path();
    let run_dir = repo_root.join(".sruja").join("runs").join("sparse-run");
    fs::create_dir_all(&run_dir).expect("failed to create run dir");

    // Iterations where some usage fields are missing (handler defaults to 0).
    let loop_json = json!({
        "run_id": "sparse-run",
        "iterations": [
            {
                "status": "ok",
                "usage": {
                    "prompt_tokens": 500
                    // no completion_tokens, cache_read_tokens, cache_write_tokens
                }
            }
        ]
    });
    fs::write(
        run_dir.join("loop.json"),
        serde_json::to_string_pretty(&loop_json).unwrap(),
    )
    .expect("failed to write fixture");

    let repo_str = repo_root.to_str().unwrap();
    let result =
        sruja_cli::commands::compress_stats::compress_stats(repo_str, Some("sparse-run"), "text")
            .await;

    assert!(
        result.is_ok(),
        "compress_stats failed on missing usage fields: {result:?}"
    );

    tmp.close().ok();
}

#[tokio::test]
async fn compress_stats_returns_error_for_missing_run() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let repo_root = tmp.path();

    let repo_str = repo_root.to_str().unwrap();
    let result = sruja_cli::commands::compress_stats::compress_stats(
        repo_str,
        Some("nonexistent-run"),
        "text",
    )
    .await;

    assert!(result.is_err(), "expected error for missing run dir");

    tmp.close().ok();
}

#[tokio::test]
async fn compress_stats_nested_phases_token_aggregation() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let repo_root = tmp.path();
    let run_dir = repo_root.join(".sruja").join("runs").join("nested-run");
    fs::create_dir_all(&run_dir).expect("failed to create run dir");

    // Iteration with top-level usage AND nested comprehension/plan/critique phases.
    let loop_json = json!({
        "run_id": "nested-run",
        "iterations": [
            {
                "status": "ok",
                "usage": {
                    "prompt_tokens": 1000,
                    "completion_tokens": 500,
                    "cache_read_tokens": 0,
                    "cache_write_tokens": 0
                },
                "phases": {
                    "comprehension": {
                        "usage": {
                            "prompt_tokens": 100,
                            "completion_tokens": 50,
                            "cache_read_tokens": 10,
                            "cache_write_tokens": 5
                        }
                    },
                    "plan": {
                        "usage": {
                            "prompt_tokens": 200,
                            "completion_tokens": 80,
                            "cache_read_tokens": 20,
                            "cache_write_tokens": 10
                        }
                    },
                    "critique": {
                        "usage": {
                            "prompt_tokens": 150,
                            "completion_tokens": 60,
                            "cache_read_tokens": 15,
                            "cache_write_tokens": 8
                        }
                    }
                }
            }
        ]
    });

    fs::write(
        run_dir.join("loop.json"),
        serde_json::to_string_pretty(&loop_json).unwrap(),
    )
    .expect("failed to write fixture");

    let repo_str = repo_root.to_str().unwrap();
    let result =
        sruja_cli::commands::compress_stats::compress_stats(repo_str, Some("nested-run"), "json")
            .await;

    assert!(
        result.is_ok(),
        "compress_stats failed on nested phases: {result:?}"
    );

    tmp.close().ok();
}
