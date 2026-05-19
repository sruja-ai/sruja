pub mod draft_summary;
pub mod drift;
pub mod output;
pub mod quickstart;

use crate::commands::CliError;
use std::fs;
use std::path::Path;

pub use drift::{drift, drift_json_string, drift_pr, status_result};
pub use quickstart::quickstart;

pub async fn scan(repo_root: &str, output: &str) -> Result<(), CliError> {
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Scanning repository...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let graph_result = match std::env::var("SRUJA_TIMEOUT")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
    {
        Some(secs) => {
            let repo_root_owned = repo_root.to_string();
            let result = tokio::time::timeout(
                std::time::Duration::from_secs(secs),
                tokio::task::spawn_blocking(move || {
                    sruja_scan::scan_repo(Path::new(&repo_root_owned))
                }),
            )
            .await;
            match result {
                Ok(Ok(g)) => g.map_err(|e| CliError::scan(e.to_string())),
                Ok(Err(_)) => Err(CliError::timeout("Scan task panicked")),
                Err(_) => Err(CliError::timeout(format!(
                    "Scan exceeded {}s timeout. Use .srujaignore to narrow scope or increase SRUJA_TIMEOUT.",
                    secs
                ))),
            }
        }
        None => {
            sruja_scan::scan_repo(Path::new(repo_root)).map_err(|e| CliError::scan(e.to_string()))
        }
    };
    pb.finish_and_clear();
    let graph = graph_result?;

    let json = serde_json::to_string_pretty(&graph)?;

    if output == "-" {
        println!("{}", json);
        return Ok(());
    }

    fs::write(output, json)?;
    println!("Wrote {}", output);
    Ok(())
}
