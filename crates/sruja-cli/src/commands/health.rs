use crate::commands::CliError;
use crate::scoring::health::{calculate_health, Deduction};
use crate::utils::{colors, progress};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HealthScoreOutput {
    pub score: u8,
    pub trend: String,
    pub architecture: String,
    pub deductions: Vec<Deduction>,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct HealthHistory {
    pub scores: Vec<HistoryEntry>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct HistoryEntry {
    pub timestamp: u64,
    pub score: u8,
}

pub async fn health(
    repo_root: &str,
    architecture: Option<&str>,
    format: &str,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);

    // 1. Parse architecture file
    let arch_path = crate::utils::architecture_path::resolve_architecture_path_or_default(
        repo_path,
        architecture,
    );
    let (_content, program) = super::parse_sruja_file(
        arch_path
            .to_str()
            .ok_or_else(|| CliError::validation("Architecture path is not valid UTF-8"))?,
    )?;

    // 2. Run drift detection to get violations
    let pb = progress::spinner("Calculating health score...");
    let graph = match sruja_scan::scan_repo(repo_path) {
        Ok(g) => g,
        Err(e) => {
            pb.abandon();
            return Err(CliError::scan_with_help(
                e.to_string(),
                "Ensure your repo has readable source files and your ignore rules are correct.",
            ));
        }
    };

    let proposed_graph = sruja_diff::program_to_graph(&program);
    let diff = sruja_diff::compare_graphs(&graph, &proposed_graph);
    pb.finish_and_clear();

    // 3. Calculate score
    let health = calculate_health(&diff.violations, &program);

    // 4. Persistence & Trend
    let dot_sruja = repo_path.join(".sruja");
    if !dot_sruja.exists() {
        let _ = fs::create_dir_all(&dot_sruja);
    }

    let history_path = dot_sruja.join("health_history.json");
    let mut history: HealthHistory = if history_path.exists() {
        let content = fs::read_to_string(&history_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HealthHistory::default()
    };

    let last_score = history.scores.last().map(|e| e.score);
    let trend = if let Some(last) = last_score {
        if health.score > last {
            "up"
        } else if health.score < last {
            "down"
        } else {
            "steady"
        }
    } else {
        "new"
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Only append if it's a new timestamp (rough avoid duplicates in same run)
    if history
        .scores
        .last()
        .map(|e| e.timestamp != now)
        .unwrap_or(true)
    {
        history.scores.push(HistoryEntry {
            timestamp: now,
            score: health.score,
        });
        // Keep last 50 entries
        if history.scores.len() > 50 {
            history.scores.remove(0);
        }
        let _ = fs::write(
            &history_path,
            serde_json::to_string_pretty(&history).unwrap_or_default(),
        );
    }

    match format {
        "json" => {
            let output = HealthScoreOutput {
                score: health.score,
                trend: trend.to_string(),
                architecture: arch_path.to_string_lossy().to_string(),
                deductions: health.deductions,
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&output)
                    .map_err(|e| CliError::validation(e.to_string()))?
            );
        }
        _ => {
            colors::print_header("🩺 Architecture Health Report");

            let trend_icon = match trend {
                "up" => colors::success("↑").to_string(),
                "down" => colors::error("↓").to_string(),
                "steady" => colors::dim("→").to_string(),
                _ => "".to_string(),
            };

            println!(
                "  Score: {} {}",
                colors::health_bar(health.score, 20),
                trend_icon
            );
            println!("  File:  {}", arch_path.display());
            println!();

            if health.deductions.is_empty() {
                println!(
                    "  {} Your architecture is in perfect health!",
                    colors::success("✨")
                );
            } else {
                println!("{}", colors::style("Deductions:").bold());
                for d in &health.deductions {
                    println!(
                        "  {} {} (-{} pts)",
                        colors::error("-"),
                        colors::dim(&d.message),
                        d.points
                    );
                }

                println!();
                println!("{}", colors::style("Recommendations:").bold());
                if health.score < 90 {
                    println!("  • Resolve architectural drift using 'sruja drift --fix'");
                    println!("  • Add missing descriptions to components in your .sruja files");
                    println!("  • Ensure all components are linked to a system or container (no orphans)");
                } else {
                    println!("  • Your architecture is looking great! Keep maintaining it.");
                }
            }
        }
    }

    Ok(())
}
