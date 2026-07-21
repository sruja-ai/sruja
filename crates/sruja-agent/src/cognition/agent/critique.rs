use super::*;

use crate::cognition::config::CritiqueMode;
use crate::cognition::prompts;

impl Agent {
    // --- Critique: review every change via the review model ---

    /// Review changes via the configured critic ensemble.
    ///
    /// When `config.critique_personas` is non-empty, this fans out N
    /// probe-bound persona critics in parallel and merges them:
    /// - `approved` = AND of all personas (one blocker vetoes).
    /// - `issues`   = union, deduped, sorted (deterministic output).
    /// - `score`    = MIN across personas (a blocking persona drags the score
    ///   down, never averaged away).
    ///
    /// When `config.critique_personas` is empty, falls back to a single call
    /// with the legacy [`CRITIQUE_SYSTEM_PROMPT`] (backward compatible).
    ///
    /// Past guardrail learnings from agentic memory are injected into every
    /// persona's prompt as a "Known blind spots to probe for" section — the
    /// compounding loop that turns past misses into permanent probes.
    pub async fn critique(
        &self,
        plan: &Plan,
        results: &[StepResult],
    ) -> Result<Critique, AgentError> {
        if let HookAction::Abort(reason) = self.hooks.before_review().await {
            return Err(AgentError::HookAborted(reason));
        }

        let step_summary: Vec<String> = results
            .iter()
            .map(|r| {
                format!(
                    "- [{}] {:?}: {}",
                    r.subtask_id,
                    r.status,
                    truncate(&r.output, 200)
                )
            })
            .collect();

        // --- U2: diff-grounded critic (fixes G2) ---
        // Obtain the real git diff instead of relying solely on the actor's
        // self-report. The diff is the ground truth; step_summary is reframed
        // as "what the actor claims it did" — divergence between claims and
        // diff is itself a finding.
        let git_diff = self.get_git_diff().await;

        // --- U4: memory injection (compounding loop) ---
        // Retrieve past GUARDRAIL learnings and render them as blind-spot
        // probes in the critic prompt. Playbooks are excluded — they inform
        // planning, not review, and would bias the critic toward the actor's
        // prior successes. Retrievals are recorded so `retrieval_count` /
        // utility counters stay accurate for the critique path, not just
        // comprehension.
        let mut injected_learning_ids: Vec<String> = Vec::new();
        let blind_spots = if let Some(ref mem) = self.memory {
            let learnings = mem.search(&plan.goal_statement, 5, None);
            let guardrails: Vec<&LearningEntry> = learnings
                .iter()
                .filter(|l| l.kind == Some(crate::LearningKind::Guardrail))
                .collect();
            if guardrails.is_empty() {
                String::new()
            } else {
                injected_learning_ids = guardrails.iter().map(|g| g.id.clone()).collect();
                let ids: Vec<&str> = guardrails.iter().map(|g| g.id.as_str()).collect();
                mem.record_retrievals(&ids);
                let body = guardrails
                    .iter()
                    .map(|g| {
                        let util = g
                            .utility_ratio()
                            .map(|u| format!("{:.0}%", u * 100.0))
                            .unwrap_or_else(|| "?".to_string());
                        format!(
                            "- [retrieved {}x, util {}] {}\n  Probe: {}",
                            g.retrieval_count, util, g.context, g.guardrail_advice
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "\n\n## Known blind spots to actively probe for\n\
                     The following guardrails were learned from earlier runs. For EACH, \
                     state whether this change is exposed to it. If yes, that is a \
                     blocking issue.\n{}",
                    body
                )
            }
        } else {
            String::new()
        };

        // --- U2: assemble shared context with diff as ground truth ---
        let diff_section = match &git_diff {
            Some(diff) => format!("\n\n## Actual Diff (ground truth)\n```diff\n{}\n```", diff),
            None => "\n\n## Actual Diff\n[diff-unavailable: not a git repository or diff failed]"
                .to_string(),
        };

        let shared_user = format!(
            "## Goal\n{}\n\n\
             ## Plan\n{}\n\n\
             ## What the actor claims it did (self-report, may be inaccurate)\n{}{}\n{}",
            plan.goal_statement,
            plan.subtasks
                .iter()
                .map(|s| format!("- [{}] {} ({:?})", s.id, s.description, s.tier))
                .collect::<Vec<_>>()
                .join("\n"),
            step_summary.join("\n"),
            diff_section,
            blind_spots,
        );

        let personas = self.config.critique_personas.clone();

        let mut critique = if personas.is_empty() {
            // Backward-compatible single-critic fallback (KD7).
            let user = format!(
                "{}\n\n## Instructions\n\
                 Review this change as a senior architect. Check:\n\
                 1. Does the output match the goal?\n\
                 2. Are acceptance criteria met?\n\
                 3. Any architectural violations or risks?\n\
                 4. Should this be approved or rejected?\n\n\
                 Respond with JSON: {{\"approved\": bool, \"score\": 0.0-1.0, \
                 \"issues\": [...], \"suggestions\": [...]}}",
                shared_user,
            );
            let req = CompletionRequest::prompt(prompts::CRITIQUE_SYSTEM_PROMPT, &user)
                .with_model(&self.config.models.review);
            let response = self.llm.complete(&req).await?;
            let mut c = parse_critique_from_response(&response.content, response.usage.clone());
            c.source = "legacy".to_string();
            c
        } else if self.config.critique_mode == CritiqueMode::Full {
            // Full ensemble mode: always run all personas.
            self.run_persona_ensemble(personas, &shared_user).await?
        } else {
            // Tiered mode: run quick check first.
            let quick_req = CompletionRequest::prompt(prompts::QUICK_CRITIQUE_PROMPT, &shared_user)
                .with_model(&self.config.models.review);
            let quick_resp = self.llm.complete(&quick_req).await?;
            let quick_critique =
                parse_critique_from_response(&quick_resp.content, quick_resp.usage.clone());

            if self.config.critique_mode == CritiqueMode::QuickOnly {
                let mut c = quick_critique;
                c.source = "quick_check".to_string();
                c
            } else if quick_critique.approved
                && quick_critique.score >= self.config.quick_critique_threshold
            {
                // Quick check passed with high confidence — skip the ensemble.
                let mut c = quick_critique;
                c.source = "quick_check".to_string();
                c
            } else {
                // Quick check didn't clear the threshold — run full ensemble.
                let mut c = self.run_persona_ensemble(personas, &shared_user).await?;
                c.source = "ensemble".to_string();
                c
            }
        };

        critique.injected_learning_ids = injected_learning_ids;

        if let HookAction::Abort(reason) = self.hooks.after_review(&critique).await {
            return Err(AgentError::HookAborted(reason));
        }

        Ok(critique)
    }

    /// Obtain the real git diff via the shell tool (U2: diff-grounded critic).
    ///
    /// Returns `Some(diff_text)` on success, `None` if not a git repo or on
    /// error (graceful degradation — the critic falls back to step_summary).
    /// The diff is truncated to a token budget to avoid overwhelming the prompt.
    pub(super) async fn get_git_diff(&self) -> Option<String> {
        let params = serde_json::json!({
            "command": "git",
            "args": ["diff", "HEAD"],
            "timeout_ms": 10_000,
        });
        match self.tools.dispatch("shell", params).await {
            Ok(output) => {
                // Parse the shell output format: "exit: {code}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
                let stdout = output
                    .split("--- stdout ---\n")
                    .nth(1)?
                    .split("\n--- stderr ---")
                    .next()?
                    .trim();
                if stdout.is_empty() {
                    None
                } else {
                    // Truncate to 12k chars (matching existing token budget patterns)
                    Some(truncate(stdout, 12_000))
                }
            }
            Err(_) => None,
        }
    }

    /// Fan out the persona ensemble in parallel and merge results.
    ///
    /// Each persona runs as an independent task with its own prompt + the
    /// shared context. Independence of *perspective* (separate prompts) is the
    /// point; parallel execution is a latency win. Errors from any persona
    /// abort the critique (a partial ensemble would silently weaken the gate).
    async fn run_persona_ensemble(
        &self,
        personas: Vec<CritiquePersona>,
        shared_user: &str,
    ) -> Result<Critique, AgentError> {
        let llm = self.llm.clone();
        let review_model = self.config.models.review.clone();

        let mut set = tokio::task::JoinSet::new();
        for persona in personas {
            let llm = llm.clone();
            let user = shared_user.to_string();
            let model = persona
                .model
                .clone()
                .unwrap_or_else(|| review_model.clone());
            set.spawn(async move {
                let req = CompletionRequest::prompt(&persona.system_prompt, user).with_model(model);
                let response = llm.complete(&req).await?;
                let parsed = parse_critique_from_response(&response.content, response.usage);
                Ok::<(CritiquePersona, Critique), LlmError>((persona, parsed))
            });
        }

        let mut persona_results: Vec<PersonaResult> = Vec::new();
        let mut approved = true;
        let mut score = 1.0_f64;
        let mut issues: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut suggestions: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut all_criteria: std::collections::HashMap<usize, CriterionStatus> =
            std::collections::HashMap::new();
        let mut usage = Usage::default();

        while let Some(join_res) = set.join_next().await {
            let (persona, parsed) = match join_res {
                Ok(Ok(pair)) => pair,
                Ok(Err(e)) => return Err(AgentError::Llm(e)),
                Err(e) => return Err(AgentError::Other(format!("critique task panicked: {e}"))),
            };
            persona_results.push(PersonaResult {
                id: persona.id,
                approved: parsed.approved,
                score: parsed.score,
                issues: parsed.issues.clone(),
            });
            approved &= parsed.approved;
            score = score.min(parsed.score);
            for issue in parsed.issues {
                issues.insert(issue);
            }
            for s in parsed.suggestions {
                suggestions.insert(s);
            }
            // U3: Merge coverage matrix from spec_coverage persona
            for criterion in parsed.criteria {
                // Use the worst verdict for each criterion (missing > partial > addressed)
                let entry = all_criteria
                    .entry(criterion.index)
                    .or_insert(criterion.clone());
                if criterion.status == CriterionVerdict::Missing
                    || (criterion.status == CriterionVerdict::Partial
                        && entry.status == CriterionVerdict::Addressed)
                {
                    *entry = criterion;
                }
            }
            usage.accumulate(&parsed.usage);
        }

        // U3: Check coverage matrix — any missing or partial criterion blocks approval
        let mut criteria_vec: Vec<CriterionStatus> = all_criteria.into_values().collect();
        criteria_vec.sort_by_key(|c| c.index);
        for criterion in &criteria_vec {
            if criterion.status == CriterionVerdict::Missing {
                approved = false;
                score = score.min(0.0);
                issues.insert(format!(
                    "criterion #{} '{}': missing",
                    criterion.index, criterion.criterion
                ));
            } else if criterion.status == CriterionVerdict::Partial {
                approved = false;
                score = score.min(0.5);
                issues.insert(format!(
                    "criterion #{} '{}': partial",
                    criterion.index, criterion.criterion
                ));
            }
        }

        let mut issues_vec: Vec<String> = issues.into_iter().collect();
        issues_vec.sort();
        let mut suggestions_vec: Vec<String> = suggestions.into_iter().collect();
        suggestions_vec.sort();

        Ok(Critique {
            approved,
            score,
            issues: issues_vec,
            suggestions: suggestions_vec,
            usage,
            persona_breakdown: persona_results,
            injected_learning_ids: Vec::new(),
            criteria: criteria_vec,
            source: "ensemble".to_string(),
        })
    }
}
