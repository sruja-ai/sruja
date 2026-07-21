use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::super::super::helpers::*;
use super::super::finish;
use crate::commands::CliError;

pub(crate) async fn handle(
    arguments: &Value,
    repo: &str,
    graph_cache: &Arc<Mutex<HashMap<String, sruja_scan::Graph>>>,
) -> Result<Option<String>, CliError> {
    let element_id = arguments.get("element_id").and_then(|v| v.as_str());
    let file = arguments.get("file").and_then(|v| v.as_str());
    let query = arguments.get("query").and_then(|v| v.as_str());
    let base_ref = arguments.get("base_ref").and_then(|v| v.as_str());
    let head_ref = arguments.get("head_ref").and_then(|v| v.as_str());
    let workflow_id = arguments.get("workflow_id").and_then(|v| v.as_str());
    let phase = arguments.get("phase").and_then(|v| v.as_str());
    let depth = arguments
        .get("depth")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as usize;
    let mut max_tokens = arguments
        .get("max_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(10000) as usize;
    if let (Some(wid), Some(ph)) = (workflow_id, phase) {
        if ph == "construction" {
            max_tokens = max_tokens.min(4000);
        }
        if let Ok(manifest) = crate::commands::workflow_get(repo, wid) {
            if !manifest.target_elements.is_empty()
                && element_id.is_none()
                && file.is_none()
            {
                let _ = manifest;
            }
        }
    }
    let enrich = arguments
        .get("enrich")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let enrich_provider = arguments.get("enrich_provider").and_then(|v| v.as_str());
    let enrich_cmd = arguments.get("enrich_cmd").and_then(|v| v.as_str());
    let enrich_model = arguments.get("enrich_model").and_then(|v| v.as_str());
    let enrich_base_url = arguments.get("enrich_base_url").and_then(|v| v.as_str());
    let enrich_timeout_ms = arguments
        .get("enrich_timeout_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(15000);
    let enrich_max_bytes = arguments
        .get("enrich_max_bytes")
        .and_then(|v| v.as_u64())
        .unwrap_or(20000) as usize;
    let cache_friendly = arguments
        .get("cache_friendly")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let graph = get_or_scan_graph(graph_cache, repo).await?;
    let selectors = crate::commands::context::logic::TaskSelectors {
        element_id,
        file,
        query,
        base_ref,
        head_ref,
        depth: Some(depth),
    };

    let ctx =
        crate::commands::context::logic::build_task_context(&graph, repo, selectors, max_tokens)?;
    if !enrich && enrich_cmd.is_none() {
        if cache_friendly {
            let arch = crate::commands::context::logic::build_architecture_context(
                &graph, repo, None, None, depth, max_tokens,
            )?;
            let export = crate::commands::context::logic::build_cache_friendly_task_export(
                repo, &arch, ctx,
            );
            return finish(Ok(serde_json::to_string_pretty(&export)?));
        }
        let mut val = serde_json::to_value(&ctx)?;
        if workflow_id.is_some() || phase.is_some() {
            if let Some(obj) = val.as_object_mut() {
                if let Some(wid) = workflow_id {
                    obj.insert(
                        "workflow_id".to_string(),
                        serde_json::Value::String(wid.to_string()),
                    );
                }
                if let Some(ph) = phase {
                    obj.insert(
                        "workflow_phase".to_string(),
                        serde_json::Value::String(ph.to_string()),
                    );
                }
                obj.insert(
                    "max_tokens_applied".to_string(),
                    serde_json::json!(max_tokens),
                );
            }
        }
        return finish(Ok(serde_json::to_string_pretty(&val)?));
    }

    let wrapped = enrich_wrapper_json(
        Path::new(&repo),
        enrich_provider,
        enrich_cmd,
        enrich_model,
        enrich_base_url,
        enrich_timeout_ms,
        enrich_max_bytes,
        "task_context",
        serde_json::to_value(&ctx)?,
    );
    finish(Ok(serde_json::to_string_pretty(&wrapped)?))
}
