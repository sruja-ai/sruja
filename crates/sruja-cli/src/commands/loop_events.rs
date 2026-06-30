//! Event rendering for the agent loop: plan preview (U2) and status bar (U3).
//!
//! Consumes `LoopEvent`s from the agent loop and renders them to stderr.
//! The plan preview surfaces the calibration verdict; the status bar shows
//! live phase progress.

use std::io::{self, IsTerminal, Write};
use std::time::Instant;

use sruja_agent::calibration::{Reversibility, Verdict};
use sruja_agent::cognition::loop_event::{LoopEvent, LoopPhase, PlanBrief};

use colored::Colorize;

// ─────────────────────────────────────────────────────────────────────────
// U2: Plan preview
// ─────────────────────────────────────────────────────────────────────────

/// Render the plan preview block to stderr.
///
/// Shows: goal, subtask list, then the AskPlan block (verdict, reason,
/// risk level). Internal fields (blast radius, confidence) are shown only
/// when the verdict is Ask or the caller requests verbose mode.
pub fn render_plan_preview(
    brief: &PlanBrief,
    ask_plan: &sruja_agent::calibration::AskPlan,
    verbose: bool,
) {
    let mut out = String::new();

    out.push_str(&format!(
        "{}\n",
        "┌─ Plan Preview ──────────────────────────────".cyan()
    ));
    out.push_str(&format!("│ Goal: {}\n", brief.goal));

    if !brief.subtasks.is_empty() {
        out.push_str("│\n");
        out.push_str(&format!("│ Subtasks ({}):\n", brief.subtasks.len()));
        for st in &brief.subtasks {
            let files_str = if st.files.is_empty() {
                String::new()
            } else {
                format!("  [{}]", st.files.join(", "))
            };
            out.push_str(&format!(
                "│   {}. ({}) {}{}\n",
                st.id, st.tier, st.description, files_str
            ));
        }
    }

    out.push_str(&format!("{}\n", "│".cyan()));
    render_verdict_block(&mut out, ask_plan, verbose);
    out.push_str(&format!(
        "{}\n",
        "└──────────────────────────────────────────────".cyan()
    ));

    eprint!("{}", out);
    let _ = io::stderr().flush();
}

fn render_verdict_block(
    out: &mut String,
    ask_plan: &sruja_agent::calibration::AskPlan,
    verbose: bool,
) {
    let label = verdict_label(ask_plan.verdict);
    out.push_str(&format!("│ Verdict: {}\n", label));
    out.push_str(&format!("│ Reason: {}\n", ask_plan.reason));

    // User-facing risk level instead of internal "reversibility" terminology.
    let risk = match ask_plan.reversibility {
        Reversibility::OneWay => "high",
        Reversibility::TwoWay => "low",
    };
    out.push_str(&format!("│ Risk level: {}\n", risk));

    // Internal fields only for Ask verdicts (human needs to decide) or verbose.
    let show_details = verbose || matches!(ask_plan.verdict, Verdict::Ask);
    if show_details {
        out.push_str(&format!("│ Blast radius: {}\n", ask_plan.blast_radius));
        if let Some(conf) = ask_plan.confidence {
            out.push_str(&format!("│ Confidence: {}%\n", conf));
        }
        if let Some(trust) = ask_plan.trust_level {
            out.push_str(&format!("│ Trust level: {}\n", trust));
        }
    }

    if ask_plan.has_precedent {
        out.push_str(&format!("│ Has precedent: {}\n", "yes".green()));
    }
    if ask_plan.policy_says_ask {
        out.push_str(&format!("│ Policy says ask: {}\n", "yes".yellow()));
    }
}

fn verdict_label(verdict: Verdict) -> colored::ColoredString {
    match verdict {
        Verdict::Ask => "ASK".red(),
        Verdict::ProceedSilent => "PROCEED".green(),
        Verdict::ProceedAndFlag => "PROCEED (flagged)".yellow(),
        Verdict::ProceedCitingPrecedent => "PROCEED (precedent)".yellow(),
    }
}

// ─────────────────────────────────────────────────────────────────────────
// U3: Status bar
// ─────────────────────────────────────────────────────────────────────────

/// Tracks status bar state and renders to stderr.
///
/// In TTY mode: rewrites a single line using `\r`, shows elapsed time,
/// and animates a thinking indicator during phases without step progress.
/// In non-TTY mode: emits one line per phase transition (greppable logs).
pub struct StatusBar {
    is_tty: bool,
    last_phase: Option<LoopPhase>,
    phase_started_at: Option<Instant>,
    step: usize,
    total: usize,
    /// Phases that don't emit step progress (Comprehend/Plan/Replan) get a
    /// thinking animation to show the user something is happening.
    needs_heartbeat: bool,
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
        }
    }

    pub fn render(&mut self, event: &LoopEvent) {
        match event {
            LoopEvent::PhaseChanged(phase) => {
                let prev = self.last_phase;
                self.last_phase = Some(*phase);
                self.phase_started_at = Some(Instant::now());
                self.step = 0;
                self.total = 0;
                self.needs_heartbeat = matches!(
                    phase,
                    LoopPhase::Comprehend | LoopPhase::Plan | LoopPhase::Replan
                );

                // In non-TTY mode, print a completion line for the previous phase.
                if !self.is_tty {
                    if let Some(prev_phase) = prev {
                        let (icon, label) = phase_activity_label(prev_phase);
                        eprintln!("{icon} {label} ... done");
                    }
                }

                self.render_status_line();
            }
            LoopEvent::StepProgress {
                step,
                total,
                description,
            } => {
                self.step = *step;
                self.total = *total;
                self.needs_heartbeat = false; // we have real progress
                if self.is_tty {
                    self.render_status_line_with(Some(description));
                }
            }
            LoopEvent::Done { .. } => {
                self.finish_line();
            }
            _ => {}
        }
    }

    /// Render the status line with the current phase, elapsed time, and
    /// optionally a thinking indicator or detail text.
    fn render_status_line(&self) {
        if let Some(phase) = self.last_phase {
            let (icon, label) = phase_activity_label(phase);
            let elapsed = self.elapsed_str();
            let extra = self.extra_str();

            if self.is_tty {
                // Pad to 60 chars to clear previous content.
                let line = format!("\r{icon} {label}  [{elapsed}]{extra}");
                let padded = format!("{:<60}", line);
                eprint!("{}", padded);
                let _ = io::stderr().flush();
            } else {
                eprintln!("{icon} {label}  [{elapsed}]{extra}");
            }
        }
    }

    /// Render status line with step progress detail.
    fn render_status_line_with(&self, desc: Option<&str>) {
        if let Some(phase) = self.last_phase {
            let (icon, label) = phase_activity_label(phase);
            let elapsed = self.elapsed_str();

            let prefix = if self.total > 0 {
                format!(" {}/{}", self.step, self.total)
            } else {
                String::new()
            };

            let desc_str = desc
                .map(|d| format!("  {d}"))
                .unwrap_or_default();

            let line = format!("\r{icon} {label}{prefix}  [{elapsed}]{desc_str}");
            let padded = format!("{:<60}", line);
            eprint!("{}", padded);
            let _ = io::stderr().flush();
        }
    }

    /// Elapsed time in `M:SS` or `S` format since phase started.
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

    /// Extra content for the status line: thinking dots during idle phases.
    fn extra_str(&self) -> String {
        if self.needs_heartbeat && self.is_tty {
            // Show a thinking animation based on elapsed time since phase start.
            let elapsed = self
                .phase_started_at
                .map(|t| t.elapsed())
                .unwrap_or_default()
                .as_secs();
            let cycle = (elapsed / 2) % 4;
            let dots = match cycle {
                0 => "  .",
                1 => "  ..",
                2 => "  ...",
                _ => "",
            };
            return dots.to_string();
        }
        String::new()
    }

    /// Clear the status line and print the final phase completion message.
    pub fn finish_line(&mut self) {
        if self.is_tty {
            // Print the final status line cleanly, then move to next line.
            if let Some(phase) = self.last_phase {
                let (icon, label) = phase_activity_label(phase);
                let elapsed = self.elapsed_str();
                eprintln!("\r{icon} {label}  [{elapsed}] ✓");
            } else {
                eprintln!("\r{}", " ".repeat(60));
            }
        } else if let Some(phase) = self.last_phase {
            let (icon, label) = phase_activity_label(phase);
            eprintln!("{icon} {label} ... done ✓");
        }
        self.last_phase = None;
    }
}

/// User-facing phase labels — internal phase names stay internal.
fn phase_activity_label(phase: LoopPhase) -> (&'static str, &'static str) {
    match phase {
        LoopPhase::Comprehend => ("🔍", "Analyzing codebase"),
        LoopPhase::Plan => ("📋", "Designing approach"),
        LoopPhase::Execute => ("⚡", "Making changes"),
        LoopPhase::Critique => ("🔎", "Reviewing output"),
        LoopPhase::Replan => ("🔄", "Refining approach"),
        LoopPhase::Verify => ("✅", "Running checks"),
        LoopPhase::Complete => ("🎯", "Complete"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verdict_labels_are_distinct() {
        assert_eq!(verdict_label(Verdict::Ask).to_string(), "ASK");
        assert_eq!(verdict_label(Verdict::ProceedSilent).to_string(), "PROCEED");
        assert!(verdict_label(Verdict::ProceedAndFlag)
            .to_string()
            .contains("flagged"));
        assert!(verdict_label(Verdict::ProceedCitingPrecedent)
            .to_string()
            .contains("precedent"));
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
        let unique: Vec<&str> = {
            let mut s = icons.clone();
            s.sort();
            s.dedup();
            s
        };
        assert_eq!(
            icons.len(),
            unique.len(),
            "all phase icons should be unique"
        );
    }

    #[test]
    fn phase_labels_dont_leak_internal_names() {
        // Old labels that leaked internal phase names — make sure they're gone.
        let old_labels = [
            "Comprehending",
            "Planning",
            "Executing",
            "Critiquing",
            "Replanning",
            "Done",
        ];
        let phases = [
            LoopPhase::Comprehend,
            LoopPhase::Plan,
            LoopPhase::Execute,
            LoopPhase::Critique,
            LoopPhase::Replan,
            LoopPhase::Verify,
        ];
        for phase in &phases {
            let (_, label) = phase_activity_label(*phase);
            for old in &old_labels {
                assert!(
                    !label.contains(old),
                    "label '{label}' still contains old internal name '{old}'"
                );
            }
        }
    }
}
