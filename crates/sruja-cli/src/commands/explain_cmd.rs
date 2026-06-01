use std::collections::HashSet;
use std::path::Path;

use colored::Colorize;
use serde::Serialize;

use super::federation::{find_system_index, load_system_index, SystemIndexNode};
use super::CliError;

#[derive(Debug, Clone, Serialize)]
pub struct ExplainOutput {
    pub element_id: String,
    pub label: String,
    pub kind: String,
    pub repo_id: String,
    pub technology: Option<String>,
    pub owner: Option<String>,
    pub criticality: Option<String>,
    pub aliases: Vec<String>,
    pub incoming: Vec<ConnectionInfo>,
    pub outgoing: Vec<ConnectionInfo>,
    pub blast_upstream: usize,
    pub blast_downstream: usize,
    pub repos_affected: usize,
    pub teams: HashSet<String>,
    pub spof_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionInfo {
    pub element_id: String,
    pub label: String,
    pub kind: String,
    pub repo_id: String,
    pub edge_kind: String,
    pub edge_label: Option<String>,
}

pub async fn explain_element(
    target: &str,
    repo_root: &str,
    format: &str,
    persist: bool,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let idx_path = find_system_index(repo_path).ok_or_else(|| {
        CliError::validation("No system.index.json found. Run 'sruja compose' first.".to_string())
    })?;
    let index = load_system_index(&idx_path)?;

    let node = resolve_entity(&index.nodes, target)
        .ok_or_else(|| CliError::validation(format!("No element found matching '{}'.", target)))?;

    let output = build_explain(&index, node)?;

    if persist {
        let out_dir = repo_path.join("docs").join("architecture");
        std::fs::create_dir_all(&out_dir)?;
        let fname = format!("{}.md", output.label.replace([' ', '/'], "_"));
        let path = out_dir.join(&fname);
        std::fs::write(&path, render_markdown(&output))?;
        eprintln!("Written: {}", path.display());
        return Ok(());
    }

    match format {
        "json" => println!("{}", serde_json::to_string_pretty(&output)?),
        "md" | "markdown" => println!("{}", render_markdown(&output)),
        _ => print_text(&output),
    }

    Ok(())
}

fn resolve_entity<'a>(nodes: &'a [SystemIndexNode], query: &str) -> Option<&'a SystemIndexNode> {
    let q = query.to_lowercase();
    nodes
        .iter()
        .find(|n| n.canonical_id == query)
        .or_else(|| nodes.iter().find(|n| n.local_id == query))
        .or_else(|| nodes.iter().find(|n| n.label.to_lowercase() == q))
        .or_else(|| {
            nodes
                .iter()
                .find(|n| n.aliases.iter().any(|a| a.to_lowercase() == q))
        })
        .or_else(|| {
            nodes.iter().find(|n| {
                n.label.to_lowercase().contains(&q) || n.local_id.to_lowercase().contains(&q)
            })
        })
}

fn build_explain(
    index: &super::federation::SystemIndex,
    node: &SystemIndexNode,
) -> Result<ExplainOutput, CliError> {
    let node_map: std::collections::HashMap<&str, &SystemIndexNode> = index
        .nodes
        .iter()
        .map(|n| (n.canonical_id.as_str(), n))
        .collect();

    let mut incoming = Vec::new();
    let mut outgoing = Vec::new();

    for edge in &index.edges {
        if edge.target == node.canonical_id {
            if let Some(src) = node_map.get(edge.source.as_str()) {
                incoming.push(ConnectionInfo {
                    element_id: src.canonical_id.clone(),
                    label: src.label.clone(),
                    kind: src.kind.clone(),
                    repo_id: src.repo_id.clone(),
                    edge_kind: edge.kind.clone(),
                    edge_label: edge.label.clone(),
                });
            }
        }
        if edge.source == node.canonical_id {
            if let Some(tgt) = node_map.get(edge.target.as_str()) {
                outgoing.push(ConnectionInfo {
                    element_id: tgt.canonical_id.clone(),
                    label: tgt.label.clone(),
                    kind: tgt.kind.clone(),
                    repo_id: tgt.repo_id.clone(),
                    edge_kind: edge.kind.clone(),
                    edge_label: edge.label.clone(),
                });
            }
        }
    }

    let mut upstream_set = HashSet::new();
    let mut downstream_set = HashSet::new();
    let mut repos_affected = HashSet::new();
    let mut teams = HashSet::new();
    let mut spof_warnings = Vec::new();

    repos_affected.insert(node.repo_id.clone());
    if let Some(ref o) = node.owner {
        teams.insert(o.clone());
    }

    walk_blast(
        &index.edges,
        &node_map,
        &node.canonical_id,
        true,
        &mut upstream_set,
        &mut repos_affected,
        &mut teams,
    );
    walk_blast(
        &index.edges,
        &node_map,
        &node.canonical_id,
        false,
        &mut downstream_set,
        &mut repos_affected,
        &mut teams,
    );

    let caller_count = incoming.len();
    if caller_count == 1 && matches!(node.kind.as_str(), "service" | "container") {
        spof_warnings.push(format!(
            "{} has only 1 caller \u{2014} SPOF risk",
            node.label
        ));
    }

    Ok(ExplainOutput {
        element_id: node.canonical_id.clone(),
        label: node.label.clone(),
        kind: node.kind.clone(),
        repo_id: node.repo_id.clone(),
        technology: node.technology.clone(),
        owner: node.owner.clone(),
        criticality: node
            .criticality
            .as_ref()
            .map(|c| format!("{:?}", c).to_lowercase()),
        aliases: node.aliases.clone(),
        incoming,
        outgoing,
        blast_upstream: upstream_set.len(),
        blast_downstream: downstream_set.len(),
        repos_affected: repos_affected.len(),
        teams,
        spof_warnings,
    })
}

fn walk_blast(
    edges: &[super::federation::SystemIndexEdge],
    node_map: &std::collections::HashMap<&str, &SystemIndexNode>,
    start: &str,
    upstream: bool,
    visited: &mut HashSet<String>,
    repos: &mut HashSet<String>,
    teams: &mut HashSet<String>,
) {
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(start.to_string());
    visited.insert(start.to_string());

    while let Some(current) = queue.pop_front() {
        for edge in edges {
            let next = if upstream && edge.target == current {
                Some(&edge.source)
            } else if !upstream && edge.source == current {
                Some(&edge.target)
            } else {
                None
            };
            if let Some(next_id) = next {
                if visited.insert(next_id.clone()) {
                    if let Some(n) = node_map.get(next_id.as_str()) {
                        repos.insert(n.repo_id.clone());
                        if let Some(ref o) = n.owner {
                            teams.insert(o.clone());
                        }
                    }
                    queue.push_back(next_id.clone());
                }
            }
        }
    }
}

fn print_text(o: &ExplainOutput) {
    println!();
    println!("{} {}", "──".bold(), o.label.bold());
    let mut meta = vec![format!("repo: {}", o.repo_id)];
    if let Some(ref owner) = o.owner {
        meta.push(format!("team: {}", owner));
    }
    if let Some(ref crit) = o.criticality {
        meta.push(format!("criticality: {}", crit.to_uppercase()));
    }
    if let Some(ref tech) = o.technology {
        meta.push(format!("tech: {}", tech));
    }
    println!("  {}", meta.join("  |  ").dimmed());

    if !o.incoming.is_empty() {
        println!();
        println!("{}", "Incoming:".bold());
        for c in &o.incoming {
            println!(
                "  {} {} ({} repo) [{}]",
                "←".cyan(),
                c.label,
                c.repo_id,
                c.edge_kind
            );
        }
    }

    if !o.outgoing.is_empty() {
        println!();
        println!("{}", "Outgoing:".bold());
        for c in &o.outgoing {
            println!(
                "  {} {} ({} repo) [{}]",
                "→".green(),
                c.label,
                c.repo_id,
                c.edge_kind
            );
        }
    }

    if !o.spof_warnings.is_empty() {
        println!();
        for w in &o.spof_warnings {
            println!("  {} {}", "⚠".yellow(), w);
        }
    }

    println!();
    println!(
        "Blast radius: {} upstream · {} downstream · {} repos · {} teams",
        o.blast_upstream.to_string().bold(),
        o.blast_downstream.to_string().bold(),
        o.repos_affected.to_string().bold(),
        o.teams.len().to_string().bold(),
    );

    if !o.teams.is_empty() {
        println!(
            "Owners: {}",
            o.teams.iter().cloned().collect::<Vec<_>>().join(", ")
        );
    }
}

fn render_markdown(o: &ExplainOutput) -> String {
    let mut md = format!("# {}\n\n", o.label);
    md.push_str(&format!(
        "- **Repo:** {}\n- **Kind:** {}\n",
        o.repo_id, o.kind
    ));
    if let Some(ref t) = o.technology {
        md.push_str(&format!("- **Technology:** {}\n", t));
    }
    if let Some(ref owner) = o.owner {
        md.push_str(&format!("- **Owner:** {}\n", owner));
    }
    if let Some(ref c) = o.criticality {
        md.push_str(&format!("- **Criticality:** {}\n", c));
    }
    md.push('\n');

    if !o.incoming.is_empty() {
        md.push_str("## Incoming\n\n");
        for c in &o.incoming {
            md.push_str(&format!(
                "- {} ({} repo) [{}]\n",
                c.label, c.repo_id, c.edge_kind
            ));
        }
        md.push('\n');
    }

    if !o.outgoing.is_empty() {
        md.push_str("## Outgoing\n\n");
        for c in &o.outgoing {
            md.push_str(&format!(
                "- {} ({} repo) [{}]\n",
                c.label, c.repo_id, c.edge_kind
            ));
        }
        md.push('\n');
    }

    md.push_str(&format!(
        "## Blast Radius\n\n- Upstream: {}\n- Downstream: {}\n- Repos: {}\n- Teams: {}\n",
        o.blast_upstream,
        o.blast_downstream,
        o.repos_affected,
        o.teams.len()
    ));
    md
}
