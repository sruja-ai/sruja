use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use sruja_scan::graph::compute_all_centrality;
use sruja_scan::{EdgeKind, Graph, NodeKind};

use super::models::{
    kind_priority, relative_graph_path, DiscoverConfidence, DiscoverContextJson,
    DiscoverDirectorySummary, DiscoverElementSummary, DiscoverRelationshipSummary,
};

pub(crate) fn discover_top_directories(
    repo: &str,
    repo_path: &Path,
    graph: &Graph,
) -> Vec<DiscoverDirectorySummary> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for node in &graph.nodes {
        let Some(path) = node.path.as_deref() else {
            continue;
        };
        let relative = relative_graph_path(repo, repo_path, path);
        if !relative.contains('/') {
            continue;
        }
        let first = relative.split('/').next().unwrap_or("").trim();
        if !first.is_empty()
            && !first.starts_with('.')
            && first != "test-repos"
            && first != "evaluation"
        {
            *counts.entry(first.to_string()).or_default() += 1;
        }
    }

    let mut dirs: Vec<_> = counts
        .into_iter()
        .map(|(area, nodes)| DiscoverDirectorySummary { area, nodes })
        .collect();
    dirs.sort_by(|a, b| b.nodes.cmp(&a.nodes).then_with(|| a.area.cmp(&b.area)));
    dirs.truncate(5);
    dirs
}

pub(crate) fn discover_kind_counts(graph: &Graph) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for node in &graph.nodes {
        *counts.entry(node.kind.as_str().to_string()).or_insert(0) += 1;
    }
    counts
}

pub(crate) fn discover_key_elements(
    repo: &str,
    repo_path: &Path,
    graph: &Graph,
) -> Vec<DiscoverElementSummary> {
    use super::models::explain_node_relevance;

    let centrality = compute_all_centrality(graph);
    let mut incoming_counts: HashMap<&str, usize> = HashMap::new();
    let mut outgoing_counts: HashMap<&str, usize> = HashMap::new();

    for edge in &graph.edges {
        *incoming_counts.entry(edge.target.as_str()).or_default() += 1;
        *outgoing_counts.entry(edge.source.as_str()).or_default() += 1;
    }

    let mut nodes: Vec<_> = graph.nodes.iter().collect();
    let preferred_nodes: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| !node.id.contains('#'))
        .collect();
    if preferred_nodes.len() >= 3 {
        nodes = preferred_nodes;
    }
    nodes.sort_by(|a, b| {
        let a_rank = centrality
            .get(&a.id)
            .map(|s| s.pagerank)
            .unwrap_or_default();
        let b_rank = centrality
            .get(&b.id)
            .map(|s| s.pagerank)
            .unwrap_or_default();
        b_rank
            .total_cmp(&a_rank)
            .then_with(|| kind_priority(b.kind.clone()).cmp(&kind_priority(a.kind.clone())))
            .then_with(|| a.id.cmp(&b.id))
    });

    nodes.truncate(5);
    nodes
        .into_iter()
        .map(|node| {
            let incoming = incoming_counts.get(node.id.as_str()).copied().unwrap_or(0);
            let outgoing = outgoing_counts.get(node.id.as_str()).copied().unwrap_or(0);
            let pagerank = centrality
                .get(&node.id)
                .map(|s| s.pagerank)
                .unwrap_or_default();
            DiscoverElementSummary {
                id: node.id.clone(),
                label: node.label.clone(),
                kind: node.kind.as_str().to_string(),
                path: node
                    .path
                    .as_deref()
                    .map(|path| relative_graph_path(repo, repo_path, path)),
                incoming,
                outgoing,
                pagerank,
                why_it_matters: explain_node_relevance(node, incoming, outgoing),
            }
        })
        .collect()
}

pub(crate) fn discover_key_relationships(graph: &Graph) -> Vec<DiscoverRelationshipSummary> {
    use super::models::explain_edge_relevance;

    let centrality = compute_all_centrality(graph);
    let node_kind_by_id: HashMap<&str, NodeKind> = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node.kind.clone()))
        .collect();

    let mut edges: Vec<_> = graph
        .edges
        .iter()
        .filter(|edge| !matches!(edge.kind.as_str(), EdgeKind::CONTAINS | EdgeKind::OWNS))
        .collect();
    let preferred_edges: Vec<_> = edges
        .iter()
        .copied()
        .filter(|edge| !edge.source.contains('#') && !edge.target.contains('#'))
        .collect();
    if preferred_edges.len() >= 3 {
        edges = preferred_edges;
    }

    edges.sort_by(|a, b| {
        let a_score = centrality
            .get(&a.source)
            .map(|s| s.pagerank)
            .unwrap_or_default()
            + centrality
                .get(&a.target)
                .map(|s| s.pagerank)
                .unwrap_or_default();
        let b_score = centrality
            .get(&b.source)
            .map(|s| s.pagerank)
            .unwrap_or_default()
            + centrality
                .get(&b.target)
                .map(|s| s.pagerank)
                .unwrap_or_default();
        b_score
            .total_cmp(&a_score)
            .then_with(|| b.evidence.len().cmp(&a.evidence.len()))
            .then_with(|| a.source.cmp(&b.source))
            .then_with(|| a.target.cmp(&b.target))
            .then_with(|| match (a.kind.clone(), b.kind.clone()) {
                (ak, bk) if ak == bk => Ordering::Equal,
                (ak, bk) => ak.as_str().cmp(bk.as_str()),
            })
    });

    edges.truncate(5);
    edges
        .into_iter()
        .map(|edge| DiscoverRelationshipSummary {
            source: edge.source.clone(),
            target: edge.target.clone(),
            kind: edge.kind.as_str().to_string(),
            confidence: format!("{:?}", edge.confidence).to_lowercase(),
            why_it_matters: explain_edge_relevance(
                edge,
                node_kind_by_id.get(edge.target.as_str()).cloned(),
                edge.evidence.len(),
            ),
        })
        .collect()
}

pub(crate) fn discover_reasoning(
    context: &DiscoverContextJson,
    graph: &Graph,
    top_directories: &[DiscoverDirectorySummary],
) -> Vec<String> {
    let service_count = graph
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::SERVICE)
        .count();
    let exported_interfaces = graph
        .nodes
        .iter()
        .filter(|node| node.id.contains('#'))
        .count();

    let mut reasoning = Vec::new();
    let architecture_reason = if service_count == 0 {
        "No service nodes were detected, so Sruja is currently reading this repo mostly as a module-level graph inside one codebase."
            .to_string()
    } else {
        match context.architecture_style.as_str() {
            "microservices" => format!(
                "Detected {service_count} service node(s), so this reads as a multi-service codebase rather than a single deployable unit."
            ),
            "monolith" => format!(
                "Detected {service_count} service node(s), which points to a single deployable unit or a monolith with internal modules."
            ),
            _ => format!(
                "Detected {service_count} service node(s), so Sruja is keeping the architecture style flexible instead of forcing a monolith or microservices label."
            ),
        }
    };
    reasoning.push(architecture_reason);

    if let Some(framework) = &context.framework {
        reasoning.push(format!(
            "Framework markers suggest {framework}, which helps Sruja recognize likely entry points and boundaries."
        ));
    } else {
        reasoning.push(
            "No strong framework markers were detected, so Sruja is leaning more on file structure and dependency edges."
                .to_string(),
        );
    }

    if !top_directories.is_empty() {
        let areas = top_directories
            .iter()
            .take(3)
            .map(|dir| format!("`{}`", dir.area))
            .collect::<Vec<_>>()
            .join(", ");
        reasoning.push(format!(
            "Most discovered elements cluster under {areas}, which is where the scanner sees the clearest architectural seams."
        ));
    }

    if exported_interfaces > 0 {
        reasoning.push(format!(
            "Found {exported_interfaces} exported interface node(s), which gives the scan stable API surfaces to anchor on."
        ));
    } else {
        reasoning.push(
            "The scan found few exported interfaces, so the graph is driven mostly by file-level structure and imports."
                .to_string(),
        );
    }

    reasoning
}

pub(crate) fn discover_confidence(
    context: &DiscoverContextJson,
    graph: &Graph,
    top_directories: &[DiscoverDirectorySummary],
) -> DiscoverConfidence {
    let nodes_with_paths = graph
        .nodes
        .iter()
        .filter(|node| node.path.is_some())
        .count();
    let level = if graph.nodes.is_empty() {
        "AMBIGUOUS"
    } else if graph.edges.is_empty() || context.framework.is_none() || top_directories.is_empty() {
        "INFERRED"
    } else {
        "EXTRACTED"
    };

    let mut signals = vec![format!(
        "Static analysis produced {} node(s) and {} relationship(s).",
        graph.nodes.len(),
        graph.edges.len()
    )];
    signals.push(format!(
        "{} of those node(s) map back to concrete file paths.",
        nodes_with_paths
    ));
    if let Some(framework) = &context.framework {
        signals.push(format!("Framework detection matched {framework}."));
    }
    if !top_directories.is_empty() {
        signals.push(format!(
            "The scan found clear top-level hotspots in {}.",
            top_directories
                .iter()
                .take(3)
                .map(|dir| format!("`{}`", dir.area))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let mut blind_spots = vec![
        "This is static analysis, so runtime-only calls, reflection, and generated code can still be missing.".to_string(),
        "Ownership, domain labels, and external system names are strongest after a reviewed repo.sruja baseline exists.".to_string(),
    ];
    if context.framework.is_none() {
        blind_spots.push(
            "Because the framework is unclear, boundary naming may need extra human review before you commit a baseline."
                .to_string(),
        );
    }

    DiscoverConfidence {
        level: level.to_string(),
        signals,
        blind_spots,
    }
}

pub(crate) fn discover_next_steps(graph: &Graph) -> Vec<String> {
    if graph.nodes.is_empty() {
        return vec![
            "Verify you are at the repo root and that the repo uses a supported language before relying on this scan.".to_string(),
            "Run `sruja discover context -r . --format json` to inspect what Sruja can detect from manifests and paths.".to_string(),
            "If the repo should have been discovered, capture a minimal repro and open an issue so scanner coverage can improve.".to_string(),
        ];
    }

    vec![
        "Review the highlighted elements and rename or regroup them in `repo.sruja` if they do not match your team language.".to_string(),
        "Run `sruja quickstart -r . --generate-baseline` for a structural draft (repo.sruja.draft), then author reviewed intent in repo.sruja with the sruja-architecture skill.".to_string(),
        "After repo.sruja exists, run `sruja drift -r . -a repo.sruja` in CI to keep declared architecture aligned with code.".to_string(),
    ]
}

pub(crate) fn discover_surprising_connections(graph: &Graph) -> Vec<DiscoverRelationshipSummary> {
    let mut surprising = Vec::new();
    for edge in &graph.edges {
        let source_parts: Vec<&str> = edge.source.split('_').collect();
        let target_parts: Vec<&str> = edge.target.split('_').collect();

        if source_parts.get(1) != target_parts.get(1)
            && !edge.source.contains("module:")
            && !edge.target.contains("module:")
        {
            surprising.push(DiscoverRelationshipSummary {
                source: edge.source.clone(),
                target: edge.target.clone(),
                kind: edge.kind.as_str().to_string(),
                confidence: format!("{:?}", edge.confidence).to_lowercase(),
                why_it_matters: "Crosses module boundaries and has low evidence, potentially indicating a hidden architectural coupling.".to_string(),
            });
        }
    }
    surprising.truncate(3);
    surprising
}

pub(crate) fn discover_suggested_questions(
    god_nodes: &[DiscoverElementSummary],
    surprising: &[DiscoverRelationshipSummary],
) -> Vec<String> {
    let mut questions = Vec::new();
    for node in god_nodes.iter().take(2) {
        questions.push(format!(
            "Why is `{}` a central hub (incoming: {}, outgoing: {})? Should its responsibilities be split?",
            node.id, node.incoming, node.outgoing
        ));
    }
    for edge in surprising.iter().take(2) {
        questions.push(format!(
            "What is the nature of the direct dependency from `{}` to `{}`? Can this coupling be decoupled or abstracted?",
            edge.source, edge.target
        ));
    }
    questions.push("What boundaries should be introduced to isolate changing core domains from external adapters?".to_string());
    questions.push(
        "How does the data access pattern scale with the observed coupling to data stores?"
            .to_string(),
    );
    questions
}
