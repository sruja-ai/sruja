use super::*;

use crate::cognition::prompts;

impl Agent {
    // --- Plan: complexity-tagged subtask decomposition ---

    /// Produce a plan with complexity-tagged subtasks.
    ///
    /// If `config.tdd` is true, test subtasks always precede implementation
    /// subtasks (TDD pipeline).
    pub async fn plan(
        &self,
        goal: &crate::goal::GoalSpec,
        comprehension: &Comprehension,
    ) -> Result<Plan, AgentError> {
        let goal_str = goal.statement.as_str();
        if let HookAction::Abort(reason) = self.hooks.before_plan(goal_str).await {
            return Err(AgentError::HookAborted(reason));
        }

        // Complexity-aware: skip TDD for trivial tasks even if config.tdd is on.
        let enforce_tdd = self.config.tdd && comprehension.complexity.enforce_tdd();

        let tdd_note = if enforce_tdd {
            "\n\nTDD MODE IS ON: You MUST emit test_author subtasks BEFORE any implement subtasks. \
             The framework enforces this — tests are written first, reviewed, then code is written \
             to pass the frozen tests. Tests and code are NEVER in flux simultaneously."
        } else {
            ""
        };

        // Complexity-aware prompt selection.
        let (system_prompt, plan_instructions) = match comprehension.complexity {
            TaskComplexity::Trivial => (
                prompts::PLAN_TRIVIAL_SYSTEM_PROMPT,
                "This is a trivial change (e.g. comment, typo, rename, format). \
                 Output a SINGLE implement subtask that directly makes the change. \
                 Do NOT add test, verify, or review subtasks. \
                 Do NOT call any tools — just output the plan JSON.\n",
            ),
            TaskComplexity::Simple => (
                prompts::PLAN_SYSTEM_PROMPT,
                "Break this goal into 1-2 concrete subtasks. Keep it minimal.\n",
            ),
            _ => (
                prompts::PLAN_SYSTEM_PROMPT,
                "Break this goal into concrete subtasks. Each subtask must specify:\n\
                 - `id`: a short unique identifier (e.g. \"s1\", \"s2\")\n\
                 - `description`: what to do (concise, actionable)\n\
                 - `tier`: cheap (classification/extraction), mid (standard coding), \
                  or premium (hard architecture reasoning)\n\
                 - `kind`: test_author, implement, verify, or review\n\
                 - `files`: list of files this subtask touches\n\
                 - `acceptance_criteria`: how to verify completion\n\n",
            ),
        };

        let user = format!(
            "## Goal\n{goal_str}\n\n\
             ## Comprehension\n{}\n\n\
             ## Architecture Elements Cited\n{:?}\n\n\
             ## Instructions\n\
             {plan_instructions}\
             Output a JSON object with `subtasks` array and `risks` array.\n\
             {tdd_note}",
            comprehension.summary, comprehension.cited_elements,
        );

        let mut req = CompletionRequest::prompt(system_prompt, user);
        // For trivial plans, do not attach tools — the prompt says "Do NOT
        // call any tools" and attaching schemas causes the model to ignore
        // that instruction and explore indefinitely.
        if !matches!(comprehension.complexity, TaskComplexity::Trivial) {
            req = req.with_tools(self.tools.schemas());
        }

        let max_iters = match comprehension.complexity {
            TaskComplexity::Trivial => 3,
            TaskComplexity::Simple => 5,
            _ => self.config.max_tool_iterations,
        };
        let (response, _usage, _signals, _converged) =
            self.run_tool_loop_with_limit(req, max_iters).await?;

        // Parse the plan from the LLM response.
        match parse_plan_from_response(&response.content, goal, enforce_tdd) {
            Ok(plan) => {
                tracing::warn!(
                    response_len = response.content.len(),
                    subtask_count = plan.subtasks.len(),
                    risks = plan.risks.len(),
                    "plan:parsed"
                );
                if plan.subtasks.is_empty() {
                    tracing::warn!(
                        raw_response = %response.content.chars().take(2000).collect::<String>(),
                        "plan:empty — model returned 0 subtasks"
                    );
                }

                let mut plan = plan;
                plan.complexity = comprehension.complexity;
                if let HookAction::Abort(reason) = self.hooks.after_plan(&mut plan).await {
                    return Err(AgentError::HookAborted(reason));
                }

                Ok(plan)
            }
            Err(parse_err) => {
                // Format-correction retry: issue one re-prompt with the parse reason.
                tracing::warn!(error = %parse_err, "plan:parse_failed — issuing correction re-prompt");
                let correction_user = format!(
                    "## Previous plan (rejected)\n{}\n\n\
                     ## Parse error\n{parse_err}\n\n\
                     ## Instructions\n\
                     The plan JSON above was rejected because: {parse_err}.\n\
                     Re-emit a VALID plan JSON. Each subtask MUST have `id`, `description`, \
                     `tier`, and `kind` fields. Output a JSON object with `subtasks` array \
                     and `risks` array.",
                    response.content,
                );
                let correction_req = CompletionRequest::prompt(system_prompt, correction_user);
                let correction_req = if matches!(comprehension.complexity, TaskComplexity::Trivial)
                {
                    correction_req
                } else {
                    correction_req.with_tools(self.tools.schemas())
                };
                let (retry_response, _retry_usage, _signals, _converged) = self
                    .run_tool_loop_with_limit(correction_req, max_iters)
                    .await?;
                let plan = parse_plan_from_response(&retry_response.content, goal, enforce_tdd)
                    .map_err(AgentError::PlanParseFailed)?;

                tracing::warn!(
                    response_len = retry_response.content.len(),
                    subtask_count = plan.subtasks.len(),
                    risks = plan.risks.len(),
                    "plan:parsed_after_correction"
                );

                let mut plan = plan;
                plan.complexity = comprehension.complexity;
                if let HookAction::Abort(reason) = self.hooks.after_plan(&mut plan).await {
                    return Err(AgentError::HookAborted(reason));
                }

                Ok(plan)
            }
        }
    }

    /// Re-plan using the prior critique as feedback.
    ///
    /// This is the feedback edge that closes the outer ReAct loop: when the
    /// independent critic rejects a change, its `issues` and `suggestions`
    /// are injected into a new plan rather than discarded.
    ///
    /// The `failure_tracker` accumulates failed approaches across iterations
    /// and injects them into the replanning prompt so the agent tries
    /// genuinely different strategies instead of repeating mistakes.
    pub async fn replan(
        &self,
        goal: &crate::goal::GoalSpec,
        comprehension: &Comprehension,
        critique: &Critique,
        convergence_pressure: Option<&str>,
        failure_tracker: &FailureTracker,
    ) -> Result<Plan, AgentError> {
        let goal_str = goal.statement.as_str();
        if let HookAction::Abort(reason) = self.hooks.before_plan(goal_str).await {
            return Err(AgentError::HookAborted(reason));
        }

        let tdd_note = if self.config.tdd && comprehension.complexity.enforce_tdd() {
            "\n\nTDD MODE IS ON: keep test_author subtasks BEFORE implement subtasks."
        } else {
            ""
        };

        let pressure_note = if let Some(pressure) = convergence_pressure {
            format!(
                "\n\n## CONVERGENCE PRESSURE\n\
                 The previous replan was flagged: {pressure}.\n\
                 You MUST change the subtasks or risks to address the critic's issues. \
                 Emitting an identical plan will be flagged again."
            )
        } else {
            String::new()
        };

        let plan_instructions: &str = match comprehension.complexity {
            TaskComplexity::Trivial => {
                "This is a trivial change (e.g. comment, typo, rename, format). \
                 Output a SINGLE implement subtask that directly makes the change. \
                 Do NOT add test, verify, or review subtasks. \
                 Do NOT call any tools — just output the plan JSON.\n"
            }
            _ => "Produce a revised plan that addresses the critic's feedback.\n",
        };
        let system_prompt: &str = match comprehension.complexity {
            TaskComplexity::Trivial => prompts::PLAN_TRIVIAL_SYSTEM_PROMPT,
            _ => prompts::PLAN_SYSTEM_PROMPT,
        };

        let failure_context = failure_tracker.format_for_prompt();

        // Structured critique JSON instead of flat text soup.
        let critique_json = serde_json::to_string_pretty(&serde_json::json!({
            "approved": critique.approved,
            "score": critique.score,
            "issues": critique.issues,
            "suggestions": critique.suggestions,
            "persona_breakdown": critique.persona_breakdown.iter().map(|p| {
                serde_json::json!({
                    "persona_id": p.id,
                    "approved": p.approved,
                    "issues": p.issues,
                })
            }).collect::<Vec<_>>(),
            "criteria_matrix": critique.criteria.iter().map(|c| {
                serde_json::json!({
                    "index": c.index,
                    "criterion": c.criterion,
                    "status": c.status,
                    "reason": c.reason,
                })
            }).collect::<Vec<_>>(),
        }))
        .unwrap_or_else(|_| "{}".to_string());

        let user = format!(
            "## Goal\n{goal_str}\n\n\
             ## Comprehension\n{}\n\n\
             ## Prior Review Outcome (Structured)\n\
             The independent critic REJECTED the previous attempt (score: {:.0}%).\n\
             Below is the full structured critique — each issue is tagged with\n\
             its originating persona, and the criteria matrix shows which\n\
             acceptance criteria are missing or partial.\n\n\
             ```json\n{critique_json}\n```\n\
             {failure_context}\
             ## Instructions\n\
             {plan_instructions}\
             Output a JSON object with `subtasks` array and `risks` array. \
             Do not repeat failed approaches. Try a DIFFERENT strategy. \
             Each criterion marked `missing` or `partial` in the criteria matrix \
             MUST be addressed by new subtasks.{tdd_note}{pressure_note}",
            comprehension.summary,
            critique.score * 100.0,
        );

        let mut req = CompletionRequest::prompt(system_prompt, user);
        if !matches!(comprehension.complexity, TaskComplexity::Trivial) {
            req = req.with_tools(self.tools.schemas());
        }

        let max_iters = match comprehension.complexity {
            TaskComplexity::Trivial => 3,
            TaskComplexity::Simple => 5,
            _ => self.config.max_tool_iterations,
        };
        let (response, _usage, _signals, _converged) =
            self.run_tool_loop_with_limit(req, max_iters).await?;

        match parse_plan_from_response(
            &response.content,
            goal,
            self.config.tdd && comprehension.complexity.enforce_tdd(),
        ) {
            Ok(plan) => {
                tracing::warn!(
                    response_len = response.content.len(),
                    subtask_count = plan.subtasks.len(),
                    "replan:parsed"
                );
                if plan.subtasks.is_empty() {
                    tracing::warn!(
                        raw_response = %response.content.chars().take(2000).collect::<String>(),
                        "replan:empty — model returned 0 subtasks"
                    );
                }

                let mut plan = plan;
                plan.complexity = comprehension.complexity;
                if let HookAction::Abort(reason) = self.hooks.after_plan(&mut plan).await {
                    return Err(AgentError::HookAborted(reason));
                }
                Ok(plan)
            }
            Err(parse_err) => {
                // Format-correction retry: issue one re-prompt with the parse reason.
                tracing::warn!(error = %parse_err, "replan:parse_failed — issuing correction re-prompt");
                let correction_user = format!(
                    "## Previous plan (rejected)\n{}\n\n\
                     ## Parse error\n{parse_err}\n\n\
                     ## Instructions\n\
                     The plan JSON above was rejected because: {parse_err}.\n\
                     Re-emit a VALID plan JSON. Each subtask MUST have `id`, `description`, \
                     `tier`, and `kind` fields. Output a JSON object with `subtasks` array \
                     and `risks` array.",
                    response.content,
                );
                let correction_req = CompletionRequest::prompt(system_prompt, correction_user);
                let correction_req = if matches!(comprehension.complexity, TaskComplexity::Trivial)
                {
                    correction_req
                } else {
                    correction_req.with_tools(self.tools.schemas())
                };
                let (retry_response, _retry_usage, _signals, _converged) = self
                    .run_tool_loop_with_limit(correction_req, max_iters)
                    .await?;
                let plan = parse_plan_from_response(
                    &retry_response.content,
                    goal,
                    self.config.tdd && comprehension.complexity.enforce_tdd(),
                )
                .map_err(AgentError::PlanParseFailed)?;

                tracing::warn!(
                    response_len = retry_response.content.len(),
                    subtask_count = plan.subtasks.len(),
                    "replan:parsed_after_correction"
                );

                let mut plan = plan;
                plan.complexity = comprehension.complexity;
                if let HookAction::Abort(reason) = self.hooks.after_plan(&mut plan).await {
                    return Err(AgentError::HookAborted(reason));
                }
                Ok(plan)
            }
        }
    }
}
