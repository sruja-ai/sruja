use crate::commands::CliError;

#[allow(dead_code)]
pub async fn run(run_id: Option<&str>, format: &str) -> Result<(), CliError> {
    let repo = std::env::current_dir()
        .map_err(|e| CliError::validation(e.to_string()))?
        .to_string_lossy()
        .to_string();
    compress_stats(&repo, run_id, format).await
}

#[allow(dead_code)]
pub async fn compress_stats(
    repo: &str,
    run_id: Option<&str>,
    format: &str,
) -> Result<(), CliError> {
    let repo_root = std::path::Path::new(repo);
    let runs_dir = repo_root.join(".sruja").join("runs");

    if !runs_dir.exists() {
        return Err(CliError::validation(format!(
            "Runs directory does not exist: {}",
            runs_dir.display()
        )));
    }

    let (display_run_id, loop_json_path) = match run_id {
        Some(id) => {
            let run_dir_path = runs_dir.join(id);
            if !run_dir_path.exists() {
                return Err(CliError::validation(format!(
                    "Run directory does not exist: {}",
                    run_dir_path.display()
                )));
            }
            let path = run_dir_path.join("loop.json");
            if !path.exists() {
                return Err(CliError::validation(format!(
                    "loop.json does not exist: {}",
                    path.display()
                )));
            }
            (id.to_string(), path)
        }
        None => {
            let mut entries: Vec<_> = std::fs::read_dir(&runs_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .collect();
            entries.sort_by(|a, b| {
                b.metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                    .cmp(
                        &a.metadata()
                            .and_then(|m| m.modified())
                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                    )
            });
            let latest = entries
                .first()
                .ok_or_else(|| CliError::validation("No runs found in .sruja/runs/".to_string()))?;
            let run_dir_path = latest.path();
            let path = run_dir_path.join("loop.json");
            if !path.exists() {
                return Err(CliError::validation(format!(
                    "loop.json does not exist: {}",
                    path.display()
                )));
            }
            let dir_name = run_dir_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            (dir_name, path)
        }
    };

    let content = std::fs::read_to_string(&loop_json_path)?;
    let loop_json: serde_json::Value = serde_json::from_str(&content)?;

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
            total_input_tokens += usage
                .get("prompt_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            total_output_tokens += usage
                .get("completion_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            total_cache_read_tokens += usage
                .get("cache_read_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            total_cache_write_tokens += usage
                .get("cache_write_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
        }

        // Aggregate nested phase usage (e.g. comprehension, plan, critique)
        if let Some(phases) = iter.get("phases").and_then(|v| v.as_object()) {
            for (_phase_name, phase_val) in phases {
                if let Some(phase_usage) = phase_val.get("usage") {
                    total_input_tokens += phase_usage
                        .get("prompt_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    total_output_tokens += phase_usage
                        .get("completion_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    total_cache_read_tokens += phase_usage
                        .get("cache_read_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    total_cache_write_tokens += phase_usage
                        .get("cache_write_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                }
            }
        }
    }

    let iteration_count = iterations.len();

    let total_tokens = total_input_tokens
        + total_output_tokens
        + total_cache_read_tokens
        + total_cache_write_tokens;

    let count = iteration_count as u64;
    let avg_input = if count == 0 {
        0
    } else {
        total_input_tokens / count
    };
    let avg_output = if count == 0 {
        0
    } else {
        total_output_tokens / count
    };
    let avg_cache_read = if count == 0 {
        0
    } else {
        total_cache_read_tokens / count
    };
    let avg_cache_write = if count == 0 {
        0
    } else {
        total_cache_write_tokens / count
    };
    let avg_tokens_per_iter = if count == 0 { 0 } else { total_tokens / count };

    // Cost rates: $3.00/M input, $15.00/M output, $0.30/M cache_read, $3.75/M cache_write
    let input_cost = total_input_tokens as f64 / 1_000_000.0 * 3.0;
    let output_cost = total_output_tokens as f64 / 1_000_000.0 * 15.0;
    let cache_read_cost = total_cache_read_tokens as f64 / 1_000_000.0 * 0.30;
    let cache_write_cost = total_cache_write_tokens as f64 / 1_000_000.0 * 3.75;
    let total_cost = input_cost + output_cost + cache_read_cost + cache_write_cost;

    if format == "json" {
        let output = serde_json::json!({
            "schema_version": "compress_stats/v1",
            "run_id": display_run_id,
            "iterations": iteration_count,
            "tokens": {
                "total_input": total_input_tokens,
                "total_output": total_output_tokens,
                "total_cache_read": total_cache_read_tokens,
                "total_cache_write": total_cache_write_tokens,
                "total": total_tokens,
                "avg_per_iteration": {
                    "input": avg_input,
                    "output": avg_output,
                    "cache_read": avg_cache_read,
                    "cache_write": avg_cache_write,
                    "total": avg_tokens_per_iter
                }
            },
            "cost_estimate_usd": {
                "input": input_cost,
                "output": output_cost,
                "cache_read": cache_read_cost,
                "cache_write": cache_write_cost,
                "total": total_cost
            }
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Token Usage Statistics");
        println!("Run ID: {}", display_run_id);
        println!();
        println!("Iterations: {}", iteration_count);
        println!();
        println!("Tokens:");
        println!("  Input:         {}", total_input_tokens);
        println!("  Output:        {}", total_output_tokens);
        println!("  Cache Read:    {}", total_cache_read_tokens);
        println!("  Cache Write:   {}", total_cache_write_tokens);
        println!("  Total:         {}", total_tokens);
        println!("  Avg/Iteration: {}", avg_tokens_per_iter);
        println!();
        println!("Cost Estimate (USD):");
        println!("  Input:         ${:.6}", input_cost);
        println!("  Output:        ${:.6}", output_cost);
        println!("  Cache Read:    ${:.6}", cache_read_cost);
        println!("  Cache Write:   ${:.6}", cache_write_cost);
        println!("  Total:         ${:.6}", total_cost);
    }

    Ok(())
}
