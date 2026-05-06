use super::{Graph, Node};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityInfo {
    pub id: u32,
    pub members: Vec<String>,
    pub member_count: usize,
    pub internal_edges: usize,
    pub external_edges: usize,
    pub cohesion: f64,
    pub suggested_label: String,
}

/// Detect communities using Label Propagation Algorithm.
/// Returns node_id → community_id (1-indexed) mapping.
pub fn detect_communities(graph: &Graph) -> HashMap<String, u32> {
    let mut node_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
    if node_ids.is_empty() {
        return HashMap::new();
    }
    node_ids.sort();

    // Initialize each node with its own unique label
    let mut labels: HashMap<String, u32> = node_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.clone(), i as u32))
        .collect();

    // Adjacency list for undirected propagation
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for id in &node_ids {
        adj.insert(id.clone(), Vec::new());
    }
    for edge in &graph.edges {
        if edge.source != edge.target
            && labels.contains_key(&edge.source)
            && labels.contains_key(&edge.target)
        {
            adj.get_mut(&edge.source).unwrap().push(edge.target.clone());
            adj.get_mut(&edge.target).unwrap().push(edge.source.clone());
        }
    }

    // Propagate labels
    for _ in 0..30 {
        let mut changed = false;
        for node_id in &node_ids {
            let neighbors = &adj[node_id];
            if neighbors.is_empty() {
                continue;
            }

            let mut counts: HashMap<u32, usize> = HashMap::new();
            for neighbor in neighbors {
                let label = labels[neighbor];
                *counts.entry(label).or_default() += 1;
            }

            let mut max_count = 0;
            let mut best_label = labels[node_id];
            for (label, count) in counts {
                if count > max_count || (count == max_count && label < best_label) {
                    max_count = count;
                    best_label = label;
                }
            }

            if labels[node_id] != best_label {
                labels.insert(node_id.clone(), best_label);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    // Group by label to normalize and sort deterministically
    let mut communities_map: HashMap<u32, Vec<String>> = HashMap::new();
    for (id, label) in labels {
        communities_map.entry(label).or_default().push(id);
    }
    let mut communities_list: Vec<(u32, Vec<String>)> = communities_map.into_iter().collect();
    communities_list.sort_by(|a, b| {
        b.1.len().cmp(&a.1.len()).then_with(|| {
            let mut a_sorted = a.1.clone();
            a_sorted.sort();
            let mut b_sorted = b.1.clone();
            b_sorted.sort();
            a_sorted[0].cmp(&b_sorted[0])
        })
    });

    let mut normalized_labels: HashMap<String, u32> = HashMap::new();
    for (new_id, (_, members)) in communities_list.iter().enumerate() {
        for member in members {
            normalized_labels.insert(member.clone(), (new_id + 1) as u32);
        }
    }

    normalized_labels
}

/// Build community summaries from a community assignment.
pub fn summarize_communities(
    graph: &Graph,
    communities: &HashMap<String, u32>,
) -> Vec<CommunityInfo> {
    if communities.is_empty() {
        return Vec::new();
    }

    let node_map: HashMap<&str, &Node> = graph.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    let mut grouped_members: HashMap<u32, Vec<String>> = HashMap::new();
    for (id, &label) in communities {
        grouped_members.entry(label).or_default().push(id.clone());
    }

    let mut summaries = Vec::new();

    for (id, mut members) in grouped_members {
        members.sort();

        let mut internal_edges = 0;
        let mut external_edges = 0;
        let member_set: HashSet<&String> = members.iter().collect();

        for edge in &graph.edges {
            let source_in = member_set.contains(&edge.source);
            let target_in = member_set.contains(&edge.target);
            if source_in && target_in {
                internal_edges += 1;
            } else if source_in || target_in {
                external_edges += 1;
            }
        }

        let cohesion = if internal_edges + external_edges > 0 {
            internal_edges as f64 / (internal_edges + external_edges) as f64
        } else {
            1.0
        };

        // Determine suggested label
        let suggested_label = determine_suggested_label(&members, &node_map);

        summaries.push(CommunityInfo {
            id,
            member_count: members.len(),
            members,
            internal_edges,
            external_edges,
            cohesion,
            suggested_label,
        });
    }

    // Sort communities by ID ascending
    summaries.sort_by_key(|c| c.id);
    summaries
}

fn determine_suggested_label(members: &[String], node_map: &HashMap<&str, &Node>) -> String {
    // 1. Try to find longest common directory/path prefix
    let paths: Vec<&str> = members
        .iter()
        .filter_map(|id| node_map.get(id.as_str()).and_then(|n| n.path.as_deref()))
        .collect();

    if !paths.is_empty() {
        if let Some(prefix) = longest_common_prefix(&paths, '/') {
            if !prefix.is_empty() {
                return prefix;
            }
        }
    }

    // 2. Try to find longest common FQN dot-prefix
    let fqns: Vec<&str> = members.iter().map(|s| s.as_str()).collect();
    if let Some(prefix) = longest_common_prefix(&fqns, '.') {
        if !prefix.is_empty() {
            return prefix;
        }
    }

    // 3. Fallback to most common NodeKind
    let mut kind_counts: HashMap<String, usize> = HashMap::new();
    for id in members {
        if let Some(node) = node_map.get(id.as_str()) {
            *kind_counts
                .entry(node.kind.as_str().to_string())
                .or_default() += 1;
        }
    }
    let mut counts_vec: Vec<(String, usize)> = kind_counts.into_iter().collect();
    counts_vec.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    if let Some((kind, _)) = counts_vec.first() {
        format!("community_{kind}")
    } else {
        "module_group".to_string()
    }
}

fn longest_common_prefix(items: &[&str], separator: char) -> Option<String> {
    if items.is_empty() {
        return None;
    }
    let split_items: Vec<Vec<&str>> = items.iter().map(|s| s.split(separator).collect()).collect();
    let first = &split_items[0];
    let mut common = Vec::new();

    for (i, &part) in first.iter().enumerate() {
        let mut match_all = true;
        for other in &split_items[1..] {
            if i >= other.len() || other[i] != part {
                match_all = false;
                break;
            }
        }
        if match_all {
            common.push(part);
        } else {
            break;
        }
    }

    if common.is_empty() {
        None
    } else {
        Some(common.join(&separator.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::EdgeConfidence;
    use crate::{Edge, EdgeKind, Graph, Node, NodeKind};

    #[test]
    fn test_disconnected_triangles() {
        // Two disconnected triangles: (A, B, C) and (D, E, F)
        let nodes = vec![
            Node {
                id: "A".to_string(),
                kind: NodeKind::Module,
                label: "A".to_string(),
                path: Some("crates/module_a/A.rs".to_string()),
                ..Default::default()
            },
            Node {
                id: "B".to_string(),
                kind: NodeKind::Module,
                label: "B".to_string(),
                path: Some("crates/module_a/B.rs".to_string()),
                ..Default::default()
            },
            Node {
                id: "C".to_string(),
                kind: NodeKind::Module,
                label: "C".to_string(),
                path: Some("crates/module_a/C.rs".to_string()),
                ..Default::default()
            },
            Node {
                id: "D".to_string(),
                kind: NodeKind::Module,
                label: "D".to_string(),
                path: Some("crates/module_b/D.rs".to_string()),
                ..Default::default()
            },
            Node {
                id: "E".to_string(),
                kind: NodeKind::Module,
                label: "E".to_string(),
                path: Some("crates/module_b/E.rs".to_string()),
                ..Default::default()
            },
            Node {
                id: "F".to_string(),
                kind: NodeKind::Module,
                label: "F".to_string(),
                path: Some("crates/module_b/F.rs".to_string()),
                ..Default::default()
            },
        ];

        let edges = vec![
            Edge {
                source: "A".to_string(),
                target: "B".to_string(),
                kind: EdgeKind::Calls,
                evidence: Vec::new(),
                confidence: EdgeConfidence::Extracted,
            },
            Edge {
                source: "B".to_string(),
                target: "C".to_string(),
                kind: EdgeKind::Calls,
                evidence: Vec::new(),
                confidence: EdgeConfidence::Extracted,
            },
            Edge {
                source: "C".to_string(),
                target: "A".to_string(),
                kind: EdgeKind::Calls,
                evidence: Vec::new(),
                confidence: EdgeConfidence::Extracted,
            },
            Edge {
                source: "D".to_string(),
                target: "E".to_string(),
                kind: EdgeKind::Calls,
                evidence: Vec::new(),
                confidence: EdgeConfidence::Extracted,
            },
            Edge {
                source: "E".to_string(),
                target: "F".to_string(),
                kind: EdgeKind::Calls,
                evidence: Vec::new(),
                confidence: EdgeConfidence::Extracted,
            },
            Edge {
                source: "F".to_string(),
                target: "D".to_string(),
                kind: EdgeKind::Calls,
                evidence: Vec::new(),
                confidence: EdgeConfidence::Extracted,
            },
        ];

        let graph = Graph {
            nodes,
            edges,
            ..Default::default()
        };
        let communities = detect_communities(&graph);

        // A, B, C must have the same label
        assert_eq!(communities["A"], communities["B"]);
        assert_eq!(communities["B"], communities["C"]);

        // D, E, F must have the same label
        assert_eq!(communities["D"], communities["E"]);
        assert_eq!(communities["E"], communities["F"]);

        // They must be different communities
        assert_ne!(communities["A"], communities["D"]);

        // Verify summarize_communities works
        let summaries = summarize_communities(&graph, &communities);
        assert_eq!(summaries.len(), 2);

        assert_eq!(summaries[0].suggested_label, "crates/module_a");
        assert_eq!(summaries[1].suggested_label, "crates/module_b");
    }

    #[test]
    fn test_longest_common_prefix() {
        let items = vec![
            "crates/sruja-scan/src/lib.rs",
            "crates/sruja-scan/src/graph/mod.rs",
        ];
        assert_eq!(
            longest_common_prefix(&items, '/'),
            Some("crates/sruja-scan/src".to_string())
        );

        let dots = vec!["Sruja.Scan.Graph.Node", "Sruja.Scan.Parser"];
        assert_eq!(
            longest_common_prefix(&dots, '.'),
            Some("Sruja.Scan".to_string())
        );
    }
}
