use git2::{DiffOptions, Repository};
use sruja_scan::Graph;
use std::collections::HashMap;
use std::path::Path;

use crate::types::ComponentDiff;

/// Map a git diff between two refs to architectural components in the graph.
///
/// This provides a micro-to-macro mapping, identifying which high-level components
/// are being touched by low-level code changes.
pub fn map_git_diff(
    repo_path: &Path,
    base_ref: &str,
    head_ref: &str,
    graph: &Graph,
) -> Result<Vec<ComponentDiff>, git2::Error> {
    let repo = Repository::open(repo_path)?;

    // Resolve refs to trees
    let base_obj = repo.revparse_single(base_ref)?;
    let head_obj = repo.revparse_single(head_ref)?;

    let base_tree = base_obj.peel_to_tree()?;
    let head_tree = head_obj.peel_to_tree()?;

    let mut opts = DiffOptions::new();
    opts.pathspec("*"); // Ensure we catch all files
    opts.context_lines(0);

    let diff = repo.diff_tree_to_tree(Some(&base_tree), Some(&head_tree), Some(&mut opts))?;

    let component_changes = std::cell::RefCell::new(HashMap::<String, ComponentDiff>::new());

    diff.foreach(
        &mut |delta, _| {
            // We use the new_file path for mapping
            if let Some(new_file) = delta.new_file().path() {
                let file_path = new_file.to_string_lossy().to_string();

                let mapped_components = find_components_for_file(&file_path, graph);
                let mut changes = component_changes.borrow_mut();
                for comp_id in mapped_components {
                    let entry = changes
                        .entry(comp_id.clone())
                        .or_insert_with(|| ComponentDiff {
                            component_id: comp_id,
                            files_changed: Vec::new(),
                            lines_added: 0,
                            lines_deleted: 0,
                        });
                    if !entry.files_changed.contains(&file_path) {
                        entry.files_changed.push(file_path.clone());
                    }
                }
            }
            true
        },
        None,
        None,
        Some(&mut |delta, _hunk, line| {
            if let Some(new_file) = delta.new_file().path() {
                let file_path = new_file.to_string_lossy().to_string();
                let mapped_components = find_components_for_file(&file_path, graph);

                let origin = line.origin();
                let mut changes = component_changes.borrow_mut();
                for comp_id in mapped_components {
                    if let Some(entry) = changes.get_mut(&comp_id) {
                        match origin {
                            '+' => entry.lines_added += 1,
                            '-' => entry.lines_deleted += 1,
                            _ => {}
                        }
                    }
                }
            }
            true
        }),
    )?;

    Ok(component_changes.into_inner().into_values().collect())
}

/// Helper to find which architectural components a file belongs to.
fn find_components_for_file(file_path: &str, graph: &Graph) -> Vec<String> {
    let mut matching = Vec::new();

    // Normalize path for comparison (strip leading ./ if present)
    let norm_file = file_path.strip_prefix("./").unwrap_or(file_path);

    for node in &graph.nodes {
        let mut matched = false;

        // 1. Check explicit sources
        for source in &node.sources {
            let norm_source = source.path.strip_prefix("./").unwrap_or(&source.path);
            if norm_file == norm_source {
                matched = true;
                break;
            }
        }

        // 2. Check base path (directory or exact file)
        if !matched {
            if let Some(ref node_path) = node.path {
                let norm_node_path = node_path.strip_prefix("./").unwrap_or(node_path);
                if norm_file == norm_node_path
                    || norm_file.starts_with(&format!("{}/", norm_node_path))
                {
                    matched = true;
                }
            }
        }

        if matched {
            matching.push(node.id.clone());
        }
    }

    matching
}
