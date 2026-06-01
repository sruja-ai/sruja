use std::collections::HashMap;
use std::path::Path;

use colored::Colorize;
use serde::Serialize;

use super::CliError;

#[derive(Debug, Clone, Serialize)]
pub struct CognitiveDebtOutput {
    pub system_score: u8,
    pub threshold: u8,
    pub elements: Vec<DebtElement>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DebtElement {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub repo_id: String,
    pub downstream_count: usize,
    pub owner: Option<String>,
    pub risk_level: String,
    pub reasoning: Vec<String>,
}

pub async fn cognitive_debt(repo_root: &str, format: &str, ci: bool) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let graph = crate::commands::scan_repo_cached(repo_path)?;

    let repo_id = repo_path
        .canonicalize()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "local".to_string());

    let centrality = sruja_scan::graph::compute_all_centrality(&graph);
    let node_map: HashMap<&str, &sruja_scan::Node> =
        graph.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    let mut elements = Vec::new();

    for node in &graph.nodes {
        let downstream: Vec<&str> = graph
            .edges
            .iter()
            .filter(|e| e.source == node.id)
            .filter_map(|e| {
                if node_map.contains_key(e.target.as_str()) {
                    Some(e.target.as_str())
                } else {
                    None
                }
            })
            .collect();

        if downstream.is_empty()
            && !matches!(node.kind.as_str(), "service" | "container" | "database")
        {
            continue;
        }

        let importance = centrality.get(&node.id);
        let pr = importance.map(|c| c.pagerank).unwrap_or(0.0);
        let has_docs = !node.gotchas.is_empty() || !node.runbooks.is_empty();
        let has_owner = node.owner.is_some();

        let mut risk_score: f64 = 0.0;
        let mut reasoning = Vec::new();

        if downstream.len() >= 3 {
            risk_score += (downstream.len() as f64) * 0.3;
            reasoning.push(format!("{} downstream consumers", downstream.len()));
        }

        if pr > 0.01 && !has_docs {
            risk_score += pr * 100.0;
            reasoning.push("High centrality, no documentation".to_string());
        }

        if !has_owner {
            risk_score += 0.5;
            reasoning.push("No owner assigned".to_string());
        }

        if node.gotchas.is_empty()
            && node.runbooks.is_empty()
            && matches!(node.kind.as_str(), "service" | "container")
        {
            risk_score += 0.3;
            reasoning.push("No gotchas or runbooks".to_string());
        }

        let risk_level = if risk_score >= 2.0 {
            "HIGH".to_string()
        } else if risk_score >= 1.0 {
            "MEDIUM".to_string()
        } else {
            "LOW".to_string()
        };

        if risk_level != "LOW" || downstream.len() >= 2 {
            elements.push(DebtElement {
                id: node.id.clone(),
                label: node.label.clone(),
                kind: node.kind.as_str().to_string(),
                repo_id: repo_id.clone(),
                downstream_count: downstream.len(),
                owner: node.owner.clone(),
                risk_level,
                reasoning,
            });
        }
    }

    elements.sort_by(|a, b| {
        let risk_ord = |r: &str| match r {
            "HIGH" => 0,
            "MEDIUM" => 1,
            _ => 2,
        };
        risk_ord(&a.risk_level)
            .cmp(&risk_ord(&b.risk_level))
            .then_with(|| b.downstream_count.cmp(&a.downstream_count))
    });

    let high_count = elements.iter().filter(|e| e.risk_level == "HIGH").count();
    let system_score = if high_count == 0 {
        85u8
    } else {
        100u8.saturating_sub((high_count as u8).saturating_mul(15))
    };
    let threshold = 70;

    let output = CognitiveDebtOutput {
        system_score,
        threshold,
        elements,
    };

    match format {
        "json" => println!("{}", serde_json::to_string_pretty(&output)?),
        _ => {
            print_debt_text(&output);
            if ci && system_score < threshold {
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn print_debt_text(o: &CognitiveDebtOutput) {
    println!();
    println!(
        "{}",
        "\u{2500}\u{2500} Cognitive Debt Report \u{2500}\u{2500}".bold()
    );
    println!();

    for elem in &o.elements {
        let risk_color = match elem.risk_level.as_str() {
            "HIGH" => format!("{}", elem.risk_level.red().bold()),
            "MEDIUM" => format!("{}", elem.risk_level.yellow()),
            _ => format!("{}", elem.risk_level.green()),
        };
        println!(
            "  [{}] {} ({} repo) \u{2014} {} downstream",
            risk_color,
            elem.label.bold(),
            elem.repo_id,
            elem.downstream_count
        );
        for reason in &elem.reasoning {
            println!("    {}", reason.dimmed());
        }
    }

    println!();
    println!(
        "System score: {}/100  (threshold: {})",
        o.system_score.to_string().bold(),
        o.threshold
    );
    if o.system_score < o.threshold {
        println!(
            "  {} Score below threshold \u{2014} needs attention",
            "\u{26a0}".yellow()
        );
    }
    println!();
}
