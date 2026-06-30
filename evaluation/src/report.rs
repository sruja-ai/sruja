//! Report generation for eval runner analysis.
//!
//! Produces human-readable markdown summaries and structured JSON reports.

use crate::analyze::{Analysis, Recommendation};

/// Print a human-readable analysis report to stderr.
pub fn print_analysis(analysis: &Analysis) {
    eprintln!();
    eprintln!("═══════════════════════════════════════════════════════════");
    eprintln!("  Eval Runner Analysis Report");
    eprintln!("═══════════════════════════════════════════════════════════");
    eprintln!();
    eprintln!("  Run: {} ({})", analysis.run_id, analysis.run_tag);

    if let (Some(ref base_id), Some(ref base_tag)) = (&analysis.baseline_id, &analysis.baseline_tag)
    {
        eprintln!("  Baseline: {} ({})", base_id, base_tag);
    }
    eprintln!();

    // ── Pass rate ─────────────────────────────────────────────────────
    eprintln!("── Pass Rate ──");
    eprintln!("  Current: {:.1}%", analysis.pass_rate);
    if let Some(baseline_rate) = analysis.baseline_pass_rate {
        let diff = analysis.pass_rate - baseline_rate;
        let sign = if diff >= 0.0 { "+" } else { "" };
        eprintln!("  Baseline: {:.1}% ({sign}{:.1}pp)", baseline_rate, diff);
    }
    eprintln!("  Total duration: {}ms", analysis.total_duration_ms);
    eprintln!("  Avg per task: {:.0}ms", analysis.avg_duration_per_task_ms);
    eprintln!();

    // ── Success matrix ───────────────────────────────────────────────
    eprintln!("── Results ──");
    eprintln!(
        "  {:<45} {:<12} {:<6} {:<6} {}",
        "Task", "Category", "Diff", "Status", "Error Class"
    );
    eprintln!("  {}", "-".repeat(90));
    for row in &analysis.success_matrix {
        let status = if row.passed { "PASS" } else { "FAIL" };
        let regression = if row.regressed == Some(true) {
            " (REGRESSED)"
        } else {
            ""
        };
        let improved = if row.improved == Some(true) {
            " (IMPROVED)"
        } else {
            ""
        };
        let extra = format!("{}{}", regression, improved);
        let cls = row.error_class.as_deref().unwrap_or("-");
        eprintln!(
            "  {:<45} {:<12} {:<6} {:<6}{} {}",
            row.instance_id, row.category, row.difficulty, status, extra, cls
        );
    }
    eprintln!();

    // ── Category breakdown ───────────────────────────────────────────
    eprintln!("── By Category ──");
    eprintln!(
        "  {:<20} {:<6} {:<6} {:<8}",
        "Category", "Total", "Pass", "Rate"
    );
    eprintln!("  {}", "-".repeat(45));
    for cat in &analysis.category_breakdown {
        eprintln!(
            "  {:<20} {:<6} {:<6} {:<8}",
            cat.category,
            cat.total,
            cat.passed,
            format!("{:.0}%", cat.pass_rate)
        );
    }
    eprintln!();

    // ── Difficulty breakdown ─────────────────────────────────────────
    eprintln!("── By Difficulty ──");
    eprintln!(
        "  {:<10} {:<6} {:<6} {:<8}",
        "Difficulty", "Total", "Pass", "Rate"
    );
    eprintln!("  {}", "-".repeat(35));
    for diff in &analysis.difficulty_breakdown {
        eprintln!(
            "  {:<10} {:<6} {:<6} {:<8}",
            diff.difficulty,
            diff.total,
            diff.passed,
            format!("{:.0}%", diff.pass_rate)
        );
    }
    eprintln!();

    // ── Failure clusters ─────────────────────────────────────────────
    if !analysis.failure_clusters.is_empty() {
        eprintln!("── Failure Clusters ──");
        for cluster in &analysis.failure_clusters {
            eprintln!(
                "  {} × {}: {} task(s) at avg difficulty {:.1}",
                cluster.error_class, cluster.category, cluster.count, cluster.avg_difficulty
            );
            if let Some(ref reason) = cluster.representative_reason {
                if reason.len() > 120 {
                    eprintln!("    e.g.: {}...", &reason[..120]);
                } else {
                    eprintln!("    e.g.: {}", reason);
                }
            }
        }
        eprintln!();
    }

    // ── Memory analysis ──────────────────────────────────────────────
    if let Some(ref mem) = analysis.memory_analysis {
        eprintln!("── Memory Health ──");
        eprintln!("  Total learnings: {}", mem.total_learnings);
        eprintln!(
            "  Guardrails: {} | Playbooks: {} | Invariants: {}",
            mem.guardrails, mem.playbooks, mem.invariants
        );
        eprintln!("  Avg retrieval count: {:.2}", mem.avg_retrieval_count);
        if let Some(ratio) = mem.avg_utility_ratio {
            eprintln!("  Avg utility ratio: {:.2}%", ratio * 100.0);
        }
        eprintln!("  Low-utility entries: {}", mem.low_utility_count);
        eprintln!();
    }

    // ── Recommendations ──────────────────────────────────────────────
    if !analysis.recommendations.is_empty() {
        eprintln!("── Recommendations ──");
        let mut by_priority: Vec<&Recommendation> = analysis.recommendations.iter().collect();
        by_priority.sort_by_key(|r| match r.priority.as_str() {
            "critical" => 0,
            "high" => 1,
            "medium" => 2,
            "low" => 3,
            _ => 4,
        });

        for rec in &by_priority {
            let icon = match rec.priority.as_str() {
                "critical" => "🔴",
                "high" => "🟡",
                "medium" => "🔵",
                _ => "⚪",
            };
            eprintln!("  {icon} [{}] {}: {}", rec.priority, rec.area, rec.title);
            eprintln!("     {}", rec.description);
            eprintln!("     Action: {}", rec.suggested_action);
            eprintln!();
        }
    }

    eprintln!("═══════════════════════════════════════════════════════════");
}
