use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use crate::llm::{CompletionRequest, LlmClient, OpenAiClient, TieredClient};

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
    tiered_llm: Option<Arc<TieredClient>>,
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
            tiered_llm: None,
        }
    }

    /// Set the LLM client (single-provider path).
    pub fn with_llm(mut self, llm: Arc<dyn LlmClient>) -> Self {
        self.llm = Some(llm);
        self
    }

    /// Set a tiered LLM client for multi-provider routing.
    ///
    /// Routes each pipeline stage's model name through the correct provider
    /// (e.g. GLM-5.2 → ZAI, mimo-v2.5-pro → Ximimo).
    pub fn with_tiered(mut self, client: TieredClient) -> Self {
        self.tiered_llm = Some(Arc::new(client));
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
        let mut auditor_rejected = false;

        for cycle in 0..cycles {
            self.budgets.start_new_cycle();

            // Clone the stage definitions to avoid borrowing self.manifest
            let enabled_stages: Vec<StageDef> = self.manifest.enabled_stages().into_iter().cloned().collect();
            for stage in &enabled_stages {
                if self.dry_run && matches!(stage.id.as_str(), "fixer" | "auditor" | "retester") {
                    continue;
                }

                // Skip fix/audit/retest stages when there's nothing to process yet.
                // Prevents wasting API calls and cascading empty artifacts.
                if matches!(stage.id.as_str(), "fixer" | "auditor" | "retester") && bugs.is_empty() {
                    println!("  ⏭ {} stage (skipped — no bugs to process)", stage.id);
                    continue;
                }

                let stage_models = self.manifest.resolve_models(&stage.model);
                let model_name = stage_models.first().map(|m| m.as_str()).unwrap_or("default");
                println!("  ▶ {} stage (model: {model_name})...", stage.id);
                let (result, duration) = self.run_stage(stage, &gaps, &bugs, &fixes, cycle).await;

                self.extract_artifact(&result, &mut gaps, &mut bugs, &mut fixes, &mut scorecards);

                // Track auditor rejection as blocking
                if let Some(PipelineArtifact::AuditResult(ref a)) = result.artifact {
                    if !a.approves {
                        auditor_rejected = true;
                    }
                }

                let artifact_name = self.artifact_name(&result);
                self.live_report.update(&result, &artifact_name, duration.as_millis() as u64);

                self.update_budget(&stage.id);
                all_stages.push(result);
            }

            cycle_count = cycle + 1;

            let current_score = scorecards.last().map(|s| s.total);
            let has_blocking = auditor_rejected
                || bugs.iter().any(|b| b.severity == "critical" || b.severity == "high");

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

        // Route judge through the stage's model from manifest
        let req = build_completion_request(&role_prompt, &task, &self.manifest, &judge_stage.model);

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

        // Call LLM with model routing from manifest.
        // When parallel + multiple models, run concurrent calls on different providers
        // to catch model-specific blind spots (the "cross-model review" pattern).
        let stage_models = self.manifest.resolve_models(&stage.model);
        let effective_models: Vec<String> = if stage.parallel && stage_models.len() > 1 {
            stage_models
        } else {
            stage_models.into_iter().take(1).collect()
        };

        let response = if effective_models.len() > 1 {
            // Parallel multi-model: spawn concurrent calls, merge results
            let mut handles = Vec::new();
            for model in &effective_models {
                let prompt = system_prompt.clone();
                let task_clone = task.clone();
                let model = model.clone();
                let llm_ref = Arc::clone(&llm);
                handles.push(tokio::spawn(async move {
                    let req = CompletionRequest::prompt(&prompt, &task_clone)
                        .with_model(model)
                        .with_json();
                    llm_ref.complete(&req).await
                }));
            }

            let mut contents = Vec::new();
            let mut errors = Vec::new();
            for handle in handles {
                match handle.await {
                    Ok(Ok(resp)) => contents.push(resp.content),
                    Ok(Err(e)) => errors.push(format!("LLM error: {e}")),
                    Err(e) => errors.push(format!("spawn error: {e}")),
                }
            }

            if contents.is_empty() {
                return (self.failed_result(stage, errors), start.elapsed());
            }

            // Merge: concatenate all responses for artifact parsing
            // (parse_artifact handles arrays by taking all items)
            let merged = merge_multi_model_responses(&stage.id, &contents);
            if !errors.is_empty() {
                println!("  ⚠ some parallel models failed: {}", errors.join("; "));
            }
            merged
        } else {
            // Single model: direct call
            let model = effective_models.first().filter(|m| !m.is_empty());
            let req = if let Some(m) = model {
                CompletionRequest::prompt(&system_prompt, &task)
                    .with_model(m.clone())
                    .with_json()
            } else {
                CompletionRequest::prompt(&system_prompt, &task)
                    .with_json()
            };

            match llm.complete(&req).await {
                Ok(r) => r.content,
                Err(e) => return (self.failed_result(stage, vec![format!("LLM call failed: {e}")]), start.elapsed()),
            }
        };

        // For fixer stage: apply code changes to disk
        if stage.id == "fixer" {
            let fix_report = self.apply_fixer_changes(&response, &stage.id, bugs, start).await;
            return (StageResult {
                stage_id: stage.id.clone(), role,
                success: fix_report.status != FixStatus::Failed,
                artifact: Some(PipelineArtifact::FixReport(fix_report)),
                duration: start.elapsed(), errors: vec![],
            }, start.elapsed());
        }

        let artifact = parse_artifact(&stage.id, &response, cycle);
        let elapsed = start.elapsed();

        (StageResult {
            stage_id: stage.id.clone(), role,
            success: true, artifact,
            duration: elapsed, errors: vec![],
        }, elapsed)
    }

    /// Apply code changes returned by the fixer LLM to actual files on disk.
    ///
    /// The LLM returns JSON with patches. Two modes:
    ///
    /// **Full file replacement** (use with care):
    /// ```json
    /// {"fixes": [{"bug_id":"bug-1", "file":"path/to/file.py",
    ///            "new_content":"entire file content", "description":"..."}]}
    /// ```
    ///
    /// **Targeted patches** (preferred — preserves rest of file):
    /// ```json
    /// {"fixes": [{"bug_id":"bug-1", "file":"path/to/file.py",
    ///            "description":"add input validation",
    ///            "patches": [
    ///              {"old": "original code block",
    ///               "new": "replacement code block"}
    ///            ]}]}
    /// ```
    ///
    /// The orchestrator reads each file, applies all patches in order,
    /// then writes the result. Returns a `FixReport` for each file.
    async fn apply_fixer_changes(
        &self,
        content: &str,
        _stage_id: &str,
        _bugs: &[Bug],
        start: std::time::Instant,
    ) -> FixReport {
        let cleaned = strip_fences(content);
        let v: serde_json::Value = serde_json::from_str(&cleaned).unwrap_or_default();

        let fixes_arr = v.get("fixes")
            .or_else(|| v.get("files"))
            .and_then(|a| a.as_array())
            .cloned()
            .unwrap_or_default();

        if fixes_arr.is_empty() {
            // Maybe the LLM returned a single-file response
            let file = v.get("file").and_then(|f| f.as_str()).unwrap_or("");
            let new_content = v.get("new_content").and_then(|c| c.as_str()).unwrap_or("");
            let desc = v.get("description").and_then(|d| d.as_str()).unwrap_or("fix");
            if !file.is_empty() && !new_content.is_empty() {
                return self.write_single_fix(file, new_content, "bug-1", desc, start).await;
            }
            return FixReport {
                bug_id: "unknown".into(),
                status: FixStatus::Failed,
                fix_description: "No fixes returned by LLM".into(),
                modified_files: vec![],
                verify_output: vec!["No fixes in response".into()],
                root_cause: "LLM returned empty fixes array".into(),
            };
        }

        let mut combined_report: Option<FixReport> = None;
        for fix_val in &fixes_arr {
            let file = fix_val.get("file").and_then(|f| f.as_str()).unwrap_or("");
            let bug_id = fix_val.get("bug_id").and_then(|b| b.as_str()).unwrap_or("bug-1");
            let desc = fix_val.get("description").and_then(|d| d.as_str()).unwrap_or("fix");

            if file.is_empty() {
                continue;
            }

            // Check if LLM returned full file content or targeted patches
            let new_content = fix_val.get("new_content").and_then(|c| c.as_str()).unwrap_or("");
            let patches = fix_val.get("patches")
                .and_then(|p| p.as_array()).cloned().unwrap_or_default();

            let report = if !new_content.is_empty() {
                // Full file replacement mode
                self.write_single_fix(file, new_content, bug_id, desc, start).await
            } else if !patches.is_empty() {
                // Targeted patch mode — apply patches to existing file
                self.apply_patches_to_file(file, &patches, bug_id, desc, start).await
            } else {
                continue;
            };

            combined_report = match combined_report {
                Some(mut r) => {
                    r.modified_files.extend(report.modified_files);
                    r.verify_output.extend(report.verify_output);
                    if report.status == FixStatus::Failed {
                        r.status = FixStatus::Failed;
                    }
                    r.fix_description.push_str(&format!("\n---\n{}", report.fix_description));
                    Some(r)
                }
                None => Some(report),
            };
        }

        combined_report.unwrap_or_else(|| FixReport {
            bug_id: "unknown".into(),
            status: FixStatus::Failed,
            fix_description: "Failed to apply any fixes".into(),
            modified_files: vec![],
            verify_output: vec![],
            root_cause: "All fixes had empty file or content".into(),
        })
    }

    /// Write a single file fix and return its report.
    async fn write_single_fix(
        &self,
        file: &str,
        new_content: &str,
        bug_id: &str,
        description: &str,
        _start: std::time::Instant,
    ) -> FixReport {
        let file_path = self.repo.join(file);
        if !file_path.exists() {
            return FixReport {
                bug_id: bug_id.into(),
                status: FixStatus::Failed,
                fix_description: format!("File not found: {file}"),
                modified_files: vec![],
                verify_output: vec!["File does not exist".into()],
                root_cause: "Target file not found in repository".into(),
            };
        }

        // Read current content to confirm the file is valid
        let original = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => return FixReport {
                bug_id: bug_id.into(),
                status: FixStatus::Failed,
                fix_description: format!("Cannot read {file}: {e}"),
                modified_files: vec![],
                verify_output: vec![format!("Read error: {e}")],
                root_cause: format!("File read error: {e}"),
            },
        };

        // Skip if content hasn't changed
        if original == new_content {
            return FixReport {
                bug_id: bug_id.into(),
                status: FixStatus::Resolved,
                fix_description: format!("{description} (no changes needed — content already matches)"),
                modified_files: vec![file.into()],
                verify_output: vec!["No changes applied (content identical)".into()],
                root_cause: String::new(),
            };
        }

        // Write the new content
        match std::fs::write(&file_path, new_content) {
            Ok(_) => {
                // Run verify command if configured
                let verify = self.run_verify_steps().await;
                let mut verify_output = vec![format!("Wrote {file}")];
                if !verify.all_passed {
                    verify_output.push(verify.details);
                }
                FixReport {
                    bug_id: bug_id.into(),
                    status: FixStatus::Resolved,
                    fix_description: description.into(),
                    modified_files: vec![file.into()],
                    verify_output,
                    root_cause: String::new(),
                }
            }
            Err(e) => FixReport {
                bug_id: bug_id.into(),
                status: FixStatus::Failed,
                fix_description: format!("Write failed for {file}: {e}"),
                modified_files: vec![],
                verify_output: vec![format!("Write error: {e}")],
                root_cause: format!("File write error: {e}"),
            },
        }
    }

    /// Apply targeted patches to a file — find/replace individual text blocks.
    ///
    /// Each patch in `patches` has `old` (existing text to find) and
    /// `new` (replacement text). Patches are applied in order.
    /// Fails if any `old` text is not found in the file.
    async fn apply_patches_to_file(
        &self,
        file: &str,
        patches: &[serde_json::Value],
        bug_id: &str,
        description: &str,
        _start: std::time::Instant,
    ) -> FixReport {
        let file_path = self.repo.join(file);
        if !file_path.exists() {
            return FixReport {
                bug_id: bug_id.into(), status: FixStatus::Failed,
                fix_description: format!("File not found: {file}"),
                modified_files: vec![],
                verify_output: vec!["File does not exist".into()],
                root_cause: "Target file not found".into(),
            };
        }

        let mut content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => return FixReport {
                bug_id: bug_id.into(), status: FixStatus::Failed,
                fix_description: format!("Cannot read {file}: {e}"),
                modified_files: vec![],
                verify_output: vec![format!("Read error: {e}")],
                root_cause: format!("File read error: {e}"),
            },
        };

        let mut applied_count = 0;
        let mut errors = Vec::new();

        for patch in patches {
            let old = patch.get("old").and_then(|v| v.as_str()).unwrap_or("");
            let new = patch.get("new").and_then(|v| v.as_str()).unwrap_or("");

            if old.is_empty() {
                errors.push("Patch with empty 'old' field".into());
                continue;
            }

            if content.contains(old) {
                content = content.replacen(old, new, 1);
                applied_count += 1;
            } else {
                errors.push(format!("Could not find expected text in {file}"));
            }
        }

        if applied_count == 0 {
            return FixReport {
                bug_id: bug_id.into(), status: FixStatus::Failed,
                fix_description: format!("{description} — no patches applied"),
                modified_files: vec![],
                verify_output: errors.clone(),
                root_cause: format!("No matching text found for any patch in {file}"),
            };
        }

        // Write the patched content
        match std::fs::write(&file_path, &content) {
            Ok(_) => {
                let mut verify_output = vec![format!("Applied {applied_count} patches to {file}")];
                verify_output.extend(errors);
                FixReport {
                    bug_id: bug_id.into(), status: FixStatus::Resolved,
                    fix_description: description.into(),
                    modified_files: vec![file.into()],
                    verify_output,
                    root_cause: String::new(),
                }
            }
            Err(e) => FixReport {
                bug_id: bug_id.into(), status: FixStatus::Failed,
                fix_description: format!("Write failed for {file}: {e}"),
                modified_files: vec![],
                verify_output: vec![format!("Write error: {e}")],
                root_cause: format!("File write error: {e}"),
            },
        }
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
            Some(PipelineArtifact::AuditResult(a)) => {
                if !a.approves {
                    println!("  ⚠ auditor rejected with {} issues", a.issues.len());
                }
            }
            Some(PipelineArtifact::RetestResult(r)) => {
                if matches!(r.verdict, RetestVerdict::Regression) {
                    println!("  ⚠ regression detected: {}", r.details);
                }
            }
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
        // Prefer tiered client (multi-provider routing by model name)
        if let Some(ref tiered) = self.tiered_llm {
            return Ok(tiered.clone() as Arc<dyn LlmClient>);
        }
        // Fallback to single-provider client
        if let Some(ref llm) = self.llm {
            return Ok(llm.clone());
        }
        // Last resort: OpenAI-compatible from env
        OpenAiClient::from_env()
            .map(|c| Arc::new(c) as Arc<dyn LlmClient>)
            .map_err(|e| PipelineError::Stage {
                stage: "setup".into(),
                message: format!("No LLM client configured. Set up providers in .sruja/config.toml or set OPENAI_API_KEY: {e}"),
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

/// Build a CompletionRequest with the model resolved from the manifest.
fn build_completion_request(
    system_prompt: &str,
    task: &str,
    manifest: &PipelineManifest,
    model_key: &str,
) -> CompletionRequest {
    let models = manifest.resolve_models(model_key);
    if let Some(model) = models.first().filter(|m| !m.is_empty()) {
        CompletionRequest::prompt(system_prompt, task)
            .with_model(model.clone())
            .with_json()
    } else {
        CompletionRequest::prompt(system_prompt, task)
            .with_json()
    }
}

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
        "prober" => {
            let json = serde_json::to_string_pretty(gaps).unwrap_or_default();
            format!("{base}\n\nWrite test cases from these gaps. Each needs \
                     input, expected behavior, why it fails before fix.\n\n{json}")
        }
        "fixer" => {
            let json = serde_json::to_string_pretty(bugs).unwrap_or_default();
            format!("{base}\n\nFix each bug at the root cause. Write tests. \
                     Run the test suite.\n\n{json}")
        }
        "judge" => {
            let gap_count = gaps.len();
            let bug_count = bugs.len();
            let fix_count = fixes.len();
            let has_fixes = if fix_count > 0 {
                let json = serde_json::to_string_pretty(fixes).unwrap_or_default();
                format!("\n\nFixes applied:\n{json}")
            } else {
                String::new()
            };
            format!(
                "{base}\n\nPipeline found {gap_count} gaps and {bug_count} bugs, \
                 applied {fix_count} fixes.{has_fixes}\n\n\
                 Score the project 0-5 across: functional correctness, \
                 code quality, test coverage, UX quality, cost efficiency. \
                 Read actual code files. Cite evidence (file:line). \
                 Return JSON with fields: functional_correctness, code_quality, \
                 test_coverage, ux_quality, cost_efficiency, total (average), \
                 summary, evidence."
            )
        }
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
        "auditor" => Some(PipelineArtifact::AuditResult(parse_audit(content))),
        "retester" => Some(PipelineArtifact::RetestResult(parse_retest(content))),
        _ => None,
    }
}

/// Extract the first complete JSON object `{...}` or array `[...]` from mixed text.
///
/// Handles cases where the LLM wraps JSON in prose like:
/// "I'll explore...\n\n{\"key\": \"value\"}\n\nDone."
///
/// Tracks brace/bracket depth to avoid matching from the first `{` to the last `}`
/// when the response contains multiple separate JSON objects.
fn extract_first_json_blob(text: &str) -> Option<serde_json::Value> {
    let bytes = text.as_bytes();
    // Find the first opening brace/bracket
    let start = bytes.iter().position(|&b| b == b'{' || b == b'[')?;
    let open = bytes[start];
    let close = if open == b'{' { b'}' } else { b']' };

    let mut depth = 0u32;
    for (i, &b) in bytes[start..].iter().enumerate() {
        if b == open {
            depth += 1;
        } else if b == close {
            depth -= 1;
            if depth == 0 {
                let end = start + i + 1;
                return serde_json::from_str(&text[start..end]).ok();
            }
        }
    }
    None
}

/// Strip markdown code fences from JSON content.
fn strip_fences(content: &str) -> String {
    content
        .trim()
        .strip_prefix("```json")
        .or_else(|| content.trim().strip_prefix("```"))
        .map(|s| s.strip_suffix("```").unwrap_or(s))
        .unwrap_or(content)
        .trim()
        .to_string()
}

pub fn parse_gaps(content: &str, cycle: usize) -> GapReport {
    let cleaned = strip_fences(content);
    let v: serde_json::Value = serde_json::from_str(&cleaned)
        .ok()
        .or_else(|| extract_first_json_blob(&cleaned))
        .unwrap_or_default();
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
    let cleaned = strip_fences(content);
    let v: serde_json::Value = serde_json::from_str(&cleaned)
        .ok()
        .or_else(|| extract_first_json_blob(&cleaned))
        .unwrap_or_default();
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

pub fn parse_audit(content: &str) -> AuditResult {
    let cleaned = strip_fences(content);
    let v: serde_json::Value = serde_json::from_str(&cleaned)
        .ok()
        .or_else(|| extract_first_json_blob(&cleaned))
        .unwrap_or_default();

    let verdict_str = v.get("verdict")
        .and_then(|val| val.as_str())
        .unwrap_or("request_changes");
    let verdict = match verdict_str {
        "approved" => AuditVerdict::Approved,
        "rejected" => AuditVerdict::Rejected,
        _ => AuditVerdict::RequestChanges,
    };

    let issues = v.get("issues")
        .and_then(|a| a.as_array())
        .map(|a| a.iter().filter_map(|i| i.as_str().map(String::from)).collect())
        .unwrap_or_default();

    AuditResult {
        fix_index: 0,
        bug_id: v.get("bug_id").and_then(|b| b.as_str()).unwrap_or("").to_string(),
        verdict: verdict.clone(),
        issues,
        approves: matches!(verdict, AuditVerdict::Approved),
    }
}

pub fn parse_retest(content: &str) -> RetestResult {
    let cleaned = strip_fences(content);
    let v: serde_json::Value = serde_json::from_str(&cleaned)
        .ok()
        .or_else(|| extract_first_json_blob(&cleaned))
        .unwrap_or_default();

    let verdict_str = v.get("verdict")
        .and_then(|val| val.as_str())
        .unwrap_or("resolved");
    let verdict = match verdict_str {
        "incomplete" => RetestVerdict::Incomplete,
        "regression" => RetestVerdict::Regression,
        _ => RetestVerdict::Resolved,
    };

    RetestResult {
        bug_id: v.get("bug_id").and_then(|b| b.as_str()).unwrap_or("").to_string(),
        verdict,
        details: v.get("details").and_then(|d| d.as_str())
            .unwrap_or("")
            .to_string(),
        tester_role: PipelineRole::ReTester,
    }
}

/// Merge responses from multiple parallel model calls into a single parseable
/// JSON structure. For gap/bug reports, deduplicates by description.
fn merge_multi_model_responses(stage_id: &str, contents: &[String]) -> String {
    match stage_id {
        "analyzer" | "self_review" | "analyzer_self_review" => {
            let mut all_gaps = Vec::new();
            let mut seen = std::collections::HashSet::new();
            for content in contents {
                let cleaned = strip_fences(content);
                let v: serde_json::Value = serde_json::from_str(&cleaned)
                    .ok()
                    .or_else(|| extract_first_json_blob(&cleaned))
                    .unwrap_or_default();
                if let Some(arr) = v.get("gaps").and_then(|g| g.as_array()) {
                    for gap in arr {
                        let desc = gap.get("description").and_then(|d| d.as_str()).unwrap_or("");
                        if seen.insert(desc.to_string()) {
                            all_gaps.push(gap.clone());
                        }
                    }
                }
            }
            serde_json::json!({ "gaps": all_gaps }).to_string()
        }
        "prober" | "confirmer" => {
            let mut all_bugs = Vec::new();
            let mut seen = std::collections::HashSet::new();
            for content in contents {
                let cleaned = strip_fences(content);
                let v: serde_json::Value = serde_json::from_str(&cleaned)
                    .ok()
                    .or_else(|| extract_first_json_blob(&cleaned))
                    .unwrap_or_default();
                let arr = v.get("bugs").or_else(|| v.get("validations"))
                    .and_then(|g| g.as_array());
                if let Some(arr) = arr {
                    for bug in arr {
                        let desc = bug.get("description").or_else(|| bug.get("bug_description"))
                            .and_then(|d| d.as_str()).unwrap_or("");
                        if seen.insert(desc.to_string()) {
                            all_bugs.push(bug.clone());
                        }
                    }
                }
            }
            serde_json::json!({ "bugs": all_bugs }).to_string()
        }
        _ => {
            // For other stages, use the first successful response
            contents.first().cloned().unwrap_or_default()
        }
    }
}

pub fn parse_scorecard(content: &str) -> Scorecard {
    let cleaned = strip_fences(content);

    // Try direct parse first, then extract JSON object from mixed text
    let v: serde_json::Value = serde_json::from_str(&cleaned)
        .ok()
        .or_else(|| extract_first_json_blob(&cleaned))
        .unwrap_or_default();
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
