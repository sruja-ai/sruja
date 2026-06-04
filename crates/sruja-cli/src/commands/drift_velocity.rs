//! Drift Velocity - Trend tracking for architecture changes

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sruja_graph::{GraphDelta, GraphSnapshot};
use std::path::Path;

use crate::graph_store::SNAPSHOTS_FILE;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftVelocity {
    pub period: String,
    pub node_count_delta: i64,
    pub edge_count_delta: i64,
    pub violation_count_delta: i64,
    pub complexity_delta: f64,
    pub trend: TrendDirection,
    pub snapshot_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
}

impl std::fmt::Display for TrendDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrendDirection::Improving => write!(f, "Improving"),
            TrendDirection::Stable => write!(f, "Stable"),
            TrendDirection::Degrading => write!(f, "Degrading"),
        }
    }
}

/// Compute drift velocity for a given period
pub fn compute_velocity(
    repo: &Path,
    period_days: i64,
) -> Result<DriftVelocity, crate::commands::CliError> {
    let snapshots = load_snapshots(repo)?;
    let cutoff = Utc::now() - Duration::days(period_days);

    let recent: Vec<_> = snapshots.iter().filter(|s| s.timestamp >= cutoff).collect();

    let mut node_delta: i64 = 0;
    let mut edge_delta: i64 = 0;
    let mut decision_delta: i64 = 0;
    let mut learning_delta: i64 = 0;

    for snapshot in &recent {
        for delta in &snapshot.deltas {
            match delta {
                GraphDelta::NodeAdded { .. } => node_delta += 1,
                GraphDelta::NodeRemoved { .. } => node_delta -= 1,
                GraphDelta::EdgeAdded { .. } => edge_delta += 1,
                GraphDelta::EdgeRemoved { .. } => edge_delta -= 1,
                GraphDelta::DecisionAdded { .. } => decision_delta += 1,
                GraphDelta::DecisionStatusChanged { .. } => {} // status change, not count
                GraphDelta::LearningAdded { .. } => learning_delta += 1,
                GraphDelta::LearningChanged { .. } => {} // field change, not count
                GraphDelta::LearningRemoved { .. } => learning_delta -= 1,
                GraphDelta::NodeChanged { .. } => {} // field change, not count
            }
        }
    }

    let trend = determine_trend(node_delta, edge_delta, decision_delta, learning_delta);

    Ok(DriftVelocity {
        period: format!("{}d", period_days),
        node_count_delta: node_delta,
        edge_count_delta: edge_delta,
        violation_count_delta: decision_delta + learning_delta,
        complexity_delta: 0.0,
        trend,
        snapshot_count: recent.len(),
    })
}

/// Determine trend direction based on graph changes.
/// Positive deltas indicate growth (more nodes/edges/decisions), which is generally
/// degrading for maintainability. Negative deltas indicate simplification.
fn determine_trend(
    node_delta: i64,
    edge_delta: i64,
    decision_delta: i64,
    learning_delta: i64,
) -> TrendDirection {
    // Weight: nodes and edges are the primary signal, decisions/learnings secondary
    let structural_delta = node_delta + edge_delta;
    let knowledge_delta = decision_delta + learning_delta;

    // Positive structural growth is degrading; negative is improving
    // Knowledge growth (decisions, learnings) is generally positive (more documented context)
    let score = structural_delta - (knowledge_delta / 2);

    if score > 5 {
        TrendDirection::Degrading
    } else if score < -5 {
        TrendDirection::Improving
    } else {
        TrendDirection::Stable
    }
}

fn load_snapshots(repo: &Path) -> Result<Vec<GraphSnapshot>, crate::commands::CliError> {
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

/// Format drift velocity for text output
pub fn format_velocity_text(velocity: &DriftVelocity) -> String {
    use std::fmt::Write;

    let mut output = String::new();

    writeln!(output, "\nVelocity ({}):", velocity.period).unwrap();
    writeln!(output, "  Nodes: {:+}", velocity.node_count_delta).unwrap();
    writeln!(output, "  Edges: {:+}", velocity.edge_count_delta).unwrap();

    if velocity.violation_count_delta != 0 {
        writeln!(output, "  Violations: {:+}", velocity.violation_count_delta).unwrap();
    }

    writeln!(output, "  Trend: {}", velocity.trend).unwrap();

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_trend() {
        // Structural growth is degrading
        assert_eq!(determine_trend(10, 0, 0, 0), TrendDirection::Degrading);
        // Structural shrinkage is improving
        assert_eq!(determine_trend(-10, 0, 0, 0), TrendDirection::Improving);
        // Balanced is stable
        assert_eq!(determine_trend(2, 2, 0, 0), TrendDirection::Stable);
        // Knowledge growth offsets structural growth
        assert_eq!(determine_trend(10, 0, 5, 5), TrendDirection::Stable);
    }

    #[test]
    fn test_format_velocity_text() {
        let velocity = DriftVelocity {
            period: "7d".to_string(),
            node_count_delta: 3,
            edge_count_delta: 5,
            violation_count_delta: 2,
            complexity_delta: 0.0,
            trend: TrendDirection::Degrading,
            snapshot_count: 5,
        };

        let output = format_velocity_text(&velocity);
        assert!(output.contains("Velocity (7d)"));
        assert!(output.contains("Nodes: +3"));
        assert!(output.contains("Edges: +5"));
        assert!(output.contains("Violations: +2"));
        assert!(output.contains("Degrading"));
    }
}
