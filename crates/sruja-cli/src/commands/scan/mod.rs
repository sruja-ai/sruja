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

    let graph_result =
        sruja_scan::scan_repo(Path::new(repo_root)).map_err(|e| CliError::scan(e.to_string()));
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
