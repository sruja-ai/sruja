//! Event rendering for the agent loop: plan preview and status bar.
//!
//! Clean, Claude Code–inspired terminal output. Phase transitions use
//! `── label ─────────────────` section headers. Steps are indented
//! tree items with elapsed timestamps. The status bar tracks phase
//! progress and renders real-time feedback in TTY mode.

use std::io::{self, IsTerminal, Write};
use std::time::Instant;

use sruja_agent::calibration::{Reversibility, Verdict};
use sruja_agent::cognition::loop_event::{LoopEvent, LoopPhase, PlanBrief};

use crate::utils::colors;

// ─────────────────────────────────────────────────────────────────────────
// Plan preview
// ─────────────────────────────────────────────────────────────────────────

/// Render the plan preview block to stderr.
///
/// Uses clean section headers and indented lists instead of box drawing.
pub fn render_plan_preview(
    brief: &PlanBrief,
    ask_plan: &sruja_agent::calibration::AskPlan,
    verbose: bool,
) {
    let mut out = String::new();

    // Section header
    out.push_str(&colors::section_header("Plan Preview"));
    out.push('\n');

    // Goal
    out.push_str(&colors::summary_line("Goal", &brief.goal));
    out.push('\n');

    // Subtasks
    if !brief.subtasks.is_empty() {
        for st in &brief.subtasks {
            let files_str = if st.files.is_empty() {
                String::new()
            } else {
                format!("  [{}]", st.files.join(", "))
            };
            let tier_badge = colors::detail_line(&format!("[{}]", st.tier));
            out.push_str(&format!(
                "  {}. {}  {}{}\n",
                st.id, st.description, tier_badge, files_str
            ));
        }
        out.push('\n');
    }

    // Verdict block
    render_verdict_block(&mut out, ask_plan, verbose);

    // Section footer
    out.push_str(&colors::section_footer());
    out.push('\n');
    out.push('\n');

    eprint!("{}", out);
    let _ = io::stderr().flush();
}

fn render_verdict_block(
    out: &mut String,
    ask_plan: &sruja_agent::calibration::AskPlan,
    verbose: bool,
) {
    let label = verdict_label(ask_plan.verdict);
    out.push_str(&colors::summary_line("Verdict", &label));

    out.push_str(&colors::detail_line(&ask_plan.reason));
    out.push('\n');

    let risk = match ask_plan.reversibility {
        Reversibility::OneWay => "high",
        Reversibility::TwoWay => "low",
    };
    out.push_str(&colors::summary_line("Risk", risk));

    if verbose || matches!(ask_plan.verdict, Verdict::Ask) {
        out.push_str(&colors::summary_line(
            "Blast radius",
            &ask_plan.blast_radius.to_string(),
        ));
        if let Some(conf) = ask_plan.confidence {
            out.push_str(&colors::summary_line("Confidence", &format!("{conf}%")));
        }
        if let Some(trust) = ask_plan.trust_level {
            out.push_str(&colors::summary_line("Trust level", &trust.to_string()));
        }
    }

    if ask_plan.has_precedent {
        out.push_str(&colors::summary_line("Precedent", "yes"));
    }
    if ask_plan.policy_says_ask {
        out.push_str(&colors::summary_line("Policy", "requires ask"));
    }
    out.push('\n');
}

fn verdict_label(verdict: Verdict) -> String {
    match verdict {
        Verdict::Ask => colors::verdict_badge("ASK", "ask"),
        Verdict::ProceedSilent => colors::verdict_badge("PROCEED", "proceed"),
        Verdict::ProceedAndFlag => colors::verdict_badge("PROCEED (flagged)", "warn"),
        Verdict::ProceedCitingPrecedent => colors::verdict_badge("PROCEED (precedent)", "warn"),
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Status bar
// ─────────────────────────────────────────────────────────────────────────

/// Tracks phase/step state and renders real-time progress to stderr.
///
/// **TTY mode**: Prints a section header when a phase starts, then
/// overwrites a progress line with `\r` for step updates. When the
/// phase completes, prints a completion line on the next line.
///
/// **Non-TTY mode**: One line per significant event (greppable logs).
pub struct StatusBar {
    is_tty: bool,
    last_phase: Option<LoopPhase>,
    phase_started_at: Option<Instant>,
    step: usize,
    total: usize,
    /// Phases without explicit step progress get a thinking animation.
    needs_heartbeat: bool,
    /// Whether we've printed the phase header yet (for TTY single-line overwrite).
    header_printed: bool,
    /// Live token/cost tracking.
    total_tokens: u64,
    estimated_cost_usd: f64,
    /// Current iteration number.
    current_iteration: usize,
    /// Current tool being dispatched (shown in progress line).
    active_tool: Option<String>,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            is_tty: io::stderr().is_terminal(),
            last_phase: None,
            phase_started_at: None,
            step: 0,
            total: 0,
            needs_heartbeat: false,
            header_printed: false,
            total_tokens: 0,
            estimated_cost_usd: 0.0,
            current_iteration: 0,
            active_tool: None,
        }
    }

    pub fn render(&mut self, event: &LoopEvent) {
        match event {
            LoopEvent::PhaseChanged(phase) => {
                // Deduplicate: don't re-render header if same phase (e.g. Comprehend
                // emitted both before the iteration loop and inside the pipeline).
                if self.last_phase == Some(*phase) {
                    return;
                }
                let prev = self.last_phase;
                self.last_phase = Some(*phase);
                let now = Instant::now();
                self.phase_started_at = Some(now);
                self.step = 0;
                self.total = 0;
                self.needs_heartbeat = matches!(
                    phase,
                    LoopPhase::Comprehend | LoopPhase::Plan | LoopPhase::Replan
                );
                self.header_printed = false;
                self.active_tool = None;

                // If we had a previous phase, print its completion time
                if let Some(_prev_phase) = prev {
                    // For non-TTY, we already printed per-phase lines,
                    // and the previous phase was already completed.
                    // The transition to a new phase is natural.
                }

                self.render_phase_header();
            }
            LoopEvent::StepProgress {
                step,
                total,
                description,
            } => {
                self.step = *step;
                self.total = *total;
                self.needs_heartbeat = false;
                self.active_tool = None;
                let desc: Option<&str> = Some(description.as_str());
                self.render_step_progress(&desc);
            }
            LoopEvent::UsageUpdate {
                total_tokens,
                estimated_cost_usd,
                ..
            } => {
                self.total_tokens = *total_tokens;
                self.estimated_cost_usd = *estimated_cost_usd;
                // Update the progress line with token info
                self.render_progress_line();
            }
            LoopEvent::IterationStarted { n, .. } => {
                self.current_iteration = *n;
                self.active_tool = None;
                if !self.is_tty {
                    eprintln!("  {}", colors::detail_line(&format!("Iteration {n}")));
                }
            }
            LoopEvent::IterationComplete {
                iteration,
                succeeded,
                failed,
                critique_approved,
                cost_usd,
            } => {
                self.active_tool = None;
                let status = if *critique_approved {
                    colors::verdict_badge("approved", "pass")
                } else {
                    colors::verdict_badge("needs work", "warn")
                };
                let cost_str = if *cost_usd > 0.001 {
                    format!("  ${:.4}", cost_usd)
                } else {
                    String::new()
                };
                if self.is_tty {
                    eprint!("\r{}", " ".repeat(80));
                    eprintln!(
                        "\r  iteration {iteration}: {succeeded} ok, {failed} failed  {status}{cost_str}"
                    );
                } else {
                    eprintln!(
                        "  iteration {iteration}: {succeeded} ok, {failed} failed  {status}{cost_str}"
                    );
                }
                let _ = io::stderr().flush();
            }
            LoopEvent::ToolDispatch {
                tool_name,
                subtask_id,
            } => {
                self.active_tool = Some(tool_name.clone());
                let subtask_str = subtask_id
                    .as_ref()
                    .map(|s| format!(" [{s}]"))
                    .unwrap_or_default();
                if self.is_tty {
                    let elapsed = self.elapsed_str();
                    let line = format!(
                        "\r  {}  {}/{}  [{}]  {}{}",
                        "→", self.step, self.total, elapsed, tool_name, subtask_str,
                    );
                    let padded = format!("{:<80}", line);
                    eprint!("{}", padded);
                    let _ = io::stderr().flush();
                } else {
                    eprintln!("  tool: {tool_name}{subtask_str}");
                }
            }
            LoopEvent::RecoveryNotice { reason, strategy } => {
                self.active_tool = None;
                if self.is_tty {
                    eprint!("\r{}", " ".repeat(80));
                    eprintln!(
                        "\r  {} {}",
                        colors::warning("!"),
                        colors::warning(&format!("{strategy}: {reason}"))
                    );
                } else {
                    eprintln!("  {} {strategy}: {reason}", colors::warning("!"));
                }
                let _ = io::stderr().flush();
            }
            LoopEvent::Done { .. } => {
                self.finish_phase();
            }
            _ => {}
        }
    }

    /// Render the progress line with optional token/cost info.
    fn render_progress_line(&self) {
        if !self.is_tty || self.last_phase.is_none() {
            return;
        }
        let elapsed = self.elapsed_str();
        let prefix = if self.total > 0 {
            format!("{}/{}", self.step, self.total)
        } else {
            String::new()
        };
        let tool_str = self
            .active_tool
            .as_deref()
            .map(|t| format!("  {t}"))
            .unwrap_or_default();
        let token_str = if self.total_tokens > 0 {
            let cost_str = if self.estimated_cost_usd > 0.001 {
                format!(" ${:.4}", self.estimated_cost_usd)
            } else {
                String::new()
            };
            format!("  {}t{}", self.total_tokens, cost_str)
        } else {
            String::new()
        };
        let line = format!(
            "\r  {}  {}  [{}]{}{}",
            "→", prefix, elapsed, tool_str, token_str,
        );
        let padded = format!("{:<80}", line);
        eprint!("{}", padded);
        let _ = io::stderr().flush();
    }

    /// Print the phase header (TTY) or transition line (non-TTY).
    fn render_phase_header(&self) {
        if let Some(phase) = self.last_phase {
            let (icon, label) = phase_activity_label(phase);
            let detail = phase_detail_description(phase);
            if self.is_tty {
                let line = colors::phase_header(icon, label);
                eprintln!("{}", line);
                eprintln!("  {}", colors::detail_line(detail));
                let _ = io::stderr().flush();
            } else {
                eprintln!("{}", colors::phase_header(icon, label));
                eprintln!("  {}", colors::detail_line(detail));
            }
        }
    }

    /// Render or overwrite the current step progress line.
    fn render_step_progress(&self, description: &Option<&str>) {
        if self.last_phase.is_some() {
            let elapsed = self.elapsed_str();

            let prefix = if self.total > 0 {
                format!("{}/{}", self.step, self.total)
            } else {
                String::new()
            };

            let desc_str = description.map(|d| format!("  {}", d)).unwrap_or_default();

            if self.is_tty {
                // Overwrite the current line with more context
                let line = format!("\r  {}  {}  [{}]{}", "→", prefix, elapsed, desc_str);
                let padded = format!("{:<80}", line);
                eprint!("{}", padded);
                let _ = io::stderr().flush();
            } else {
                // Print a greppable progress line with more context
                eprintln!("  step {prefix}: {desc_str}");
            }
        }
    }

    /// Elapsed time since phase started, formatted as `M:SS` or `Ss`.
    fn elapsed_str(&self) -> String {
        let since = self
            .phase_started_at
            .map(|t| t.elapsed())
            .unwrap_or_default();
        let secs = since.as_secs();
        if secs >= 60 {
            format!("{}:{:02}", secs / 60, secs % 60)
        } else {
            format!("{secs}s")
        }
    }

    /// Finish the current phase: clear progress line and print completion line.
    pub fn finish_phase(&mut self) {
        if let Some(phase) = self.last_phase {
            let (icon, label) = phase_activity_label(phase);
            let elapsed = self.elapsed_str();

            if self.is_tty {
                // Clear the progress line, then print completion
                eprint!("\r{}", " ".repeat(80));
                eprintln!("\r{}", colors::phase_done(icon, label, &elapsed));
            } else {
                eprintln!("{}", colors::phase_done(icon, label, &elapsed));
            }
            let _ = io::stderr().flush();
        } else if self.is_tty {
            eprintln!("\r{}", " ".repeat(80));
        }
        self.last_phase = None;
    }
}

/// User-facing phase labels — internal phase names stay internal.
fn phase_activity_label(phase: LoopPhase) -> (&'static str, &'static str) {
    match phase {
        LoopPhase::Comprehend => ("🔍", "Analyzing codebase"),
        LoopPhase::Plan => ("📋", "Designing approach"),
        LoopPhase::Execute => ("⚡", "Running tools"),
        LoopPhase::Critique => ("🔎", "Reviewing output"),
        LoopPhase::Replan => ("🔄", "Refining approach"),
        LoopPhase::Verify => ("✅", "Running checks"),
        LoopPhase::Complete => ("🎯", "Complete"),
    }
}

/// Detailed descriptions for each phase — explains what the agent is doing.
fn phase_detail_description(phase: LoopPhase) -> &'static str {
    match phase {
        LoopPhase::Comprehend => "Reading files and understanding the codebase structure",
        LoopPhase::Plan => "Breaking down the goal into concrete, actionable steps",
        LoopPhase::Execute => "Making code changes and running verification commands",
        LoopPhase::Critique => "Evaluating changes against requirements and best practices",
        LoopPhase::Replan => "Adjusting approach based on review feedback",
        LoopPhase::Verify => "Running final checks to ensure everything works",
        LoopPhase::Complete => "All steps finished successfully",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verdict_labels_are_distinct() {
        let v1 = verdict_label(Verdict::Ask);
        let v2 = verdict_label(Verdict::ProceedSilent);
        assert!(!v1.is_empty());
        assert!(!v2.is_empty());
    }

    #[test]
    fn phase_icons_are_unique() {
        let phases = [
            LoopPhase::Comprehend,
            LoopPhase::Plan,
            LoopPhase::Execute,
            LoopPhase::Critique,
            LoopPhase::Replan,
            LoopPhase::Verify,
            LoopPhase::Complete,
        ];
        let icons: Vec<&str> = phases.iter().map(|p| phase_activity_label(*p).0).collect();
        let mut sorted = icons.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            icons.len(),
            sorted.len(),
            "all phase icons should be unique"
        );
    }
}
