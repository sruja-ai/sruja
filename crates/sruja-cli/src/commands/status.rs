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
            colors::print_header("🩺 Sruja Repo Status");
            
            if let Some(ref base) = out.baseline {
                println!("  {} {}", colors::dim("Baseline: "), colors::style(base).bold());
            } else {
                println!("  {} {}", colors::dim("Baseline: "), colors::warning("not found"));
                println!("    {} sruja start -r {} --prompt", colors::dim("Run: "), repo_root);
            }

            let status_color = match out.truth_status.as_str() {
                "reviewed" => colors::success(&out.truth_status),
                "drifted" => colors::error(&out.truth_status),
                _ => colors::warning(&out.truth_status),
            };

            println!(
                "  {} {} ({} violations)",
                colors::dim("Truth Status: "),
                status_color,
                out.violations_count
            );

            if let Some(score) = out.health_score {
                println!(
                    "  {} {}",
                    colors::dim("Health Score: "),
                    colors::health_bar(score, 20)
                );
            }

            if let Some(ref ts) = out.context_updated_at {
                // Try to parse ISO 8601 for relative display
                let time_display = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
                    let now = chrono::Utc::now();
                    let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                    if duration.num_seconds() > 0 {
                        format!("{} ago", colors::elapsed_display(std::time::Duration::from_secs(duration.num_seconds() as u64)))
                    } else {
                        "just now".to_string()
                    }
                } else {
                    ts.clone()
                };
                println!("  {} {}", colors::dim("Last Refresh: "), time_display);
            }

            println!();
            println!("{}", colors::style("Environment Diagnostics:").bold());
            
            let srujaignore_exists = repo_path.join(".srujaignore").exists();
            println!(
                "  {} .srujaignore: {}",
                if srujaignore_exists { colors::success("✓") } else { colors::dim("•") },
                if srujaignore_exists { "active".to_string() } else { colors::dim("none (using defaults)").to_string() }
            );

            let git_dir = repo_path.join(".git");
            let hook_exists = git_dir.join("hooks/pre-commit").exists();
            let is_git = git_dir.exists();
            
            if is_git {
                println!(
                    "  {} Git Hooks:    {}",
                    if hook_exists { colors::success("✓") } else { colors::dim("•") },
                    if hook_exists { "pre-commit installed".to_string() } else { colors::dim("not installed (sruja init --hook)").to_string() }
                );
            }

            println!("  {} Version:      {}", colors::success("✓"), env!("CARGO_PKG_VERSION"));
            
            println!();
            if out.baseline.is_some() {
                println!("  {} sruja daily -r {}", colors::dim("Run: "), repo_root);
            }
        }
    }

    Ok(())
}
