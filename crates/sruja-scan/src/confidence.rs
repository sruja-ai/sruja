use crate::graph::{Edge, EdgeConfidence, Graph, NodeKind};

pub struct ConfidenceScorer;

impl ConfidenceScorer {
    pub fn score_graph(graph: &mut Graph) {
        let node_count = graph.nodes.len();

        // Score edges
        for edge in &mut graph.edges {
            edge.confidence = Self::calculate_edge_confidence(edge);
        }

        if node_count == 0 {
            graph.confidence = Some(100);
            return;
        }

        let mut node_scores = Vec::with_capacity(node_count);
        for node in &graph.nodes {
            let score = Self::calculate_node_confidence(node, &graph.edges);
            node_scores.push(score);
        }

        let mut total_score: usize = 0;
        for (i, node) in graph.nodes.iter_mut().enumerate() {
            let score = node_scores[i];
            node.confidence = Some(score);
            total_score += score as usize;
        }

        let avg_score = (total_score / node_count) as u8;
        graph.confidence = Some(avg_score);
    }

    fn calculate_edge_confidence(edge: &Edge) -> EdgeConfidence {
        if edge.evidence.is_empty() {
            return EdgeConfidence::Ambiguous;
        }

        let mut has_extracted = false;
        let mut has_inferred = false;

        for ev in &edge.evidence {
            let rule = ev.rule.to_lowercase();
            if rule.contains("import")
                || rule.contains("use")
                || rule.contains("require")
                || rule.contains("dependency")
                || rule.contains("manifest")
                || rule.contains("ast")
            {
                has_extracted = true;
            } else if rule.contains("path")
                || rule.contains("pattern")
                || rule.contains("naming")
                || rule.contains("proximity")
                || rule.contains("structure")
            {
                has_inferred = true;
            }
        }

        if has_extracted {
            EdgeConfidence::Extracted
        } else if has_inferred {
            EdgeConfidence::Inferred
        } else {
            EdgeConfidence::Ambiguous
        }
    }

    fn calculate_node_confidence(node: &crate::graph::Node, edges: &[Edge]) -> u8 {
        let mut score: i32 = 50; // Initial baseline

        // Rule 1: Technology & Kind alignment
        if let Some(tech) = &node.technology {
            if (tech == "Next.js" || tech == "Express") && node.kind != NodeKind::MODULE {
                score += 20;
            }
        }

        // Rule 2: Connectivity
        let incoming_count = edges.iter().filter(|e| e.target == node.id).count();
        let outgoing_count = edges.iter().filter(|e| e.source == node.id).count();

        if (incoming_count > 0 && outgoing_count > 0) || node.kind == NodeKind::SERVICE {
            score += 15;
        }

        // Rule 3: Path clarity
        if let Some(path) = &node.path {
            let path_lower = path.to_lowercase();
            if path_lower.contains("/api/")
                || path_lower.contains("/src/")
                || path_lower.contains("/app/")
            {
                score += 10;
            }
        }

        // Penalty for orphans
        if incoming_count == 0 && outgoing_count == 0 && node.kind == NodeKind::MODULE {
            score -= 30;
        }

        score.clamp(0, 100) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Edge, EdgeEvidence, EdgeKind, Graph, Node, NodeKind};

    fn edge_with_rule(rule: &str) -> Edge {
        Edge {
            source: "a".to_string(),
            target: "b".to_string(),
            kind: EdgeKind::new(EdgeKind::CALLS),
            evidence: vec![EdgeEvidence {
                rule: rule.to_string(),
                file: None,
                line: None,
                detail: None,
            }],
            confidence: EdgeConfidence::default(),
        }
    }

    #[test]
    fn score_graph_empty_graph_is_fully_confident() {
        let mut graph = Graph::new();
        ConfidenceScorer::score_graph(&mut graph);
        assert_eq!(graph.confidence, Some(100));
    }

    #[test]
    fn score_graph_classifies_edge_evidence() {
        let mut graph = Graph::new();
        graph.edges.push(edge_with_rule("typescript_import"));
        graph.edges.push(edge_with_rule("path_proximity"));
        graph.edges.push(Edge {
            source: "x".to_string(),
            target: "y".to_string(),
            kind: EdgeKind::new(EdgeKind::CALLS),
            evidence: vec![],
            confidence: EdgeConfidence::default(),
        });

        ConfidenceScorer::score_graph(&mut graph);

        assert_eq!(graph.edges[0].confidence, EdgeConfidence::Extracted);
        assert_eq!(graph.edges[1].confidence, EdgeConfidence::Inferred);
        assert_eq!(graph.edges[2].confidence, EdgeConfidence::Ambiguous);
    }

    #[test]
    fn score_graph_boosts_connected_nodes_and_penalizes_orphan_modules() {
        let mut graph = Graph::new();
        graph.nodes.push(Node {
            id: "hub".to_string(),
            kind: NodeKind::new(NodeKind::MODULE),
            label: "hub".to_string(),
            path: Some("src/api/handler.rs".to_string()),
            technology: Some("Express".to_string()),
            ..Default::default()
        });
        graph.nodes.push(Node {
            id: "orphan".to_string(),
            kind: NodeKind::new(NodeKind::MODULE),
            label: "orphan".to_string(),
            ..Default::default()
        });
        graph.edges.push(Edge {
            source: "hub".to_string(),
            target: "orphan".to_string(),
            kind: EdgeKind::new(EdgeKind::CALLS),
            evidence: vec![EdgeEvidence {
                rule: "import".to_string(),
                file: None,
                line: None,
                detail: None,
            }],
            confidence: EdgeConfidence::default(),
        });

        ConfidenceScorer::score_graph(&mut graph);

        let hub = graph.nodes.iter().find(|n| n.id == "hub").expect("hub");
        let orphan = graph
            .nodes
            .iter()
            .find(|n| n.id == "orphan")
            .expect("orphan");
        assert!(hub.confidence.unwrap_or(0) > orphan.confidence.unwrap_or(0));
        assert!(graph.confidence.is_some());
    }
}
