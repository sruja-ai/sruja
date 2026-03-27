use crate::graph::{Edge, Graph, NodeKind};

pub struct ConfidenceScorer;

impl ConfidenceScorer {
    pub fn score_graph(graph: &mut Graph) {
        let node_count = graph.nodes.len();

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

    fn calculate_node_confidence(node: &crate::graph::Node, edges: &[Edge]) -> u8 {
        let mut score: i32 = 50; // Initial baseline

        // Rule 1: Technology & Kind alignment
        if let Some(tech) = &node.technology {
            if (tech == "Next.js" || tech == "Express")
                && node.kind != crate::graph::NodeKind::Module
            {
                score += 20;
            }
        }

        // Rule 2: Connectivity
        let incoming_count = edges.iter().filter(|e| e.target == node.id).count();
        let outgoing_count = edges.iter().filter(|e| e.source == node.id).count();

        if (incoming_count > 0 && outgoing_count > 0) || node.kind == NodeKind::Service {
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
        if incoming_count == 0 && outgoing_count == 0 && node.kind == NodeKind::Module {
            score -= 30;
        }

        score.clamp(0, 100) as u8
    }
}
