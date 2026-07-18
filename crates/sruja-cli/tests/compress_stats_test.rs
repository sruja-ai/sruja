//! Integration tests for the `compress_stats` command handler.

use serde_json::json;
use std::fs;
use std::path::PathBuf;

/// Create a temporary repo fixture with a `loop.json` file containing known
/// token usage data, then invoke `compress_stats` and capture stdout.
///
/// Returns captured stdout as a `String`.
async fn run_compress_stats_with_fixture(format: &str) -> String {
    // Create a unique temp directory to avoid test interference.
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let repo_root = tmp.path();

    // Build the fixture path: <repo>/.sruja/runs/test-run/loop.json
    let run_dir = repo_root.join(".sruja").join("runs").join("test-run");
    fs::create_dir_all(&run_dir).expect("failed to create run dir");

    // Known token usage data for deterministic assertions.
    //
    // 2 iterations with carefully chosen numbers:
    //   Iter 1: input=1000, output=500, cache_read=200, cache_write=300
    //   Iter 2: input=2000, output=800, cache_read=400, cache_write=600
    //
    // Totals:
    //   input   = 3000
    //   output  = 1300
    //   cache_read  = 600
    //   cache_write = 900
    //   total   = 5800
    //   avg/iter = 2900
    //
    // Cost (Claude Sonnet pricing from handler):
    //   input       = 3000 / 1M * 3.0   = 0.009
    //   output      = 1300 / 1M * 15.0  = 0.0195
    //   cache_read  =  600 / 1M * 0.3   = 0.00018
    //   cache_write =  900 / 1M * 3.75  = 0.003375
    //   total cost  = 0.032055
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

    // Capture stdout by redirecting it. We use a pipe via `gag` or, more
    // portably, we capture by having the function write to a Vec<u8>.
    // Since compress_stats writes to stdout directly, we redirect it.
    let buf = capture_stdout(|| {
        // We need a runtime for the async call.
        let rt = tokio::runtime::Runtime::new().unwrap();
        let repo_str = repo_root.to_str().unwrap();
        rt.block_on(async {
            sruja_cli::commands::compress_stats::compress_stats(repo_str, Some("test-run"), format)
                .await
                .expect("compress_stats returned error");
        });
    });

    tmp.close().expect("failed to clean up temp dir");
    buf
}

/// Capture stdout from a closure by temporarily redirecting fd 1.
fn capture_stdout(f: impl FnOnce()) -> String {
    use std::io::Read;
    use std::os::unix::io::AsRawFd;

    let (mut pipe_read, pipe_write) = os_pipe::pipe().expect("failed to create pipe");

    // Save original stdout fd.
    let old_stdout = unsafe { libc::dup(libc::STDOUT_FILENO) };
    assert!(old_stdout >= 0, "failed to dup stdout");

    // Redirect stdout to the write end of the pipe.
    unsafe {
        libc::dup2(pipe_write.as_raw_fd(), libc::STDOUT_FILENO);
    }
    // Drop our Rust handle to the write end; fd 1 still points to it.
    drop(pipe_write);

    f();

    // Restore original stdout — this closes the pipe's write end on fd 1.
    unsafe {
        libc::dup2(old_stdout, libc::STDOUT_FILENO);
        libc::close(old_stdout);
    }

    // All write ends are now closed; read everything from the pipe.
    let mut bytes = Vec::new();
    pipe_read
        .read_to_end(&mut bytes)
        .expect("failed to read captured stdout");

    String::from_utf8(bytes).expect("stdout was not valid UTF-8")
}

// ---------- Tests ----------

#[tokio::test]
async fn compress_stats_text_format_shows_token_counts() {
    let output = run_compress_stats_with_fixture("text").await;

    // Check header
    assert!(
        output.contains("Token Usage Statistics"),
        "missing header, got:\n{output}"
    );

    // Run ID
    assert!(
        output.contains("test-run"),
        "missing run id, got:\n{output}"
    );

    // Iterations count
    assert!(
        output.contains("Iterations: 2"),
        "missing iteration count, got:\n{output}"
    );

    // Token totals
    assert!(
        output.contains("Input:         3000"),
        "missing input total, got:\n{output}"
    );
    assert!(
        output.contains("Output:        1300"),
        "missing output total, got:\n{output}"
    );
    assert!(
        output.contains("Cache Read:    600"),
        "missing cache read total, got:\n{output}"
    );
    assert!(
        output.contains("Cache Write:   900"),
        "missing cache write total, got:\n{output}"
    );
    assert!(
        output.contains("Total:         5800"),
        "missing total tokens, got:\n{output}"
    );
    assert!(
        output.contains("Avg/Iteration: 2900"),
        "missing avg per iteration, got:\n{output}"
    );
}

#[tokio::test]
async fn compress_stats_text_format_shows_cost_breakdown() {
    let output = run_compress_stats_with_fixture("text").await;

    // Cost section exists
    assert!(
        output.contains("Cost Estimate (USD)"),
        "missing cost header, got:\n{output}"
    );

    // Total cost line — the handler formats with 6 decimal places
    // 0.032055
    assert!(
        output.contains("Total:  $0.032055"),
        "missing total cost, got:\n{output}"
    );
}

#[tokio::test]
async fn compress_stats_json_format_contains_all_fields() {
    let output = run_compress_stats_with_fixture("json").await;

    // Parse the JSON output (it's printed as pretty JSON)
    let v: serde_json::Value =
        serde_json::from_str(&output).expect("json output is not valid JSON");

    // schema_version
    assert_eq!(v["schema_version"], "compress_stats/v1");

    // run_id
    assert_eq!(v["run_id"], "test-run");

    // iterations
    assert_eq!(v["iterations"], 2);

    // token totals
    assert_eq!(v["tokens"]["total_input"], 3000);
    assert_eq!(v["tokens"]["total_output"], 1300);
    assert_eq!(v["tokens"]["total_cache_read"], 600);
    assert_eq!(v["tokens"]["total_cache_write"], 900);
    assert_eq!(v["tokens"]["total"], 5800);
    assert_eq!(v["tokens"]["avg_per_iteration"], 2900);

    // cost estimates
    // input = 3000/1M * 3.0 = 0.009
    let input_cost = v["cost_estimate_usd"]["input"].as_f64().unwrap();
    assert!(
        (input_cost - 0.009).abs() < 1e-10,
        "input cost: expected ~0.009, got {input_cost}"
    );

    // output = 1300/1M * 15.0 = 0.0195
    let output_cost = v["cost_estimate_usd"]["output"].as_f64().unwrap();
    assert!(
        (output_cost - 0.0195).abs() < 1e-10,
        "output cost: expected ~0.0195, got {output_cost}"
    );

    // cache_read = 600/1M * 0.3 = 0.00018
    let cr_cost = v["cost_estimate_usd"]["cache_read"].as_f64().unwrap();
    assert!(
        (cr_cost - 0.00018).abs() < 1e-10,
        "cache_read cost: expected ~0.00018, got {cr_cost}"
    );

    // cache_write = 900/1M * 3.75 = 0.003375
    let cw_cost = v["cost_estimate_usd"]["cache_write"].as_f64().unwrap();
    assert!(
        (cw_cost - 0.003375).abs() < 1e-10,
        "cache_write cost: expected ~0.003375, got {cw_cost}"
    );

    // total cost
    let total_cost = v["cost_estimate_usd"]["total"].as_f64().unwrap();
    let expected_total = 0.009 + 0.0195 + 0.00018 + 0.003375; // 0.032055
    assert!(
        (total_cost - expected_total).abs() < 1e-10,
        "total cost: expected {expected_total}, got {total_cost}"
    );
}

#[tokio::test]
async fn compress_stats_json_format_is_valid_json() {
    let output = run_compress_stats_with_fixture("json").await;
    let result = serde_json::from_str::<serde_json::Value>(&output);
    assert!(result.is_ok(), "JSON output is not valid: {output}");
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

    // For empty iterations we just ensure it doesn't panic and returns Ok.
    let rt = tokio::runtime::Runtime::new().unwrap();
    let repo_str = repo_root.to_str().unwrap();
    let result = rt.block_on(async {
        sruja_cli::commands::compress_stats::compress_stats(repo_str, Some("empty-run"), "text")
            .await
    });

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

    let buf = capture_stdout(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let repo_str = repo_root.to_str().unwrap();
        rt.block_on(async {
            sruja_cli::commands::compress_stats::compress_stats(
                repo_str,
                Some("sparse-run"),
                "text",
            )
            .await
            .expect("compress_stats returned error");
        });
    });

    // Input should be 500, everything else defaults to 0.
    assert!(
        buf.contains("Input:         500"),
        "expected 500 input tokens, got:\n{buf}"
    );
    assert!(
        buf.contains("Output:        0"),
        "expected 0 output tokens, got:\n{buf}"
    );
    assert!(
        buf.contains("Cache Read:    0"),
        "expected 0 cache read, got:\n{buf}"
    );
    assert!(
        buf.contains("Cache Write:   0"),
        "expected 0 cache write, got:\n{buf}"
    );

    tmp.close().ok();
}

#[tokio::test]
async fn compress_stats_returns_error_for_missing_run() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let repo_root = tmp.path();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let repo_str = repo_root.to_str().unwrap();
    let result = rt.block_on(async {
        sruja_cli::commands::compress_stats::compress_stats(
            repo_str,
            Some("nonexistent-run"),
            "text",
        )
        .await
    });

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

    let result = capture_stdout(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let repo_str = repo_root.to_str().unwrap();
        rt.block_on(async {
            sruja_cli::commands::compress_stats::compress_stats(
                repo_str,
                Some("nested-run"),
                "json",
            )
            .await
            .expect("compress_stats returned error");
        });
    });

    let v: serde_json::Value =
        serde_json::from_str(&result).expect("json output is not valid JSON");

    // top-level: input=1000, output=500
    // comprehension: input=100, output=50
    // plan: input=200, output=80
    // critique: input=150, output=60
    // total input = 1000 + 100 + 200 + 150 = 1450
    // total output = 500 + 50 + 80 + 60 = 690
    assert_eq!(
        v["tokens"]["total_input"], 1450,
        "nested input tokens should aggregate"
    );
    assert_eq!(
        v["tokens"]["total_output"], 690,
        "nested output tokens should aggregate"
    );

    tmp.close().ok();
}
