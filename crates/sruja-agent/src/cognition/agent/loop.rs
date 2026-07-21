use super::*;

use crate::cognition::prompts;

impl Agent {
    // --- Reflect: extract learnings from a completed run ---

    /// Extract lessons learned from a completed run.
    ///
    /// Each run produces learnings that future runs retrieve — the compound
    /// self-learning loop that makes the agent improve over time.
    pub async fn reflect(
        &self,
        comprehension: &Comprehension,
        plan: &Plan,
        results: &[StepResult],
        critique: Option<&Critique>,
    ) -> Result<Vec<LearningEntry>, AgentError> {
        let successes = results
            .iter()
            .filter(|r| r.status == StepStatus::Ok)
            .count();
        let failures = results
            .iter()
            .filter(|r| r.status == StepStatus::Failed)
            .count();

        let user = format!(
            "## Goal\n{}\n\n\
             ## What happened\n\
             - {} subtasks succeeded, {} failed\n\
             - Comprehension cited elements: {:?}\n\
             - Critique: {}\n\n\
             ## Instructions\n\
             Extract 1-3 learnings from this run. For each, produce JSON:\n\
             {{\"context\": \"...\", \"hypothesis\": \"...\", \"guardrail_advice\": \"...\", \
             \"kind\": \"playbook|guardrail\"}}\n\
             Playbooks = what worked. Guardrails = what to avoid.",
            plan.goal_statement,
            successes,
            failures,
            comprehension.cited_elements,
            critique
                .map(|c| format!("approved={}, score={}", c.approved, c.score))
                .unwrap_or_else(|| "skipped".into()),
        );

        let req = CompletionRequest::prompt(prompts::REFLECTION_SYSTEM_PROMPT, &user)
            .with_model(&self.config.models.cheap);

        let (response, _usage, _signals) = self.run_tool_loop(req).await?;

        let learnings = parse_learnings_from_response(&response.content);

        for entry in &learnings {
            self.hooks.on_learning(entry).await;
        }

        // Persist learnings to memory.
        if let Some(ref mem) = self.memory {
            for entry in &learnings {
                if let Err(e) = mem.record(entry.clone()) {
                    tracing::warn!(error = %e, "failed to record learning to memory");
                }
            }
            if let Some(ref repo) = self.repo_root {
                if let Err(e) = mem.save_to_path(repo) {
                    tracing::warn!(error = %e, "failed to persist learnings to disk");
                }
            }
        }

        Ok(learnings)
    }

    /// Helper: best-effort emit a LoopEvent. Logs but ignores errors.
    fn emit_event(events: Option<&mpsc::Sender<LoopEvent>>, event: LoopEvent) {
        if let Some(sender) = events {
            if let Err(e) = sender.try_send(event) {
                tracing::warn!(error = %e, "loop_event: failed to send (receiver closed?)");
            }
        }
    }

    /// Run the outer ReAct loop: comprehend once, then iterate
    /// Simplified agent loop: trust the model, give it tools, verify results.
    ///
    /// Flow:
    /// 1. Classify complexity (deterministic)
    /// 2. Model drives via tool loop (read files, edit, run commands)
    /// 3. Deterministic verification (lint, test, drift)
    /// 4. If verification fails, feed errors back for one retry
    ///
    /// No planning phase. No critique ensemble. No TDD enforcement.
    /// The model decides what to do. Deterministic checks catch mistakes.
    pub async fn run_loop(
        &self,
        goal: &crate::goal::GoalSpec,
        loop_config: &LoopConfig,
        events: Option<&mpsc::Sender<LoopEvent>>,
        _calibration: Option<&crate::calibration::AskPlan>,
    ) -> Result<LoopResult, AgentError> {
        let max_iterations = loop_config.max_iterations.max(1);

        Self::emit_event(events, LoopEvent::PhaseChanged(LoopPhase::Comprehend));

        // Validate target element IDs before comprehension when available.
        if !goal.target_elements.is_empty() {
            if let Err(unknown) = goal.validate(None) {
                return Err(AgentError::Validation(format!(
                    "unknown target element IDs: {}",
                    unknown.join(", ")
                )));
            }
        }

        let complexity = classify_task_complexity(
            &goal.statement,
            &goal.target_files,
            &goal.target_elements,
        );

        // Build synthetic comprehension for backward compatibility.
        let mut comprehension = Comprehension {
            goal: goal.statement.clone(),
            summary: goal.statement.clone(),
            cited_elements: goal.target_elements.clone(),
            key_findings: vec![],
            risks: vec![],
            usage: Usage::default(),
            retrieved_learning_ids: vec![],
            complexity,
            pre_conditions: vec![],
        };

        Self::emit_event(
            events,
            LoopEvent::Started {
                goal: goal.statement.clone(),
                max_iterations,
            },
        );

        // --- Build initial request ---
        let system = prompts::AGENT_LOOP_SYSTEM_PROMPT;
        let pre_condition_section = if !comprehension.pre_conditions.is_empty() {
            format!(
                "\n\n## Pre-conditions from Error History\n\
                 This repo has patterns of recurring failures. Address these proactively:\n{}\n",
                comprehension
                    .pre_conditions
                    .iter()
                    .map(|p| format!("- {p}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            String::new()
        };
        let initial_user = format!("{}{}", goal.statement, pre_condition_section);
        let mut req =
            CompletionRequest::prompt(system, &initial_user).with_tools(self.tools.schemas());
        req.model = Some(self.config.models.mid.clone());

        // Research tasks use the premium model for deeper analysis.
        if complexity == TaskComplexity::Research {
            req.model = Some(self.config.models.premium.clone());
        }

        let pipeline = &loop_config.pipeline;
        let mut pipeline_stages = pipeline.stages.clone();

        // Dynamically select stages based on LLM task classification.
        // The LLM's classification is more reliable than a static pipeline
        // because it considers the actual goal semantics at runtime.
        if pipeline_stages == crate::manifest::PipelineConfig::default().stages {
            pipeline_stages = match complexity {
                TaskComplexity::Research => vec![
                    crate::manifest::StageKind::Comprehend,
                    crate::manifest::StageKind::Reflect,
                ],
                TaskComplexity::Trivial => vec![crate::manifest::StageKind::Implement],
                _ => pipeline_stages, // keep Simple/Moderate/Complex as configured
            };
        }
        let mut total_usage = Usage::default();
        let mut iterations: Vec<LoopIteration> = Vec::new();
        let mut converged = false;
        let mut termination = LoopTermination::MaxIterations;
        let mut _last_output = String::new();
        let mut step_results: Vec<StepResult> = Vec::new();
        let mut non_converged_count: usize = 0;
        let mut seen_signatures: Vec<String> = Vec::new();
        let mut failure_tracker: FailureTracker = FailureTracker::default();
        let mut scope_drift: ScopeDrift = ScopeDrift::default();
        let mut current_plan: Option<Plan> = None;
        let mut current_critique: Option<Critique> = None;

        // Research tasks hard-cap at 1 iteration — comprehension IS the output.
        let effective_iterations = if complexity == TaskComplexity::Research {
            1
        } else {
            max_iterations
        };

        // --- Write initial checkpoint after comprehension ---
        // This enables resume from comprehension state if something goes wrong
        if let Some(ref checkpoint_dir) = loop_config.checkpoint_dir {
            let checkpoint = RunCheckpoint {
                goal: goal.statement.clone(),
                comprehension: comprehension.clone(),
                iterations: Vec::new(),
                last_plan: None,
                last_steps: Vec::new(),
                last_critique: None,
                failure_tracker: FailureTracker::default(),
                total_usage: Usage::default(),
                converged: false,
                termination: LoopTermination::MaxIterations,
                seen_signatures: Vec::new(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            if let Err(e) = checkpoint.write(checkpoint_dir) {
                tracing::warn!(error = %e, "checkpoint: failed to write initial checkpoint");
            }
        }

        for iteration in 1..=effective_iterations {
            Self::emit_event(
                events,
                LoopEvent::IterationStarted {
                    n: iteration,
                    reason: if iteration > 1 {
                        Some("Addressing feedback from previous iteration".into())
                    } else {
                        None
                    },
                },
            );

            let mut iteration_verify_failed: Vec<String> = Vec::new();

            let stages_this_iteration: Vec<crate::manifest::StageKind> = pipeline_stages.clone();
            for &stage_kind in &stages_this_iteration {
                // Skip "comprehend" after the first iteration — understood once.
                if stage_kind == crate::manifest::StageKind::Comprehend && iteration > 1 {
                    continue;
                }

                // Set file permissions for this stage.
                let phase = stage_kind.to_file_guard_phase();
                if self.guard.phase() != phase {
                    self.guard.set_phase(phase);
                    self.hooks.on_phase_change(phase).await;
                }
                Self::emit_event(events, LoopEvent::PhaseChanged(stage_kind.to_loop_phase()));

                match stage_kind {
                    crate::manifest::StageKind::Comprehend
                    | crate::manifest::StageKind::TestReview => {
                        // Use full LLM-based comprehension for all non-trivial tasks.
                        // Only Trivial tasks keep the synthetic comprehension.
                        if stage_kind == crate::manifest::StageKind::Comprehend
                            && iteration == 1
                            && complexity != TaskComplexity::Trivial
                        {
                            let real_comprehension = self.comprehend(goal).await?;
                            total_usage.accumulate(&real_comprehension.usage);
                            _last_output = real_comprehension.summary.clone();
                            comprehension = real_comprehension;

                            step_results.push(StepResult {
                                subtask_id: "research_comprehend".into(),
                                status: StepStatus::Ok,
                                output: _last_output.clone(),
                                usage: comprehension.usage.clone(),
                                tool_signals: vec![],
                                converged: true,
                            });
                        }
                        // else: common comprehend / TestReview are no-ops.
                    }

                    crate::manifest::StageKind::Plan => {
                        let mut plan = self.plan(goal, &comprehension).await?;
                        // Notify hooks about the plan.
                        self.hooks.after_plan(&mut plan).await;
                        current_plan = Some(plan);
                    }

                    crate::manifest::StageKind::Critique => {
                        if let Some(ref plan) = current_plan {
                            let critique = self.critique(plan, &step_results).await?;
                            self.hooks.after_review(&critique).await;
                            current_critique = Some(critique);
                        }
                    }

                    crate::manifest::StageKind::Fix => {
                        // Targeted fix: if critique has file-level issues, run a
                        // focused tool loop to apply targeted edits instead of
                        // regenerating the full plan.
                        if let Some(ref critique) = current_critique {
                            if !critique.approved && !critique.issues.is_empty() {
                                let file_refs = crate::cognition::parsing::extract_file_references(
                                    &critique.issues,
                                );
                                if !file_refs.is_empty() {
                                    let git_diff = self.get_git_diff().await.unwrap_or_default();
                                    let critique_json =
                                        serde_json::to_string_pretty(&serde_json::json!({
                                            "approved": critique.approved,
                                            "score": critique.score,
                                            "issues": critique.issues,
                                            "suggestions": critique.suggestions,
                                            "file_references": file_refs.iter().map(|(f, lines)| {
                                                serde_json::json!({"file": f, "lines": lines})
                                            }).collect::<Vec<_>>(),
                                        }))
                                        .unwrap_or_default();

                                    let fix_user = format!(
                                        "## Current Diff\n```diff\n{}\n```\n\n\
                                         ## Critique Issues\n```json\n{}\n```\n\n\
                                         ## Instructions\n\
                                         Fix the issues above by editing the flagged files.\n\
                                         Only modify files referenced in the critique.",
                                        git_diff, critique_json,
                                    );
                                    let mut fix_req = CompletionRequest::prompt(
                                        prompts::FIX_SYSTEM_PROMPT,
                                        &fix_user,
                                    )
                                    .with_tools(self.tools.schemas());
                                    fix_req.model = Some(self.config.models.premium.clone());

                                    let (response, usage, tool_signals, step_converged) =
                                        self.run_tool_loop_with_limit(fix_req, 5).await?;
                                    total_usage.accumulate(&usage);
                                    scope_drift.record_tool_signals(&tool_signals);
                                    // Emit streaming token update
                                    Self::emit_event(
                                        events,
                                        LoopEvent::UsageUpdate {
                                            prompt_tokens: total_usage.prompt_tokens,
                                            completion_tokens: total_usage.completion_tokens,
                                            total_tokens: total_usage.total_tokens,
                                            estimated_cost_usd: total_usage.estimated_cost_usd(),
                                        },
                                    );

                                    let status = if step_has_quality(
                                        step_converged,
                                        &tool_signals,
                                        &response.content,
                                    ) {
                                        StepStatus::Ok
                                    } else {
                                        StepStatus::Failed
                                    };
                                    let output = response.content;
                                    let result = StepResult {
                                        subtask_id: format!("{iteration}_fix"),
                                        status,
                                        output: output.clone(),
                                        usage,
                                        tool_signals,
                                        converged: step_converged,
                                    };
                                    // Pass a placeholder subtask for the hook
                                    let fix_subtask = Subtask {
                                        id: format!("{iteration}_fix"),
                                        description: "targeted fix from critique feedback".into(),
                                        tier: TaskTier::Premium,
                                        kind: SubtaskKind::Implement,
                                        files: file_refs.iter().map(|(f, _)| f.clone()).collect(),
                                        acceptance_criteria: vec![],
                                    };
                                    self.hooks.after_step(&fix_subtask, &result).await;
                                    step_results.push(result);
                                    _last_output = output;
                                } else {
                                    tracing::info!(
                                        "fix stage: no file-level references — skipping"
                                    );
                                }
                            } else {
                                tracing::info!("fix stage: critique approved or empty — skipping");
                            }
                        } else {
                            tracing::info!("fix stage: no critique available — skipping");
                        }
                    }

                    crate::manifest::StageKind::Reflect => {
                        let reflect_plan = current_plan.clone().unwrap_or_else(|| Plan {
                            goal: goal.to_string(),
                            goal_statement: goal.statement.clone(),
                            criteria: goal.acceptance_criteria.clone(),
                            subtasks: vec![Subtask {
                                id: "research".into(),
                                description: goal.statement.clone(),
                                tier: TaskTier::Mid,
                                kind: SubtaskKind::Comprehend,
                                files: goal.target_files.clone(),
                                acceptance_criteria: goal.acceptance_criteria.clone(),
                            }],
                            tdd: false,
                            risks: vec![],
                            schema_version: "1.0".into(),
                            complexity,
                        });
                        let _ = self
                            .reflect(
                                &comprehension,
                                &reflect_plan,
                                &step_results,
                                current_critique.as_ref(),
                            )
                            .await;
                    }

                    crate::manifest::StageKind::Implement
                    | crate::manifest::StageKind::TestAuthor => {
                        // When a plan exists, use structured per-subtask execution
                        // with phase enforcement and TDD support.
                        if let Some(ref plan) = current_plan {
                            let exec_results = self.execute(plan).await?;
                            let mut usage_sum = crate::llm::Usage::default();
                            for r in &exec_results {
                                usage_sum.accumulate(&r.usage);
                            }
                            total_usage.accumulate(&usage_sum);
                            if let Some(last) = exec_results.last() {
                                _last_output = last.output.clone();
                                if !last.converged {
                                    non_converged_count += 1;
                                }
                                scope_drift.record_tool_signals(&last.tool_signals);
                            }
                            step_results.extend(exec_results);

                            Self::emit_event(
                                events,
                                LoopEvent::StepProgress {
                                    step: iteration,
                                    total: max_iterations,
                                    description: format!(
                                        "{} ({} subtask(s))",
                                        stage_kind.user_friendly_description(),
                                        plan.subtasks.len(),
                                    ),
                                },
                            );
                        } else {
                            // Fallback: raw tool loop when no plan is available.
                            let max_iters =
                                complexity.max_tool_iterations(self.config.max_tool_iterations);
                            let (response, usage, tool_signals, step_converged) = self
                                .run_tool_loop_with_limit(req.clone(), max_iters)
                                .await?;
                            total_usage.accumulate(&usage);
                            scope_drift.record_tool_signals(&tool_signals);
                            // Emit streaming token update
                            Self::emit_event(
                                events,
                                LoopEvent::UsageUpdate {
                                    prompt_tokens: total_usage.prompt_tokens,
                                    completion_tokens: total_usage.completion_tokens,
                                    total_tokens: total_usage.total_tokens,
                                    estimated_cost_usd: total_usage.estimated_cost_usd(),
                                },
                            );
                            _last_output = response.content.clone();

                            if !step_converged {
                                non_converged_count += 1;
                            }

                            let status = if !step_has_quality(
                                step_converged,
                                &tool_signals,
                                &response.content,
                            ) {
                                StepStatus::Failed
                            } else {
                                StepStatus::Ok
                            };
                            step_results.push(StepResult {
                                subtask_id: format!("{iteration}_{stage_kind:?}"),
                                status,
                                output: response.content.clone(),
                                usage: usage.clone(),
                                tool_signals,
                                converged: step_converged,
                            });
                        }
                    }

                    crate::manifest::StageKind::Replan => {
                        // Structured replan: use critique feedback to generate
                        // a revised plan instead of a raw tool loop.
                        if let (Some(ref critique), Some(ref _plan)) =
                            (&current_critique, &current_plan)
                        {
                            let pressure = if non_converged_count > 0 {
                                Some(format!(
                                    "{} of {} iterations failed to converge. \
                                     Change the approach significantly.",
                                    non_converged_count, iteration
                                ))
                            } else {
                                None
                            };
                            let new_plan = self
                                .replan(
                                    goal,
                                    &comprehension,
                                    critique,
                                    pressure.as_deref(),
                                    &failure_tracker,
                                )
                                .await?;
                            current_plan = Some(new_plan);
                        } else {
                            // Fallback: raw tool loop when critique or plan missing.
                            let max_iters =
                                complexity.max_tool_iterations(self.config.max_tool_iterations);
                            let (response, usage, tool_signals, step_converged) = self
                                .run_tool_loop_with_limit(req.clone(), max_iters)
                                .await?;
                            total_usage.accumulate(&usage);
                            scope_drift.record_tool_signals(&tool_signals);
                            // Emit streaming token update
                            Self::emit_event(
                                events,
                                LoopEvent::UsageUpdate {
                                    prompt_tokens: total_usage.prompt_tokens,
                                    completion_tokens: total_usage.completion_tokens,
                                    total_tokens: total_usage.total_tokens,
                                    estimated_cost_usd: total_usage.estimated_cost_usd(),
                                },
                            );
                            _last_output = response.content.clone();

                            if !step_converged {
                                non_converged_count += 1;
                            }

                            let status = if !step_has_quality(
                                step_converged,
                                &tool_signals,
                                &response.content,
                            ) {
                                StepStatus::Failed
                            } else {
                                StepStatus::Ok
                            };
                            step_results.push(StepResult {
                                subtask_id: format!("{iteration}_{stage_kind:?}"),
                                status,
                                output: response.content.clone(),
                                usage: usage.clone(),
                                tool_signals,
                                converged: step_converged,
                            });
                        }
                    }

                    crate::manifest::StageKind::Verify => {
                        // --- Deterministic verification ---
                        Self::emit_event(events, LoopEvent::PhaseChanged(LoopPhase::Verify));
                        iteration_verify_failed = if let Some(vconf) = &loop_config.verifier {
                            let results = run_verification_steps(
                                &vconf.steps,
                                &vconf.options,
                                &vconf.workdir,
                            )
                            .await;
                            for r in &results {
                                Self::emit_event(
                                    events,
                                    LoopEvent::VerifyResult {
                                        step: r.step_id.clone(),
                                        ok: r.status.is_pass(),
                                    },
                                );
                            }
                            summarize_verify_failures(&results)
                        } else {
                            Vec::new()
                        };

                        // --- Scope drift detection (escalate pipeline if needed) ---
                        if !scope_drift.escalated && scope_drift.detect(complexity) {
                            let new_stages = scope_drift.escalated_stages(&pipeline_stages);
                            tracing::info!(
                                from = ?pipeline_stages,
                                to = ?new_stages,
                                "pipeline: escalating due to scope drift"
                            );
                            pipeline_stages = new_stages;
                            scope_drift.escalated = true;
                        }
                    }
                }

                // --- Write checkpoint after each stage ---
                // This enables resume from last good state if something goes wrong
                if let Some(ref checkpoint_dir) = loop_config.checkpoint_dir {
                    let checkpoint = RunCheckpoint {
                        goal: goal.statement.clone(),
                        comprehension: comprehension.clone(),
                        iterations: iterations.clone(),
                        last_plan: current_plan.clone(),
                        last_steps: step_results.clone(),
                        last_critique: current_critique.clone(),
                        failure_tracker: failure_tracker.clone(),
                        total_usage: total_usage.clone(),
                        converged: false,
                        termination: LoopTermination::MaxIterations, // placeholder
                        seen_signatures: seen_signatures.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    };
                    if let Err(e) = checkpoint.write(checkpoint_dir) {
                        tracing::warn!(error = %e, "checkpoint: failed to write after stage");
                    }
                }
            }

            // Determine approval from the last work stage result.
            let last_work = step_results.last();
            let step_converged = last_work.is_some_and(|r| r.converged);
            let last_tool_signals = last_work.map(|r| r.tool_signals.as_slice()).unwrap_or(&[]);
            let critique_approved = current_critique
                .as_ref()
                .map(|c| c.approved)
                .unwrap_or(true);
            let approved = if complexity == TaskComplexity::Research {
                // For Research, comprehension IS the deliverable — no verify gate needed.
                step_has_quality(step_converged, last_tool_signals, &_last_output)
            } else {
                iteration_verify_failed.is_empty()
                    && step_has_quality(step_converged, last_tool_signals, &_last_output)
                    && critique_approved
            };

            // Record iteration.
            iterations.push(LoopIteration {
                iteration,
                replanned: iteration > 1,
                plan_goal: goal.statement.clone(),
                subtask_count: pipeline_stages.len(),
                succeeded: if approved { 1 } else { 0 },
                failed: if approved { 0 } else { 1 },
                critique_approved: approved,
                critique_score: if approved { 1.0 } else { 0.0 },
                critique_issues: iteration_verify_failed.clone(),
                verify_failed: iteration_verify_failed.clone(),
                injected_learning_ids: vec![],
                usage: total_usage.clone(),
                cost_usd: total_usage.estimated_cost_usd(),
                plan_parse_error: None,
                incorporation_gap: None,
            });

            // --- Emit live usage update ---
            Self::emit_event(
                events,
                LoopEvent::UsageUpdate {
                    prompt_tokens: total_usage.prompt_tokens,
                    completion_tokens: total_usage.completion_tokens,
                    total_tokens: total_usage.total_tokens,
                    estimated_cost_usd: total_usage.estimated_cost_usd(),
                },
            );

            // --- Emit iteration complete summary ---
            let iter_succeeded = if approved { 1 } else { 0 };
            let iter_failed = if approved { 0 } else { 1 };
            Self::emit_event(
                events,
                LoopEvent::IterationComplete {
                    iteration,
                    succeeded: iter_succeeded,
                    failed: iter_failed,
                    critique_approved: approved,
                    cost_usd: total_usage.estimated_cost_usd(),
                },
            );

            // --- Write checkpoint after each iteration ---
            // This enables resume from last good state if something goes wrong
            if let Some(ref checkpoint_dir) = loop_config.checkpoint_dir {
                let checkpoint = RunCheckpoint {
                    goal: goal.statement.clone(),
                    comprehension: comprehension.clone(),
                    iterations: iterations.clone(),
                    last_plan: current_plan.clone(),
                    last_steps: step_results.clone(),
                    last_critique: current_critique.clone(),
                    failure_tracker: failure_tracker.clone(),
                    total_usage: total_usage.clone(),
                    converged: false,
                    termination: LoopTermination::MaxIterations, // placeholder
                    seen_signatures: seen_signatures.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                if let Err(e) = checkpoint.write(checkpoint_dir) {
                    tracing::warn!(error = %e, "checkpoint: failed to write after iteration");
                }
            }

            // --- Spend cap (check BEFORE convergence to enforce budget) ---
            if let Some(cap) = loop_config.spend_cap_usd {
                let cost: f64 = iterations.iter().map(|i| i.cost_usd).sum();
                if cost >= cap {
                    termination = LoopTermination::SpendCapExceeded(cost);
                    break;
                }
            }

            // --- Convergence check ---
            if approved && loop_config.stop_on_approval {
                converged = true;
                termination = LoopTermination::Approved;
                break;
            }

            // --- Non-convergence fail-fast ---
            if iterations.len() >= 2 {
                let non_converged_fraction = non_converged_count as f64 / iterations.len() as f64;
                if self.config.max_non_converged_fraction <= 1.0
                    && non_converged_fraction > self.config.max_non_converged_fraction
                {
                    termination = LoopTermination::ModelNotConverging(non_converged_fraction);
                    break;
                }
            }

            // --- Oscillation detection ---
            if loop_config.detect_oscillation {
                let signature = iteration_verify_failed.join("|");
                if !signature.is_empty() {
                    if seen_signatures.last() == Some(&signature) {
                        tracing::warn!(
                            iteration,
                            "oscillation: same verify_failed pattern repeated consecutively"
                        );
                        termination = LoopTermination::Oscillation;
                        break;
                    }
                    seen_signatures.push(signature);
                }
            }

            // --- Recovery strategy: structured error feedback ---
            if !iteration_verify_failed.is_empty() {
                let error_class = classify_error(&iteration_verify_failed, &step_results);
                failure_tracker.record(
                    format!("iteration {iteration}"),
                    iteration_verify_failed.join("; "),
                    iteration,
                    error_class,
                );

                let retries_remaining = pipeline
                    .max_retries
                    .saturating_sub(failure_tracker.failures.len());

                match pipeline.recovery {
                    crate::manifest::RecoveryStrategy::Retry if retries_remaining > 0 => {
                        let mut feedback = format!(
                            "Verification failed:\n{}",
                            iteration_verify_failed.join("\n")
                        );
                        feedback.push_str(&failure_tracker.format_for_prompt());
                        req.messages.push(Message::user(&feedback));
                    }
                    crate::manifest::RecoveryStrategy::DiagnoseThenRetry
                        if retries_remaining > 0 =>
                    {
                        let mut feedback = String::from(
                            "[Diagnostic mode]\nAnalyze the failure before retrying.\n",
                        );
                        feedback.push_str(&format!(
                            "Failure:\n{}\n",
                            iteration_verify_failed.join("\n")
                        ));
                        failure_tracker.diagnostic = Some(iteration_verify_failed.join(", "));
                        feedback.push_str(&failure_tracker.format_for_prompt());
                        req.messages.push(Message::user(&feedback));
                    }
                    crate::manifest::RecoveryStrategy::Escalate => {
                        tracing::info!("recovery: escalate — stopping for human input");
                        termination = LoopTermination::NoReplan;
                        break;
                    }
                    crate::manifest::RecoveryStrategy::Fail => {
                        tracing::info!("recovery: fail — stopping pipeline");
                        termination = LoopTermination::NoReplan;
                        break;
                    }
                    _ => {
                        // No retries remaining — stop.
                        tracing::info!(
                            retries_remaining,
                            "recovery: no retries remaining — stopping"
                        );
                        termination = LoopTermination::NoReplan;
                        break;
                    }
                }
            }
        }

        // --- Cleanup checkpoint on convergence ---
        if converged {
            if let Some(ref checkpoint_dir) = loop_config.checkpoint_dir {
                if let Err(e) = RunCheckpoint::cleanup(checkpoint_dir) {
                    tracing::warn!(error = %e, "checkpoint: cleanup failed");
                }
            }
        } else {
            // --- Write final checkpoint for non-converged runs ---
            // This enables resume from last state
            if let Some(ref checkpoint_dir) = loop_config.checkpoint_dir {
                let checkpoint = RunCheckpoint {
                    goal: goal.statement.clone(),
                    comprehension: comprehension.clone(),
                    iterations: iterations.clone(),
                    last_plan: current_plan.clone(),
                    last_steps: step_results.clone(),
                    last_critique: current_critique.clone(),
                    failure_tracker: failure_tracker.clone(),
                    total_usage: total_usage.clone(),
                    converged: false,
                    termination: termination.clone(),
                    seen_signatures: seen_signatures.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                if let Err(e) = checkpoint.write(checkpoint_dir) {
                    tracing::warn!(error = %e, "checkpoint: failed to write final checkpoint");
                }
            }
        }

        // --- Build Plan for the result ---
        // Use the real plan if Plan stage ran, otherwise fall back to synthetic.
        let plan = current_plan.unwrap_or_else(|| Plan {
            goal: goal.to_string(),
            goal_statement: goal.statement.clone(),
            criteria: goal.acceptance_criteria.clone(),
            subtasks: vec![Subtask {
                id: "work".into(),
                description: goal.statement.clone(),
                tier: TaskTier::Mid,
                kind: SubtaskKind::Implement,
                files: goal.target_files.clone(),
                acceptance_criteria: goal.acceptance_criteria.clone(),
            }],
            tdd: false,
            risks: vec![],
            schema_version: "1.0".into(),
            complexity,
        });

        let final_result = AgentRunResult {
            goal: goal.statement.clone(),
            comprehension,
            plan,
            step_results,
            critique: current_critique,
            decision: None,
            runbook: None,
            total_usage: total_usage.clone(),
        };

        let outcome_summary = if converged {
            format!(
                "Completed successfully in {} iteration(s)",
                iterations.len()
            )
        } else {
            format!(
                "Stopped after {} iteration(s) - {}",
                iterations.len(),
                match termination {
                    crate::cognition::LoopTermination::Approved => "approved",
                    crate::cognition::LoopTermination::MaxIterations => "max iterations reached",
                    crate::cognition::LoopTermination::NoReplan => "no replan",
                    crate::cognition::LoopTermination::SpendCapExceeded(_) => "budget exceeded",
                    crate::cognition::LoopTermination::Oscillation => "oscillation detected",
                    crate::cognition::LoopTermination::ModelNotConverging(_) =>
                        "model not converging",
                    crate::cognition::LoopTermination::Aborted(_) => "aborted",
                }
            )
        };
        Self::emit_event(events, LoopEvent::Done { outcome_summary });

        Ok(LoopResult {
            goal: goal.statement.clone(),
            iterations,
            converged,
            termination,
            total_usage,
            grader_source: "simple".into(),
            final_result,
        })
    }

    pub async fn resume_loop(
        &self,
        goal: &crate::goal::GoalSpec,
        loop_config: &LoopConfig,
    ) -> Result<LoopResult, AgentError> {
        let checkpoint_dir = loop_config.checkpoint_dir.as_ref().ok_or_else(|| {
            AgentError::Checkpoint("no checkpoint_dir configured for resume".into())
        })?;

        let checkpoint = RunCheckpoint::load(checkpoint_dir)
            .map_err(|e| AgentError::Checkpoint(format!("failed to load checkpoint: {e}")))?;

        tracing::info!(
            goal = %checkpoint.goal,
            iteration = checkpoint.iterations.len(),
            timestamp = %checkpoint.timestamp,
            "resume_loop: loaded checkpoint"
        );

        if checkpoint.goal != goal.statement {
            tracing::warn!(
                checkpoint_goal = %checkpoint.goal,
                requested_goal = %goal.statement,
                "resume_loop: goal mismatch — checkpoint goal differs from requested goal"
            );
        }

        // If converged, return checkpoint result directly.
        if checkpoint.converged {
            tracing::info!("resume_loop: checkpoint already converged — nothing to resume");
            let final_result = AgentRunResult {
                goal: checkpoint.goal,
                comprehension: checkpoint.comprehension,
                plan: checkpoint.last_plan.unwrap_or_else(|| Plan {
                    goal: String::new(),
                    goal_statement: goal.statement.clone(),
                    criteria: Vec::new(),
                    subtasks: Vec::new(),
                    tdd: false,
                    risks: Vec::new(),
                    schema_version: "1.0".into(),
                    complexity: TaskComplexity::Simple,
                }),
                step_results: checkpoint.last_steps,
                critique: checkpoint.last_critique,
                decision: None,
                runbook: None,
                total_usage: checkpoint.total_usage.clone(),
            };
            return Ok(LoopResult {
                goal: goal.statement.clone(),
                iterations: checkpoint.iterations,
                converged: true,
                termination: LoopTermination::Approved,
                total_usage: checkpoint.total_usage,
                grader_source: "checkpoint".to_string(),
                final_result,
            });
        }

        // Stale checkpoint from old pipeline: clean up and start fresh.
        tracing::info!("resume_loop: checkpoint is non-converged — cleaning up and starting fresh");
        if let Err(e) = RunCheckpoint::cleanup(checkpoint_dir) {
            tracing::warn!(error = %e, "resume_loop: failed to clean up stale checkpoint");
        }
        self.run_loop(goal, loop_config, None, None).await
    }
}
