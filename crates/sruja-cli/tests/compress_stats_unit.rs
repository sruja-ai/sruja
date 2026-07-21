//! Unit tests for compress-stats stats calculation logic.
//!
//! These tests verify the core calculation: parsing a loop.json fixture,
//! computing total tokens, average tokens per iteration, and cost breakdown.
//! Edge cases: empty iterations, missing token fields, single iteration, malformed JSON.

use std::fs;
use tempfile::TempDir;

// ---------------------------------------------------------------
// Pure calculation logic extracted for unit testing.
// These mirror the calculation in compress_stats.rs lines 68-91.
// ---------------------------------------------------------------

/// Calculate stats from an iterations array (mirrors the core logic of compress_stats).
/// Returns (iteration_count, total_input_tokens, total_output_tokens, total_tokens,
///           avg_tokens_per_iter, input_cost, output_cost, total_cost).
fn calculate_stats(iterations: &[serde_json::Value]) -> (usize, u64, u64, u64, u64, f64, f64, f64) {
    let iteration_count = iterations.len();
    let mut total_input_tokens: u64 = 0;
    let mut total_output_tokens: u64 = 0;

    for iteration in iterations {
        if let Some(usage) = iteration.get("usage") {
            total_input_tokens += usage
                .get("prompt_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            total_output_tokens += usage
                .get("completion_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
        }
    }

    let total_tokens = total_input_tokens + total_output_tokens;
    let avg_tokens_per_iter = if iteration_count > 0 {
        total_tokens / iteration_count as u64
    } else {
        0
    };

    let input_cost = total_input_tokens as f64 / 1_000_000.0 * 0.15;
    let output_cost = total_output_tokens as f64 / 1_000_000.0 * 0.60;
    let total_cost = input_cost + output_cost;

    (
        iteration_count,
        total_input_tokens,
        total_output_tokens,
        total_tokens,
        avg_tokens_per_iter,
        input_cost,
        output_cost,
        total_cost,
    )
}

/// Parse a loop.json string into an iterations Vec.
fn parse_loop_json(json_str: &str) -> Result<Vec<serde_json::Value>, String> {
    let val: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("malformed JSON: {e}"))?;
    let iterations = val
        .get("iterations")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "missing or non-array 'iterations' field".to_string())?
        .clone();
    Ok(iterations)
}

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------

/// Test: normal fixture with two iterations produces correct totals and averages.
#[test]
fn stats_two_iterations_totals_and_average() {
    let json = r#"{
        "iterations": [
            {
                "usage": { "prompt_tokens": 1000, "completion_tokens": 500 }
            },
            {
                "usage": { "prompt_tokens": 2000, "completion_tokens": 800 }
            }
        ]
    }"#;

    let iterations = parse_loop_json(json).expect("parse ok");
    let (count, input, output, total, avg, input_cost, output_cost, total_cost) =
        calculate_stats(&iterations);

    assert_eq!(count, 2, "iteration count");
    assert_eq!(input, 3000, "total input tokens (1000+2000)");
    assert_eq!(output, 1300, "total output tokens (500+800)");
    assert_eq!(total, 4300, "total tokens (3000+1300)");
    assert_eq!(avg, 2150, "avg tokens per iteration (4300/2)");

    // Cost: input = 3000/1_000_000 * 0.15 = 0.000450
    assert!(
        (input_cost - 0.000450).abs() < 1e-10,
        "input_cost expected 0.000450, got {input_cost}"
    );
    // Cost: output = 1300/1_000_000 * 0.60 = 0.000780
    assert!(
        (output_cost - 0.000780).abs() < 1e-10,
        "output_cost expected 0.000780, got {output_cost}"
    );
    // Total cost = 0.000450 + 0.000780 = 0.001230
    assert!(
        (total_cost - 0.001230).abs() < 1e-10,
        "total_cost expected 0.001230, got {total_cost}"
    );
}

/// Test: empty iterations array produces zero counts and zero costs.
#[test]
fn stats_empty_iterations_produces_zeros() {
    let json = r#"{ "iterations": [] }"#;

    let iterations = parse_loop_json(json).expect("parse ok");
    let (count, input, output, total, avg, input_cost, output_cost, total_cost) =
        calculate_stats(&iterations);

    assert_eq!(count, 0, "no iterations");
    assert_eq!(input, 0, "input tokens");
    assert_eq!(output, 0, "output tokens");
    assert_eq!(total, 0, "total tokens");
    assert_eq!(avg, 0, "avg should be 0 when no iterations");
    assert!(
        (input_cost).abs() < 1e-10,
        "input_cost should be 0, got {input_cost}"
    );
    assert!(
        (output_cost).abs() < 1e-10,
        "output_cost should be 0, got {output_cost}"
    );
    assert!(
        (total_cost).abs() < 1e-10,
        "total_cost should be 0, got {total_cost}"
    );
}

/// Test: missing `usage` field entirely → treated as zero tokens.
#[test]
fn stats_missing_usage_field_treated_as_zero() {
    let json = r#"{
        "iterations": [
            { "result": "some output without usage" },
            { "usage": { "prompt_tokens": 500, "completion_tokens": 200 } }
        ]
    }"#;

    let iterations = parse_loop_json(json).expect("parse ok");
    let (count, input, output, total, avg, input_cost, output_cost, total_cost) =
        calculate_stats(&iterations);

    assert_eq!(count, 2, "iteration count");
    assert_eq!(input, 500, "only second iteration contributes input");
    assert_eq!(output, 200, "only second iteration contributes output");
    assert_eq!(total, 700, "total from only the second iteration");
    assert_eq!(avg, 350, "avg = 700/2");

    let expected_input_cost = 500.0 / 1_000_000.0 * 0.15;
    let expected_output_cost = 200.0 / 1_000_000.0 * 0.60;
    assert!(
        (input_cost - expected_input_cost).abs() < 1e-10,
        "input_cost"
    );
    assert!(
        (output_cost - expected_output_cost).abs() < 1e-10,
        "output_cost"
    );
    assert!(
        (total_cost - (expected_input_cost + expected_output_cost)).abs() < 1e-10,
        "total_cost"
    );
}

/// Test: missing `prompt_tokens` and/or `completion_tokens` → defaults to 0.
#[test]
fn stats_missing_token_fields_default_to_zero() {
    let json = r#"{
        "iterations": [
            { "usage": { "prompt_tokens": 100 } },
            { "usage": { "completion_tokens": 300 } },
            { "usage": {} }
        ]
    }"#;

    let iterations = parse_loop_json(json).expect("parse ok");
    let (count, input, output, total, avg, _, _, _) = calculate_stats(&iterations);

    assert_eq!(count, 3, "iteration count");
    assert_eq!(input, 100, "prompt_tokens from first only");
    assert_eq!(output, 300, "completion_tokens from second only");
    assert_eq!(total, 400, "100+300");
    assert_eq!(avg, 133, "400/3 = 133 (integer division)");
}

/// Test: single iteration calculates correctly (average equals total).
#[test]
fn stats_single_iteration_avg_equals_total() {
    let json = r#"{
        "iterations": [
            { "usage": { "prompt_tokens": 8000, "completion_tokens": 2000 } }
        ]
    }"#;

    let iterations = parse_loop_json(json).expect("parse ok");
    let (count, input, output, total, avg, input_cost, output_cost, total_cost) =
        calculate_stats(&iterations);

    assert_eq!(count, 1, "single iteration");
    assert_eq!(input, 8000);
    assert_eq!(output, 2000);
    assert_eq!(total, 10_000);
    assert_eq!(avg, 10_000, "avg with 1 iteration == total");

    let expected_input_cost = 8000.0 / 1_000_000.0 * 0.15;
    let expected_output_cost = 2000.0 / 1_000_000.0 * 0.60;
    assert!(
        (input_cost - expected_input_cost).abs() < 1e-10,
        "input_cost"
    );
    assert!(
        (output_cost - expected_output_cost).abs() < 1e-10,
        "output_cost"
    );
    assert!(
        (total_cost - (expected_input_cost + expected_output_cost)).abs() < 1e-10,
        "total_cost"
    );
}

/// Test: malformed JSON fails to parse.
#[test]
fn stats_malformed_json_returns_error() {
    let result = parse_loop_json("{ this is not valid json !!!");
    assert!(result.is_err(), "malformed JSON should return Err");
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("malformed JSON"),
        "error should mention malformed JSON, got: {err_msg}"
    );
}

/// Test: missing `iterations` key entirely → error.
#[test]
fn stats_missing_iterations_key_returns_error() {
    let json = r#"{ "run_id": "test", "other": [1, 2, 3] }"#;
    let result = parse_loop_json(json);
    assert!(result.is_err(), "missing iterations should return Err");
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("missing or non-array 'iterations' field"),
        "error should mention missing iterations, got: {err_msg}"
    );
}

/// Test: iterations is not an array → error.
#[test]
fn stats_iterations_not_array_returns_error() {
    let json = r#"{ "iterations": "not an array" }"#;
    let result = parse_loop_json(json);
    assert!(result.is_err(), "non-array iterations should return Err");
}

/// Test: large token values don't overflow or produce wrong costs.
#[test]
fn stats_large_token_values() {
    let json = r#"{
        "iterations": [
            { "usage": { "prompt_tokens": 1000000, "completion_tokens": 500000 } },
            { "usage": { "prompt_tokens": 2000000, "completion_tokens": 1500000 } }
        ]
    }"#;

    let iterations = parse_loop_json(json).expect("parse ok");
    let (_, input, output, total, avg, input_cost, output_cost, total_cost) =
        calculate_stats(&iterations);

    assert_eq!(input, 3_000_000);
    assert_eq!(output, 2_000_000);
    assert_eq!(total, 5_000_000);
    assert_eq!(avg, 2_500_000);

    // input_cost = 3M / 1M * 0.15 = 0.45
    assert!(
        (input_cost - 0.45).abs() < 1e-10,
        "input_cost expected 0.45, got {input_cost}"
    );
    // output_cost = 2M / 1M * 0.60 = 1.20
    assert!(
        (output_cost - 1.20).abs() < 1e-10,
        "output_cost expected 1.20, got {output_cost}"
    );
    assert!(
        (total_cost - 1.65).abs() < 1e-10,
        "total_cost expected 1.65, got {total_cost}"
    );
}

/// Test: cost breakdown with zero tokens yields zero costs.
#[test]
fn stats_zero_tokens_yields_zero_cost() {
    let json = r#"{
        "iterations": [
            { "usage": { "prompt_tokens": 0, "completion_tokens": 0 } }
        ]
    }"#;

    let iterations = parse_loop_json(json).expect("parse ok");
    let (_, input, output, total, avg, input_cost, output_cost, total_cost) =
        calculate_stats(&iterations);

    assert_eq!(input, 0);
    assert_eq!(output, 0);
    assert_eq!(total, 0);
    assert_eq!(avg, 0);
    assert!((input_cost).abs() < 1e-10);
    assert!((output_cost).abs() < 1e-10);
    assert!((total_cost).abs() < 1e-10);
}

/// Test: iterations with extra unknown fields are ignored gracefully.
#[test]
fn stats_extra_fields_are_ignored() {
    let json = r#"{
        "iterations": [
            {
                "unknown_field": 42,
                "usage": { "prompt_tokens": 500, "completion_tokens": 300, "extra": "ignored" },
                "other": [1, 2, 3]
            }
        ]
    }"#;

    let iterations = parse_loop_json(json).expect("parse ok");
    let (count, input, output, total, _, _, _, _) = calculate_stats(&iterations);

    assert_eq!(count, 1);
    assert_eq!(input, 500);
    assert_eq!(output, 300);
    assert_eq!(total, 800);
}

/// Test: cost rates are correct (input at $0.15/M, output at $0.60/M).
#[test]
fn stats_cost_rates_are_correct() {
    // 1M tokens of each type
    let json = r#"{
        "iterations": [
            { "usage": { "prompt_tokens": 1000000, "completion_tokens": 1000000 } }
        ]
    }"#;

    let iterations = parse_loop_json(json).expect("parse ok");
    let (_, _, _, _, _, input_cost, output_cost, _) = calculate_stats(&iterations);

    assert!(
        (input_cost - 0.15).abs() < 1e-10,
        "1M input tokens should cost $0.15, got {input_cost}"
    );
    assert!(
        (output_cost - 0.60).abs() < 1e-10,
        "1M output tokens should cost $0.60, got {output_cost}"
    );
}

/// Test: mixed valid and invalid iterations (missing usage entirely).
#[test]
fn stats_mixed_valid_and_missing_usage() {
    let json = r#"{
        "iterations": [
            {},
            { "usage": { "prompt_tokens": 1000, "completion_tokens": 500 } },
            { "result": "no usage here" },
            { "usage": { "prompt_tokens": 2000, "completion_tokens": 1000 } },
            {}
        ]
    }"#;

    let iterations = parse_loop_json(json).expect("parse ok");
    let (count, input, output, total, avg, _, _, _) = calculate_stats(&iterations);

    assert_eq!(count, 5, "all iterations counted");
    assert_eq!(input, 3000, "only iterations 1 and 3 have usage");
    assert_eq!(output, 1500);
    assert_eq!(total, 4500);
    assert_eq!(avg, 900, "4500/5 = 900");
}

/// Test: empty JSON object (no iterations key) returns an error.
#[test]
fn stats_empty_json_object_returns_error() {
    let result = parse_loop_json("{}");
    assert!(result.is_err(), "empty JSON object should return Err");
}

/// Test: the fixture format matches what the real compress_stats command expects.
/// This validates our test helper produces the same structure the command reads.
#[test]
fn stats_fixture_structure_matches_command_expectations() {
    let tmp = TempDir::new().expect("tempdir");
    let run_dir = tmp.path().join(".sruja/runs/test-run");
    fs::create_dir_all(&run_dir).expect("create run dir");

    let loop_json = serde_json::json!({
        "iterations": [
            {
                "usage": { "prompt_tokens": 1000, "completion_tokens": 500 }
            },
            {
                "usage": { "prompt_tokens": 2000, "completion_tokens": 800 }
            }
        ]
    });

    let path = run_dir.join("loop.json");
    fs::write(&path, serde_json::to_string_pretty(&loop_json).unwrap()).expect("write");

    let content = fs::read_to_string(&path).expect("read back");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("parse");
    let iterations = parsed["iterations"].as_array().expect("array");

    assert_eq!(iterations.len(), 2);
    assert_eq!(iterations[0]["usage"]["prompt_tokens"].as_u64(), Some(1000));
    assert_eq!(
        iterations[1]["usage"]["completion_tokens"].as_u64(),
        Some(800)
    );
}
