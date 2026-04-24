use git2::{DiffOptions, Repository};
use serde::{Deserialize, Serialize};
use sruja_scan::Graph;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::proposal::{Proposal, ProposalStatus};
use crate::types::ComponentDiff;

/// Architectural velocity and supervision metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalVelocity {
    /// Total graph nodes touched by code changes
    pub nodes_changed: usize,
    /// Nodes that had a corresponding approved proposal
    pub nodes_with_intent: usize,
    /// supervision_ratio = nodes_with_intent / nodes_changed (1.0 = fully supervised)
    pub supervision_ratio: f32,
    /// Per-component breakdown
    pub component_diffs: Vec<ComponentDiff>,
    /// Nodes changed without intent
    pub unsupervised_nodes: Vec<String>,
}

/// Calculate architectural velocity by comparing git changes to approved proposals.
pub fn architectural_velocity(
    repo_path: &Path,
    base_ref: &str,
    head_ref: &str,
    graph: &Graph,
) -> Result<ArchitecturalVelocity, git2::Error> {
    let diffs = map_git_diff(repo_path, base_ref, head_ref, graph)?;
    let proposals = Proposal::load_all(repo_path).unwrap_or_default();
    
    let approved_ids: HashSet<String> = proposals
        .iter()
        .filter(|p| p.status == ProposalStatus::Approved || p.status == ProposalStatus::Implemented)
        .flat_map(|p| p.get_affected_ids())
        .collect();

    let nodes_changed = diffs.len();
    let mut nodes_with_intent = 0;
    let mut unsupervised_nodes = Vec::new();

    for diff in &diffs {
        if approved_ids.contains(&diff.component_id) {
            nodes_with_intent += 1;
        } else {
            unsupervised_nodes.push(diff.component_id.clone());
        }
    }

    Ok(ArchitecturalVelocity {
        nodes_changed,
        nodes_with_intent,
        supervision_ratio: if nodes_changed > 0 {
            nodes_with_intent as f32 / nodes_changed as f32
        } else {
            1.0
        },
        component_diffs: diffs,
        unsupervised_nodes,
    })
}

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
pub fn find_components_for_file(file_path: &str, graph: &Graph) -> Vec<String> {
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
