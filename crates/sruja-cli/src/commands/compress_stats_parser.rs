//! Parser for `.sruja/runs/*/loop.json` files.
//!
//! Extracts iteration token usage and computes aggregate statistics
//! including total tokens, iteration count, average tokens per iteration,
//! and cost breakdown.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Token usage for a single iteration or phase.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
}

/// Cost breakdown in USD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_write: f64,
    pub total: f64,
}

/// Aggregated statistics from a loop.json file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopStats {
    pub run_id: String,
    pub iterations_count: usize,
    pub total_tokens: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub total_cache_write_tokens: u64,
    pub avg_tokens_per_iteration: u64,
    pub avg_input_per_iteration: u64,
    pub avg_output_per_iteration: u64,
    pub avg_cache_read_per_iteration: u64,
    pub avg_cache_write_per_iteration: u64,
    pub cost_breakdown: CostBreakdown,
}

/// Parse a loop.json file and extract token usage statistics.
///
/// Reads the file at `path`, aggregates token usage across all iterations
/// (including nested phase usage), and computes totals, averages, and cost estimates.
///
/// # Arguments
/// * `path` - Path to a loop.json file (typically `.sruja/runs/<run_id>/loop.json`)
///
/// # Returns
/// A `LoopStats` struct with aggregated statistics, or an error if parsing fails.
pub fn parse_loop_json(path: &Path) -> Result<LoopStats, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let loop_json: serde_json::Value = serde_json::from_str(&content)?;

    // Extract run_id from parent directory name
    let run_id = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let iterations = loop_json
        .get("iterations")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut total_input_tokens: u64 = 0;
    let mut total_output_tokens: u64 = 0;
    let mut total_cache_read_tokens: u64 = 0;
    let mut total_cache_write_tokens: u64 = 0;

    for iter in &iterations {
        // Aggregate top-level usage
        if let Some(usage) = iter.get("usage") {
            total_input_tokens += usage.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            total_output_tokens += usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            total_cache_read_tokens += usage.get("cache_read_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            total_cache_write_tokens += usage.get("cache_write_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
        }

        // Aggregate nested phase usage (e.g. comprehension, plan, critique)
        if let Some(phases) = iter.get("phases").and_then(|v| v.as_object()) {
            for (_phase_name, phase_val) in phases {
                if let Some(phase_usage) = phase_val.get("usage") {
                    total_input_tokens += phase_usage.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                    total_output_tokens += phase_usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                    total_cache_read_tokens += phase_usage.get("cache_read_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                    total_cache_write_tokens += phase_usage.get("cache_write_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                }
            }
        }
    }

    let iterations_count = iterations.len();
    let total_tokens = total_input_tokens + total_output_tokens + total_cache_read_tokens + total_cache_write_tokens;

    let count = iterations_count as u64;
    let avg_tokens_per_iteration = if count == 0 { 0 } else { total_tokens / count };
    let avg_input_per_iteration = if count == 0 { 0 } else { total_input_tokens / count };
    let avg_output_per_iteration = if count == 0 { 0 } else { total_output_tokens / count };
    let avg_cache_read_per_iteration = if count == 0 { 0 } else { total_cache_read_tokens / count };
    let avg_cache_write_per_iteration = if count == 0 { 0 } else { total_cache_write_tokens / count };

    // Cost rates: $3.00/M input, $15.00/M output, $0.30/M cache_read, $3.75/M cache_write
    let input_cost = total_input_tokens as f64 / 1_000_000.0 * 3.0;
    let output_cost = total_output_tokens as f64 / 1_000_000.0 * 15.0;
    let cache_read_cost = total_cache_read_tokens as f64 / 1_000_000.0 * 0.30;
    let cache_write_cost = total_cache_write_tokens as f64 / 1_000_000.0 * 3.75;
    let total_cost = input_cost + output_cost + cache_read_cost + cache_write_cost;

    let cost_breakdown = CostBreakdown {
        input: input_cost,
        output: output_cost,
        cache_read: cache_read_cost,
        cache_write: cache_write_cost,
        total: total_cost,
    };

    Ok(LoopStats {
        run_id,
        iterations_count,
        total_tokens,
        total_input_tokens,
        total_output_tokens,
        total_cache_read_tokens,
        total_cache_write_tokens,
        avg_tokens_per_iteration,
        avg_input_per_iteration,
        avg_output_per_iteration,
        avg_cache_read_per_iteration,
        avg_cache_write_per_iteration,
        cost_breakdown,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_loop_json_basic() {
        let dir = tempfile::tempdir().unwrap();
        let run_dir = dir.path().join("run_123");
        std::fs::create_dir_all(&run_dir).unwrap();
        let loop_path = run_dir.join("loop.json");

        let data = serde_json::json!({
            "iterations": [
                {
                    "usage": {
                        "prompt_tokens": 1000,
                        "completion_tokens": 500,
                        "cache_read_tokens": 200,
                        "cache_write_tokens": 100
                    }
                },
                {
                    "usage": {
                        "prompt_tokens": 2000,
                        "completion_tokens": 1000,
                        "cache_read_tokens": 400,
                        "cache_write_tokens": 200
                    }
                }
            ]
        });

        let mut file = std::fs::File::create(&loop_path).unwrap();
        write!(file, "{}", data).unwrap();

        let stats = parse_loop_json(&loop_path).unwrap();

        assert_eq!(stats.run_id, "run_123");
        assert_eq!(stats.iterations_count, 2);
        assert_eq!(stats.total_input_tokens, 3000);
        assert_eq!(stats.total_output_tokens, 1500);
        assert_eq!(stats.total_cache_read_tokens, 600);
        assert_eq!(stats.total_cache_write_tokens, 300);
        assert_eq!(stats.total_tokens, 5400);
        assert_eq!(stats.avg_tokens_per_iteration, 2700);
        assert_eq!(stats.avg_input_per_iteration, 1500);
        assert_eq!(stats.avg_output_per_iteration, 750);
    }

    #[test]
    fn test_parse_loop_json_with_phases() {
        let dir = tempfile::tempdir().unwrap();
        let run_dir = dir.path().join("run_456");
        std::fs::create_dir_all(&run_dir).unwrap();
        let loop_path = run_dir.join("loop.json");

        let data = serde_json::json!({
            "iterations": [
                {
                    "usage": {
                        "prompt_tokens": 1000,
                        "completion_tokens": 500,
                        "cache_read_tokens": 0,
                        "cache_write_tokens": 0
                    },
                    "phases": {
                        "comprehension": {
                            "usage": {
                                "prompt_tokens": 500,
                                "completion_tokens": 250,
                                "cache_read_tokens": 100,
                                "cache_write_tokens": 50
                            }
                        },
                        "plan": {
                            "usage": {
                                "prompt_tokens": 300,
                                "completion_tokens": 150,
                                "cache_read_tokens": 0,
                                "cache_write_tokens": 0
                            }
                        }
                    }
                }
            ]
        });

        let mut file = std::fs::File::create(&loop_path).unwrap();
        write!(file, "{}", data).unwrap();

        let stats = parse_loop_json(&loop_path).unwrap();

        assert_eq!(stats.iterations_count, 1);
        assert_eq!(stats.total_input_tokens, 1800); // 1000 + 500 + 300
        assert_eq!(stats.total_output_tokens, 900);  // 500 + 250 + 150
        assert_eq!(stats.total_cache_read_tokens, 100);
        assert_eq!(stats.total_cache_write_tokens, 50);
        assert_eq!(stats.total_tokens, 2850);
    }

    #[test]
    fn test_parse_loop_json_empty_iterations() {
        let dir = tempfile::tempdir().unwrap();
        let run_dir = dir.path().join("run_empty");
        std::fs::create_dir_all(&run_dir).unwrap();
        let loop_path = run_dir.join("loop.json");

        let data = serde_json::json!({
            "iterations": []
        });

        let mut file = std::fs::File::create(&loop_path).unwrap();
        write!(file, "{}", data).unwrap();

        let stats = parse_loop_json(&loop_path).unwrap();

        assert_eq!(stats.iterations_count, 0);
        assert_eq!(stats.total_tokens, 0);
        assert_eq!(stats.avg_tokens_per_iteration, 0);
        assert_eq!(stats.cost_breakdown.total, 0.0);
    }

    #[test]
    fn test_parse_loop_json_missing_iterations() {
        let dir = tempfile::tempdir().unwrap();
        let run_dir = dir.path().join("run_missing");
        std::fs::create_dir_all(&run_dir).unwrap();
        let loop_path = run_dir.join("loop.json");

        let data = serde_json::json!({});

        let mut file = std::fs::File::create(&loop_path).unwrap();
        write!(file, "{}", data).unwrap();

        let stats = parse_loop_json(&loop_path).unwrap();

        assert_eq!(stats.iterations_count, 0);
        assert_eq!(stats.total_tokens, 0);
    }

    #[test]
    fn test_cost_breakdown() {
        let dir = tempfile::tempdir().unwrap();
        let run_dir = dir.path().join("run_cost");
        std::fs::create_dir_all(&run_dir).unwrap();
        let loop_path = run_dir.join("loop.json");

        let data = serde_json::json!({
            "iterations": [
                {
                    "usage": {
                        "prompt_tokens": 1_000_000,
                        "completion_tokens": 1_000_000,
                        "cache_read_tokens": 1_000_000,
                        "cache_write_tokens": 1_000_000
                    }
                }
            ]
        });

        let mut file = std::fs::File::create(&loop_path).unwrap();
        write!(file, "{}", data).unwrap();

        let stats = parse_loop_json(&loop_path).unwrap();

        // $3.00/M input
        assert!((stats.cost_breakdown.input - 3.0).abs() < 0.001);
        // $15.00/M output
        assert!((stats.cost_breakdown.output - 15.0).abs() < 0.001);
        // $0.30/M cache_read
        assert!((stats.cost_breakdown.cache_read - 0.30).abs() < 0.001);
        // $3.75/M cache_write
        assert!((stats.cost_breakdown.cache_write - 3.75).abs() < 0.001);
        // Total: 3 + 15 + 0.30 + 3.75 = 22.05
        assert!((stats.cost_breakdown.total - 22.05).abs() < 0.001);
    }

    #[test]
    fn test_parse_nonexistent_file() {
        let result = parse_loop_json(Path::new("/nonexistent/path/loop.json"));
        assert!(result.is_err());
    }
}
