//! Status command: show repo health, baseline, and truth status.

use super::scan::status_result;
use super::CliError;

fn escape_github_actions_message(input: &str) -> String {
    input
        .replace('%', "%25")
        .replace('\n', "%0A")
        .replace('\r', "%0D")
}

/// Run status and print text or JSON.
pub async fn status(repo_root: &str, format: &str) -> Result<(), CliError> {
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
            if let Some(ref base) = out.baseline {
                println!("Baseline: {}", base);
            } else {
                println!("No baseline (repo.sruja / architecture.sruja)");
                println!("Start here: sruja start -r {} --prompt", repo_root);
                println!("Quick repo overview: sruja overview -r {}", repo_root);
            }
            println!(
                "Truth: {} ({} violation(s))",
                out.truth_status, out.violations_count
            );
            if let Some(score) = out.health_score {
                println!("Structural health score: {}/100", score);
            }
            if let Some(ref ts) = out.context_updated_at {
                println!("Last evidence refresh: {}", ts);
            } else if out.baseline.is_some() {
                println!("Last evidence refresh: not generated yet");
            }
            if out.baseline.is_some() {
                println!("Daily review: sruja daily -r {}", repo_root);
            }
        }
    }

    Ok(())
}
