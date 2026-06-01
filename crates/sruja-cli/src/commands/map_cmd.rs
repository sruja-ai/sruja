use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use colored::Colorize;
use serde::Serialize;

use super::federation::SystemIndexNode;
use super::CliError;

#[derive(Debug, Clone, Serialize)]
pub struct MapOutput {
    pub title: String,
    pub repo_count: usize,
    pub service_count: usize,
    pub database_count: usize,
    pub queue_count: usize,
    pub team_count: usize,
    pub groups: Vec<MapGroup>,
    pub spofs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MapGroup {
    pub repo_id: String,
    pub owner: Option<String>,
    pub nodes: Vec<MapNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MapNode {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub technology: Option<String>,
    pub criticality: Option<String>,
    pub targets: Vec<MapTarget>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MapTarget {
    pub label: String,
    pub repo_id: String,
    pub kind: String,
    pub edge_kind: String,
}

pub async fn system_map(
    repo_root: &str,
    format: &str,
    team: Option<&str>,
    focus: Option<&str>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let index = super::federation::find_or_generate_system_index(repo_path)?;

    let title = index
        .repos
        .first()
        .map(|r| r.repo_id.clone())
        .unwrap_or_else(|| "System".to_string());

    let node_map: std::collections::HashMap<&str, &SystemIndexNode> = index
        .nodes
        .iter()
        .map(|n| (n.canonical_id.as_str(), n))
        .collect();

    #[allow(clippy::type_complexity)]
    let mut outgoing_map: std::collections::HashMap<&str, Vec<(&str, &str, Option<&str>)>> =
        std::collections::HashMap::new();
    for edge in &index.edges {
        outgoing_map.entry(edge.source.as_str()).or_default().push((
            edge.target.as_str(),
            edge.kind.as_str(),
            edge.label.as_deref(),
        ));
    }

    let filtered_nodes: Vec<&SystemIndexNode> = index
        .nodes
        .iter()
        .filter(|n| {
            if let Some(t) = team {
                n.owner.as_deref() == Some(t)
            } else {
                true
            }
        })
        .filter(|n| {
            if let Some(f) = focus {
                let f_lower = f.to_lowercase();
                n.canonical_id.to_lowercase().contains(&f_lower)
                    || n.label.to_lowercase().contains(&f_lower)
                    || n.local_id.to_lowercase().contains(&f_lower)
            } else {
                true
            }
        })
        .collect();

    let mut repo_groups: BTreeMap<String, Vec<&SystemIndexNode>> = BTreeMap::new();
    for node in &filtered_nodes {
        repo_groups
            .entry(node.repo_id.clone())
            .or_default()
            .push(node);
    }

    let mut groups = Vec::new();
    let mut service_count = 0;
    let mut database_count = 0;
    let mut queue_count = 0;
    let mut all_teams = HashSet::new();

    for (repo_id, nodes) in &repo_groups {
        let owner = nodes.iter().find_map(|n| n.owner.clone());
        let mut map_nodes = Vec::new();

        for node in nodes {
            let targets: Vec<MapTarget> = outgoing_map
                .get(node.canonical_id.as_str())
                .map(|v| {
                    v.iter()
                        .filter_map(|(tgt_id, kind, _label)| {
                            node_map.get(tgt_id).map(|t| MapTarget {
                                label: t.label.clone(),
                                repo_id: t.repo_id.clone(),
                                kind: t.kind.clone(),
                                edge_kind: kind.to_string(),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            match node.kind.as_str() {
                "service" => service_count += 1,
                "database" => database_count += 1,
                "queue" => queue_count += 1,
                _ => {}
            }
            if let Some(ref o) = node.owner {
                all_teams.insert(o.clone());
            }

            map_nodes.push(MapNode {
                id: node.canonical_id.clone(),
                label: node.label.clone(),
                kind: node.kind.clone(),
                technology: node.technology.clone(),
                criticality: node
                    .criticality
                    .as_ref()
                    .map(|c| format!("{:?}", c).to_lowercase()),
                targets,
            });
        }

        groups.push(MapGroup {
            repo_id: repo_id.clone(),
            owner,
            nodes: map_nodes,
        });
    }

    let spofs: Vec<String> = index
        .nodes
        .iter()
        .filter(|n| matches!(n.kind.as_str(), "service" | "container"))
        .filter(|n| {
            index
                .edges
                .iter()
                .filter(|e| e.target == n.canonical_id)
                .count()
                <= 1
        })
        .map(|n| n.label.clone())
        .collect();

    let output = MapOutput {
        title,
        repo_count: index.repos.len(),
        service_count,
        database_count,
        queue_count,
        team_count: all_teams.len(),
        groups,
        spofs,
    };

    match format {
        "json" => println!("{}", serde_json::to_string_pretty(&output)?),
        _ => print_map_text(&output),
    }

    Ok(())
}

fn print_map_text(o: &MapOutput) {
    println!();
    println!(
        "{} {}",
        "──".bold(),
        format!(
            "{} · {} repos · {} services · {} databases · {} topics · {} teams",
            o.title, o.repo_count, o.service_count, o.database_count, o.queue_count, o.team_count
        )
        .bold()
    );
    println!();

    for group in &o.groups {
        let owner_tag = group.owner.as_deref().unwrap_or("unowned");
        println!(
            "{} ({} repo · {})",
            group.repo_id.bold(),
            group.repo_id,
            owner_tag.dimmed()
        );

        for (i, node) in group.nodes.iter().enumerate() {
            let is_last = i == group.nodes.len() - 1;
            let connector = if is_last { "└─" } else { "├─" };
            let tech_tag = node
                .technology
                .as_deref()
                .map(|t| format!(" [{}]", t))
                .unwrap_or_default();
            let crit_tag = node
                .criticality
                .as_deref()
                .map(|c| format!(" ★ {}", c.to_uppercase()))
                .unwrap_or_default();

            println!(
                "  {} {} {}{}{}",
                connector.dimmed(),
                node.kind.dimmed(),
                node.label.bold(),
                tech_tag.dimmed(),
                crit_tag.yellow()
            );

            for (j, tgt) in node.targets.iter().enumerate() {
                let tgt_is_last = j == node.targets.len() - 1 && is_last;
                let tgt_connector = if tgt_is_last {
                    "   └─"
                } else {
                    "   ├─"
                };
                let cross_repo = if tgt.repo_id != group.repo_id {
                    format!(" ({} repo)", tgt.repo_id).dimmed().to_string()
                } else {
                    String::new()
                };
                println!(
                    "  {} {} {}{}",
                    tgt_connector.dimmed(),
                    tgt.kind.dimmed(),
                    tgt.label,
                    cross_repo
                );
            }
        }
        println!();
    }

    if !o.spofs.is_empty() {
        for spof in &o.spofs {
            println!("  {} SPOF: {}", "★".yellow(), spof);
        }
        println!();
    }
}
