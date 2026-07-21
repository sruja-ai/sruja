use std::path::Path;
use std::process::Command;

use crate::commands::CliError;
use sruja_graph::KnowledgeGraph;

pub fn resolve_target(
    graph: &KnowledgeGraph,
    repo_path: &Path,
    file: Option<&str>,
    element_id: Option<&str>,
) -> Result<String, CliError> {
    if let Some(eid) = element_id {
        if graph.nodes.contains_key(eid) {
            return Ok(eid.to_string());
        }
        let suffix = format!(".{}", eid);
        let mut matches: Vec<&str> = graph
            .nodes
            .keys()
            .filter(|k| *k == eid || k.ends_with(&suffix))
            .map(|k| k.as_str())
            .collect();
        matches.sort_unstable();
        matches.dedup();
        match matches.len() {
            0 => Err(CliError::validation(format!(
                "No architecture element matches '{}'. Run 'sruja list repo.sruja' to see available elements.",
                eid
            ))),
            1 => Ok(matches[0].to_string()),
            _ => {
                let preview: Vec<&str> = matches.iter().take(5).copied().collect();
                Err(CliError::validation(format!(
                    "Ambiguous element '{}'. Matches: {}",
                    eid,
                    preview.join(", ")
                )))
            }
        }?;
    }

    if let Some(file_path) = file {
        let scan = sruja_scan::scan_repo(repo_path).map_err(|e| {
            CliError::validation(format!(
                "Failed to scan repo for file focus resolution: {e}"
            ))
        })?;
        let centrality = crate::commands::compute_all_centrality_cached(repo_path, &scan, false)?;
        let focus_ctx = crate::commands::context::logic::build_focus_context(
            &scan,
            repo_path.to_string_lossy().as_ref(),
            file_path,
            None,
            0,
            0,
            &centrality,
        )?;
        if let Some(first) = focus_ctx.matched_nodes.first() {
            return Ok(first.id.clone());
        }

        return Err(CliError::validation(format!(
            "Could not resolve file '{}' to an architecture element. Try --element-id instead, or ensure your .sruja maps this file.",
            file_path
        )));
    }

    Err(CliError::validation(
        "Provide --file or --element-id to focus on a specific part of the architecture."
            .to_string(),
    ))
}

pub(super) fn git_arch_blob_blake3(repo: &Path, git_ref: &str, path_in_repo: &str) -> Option<String> {
    let spec = format!("{git_ref}:{path_in_repo}");
    let out = Command::new("git")
        .args(["show", spec.as_str()])
        .current_dir(repo)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(blake3::hash(&out.stdout).to_hex().to_string())
}
