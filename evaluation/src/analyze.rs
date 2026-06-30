//! Cross-run pattern analysis engine.
//!
//! Reads structured results from one or more eval runs and produces:
//! - Success matrix (task × run, pass/fail with category/difficulty)
//! - Failure clusters (grouped by ErrorClass × category × agent phase)
//! - Memory effectiveness scores
//! - Improvement recommendations

use crate::report;
use crate::runner::RunResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Analysis output: all computed metrics and recommendations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    /// Run being analyzed.
    pub run_id: String,
    pub run_tag: String,
    /// Baseline run for comparison (if any).
    pub baseline_id: Option<String>,
    pub baseline_tag: Option<String>,

    /// Success matrix (task × run).
    pub success_matrix: Vec<SuccessRow>,
    /// Overall pass rate.
    pub pass_rate: f64,
    /// Baseline pass rate (if compared).
    pub baseline_pass_rate: Option<f64>,

    /// Failure clusters by error class.
    pub failure_clusters: Vec<FailureCluster>,
    /// Performance by category.
    pub category_breakdown: Vec<CategoryBreakdown>,
    /// Performance by difficulty.
    pub difficulty_breakdown: Vec<DifficultyBreakdown>,

    /// Memory effectiveness (if memory mode).
    pub memory_analysis: Option<MemoryAnalysis>,

    /// Generated improvement recommendations.
    pub recommendations: Vec<Recommendation>,

    /// Duration and cost summaries.
    pub total_duration_ms: u64,
    pub avg_duration_per_task_ms: f64,
}

/// A row in the success matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessRow {
    pub instance_id: String,
    pub category: String,
    pub difficulty: u8,
    pub passed: bool,
    pub status: String,
    pub baseline_passed: Option<bool>,
    pub regressed: Option<bool>, // true if was passing but now failing
    pub improved: Option<bool>,  // true if was failing but now passing
    pub error_class: Option<String>,
    pub duration_ms: u64,
}

/// A cluster of failures with a common error class and category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureCluster {
    pub error_class: String,
    pub category: String,
    pub count: usize,
    pub task_ids: Vec<String>,
    pub avg_difficulty: f64,
    pub representative_reason: Option<String>,
}

/// Performance breakdown by category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryBreakdown {
    pub category: String,
    pub total: usize,
    pub passed: usize,
    pub pass_rate: f64,
}

/// Performance breakdown by difficulty level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyBreakdown {
    pub difficulty: u8,
    pub total: usize,
    pub passed: usize,
    pub pass_rate: f64,
}

/// Memory effectiveness analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAnalysis {
    pub total_learnings: usize,
    pub guardrails: usize,
    pub playbooks: usize,
    pub invariants: usize,
    pub avg_retrieval_count: f64,
    pub avg_utility_ratio: Option<f64>,
    pub low_utility_count: usize,
}

/// A generated improvement recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub priority: String, // "critical", "high", "medium", "low"
    pub area: String,     // e.g., "comprehension", "planning", "critique", "routing"
    pub title: String,
    pub description: String,
    pub evidence: String,
    pub suggested_action: String,
}

/// Load a run result from a results directory.
fn load_run_result(run_dir: &PathBuf) -> Result<RunResult, String> {
    let result_path = run_dir.join("results.json");
    if !result_path.exists() {
        return Err(format!("results.json not found in {}", run_dir.display()));
    }
    let content = std::fs::read_to_string(&result_path)
        .map_err(|e| format!("Failed to read {}: {e}", result_path.display()))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {e}", result_path.display()))
}

/// Find the latest run directory.
fn find_latest_run(results_dir: &PathBuf) -> Result<PathBuf, String> {
    let mut entries: Vec<_> = std::fs::read_dir(results_dir)
        .map_err(|e| format!("Failed to read {}: {e}", results_dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|e| e.path().join("results.json").exists())
        .collect();

    entries.sort_by_key(|e| {
        e.path()
            .join("results.json")
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
    });

    entries
        .last()
        .map(|e| e.path())
        .ok_or_else(|| "No run results found".to_string())
}

/// Resolve a run ID or tag to a directory path.
fn resolve_run_dir(results_dir: &PathBuf, run_id_or_tag: &str) -> Result<PathBuf, String> {
    // First try as directory name
    let dir = results_dir.join(run_id_or_tag);
    if dir.join("results.json").exists() {
        return Ok(dir);
    }

    // Try as tag reference file
    let tag_path = results_dir.join(format!("{}.tag", run_id_or_tag));
    if tag_path.exists() {
        let resolved = std::fs::read_to_string(&tag_path)
            .map_err(|e| format!("Failed to read tag file: {e}"))?;
        let tag = resolved.trim();
        let tag_dir = results_dir.join(tag);
        if tag_dir.join("results.json").exists() {
            return Ok(tag_dir);
        }
    }

    Err(format!(
        "Run '{}' not found in {}",
        run_id_or_tag,
        results_dir.display()
    ))
}

/// Main analysis entry point.
pub async fn analyze(
    eval_dir: PathBuf,
    run_id: Option<String>,
    compare_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let results_dir = eval_dir.join("results");

    let run_dir = if let Some(ref id) = run_id {
        resolve_run_dir(&results_dir, id)?
    } else {
        find_latest_run(&results_dir)?
    };

    let _run_name = run_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let result = load_run_result(&run_dir)?;

    let baseline = if let Some(ref compare) = compare_id {
        match resolve_run_dir(&results_dir, compare) {
            Ok(dir) => Some(load_run_result(&dir)?),
            Err(e) => {
                eprintln!("Warning: could not load baseline: {e}");
                None
            }
        }
    } else {
        None
    };

    let analysis = compute_analysis(&result, baseline.as_ref());

    // Write analysis to disk
    let analysis_path = run_dir.join("analysis.json");
    let analysis_json = serde_json::to_string_pretty(&analysis)?;
    std::fs::write(&analysis_path, &analysis_json)?;
    eprintln!("Analysis saved to: {}", analysis_path.display());

    // Generate and print report
    report::print_analysis(&analysis);
    println!("{}", analysis_json);

    Ok(())
}

/// Analyze by comparing two tagged runs.
pub async fn analyze_with_tags(
    repo_root: &PathBuf,
    baseline_tag: &str,
    mem_tag: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results_dir = repo_root.join("evaluation").join("results");

    let baseline_dir = resolve_run_dir(&results_dir, baseline_tag)?;
    let mem_dir = resolve_run_dir(&results_dir, mem_tag)?;

    let baseline_name = baseline_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mem_name = mem_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let baseline = load_run_result(&baseline_dir)?;
    let mem = load_run_result(&mem_dir)?;

    eprintln!("═══ Cross-Run Analysis ═══");
    eprintln!(
        "  Baseline: {} ({:.1}% pass rate)",
        baseline_name, baseline.pass_rate
    );
    eprintln!(
        "  With memory: {} ({:.1}% pass rate)",
        mem_name, mem.pass_rate
    );
    eprintln!();

    let analysis = compute_analysis(&mem, Some(&baseline));

    let analysis_path = mem_dir.join("analysis.json");
    let analysis_json = serde_json::to_string_pretty(&analysis)?;
    std::fs::write(&analysis_path, &analysis_json)?;
    eprintln!("Analysis saved to: {}", analysis_path.display());

    report::print_analysis(&analysis);
    println!("{}", analysis_json);

    Ok(())
}

/// Compute the full analysis from a run result, optionally comparing to a baseline.
fn compute_analysis(result: &RunResult, baseline: Option<&RunResult>) -> Analysis {
    // Build success matrix with comparison
    let success_matrix: Vec<SuccessRow> = result
        .tasks
        .iter()
        .map(|t| {
            let baseline_passed = baseline.and_then(|b| {
                b.tasks
                    .iter()
                    .find(|bt| bt.instance_id == t.instance_id)
                    .map(|bt| bt.passed)
            });

            let regressed = baseline_passed.map(|bp| bp && !t.passed);
            let improved = baseline_passed.map(|bp| !bp && t.passed);

            SuccessRow {
                instance_id: t.instance_id.clone(),
                category: t.category.clone(),
                difficulty: t.difficulty,
                passed: t.passed,
                status: t.status.clone(),
                baseline_passed,
                regressed,
                improved,
                error_class: t.error_class.clone(),
                duration_ms: t.duration_ms,
            }
        })
        .collect();

    // Failure clusters by (error_class, category)
    let mut cluster_map: HashMap<(String, String), Vec<&SuccessRow>> = HashMap::new();
    for row in &success_matrix {
        if !row.passed {
            let cls = row
                .error_class
                .clone()
                .unwrap_or_else(|| "other".to_string());
            cluster_map
                .entry((cls, row.category.clone()))
                .or_default()
                .push(row);
        }
    }

    let failure_clusters: Vec<FailureCluster> = cluster_map
        .into_iter()
        .map(|((error_class, category), rows)| {
            let task_ids: Vec<String> = rows.iter().map(|r| r.instance_id.clone()).collect();
            let avg_difficulty =
                rows.iter().map(|r| r.difficulty as f64).sum::<f64>() / rows.len() as f64;
            let representative_reason = rows.first().and_then(|r| {
                result
                    .tasks
                    .iter()
                    .find(|t| t.instance_id == r.instance_id)
                    .and_then(|t| t.failure_reason.clone())
            });

            FailureCluster {
                error_class,
                category,
                count: rows.len(),
                task_ids,
                avg_difficulty,
                representative_reason,
            }
        })
        .collect();

    // Category breakdown
    let mut cat_map: HashMap<String, Vec<&SuccessRow>> = HashMap::new();
    for row in &success_matrix {
        cat_map.entry(row.category.clone()).or_default().push(row);
    }
    let mut category_breakdown: Vec<CategoryBreakdown> = cat_map
        .into_iter()
        .map(|(category, rows)| {
            let total = rows.len();
            let passed = rows.iter().filter(|r| r.passed).count();
            CategoryBreakdown {
                pass_rate: if total > 0 {
                    passed as f64 / total as f64 * 100.0
                } else {
                    0.0
                },
                category,
                total,
                passed,
            }
        })
        .collect();
    category_breakdown.sort_by(|a, b| b.pass_rate.partial_cmp(&a.pass_rate).unwrap());

    // Difficulty breakdown
    let mut diff_map: HashMap<u8, Vec<&SuccessRow>> = HashMap::new();
    for row in &success_matrix {
        diff_map.entry(row.difficulty).or_default().push(row);
    }
    let mut difficulty_breakdown: Vec<DifficultyBreakdown> = diff_map
        .into_iter()
        .map(|(difficulty, rows)| {
            let total = rows.len();
            let passed = rows.iter().filter(|r| r.passed).count();
            DifficultyBreakdown {
                pass_rate: if total > 0 {
                    passed as f64 / total as f64 * 100.0
                } else {
                    0.0
                },
                difficulty,
                total,
                passed,
            }
        })
        .collect();
    difficulty_breakdown.sort_by_key(|d| d.difficulty);

    // Memory analysis
    let memory_analysis = if result.mode == "with-memory" {
        let repo_root = PathBuf::from(".");
        let memory = sruja_agent::AgenticMemory::load(&repo_root).ok();
        memory.map(|m| {
            let guardrails = m
                .learnings
                .iter()
                .filter(|l| l.kind == Some(sruja_agent::LearningKind::Guardrail))
                .count();
            let playbooks = m
                .learnings
                .iter()
                .filter(|l| l.kind == Some(sruja_agent::LearningKind::Playbook))
                .count();
            let invariants = m
                .learnings
                .iter()
                .filter(|l| l.kind == Some(sruja_agent::LearningKind::Invariant))
                .count();

            let avg_retrieval = if m.learnings.is_empty() {
                0.0
            } else {
                m.learnings
                    .iter()
                    .map(|l| l.retrieval_count as f64)
                    .sum::<f64>()
                    / m.learnings.len() as f64
            };

            let total_with_outcomes: usize = m
                .learnings
                .iter()
                .filter(|l| l.task_total_after > 0)
                .count();
            let avg_utility = if total_with_outcomes > 0 {
                Some(
                    m.learnings
                        .iter()
                        .filter(|l| l.task_total_after > 0)
                        .map(|l| l.task_success_after as f64 / l.task_total_after as f64)
                        .sum::<f64>()
                        / total_with_outcomes as f64,
                )
            } else {
                None
            };

            let low_utility = m.low_utility_entries(3, 0.25).len();

            MemoryAnalysis {
                total_learnings: m.learnings.len(),
                guardrails,
                playbooks,
                invariants,
                avg_retrieval_count: avg_retrieval,
                avg_utility_ratio: avg_utility,
                low_utility_count: low_utility,
            }
        })
    } else {
        None
    };

    // Generate recommendations
    let recommendations =
        generate_recommendations(&success_matrix, &failure_clusters, &memory_analysis);

    Analysis {
        run_id: result.run_id.clone(),
        run_tag: result.tag.clone(),
        baseline_id: baseline.map(|b| b.run_id.clone()),
        baseline_tag: baseline.map(|b| b.tag.clone()),
        success_matrix,
        pass_rate: result.pass_rate,
        baseline_pass_rate: baseline.map(|b| b.pass_rate),
        failure_clusters,
        category_breakdown,
        difficulty_breakdown,
        memory_analysis,
        recommendations,
        total_duration_ms: result.total_duration_ms,
        avg_duration_per_task_ms: if result.total_tasks > 0 {
            result.total_duration_ms as f64 / result.total_tasks as f64
        } else {
            0.0
        },
    }
}

/// Generate improvement recommendations from analysis data.
fn generate_recommendations(
    success_matrix: &[SuccessRow],
    failure_clusters: &[FailureCluster],
    memory_analysis: &Option<MemoryAnalysis>,
) -> Vec<Recommendation> {
    let mut recommendations = Vec::new();

    // 1. Check for regressions
    let regressions: Vec<&SuccessRow> = success_matrix
        .iter()
        .filter(|r| r.regressed == Some(true))
        .collect();
    if !regressions.is_empty() {
        let task_list: Vec<String> = regressions.iter().map(|r| r.instance_id.clone()).collect();
        recommendations.push(Recommendation {
            priority: "critical".to_string(),
            area: "regression".to_string(),
            title: format!("{} task(s) regressed", regressions.len()),
            description: format!(
                "Previously passing tasks now fail: {}. This indicates a regression in the agent's ability.",
                task_list.join(", ")
            ),
            evidence: format!(
                "Tasks: {}",
                serde_json::to_string(&task_list).unwrap_or_default()
            ),
            suggested_action: "Review the trajectory files for these tasks to identify what changed. Consider reverting recent changes to the agent loop or memory injection logic.".to_string(),
        });
    }

    // 2. Check for error class patterns
    let mut error_class_counts: HashMap<String, usize> = HashMap::new();
    for cluster in failure_clusters {
        *error_class_counts
            .entry(cluster.error_class.clone())
            .or_default() += cluster.count;
    }
    for (cls, count) in &error_class_counts {
        if *count >= 2 {
            let (area, action) = match cls.as_str() {
                "compilation" => (
                    "execution",
                    "Add a `cargo check` verification step before the agent attempts compilation. Pre-load key type definitions into the comprehension prompt.",
                ),
                "test" => (
                    "verification",
                    "Improve the test-author phase to generate more targeted tests. Add test output parsing to detect specific assertion failures and inject them into the replanning prompt.",
                ),
                "type" => (
                    "planning",
                    "Add type-aware system hints to the agent's comprehension phase. Pre-load type definitions for all files in the task's scope.",
                ),
                "runtime" => (
                    "execution",
                    "Add a system hint about checking for unwrap() calls and index bounds before running. Consider adding a pre-flight safety check.",
                ),
                "architecture" => (
                    "critique",
                    "Strengthen the architecture critic persona to detect boundary violations earlier. Add a pre-change drift check step.",
                ),
                "spec_gap" => (
                    "comprehension",
                    "Improve the comprehension phase to extract all acceptance criteria explicitly. Add a checklist step before planning.",
                ),
                _ => (
                    "general",
                    "Review the agent trajectory for these failures to identify common patterns. Consider adding task-specific system hints.",
                ),
            };
            recommendations.push(Recommendation {
                priority: if *count >= 3 { "critical".to_string() } else { "high".to_string() },
                area: area.to_string(),
                title: format!("{} failures classified as '{}'", count, cls),
                description: format!(
                    "{} task(s) failed with '{}' error classification. This is the most common failure mode.",
                    count, cls
                ),
                evidence: format!("Error class '{}' appears {} times", cls, count),
                suggested_action: action.to_string(),
            });
        }
    }

    // 3. Check category-specific issues
    let mut cat_failures: HashMap<String, usize> = HashMap::new();
    for row in success_matrix.iter().filter(|r| !r.passed) {
        *cat_failures.entry(row.category.clone()).or_default() += 1;
    }
    for (cat, count) in &cat_failures {
        let total = success_matrix.iter().filter(|r| &r.category == cat).count();
        if total > 0 && (*count as f64 / total as f64) > 0.5 {
            recommendations.push(Recommendation {
                priority: "high".to_string(),
                area: "routing".to_string(),
                title: format!("'{}' tasks fail {}% of the time", cat, (count * 100 / total)),
                description: format!(
                    "{} out of {} tasks in category '{}' failed. This category needs routing or prompt improvements.",
                    count, total, cat
                ),
                evidence: format!("Category '{}': {}/{} failed", cat, count, total),
                suggested_action: format!(
                    "Consider adding a specialized system hint for '{}' tasks, or routing them through a different prompt template.",
                    cat
                ),
            });
        }
    }

    // 4. Memory health recommendations
    if let Some(ref mem) = memory_analysis {
        if mem.total_learnings == 0 {
            recommendations.push(Recommendation {
                priority: "medium".to_string(),
                area: "memory".to_string(),
                title: "No learnings recorded in memory".to_string(),
                description: "The agent has no learnings to draw from. This means each task starts from scratch with no accumulated knowledge.".to_string(),
                evidence: "Memory is empty (0 entries)".to_string(),
                suggested_action: "Run `eval-runner run --with-memory` multiple times to build up a corpus of learnings. Review generated learnings for quality.".to_string(),
            });
        } else if mem.low_utility_count > mem.total_learnings / 2 {
            recommendations.push(Recommendation {
                priority: "medium".to_string(),
                area: "curation".to_string(),
                title: "{}% of learnings have low utility".to_string(),
                description: format!(
                    "{} out of {} learnings are low-utility (retrieved 3+ times with <25% success rate). These entries may be misleading.",
                    mem.low_utility_count, mem.total_learnings
                ),
                evidence: format!("{}/{} low-utility learnings", mem.low_utility_count, mem.total_learnings),
                suggested_action: "Run `sruja agent curate` to prune low-utility entries and archive stale ones. Consider manually reviewing guardrails for accuracy.".to_string(),
            });
        }
    }

    // 5. Specific task recommendations
    for row in success_matrix.iter().filter(|r| !r.passed) {
        let error_cls = row
            .error_class
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        recommendations.push(Recommendation {
            priority: if row.baseline_passed == Some(true) {
                "critical".to_string()
            } else {
                "medium".to_string()
            },
            area: "task-specific".to_string(),
            title: format!("Failed: {} (difficulty {})", row.instance_id, row.difficulty),
            description: format!(
                "Task '{}' ({}) failed with error class '{}'.",
                row.instance_id, row.category, error_cls
            ),
            evidence: format!(
                "Duration: {}ms | Error: {}",
                row.duration_ms,
                error_cls
            ),
            suggested_action: format!(
                "Review trajectory for '{}' at .sruja/runs/<run_id>/loop.json. The failure was classified as '{}'. Consider adding task-specific system hints or adjusting the routing for this task category.",
                row.instance_id, error_cls
            ),
        });
    }

    recommendations
}
