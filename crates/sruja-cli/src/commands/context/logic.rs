use super::super::federation::{find_system_index, infer_repo_id, load_system_index};
use super::types::*;
use crate::commands::CliError;
use sruja_scan::graph::ComponentImportance;
use sruja_scan::{Graph, NodeKind};
use std::collections::HashMap;
use std::path::Path;

pub fn build_architecture_context(
    graph: &Graph,
    repo: &str,
    file: Option<&str>,
    intent: Option<&str>,
    depth: usize,
    max_tokens: usize,
) -> Result<ArchitectureContext, CliError> {
    let modules = count_kind(graph, NodeKind::Module);
    let services = count_kind(graph, NodeKind::Service);
    let databases = count_kind(graph, NodeKind::Database);
    let external_apis = count_kind(graph, NodeKind::ExternalApi);

    let layers = infer_layers(graph);

    let max_boundary_rules = if max_tokens < 3000 { 5 } else { 30 };
    let mut boundaries = infer_boundaries(graph);
    boundaries.truncate(max_boundary_rules);

    let forbidden_patterns = vec![
        "Avoid direct database access from routes/handlers - use a service layer".to_string(),
        "Do not import internal modules from other services directly".to_string(),
        "UI components should not directly call database layers".to_string(),
    ];

    let centrality = sruja_scan::graph::compute_all_centrality(graph);

    let focus = file
        .map(|f| build_focus_context(graph, repo, f, intent, depth, max_tokens, &centrality))
        .transpose()?;

    let system_context = build_system_context(repo);

    Ok(ArchitectureContext {
        repo: repo.to_string(),
        summary: ContextSummary {
            total_modules: modules,
            total_services: services,
            total_databases: databases,
            total_external_apis: external_apis,
        },
        layers,
        boundaries,
        forbidden_patterns,
        focus,
        system_context,
        max_tokens,
    })
}

pub fn count_kind(graph: &Graph, kind: NodeKind) -> usize {
    graph.nodes.iter().filter(|n| n.kind == kind).count()
}

pub fn build_system_context(repo_root: &str) -> Option<SystemContext> {
    let root = Path::new(repo_root);
    let index_path = find_system_index(root)?;
    let index = load_system_index(&index_path).ok()?;
    let current_repo_id = infer_repo_id(root);

    let mut cross_repo_elements = Vec::new();
    let mut cross_repo_edges = Vec::new();
    let mut conflicts = Vec::new();

    let current_repo_prefix = format!("{}::", current_repo_id);
    let mut relevant_canonical_ids = std::collections::HashSet::new();

    for edge in &index.edges {
        let is_source_current = edge.source.starts_with(&current_repo_prefix);
        let is_target_current = edge.target.starts_with(&current_repo_prefix);

        if is_source_current || is_target_current {
            cross_repo_edges.push(CrossRepoEdge {
                source: edge.source.clone(),
                target: edge.target.clone(),
                label: edge.label.clone(),
                repo_id: edge.repo_id.clone(),
            });

            if !is_source_current {
                relevant_canonical_ids.insert(edge.source.as_str());
            }
            if !is_target_current {
                relevant_canonical_ids.insert(edge.target.as_str());
            }
        }
    }

    for node in &index.nodes {
        if relevant_canonical_ids.contains(node.canonical_id.as_str()) {
            cross_repo_elements.push(CrossRepoElement {
                canonical_id: node.canonical_id.clone(),
                kind: node.kind.clone(),
                label: node.label.clone(),
                technology: node.technology.clone(),
                repo_id: node.repo_id.clone(),
                owner: node.owner.clone(),
                domain: node.domain.clone(),
                criticality: node.criticality,
            });
        }
    }

    for conflict in &index.conflicts {
        if conflict.repos.contains(&current_repo_id) {
            conflicts.push(format!("{}: {}", conflict.key, conflict.message));
        }
    }

    Some(SystemContext {
        index_path: index_path.to_string_lossy().to_string(),
        total_repos: index.repos.len(),
        total_nodes: index.nodes.len(),
        total_edges: index.edges.len(),
        total_conflicts: index.conflicts.len(),
        cross_repo_elements,
        cross_repo_edges,
        conflicts,
    })
}

pub fn build_focus_context(
    graph: &Graph,
    repo_root: &str,
    file: &str,
    intent: Option<&str>,
    depth: usize,
    _max_tokens: usize,
    centrality: &HashMap<String, ComponentImportance>,
) -> Result<FocusContext, CliError> {
    let repo_path = Path::new(repo_root);
    let repo_canon = repo_path
        .canonicalize()
        .unwrap_or_else(|_| repo_path.to_path_buf());

    let requested_path = Path::new(file);
    let absolute = if requested_path.is_absolute() {
        requested_path.to_path_buf()
    } else {
        repo_path.join(requested_path)
    };

    let absolute_canon = absolute.canonicalize().unwrap_or(absolute.clone());
    let rel = absolute_canon
        .strip_prefix(&repo_canon)
        .ok()
        .map(|p| p.to_string_lossy().to_string());

    let absolute_str = absolute.to_string_lossy().to_string();
    let absolute_canon_str = absolute_canon.to_string_lossy().to_string();
    let mut candidates: Vec<String> = vec![absolute_str, absolute_canon_str];
    if let Some(ref r) = rel {
        candidates.push(r.clone());
    }
    for c in &mut candidates {
        *c = normalize_path(c);
    }
    candidates.sort();
    candidates.dedup();

    let mut matched: Vec<&sruja_scan::Node> = graph
        .nodes
        .iter()
        .filter(|n| {
            n.path
                .as_ref()
                .is_some_and(|p| path_matches_any(p, &candidates))
        })
        .collect();

    matched.sort_by(|a, b| {
        let a_score = score_path_match(a.path.as_deref(), &candidates);
        let b_score = score_path_match(b.path.as_deref(), &candidates);

        let a_centrality = centrality.get(&a.id).map(|s| s.pagerank).unwrap_or(0.0);
        let b_centrality = centrality.get(&b.id).map(|s| s.pagerank).unwrap_or(0.0);

        a_score
            .cmp(&b_score)
            .reverse()
            .then_with(|| {
                b_centrality
                    .partial_cmp(&a_centrality)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.id.cmp(&b.id))
    });

    let matched_nodes: Vec<FocusNode> = matched
        .iter()
        .take(10)
        .map(|n| FocusNode {
            id: n.id.clone(),
            kind: n.kind,
            label: n.label.clone(),
            path: n.path.clone(),
            owner: n.owner.clone(),
            domain: n.domain.clone(),
            criticality: n.criticality,
        })
        .collect();

    let blast_target = matched
        .iter()
        .find(|n| !n.id.contains('#'))
        .or_else(|| matched.first())
        .map(|n| n.id.as_str());

    let mut blast_radius = blast_target
        .filter(|_| depth > 0)
        .map(|id| graph.blast_radius(id, depth));

    if let Some(ref mut br) = blast_radius {
        let sort_blast = |nodes: &mut Vec<sruja_scan::BlastRadiusNode>| {
            nodes.sort_by(|a, b| {
                let a_c = centrality.get(&a.id).map(|s| s.pagerank).unwrap_or(0.0);
                let b_c = centrality.get(&b.id).map(|s| s.pagerank).unwrap_or(0.0);
                a.depth
                    .cmp(&b.depth)
                    .then_with(|| b_c.partial_cmp(&a_c).unwrap_or(std::cmp::Ordering::Equal))
                    .then_with(|| a.id.cmp(&b.id))
            });
            nodes.truncate(20);
        };
        sort_blast(&mut br.upstream);
        sort_blast(&mut br.downstream);
    }

    let suggested_checks = suggested_checks(intent);

    Ok(FocusContext {
        file: file.to_string(),
        intent: intent.map(|s| s.to_string()),
        depth,
        matched_nodes,
        blast_radius,
        suggested_checks,
    })
}

pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

pub fn path_matches_any(node_path: &str, candidates: &[String]) -> bool {
    let node_norm = normalize_path(node_path);
    if candidates.contains(&node_norm) {
        return true;
    }
    candidates.iter().any(|c| node_norm.ends_with(c))
}

pub fn score_path_match(node_path: Option<&str>, candidates: &[String]) -> usize {
    let Some(p) = node_path else {
        return 0;
    };
    let p_norm = normalize_path(p);
    if candidates.contains(&p_norm) {
        return 3;
    }
    if candidates.iter().any(|c| p_norm.ends_with(c)) {
        return 2;
    }
    1
}

pub fn suggested_checks(intent: Option<&str>) -> Vec<String> {
    let mut checks: Vec<String> = vec![
        "cargo fmt --all".to_string(),
        "cargo clippy -- -D warnings".to_string(),
        "cargo test --workspace".to_string(),
        "sruja drift -r .".to_string(),
    ];

    match intent {
        Some("add-test") => {
            checks.insert(0, "cargo test -p <crate> <test_name>".to_string());
        }
        Some("fix-bug") | Some("refactor") | Some("add-feature") => {
            checks.insert(0, "cargo test --workspace".to_string());
        }
        _ => {}
    }

    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    checks
        .into_iter()
        .filter(|c| seen.insert(c.clone()))
        .collect()
}

pub fn infer_layers(graph: &Graph) -> Vec<LayerInfo> {
    let mut layer_counts: HashMap<String, usize> = HashMap::new();

    for node in &graph.nodes {
        if node.kind == NodeKind::Module {
            if let Some(path) = &node.path {
                let layer = infer_layer_from_path(path);
                *layer_counts.entry(layer).or_default() += 1;
            }
        }
    }

    let mut layers: Vec<LayerInfo> = layer_counts
        .into_iter()
        .map(|(name, count): (String, usize)| {
            let can_depend_on = match name.as_str() {
                "api" | "routes" | "handlers" => vec!["services".to_string()],
                "services" => vec!["data".to_string(), "models".to_string()],
                "data" | "db" | "repository" => vec!["models".to_string()],
                "models" | "entities" => vec![],
                "utils" | "lib" | "common" => vec![],
                _ => vec![],
            };
            LayerInfo {
                name,
                modules: count,
                can_depend_on,
            }
        })
        .collect();
    layers.sort_by(|a, b| a.name.cmp(&b.name));
    layers
}

pub fn infer_layer_from_path(path: &str) -> String {
    let path_lower = path.to_lowercase();
    let parts: Vec<&str> = path_lower.split('/').collect();

    for part in &parts {
        match *part {
            "api" | "routes" | "handlers" | "controllers" | "endpoints" => {
                return "api".to_string()
            }
            "services" | "service" => return "services".to_string(),
            "data" | "db" | "database" | "repository" | "repos" | "dal" => {
                return "data".to_string()
            }
            "models" | "model" | "entities" | "entity" | "domain" => return "models".to_string(),
            "utils" | "lib" | "common" | "shared" | "helpers" => return "utils".to_string(),
            "components" | "ui" | "views" | "pages" => return "ui".to_string(),
            _ => {}
        }
    }

    "other".to_string()
}

pub fn infer_boundaries(graph: &Graph) -> Vec<BoundaryRule> {
    let mut boundaries = Vec::new();

    let mut services: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Service)
        .collect();
    services.sort_by(|a, b| a.id.cmp(&b.id));

    if services.len() > 1 {
        for s1 in &services {
            for s2 in &services {
                if s1.id != s2.id {
                    boundaries.push(BoundaryRule {
                        from: s1.id.clone(),
                        to: s2.id.clone(),
                        allowed: false,
                        reason: "Services should communicate via APIs/events, not direct imports"
                            .to_string(),
                    });
                }
            }
        }
    }

    boundaries.push(BoundaryRule {
        from: "ui".to_string(),
        to: "data".to_string(),
        allowed: false,
        reason: "UI should not directly access data layer - use services".to_string(),
    });

    boundaries
}
