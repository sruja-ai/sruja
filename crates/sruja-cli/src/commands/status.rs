//! Status command: show repo health, baseline, and truth status.

use super::scan::status_result;
use super::CliError;
use crate::utils::colors;
use std::path::Path;

fn escape_github_actions_message(input: &str) -> String {
    input
        .replace('%', "%25")
        .replace('\n', "%0A")
        .replace('\r', "%0D")
}

/// Run status and print text or JSON.
pub async fn status(repo_root: &str, format: &str, evolution: bool) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);

    if !repo_path.join(".sruja").exists() {
        return Err(CliError::NotInitialized {
            path: repo_root.to_string(),
        });
    }

    if evolution {
        let log_path = repo_path.join(".sruja").join("evolution.log");
        println!("====================================================");
        println!("📊 SRUJA EVOLUTIONARY HEALTH & METRICS");
        println!("====================================================");

        if !log_path.exists() {
            println!("No evolution history found under .sruja/evolution.log.");
            println!(
                "Run 'sruja intent evaluate' to execute fitness functions and populate history."
            );
            return Ok(());
        }

        let contents = std::fs::read_to_string(&log_path)?;
        let lines: Vec<&str> = contents.lines().filter(|l| !l.trim().is_empty()).collect();
        let total = lines.len();

        let mut passed = 0;
        let mut failed = 0;
        let mut error = 0;
        let mut latest_mutation = "None".to_string();

        for line in &lines {
            if line.contains("Result: PASS") {
                passed += 1;
            } else if line.contains("Result: FAIL") {
                failed += 1;
            } else if line.contains("Result: ERROR") {
                error += 1;
            }
        }

        if let Some(last) = lines.last() {
            latest_mutation = last.to_string();
        }

        let pass_ratio = if total > 0 {
            (passed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        use crate::utils::table_formatter::TableFormatter;
        let formatter = TableFormatter::auto();
        let mut blocks = Vec::new();

        let mut metrics_info = String::new();
        metrics_info.push_str(&format!(
            "Total Evaluations: {}\n",
            colors::info(&total.to_string())
        ));
        metrics_info.push_str(&format!(
            "Passed Checks:     {}\n",
            colors::success(&passed.to_string())
        ));
        metrics_info.push_str(&format!(
            "Failed Checks:     {}\n",
            colors::error(&failed.to_string())
        ));
        if error > 0 {
            metrics_info.push_str(&format!(
                "Exec Errors:       {}\n",
                colors::warning(&error.to_string())
            ));
        }
        let bar_score = pass_ratio as u8;
        let pass_bar = colors::health_bar(bar_score, 20);
        metrics_info.push_str(&format!(
            "Pass Ratio:        {} ({:.1}%)\n",
            pass_bar, pass_ratio
        ));
        blocks.push(("Evolutionary Metrics".to_string(), metrics_info));

        let mut history_info = String::new();
        history_info.push_str(&format!("Latest Mutation:\n{}\n", latest_mutation));
        blocks.push(("Activity Ledger".to_string(), history_info));

        println!(
            "{}",
            formatter.format_dashboard("EVOLUTIONARY HEALTH", blocks)
        );

        return Ok(());
    }

    let out = status_result(repo_root).await?;

    match format {
        "json" => {
            let mut v = serde_json::to_value(&out)?;
            if let serde_json::Value::Object(ref mut map) = v {
                map.insert(
                    "metric_type".to_string(),
                    serde_json::Value::String("truth_freshness".to_string()),
                );
                map.insert(
                    "metric_description".to_string(),
                    serde_json::Value::String(
                        "Repository status: truth freshness, structural health, AI readiness, and agent memory.".to_string(),
                    ),
                );
            }
            println!("{}", serde_json::to_string_pretty(&v)?);
        }
        "github" | "github-actions" => {
            let file = out.baseline.as_deref().unwrap_or(".sruja/context.json");
            let msg = format!(
                "Truth status: {}. Violations: {}.",
                out.truth_status, out.violations_count
            );
            println!(
                "::notice file={}::title=Sruja Status::{}",
                file,
                escape_github_actions_message(&msg)
            );
            if let Some(ref memory) = out.agent_memory {
                if memory.is_stale {
                    let memory_msg = format!(
                        "Agent memory adoption low: {} learnings{}. Record guardrails when Sruja catches a miss.",
                        memory.learnings_count,
                        memory
                            .age_days
                            .map(|d| format!(", last update {d} days ago"))
                            .unwrap_or_default()
                    );
                    println!(
                        "::warning file=.sruja/agent_memory.json::title=Agent Memory Adoption::{}",
                        escape_github_actions_message(&memory_msg)
                    );
                }
            }
        }
        _ => {
            use crate::utils::table_formatter::TableFormatter;
            let formatter = TableFormatter::auto();

            println!(
                "{}",
                colors::dim(
                    "Shows: truth freshness, structural health, AI readiness, density, and agent memory."
                )
            );

            let mut blocks = Vec::new();

            // 1. Truth snapshot (may include quick health/context bars for orientation)
            let mut health_info = String::new();
            if let Some(score) = out.health_score {
                let health_bar = colors::health_bar(score, 20);
                let trend = if !out.health_history.is_empty() {
                    format!(
                        " | Trend: {}",
                        TableFormatter::format_sparkline(&out.health_history)
                    )
                } else {
                    "".to_string()
                };
                health_info.push_str(&format!("Health: {}{}\n", health_bar, trend));
            }

            if let Some(score) = out.context_score {
                let context_bar = colors::health_bar(score, 20);
                health_info.push_str(&format!("Context: {}\n", context_bar));
            }

            let status_color = match out.truth_status.as_str() {
                "reviewed" => colors::success(&out.truth_status),
                "drifted" => colors::error(&out.truth_status),
                _ => colors::warning(&out.truth_status),
            };
            health_info.push_str(&format!(
                "Truth:  {} ({} violations)\n",
                status_color, out.violations_count
            ));

            if let Some(ref base) = out.baseline {
                health_info.push_str(&format!("Config: {}\n", colors::info(base)));
            } else {
                health_info.push_str(&format!("Config: {}\n", colors::warning("not found")));
            }

            blocks.push(("Truth & signals".to_string(), health_info));

            // 1b. Density Tier
            let density = crate::commands::density::compute_density(repo_path);
            let mut density_info = String::new();
            let tier_label = format!("Tier {} ({})", density.tier as u8, density.tier_name);
            let colored_tier = match density.tier {
                crate::commands::density::DensityTier::Sparse => colors::warning(&tier_label),
                crate::commands::density::DensityTier::Medium => colors::info(&tier_label),
                crate::commands::density::DensityTier::Dense => colors::success(&tier_label),
                crate::commands::density::DensityTier::Rich => colors::success(&tier_label),
            };
            density_info.push_str(&format!("Current: {}\n", colored_tier));
            if let Some(ref next) = density.next_step {
                density_info.push_str(&format!("Next:    {}\n", colors::dim(next)));
            }
            blocks.push(("Density".to_string(), density_info));

            // 2. Supervision / Velocity Block
            if let Some(ref velocity) = out.velocity {
                let mut supervision_info = String::new();
                let ratio = (velocity.supervision_ratio * 100.0) as u8;
                let bar = colors::health_bar(ratio, 20);
                supervision_info.push_str(&format!("Ratio:   {}\n", bar));
                supervision_info.push_str(&format!(
                    "Changed: {} components\n",
                    colors::info(&velocity.nodes_changed.to_string())
                ));
                supervision_info.push_str(&format!(
                    "Intent:  {}/{} proposed\n",
                    velocity.nodes_with_intent, velocity.nodes_changed
                ));

                if !velocity.unsupervised_nodes.is_empty() {
                    let uncovered = velocity.unsupervised_nodes.join(", ");
                    let truncated = if uncovered.len() > 40 {
                        format!("{}...", &uncovered[..37])
                    } else {
                        uncovered
                    };
                    supervision_info.push_str(&format!("Gap:     {}\n", colors::error(&truncated)));
                }

                blocks.push(("Supervision (Recent)".to_string(), supervision_info));
            }

            // 3. Findings Preview
            if !out.top_findings.is_empty() {
                let mut findings_info = String::new();
                for (i, f) in out.top_findings.iter().enumerate() {
                    let severity_icon = match f.severity.as_str() {
                        "error" => colors::error("✗"),
                        "warning" => colors::warning("⚠"),
                        _ => colors::info("ℹ"),
                    };
                    findings_info.push_str(&format!(
                        "{}. {} {} - {}\n",
                        i + 1,
                        severity_icon,
                        colors::style(&f.kind).bold(),
                        f.message
                    ));
                }
                blocks.push(("Top Issues".to_string(), findings_info));
            }

            // 3. Environment/Sync Info
            let mut env_info = String::new();
            if let Some(ref ts) = out.context_updated_at {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
                    let now = chrono::Utc::now();
                    let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                    let elapsed = if duration.num_seconds() > 0 {
                        colors::elapsed_display(std::time::Duration::from_secs(
                            duration.num_seconds() as u64,
                        ))
                    } else {
                        "just now".to_string()
                    };

                    let color = if duration.num_hours() > 24 {
                        colors::error(&elapsed)
                    } else if duration.num_hours() > 1 {
                        colors::warning(&elapsed)
                    } else {
                        colors::success(&elapsed)
                    };
                    env_info.push_str(&format!("Last Sync:  {} ago\n", color));
                }
            }

            let git_dir = repo_path.join(".git");
            let hook_exists = git_dir.join("hooks/pre-commit").exists();
            env_info.push_str(&format!(
                "Git Hook:   {}\n",
                if hook_exists {
                    colors::success("installed")
                } else {
                    colors::dim("missing")
                }
            ));
            env_info.push_str(&format!(
                "CLI Ver:    {}\n",
                colors::dim(env!("CARGO_PKG_VERSION"))
            ));

            blocks.push(("Environment".to_string(), env_info));

            if let Some(ref memory) = out.agent_memory {
                let mut memory_info = String::new();
                memory_info.push_str(&format!(
                    "Learnings:  {}\n",
                    colors::info(&memory.learnings_count.to_string())
                ));
                if let Some(ref ts) = memory.latest_learning_at {
                    memory_info.push_str(&format!("Latest:     {}\n", colors::dim(ts)));
                }
                if let Some(days) = memory.age_days {
                    let age_text = format!("{days} days ago");
                    let age = if memory.is_stale {
                        colors::warning(&age_text)
                    } else {
                        colors::success(&age_text)
                    };
                    memory_info.push_str(&format!("Age:        {}\n", age));
                }
                if memory.is_stale {
                    memory_info.push_str(&format!(
                        "Signal:     {}\n",
                        colors::warning("adoption low")
                    ));
                    if let Some(ref rec) = memory.recommendation {
                        memory_info.push_str(&format!("Tip:        {}\n", colors::dim(rec)));
                    }
                } else {
                    memory_info.push_str(&format!("Signal:     {}\n", colors::success("healthy")));
                }
                blocks.push(("Agent Memory".to_string(), memory_info));
            }

            // 4. Recommendation (The "Smarter" bit)
            let mut recommendation = String::new();
            if out.baseline.is_none() {
                recommendation.push_str(&format!(
                    "Repository not setup. Run: {}\n",
                    colors::success("sruja start")
                ));
            } else if out.truth_status == "drifted" {
                recommendation.push_str(&format!(
                    "Architecture drifted. Run: {} to review changes.\n",
                    colors::info("sruja daily")
                ));
            } else if out.violations_count > 0 {
                recommendation.push_str("Maintain health by resolving active violations.\n");
            } else {
                recommendation.push_str("Looking good! Keep running sruja in your PR pipeline.\n");
            }
            if out.agent_memory.as_ref().is_some_and(|m| m.is_stale) {
                recommendation.push_str(
                    "Agent memory adoption is low — record guardrails when Sruja catches a miss.\n",
                );
            }
            blocks.push(("Recommended Next Step".to_string(), recommendation));

            println!(
                "{}",
                formatter.format_dashboard("SRUJA REPOSITORY STATUS", blocks)
            );
        }
    }

    Ok(())
}
