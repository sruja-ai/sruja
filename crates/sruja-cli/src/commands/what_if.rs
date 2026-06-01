use std::collections::HashSet;
use std::path::Path;

use colored::Colorize;
use serde::Serialize;

use super::federation::{find_system_index, load_system_index, SystemIndexNode};
use super::CliError;

#[derive(Debug, Clone, Serialize)]
pub struct WhatIfOutput {
    pub query: String,
    pub direct_effects: Vec<DirectEffect>,
    pub blast_repos: usize,
    pub blast_teams: HashSet<String>,
    pub blast_databases: Vec<String>,
    pub blast_queues: Vec<String>,
    pub risks: Vec<RiskItem>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DirectEffect {
    pub element_id: String,
    pub label: String,
    pub effect: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RiskItem {
    pub level: String,
    pub description: String,
}

pub async fn what_if(query: &str, repo_root: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let idx_path = find_system_index(repo_path).ok_or_else(|| {
        CliError::validation("No system.index.json found. Run 'sruja compose' first.".to_string())
    })?;
    let index = load_system_index(&idx_path)?;

    let target_node = resolve_whatif_target(&index.nodes, query)
        .ok_or_else(|| CliError::validation(format!("No element found matching '{}'.", query)))?;

    let mut direct_effects = Vec::new();
    let mut blast_repos = HashSet::new();
    let mut blast_teams = HashSet::new();
    let mut blast_databases = Vec::new();
    let mut blast_queues = Vec::new();
    let mut risks = Vec::new();

    blast_repos.insert(target_node.repo_id.clone());
    if let Some(ref owner) = target_node.owner {
        blast_teams.insert(owner.clone());
    }

    let node_map: std::collections::HashMap<&str, &SystemIndexNode> = index
        .nodes
        .iter()
        .map(|n| (n.canonical_id.as_str(), n))
        .collect();

    let mut outgoing: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    for edge in &index.edges {
        outgoing
            .entry(edge.source.as_str())
            .or_default()
            .push(edge.target.as_str());
    }

    if let Some(targets) = outgoing.get(target_node.canonical_id.as_str()) {
        for &next in targets {
            if let Some(n) = node_map.get(next) {
                direct_effects.push(DirectEffect {
                    element_id: n.canonical_id.clone(),
                    label: n.label.clone(),
                    effect: format!("directly depends on {}", target_node.label),
                });
            }
        }
    }

    let mut visited = HashSet::new();
    visited.insert(target_node.canonical_id.as_str());
    let mut queue = std::collections::VecDeque::new();
    queue.push_back((target_node.canonical_id.as_str(), 0usize));

    while let Some((current, depth)) = queue.pop_front() {
        if depth >= 3 {
            continue;
        }
        if let Some(targets) = outgoing.get(current) {
            for &next in targets {
                if visited.insert(next) {
                    if let Some(n) = node_map.get(next) {
                        blast_repos.insert(n.repo_id.clone());
                        if let Some(ref owner) = n.owner {
                            blast_teams.insert(owner.clone());
                        }
                        if n.kind == "database" {
                            blast_databases.push(n.label.clone());
                        }
                        if n.kind == "queue" {
                            blast_queues.push(n.label.clone());
                        }
                        queue.push_back((next, depth + 1));
                    }
                }
            }
        }
    }

    if blast_repos.len() > 2 {
        risks.push(RiskItem {
            level: "HIGH".to_string(),
            description: format!("Change affects {} repos", blast_repos.len()),
        });
    }
    if blast_teams.len() > 2 {
        risks.push(RiskItem {
            level: "MEDIUM".to_string(),
            description: format!("{} teams need to coordinate", blast_teams.len()),
        });
    }

    let output = WhatIfOutput {
        query: query.to_string(),
        direct_effects,
        blast_repos: blast_repos.len(),
        blast_teams,
        blast_databases,
        blast_queues,
        risks,
        next_steps: vec![
            format!("sruja explain {}", target_node.label),
            "sruja drift".to_string(),
        ],
    };

    match format {
        "json" => println!("{}", serde_json::to_string_pretty(&output)?),
        _ => print_whatif_text(&output),
    }

    Ok(())
}

fn resolve_whatif_target<'a>(
    nodes: &'a [SystemIndexNode],
    query: &str,
) -> Option<&'a SystemIndexNode> {
    let q = query.to_lowercase();
    nodes
        .iter()
        .find(|n| n.canonical_id == query)
        .or_else(|| nodes.iter().find(|n| n.label.to_lowercase() == q))
        .or_else(|| {
            nodes
                .iter()
                .find(|n| n.local_id.to_lowercase().contains(&q))
        })
        .or_else(|| nodes.iter().find(|n| n.label.to_lowercase().contains(&q)))
}

fn print_whatif_text(o: &WhatIfOutput) {
    println!();
    println!("{} {}", "\u{2500}\u{2500} What-If:".bold(), o.query.bold());
    println!();

    if !o.direct_effects.is_empty() {
        println!("{}", "Direct effect:".bold());
        for effect in &o.direct_effects {
            println!("  {}: {}", effect.label, effect.effect);
        }
    }

    println!();
    println!(
        "Blast radius: {} repos \u{00b7} {} teams \u{00b7} {} databases \u{00b7} {} topics",
        o.blast_repos.to_string().bold(),
        o.blast_teams.len().to_string().bold(),
        o.blast_databases.len().to_string().bold(),
        o.blast_queues.len().to_string().bold(),
    );

    if !o.risks.is_empty() {
        println!();
        println!("{}", "Risks:".bold());
        for risk in &o.risks {
            let level_str = match risk.level.as_str() {
                "HIGH" => risk.level.red().bold().to_string(),
                "MEDIUM" => risk.level.yellow().to_string(),
                _ => risk.level.green().to_string(),
            };
            println!("  {} \u{2014} {}", level_str, risk.description);
        }
    }

    if !o.next_steps.is_empty() {
        println!();
        println!("{}", "Next steps:".bold());
        for (i, step) in o.next_steps.iter().enumerate() {
            println!("  {}. {}", i + 1, step);
        }
    }
    println!();
}
