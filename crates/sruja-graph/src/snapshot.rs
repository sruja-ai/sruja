//! Graph Snapshot and Delta Tracking
//!
//! Provides delta-based change tracking for the knowledge graph,
//! enabling temporal queries and drift velocity computation.

use crate::{Decision, GraphNode, KnowledgeGraph};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSnapshot {
    pub timestamp: DateTime<Utc>,
    pub commit_sha: String,
    pub deltas: Vec<GraphDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphDelta {
    NodeAdded {
        node_id: String,
        kind: String,
        label: String,
    },
    NodeRemoved {
        node_id: String,
    },
    NodeChanged {
        node_id: String,
        field: String,
        old: String,
        new: String,
    },
    EdgeAdded {
        source: String,
        target: String,
        kind: String,
    },
    EdgeRemoved {
        source: String,
        target: String,
        kind: String,
    },
    DecisionAdded {
        decision_id: String,
        title: String,
    },
    DecisionStatusChanged {
        decision_id: String,
        old: String,
        new: String,
    },
    LearningAdded {
        learning_id: String,
        affected_elements: Vec<String>,
    },
    LearningChanged {
        learning_id: String,
        field: String,
        old: String,
        new: String,
    },
    LearningRemoved {
        learning_id: String,
    },
}

impl GraphDelta {
    pub fn references_element(&self, element: &str) -> bool {
        match self {
            GraphDelta::NodeAdded { node_id, .. } => node_id == element,
            GraphDelta::NodeRemoved { node_id } => node_id == element,
            GraphDelta::NodeChanged { node_id, .. } => node_id == element,
            GraphDelta::EdgeAdded { source, target, .. } => {
                source == element || target == element
            }
            GraphDelta::EdgeRemoved { source, target, .. } => {
                source == element || target == element
            }
            GraphDelta::DecisionAdded { decision_id, .. } => decision_id == element,
            GraphDelta::DecisionStatusChanged { decision_id, .. } => decision_id == element,
            GraphDelta::LearningAdded { learning_id, .. } => learning_id == element,
            GraphDelta::LearningChanged { learning_id, .. } => learning_id == element,
            GraphDelta::LearningRemoved { learning_id } => learning_id == element,
        }
    }

    pub fn kind_str(&self) -> &str {
        match self {
            GraphDelta::NodeAdded { .. } => "node_added",
            GraphDelta::NodeRemoved { .. } => "node_removed",
            GraphDelta::NodeChanged { .. } => "node_changed",
            GraphDelta::EdgeAdded { .. } => "edge_added",
            GraphDelta::EdgeRemoved { .. } => "edge_removed",
            GraphDelta::DecisionAdded { .. } => "decision_added",
            GraphDelta::DecisionStatusChanged { .. } => "decision_status_changed",
            GraphDelta::LearningAdded { .. } => "learning_added",
            GraphDelta::LearningChanged { .. } => "learning_changed",
            GraphDelta::LearningRemoved { .. } => "learning_removed",
        }
    }
}

/// Compute deltas between old and new knowledge graphs
pub fn compute_deltas(old: &KnowledgeGraph, new: &KnowledgeGraph) -> Vec<GraphDelta> {
    let mut deltas = Vec::new();

    // Nodes added (in new, not in old)
    for (id, node) in &new.nodes {
        if !old.nodes.contains_key(id) {
            deltas.push(GraphDelta::NodeAdded {
                node_id: id.clone(),
                kind: node.kind.kind_str().to_string(),
                label: node.label.clone(),
            });
        }
    }

    // Nodes removed (in old, not in new)
    for id in old.nodes.keys() {
        if !new.nodes.contains_key(id) {
            deltas.push(GraphDelta::NodeRemoved { node_id: id.clone() });
        }
    }

    // Nodes changed (field-level diff)
    for (id, old_node) in &old.nodes {
        if let Some(new_node) = new.nodes.get(id) {
            compare_nodes(id, old_node, new_node, &mut deltas);
        }
    }

    // Edges added/removed (compare by source+target+kind tuple)
    let old_edges: HashSet<_> = old
        .edges
        .iter()
        .map(|e| (&e.source, &e.target, e.kind.kind_str()))
        .collect();
    let new_edges: HashSet<_> = new
        .edges
        .iter()
        .map(|e| (&e.source, &e.target, e.kind.kind_str()))
        .collect();

    for edge in &new.edges {
        let key = (&edge.source, &edge.target, edge.kind.kind_str());
        if !old_edges.contains(&key) {
            deltas.push(GraphDelta::EdgeAdded {
                source: edge.source.clone(),
                target: edge.target.clone(),
                kind: edge.kind.kind_str().to_string(),
            });
        }
    }

    for edge in &old.edges {
        let key = (&edge.source, &edge.target, edge.kind.kind_str());
        if !new_edges.contains(&key) {
            deltas.push(GraphDelta::EdgeRemoved {
                source: edge.source.clone(),
                target: edge.target.clone(),
                kind: edge.kind.kind_str().to_string(),
            });
        }
    }

    // Decisions added/changed
    for (id, old_decision) in &old.decisions {
        if let Some(new_decision) = new.decisions.get(id) {
            compare_decisions(id, old_decision, new_decision, &mut deltas);
        } else {
            // Decision removed - not tracked explicitly as we focus on additions/changes
        }
    }

    for (id, new_decision) in &new.decisions {
        if !old.decisions.contains_key(id) {
            deltas.push(GraphDelta::DecisionAdded {
                decision_id: id.clone(),
                title: new_decision.title.clone(),
            });
        }
    }

    // Learnings added/changed/removed
    for (id, new_learning) in &new.learnings {
        if let Some(old_learning) = old.learnings.get(id) {
            // Compare learning fields
            if old_learning.outcome != new_learning.outcome {
                deltas.push(GraphDelta::LearningChanged {
                    learning_id: id.clone(),
                    field: "outcome".to_string(),
                    old: old_learning.outcome.clone(),
                    new: new_learning.outcome.clone(),
                });
            }
            if old_learning.guardrail_advice != new_learning.guardrail_advice {
                deltas.push(GraphDelta::LearningChanged {
                    learning_id: id.clone(),
                    field: "guardrail_advice".to_string(),
                    old: old_learning.guardrail_advice.clone(),
                    new: new_learning.guardrail_advice.clone(),
                });
            }
            if old_learning.confidence != new_learning.confidence {
                deltas.push(GraphDelta::LearningChanged {
                    learning_id: id.clone(),
                    field: "confidence".to_string(),
                    old: old_learning.confidence.clone().unwrap_or_default(),
                    new: new_learning.confidence.clone().unwrap_or_default(),
                });
            }
        } else {
            deltas.push(GraphDelta::LearningAdded {
                learning_id: id.clone(),
                affected_elements: new_learning.affected_elements.clone(),
            });
        }
    }

    // Learnings removed (in old, not in new)
    for id in old.learnings.keys() {
        if !new.learnings.contains_key(id) {
            deltas.push(GraphDelta::LearningRemoved {
                learning_id: id.clone(),
            });
        }
    }

    deltas
}

fn compare_nodes(id: &str, old: &GraphNode, new: &GraphNode, deltas: &mut Vec<GraphDelta>) {
    if old.label != new.label {
        deltas.push(GraphDelta::NodeChanged {
            node_id: id.to_string(),
            field: "label".to_string(),
            old: old.label.clone(),
            new: new.label.clone(),
        });
    }

    if old.description != new.description {
        deltas.push(GraphDelta::NodeChanged {
            node_id: id.to_string(),
            field: "description".to_string(),
            old: old.description.clone().unwrap_or_default(),
            new: new.description.clone().unwrap_or_default(),
        });
    }

    if old.technology() != new.technology() {
        deltas.push(GraphDelta::NodeChanged {
            node_id: id.to_string(),
            field: "technology".to_string(),
            old: old.technology().unwrap_or("").to_string(),
            new: new.technology().unwrap_or("").to_string(),
        });
    }
}

fn compare_decisions(
    id: &str,
    old: &Decision,
    new: &Decision,
    deltas: &mut Vec<GraphDelta>,
) {
    if old.status != new.status {
        deltas.push(GraphDelta::DecisionStatusChanged {
            decision_id: id.to_string(),
            old: format!("{:?}", old.status),
            new: format!("{:?}", new.status),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GraphNode, NodeKind, SourceReference};

    fn create_test_node(id: &str, label: &str) -> GraphNode {
        GraphNode::new(
            id.to_string(),
            NodeKind::new(NodeKind::SERVICE),
            label.to_string(),
            None,
            std::collections::HashMap::new(),
            SourceReference::manual(),
            Utc::now(),
            Utc::now(),
        )
    }

    #[test]
    fn test_compute_deltas_node_added() {
        let old = KnowledgeGraph::new();
        let mut new = KnowledgeGraph::new();
        new.add_node(create_test_node("svc1", "Service 1")).unwrap();

        let deltas = compute_deltas(&old, &new);
        assert_eq!(deltas.len(), 1);
        assert!(matches!(deltas[0], GraphDelta::NodeAdded { ref node_id, .. } if node_id == "svc1"));
    }

    #[test]
    fn test_compute_deltas_node_removed() {
        let mut old = KnowledgeGraph::new();
        old.add_node(create_test_node("svc1", "Service 1")).unwrap();
        let new = KnowledgeGraph::new();

        let deltas = compute_deltas(&old, &new);
        assert_eq!(deltas.len(), 1);
        assert!(matches!(deltas[0], GraphDelta::NodeRemoved { ref node_id } if node_id == "svc1"));
    }

    #[test]
    fn test_compute_deltas_node_changed() {
        let mut old = KnowledgeGraph::new();
        old.add_node(create_test_node("svc1", "Old Label")).unwrap();

        let mut new = KnowledgeGraph::new();
        new.add_node(create_test_node("svc1", "New Label")).unwrap();

        let deltas = compute_deltas(&old, &new);
        assert_eq!(deltas.len(), 1);
        assert!(matches!(deltas[0], GraphDelta::NodeChanged { ref field, .. } if field == "label"));
    }

    #[test]
    fn test_compute_deltas_no_changes() {
        let mut old = KnowledgeGraph::new();
        old.add_node(create_test_node("svc1", "Service 1")).unwrap();

        let mut new = KnowledgeGraph::new();
        new.add_node(create_test_node("svc1", "Service 1")).unwrap();

        let deltas = compute_deltas(&old, &new);
        assert!(deltas.is_empty());
    }

    #[test]
    fn test_graph_delta_references_element() {
        let delta = GraphDelta::NodeAdded {
            node_id: "svc1".to_string(),
            kind: "service".to_string(),
            label: "Test".to_string(),
        };
        assert!(delta.references_element("svc1"));
        assert!(!delta.references_element("svc2"));
    }

    #[test]
    fn test_graph_delta_kind_str() {
        let delta = GraphDelta::EdgeAdded {
            source: "a".to_string(),
            target: "b".to_string(),
            kind: "calls".to_string(),
        };
        assert_eq!(delta.kind_str(), "edge_added");
    }
}
