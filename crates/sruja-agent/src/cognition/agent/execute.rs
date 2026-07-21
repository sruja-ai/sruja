use super::*;

use crate::cognition::prompts;

impl Agent {
    // --- Tool-calling loop (shared by all phases) ---

    /// Runs the main LLM tool-calling loop, repeatedly invoking the LLM and
    /// dispatching tool calls until the model stops requesting tools or the
    /// configured iteration limit is reached.
    ///
    /// # When to use
    ///
    /// This is the primary entry-point for non-streaming phases (`comprehend`,
    /// `plan`, `execute`, `reflect`, etc.).  Each phase builds a
    /// [`CompletionRequest`], hands it here, and consumes the returned
    /// response to decide what to do next.
    ///
    /// If you need a lower iteration cap for a lightweight or low-stakes task
    /// (e.g. a quick comment-only edit), call [`run_tool_loop_with_limit`]
    /// directly with a custom limit instead.
    ///
    /// # Relationship to [`run_tool_loop_with_limit`]
    ///
    /// `run_tool_loop` is a thin convenience wrapper around
    /// [`run_tool_loop_with_limit`](Self::run_tool_loop_with_limit) that
    /// forwards `self.config.max_tool_iterations` as the limit.  All
    /// iteration, convergence-pressure, and graceful-degradation logic lives
    /// in the `_with_limit` variant; this method simply picks the default.
    ///
    /// # Returns
    ///
    /// A tuple of:
    /// * **`CompletionResponse`** — the final LLM response whose content is
    ///   the answer (tool-calling responses are consumed inside the loop).
    /// * **`Usage`** — cumulative prompt, completion, and total token counts
    ///   across every LLM call made during the loop.
    /// * **`Vec<ToolSignal>`** — a per-call list of [`ToolSignal`] outcomes
    ///   (ok, empty, error, etc.) that downstream executors fold into
    ///   [`StepResult`].
    pub async fn run_tool_loop(
        &self,
        req: CompletionRequest,
    ) -> Result<(CompletionResponse, Usage, Vec<ToolSignal>), AgentError> {
        let (response, usage, signals, _converged) = self
            .run_tool_loop_with_limit(req, self.config.max_tool_iterations)
            .await?;
        Ok((response, usage, signals))
    }

    /// Run the LLM tool-calling loop with an explicit iteration limit.
    ///
    /// Use this to cap iterations for trivial tasks (e.g. max 3 for a comment
    /// change). Pass `0` or omit to use the agent's configured default.
    ///
    /// The loop is also bounded by a wall-clock timeout (`config.loop_timeout_secs`)
    /// to prevent indefinite hangs when tools or LLM calls are slow.
    pub async fn run_tool_loop_with_limit(
        &self,
        req: CompletionRequest,
        max_iterations: usize,
    ) -> Result<(CompletionResponse, Usage, Vec<ToolSignal>, bool), AgentError> {
        let timeout = std::time::Duration::from_secs(self.config.loop_timeout_secs);
        match tokio::time::timeout(timeout, self.run_tool_loop_inner(req, max_iterations)).await {
            Ok(result) => result,
            Err(_) => {
                tracing::warn!(
                    timeout_secs = self.config.loop_timeout_secs,
                    "tool_loop: wall-clock timeout exceeded"
                );
                Err(AgentError::Timeout(self.config.loop_timeout_secs))
            }
        }
    }

    /// Stream an LLM completion and dispatch tool calls the instant they arrive,
    /// overlapping tool execution with the tail of generation.
    ///
    /// This is the Claude Code "streaming tool execution" pattern: instead of
    /// blocking on a single `complete()` call and only then running tools, we
    /// open the stream, accumulate tool calls as their argument JSON completes,
    /// and run each one concurrently while the model keeps generating.
    ///
    /// Model-agnostic: every [`LlmClient`] implements `complete_stream` (the
    /// default buffers non-streaming providers into a correct event stream), so
    /// this path works on any OpenAI-compatible endpoint.
    ///
    /// Returns the fully reassembled final [`CompletionResponse`] plus the
    /// accumulated tool results keyed by tool-call id in arrival order.
    async fn stream_and_dispatch(
        &self,
        req: CompletionRequest,
    ) -> Result<
        (
            CompletionResponse,
            Usage,
            Vec<(
                String,
                String,
                serde_json::Value,
                Result<(String, crate::tool::ToolCallRecord), crate::tool::ToolError>,
            )>,
        ),
        AgentError,
    > {
        use crate::llm::stream::StreamEvent;
        use futures::StreamExt;

        let model = req.model.clone().unwrap_or_default();
        let mut stream = self.llm.complete_stream(&req);

        // Accumulators for reassembly.
        let mut content = String::new();
        let mut usage = Usage::default();
        let mut finish_reason = crate::llm::FinishReason::Stop;

        // Tool-call assembly buffers (args arrive as JSON-string fragments).
        let mut accs: std::collections::BTreeMap<usize, (Option<String>, Option<String>, String)> =
            Default::default();

        /// A dispatched tool task: (tool name, arguments JSON, join handle for result).
        type DispatchedTask = (
            String,
            serde_json::Value,
            tokio::task::JoinHandle<
                Result<(String, crate::tool::ToolCallRecord), crate::tool::ToolError>,
            >,
        );

        // Dispatched tasks keyed by call id.
        let mut dispatched: std::collections::HashMap<String, DispatchedTask> =
            std::collections::HashMap::new();

        while let Some(event) = stream.next().await {
            let event = event.map_err(|e| AgentError::Other(e.to_string()))?;
            match event {
                StreamEvent::ContentDelta(s) => content.push_str(&s),
                StreamEvent::ToolCallStart { index, id, name } => {
                    let entry = accs.entry(index).or_default();
                    entry.0 = Some(id);
                    entry.1 = Some(name);
                }
                StreamEvent::ToolCallArguments { index, fragment } => {
                    let entry = accs.entry(index).or_default();
                    entry.2.push_str(&fragment);
                    // Dispatch the moment args parse as valid complete JSON.
                    if let Some((id, name, buf)) = accs.get(&index).map(|e| {
                        (
                            e.0.clone().unwrap_or_default(),
                            e.1.clone().unwrap_or_default(),
                            e.2.clone(),
                        )
                    }) {
                        if !dispatched.contains_key(&id) {
                            if let Ok(args) = serde_json::from_str::<serde_json::Value>(&buf) {
                                let name_c = name.clone();
                                let args_c = args.clone();
                                let id_c = id.clone();
                                let tools = self.tools.clone();
                                let handle = tokio::spawn(async move {
                                    tools.dispatch_record(&name_c, args_c).await
                                });
                                dispatched.insert(id_c, (name, args, handle));
                            }
                        }
                    }
                }
                StreamEvent::Usage(u) => usage = u,
                StreamEvent::Finish { finish_reason: fr } => {
                    finish_reason = fr;
                    // Finalize any tool call whose args never parsed mid-stream.
                    for (id, name, buf) in accs.values() {
                        let id = id.clone().unwrap_or_default();
                        if dispatched.contains_key(&id) {
                            continue;
                        }
                        let args = serde_json::from_str(buf).unwrap_or(serde_json::json!({}));
                        let name_c = name.clone().unwrap_or_default();
                        let args_c = args.clone();
                        let id_c = id.clone();
                        let tools = self.tools.clone();
                        let name_in_closure = name_c.clone();
                        let handle = tokio::spawn(async move {
                            tools.dispatch_record(&name_in_closure, args_c).await
                        });
                        dispatched.insert(id_c, (name_c, args, handle));
                    }
                }
            }
        }

        // Wait for all dispatched tool tasks.
        let mut tool_results = Vec::new();
        for (id, (_name, args, handle)) in dispatched.into_iter() {
            let name = _name;
            let result = handle
                .await
                .map_err(|e| AgentError::Other(format!("tool task join: {e}")))?;
            tool_results.push((id, name, args, result));
        }

        // Reassemble tool_calls in index order for the final response.
        let mut ordered: Vec<(usize, crate::llm::ToolCall)> = Vec::new();
        for (idx, (id, name, buf)) in accs {
            let arguments = serde_json::from_str(&buf).unwrap_or(serde_json::json!({}));
            ordered.push((
                idx,
                crate::llm::ToolCall {
                    id: id.unwrap_or_default(),
                    name: name.unwrap_or_default(),
                    arguments,
                },
            ));
        }
        ordered.sort_by_key(|(idx, _)| *idx);
        let tool_calls = ordered.into_iter().map(|(_, tc)| tc).collect();

        let response = CompletionResponse {
            content,
            tool_calls,
            usage: usage.clone(),
            model,
            finish_reason,
        };

        Ok((response, usage, tool_results))
    }

    /// Inner implementation of the tool loop (extracted for timeout wrapping).
    ///
    /// Returns `(response, usage, tool_signals, converged)`. When `converged`
    /// is false, the model never stopped calling tools on its own and the
    /// fallback path was taken — the output may be incomplete.
    async fn run_tool_loop_inner(
        &self,
        mut req: CompletionRequest,
        max_iterations: usize,
    ) -> Result<(CompletionResponse, Usage, Vec<ToolSignal>, bool), AgentError> {
        let mut total_usage = Usage::default();
        let mut tool_signals: Vec<ToolSignal> = Vec::new();
        let mut last_response: Option<CompletionResponse> = None;
        let mut soft_sent = false;
        let mut hard_sent = false;
        // Circuit breaker: track consecutive tool errors and inject recovery.
        let mut consecutive_errors: usize = 0;
        let mut recovery_sent = false;
        // Track consecutive iterations where the model only called tools
        // (no meaningful content). After 3, abort early — the model is
        // stuck in a tool-calling loop it won't exit.
        let mut consecutive_tool_only: usize = 0;
        // Track repeated tool+arg signatures to detect stuck loops
        // even when the model produces some text. Same tool with same
        // args 3+ times = loop.
        let mut last_tool_signature: Option<String> = None;
        let mut consecutive_same_tool_call: usize = 0;
        // Progress tracking: track actual changes made
        let mut file_changes: usize = 0;
        let mut successful_tool_calls: usize = 0;
        let mut last_file_change_iteration: Option<usize> = None;

        for iteration in 0..max_iterations {
            // Stream the completion and dispatch tool calls as they arrive,
            // overlapping tool execution with the tail of generation.
            let (response, _usage, tool_results) = self.stream_and_dispatch(req.clone()).await?;
            total_usage.prompt_tokens += response.usage.prompt_tokens;
            total_usage.completion_tokens += response.usage.completion_tokens;
            total_usage.total_tokens += response.usage.total_tokens;

            let tool_names: Vec<&str> = response
                .tool_calls
                .iter()
                .map(|c| c.name.as_str())
                .collect();
            let content_preview: String = response.content.chars().take(120).collect();
            tracing::info!(
                iteration,
                finish_reason = ?response.finish_reason,
                tool_calls = tool_names.len(),
                tool_names = ?tool_names,
                content_preview = %content_preview,
                "tool_loop: LLM response"
            );
            // Termination: stop when the model stops requesting tools.
            // Also stop when finish_reason is Stop — some OpenAI-compatible
            // servers emit tool_calls alongside a "stop" finish_reason.
            if !response.wants_tools() || response.finish_reason == crate::llm::FinishReason::Stop {
                if response.wants_tools() {
                    tracing::warn!(
                        iteration,
                        tool_names = ?tool_names,
                        "tool_loop: finish_reason=Stop but tool_calls present — \
                         treating content as final answer (server quirk)"
                    );
                    let mut response = response;
                    response.tool_calls.clear();
                    return Ok((response, total_usage, tool_signals, true));
                }
                return Ok((response, total_usage, tool_signals, true));
            }

            // Track tool-only iterations for early-abort.
            if response.content.trim().is_empty() {
                consecutive_tool_only += 1;
            } else {
                consecutive_tool_only = 0;
            }
            if self.config.max_consecutive_tool_only > 0
                && consecutive_tool_only >= self.config.max_consecutive_tool_only
            {
                // Check if we're making progress (file changes)
                let recent_file_change = last_file_change_iteration
                    .map(|last| iteration.saturating_sub(last) <= 2)
                    .unwrap_or(false);

                if recent_file_change || file_changes > 0 {
                    // We're making progress, don't abort
                    tracing::info!(
                        iteration,
                        consecutive_tool_only,
                        file_changes,
                        "tool_loop: model making progress despite tool-only iterations"
                    );
                    consecutive_tool_only = 0; // Reset counter
                } else {
                    tracing::warn!(
                        iteration,
                        consecutive_tool_only,
                        file_changes,
                        "tool_loop: model has called tools 3+ times with no output — aborting early"
                    );
                    let mut fallback = last_response.unwrap_or_else(|| {
                        CompletionResponse::text(
                            "ERROR: model stuck in tool-calling loop with no output.",
                        )
                    });
                    fallback.tool_calls.clear();
                    fallback.finish_reason = crate::llm::FinishReason::Stop;
                    return Ok((fallback, total_usage, tool_signals, false));
                }
            }

            last_response = Some(response.clone());

            // Push the assistant's tool-call message.
            req.messages.push(Message {
                role: crate::llm::MessageRole::Assistant,
                content: response.content.clone(),
                tool_calls: response.tool_calls.clone(),
                tool_call_id: None,
            });

            // Tool calls were already dispatched (concurrently, as they streamed
            // in). Map the streamed results into the per-call handling below.
            let call_id_to_result: std::collections::HashMap<
                String,
                Result<(String, crate::tool::ToolCallRecord), crate::tool::ToolError>,
            > = tool_results
                .into_iter()
                .map(|(id, _name, _args, res)| (id, res))
                .collect();

            for call in response.tool_calls.iter() {
                let result = call_id_to_result.get(&call.id);
                let (result, record) = match result {
                    Some(Ok(ok)) => ok.clone(),
                    Some(Err(e)) => {
                        let record = crate::tool::ToolCallRecord {
                            ok: false,
                            empty: false,
                            elapsed_ms: 0,
                            source: crate::tool::ToolRegistry::classify_source(&call.name),
                            truncated: false,
                            payload: e.to_string(),
                        };
                        (format!("ERROR: {e}"), record)
                    }
                    None => {
                        let record = crate::tool::ToolCallRecord {
                            ok: false,
                            empty: false,
                            elapsed_ms: 0,
                            source: crate::tool::ToolRegistry::classify_source(&call.name),
                            truncated: false,
                            payload: "streamed tool result missing".to_string(),
                        };
                        ("ERROR: streamed tool result missing".to_string(), record)
                    }
                };
                tracing::debug!(
                    tool = %call.name,
                    args_preview = %call.arguments.to_string().chars().take(200).collect::<String>(),
                    "tool_loop: processing tool result"
                );
                // U5: emit tool_call event before dispatch (when tracing enabled).
                if self.config.enable_tool_call_tracing {
                    if let (
                        Some(ref tracer),
                        Some(ref repo),
                        Some(ref run_id),
                        Some(ref trace_id),
                    ) = (
                        &self.tool_call_tracer,
                        &self.repo_root,
                        &self.trace_run_id,
                        &self.trace_id,
                    ) {
                        let args_keys: Vec<String> = call
                            .arguments
                            .as_object()
                            .map(|m| {
                                let mut keys = m.keys().cloned().collect::<Vec<_>>();
                                keys.sort();
                                keys
                            })
                            .unwrap_or_default();
                        tracer.on_tool_call(repo, run_id, trace_id, &call.name, &args_keys);
                    }
                }

                let truncated_text = truncate(&result, 8_000);
                let was_truncated = result.len() > 8_000;
                let mut record = record;
                if was_truncated {
                    record.truncated = true;
                }
                tracing::debug!(
                    tool = %call.name,
                    result_len = truncated_text.len(),
                    result_preview = %truncated_text.chars().take(120).collect::<String>(),
                    "tool_loop: tool result"
                );
                tool_signals.push(ToolSignal {
                    tool: call.name.clone(),
                    ok: record.ok,
                    empty: record.empty,
                    elapsed_ms: record.elapsed_ms,
                    source: record.source,
                });

                // Track consecutive errors for circuit breaker.
                if !record.ok {
                    consecutive_errors += 1;
                } else {
                    consecutive_errors = 0;
                    successful_tool_calls += 1;
                }

                // Track file changes for progress detection
                if record.ok
                    && (call.name == "file_write"
                        || call.name == "file_edit"
                        || call.name == "diff_edit")
                {
                    file_changes += 1;
                    last_file_change_iteration = Some(iteration);
                    tracing::debug!(iteration, file_changes, "tool_loop: file change detected");
                }

                // U5: emit tool_result event after dispatch (when tracing enabled).
                if self.config.enable_tool_call_tracing {
                    if let (
                        Some(ref tracer),
                        Some(ref repo),
                        Some(ref run_id),
                        Some(ref trace_id),
                    ) = (
                        &self.tool_call_tracer,
                        &self.repo_root,
                        &self.trace_run_id,
                        &self.trace_id,
                    ) {
                        tracer.on_tool_result(
                            repo,
                            run_id,
                            trace_id,
                            &call.name,
                            record.ok,
                            record.empty,
                            record.elapsed_ms,
                        );
                    }
                }

                req.messages
                    .push(Message::tool_result(&call.id, truncated_text));
            }

            // ── Tool call signature tracking ────────────────────────────
            // Detect repeated tool+arg patterns to distinguish productive
            // exploration from stuck loops. Build a signature from (tool_name, arg_keys)
            // pairs — if the same signature appears 3+ times in a row, abort.
            {
                let sig: String = response
                    .tool_calls
                    .iter()
                    .map(|call| {
                        let keys: Vec<String> = call
                            .arguments
                            .as_object()
                            .map(|m| {
                                let mut k: Vec<String> = m.keys().cloned().collect();
                                k.sort();
                                k
                            })
                            .unwrap_or_default();
                        format!("{}:{:?}", call.name, keys)
                    })
                    .collect::<Vec<_>>()
                    .join("|");
                if last_tool_signature.as_deref() == Some(&sig) {
                    consecutive_same_tool_call += 1;
                } else {
                    consecutive_same_tool_call = 0;
                }
                last_tool_signature = Some(sig);
                if self.config.max_consecutive_same_call > 0
                    && consecutive_same_tool_call >= self.config.max_consecutive_same_call
                {
                    // Check if we're making progress (file changes)
                    let recent_file_change = last_file_change_iteration
                        .map(|last| iteration.saturating_sub(last) <= 2)
                        .unwrap_or(false);

                    if recent_file_change || file_changes > 0 {
                        // We're making progress, don't abort
                        tracing::info!(
                            iteration,
                            consecutive_same_tool_call,
                            file_changes,
                            "tool_loop: same tool+args called but making progress"
                        );
                        consecutive_same_tool_call = 0; // Reset counter
                    } else {
                        tracing::warn!(
                            iteration,
                            consecutive_same_tool_call,
                            "tool_loop: same tool+args called 3+ times in a row — aborting"
                        );
                        let mut fallback = last_response.unwrap_or_else(|| {
                            CompletionResponse::text(
                                "ERROR: model stuck repeating the same tool call with same arguments.",
                            )
                        });
                        fallback.tool_calls.clear();
                        fallback.finish_reason = crate::llm::FinishReason::Stop;
                        return Ok((fallback, total_usage, tool_signals, false));
                    }
                }
            }

            // ── Circuit breaker: consecutive error recovery ──────────────
            // Self-Harness paper: "tool-error-triggered recovery injection"
            // When the model hits 3 consecutive tool failures, inject a
            // redirect: diagnose, try different approach, don't abandon work.
            if consecutive_errors >= 3 && !recovery_sent {
                recovery_sent = true;
                tracing::warn!(
                    iteration,
                    consecutive_errors,
                    "tool_loop: injecting error-recovery message (circuit breaker)"
                );
                req.messages.push(Message::user(
                    "You have had 3 consecutive tool errors. STOP and diagnose:\n\
                     1. What exactly is going wrong? Read the error message carefully.\n\
                     2. Is the file path correct? Does the file exist?\n\
                     3. Try a completely different approach.\n\
                     Do NOT retry the same command. Do NOT delete files.\n\
                     If you cannot fix the error, make your best attempt with \
                     what you know and write your final answer.",
                ));
                // Reset counter so recovery gets a fair chance.
                consecutive_errors = 0;
            }

            // ── Progress-based recovery injection ──────────────────────
            // If the model has been calling tools for a while without making
            // file changes, inject a message to help it get unstuck.
            if iteration >= 3 && file_changes == 0 && successful_tool_calls >= 3 {
                let tool_names: Vec<&str> = tool_signals
                    .iter()
                    .skip(tool_signals.len().saturating_sub(3))
                    .map(|s| s.tool.as_str())
                    .collect();
                let has_read = tool_names.iter().any(|t| t.contains("read"));
                let has_write = tool_names
                    .iter()
                    .any(|t| t.contains("write") || t.contains("edit"));

                if has_read && !has_write {
                    tracing::info!(
                        iteration,
                        successful_tool_calls,
                        file_changes,
                        "tool_loop: injecting progress recovery message"
                    );
                    req.messages.push(Message::user(
                        "You have been reading files but haven't made any changes yet. \
                         The goal requires code changes. Please:\n\
                         1. Identify the file(s) that need to be modified\n\
                         2. Use diff_edit, file_edit, or file_write to make the changes\n\
                         3. Don't just keep reading — take action!\n\
                         If you're unsure, make your best attempt and move on.",
                    ));
                }
            }

            // ── Tiered convergence pressure ─────────────────────────────
            // Two-stage pressure: soft reminder at 50% remaining, hard
            // cutoff at 25% remaining. This gives models a chance to wrap
            // up gradually instead of a one-shot ultimatum.
            let remaining = max_iterations - iteration - 1;
            let quarter = (max_iterations / 4).max(1);
            let half = (max_iterations / 2).max(1);
            if remaining > 0 && remaining <= quarter && !hard_sent {
                hard_sent = true;
                tracing::warn!(
                    iteration,
                    remaining,
                    "tool_loop: injecting hard convergence message"
                );
                req.messages.push(Message::user(prompts::CONVERGENCE_HARD));
            } else if remaining > 0 && remaining <= half && !soft_sent && !hard_sent {
                soft_sent = true;
                tracing::info!(
                    iteration,
                    remaining,
                    "tool_loop: injecting soft convergence reminder"
                );
                req.messages.push(Message::user(prompts::CONVERGENCE_SOFT));
            }
        }

        // Graceful degradation: the model didn't self-terminate.
        tracing::warn!(
            max_iterations,
            "tool_loop: model did not converge — returning last response as fallback"
        );
        let mut fallback = last_response.unwrap_or_else(|| {
            CompletionResponse::text(
                "ERROR: tool loop exhausted without any response from the model.",
            )
        });
        fallback.tool_calls.clear();
        fallback.finish_reason = crate::llm::FinishReason::Stop;
        Ok((fallback, total_usage, tool_signals, false))
    }

    /// Resolve the model name configured for a complexity tier.
    fn model_for_tier(&self, tier: TaskTier) -> &str {
        match tier {
            TaskTier::Cheap => &self.config.models.cheap,
            TaskTier::Mid => &self.config.models.mid,
            TaskTier::Premium => &self.config.models.premium,
        }
    }

    /// Route a request to the model configured for a specific tier.
    pub async fn complete_tiered(
        &self,
        tier: TaskTier,
        req: CompletionRequest,
    ) -> Result<CompletionResponse, AgentError> {
        let model = self.model_for_tier(tier);
        let req = CompletionRequest {
            model: Some(model.to_string()),
            ..req
        };
        Ok(self.llm.complete(&req).await?)
    }

    // --- Execute: run subtasks with phase enforcement ---

    /// Execute a plan, enforcing the TDD phase pipeline.
    ///
    /// Each subtask transitions the [`FileGuard`] to the appropriate phase,
    /// ensuring tests and code are never in flux simultaneously.
    pub async fn execute(&self, plan: &Plan) -> Result<Vec<StepResult>, AgentError> {
        let mut results = Vec::new();

        for step in &plan.subtasks {
            // Transition the file guard to this subtask's phase.
            let phase = step.kind.phase();
            self.guard.set_phase(phase);
            self.hooks.on_phase_change(phase).await;

            // Hook gate.
            match self.hooks.before_step(step).await {
                HookAction::Skip => {
                    results.push(StepResult {
                        subtask_id: step.id.clone(),
                        status: StepStatus::Skipped,
                        output: "skipped by hook".into(),
                        usage: Usage::default(),
                        tool_signals: Vec::new(),
                        converged: true,
                    });
                    continue;
                }
                HookAction::Abort(reason) => {
                    return Err(AgentError::HookAborted(reason));
                }
                HookAction::Continue => {}
            }

            // Route to the appropriate model tier. The tool loop runs on the
            // tiered model so mutations respect the configured complexity
            // routing — no separate one-shot call beforehand.
            let tier = step.tier;
            let system = prompts::EXECUTION_SYSTEM_PROMPT;
            let execute_instruction = match plan.complexity {
                TaskComplexity::Trivial => {
                    "Make the change directly. Read the file, make the edit, and stop. \
                     Do NOT read other files. Do NOT explore."
                }
                _ => {
                    "Execute this subtask using the available tools. \
                     Be precise. Cite evidence."
                }
            };
            let user = format!(
                "## Subtask: {}\n\n\
                 ## Description\n{}\n\n\
                 ## Acceptance Criteria\n{}\n\n\
                 ## Phase\n{:?}\n\n\
                 {execute_instruction}",
                step.id,
                step.description,
                step.acceptance_criteria.join("\n"),
                phase,
            );

            let mut req = CompletionRequest::prompt(system, user).with_tools(self.tools.schemas());
            req.model = Some(self.model_for_tier(tier).to_string());

            let max_iters = match plan.complexity {
                TaskComplexity::Trivial => 5,
                TaskComplexity::Simple => 8,
                _ => self.config.max_tool_iterations,
            };
            let (response, tool_usage, tool_signals, step_converged) =
                self.run_tool_loop_with_limit(req, max_iters).await?;

            let status = if response.content.contains("ERROR")
                || tool_signals.iter().any(|s| !s.ok || s.empty)
            {
                StepStatus::Failed
            } else {
                StepStatus::Ok
            };

            let result = StepResult {
                subtask_id: step.id.clone(),
                status,
                output: response.content,
                usage: tool_usage,
                tool_signals,
                converged: step_converged,
            };

            self.hooks.after_step(step, &result).await;
            results.push(result);
        }

        // After all subtasks, reset to Comprehend phase.
        self.guard.set_phase(Phase::Comprehend);

        Ok(results)
    }
}
