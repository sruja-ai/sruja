//! MCP tools for classification and IDE rules management.

use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::finish;
use crate::commands::CliError;

pub(crate) async fn try_run(
    name: &str,
    arguments: &Value,
    repo: &str,
    _graph_cache: &Arc<Mutex<HashMap<String, sruja_scan::Graph>>>,
) -> Result<Option<String>, CliError> {
    match name {
        "sruja_classify" => {
            let force = arguments
                .get("force")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            // If classification JSON is provided, write it directly.
            if let Some(classification) = arguments.get("classification") {
                let repo_path = Path::new(repo);
                let sruja_dir = repo_path.join(".sruja");
                if !sruja_dir.exists() {
                    std::fs::create_dir_all(&sruja_dir)?;
                }
                let classification_path = sruja_dir.join("classification.json");
                if classification_path.exists() && !force {
                    return finish(Err(CliError::validation(format!(
                        "Classification already exists at {}. Use force=true to overwrite.",
                        classification_path.display()
                    ))));
                }
                let json = serde_json::to_string_pretty(classification).map_err(|e| {
                    CliError::validation(format!("Failed to serialize classification: {}", e))
                })?;
                std::fs::write(&classification_path, json)?;
                return finish(Ok(format!(
                    "Classification written to {}",
                    classification_path.display()
                )));
            }

            // Otherwise, run heuristic classification.
            crate::commands::utility_domain::classify(
                crate::commands::utility_domain::ClassifyOptions { repo, force },
            )?;
            finish(Ok("Classification generated successfully".to_string()))
        }

        "sruja_sync_ide_rules" => {
            let max_tokens = arguments
                .get("max_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(10000) as usize;
            let check = arguments
                .get("check")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            crate::commands::utility_domain::sync_ide_rules(
                crate::commands::utility_domain::SyncIdeRulesOptions {
                    repo,
                    max_tokens,
                    check,
                },
            )
            .await?;
            finish(Ok("IDE rules synced successfully".to_string()))
        }

        _ => Ok(None),
    }
}
