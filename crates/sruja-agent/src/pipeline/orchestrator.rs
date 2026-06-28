use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use crate::llm::{CompletionRequest, LlmClient, OpenAiClient};

use super::budget::BudgetTracker;
use super::config::{self, PipelineManifest};
use super::lessons::LessonStore;
use super::live_report::LiveReport;
use super::stages::prompts;
use super::types::*;

/// Orchestrates the multi-agent role pipeline.
///
/// ## Goal-driven pipeline generation
///
/// The pipeline is created *from your goal*. Provide a natural-language goal
/// and the pipeline auto-generates:
/// - `.sruja/pipeline.toml` — stage definitions, model assignments, budgets
/// - `.sruja/agents/*.md` — role-specific system prompts
///
/// Then it runs immediately. The generated files stay so you can tweak
/// models, prompts, budgets, or stages, then re-run.
///
/// ## Config-driven (after generation)
///
/// Once the config exists, the pipeline uses it directly. Change models,
/// add/remove stages, adjust budgets, swap prompt files — all without
/// recompilation.
pub struct PipelineOrchestrator {
    repo: std::path::PathBuf,
    manifest: PipelineManifest,
    lessons: LessonStore,
    budgets: BudgetTracker,
    live_report: LiveReport,
    dry_run: bool,
    focus: Option<String>,
    goal: String,
    llm: Option<Arc<dyn LlmClient>>,
}

impl PipelineOrchestrator {
    /// Create a new orchestrator.
    ///
    /// If a pipeline definition already exists in `.sruja/pipelines/{name}.toml`,
    /// it's loaded and used directly. Otherwise the pipeline is auto-generated
    /// from the goal and saved so you can tweak and re-run it.
    ///
    /// The pipeline name is derived from the goal (e.g. "add_jwt_auth").
    pub fn new(
        repo: &Path,
        goal: String,
        dry_run: bool,
        focus: Option<String>,
        max_cycles: Option<usize>,
    ) -> Self {
        let name = goal_to_name(&goal);

        // Load existing pipeline definition or generate from goal
        let mut manifest = PipelineManifest::load(repo, &name);
        if !manifest.has_stages() {
            let gen = config::generate_from_goal(&goal);
            // Save so user can edit and re-run
            let _ = gen.save(repo, &name);
            generate_prompt_files(repo, &goal, &gen, &name);
            manifest = gen;
        }

        if let Some(cycles) = max_cycles {
            manifest.budgets.max_cycles = cycles;
        }

        Self {
            repo: repo.to_path_buf(),
            budgets: BudgetTracker::new(manifest.budgets.clone()),
            lessons: LessonStore::new(manifest.max_lessons_per_role),
            live_report: LiveReport::new(repo),
            manifest,
            dry_run,
            focus,
            goal,
            llm: None,
        }
    }

    /// Set the LLM client.
    pub fn with_llm(mut self, llm: Arc<dyn LlmClient>) -> Self {
        self.llm = Some(llm);
        self
    }

    /// Run the full pipeline.
    pub async fn run(&mut self) -> PipelineResult {
        let cycles = self.manifest.budgets.max_cycles;
        let focus = self.focus.clone();
        self.live_report.init(1, cycles, focus.as_deref());

        let mut all_stages: Vec<StageResult> = Vec::new();
        let mut scorecards: Vec<Scorecard> = Vec::new();
        let mut cycle_count = 0;

        let mut gaps: Vec<Gap> = Vec::new();
        let mut bugs: Vec<Bug> = Vec::new();
        let mut fixes: Vec<FixReport> = Vec::new();

        for cycle in 0..cycles {
            self.budgets.start_new_cycle();

            // Clone the stage definitions to avoid borrowing self.manifest
            let enabled_stages: Vec<StageDef> = self.manifest.enabled_stages().into_iter().cloned().collect();
            for stage in &enabled_stages {
                if self.dry_run && matches!(stage.id.as_str(), "fixer" | "auditor" | "retester") {
                    continue;
                }

                let (result, duration) = self.run_stage(stage, &gaps, &bugs, &fixes, cycle).await;

                self.extract_artifact(&result, &mut gaps, &mut bugs, &mut fixes, &mut scorecards);

                let artifact_name = self.artifact_name(&result);
                self.live_report.update(&result, &artifact_name, duration.as_millis() as u64);

                self.update_budget(&stage.id);
                all_stages.push(result);
            }

            cycle_count = cycle + 1;

            let current_score = scorecards.last().map(|s| s.total);
            let has_blocking = bugs.iter().any(|b| b.severity == "critical" || b.severity == "high");

            if let Some(score) = current_score {
                self.budgets.previous_score = Some(score);
            }

            let conv = self.budgets.check_convergence(
                current_score,
                has_blocking,
                cycle + 1 < cycles,
            );

            if conv.converged {
                let result = PipelineResult {
                    scorecard: scorecards.into_iter().last(),
                    stages: all_stages.clone(),
                    cycles: cycle_count,
                    converged: true,
                    reason: conv.reason,
                    lessons_recorded: self.lessons.total(),
                };
                self.live_report.add_result(&result);
                return result;
            }
        }

        PipelineResult {
            scorecard: scorecards.into_iter().last(),
            stages: all_stages,
            cycles: cycle_count,
            converged: false,
            reason: format!("max cycles reached ({})", cycles),
            lessons_recorded: self.lessons.total(),
        }
    }

    /// Judge-only mode: re-score using the judge prompt.
    pub async fn judge_only(&self) -> Result<Scorecard, PipelineError> {
        let llm = self.get_llm()?;

        let judge_stage = self.manifest.stages.iter()
            .find(|s| s.id == "judge")
            .ok_or_else(|| PipelineError::Stage {
                stage: "judge".into(),
                message: "No judge stage defined".into(),
            })?;

        let prompt_path = self.manifest.prompt_path(judge_stage, &self.repo);
        let role_prompt = prompts::load_role_prompt(prompt_path.as_deref())
            .map_err(|e| PipelineError::Stage { stage: "judge".into(), message: e })?;

        let task = format!(
            "Goal: {}\n\n\
             Evaluate the project and produce a scorecard across 5 dimensions. \
             Read actual code files. Cite evidence.",
            self.goal
        );

        let req = CompletionRequest::prompt(&role_prompt, &task)
            .with_json();

        let response = llm.complete(&req)
            .await
            .map_err(|e| PipelineError::Stage {
                stage: "judge".into(),
                message: e.to_string(),
            })?;

        Ok(parse_scorecard(&response.content))
    }

    /// Run a single pipeline stage. Returns the result and elapsed duration.
    async fn run_stage(
        &self,
        stage: &StageDef,
        gaps: &[Gap],
        bugs: &[Bug],
        fixes: &[FixReport],
        cycle: usize,
    ) -> (StageResult, std::time::Duration) {
        let start = Instant::now();
        let llm = match self.get_llm() {
            Ok(l) => l,
            Err(e) => return (self.failed_result(stage, vec![e.to_string()]), start.elapsed()),
        };

        let role = PipelineRole::from_str(&stage.id).unwrap_or(PipelineRole::Analyzer);

        // Load stage prompt from file
        let prompt_path = self.manifest.prompt_path(stage, &self.repo);
        let base_prompt = match prompts::load_role_prompt(prompt_path.as_deref()) {
            Ok(p) => p,
            Err(e) => return (self.failed_result(stage, vec![e]), start.elapsed()),
        };

        // Add lessons for this role
        let lessons_text = self.lessons.format_for_prompt(role);
        let system_prompt = if lessons_text.is_empty() {
            base_prompt
        } else {
            format!("{base_prompt}\n\n{lessons_text}")
        };

        // Phase-1 verify (judge only)
        if stage.phase_1_verify {
            let check = self.run_verify_steps().await;
            if !check.all_passed {
                return (StageResult {
                    stage_id: stage.id.clone(), role,
                    success: true,
                    artifact: Some(PipelineArtifact::VerifyReport(check)),
                    duration: start.elapsed(), errors: vec![],
                }, start.elapsed());
            }
        }

        // Build task from goal + artifacts
        let task = build_task_for_stage(&stage.id, &self.goal, gaps, bugs, fixes, cycle);

        // Call LLM
        let req = CompletionRequest::prompt(&system_prompt, &task)
            .with_json();

        let response = match llm.complete(&req).await {
            Ok(r) => r,
            Err(e) => return (self.failed_result(stage, vec![format!("LLM call failed: {e}")]), start.elapsed()),
        };

        let artifact = parse_artifact(&stage.id, &response.content, cycle);
        let elapsed = start.elapsed();

        (StageResult {
            stage_id: stage.id.clone(), role,
            success: true, artifact,
            duration: elapsed, errors: vec![],
        }, elapsed)
    }

    fn failed_result(&self, stage: &StageDef, errors: Vec<String>) -> StageResult {
        StageResult {
            stage_id: stage.id.clone(),
            role: PipelineRole::from_str(&stage.id).unwrap_or(PipelineRole::Analyzer),
            success: false,
            artifact: None,
            duration: std::time::Duration::default(),
            errors,
        }
    }

    /// Run configured verify steps (Phase-1 checks).
    async fn run_verify_steps(&self) -> VerifyReport {
        let mut failures = Vec::new();
        let mut all_passed = true;
        let mut details = String::new();

        for step in &self.manifest.verify {
            let output = std::process::Command::new(&step.command)
                .args(&step.args)
                .current_dir(&self.repo)
                .output();

            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let passed = out.status.success();
                    if !passed {
                        all_passed = false;
                        failures.push(step.id.clone());
                        details.push_str(&format!("{} FAILED:\n{stdout}{stderr}\n", step.id));
                    }
                }
                Err(e) => {
                    all_passed = false;
                    failures.push(step.id.clone());
                    details.push_str(&format!("{} error: {e}\n", step.id));
                }
            }
        }

        VerifyReport { all_passed, failures, details }
    }

    fn extract_artifact(
        &self,
        result: &StageResult,
        gaps: &mut Vec<Gap>,
        bugs: &mut Vec<Bug>,
        fixes: &mut Vec<FixReport>,
        scorecards: &mut Vec<Scorecard>,
    ) {
        match &result.artifact {
            Some(PipelineArtifact::GapReport(r)) => *gaps = r.gaps.clone(),
            Some(PipelineArtifact::BugReport(r)) => *bugs = r.bugs.clone(),
            Some(PipelineArtifact::FixReport(f)) => {
                if !fixes.iter().any(|x| x.bug_id == f.bug_id) {
                    fixes.push(f.clone());
                }
            }
            Some(PipelineArtifact::Scorecard(s)) => scorecards.push(s.clone()),
            _ => {}
        }
    }

    fn artifact_name(&self, result: &StageResult) -> String {
        match &result.artifact {
            Some(PipelineArtifact::GapReport(r)) => format!("{} gaps", r.gaps.len()),
            Some(PipelineArtifact::BugReport(r)) => format!("{} bugs", r.bugs.len()),
            Some(PipelineArtifact::FixReport(_)) => "fix".into(),
            Some(PipelineArtifact::Scorecard(s)) => format!("score {:.1}", s.total),
            Some(PipelineArtifact::AuditResult(a)) =>
                if a.approves { "approved".into() } else { "changes requested".into() },
            Some(PipelineArtifact::RetestResult(_)) => "verified".into(),
            Some(PipelineArtifact::VerifyReport(v)) =>
                if v.all_passed { "verify passed".into() } else { "verify FAILED".into() },
            None => "no artifact".into(),
        }
    }

    fn update_budget(&mut self, stage_id: &str) {
        match stage_id {
            "analyzer" | "self_review" => self.budgets.record_analyzer_pass(),
            "prober" => self.budgets.record_prober_pass(),
            _ => {}
        }
    }

    fn get_llm(&self) -> Result<Arc<dyn LlmClient>, PipelineError> {
        if let Some(ref llm) = self.llm {
            return Ok(llm.clone());
        }
        OpenAiClient::from_env()
            .map(|c| Arc::new(c) as Arc<dyn LlmClient>)
            .map_err(|e| PipelineError::Stage {
                stage: "setup".into(),
                message: format!("No LLM client. Set OPENAI_API_KEY or use with_llm(): {e}"),
            })
    }

    /// Record a lesson from a reviewer rejection.
    pub fn record_lesson(&mut self, role: PipelineRole, what_wrong: String, correction: String) {
        self.lessons.record(Lesson {
            id: format!("pl-{}-{}", role, self.lessons.total() + 1),
            role,
            cycle: self.budgets.cycle,
            what_wrong,
            correction,
        });
    }
}

// ---------------------------------------------------------------------------
// Prompt file generation
// ---------------------------------------------------------------------------

/// Derive a pipeline name from the goal (filesystem-safe slug).
fn goal_to_name(goal: &str) -> String {
    goal.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_lowercase()
        .chars()
        .take(40)
        .collect()
}

/// Write prompt files for all stages in the manifest.
fn generate_prompt_files(repo: &Path, goal: &str, manifest: &PipelineManifest, _name: &str) {
    let agents_dir = repo.join(&manifest.agents_dir);
    let _ = std::fs::create_dir_all(&agents_dir);

    for stage in &manifest.stages {
        let prompt_path = manifest.prompt_path(stage, repo);
        if let Some(path) = prompt_path {
            if !path.exists() {
                let content = config::generate_prompt_file(goal, &stage.id);
                let _ = std::fs::write(&path, &content);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Task construction
// ---------------------------------------------------------------------------

fn build_task_for_stage(
    stage_id: &str,
    goal: &str,
    gaps: &[Gap],
    bugs: &[Bug],
    fixes: &[FixReport],
    _cycle: usize,
) -> String {
    let base = format!("Pipeline goal: {goal}");

    match stage_id {
        "analyzer" => format!(
            "{base}\n\nScan the project. Identify gaps between what's \
             implemented and what the goal requires. For each gap, cite \
             evidence (file:line). Return JSON with a `gaps` array."
        ),
        "self_review" | "analyzer_self_review" => {
            let json = serde_json::to_string_pretty(gaps).unwrap_or_default();
            format!("{base}\n\nSelf-review these gaps. Drop unsubstantiated ones. \
                     Return only survivors.\n\n{json}")
        }
        "prober" => {
            let json = serde_json::to_string_pretty(gaps).unwrap_or_default();
            format!("{base}\n\nWrite test cases from these gaps. Each needs \
                     input, expected behavior, why it fails before fix.\n\n{json}")
        }
        "confirmer" => {
            let json = serde_json::to_string_pretty(bugs).unwrap_or_default();
            format!("{base}\n\nIndependently validate each test case. Confirm, \
                     reject, or adjust severity.\n\n{json}")
        }
        "fixer" => {
            let json = serde_json::to_string_pretty(bugs).unwrap_or_default();
            format!("{base}\n\nFix each bug at the root cause. Write tests. \
                     Run the test suite.\n\n{json}")
        }
        "auditor" => {
            let json = serde_json::to_string_pretty(fixes).unwrap_or_default();
            format!("{base}\n\nCode-review each fix. Approve or request \
                     changes.\n\n{json}")
        }
        "retester" => {
            let json = serde_json::to_string_pretty(fixes).unwrap_or_default();
            format!("{base}\n\nRe-test each fix. Resolved, incomplete, or \
                     regression?\n\n{json}")
        }
        "judge" => format!(
            "{base}\n\nScore the project 0-5 across: functional correctness, \
             code quality, test coverage, UX quality, cost efficiency. \
             Read files. Cite evidence."
        ),
        _ => format!("{base}\n\nExecute your role and produce structured output."),
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

fn parse_artifact(stage_id: &str, content: &str, cycle: usize) -> Option<PipelineArtifact> {
    match stage_id {
        "analyzer" | "self_review" | "analyzer_self_review" =>
            Some(PipelineArtifact::GapReport(parse_gaps(content, cycle))),
        "prober" | "confirmer" =>
            Some(PipelineArtifact::BugReport(parse_bugs(content, cycle))),
        "judge" => Some(PipelineArtifact::Scorecard(parse_scorecard(content))),
        "auditor" => Some(PipelineArtifact::AuditResult(AuditResult {
            fix_index: 0, bug_id: String::new(),
            verdict: AuditVerdict::Approved, issues: vec![], approves: true,
        })),
        "retester" => Some(PipelineArtifact::RetestResult(RetestResult {
            bug_id: String::new(), verdict: RetestVerdict::Resolved,
            details: content.chars().take(500).collect(),
            tester_role: PipelineRole::ReTester,
        })),
        _ => None,
    }
}

pub fn parse_gaps(content: &str, cycle: usize) -> GapReport {
    let v: serde_json::Value = serde_json::from_str(content).unwrap_or_default();
    let gaps_array = v.get("gaps").and_then(|g| g.as_array()).cloned().unwrap_or_default();
    let gaps: Vec<Gap> = gaps_array.into_iter().enumerate().map(|(i, g)| Gap {
        id: format!("gap-{}", i + 1),
        area: g.get("area").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
        description: g.get("description").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        severity: g.get("severity").and_then(|v| v.as_str()).unwrap_or("medium").to_string(),
        evidence: g.get("evidence").and_then(|e| e.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
        substantiated: g.get("substantiated").and_then(|v| v.as_bool()).unwrap_or(true),
    }).collect();
    GapReport { gaps, summary: String::new(), cycle }
}

pub fn parse_bugs(content: &str, cycle: usize) -> BugReport {
    let v: serde_json::Value = serde_json::from_str(content).unwrap_or_default();
    let arr = v.get("bugs").or_else(|| v.get("validations"))
        .and_then(|g| g.as_array()).cloned().unwrap_or_default();
    let bugs: Vec<Bug> = arr.into_iter().enumerate().map(|(i, b)| Bug {
        id: format!("bug-{}", i + 1),
        gap_id: b.get("gap_id").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        area: b.get("area").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
        description: b.get("description").or_else(|| b.get("bug_description"))
            .and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        severity: b.get("severity").or_else(|| b.get("adjusted_severity"))
            .and_then(|v| v.as_str()).unwrap_or("medium").to_string(),
        test_case: b.get("test_case").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        evidence: b.get("evidence").and_then(|e| e.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
    }).collect();
    BugReport { bugs, summary: String::new(), cycle }
}

pub fn parse_scorecard(content: &str) -> Scorecard {
    let v: serde_json::Value = serde_json::from_str(content).unwrap_or_default();
    let s = |k: &str| v.get(k).and_then(|v| v.as_u64()).unwrap_or(0) as u8;
    let scores = [s("functional_correctness"), s("code_quality"), s("test_coverage"),
                   s("ux_quality"), s("cost_efficiency")];
    let total = v.get("total").and_then(|v| v.as_f64())
        .unwrap_or_else(|| scores.iter().sum::<u8>() as f64 / scores.len() as f64);
    Scorecard {
        functional_correctness: scores[0], code_quality: scores[1],
        test_coverage: scores[2], ux_quality: scores[3], cost_efficiency: scores[4],
        evidence: v.get("evidence").and_then(|e| e.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
        total, summary: v.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        improved_from_previous: v.get("improved_from_previous").and_then(|v| v.as_bool()).unwrap_or(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gaps() {
        let r = parse_gaps(r#"{"gaps":[{"description":"X","area":"core","severity":"high"}]}"#, 1);
        assert_eq!(r.gaps.len(), 1);
        assert_eq!(r.gaps[0].description, "X");
    }

    #[test]
    fn test_parse_scorecard() {
        let s = parse_scorecard(r#"{"functional_correctness":4,"total":4.0}"#);
        assert_eq!(s.functional_correctness, 4);
        assert_eq!(s.total, 4.0);
    }

    #[test]
    fn test_parse_bugs_empty() {
        let r = parse_bugs("{}", 1);
        assert!(r.bugs.is_empty());
    }
}
