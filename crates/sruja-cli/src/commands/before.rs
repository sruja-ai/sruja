use std::collections::{HashMap, HashSet};
use std::path::Path;

use colored::Colorize;
use serde::Serialize;

use super::federation::SystemIndexNode;
use super::CliError;

#[derive(Debug, Clone, Serialize)]
pub struct BeforeOutput {
    pub target: String,
    pub target_label: String,
    pub target_kind: String,
    pub target_repo: String,
    pub target_criticality: Option<String>,
    pub downstream_count: usize,
    pub downstream: Vec<DownstreamInfo>,
    pub gotchas: Vec<String>,
    pub teams_to_notify: HashSet<String>,
    pub suggested_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownstreamInfo {
    pub label: String,
    pub repo_id: String,
    pub kind: String,
}

pub async fn before(
    repo_root: &str,
    file: &str,
    format: &str,
    ci: bool,
    threshold: usize,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let index = super::federation::find_or_generate_system_index(repo_path)?;

    let target_node = resolve_from_file(&index, file)?;

    let mut outgoing: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &index.edges {
        outgoing
            .entry(edge.source.as_str())
            .or_default()
            .push(edge.target.as_str());
    }

    let node_map: HashMap<&str, &SystemIndexNode> = index
        .nodes
        .iter()
        .map(|n| (n.canonical_id.as_str(), n))
        .collect();

    let mut downstream = Vec::new();
    let mut teams_to_notify = HashSet::new();
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
                        downstream.push(DownstreamInfo {
                            label: n.label.clone(),
                            repo_id: n.repo_id.clone(),
                            kind: n.kind.clone(),
                        });
                        if let Some(ref owner) = n.owner {
                            teams_to_notify.insert(owner.clone());
                        }
                        queue.push_back((next, depth + 1));
                    }
                }
            }
        }
    }

    if let Some(ref owner) = target_node.owner {
        teams_to_notify.insert(owner.clone());
    }

    let mut gotchas = Vec::new();
    if matches!(target_node.kind.as_str(), "service" | "container") {
        let callers = index
            .edges
            .iter()
            .filter(|e| e.target == target_node.canonical_id)
            .count();
        if callers == 1 {
            gotchas.push(format!(
                "{} has only 1 caller \u{2014} SPOF risk",
                target_node.label
            ));
        }
    }

    let output = BeforeOutput {
        target: target_node.canonical_id.clone(),
        target_label: target_node.label.clone(),
        target_kind: target_node.kind.clone(),
        target_repo: target_node.repo_id.clone(),
        target_criticality: target_node
            .criticality
            .as_ref()
            .map(|c| format!("{:?}", c).to_lowercase()),
        downstream_count: downstream.len(),
        downstream,
        gotchas,
        teams_to_notify,
        suggested_commands: vec![
            format!("sruja explain {}", target_node.label),
            "sruja drift".to_string(),
        ],
    };

    match format {
        "json" => println!("{}", serde_json::to_string_pretty(&output)?),
        _ => print_before_text(&output),
    }

    if ci && output.downstream_count > threshold {
        return Err(CliError::CiGateExceeded {
            message: format!(
                "{} downstream elements exceed threshold of {}",
                output.downstream_count, threshold
            ),
        });
    }

    Ok(())
}

fn resolve_from_file<'a>(
    index: &'a super::federation::SystemIndex,
    file: &str,
) -> Result<&'a SystemIndexNode, CliError> {
    let file_stem = Path::new(file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(file);

    let q = file_stem.to_lowercase();

    if let Some(n) = index
        .nodes
        .iter()
        .find(|n| n.canonical_id.to_lowercase().contains(&q))
    {
        return Ok(n);
    }
    if let Some(n) = index
        .nodes
        .iter()
        .find(|n| n.label.to_lowercase().contains(&q))
    {
        return Ok(n);
    }
    if let Some(n) = index
        .nodes
        .iter()
        .find(|n| n.local_id.to_lowercase().contains(&q))
    {
        return Ok(n);
    }

    Err(CliError::validation(format!(
        "Could not map '{}' to any architecture element. Use an element name, label, or ID from `sruja human map`.",
        file
    )))
}

fn print_before_text(o: &BeforeOutput) {
    println!();
    println!(
        "{} {}",
        "\u{2500}\u{2500} Impact Preview:".bold(),
        o.target_label.bold()
    );
    println!(
        "  repo: {}  |  kind: {}  |  criticality: {}",
        o.target_repo,
        o.target_kind,
        o.target_criticality
            .as_deref()
            .unwrap_or("unknown")
            .to_uppercase()
    );
    println!(
        "  {} downstream elements affected",
        o.downstream_count.to_string().bold()
    );

    if !o.downstream.is_empty() {
        println!();
        println!("{}", "Who it affects:".bold());
        for d in &o.downstream {
            println!("  {} ({} repo) \u{2014} {}", d.label, d.repo_id, d.kind);
        }
    }

    if !o.gotchas.is_empty() {
        println!();
        for g in &o.gotchas {
            println!("  {} {}", "\u{26a0}".yellow(), g);
        }
    }

    if !o.teams_to_notify.is_empty() {
        println!();
        println!(
            "Who to notify: {}",
            o.teams_to_notify
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    println!();
    println!("Run after your change:");
    for cmd in &o.suggested_commands {
        println!("  {}", cmd);
    }
    println!();
}
