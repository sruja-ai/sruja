//! Status command: show repo health, baseline, and truth status.

use std::path::Path;
use super::scan::status_result;
use super::CliError;
use crate::utils::colors;

fn escape_github_actions_message(input: &str) -> String {
    input
        .replace('%', "%25")
        .replace('\n', "%0A")
        .replace('\r', "%0D")
}

/// Run status and print text or JSON.
pub async fn status(repo_root: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let out = status_result(repo_root).await?;

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&out)?);
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
        }
        _ => {
            use crate::utils::table_formatter::TableFormatter;
            let formatter = TableFormatter::auto();
            
            let mut blocks = Vec::new();

            // 1. Core Health Block
            let mut health_info = String::new();
            if let Some(score) = out.health_score {
                let health_bar = colors::health_bar(score, 20);
                let trend = if !out.health_history.is_empty() {
                    format!(" | Trend: {}", TableFormatter::format_sparkline(&out.health_history))
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
            health_info.push_str(&format!("Truth:  {} ({} violations)\n", status_color, out.violations_count));
            
            if let Some(ref base) = out.baseline {
                health_info.push_str(&format!("Config: {}\n", colors::info(base)));
            } else {
                health_info.push_str(&format!("Config: {}\n", colors::warning("not found")));
            }

            blocks.push(("Architecture Health".to_string(), health_info));

            // 2. Findings Preview
            if !out.top_findings.is_empty() {
                let mut findings_info = String::new();
                for (i, f) in out.top_findings.iter().enumerate() {
                    let severity_icon = match f.severity.as_str() {
                        "error" => colors::error("✗"),
                        "warning" => colors::warning("⚠"),
                        _ => colors::info("ℹ"),
                    };
                    findings_info.push_str(&format!("{}. {} {} - {}\n", i + 1, severity_icon, colors::style(&f.kind).bold(), f.message));
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
                        colors::elapsed_display(std::time::Duration::from_secs(duration.num_seconds() as u64))
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
            env_info.push_str(&format!("Git Hook:   {}\n", if hook_exists { colors::success("installed") } else { colors::dim("missing") }));
            env_info.push_str(&format!("CLI Ver:    {}\n", colors::dim(env!("CARGO_PKG_VERSION"))));
            
            blocks.push(("Environment".to_string(), env_info));

            // 4. Recommendation (The "Smarter" bit)
            let mut recommendation = String::new();
            if out.baseline.is_none() {
                recommendation.push_str(&format!("Repository not setup. Run: {}\n", colors::success("sruja start")));
            } else if out.truth_status == "drifted" {
                recommendation.push_str(&format!("Architecture drifted. Run: {} to review changes.\n", colors::info("sruja daily")));
            } else if out.violations_count > 0 {
                recommendation.push_str("Maintain health by resolving active violations.\n");
            } else {
                recommendation.push_str("Looking good! Keep running sruja in your PR pipeline.\n");
            }
            blocks.push(("Recommended Next Step".to_string(), recommendation));

            println!("{}", formatter.format_dashboard("SRUJA REPOSITORY STATUS", blocks));
        }
    }

    Ok(())
}
