use sruja_scan::{Graph, NodeKind};
use std::collections::HashSet;

pub struct LayerViolationInfo {
    pub source: String,
    pub target: String,
}

pub fn find_layer_violations_advanced(graph: &Graph) -> Vec<LayerViolationInfo> {
    let mut violations = Vec::new();

    let db_nodes: HashSet<&str> = graph
        .nodes
        .iter()
        .filter(|n| {
            n.kind == NodeKind::DATABASE || {
                let p = n.path.as_deref().unwrap_or("").to_lowercase();
                let label = n.label.to_lowercase();
                p.contains("/database/")
                    || p.contains("/db/")
                    || p.contains("/datastore/")
                    || label.contains("database")
                    || label.contains("db")
                    || label.contains("repository")
            }
        })
        .map(|n| n.id.as_str())
        .collect();

    let frontend_nodes: HashSet<&str> = graph
        .nodes
        .iter()
        .filter(|n| {
            n.kind == NodeKind::FRONTEND || {
                let p = n.path.as_deref().unwrap_or("").to_lowercase();
                let label = n.label.to_lowercase();
                label.contains("frontend")
                    || label.contains("ui")
                    || label.contains("web")
                    || p.contains("/frontend/")
                    || p.contains("/ui/")
                    || p.contains("/web/")
            }
        })
        .map(|n| n.id.as_str())
        .collect();

    let core_nodes: HashSet<&str> = graph
        .nodes
        .iter()
        .filter(|n| {
            let p = n.path.as_deref().unwrap_or("").to_lowercase();
            p.contains("/domain/")
                || p.contains("/core/")
                || p.contains("/entities/")
                || p.contains("/usecases/")
        })
        .map(|n| n.id.as_str())
        .collect();

    let infra_nodes: HashSet<&str> = graph
        .nodes
        .iter()
        .filter(|n| {
            let p = n.path.as_deref().unwrap_or("").to_lowercase();
            p.contains("/infrastructure/")
                || p.contains("/infra/")
                || p.contains("/adapters/")
                || p.contains("/api/")
                || p.contains("/controllers/")
        })
        .map(|n| n.id.as_str())
        .collect();

    for edge in &graph.edges {
        if frontend_nodes.contains(edge.source.as_str()) && db_nodes.contains(edge.target.as_str())
        {
            violations.push(LayerViolationInfo {
                source: edge.source.clone(),
                target: edge.target.clone(),
            });
        }

        if core_nodes.contains(edge.source.as_str()) && infra_nodes.contains(edge.target.as_str()) {
            violations.push(LayerViolationInfo {
                source: edge.source.clone(),
                target: edge.target.clone(),
            });
        }
    }

    violations
}
