//! Context Score command: the "Lighthouse score for context engineering."
//!
//! Shows a single 0-100 score quantifying how well-equipped an AI agent is
//! to work on this codebase, with a breakdown across 5 dimensions and
//! actionable quick wins.

use std::path::Path;

use crate::commands::sync_cmd::ScoreHistoryEntry;
use crate::commands::CliError;
use crate::graph_store;
use crate::utils::colors;
use sruja_graph::{compute_context_score, ContextScore, DimensionScore};

fn load_score_history(repo_path: &Path) -> Vec<ScoreHistoryEntry> {
    let path = repo_path.join(".sruja/score_history.jsonl");
    if !path.exists() {
        return Vec::new();
    }

    let Ok(content) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };

    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<ScoreHistoryEntry>(line).ok())
        .collect()
}

fn compute_trend(history: &[ScoreHistoryEntry], current_score: u8) -> Option<(i16, u8)> {
    if history.len() < 2 {
        return None;
    }

    // Get the previous score (second to last)
    let prev = history.get(history.len() - 2)?;
    let delta = current_score as i16 - prev.score as i16;
    Some((delta, prev.score))
}

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
            let mut v = serde_json::to_value(&score)?;
            if let serde_json::Value::Object(ref mut map) = v {
                map.insert(
                    "metric_type".to_string(),
                    serde_json::Value::String("ai_readiness".to_string()),
                );
                map.insert(
                    "metric_description".to_string(),
                    serde_json::Value::String(
                        "AI preparedness (0–100). Use `sruja status` for a unified view."
                            .to_string(),
                    ),
                );

                // Add history to JSON output
                let history = load_score_history(repo_path);
                if !history.is_empty() {
                    map.insert(
                        "history".to_string(),
                        serde_json::to_value(&history).unwrap_or(serde_json::Value::Null),
                    );
                }
            }
            println!("{}", serde_json::to_string_pretty(&v)?);
        }
        _ => {
            let history = load_score_history(repo_path);
            let trend = compute_trend(&history, score.score);
            print_context_score(&score, trend.as_ref());
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

fn print_context_score(score: &ContextScore, trend: Option<&(i16, u8)>) {
    println!(
        "{}",
        colors::dim("AI preparedness (0–100). Use `sruja status` for a unified view.")
    );
    println!();
    println!("╭──────────────────────────────────────────────────────╮");
    println!(
        "│  {} {}  │",
        colors::style("Context Score:").bold(),
        colors::health_bar(score.score, 20)
    );

    // Show trend if available
    if let Some((delta, prev_score)) = trend {
        let msg = if *delta > 0 {
            format!("+{} (was {})", delta, prev_score)
        } else if *delta < 0 {
            format!("{} (was {})", delta, prev_score)
        } else {
            format!("stable (was {})", prev_score)
        };

        let trend_str = if *delta > 0 {
            colors::success(&msg)
        } else if *delta < 0 {
            colors::error(&msg)
        } else {
            colors::dim(&msg)
        };
        println!("│  {:<24} {}  │", "Trend:", trend_str);
    }

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
