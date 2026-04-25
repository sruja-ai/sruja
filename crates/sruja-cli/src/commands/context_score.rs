//! Context Score command: the "Lighthouse score for context engineering."
//!
//! Shows a single 0-100 score quantifying how well-equipped an AI agent is
//! to work on this codebase, with a breakdown across 5 dimensions and
//! actionable quick wins.

use std::path::Path;

use crate::commands::CliError;
use crate::graph_store;
use crate::utils::colors;
use sruja_graph::{compute_context_score, ContextScore, DimensionScore};

pub async fn context_score(
    repo_root: &str,
    format: &str,
    fail_under: Option<u8>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    // Load the knowledge graph (prefer architecture baseline if it exists)
    let baseline_path = crate::utils::architecture_path::resolve_architecture_path(repo_path);
    let kg = if let Some(ref path) = baseline_path {
        let content = std::fs::read_to_string(path)?;
        let parser = sruja_language::Parser::new(path.to_string_lossy().to_string());
        let program = parser.parse(&content).map_err(|diags| {
            CliError::parse_with_diagnostics(path.to_string_lossy().to_string(), diags)
        })?;

        // Convert the program to a base scan graph
        let scan_graph = sruja_diff::program_to_graph(&program);

        // Build a fresh KnowledgeGraph and merge the scan graph into it
        let mut kg = sruja_graph::KnowledgeGraph::new();
        sruja_graph::scan_merge::merge_scan_into_graph(
            &mut kg,
            &scan_graph,
            &repo_path.display().to_string(),
        );

        // Also merge rich program data (like ADRs)
        sruja_graph::scan_merge::merge_program_into_graph(
            &mut kg,
            &program,
            &repo_path.display().to_string(),
        );

        kg
    } else {
        graph_store::load_or_build_graph(repo_path)?
    };

    // Get scan node count for coverage comparison
    let scan_node_count = match sruja_scan::scan_repo(repo_path) {
        Ok(g) => g.nodes.len(),
        Err(_) => kg.nodes.len(),
    };

    // Compute context age
    let age_hours = crate::utils::context::context_age_hours(repo_path);

    // Compute score
    let score = compute_context_score(&kg, scan_node_count, repo_path, age_hours);

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&score)?);
        }
        _ => {
            print_context_score(&score);
        }
    }

    if let Some(threshold) = fail_under {
        if score.score < threshold {
            return Err(CliError::validation(format!(
                "Context score {} is below the required threshold of {}",
                score.score, threshold
            )));
        }
    }

    Ok(())
}

fn dimension_bar(dim: &DimensionScore, width: usize) -> String {
    let pct = dim.pct_u8();
    let filled_len = (pct as f32 / 100.0 * width as f32).round() as usize;
    let empty_len = width.saturating_sub(filled_len);

    let bar_char = if pct >= 80 {
        colors::success("█")
    } else if pct >= 50 {
        colors::info("█")
    } else if pct >= 30 {
        colors::warning("█")
    } else {
        colors::error("█")
    };

    let filled = bar_char.to_string().repeat(filled_len);
    let empty = colors::dim("░").to_string().repeat(empty_len);

    format!("{}{}", filled, empty)
}

fn print_context_score(score: &ContextScore) {
    println!();
    println!("╭──────────────────────────────────────────────────────╮");
    println!(
        "│  {} {}  │",
        colors::style("Context Score:").bold(),
        colors::health_bar(score.score, 20)
    );
    println!("│                                                      │");

    // Dimension breakdown
    let dims = [
        &score.architecture_coverage,
        &score.decision_completeness,
        &score.evidence_freshness,
        &score.relationship_density,
        &score.external_context,
    ];

    for dim in dims {
        let bar = dimension_bar(dim, 9);
        let pct = dim.pct_u8();
        let pct_str = format!("{:>3}%", pct);
        let pct_colored = if pct >= 80 {
            colors::success(&pct_str)
        } else if pct >= 50 {
            colors::info(&pct_str)
        } else if pct >= 30 {
            colors::warning(&pct_str)
        } else {
            colors::error(&pct_str)
        };
        println!("│  {:<24} {} {}  │", dim.name, pct_colored, bar);
    }

    println!("│                                                      │");

    // Quick wins
    if !score.quick_wins.is_empty() {
        println!(
            "│  {} Quick wins to improve:                      │",
            colors::style("⚡").bold()
        );
        for (i, qw) in score.quick_wins.iter().enumerate() {
            let action = if qw.action.len() > 42 {
                format!("{}...", &qw.action[..39])
            } else {
                qw.action.clone()
            };
            println!("│  {}. {} (→ +{} pts) │", i + 1, action, qw.impact_points);
        }
    }

    println!("╰──────────────────────────────────────────────────────╯");
    println!();

    // Dimension details
    println!("{}", colors::style("Details:").bold());
    for dim in dims {
        println!("  • {}: {}", colors::dim(&dim.name), dim.detail);
    }
    println!();
}
