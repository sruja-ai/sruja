use colored::Colorize;
use serde::Serialize;
use sruja_scan::{BlastRadiusNode, BlastRadiusResult, Graph, Node};
use std::collections::HashMap;
use std::path::Path;

use super::CliError;
use crate::utils::architecture_path;
use sruja_scan::graph::{compute_all_centrality, ComponentImportance};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NodeSummary {
    id: String,
    kind: sruja_scan::NodeKind,
    label: String,
    technology: Option<String>,
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    centrality: Option<ComponentImportance>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ImpactHit {
    depth: usize,
    node: NodeSummary,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ImpactOutput {
    target: NodeSummary,
    max_depth: usize,
    upstream: Vec<ImpactHit>,
    downstream: Vec<ImpactHit>,
}

fn resolve_target_id(graph: &Graph, repo_path: &Path, query: &str) -> Result<String, CliError> {
    if graph.nodes.iter().any(|n| n.id == query) {
        return Ok(query.to_string());
    }

    let q = query.to_lowercase();
    let mut matches: Vec<&Node> = graph
        .nodes
        .iter()
        .filter(|n| {
            n.id.to_lowercase().contains(&q)
                || n.label.to_lowercase().contains(&q)
                || n.path.as_deref().unwrap_or("").to_lowercase().contains(&q)
        })
        .collect();

    if matches.len() == 1 {
        return Ok(matches[0].id.clone());
    }

    if let Some(baseline) = architecture_path::resolve_architecture_path(repo_path) {
        if let Ok(content) = std::fs::read_to_string(&baseline) {
            let parser = sruja_language::Parser::new(baseline.to_string_lossy().as_ref());
            if let Ok(program) = parser.parse(&content) {
                let (elements, _) = sruja_language::traversal::collect_elements(&program);
                let q_lower = query.to_lowercase();
                for (fqn, elem) in &elements {
                    let label = elem
                        .assignment
                        .title
                        .as_deref()
                        .unwrap_or(&elem.assignment.name);
                    if fqn.to_lowercase() == q_lower
                        || fqn.to_lowercase().contains(&q_lower)
                        || label.to_lowercase().contains(&q_lower)
                    {
                        let label_lower = label.to_lowercase();
                        let fqn_short: Vec<&str> = fqn.split('.').collect();
                        let short = fqn_short.last().copied().unwrap_or(fqn.as_str());
                        let short_lower = short.to_lowercase();
                        let scan_match = graph.nodes.iter().find(|n| {
                            n.label.to_lowercase() == label_lower
                                || n.label.to_lowercase() == short_lower
                                || n.id.to_lowercase().contains(&short_lower)
                        });
                        if let Some(node) = scan_match {
                            return Ok(node.id.clone());
                        }
                    }
                }
            }
        }
    }

    matches.sort_by(|a, b| a.id.cmp(&b.id));

    match matches.len() {
        0 => Err(CliError::validation(format!(
            "No node found matching '{}'. Try an exact node id from `sruja scan ...` output, or a DSL element name from your .sruja baseline.",
            query
        ))),
        _ => {
            let preview = matches
                .iter()
                .take(10)
                .map(|n| format!("{} ({})", n.id, n.label))
                .collect::<Vec<_>>()
                .join(", ");
            Err(CliError::validation(format!(
                "Ambiguous node selector '{}'. Matches: {}",
                query, preview
            )))
        }
    }
}

fn compute_light_centrality(graph: &Graph) -> HashMap<String, ComponentImportance> {
    let n = graph.nodes.len();
    let mut idx: HashMap<&str, usize> = HashMap::new();
    for (i, node) in graph.nodes.iter().enumerate() {
        idx.insert(node.id.as_str(), i);
    }

    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for edge in &graph.edges {
        if let (Some(&src), Some(&tgt)) =
            (idx.get(edge.source.as_str()), idx.get(edge.target.as_str()))
        {
            adj[src].push(tgt);
        }
    }
    for neighbors in &mut adj {
        neighbors.sort_unstable();
        neighbors.dedup();
    }

    let out_degree: Vec<usize> = adj.iter().map(|v| v.len()).collect();
    let mut in_neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (src, neighbors) in adj.iter().enumerate() {
        for &tgt in neighbors {
            in_neighbors[tgt].push(src);
        }
    }
    for neighbors in &mut in_neighbors {
        neighbors.sort_unstable();
        neighbors.dedup();
    }

    let damping = 0.85;
    let tolerance = 1e-6;
    let max_iterations = 100;

    let mut rank = vec![1.0 / n.max(1) as f64; n];
    for _ in 0..max_iterations {
        let mut new_rank = vec![(1.0 - damping) / n.max(1) as f64; n];

        for v in 0..n {
            let incoming_sum: f64 = in_neighbors[v]
                .iter()
                .map(|&u| {
                    let out = out_degree[u].max(1) as f64;
                    rank[u] / out
                })
                .sum();
            new_rank[v] += damping * incoming_sum;
        }

        let diff: f64 = new_rank
            .iter()
            .zip(rank.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        rank = new_rank;
        if diff < tolerance {
            break;
        }
    }

    let mut scores: HashMap<String, ComponentImportance> = HashMap::new();
    for (i, node) in graph.nodes.iter().enumerate() {
        let in_d = graph.edges.iter().filter(|e| e.target == node.id).count();
        let out_d = graph.edges.iter().filter(|e| e.source == node.id).count();
        let degree = if n <= 1 {
            0.0
        } else {
            (in_d + out_d) as f64 / ((n - 1) as f64 * 2.0)
        };

        let imp = ComponentImportance {
            degree_centrality: degree,
            pagerank: rank.get(i).copied().unwrap_or(0.0),
            ..Default::default()
        };
        scores.insert(node.id.clone(), imp);
    }

    scores
}

fn summarize_node(node: &Node, centrality: &HashMap<String, ComponentImportance>) -> NodeSummary {
    NodeSummary {
        id: node.id.clone(),
        kind: node.kind.clone(),
        label: node.label.clone(),
        technology: node.technology.clone(),
        path: node.path.clone(),
        centrality: centrality.get(&node.id).cloned(),
    }
}

fn build_hits(
    graph: &Graph,
    nodes: &[BlastRadiusNode],
    centrality: &HashMap<String, ComponentImportance>,
) -> Vec<ImpactHit> {
    let mut by_id: HashMap<&str, &Node> = HashMap::new();
    for node in &graph.nodes {
        by_id.insert(node.id.as_str(), node);
    }

    let mut hits: Vec<ImpactHit> = nodes
        .iter()
        .filter_map(|n| {
            let node = by_id.get(n.id.as_str()).copied()?;
            Some(ImpactHit {
                depth: n.depth,
                node: summarize_node(node, centrality),
            })
        })
        .collect();

    hits.sort_by(|a, b| {
        let a_pr = a
            .node
            .centrality
            .as_ref()
            .map(|c| c.pagerank)
            .unwrap_or(0.0);
        let b_pr = b
            .node
            .centrality
            .as_ref()
            .map(|c| c.pagerank)
            .unwrap_or(0.0);
        a.depth
            .cmp(&b.depth)
            .then_with(|| b_pr.partial_cmp(&a_pr).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| a.node.id.cmp(&b.node.id))
    });

    hits
}

fn print_section(title: &str, hits: &[ImpactHit]) {
    println!("{}", title.bold());
    if hits.is_empty() {
        println!("  (none)");
        return;
    }
    for hit in hits {
        let pr = hit
            .node
            .centrality
            .as_ref()
            .map(|c| c.pagerank)
            .unwrap_or(0.0);
        println!(
            "  depth {:<2}  {:<40}  pr={:.6}  {}",
            hit.depth, hit.node.id, pr, hit.node.label
        );
    }
}

pub async fn impact(repo: &str, target: &str, depth: usize, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let output = impact_compute_output(repo_path, target, depth)?;

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            println!(
                "{} {}  kind={}  tech={}  path={}",
                "Target:".bold(),
                output.target.id.bold(),
                format_args!("{:?}", output.target.kind),
                output.target.technology.as_deref().unwrap_or("(not set)"),
                output.target.path.as_deref().unwrap_or("(not set)")
            );
            if let Some(c) = output.target.centrality.as_ref() {
                println!(
                    "centrality: degree={:.4} betweenness={:.4} closeness={:.4} eigenvector={:.4} pagerank={:.6}",
                    c.degree_centrality,
                    c.betweenness_centrality,
                    c.closeness_centrality,
                    c.eigenvector_centrality,
                    c.pagerank
                );
            }

            println!();
            print_section(
                &format!("Upstream dependents ({}):", output.upstream.len()),
                &output.upstream,
            );
            println!();
            print_section(
                &format!("Downstream dependencies ({}):", output.downstream.len()),
                &output.downstream,
            );
        }
    }

    Ok(())
}

pub(crate) fn impact_compute_output(
    repo_path: &Path,
    target: &str,
    depth: usize,
) -> Result<ImpactOutput, CliError> {
    let graph = sruja_scan::scan_repo(repo_path)?;
    let target_id = resolve_target_id(&graph, repo_path, target)?;

    let centrality = if graph.nodes.len() <= 2000 {
        compute_all_centrality(&graph)
    } else {
        compute_light_centrality(&graph)
    };

    let target_node = graph
        .nodes
        .iter()
        .find(|n| n.id == target_id)
        .ok_or_else(|| CliError::validation(format!("Target node '{}' not found", target)))?;

    let blast: BlastRadiusResult = graph.blast_radius(&target_node.id, depth);
    let upstream = build_hits(&graph, &blast.upstream, &centrality);
    let downstream = build_hits(&graph, &blast.downstream, &centrality);

    Ok(ImpactOutput {
        target: summarize_node(target_node, &centrality),
        max_depth: depth,
        upstream,
        downstream,
    })
}
