//! Event rendering for the agent loop: plan preview (U2) and status bar (U3).
//!
//! Consumes `LoopEvent`s from the agent loop and renders them to stderr.
//! The plan preview surfaces the calibration verdict; the status bar shows
//! live phase progress.

use std::io::{self, IsTerminal, Write};

use sruja_agent::calibration::{Reversibility, Verdict};
use sruja_agent::cognition::loop_event::{LoopEvent, LoopPhase, PlanBrief};

use colored::Colorize;

// ─────────────────────────────────────────────────────────────────────────
// U2: Plan preview
// ─────────────────────────────────────────────────────────────────────────

/// Render the plan preview block to stderr.
///
/// Shows: goal, subtask list, then the AskPlan block (verdict, reason,
/// reversibility, blast radius, confidence, trust, precedent, policy).
pub fn render_plan_preview(brief: &PlanBrief, ask_plan: &sruja_agent::calibration::AskPlan) {
    let mut out = String::new();

    out.push_str(&format!("{}\n", "┌─ Plan Preview ──────────────────────────────".cyan()));
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
    render_verdict_block(&mut out, ask_plan);
    out.push_str(&format!("{}\n", "└──────────────────────────────────────────────".cyan()));

    eprint!("{}", out);
    let _ = io::stderr().flush();
}

fn render_verdict_block(out: &mut String, ask_plan: &sruja_agent::calibration::AskPlan) {
    let label = verdict_label(ask_plan.verdict);
    out.push_str(&format!("│ Verdict: {}\n", label));
    out.push_str(&format!("│ Reason: {}\n", ask_plan.reason));
    out.push_str(&format!(
        "│ Reversibility: {}\n",
        reversibility_label(ask_plan.reversibility)
    ));
    out.push_str(&format!("│ Blast radius: {}\n", ask_plan.blast_radius));

    if let Some(conf) = ask_plan.confidence {
        out.push_str(&format!("│ Confidence: {}%\n", conf));
    }
    if let Some(trust) = ask_plan.trust_level {
        out.push_str(&format!("│ Trust level: {}\n", trust));
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

fn reversibility_label(r: Reversibility) -> &'static str {
    match r {
        Reversibility::OneWay => "one-way door",
        Reversibility::TwoWay => "two-way door",
    }
}

// ─────────────────────────────────────────────────────────────────────────
// U3: Status bar
// ─────────────────────────────────────────────────────────────────────────

/// Tracks status bar state and renders to stderr.
///
/// In TTY mode: rewrites a single line using `\r`.
/// In non-TTY mode: emits one line per phase transition (greppable logs).
pub struct StatusBar {
    is_tty: bool,
    last_phase: Option<LoopPhase>,
    step: usize,
    total: usize,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            is_tty: io::stderr().is_terminal(),
            last_phase: None,
            step: 0,
            total: 0,
        }
    }

    pub fn render(&mut self, event: &LoopEvent) {
        match event {
            LoopEvent::PhaseChanged(phase) => {
                self.last_phase = Some(*phase);
                self.step = 0;
                self.render_phase_line(*phase);
            }
            LoopEvent::StepProgress { step, total, description } => {
                self.step = *step;
                self.total = *total;
                if self.is_tty {
                    self.render_step_tty(description);
                }
            }
            LoopEvent::Done { .. } => {
                self.finish_line();
            }
            _ => {}
        }
    }

    fn render_phase_line(&self, phase: LoopPhase) {
        let (icon, label) = phase_icon_label(phase);

        if self.is_tty {
            // Clear line and write phase
            eprint!("\r{} {}            ", icon, label);
            let _ = io::stderr().flush();
        } else {
            // Non-TTY: one line per phase, no carriage returns
            eprintln!("{} {}", icon, label);
        }
    }

    fn render_step_tty(&self, description: &str) {
        if let Some(phase) = self.last_phase {
            let (icon, label) = phase_icon_label(phase);
            eprint!(
                "\r{} {} {}/{}: {}            ",
                icon, label, self.step, self.total, description
            );
            let _ = io::stderr().flush();
        }
    }

    /// Clear the status line and start fresh on a new line.
    pub fn finish_line(&mut self) {
        if self.is_tty {
            eprint!("\r{}\r", " ".repeat(60));
            let _ = io::stderr().flush();
        }
        self.last_phase = None;
    }
}

fn phase_icon_label(phase: LoopPhase) -> (&'static str, &'static str) {
    match phase {
        LoopPhase::Comprehend => ("🧠", "Comprehending"),
        LoopPhase::Plan => ("📋", "Planning"),
        LoopPhase::Execute => ("⚡", "Executing"),
        LoopPhase::Critique => ("⚖️", "Critiquing"),
        LoopPhase::Replan => ("🔄", "Replanning"),
        LoopPhase::Verify => ("🔍", "Verifying"),
        LoopPhase::Complete => ("✅", "Done"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verdict_labels_are_distinct() {
        assert_eq!(verdict_label(Verdict::Ask).to_string(), "ASK");
        assert_eq!(verdict_label(Verdict::ProceedSilent).to_string(), "PROCEED");
        assert!(verdict_label(Verdict::ProceedAndFlag).to_string().contains("flagged"));
        assert!(verdict_label(Verdict::ProceedCitingPrecedent).to_string().contains("precedent"));
    }

    #[test]
    fn reversibility_labels() {
        assert_eq!(reversibility_label(Reversibility::OneWay), "one-way door");
        assert_eq!(reversibility_label(Reversibility::TwoWay), "two-way door");
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
        let icons: Vec<&str> = phases.iter().map(|p| phase_icon_label(*p).0).collect();
        let unique: Vec<&str> = {
            let mut s = icons.clone();
            s.sort();
            s.dedup();
            s
        };
        assert_eq!(icons.len(), unique.len(), "all phase icons should be unique");
    }
}
