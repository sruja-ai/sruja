//! Graph history command - temporal queries on graph snapshots

use crate::commands::CliError;
use crate::graph_store::SNAPSHOTS_FILE;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sruja_graph::{GraphDelta, GraphSnapshot};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    pub timestamp: DateTime<Utc>,
    pub commit_sha: String,
    pub deltas: Vec<GraphDelta>,
    pub summary: ChangeSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSummary {
    pub nodes_added: usize,
    pub nodes_removed: usize,
    pub nodes_changed: usize,
    pub edges_added: usize,
    pub edges_removed: usize,
    pub decisions_added: usize,
    pub decisions_changed: usize,
    pub learnings_added: usize,
    pub learnings_changed: usize,
    pub learnings_removed: usize,
}

pub fn graph_history(
    repo: &str,
    since: Option<&str>,
    element: Option<&str>,
    kind: Option<&str>,
    format: &str,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let snapshots = load_snapshots(repo_path)?;

    let since_dt = parse_since(since);

    let filtered: Vec<_> = snapshots
        .into_iter()
        .filter(|s| {
            if let Some(since) = since_dt {
                s.timestamp >= since
            } else {
                true
            }
        })
        .filter(|s| {
            if let Some(elem) = element {
                s.deltas.iter().any(|d| d.references_element(elem))
            } else {
                true
            }
        })
        .filter(|s| {
            if let Some(k) = kind {
                s.deltas.iter().any(|d| d.kind_str() == k)
            } else {
                true
            }
        })
        .collect();

    let changesets = group_into_changesets(filtered);

    match format {
        "json" => println!("{}", serde_json::to_string_pretty(&changesets)?),
        _ => print_changesets_text(&changesets),
    }

    Ok(())
}

fn load_snapshots(repo: &Path) -> Result<Vec<GraphSnapshot>, CliError> {
    let path = repo.join(SNAPSHOTS_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&path)?;
    let mut snapshots = Vec::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(snapshot) = serde_json::from_str::<GraphSnapshot>(line) {
            snapshots.push(snapshot);
        }
    }

    Ok(snapshots)
}

fn parse_since(since: Option<&str>) -> Option<DateTime<Utc>> {
    let s = since?;
    let days = parse_duration_days(s)?;
    Some(Utc::now() - Duration::days(days))
}

fn parse_duration_days(s: &str) -> Option<i64> {
    let s = s.trim().to_lowercase();
    if let Some(stripped) = s.strip_suffix('d') {
        stripped.parse::<i64>().ok()
    } else if let Some(stripped) = s.strip_suffix("days") {
        stripped.parse::<i64>().ok()
    } else if let Some(stripped) = s.strip_suffix('w') {
        stripped.parse::<i64>().ok().map(|w| w * 7)
    } else if let Some(stripped) = s.strip_suffix("weeks") {
        stripped.parse::<i64>().ok().map(|w| w * 7)
    } else {
        s.parse::<i64>().ok()
    }
}

fn group_into_changesets(snapshots: Vec<GraphSnapshot>) -> Vec<ChangeSet> {
    snapshots
        .into_iter()
        .map(|s| {
            let summary = compute_summary(&s.deltas);
            ChangeSet {
                timestamp: s.timestamp,
                commit_sha: s.commit_sha,
                deltas: s.deltas,
                summary,
            }
        })
        .collect()
}

fn compute_summary(deltas: &[GraphDelta]) -> ChangeSummary {
    let mut summary = ChangeSummary {
        nodes_added: 0,
        nodes_removed: 0,
        nodes_changed: 0,
        edges_added: 0,
        edges_removed: 0,
        decisions_added: 0,
        decisions_changed: 0,
        learnings_added: 0,
        learnings_changed: 0,
        learnings_removed: 0,
    };

    for delta in deltas {
        match delta {
            GraphDelta::NodeAdded { .. } => summary.nodes_added += 1,
            GraphDelta::NodeRemoved { .. } => summary.nodes_removed += 1,
            GraphDelta::NodeChanged { .. } => summary.nodes_changed += 1,
            GraphDelta::EdgeAdded { .. } => summary.edges_added += 1,
            GraphDelta::EdgeRemoved { .. } => summary.edges_removed += 1,
            GraphDelta::DecisionAdded { .. } => summary.decisions_added += 1,
            GraphDelta::DecisionStatusChanged { .. } => summary.decisions_changed += 1,
            GraphDelta::LearningAdded { .. } => summary.learnings_added += 1,
            GraphDelta::LearningChanged { .. } => summary.learnings_changed += 1,
            GraphDelta::LearningRemoved { .. } => summary.learnings_removed += 1,
        }
    }

    summary
}

fn print_changesets_text(changesets: &[ChangeSet]) {
    if changesets.is_empty() {
        println!("No graph changes found.");
        return;
    }

    println!("Graph History ({} snapshots)", changesets.len());
    println!("{}", "-".repeat(60));

    for cs in changesets.iter().rev() {
        println!(
            "\n{} ({})",
            cs.timestamp.format("%Y-%m-%d %H:%M:%S"),
            cs.commit_sha
        );

        let s = &cs.summary;
        let mut parts = Vec::new();
        if s.nodes_added > 0 {
            parts.push(format!("+{} nodes", s.nodes_added));
        }
        if s.nodes_removed > 0 {
            parts.push(format!("-{} nodes", s.nodes_removed));
        }
        if s.nodes_changed > 0 {
            parts.push(format!("~{} nodes", s.nodes_changed));
        }
        if s.edges_added > 0 {
            parts.push(format!("+{} edges", s.edges_added));
        }
        if s.edges_removed > 0 {
            parts.push(format!("-{} edges", s.edges_removed));
        }
        if s.decisions_added > 0 {
            parts.push(format!("+{} decisions", s.decisions_added));
        }
        if s.decisions_changed > 0 {
            parts.push(format!("~{} decisions", s.decisions_changed));
        }
        if s.learnings_added > 0 {
            parts.push(format!("+{} learnings", s.learnings_added));
        }
        if s.learnings_changed > 0 {
            parts.push(format!("~{} learnings", s.learnings_changed));
        }
        if s.learnings_removed > 0 {
            parts.push(format!("-{} learnings", s.learnings_removed));
        }

        if !parts.is_empty() {
            println!("  {}", parts.join(", "));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_days() {
        assert_eq!(parse_duration_days("7d"), Some(7));
        assert_eq!(parse_duration_days("30days"), Some(30));
        assert_eq!(parse_duration_days("2w"), Some(14));
        assert_eq!(parse_duration_days("3weeks"), Some(21));
        assert_eq!(parse_duration_days("14"), Some(14));
        assert_eq!(parse_duration_days("invalid"), None);
    }

    #[test]
    fn test_compute_summary() {
        use sruja_graph::GraphDelta;

        let deltas = vec![
            GraphDelta::NodeAdded {
                node_id: "n1".to_string(),
                kind: "service".to_string(),
                label: "Test".to_string(),
            },
            GraphDelta::EdgeAdded {
                source: "a".to_string(),
                target: "b".to_string(),
                kind: "calls".to_string(),
            },
            GraphDelta::NodeRemoved {
                node_id: "n2".to_string(),
            },
        ];

        let summary = compute_summary(&deltas);
        assert_eq!(summary.nodes_added, 1);
        assert_eq!(summary.nodes_removed, 1);
        assert_eq!(summary.edges_added, 1);
    }
}
