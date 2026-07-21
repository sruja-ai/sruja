use super::super::types::*;
use crate::commands::CliError;
use sruja_scan::graph::ComponentImportance;
use sruja_scan::Graph;
use std::collections::HashMap;
use std::path::Path;

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
            kind: n.kind.clone(),
            label: n.label.clone(),
            path: n.path.clone(),
            owner: n.owner.clone(),
            domain: n.domain.clone(),
            criticality: n.criticality,
            gotchas: n.gotchas.clone(),
            operational_constraints: n.operational_constraints.clone(),
            runbooks: n.runbooks.clone(),
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
