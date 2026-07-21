//! Output formatting for the agent loop results.
//!
//! Renders a human-readable summary after the agent loop finishes, covering:
//!   - Convergence status (pass / fail badge)
//!   - Narrative list of subtask descriptions ("What I did")
//!   - Files touched during execution
//!   - Per-subtask status breakdown (ok / failed / skipped)
//!   - Verification issues and termination reason when the loop didn't converge
//!   - Token usage and cost summary (when `--show-tokens` / `--verbose`)
//!
//! All colour output is delegated to [`crate::utils::colors`].

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

    // ── Subtask details (succeeded/failed status per subtask) ───────
    if plan.subtasks.len() > 1 {
        println!();
        for st in &plan.subtasks {
            let status = if steps.iter().any(|s| {
                s.subtask_id == st.id && s.status == sruja_agent::cognition::StepStatus::Ok
            }) {
                colors::verdict_badge("ok", "pass")
            } else if steps.iter().any(|s| {
                s.subtask_id == st.id && s.status == sruja_agent::cognition::StepStatus::Failed
            }) {
                colors::verdict_badge("failed", "fail")
            } else if steps.iter().any(|s| {
                s.subtask_id == st.id && s.status == sruja_agent::cognition::StepStatus::Skipped
            }) {
                colors::verdict_badge("skipped", "info")
            } else {
                colors::verdict_badge("—", "info")
            };
            println!(
                "  {} {}. {}",
                status,
                st.id,
                colors::detail_line(&st.description)
            );
        }
    }

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
        // Show termination reason
        let reason = match &result.termination {
            sruja_agent::cognition::LoopTermination::MaxIterations => {
                "Max iterations reached without convergence"
            }
            sruja_agent::cognition::LoopTermination::Oscillation => {
                "Detected repeated failure pattern (oscillation)"
            }
            sruja_agent::cognition::LoopTermination::SpendCapExceeded(cost) => {
                &format!("Budget exceeded (${cost:.4})")
            }
            sruja_agent::cognition::LoopTermination::ModelNotConverging(frac) => {
                &format!("Model not converging ({:.0}% non-converged)", frac * 100.0)
            }
            sruja_agent::cognition::LoopTermination::NoReplan => "No replan strategy available",
            sruja_agent::cognition::LoopTermination::Aborted(msg) => &format!("Aborted: {msg}"),
            _ => "Unknown reason",
        };
        println!();
        println!("{}", colors::warning(&format!("Stopped: {reason}")));
        println!(
            "{}",
            colors::detail_line("Try rephrasing the goal or increasing --max-iterations.")
        );
    }

    // ── Token / cost ───────────────────────────────────────────────
    // Always show cost summary — users need to know what they spent.
    let cost = result.total_usage.estimated_cost_usd();
    let tokens = result.total_usage.total_tokens;
    if tokens > 0 {
        println!();
        println!(
            "{}",
            colors::detail_line(&format!("{tokens} tokens  ·  ${cost:.4}"))
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
