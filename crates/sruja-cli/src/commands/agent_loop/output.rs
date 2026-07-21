//! Output formatting for the agent loop results.

use crate::utils::colors;

/// Print the loop result in human-readable format.
pub(crate) fn print_loop_result_human(result: &sruja_agent::LoopResult, verbose: bool) {
    let plan = &result.final_result.plan;
    let steps = &result.final_result.step_results;

    // ── Collect what was done ──────────────────────────────────────────
    // Count subtasks from plan, not step_results (which includes all stages)
    let succeeded = plan
        .subtasks
        .iter()
        .filter(|st| {
            steps.iter().any(|s| {
                s.subtask_id == st.id && s.status == sruja_agent::cognition::StepStatus::Ok
            })
        })
        .count();
    let failed = plan
        .subtasks
        .iter()
        .filter(|st| {
            steps.iter().any(|s| {
                s.subtask_id == st.id && s.status == sruja_agent::cognition::StepStatus::Failed
            })
        })
        .count();
    let skipped = plan
        .subtasks
        .iter()
        .filter(|st| {
            steps.iter().any(|s| {
                s.subtask_id == st.id && s.status == sruja_agent::cognition::StepStatus::Skipped
            })
        })
        .count();

    let touched_files: Vec<&str> = plan
        .subtasks
        .iter()
        .flat_map(|st| st.files.iter().map(String::as_str))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    // ── Conversational header ──────────────────────────────────────────
    if result.converged {
        println!("{}", colors::verdict_badge("✓ Done", "pass"));
    } else {
        println!(
            "{}",
            colors::verdict_badge("⚠ Didn't converge — partial result below", "fail")
        );
    }
    println!();

    // ── What happened (narrative from subtask descriptions) ────────────
    let mut descriptions: Vec<&str> = plan
        .subtasks
        .iter()
        .map(|st| st.description.as_str())
        .collect();
    descriptions.dedup();

    if !descriptions.is_empty() {
        println!("{}", colors::summary_line("What I did", ""));
        for desc in &descriptions {
            println!("  {} {}", colors::detail_line("•"), desc);
        }
        println!();
    }

    // ── Files touched ─────────────────────────────────────────────────
    if !touched_files.is_empty() {
        if touched_files.len() == 1 {
            println!("Touched {} file:", touched_files.len());
        } else {
            println!("Touched {} files:", touched_files.len());
        }
        for file in &touched_files {
            println!("  {}", colors::detail_line(file));
        }
        println!();
    }

    // ── Step counts ───────────────────────────────────────────────────
    let mut parts: Vec<String> = Vec::new();
    if succeeded > 0 {
        parts.push(format!("{} succeeded", succeeded));
    }
    if failed > 0 {
        parts.push(format!("{} failed", failed));
    }
    if skipped > 0 {
        parts.push(format!("{} skipped", skipped));
    }
    let total_subtasks = plan.subtasks.len();
    println!(
        "{} subtask{}: {}",
        total_subtasks,
        if total_subtasks == 1 { "" } else { "s" },
        parts.join(", ")
    );

    // ── Verification ──────────────────────────────────────────────────
    if result.converged {
        let critique = result.final_result.critique.as_ref();
        if let Some(c) = critique {
            if !c.issues.is_empty() {
                println!();
                println!("{}", colors::detail_line("Issues found:"));
                for issue in &c.issues {
                    println!("  {}", colors::detail_line(&format!("⚠ {}", issue)));
                }
            }
        }
    } else if let Some(critique) = result.final_result.critique.as_ref() {
        if !critique.issues.is_empty() {
            println!();
            println!("{}", colors::detail_line("Remaining issues:"));
            for issue in &critique.issues {
                println!("  {}", colors::detail_line(&format!("• {}", issue)));
            }
        }
        println!();
        println!(
            "{}",
            colors::detail_line("Try rephrasing the goal or increasing --max-iterations.")
        );
    }

    // ── Token / cost (only when verbose) ───────────────────────────────
    if verbose {
        println!();
        println!(
            "{}",
            colors::detail_line(&format!(
                "{} tokens  ·  ${:.4}",
                result.total_usage.total_tokens,
                result.total_usage.estimated_cost_usd()
            ))
        );
    }

    // ── Artifact paths (only when verbose) ─────────────────────────────
    if verbose {
        let run_dir = std::path::Path::new(".sruja").join("runs").join(format!(
            "run_{}",
            result
                .goal
                .chars()
                .take(30)
                .collect::<String>()
                .replace(' ', "_")
        ));
        println!(
            "{}",
            colors::detail_line(&format!("Run data: {}", run_dir.display()))
        );
    }
    println!();
}
