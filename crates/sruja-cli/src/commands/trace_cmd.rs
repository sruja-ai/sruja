use std::collections::HashSet;
use std::path::Path;

use colored::Colorize;
use serde::Serialize;

use super::federation::{resolve_entity, SystemIndexNode};
use super::CliError;
use crate::utils::colors;

#[derive(Debug, Clone, Serialize)]
pub struct TraceOutput {
    pub query: String,
    pub hops: Vec<TraceHop>,
    pub repos_touched: usize,
    pub teams: HashSet<String>,
    pub warnings: Vec<String>,
    pub databases: Vec<String>,
    pub queues: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceHop {
    pub node_id: String,
    pub label: String,
    pub kind: String,
    pub repo_id: String,
    pub technology: Option<String>,
    pub owner: Option<String>,
    pub edge_kind: Option<String>,
    pub edge_label: Option<String>,
    pub depth: usize,
}

pub async fn trace(
    query: &str,
    repo_root: &str,
    depth: usize,
    team: Option<&str>,
    format: &str,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);

    let index = super::federation::find_or_generate_system_index(repo_path)?;

    let start_node = resolve_entity(&index.nodes, query).ok_or_else(|| {
        CliError::validation(format!(
            "No element found matching '{}'. Use exact name, label, or alias.",
            query
        ))
    })?;

    let output = build_trace(&index, query, start_node, depth, team)?;

    match format {
        "json" => println!("{}", serde_json::to_string_pretty(&output)?),
        _ => print_trace_text(&output),
    }

    Ok(())
}

fn build_trace(
    index: &crate::commands::federation::SystemIndex,
    query: &str,
    start_node: &SystemIndexNode,
    max_depth: usize,
    team_filter: Option<&str>,
) -> Result<TraceOutput, CliError> {
    let node_map: std::collections::HashMap<&str, &SystemIndexNode> = index
        .nodes
        .iter()
        .map(|n| (n.canonical_id.as_str(), n))
        .collect();

    #[allow(clippy::type_complexity)]
    let mut outgoing: std::collections::HashMap<&str, Vec<(&str, &str, Option<&str>)>> =
        std::collections::HashMap::new();
    for edge in &index.edges {
        outgoing.entry(edge.source.as_str()).or_default().push((
            edge.target.as_str(),
            edge.kind.as_str(),
            edge.label.as_deref(),
        ));
    }

    let mut hops = Vec::new();
    let mut visited = HashSet::new();
    let mut repos_touched = HashSet::new();
    let mut teams = HashSet::new();
    let mut warnings = Vec::new();
    let mut databases = Vec::new();
    let mut queues = Vec::new();

    hops.push(TraceHop {
        node_id: start_node.canonical_id.clone(),
        label: start_node.label.clone(),
        kind: start_node.kind.clone(),
        repo_id: start_node.repo_id.clone(),
        technology: start_node.technology.clone(),
        owner: start_node.owner.clone(),
        edge_kind: None,
        edge_label: None,
        depth: 0,
    });

    repos_touched.insert(start_node.repo_id.clone());
    if let Some(ref owner) = start_node.owner {
        teams.insert(owner.clone());
    }

    if matches!(start_node.kind.as_str(), "database") {
        databases.push(start_node.label.clone());
    }
    if matches!(start_node.kind.as_str(), "queue") {
        queues.push(start_node.label.clone());
    }

    visited.insert(start_node.canonical_id.clone());
    let mut queue_bfs = std::collections::VecDeque::new();
    queue_bfs.push_back((start_node.canonical_id.as_str(), 0usize));

    while let Some((current, depth)) = queue_bfs.pop_front() {
        if depth >= max_depth {
            continue;
        }

        if let Some(neighbors) = outgoing.get(current) {
            for &(next, edge_kind, edge_label) in neighbors {
                if visited.insert(next.to_string()) {
                    if let Some(next_node) = node_map.get(next) {
                        if let Some(filter) = team_filter {
                            if next_node.owner.as_deref() != Some(filter) {
                                continue;
                            }
                        }

                        let is_spof = matches!(next_node.kind.as_str(), "service" | "container")
                            && index
                                .edges
                                .iter()
                                .filter(|e| e.target == next_node.canonical_id)
                                .count()
                                <= 1;

                        if is_spof {
                            warnings
                                .push(format!("SPOF: {} (single incoming path)", next_node.label));
                        }

                        if matches!(next_node.kind.as_str(), "database") {
                            databases.push(next_node.label.clone());
                        }
                        if matches!(next_node.kind.as_str(), "queue") {
                            queues.push(next_node.label.clone());
                        }

                        hops.push(TraceHop {
                            node_id: next.to_string(),
                            label: next_node.label.clone(),
                            kind: next_node.kind.clone(),
                            repo_id: next_node.repo_id.clone(),
                            technology: next_node.technology.clone(),
                            owner: next_node.owner.clone(),
                            edge_kind: Some(edge_kind.to_string()),
                            edge_label: edge_label.map(|l| l.to_string()),
                            depth: depth + 1,
                        });

                        repos_touched.insert(next_node.repo_id.clone());
                        if let Some(ref owner) = next_node.owner {
                            teams.insert(owner.clone());
                        }

                        queue_bfs.push_back((next, depth + 1));
                    }
                }
            }
        }
    }

    Ok(TraceOutput {
        query: query.to_string(),
        hops,
        repos_touched: repos_touched.len(),
        teams,
        warnings: {
            let mut w = warnings;
            w.sort();
            w.dedup();
            w
        },
        databases: {
            let mut d = databases;
            d.sort();
            d.dedup();
            d
        },
        queues: {
            let mut q = queues;
            q.sort();
            q.dedup();
            q
        },
    })
}

fn print_trace_text(output: &TraceOutput) {
    println!();
    println!("{} {}", "── Trace:".bold(), colors::info(&output.query));
    println!();

    for hop in &output.hops {
        let repo_tag = format!("[{}]", hop.repo_id);
        let tech_tag = hop
            .technology
            .as_deref()
            .map(|t| format!(" [{}]", t))
            .unwrap_or_default();
        let arrow = if hop.depth == 0 { "" } else { "  →" };

        println!(
            "{:width$}{} {} {}{}",
            "",
            arrow,
            repo_tag.dimmed(),
            hop.label.bold(),
            tech_tag.dimmed(),
            width = hop.depth * 2,
        );

        if let Some(ref edge_kind) = hop.edge_kind {
            let label = hop.edge_label.as_deref().unwrap_or(edge_kind);
            println!(
                "{:width$}  {} {}",
                "",
                "via".dimmed(),
                colors::dim(label),
                width = hop.depth * 2,
            );
        }
    }

    println!();
    println!(
        "  {} repos · {} teams · {} databases · {} topics",
        output.repos_touched.to_string().bold(),
        output.teams.len().to_string().bold(),
        output.databases.len().to_string().bold(),
        output.queues.len().to_string().bold(),
    );

    if !output.warnings.is_empty() {
        println!();
        for warning in &output.warnings {
            println!("  {} {}", "⚠".yellow(), warning);
        }
    }

    if !output.teams.is_empty() {
        println!();
        println!(
            "  Owners: {}",
            output.teams.iter().cloned().collect::<Vec<_>>().join(", ")
        );
    }
}
