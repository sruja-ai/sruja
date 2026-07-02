//! Live dashboard hook for the agent loop.
//!
//! Writes a markdown report to `.sruja/runs/<run_id>/LIVE.md` after every
//! meaningful event so the user can tail -f the file for progress.
//! Also supports interactive steering (pause/stop/report between iterations).

use std::sync::Mutex;

use async_trait::async_trait;
use sruja_agent::cognition::{Hook, HookAction};
use sruja_agent::{AgentError, LoopIteration};

use crate::utils::colors;

/// Phase names for the live report.
fn phase_name(step: &sruja_agent::cognition::Subtask) -> &'static str {
    match step.kind {
        sruja_agent::cognition::SubtaskKind::Comprehend => "comprehend",
        sruja_agent::cognition::SubtaskKind::TestAuthor => "test-author",
        sruja_agent::cognition::SubtaskKind::Implement => "implement",
        sruja_agent::cognition::SubtaskKind::Verify => "verify",
        sruja_agent::cognition::SubtaskKind::AdversarialTest => "adversarial-test",
        sruja_agent::cognition::SubtaskKind::Review => "review",
    }
}

/// Mutable state accumulated across hook calls.
struct ReportState {
    goal: String,
    started_at: std::time::Instant,
    iteration: usize,
    max_iterations: usize,
    current_phase: String,
    subtasks: Vec<SubtaskInfo>,
    critique_score: Option<f64>,
    critique_approved: Option<bool>,
    persona_results: Vec<PersonaInfo>,
    issues: Vec<String>,
    verify_failures: Vec<String>,
    cost_usd: f64,
    steer: bool,
    report_dir: std::path::PathBuf,
    should_stop: bool,
    dirty: bool,
}

#[derive(Clone)]
struct SubtaskInfo {
    id: String,
    description: String,
    kind: String,
    tier: String,
    status: String,
}

#[derive(Clone)]
struct PersonaInfo {
    id: String,
    approved: bool,
    score: f64,
    issue_count: usize,
}

/// A hook that writes a live markdown dashboard and optionally prompts for
/// steering between iterations.
pub(crate) struct LiveReportHook {
    state: Mutex<ReportState>,
}

impl LiveReportHook {
    pub(crate) fn new(
        goal: &str,
        max_iterations: usize,
        steer: bool,
        report_dir: std::path::PathBuf,
    ) -> Self {
        Self {
            state: Mutex::new(ReportState {
                goal: goal.to_string(),
                started_at: std::time::Instant::now(),
                iteration: 0,
                max_iterations,
                current_phase: "starting".into(),
                subtasks: Vec::new(),
                critique_score: None,
                critique_approved: None,
                persona_results: Vec::new(),
                issues: Vec::new(),
                verify_failures: Vec::new(),
                cost_usd: 0.0,
                steer,
                report_dir,
                should_stop: false,
                dirty: false,
            }),
        }
    }

    fn write_report(&self) {
        let mut s = self.state.lock().unwrap();
        if !s.dirty {
            return;
        }
        s.dirty = false;
        let elapsed = s.started_at.elapsed();
        let mins = elapsed.as_secs() / 60;
        let secs = elapsed.as_secs() % 60;

        let status_icon = match s.critique_approved {
            Some(true) => "PASS",
            Some(false) => "FAIL",
            None => "RUN",
        };

        let mut md = String::new();
        md.push_str(&format!(
            "# Agent Loop — Live Dashboard\n\n\
             Goal: {}\n\n\
             Started: {}m {:02}s ago · Iteration {}/{} · Phase: **{}** · Status: **{}** · Cost: ~${:.4}\n\n",
            s.goal,
            mins,
            secs,
            s.iteration,
            s.max_iterations,
            s.current_phase,
            status_icon,
            s.cost_usd,
        ));

        // Subtask table
        if !s.subtasks.is_empty() {
            md.push_str("## Subtasks\n\n");
            md.push_str("| # | Kind | Tier | Status | Description |\n");
            md.push_str("|---|------|------|--------|-------------|\n");
            for st in &s.subtasks {
                md.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    st.id, st.kind, st.tier, st.status, st.description
                ));
            }
            md.push('\n');
        }

        // Critique persona breakdown
        if !s.persona_results.is_empty() {
            md.push_str("## Critique Personas\n\n");
            md.push_str("| Persona | Approved | Score | Issues |\n");
            md.push_str("|---------|----------|-------|--------|\n");
            for p in &s.persona_results {
                let icon = if p.approved { "yes" } else { "NO" };
                md.push_str(&format!(
                    "| {} | {} | {:.1} | {} |\n",
                    p.id, icon, p.score, p.issue_count
                ));
            }
            md.push('\n');
        }

        // Issues
        if !s.issues.is_empty() {
            md.push_str("## Open Issues\n\n");
            for issue in &s.issues {
                md.push_str(&format!("- {issue}\n"));
            }
            md.push('\n');
        }

        // Verify failures
        if !s.verify_failures.is_empty() {
            md.push_str("## Verify Failures (independent grader)\n\n");
            for f in &s.verify_failures {
                md.push_str(&format!("- {f}\n"));
            }
            md.push('\n');
        }

        // Write atomically
        let _ = std::fs::create_dir_all(&s.report_dir);
        let path = s.report_dir.join("LIVE.md");
        let tmp = s.report_dir.join("LIVE.md.tmp");
        if let Err(e) = std::fs::write(&tmp, &md) {
            eprintln!("  Warning: could not write live report: {e}");
            return;
        }
        if let Err(e) = std::fs::rename(&tmp, &path) {
            eprintln!("  Warning: could not rename live report: {e}");
        }
    }

    fn print_summary(&self) {
        let s = self.state.lock().unwrap();
        let mark = match s.critique_approved {
            Some(true) => colors::verdict_badge("PASS", "pass"),
            Some(false) => colors::verdict_badge("FAIL", "fail"),
            None => "---".to_string(),
        };
        let score_str = s
            .critique_score
            .map(|sc| format!("{:.1}", sc))
            .unwrap_or_else(|| "-".into());

        eprintln!(
            "{}",
            colors::summary_line(
                &format!("Iteration {}/{}", s.iteration, s.max_iterations),
                &format!(
                    "{}  {} subtasks  score {}  ~${:.4}",
                    mark,
                    s.subtasks.len(),
                    score_str,
                    s.cost_usd
                ),
            )
        );

        if !s.persona_results.is_empty() {
            for p in &s.persona_results {
                let icon = if p.approved { "✓" } else { "✗" };
                eprintln!(
                    "{}",
                    colors::detail_line(&format!(
                        "[{icon}] {}  score: {:.1}  issues: {}",
                        p.id, p.score, p.issue_count
                    ))
                );
            }
        }

        for issue in &s.issues {
            eprintln!("{}", colors::detail_line(&format!("issue: {issue}")));
        }
        for f in &s.verify_failures {
            eprintln!("{}", colors::detail_line(&format!("verify FAIL: {f}")));
        }
    }

    fn prompt_steer(&self) -> bool {
        let s = self.state.lock().unwrap();
        if !s.steer {
            return true;
        }
        drop(s);

        eprintln!();
        eprintln!("  ── Steering ──");
        eprintln!("  [Enter] continue  ·  [s] stop  ·  [r] show report");
        eprint!("  > ");
        use std::io::Write;
        let _ = std::io::stderr().flush();

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            return true;
        }
        match input.trim().to_lowercase().as_str() {
            "s" | "stop" => false,
            "r" | "report" => {
                let s = self.state.lock().unwrap();
                let path = s.report_dir.join("LIVE.md");
                drop(s);
                if let Ok(content) = std::fs::read_to_string(&path) {
                    eprintln!();
                    eprintln!("{content}");
                }
                true
            }
            _ => true,
        }
    }
}

#[async_trait]
impl Hook for LiveReportHook {
    async fn before_comprehend(&self, _goal: &str) -> HookAction {
        let mut s = self.state.lock().unwrap();
        s.current_phase = "comprehend".into();
        s.iteration = s.iteration.max(1);
        HookAction::Continue
    }

    async fn after_comprehend(
        &self,
        result: &sruja_agent::cognition::Comprehension,
    ) -> HookAction {
        let mut s = self.state.lock().unwrap();
        s.current_phase = "plan".into();
        eprintln!(
            "{}",
            colors::detail_line(&format!(
                "{} elements cited, {} findings",
                result.cited_elements.len(),
                result.key_findings.len()
            ))
        );
        s.dirty = true;
        drop(s);
        self.write_report();
        HookAction::Continue
    }

    async fn before_plan(&self, _goal: &str) -> HookAction {
        let s = self.state.lock().unwrap();
        if s.should_stop {
            eprintln!("  Stopped by user.");
            return HookAction::Abort("Stopped by user.".into());
        }
        HookAction::Continue
    }

    async fn after_plan(&self, plan: &mut sruja_agent::cognition::Plan) -> HookAction {
        let mut s = self.state.lock().unwrap();
        s.current_phase = "execute".into();
        s.subtasks = plan
            .subtasks
            .iter()
            .map(|st| SubtaskInfo {
                id: st.id.clone(),
                description: st.description.chars().take(60).collect(),
                kind: format!("{:?}", st.kind).to_lowercase(),
                tier: format!("{:?}", st.tier).to_lowercase(),
                status: "pending".into(),
            })
            .collect();
        eprintln!(
            "{}",
            colors::detail_line(&format!(
                "{} subtasks, {} risks",
                plan.subtasks.len(),
                plan.risks.len()
            ))
        );
        s.dirty = true;
        drop(s);
        self.write_report();
        HookAction::Continue
    }

    async fn before_step(&self, step: &sruja_agent::cognition::Subtask) -> HookAction {
        let mut s = self.state.lock().unwrap();
        s.current_phase = phase_name(step).into();
        if let Some(st) = s.subtasks.iter_mut().find(|st| st.id == step.id) {
            st.status = "running".into();
        }
        let kind = format!("{:?}", step.tier).to_lowercase();
        let desc_trimmed: String = step.description.chars().take(80).collect();
        eprintln!(
            "{}",
            colors::step_line("→", &step.id, &desc_trimmed, &kind, None)
        );
        s.dirty = true;
        drop(s);
        self.write_report();
        HookAction::Continue
    }

    async fn after_step(
        &self,
        step: &sruja_agent::cognition::Subtask,
        result: &sruja_agent::cognition::StepResult,
    ) {
        let mut s = self.state.lock().unwrap();
        let status = match result.status {
            sruja_agent::cognition::StepStatus::Ok => "done",
            sruja_agent::cognition::StepStatus::Failed => "FAILED",
            sruja_agent::cognition::StepStatus::Skipped => "skipped",
        };
        if let Some(st) = s.subtasks.iter_mut().find(|st| st.id == step.id) {
            st.status = status.into();
        }
        s.cost_usd += result.usage.estimated_cost_usd();
        s.dirty = true;
        drop(s);
        self.write_report();
    }

    async fn before_review(&self) -> HookAction {
        let mut s = self.state.lock().unwrap();
        s.current_phase = "critique".into();
        eprintln!("{}", colors::detail_line("Running persona ensemble..."));
        s.dirty = true;
        drop(s);
        self.write_report();
        HookAction::Continue
    }

    async fn after_review(&self, critique: &sruja_agent::cognition::Critique) -> HookAction {
        let mut s = self.state.lock().unwrap();
        s.current_phase = "done".into();
        s.critique_score = Some(critique.score);
        s.critique_approved = Some(critique.approved);
        s.issues = critique.issues.clone();
        s.cost_usd += critique.usage.estimated_cost_usd();
        s.persona_results = critique
            .persona_breakdown
            .iter()
            .map(|p| PersonaInfo {
                id: p.id.clone(),
                approved: p.approved,
                score: p.score,
                issue_count: p.issues.len(),
            })
            .collect();
        s.dirty = true;
        drop(s);

        self.print_summary();
        self.write_report();

        HookAction::Continue
    }

    async fn before_iteration(&self, iteration: usize, max_iterations: usize) {
        let mut s = self.state.lock().unwrap();
        s.iteration = iteration;
        s.max_iterations = max_iterations;
        s.current_phase = if iteration == 1 {
            "comprehend"
        } else {
            "replan"
        }
        .into();
        s.dirty = true;
        drop(s);
        self.write_report();
    }

    async fn after_iteration(
        &self,
        iteration: usize,
        max_iterations: usize,
        result: &LoopIteration,
    ) {
        {
            let mut s = self.state.lock().unwrap();
            s.iteration = iteration;
            s.max_iterations = max_iterations;
            s.critique_score = Some(result.critique_score);
            s.critique_approved = Some(result.critique_approved);
            s.issues = result.critique_issues.clone();
            s.verify_failures = result.verify_failed.clone();
            s.cost_usd = result.usage.estimated_cost_usd();
            s.dirty = true;
        }
        self.write_report();

        if !self.prompt_steer() {
            let mut s = self.state.lock().unwrap();
            s.should_stop = true;
        }
    }

    async fn on_error(&self, error: &AgentError) {
        eprintln!("  ERROR: {error}");
    }
}
