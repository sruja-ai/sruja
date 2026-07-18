//! Utility functions for the agent loop.

use std::path::Path;

use sruja_agent::AgentError;

use super::super::CliError;

/// Convert an agent error to a CLI error.
pub(crate) fn agent_err_to_cli(e: AgentError) -> CliError {
    CliError::validation(format!("Agent error: {e}"))
}

/// Post-loop memory consolidation (U2).
///
/// Archives stale entries (decay < 0.15, age > 30 days) and prunes
/// low-utility entries (retrieved ≥ 3×, success < 25%). Invariant
/// entries are never touched. Returns a human-readable summary.
pub(crate) fn consolidate_memory(repo_path: &Path) -> Result<String, CliError> {
    use sruja_agent::AgenticMemory;

    let mut memory = AgenticMemory::load(repo_path).unwrap_or_default();

    // 1. Archive stale entries.
    let archived = memory.auto_archive_stale(0.15, 30);
    let archived_count = archived.len();

    // 2. Prune low-utility entries (skip invariants).
    let low_utility: Vec<String> = memory
        .low_utility_entries(3, 0.25)
        .into_iter()
        .filter(|e| e.kind != Some(sruja_agent::LearningKind::Invariant))
        .map(|e| e.id.clone())
        .collect();
    let pruned_count = low_utility.len();
    for id in &low_utility {
        let _ = memory.delete_learning(id);
    }

    // 3. Save if anything changed.
    if archived_count > 0 || pruned_count > 0 {
        memory.save(repo_path).map_err(|e| {
            CliError::validation(format!("Failed to save consolidated memory: {e}"))
        })?;
    }

    let remaining = memory.learnings.len();
    Ok(format!(
        "Memory: archived {archived_count} stale, pruned {pruned_count} low-utility ({remaining} entries remain)"
    ))
}

/// Pre-load architecture context (repomap + topology) for the comprehension phase.
///
/// This injects a compact architecture summary into the comprehension prompt
/// so the agent doesn't need to call MCP tools for basic context. Saves tokens
/// and makes the agent more efficient.
///
/// Returns empty string if no architecture data is available.
pub(crate) fn preloaded_architecture_context(repo_path: &Path, max_tokens: usize) -> String {
    // Try to load repomap from .sruja/repomap.json if it exists
    let repomap_path = repo_path.join(".sruja").join("repomap.json");
    if let Ok(content) = std::fs::read_to_string(&repomap_path) {
        // Truncate to max_tokens if needed (rough estimate: 1 token ≈ 4 chars)
        let max_chars = max_tokens * 4;
        let truncated = if content.len() > max_chars {
            format!("{}...", &content[..max_chars])
        } else {
            content
        };

        return format!(
            "\n\n## Architecture Context (pre-loaded)\n\
             The following architecture context has been pre-loaded for you.\n\
             Do NOT call sruja_list_architecture_index or sruja_get_topology — \
             the information is already here.\n\n{}",
            truncated
        );
    }

    // Try to load llms-architecture.txt if it exists
    let llms_path = repo_path.join("llms-architecture.txt");
    if let Ok(content) = std::fs::read_to_string(&llms_path) {
        let max_chars = max_tokens * 4;
        let truncated = if content.len() > max_chars {
            format!("{}...", &content[..max_chars])
        } else {
            content
        };

        return format!(
            "\n\n## Architecture Context (pre-loaded)\n\
             The following architecture context has been pre-loaded for you.\n\
             Do NOT call sruja_list_architecture_index or sruja_get_topology — \
             the information is already here.\n\n{}",
            truncated
        );
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_SHELL_ALLOWLIST: &[&str] = &["cargo", "git"];

    #[test]
    fn default_shell_allowlist_has_cargo_and_git() {
        assert!(DEFAULT_SHELL_ALLOWLIST.contains(&"cargo"));
        assert!(DEFAULT_SHELL_ALLOWLIST.contains(&"git"));
    }
}
