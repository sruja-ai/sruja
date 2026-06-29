    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::sync::Mutex;

    // Type alias for simpler test code
    type MockLlm = ScriptedLlm;

    // --- Ensemble critic tests (U1) ---
    /// Helper for scripted ensemble tests: a mock that returns different
    /// responses based on which persona's system prompt substring it matches.

    struct DropGuard(Arc<AtomicUsize>);
    impl Drop for DropGuard {
        fn drop(&mut self) {
            self.0.fetch_sub(1, Ordering::SeqCst);
        }
    }

    struct PersonaScriptedLlm {
        responses: Vec<PersonaResponse>,
        max_concurrent: Arc<AtomicUsize>,
        active: Arc<AtomicUsize>,
        last_system_prompt: Arc<Mutex<String>>,
        last_user_prompt: Arc<Mutex<String>>,
    }

    #[derive(Debug, Clone)]
    struct PersonaResponse {
        system_prompt_contains: &'static str,
        approved: bool,
        score: f64,
        issues: Vec<String>,
    }

    impl PersonaScriptedLlm {
        fn new(responses: Vec<PersonaResponse>) -> Arc<Self> {
            Arc::new(Self {
                responses,
                max_concurrent: Arc::new(AtomicUsize::new(0)),
                active: Arc::new(AtomicUsize::new(0)),
                last_system_prompt: Arc::new(Mutex::new(String::new())),
                last_user_prompt: Arc::new(Mutex::new(String::new())),
            })
        }

        #[allow(dead_code)]
        fn max_concurrent(&self) -> usize {
            self.max_concurrent.load(Ordering::SeqCst)
        }

        /// Returns the last system prompt the mock received, for test assertions.
        fn received_system_prompt(&self) -> String {
            self.last_system_prompt.lock().unwrap().clone()
        }

        /// Returns the last user prompt the mock received, for test assertions.
        fn received_user_prompt(&self) -> String {
            self.last_user_prompt.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl LlmClient for PersonaScriptedLlm {
        async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
            let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            let prev_max = self.max_concurrent.load(Ordering::SeqCst);
            if active > prev_max {
                self.max_concurrent.store(active, Ordering::SeqCst);
            }
            let _guard = DropGuard(self.active.clone());

            // Yield to allow other spawned tasks to start, proving parallelism.
            tokio::task::yield_now().await;

            let sys = req
                .messages
                .first()
                .map(|m| m.content.as_str())
                .unwrap_or("");
            *self.last_system_prompt.lock().unwrap() = sys.to_string();
            let user = req
                .messages
                .get(1)
                .map(|m| m.content.as_str())
                .unwrap_or("");
            *self.last_user_prompt.lock().unwrap() = user.to_string();
            let content = sys
                .lines()
                .find_map(|_l| {
                    self.responses
                        .iter()
                        .find(|r| sys.contains(r.system_prompt_contains))
                        .map(|r| {
                            format!(
                                r#"{{"approved":{},"score":{},"issues":{:?},"suggestions":[]}}"#,
                                r.approved, r.score, r.issues
                            )
                        })
                })
                .unwrap_or_else(|| {
                    r#"{"approved":false,"score":0.0,"issues":[],"suggestions":[]}"#.to_string()
                });

            Ok(CompletionResponse {
                content,
                tool_calls: Vec::new(),
                usage: Usage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
                model: "scripted-ensemble".into(),
                finish_reason: crate::llm::FinishReason::Stop,
            })
        }

        fn default_model(&self) -> &str {
            "scripted-ensemble"
        }
    }

    #[test]
    fn default_config_is_tdd_and_review() {
        let config = AgentConfig::default();
        assert!(config.tdd);
        assert!(config.review_every_change);
    }

    #[test]
    fn extract_ids_works() {
        let ids = extract_element_ids("See Sruja.CLI and Sruja.Graph.KnowledgeGraph for details.");
        assert!(ids.contains(&"Sruja.CLI".to_string()));
        assert!(ids.contains(&"Sruja.Graph.KnowledgeGraph".to_string()));
    }

    // --- Task complexity routing tests ---

    #[test]
    fn classify_trivial_comment_task() {
        let c = classify_task_complexity(
            "Add a short comment to the top of lib.rs",
            &["lib.rs".to_string()],
            &[],
        );
        assert_eq!(c, TaskComplexity::Trivial);
    }

    #[test]
    fn classify_trivial_typo_task() {
        let c = classify_task_complexity(
            "Fix typo in function name",
            &["main.rs".to_string()],
            &[],
        );
        assert_eq!(c, TaskComplexity::Trivial);
    }

    #[test]
    fn classify_trivial_rename_single_file() {
        let c = classify_task_complexity(
            "Rename variable foo to bar",
            &["handler.rs".to_string()],
            &[],
        );
        assert_eq!(c, TaskComplexity::Trivial);
    }

    #[test]
    fn classify_simple_small_change() {
        let c = classify_task_complexity(
            "Add input validation to the handler",
            &["handler.rs".to_string()],
            &[],
        );
        assert_eq!(c, TaskComplexity::Simple);
    }

    #[test]
    fn classify_complex_migration() {
        let c = classify_task_complexity(
            "Migrate the database schema",
            &[],
            &[],
        );
        assert_eq!(c, TaskComplexity::Complex);
    }

    #[test]
    fn classify_complex_many_elements() {
        let c = classify_task_complexity(
            "Update all API endpoints",
            &[],
            &["System.Api".to_string(), "System.Db".to_string(), "System.Auth".to_string()],
        );
        assert_eq!(c, TaskComplexity::Complex);
    }

    #[test]
    fn classify_moderate_multi_file() {
        let c = classify_task_complexity(
            "Add JWT auth to the API",
            &["auth.rs".to_string(), "middleware.rs".to_string(), "config.rs".to_string()],
            &["System.Api".to_string(), "System.Auth".to_string()],
        );
        assert_eq!(c, TaskComplexity::Moderate);
    }

    #[test]
    fn complex_keywords_override_trivial() {
        // "migrate" should override "add a comment" — architecture keywords win.
        let c = classify_task_complexity(
            "add a comment to migrate the database",
            &["migrations.rs".to_string()],
            &[],
        );
        assert_eq!(c, TaskComplexity::Complex);
    }

    #[test]
    fn trivial_skips_tdd_and_artifacts() {
        assert!(!TaskComplexity::Trivial.enforce_tdd());
        assert!(!TaskComplexity::Trivial.generate_artifacts());
    }

    #[test]
    fn simple_enforces_tdd_and_artifacts() {
        assert!(TaskComplexity::Simple.enforce_tdd());
        assert!(TaskComplexity::Simple.generate_artifacts());
    }

    #[test]
    fn trivial_caps_tool_iterations() {
        assert_eq!(TaskComplexity::Trivial.max_tool_iterations(8), 3);
        assert_eq!(TaskComplexity::Simple.max_tool_iterations(8), 5);
        assert_eq!(TaskComplexity::Moderate.max_tool_iterations(8), 8);
    }

    #[test]
    fn parse_plan_requires_id_field() {
        // Subtasks without `id` now fail with MissingRequiredField (U2).
        let raw = r#"{"subtasks":[
            {"description":"write add()","tier":"mid","kind":"implement","files":["src/main.rs"]}
        ],"risks":[]}"#;
        let err = parse_plan_from_response(raw, &crate::goal::GoalSpec::new("add function"), false)
            .unwrap_err();
        assert!(
            matches!(err, PlanParseError::MissingRequiredField { ref field, subtask_index: 0 } if field == "id"),
            "expected MissingRequiredField for id on subtask 0, got: {err}"
        );
    }

    #[test]
    fn parse_plan_error_on_missing_required_field() {
        // A subtask missing `tier` should fail with MissingRequiredField.
        let raw = r#"{"subtasks":[
            {"id":"s1","description":"ok","tier":"mid","kind":"implement"},
            {"id":"s2","description":"no tier here","kind":"verify"}
        ],"risks":[]}"#;
        let err =
            parse_plan_from_response(raw, &crate::goal::GoalSpec::new("test"), false).unwrap_err();
        assert!(
            matches!(err, PlanParseError::MissingRequiredField { ref field, subtask_index: 1 } if field == "tier"),
            "expected MissingRequiredField for tier on subtask 1, got: {err}"
        );
    }

    #[test]
    fn parse_plan_preserves_explicit_ids() {
        let raw = r#"{"subtasks":[
            {"id":"custom-id","description":"task","tier":"premium","kind":"review"}
        ],"risks":[]}"#;
        let plan =
            parse_plan_from_response(raw, &crate::goal::GoalSpec::new("test"), false).unwrap();
        assert_eq!(plan.subtasks.len(), 1);
        assert_eq!(plan.subtasks[0].id, "custom-id");
    }

    #[test]
    fn parse_plan_empty_array_returns_no_subtasks_error() {
        // Model returned valid JSON but with an empty subtasks array —
        // now returns a typed error (U2), not a synthesized fallback.
        let raw = r#"{"subtasks":[],"risks":["nothing to do"]}"#;
        let err =
            parse_plan_from_response(raw, &crate::goal::GoalSpec::new("add the function"), false)
                .unwrap_err();
        assert!(
            matches!(err, PlanParseError::NoSubtasks),
            "expected NoSubtasks, got: {err}"
        );
    }

    #[test]
    fn parse_plan_happy_path_with_all_fields() {
        let raw = r#"{"schema_version":"1.0","subtasks":[
            {"id":"s1","description":"write add()","tier":"mid","kind":"implement","files":["src/main.rs"],"acceptance_criteria":["it works"]}
        ],"risks":["none"]}"#;
        let plan = parse_plan_from_response(raw, &crate::goal::GoalSpec::new("add function"), true)
            .unwrap();
        assert_eq!(plan.subtasks.len(), 1);
        assert_eq!(plan.subtasks[0].id, "s1");
        assert_eq!(plan.schema_version, "1.0");
        assert!(plan.tdd);
        assert_eq!(plan.risks, vec!["none"]);
    }

    #[test]
    fn parse_plan_malformed_json_returns_error() {
        let err = parse_plan_from_response(
            "not json at all",
            &crate::goal::GoalSpec::new("test"),
            false,
        )
        .unwrap_err();
        assert!(
            matches!(err, PlanParseError::MalformedJson(_)),
            "expected MalformedJson, got: {err}"
        );
    }

    #[test]
    fn parse_plan_missing_subtasks_array_returns_error() {
        let raw = r#"{"risks":["none"]}"#;
        let err =
            parse_plan_from_response(raw, &crate::goal::GoalSpec::new("test"), false).unwrap_err();
        assert!(
            matches!(err, PlanParseError::MalformedJson(_)),
            "expected MalformedJson for missing subtasks, got: {err}"
        );
    }

    #[test]
    fn parse_plan_backward_compat_no_schema_version() {
        // Old serialized plans without schema_version deserialize fine.
        let raw = r#"{"subtasks":[
            {"id":"s1","description":"task","tier":"cheap","kind":"implement"}
        ],"risks":[]}"#;
        let plan =
            parse_plan_from_response(raw, &crate::goal::GoalSpec::new("test"), false).unwrap();
        assert_eq!(plan.schema_version, "");
        assert_eq!(plan.subtasks[0].id, "s1");
    }

    #[test]
    fn parse_critique_i_do_not_approve_does_not_flip_to_approved() {
        let raw = "I do not approve this plan; it's missing tests.";
        let critique = parse_critique_from_response(raw, Usage::default());
        assert!(!critique.approved, "'I do not approve' should be rejected");
        assert_eq!(critique.score, 0.3);
        assert!(critique
            .issues
            .contains(&"could not parse structured critique".to_string()));
    }

    #[test]
    fn parse_critique_approved_keyword_at_line_start_passes() {
        let raw = "Approved - the plan looks solid.";
        let critique = parse_critique_from_response(raw, Usage::default());
        assert!(critique.approved, "'Approved' at start should pass");
        assert_eq!(critique.score, 0.8);
    }

    #[test]
    fn parse_critique_approved_keyword_on_new_line_passes() {
        let raw = "I reviewed this.\nApproved - all good.";
        let critique = parse_critique_from_response(raw, Usage::default());
        assert!(critique.approved, "'\\nApproved' should pass");
        assert_eq!(critique.score, 0.8);
    }

    #[test]
    fn parse_critique_do_not_approve_fails() {
        let raw = "do not approve - tests are missing.";
        let critique = parse_critique_from_response(raw, Usage::default());
        assert!(!critique.approved, "'do not approve' should fail");
        assert_eq!(critique.score, 0.3);
    }

    #[tokio::test]
    async fn verify_veto_when_critic_approves_but_fails_allowlisted_command() {
        use crate::verify;
        use verify::{VerifyOptions, VerifyStatus, VerifyStep};

        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();

        let steps = vec![VerifyStep {
            id: "test1".into(),
            command: "cargo".into(),
            args: vec!["test".into(), "--nonexistent-flag-xyz123".into()],
            expected: None,
        }];

        let opts = VerifyOptions {
            allowed_executables: vec!["cargo".into()],
            continue_on_error: false,
            timeout_ms: 5000,
        };

        let results = verify::run_verification_steps(&steps, &opts, repo).await;

        assert!(
            !verify::all_passed(&results),
            "failing cargo test should not pass"
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, VerifyStatus::Failed);
    }

    // --- Loop spine tests (critique -> replan closure) ---

    use async_trait::async_trait;

    /// A scripted LLM that routes by system-prompt content so the outer loop
    /// can be driven without a real provider. The critic flips to `approved`
    /// after `reject_first` rejections.
    struct ScriptedLlm {
        critique_calls: AtomicUsize,
        reject_first: usize,
    }

    impl ScriptedLlm {
        fn approve_after(reject_first: usize) -> Arc<Self> {
            Arc::new(Self {
                critique_calls: AtomicUsize::new(0),
                reject_first,
            })
        }
    }

    #[async_trait]
    impl LlmClient for ScriptedLlm {
        async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
            let sys = req
                .messages
                .first()
                .map(|m| m.content.as_str())
                .unwrap_or("");
            let content = if sys.contains("reviewing a change") {
                let n = self.critique_calls.fetch_add(1, Ordering::SeqCst);
                let approved = n >= self.reject_first;
                if approved {
                    r#"{"approved":true,"score":0.9,"issues":[],"suggestions":[]}"#.to_string()
                } else {
                    r#"{"approved":false,"score":0.2,"issues":["tests missing"],"suggestions":["add tests"]}"#
                        .to_string()
                }
            } else if sys.contains("decomposing work into concrete subtasks") {
                r#"{"subtasks":[{"id":"s1","description":"implement feature","tier":"mid","kind":"implement","files":[],"acceptance_criteria":["it works"]}],"risks":[]}"#
                    .to_string()
            } else if sys.contains("executing a specific subtask") {
                "done".to_string()
            } else if sys.contains("understand codebases thoroughly") {
                "Understood the goal.".to_string()
            } else {
                "{}".to_string()
            };

            Ok(CompletionResponse {
                content,
                tool_calls: Vec::new(),
                usage: Usage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
                model: "scripted".into(),
                finish_reason: crate::llm::FinishReason::Stop,
            })
        }

        fn default_model(&self) -> &str {
            "scripted"
        }
    }

    fn loop_test_agent(llm: Arc<dyn LlmClient>) -> Agent {
        let config = AgentConfig {
            tdd: false, // keep execution single-phase for the loop test
            ..Default::default()
        };
        Agent::builder()
            .llm(llm)
            .tools(ToolRegistry::new())
            .config(config)
            .build()
            .expect("agent builds")
    }

    #[tokio::test]
    async fn run_loop_converges_on_second_critique() {
        // Critic rejects once, then approves -> converge in 2 iterations.
        let llm = ScriptedLlm::approve_after(1);
        let agent = loop_test_agent(llm);
        let result = agent
            .run_loop(
                &crate::goal::GoalSpec::new("ship the feature"),
                &LoopConfig::default(),
                None,
            )
            .await
            .expect("loop runs");

        assert!(result.converged);
        assert_eq!(result.termination, LoopTermination::Approved);
        assert_eq!(result.iteration_count(), 2);
        // Iteration 1 rejected, iteration 2 approved.
        assert!(!result.iterations[0].critique_approved);
        assert!(result.iterations[1].critique_approved);
        // The feedback edge fired: iteration 2 was a re-plan.
        assert!(result.iterations[1].replanned);
        assert!(!result.iterations[0].replanned);
    }

    #[tokio::test]
    async fn run_loop_exhausts_budget_without_convergence() {
        // Critic never approves, and oscillation detection is off so we
        // actually exhaust the iteration budget.
        let llm = ScriptedLlm::approve_after(usize::MAX);
        let agent = loop_test_agent(llm);
        let cfg = LoopConfig {
            max_iterations: 2,
            detect_oscillation: false,
            ..Default::default()
        };
        let result = agent
            .run_loop(&crate::goal::GoalSpec::new("ship the feature"), &cfg, None)
            .await
            .expect("loop runs");

        assert!(!result.converged);
        assert_eq!(result.termination, LoopTermination::MaxIterations);
        assert_eq!(result.iteration_count(), 2);
        // Last iteration's critique issues carried forward.
        assert!(!result.iterations[1].critique_issues.is_empty());
    }

    #[tokio::test]
    async fn run_loop_no_replan_terminates_after_first_rejection() {
        let llm = ScriptedLlm::approve_after(usize::MAX);
        let agent = loop_test_agent(llm);
        let cfg = LoopConfig {
            max_iterations: 5,
            replan_on_failure: false,
            ..Default::default()
        };
        let result = agent
            .run_loop(&crate::goal::GoalSpec::new("ship the feature"), &cfg, None)
            .await
            .expect("loop runs");

        assert!(!result.converged);
        assert_eq!(result.termination, LoopTermination::NoReplan);
        assert_eq!(result.iteration_count(), 1);
    }

    #[test]
    fn loop_config_defaults_are_sane() {
        let c = LoopConfig::default();
        assert!(c.stop_on_approval);
        assert!(c.replan_on_failure);
        assert!(c.max_iterations >= 1);
        assert!(c.detect_oscillation);
        assert!(c.spend_cap_usd.is_none());
    }

    #[test]
    fn loop_result_iteration_count_counts_records() {
        let result = LoopResult {
            goal: "g".into(),
            iterations: vec![
                LoopIteration {
                    iteration: 1,
                    replanned: false,
                    plan_goal: "g".into(),
                    subtask_count: 1,
                    succeeded: 1,
                    failed: 0,
                    critique_approved: false,
                    critique_score: 0.2,
                    critique_issues: vec!["x".into()],
                    verify_failed: Vec::new(),
                    injected_learning_ids: Vec::new(),
                    usage: Usage::default(),
                    plan_parse_error: None,
                    incorporation_gap: None,
                },
                LoopIteration {
                    iteration: 2,
                    replanned: true,
                    plan_goal: "g".into(),
                    subtask_count: 1,
                    succeeded: 1,
                    failed: 0,
                    critique_approved: true,
                    critique_score: 0.9,
                    critique_issues: vec![],
                    verify_failed: Vec::new(),
                    injected_learning_ids: Vec::new(),
                    usage: Usage::default(),
                    plan_parse_error: None,
                    incorporation_gap: None,
                },
            ],
            converged: true,
            termination: LoopTermination::Approved,
            total_usage: Usage::default(),
            grader_source: "test".to_string(),
            final_result: AgentRunResult {
                goal: "g".into(),
                comprehension: Comprehension {
                    goal: "g".into(),
                    summary: String::new(),
                    cited_elements: Vec::new(),
                    key_findings: Vec::new(),
                    risks: Vec::new(),
                    usage: Usage::default(),
                    retrieved_learning_ids: Vec::new(),
                    complexity: TaskComplexity::default(),
                },
                plan: Plan {
                    goal: "g".into(),
                    goal_statement: "g".into(),
                    criteria: Vec::new(),
                    subtasks: Vec::new(),
                    tdd: false,
                    risks: Vec::new(),
                    schema_version: String::new(),
                    complexity: TaskComplexity::default(),
                },
                step_results: Vec::new(),
                critique: None,
                decision: None,
                runbook: None,
                total_usage: Usage::default(),
            },
        };
        assert_eq!(result.iteration_count(), 2);
    }

    // --- ACT phase test: the loop mutates files via tools ---

    /// A scripted LLM that issues a `file_write` tool call the first time it
    /// is asked to execute, then terminates the tool loop with plain text.
    /// Critic approves immediately so the loop converges in one iteration.
    struct ActingLlm {
        execute_calls: AtomicUsize,
    }

    #[async_trait]
    impl LlmClient for ActingLlm {
        async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
            let sys = req
                .messages
                .first()
                .map(|m| m.content.as_str())
                .unwrap_or("");
            let usage = Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            };
            let content = if sys.contains("reviewing a change") {
                r#"{"approved":true,"score":0.9,"issues":[],"suggestions":[]}"#.to_string()
            } else if sys.contains("executing a specific subtask") {
                let n = self.execute_calls.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    return Ok(CompletionResponse {
                        content: "Writing the file.".into(),
                        tool_calls: vec![crate::llm::ToolCall {
                            id: "call_1".into(),
                            name: "file_write".into(),
                            arguments: serde_json::json!({
                                "path": "src/hello.rs",
                                "content": "fn main() { println!(\"hello from the loop\"); }\n"
                            }),
                        }],
                        usage: usage.clone(),
                        model: "acting".into(),
                        finish_reason: crate::llm::FinishReason::ToolCalls,
                    });
                }
                "done".to_string()
            } else if sys.contains("decomposing work into concrete subtasks") {
                r#"{"subtasks":[{"id":"s1","description":"write hello module","tier":"mid","kind":"implement","files":["src/hello.rs"],"acceptance_criteria":["file exists"]}],"risks":[]}"#
                    .to_string()
            } else if sys.contains("understand codebases thoroughly") {
                "Understood.".to_string()
            } else {
                "{}".to_string()
            };

            Ok(CompletionResponse {
                content,
                tool_calls: Vec::new(),
                usage,
                model: "acting".into(),
                finish_reason: crate::llm::FinishReason::Stop,
            })
        }

        fn default_model(&self) -> &str {
            "acting"
        }
    }

    #[tokio::test]
    async fn run_loop_actually_mutates_files_via_tools() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();

        let llm = Arc::new(ActingLlm {
            execute_calls: AtomicUsize::new(0),
        });
        let tools = ToolRegistry::with_builtin(root.clone(), Vec::new());
        let config = AgentConfig {
            tdd: false,
            review_every_change: true,
            ..Default::default()
        };
        let agent = Agent::builder()
            .llm(llm)
            .tools(tools)
            .config(config)
            .build()
            .expect("agent builds");

        let result = agent
            .run_loop(
                &crate::goal::GoalSpec::new("create hello module"),
                &LoopConfig::default(),
                None,
            )
            .await
            .expect("loop runs");

        // The loop converged on iteration 1.
        assert!(result.converged, "expected convergence");
        assert_eq!(result.iteration_count(), 1);

        // The ACT phase mutated the filesystem — the file exists on disk.
        let written = std::fs::read_to_string(root.join("src/hello.rs")).expect("file was written");
        assert!(written.contains("hello from the loop"));
    }

    #[tokio::test]
    async fn run_loop_convergence_vetoed_by_deterministic_verifier() {
        // Critic approves immediately, but a deterministic verify step fails.
        // The independent grader must veto convergence regardless of the LLM.
        let llm = ScriptedLlm::approve_after(0);
        let agent = loop_test_agent(llm);
        let failing_step = VerifyStep {
            id: "must_fail".into(),
            command: "git".into(),
            args: vec!["not-a-real-git-subcommand".into()],
            expected: None,
        };
        let cfg = LoopConfig {
            // Oscillation detection off so a repeated verify-failure signature
            // doesn't terminate the loop before we observe the veto.
            detect_oscillation: false,
            verifier: Some(VerifierConfig {
                steps: vec![failing_step],
                options: VerifyOptions::default(),
                workdir: std::env::temp_dir(),
            }),
            ..Default::default()
        };
        let result = agent
            .run_loop(&crate::goal::GoalSpec::new("ship the feature"), &cfg, None)
            .await
            .expect("loop runs");

        assert!(!result.converged, "verifier failure must veto convergence");
        assert_ne!(
            result.termination,
            LoopTermination::Approved,
            "must not terminate as Approved when verify fails"
        );
        // Every iteration ran the grader and recorded the failure.
        assert!(result
            .iterations
            .iter()
            .all(|i| !i.verify_failed.is_empty()));
    }

    // --- Guardrail tests ---

    #[tokio::test]
    async fn run_loop_terminates_on_spend_cap() {
        // ScriptedLlm returns small but non-zero usage per call.
        // After one iteration (execute + critique), the estimated cost
        // exceeds a tiny cap → SpendCapExceeded.
        let llm = ScriptedLlm::approve_after(usize::MAX);
        let agent = loop_test_agent(llm);
        let cfg = LoopConfig {
            max_iterations: 5,
            spend_cap_usd: Some(0.00001),
            detect_oscillation: false,
            ..Default::default()
        };
        let result = agent
            .run_loop(&crate::goal::GoalSpec::new("ship the feature"), &cfg, None)
            .await
            .expect("loop runs");

        assert!(!result.converged);
        assert!(
            matches!(result.termination, LoopTermination::SpendCapExceeded(cost) if cost > 0.0),
            "expected SpendCapExceeded, got {:?}",
            result.termination
        );
        // Should have terminated before exhausting all 5 iterations.
        assert!(result.iteration_count() < 5);
    }

    #[tokio::test]
    async fn run_loop_detects_oscillation() {
        // ScriptedLlm always rejects with the same issues ("tests missing").
        // After iteration 2, the same critique signature repeats → Oscillation.
        let llm = ScriptedLlm::approve_after(usize::MAX);
        let agent = loop_test_agent(llm);
        let cfg = LoopConfig {
            max_iterations: 5,
            detect_oscillation: true,
            ..Default::default()
        };
        let result = agent
            .run_loop(&crate::goal::GoalSpec::new("ship the feature"), &cfg, None)
            .await
            .expect("loop runs");

        assert!(!result.converged);
        assert_eq!(result.termination, LoopTermination::Oscillation);
        // Oscillation is detected at iteration 2 (the first repeat).
        assert_eq!(result.iteration_count(), 2);
        // Both iterations had the same critique issues.
        assert_eq!(
            result.iterations[0].critique_issues,
            result.iterations[1].critique_issues
        );
    }

    // --- Ensemble critic tests (U1) ---

    #[tokio::test]
    async fn ensemble_one_persona_blocks_union_issues_and_min_score() {
        // One persona blocks → ensemble ANDs = false, score = min,
        // issues union, tagged. Four personas approve.
        let llm = PersonaScriptedLlm::new(vec![
            PersonaResponse {
                system_prompt_contains: "correctness",
                approved: false,
                score: 0.2,
                issues: vec!["buffer overflow on empty input".into()],
            },
            PersonaResponse {
                system_prompt_contains: "acceptance criteria",
                approved: true,
                score: 0.8,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "boundary",
                approved: true,
                score: 0.9,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "regression",
                approved: true,
                score: 0.85,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "adversarial",
                approved: true,
                score: 0.95,
                issues: vec![],
            },
        ]);

        let mut config = AgentConfig::default();
        config.critique_personas = CritiquePersona::default_personas();
        let agent = Agent::builder()
            .llm(llm)
            .tools(ToolRegistry::new())
            .config(config)
            .build()
            .expect("agent builds");
        let result = agent
            .critique(
                &Plan {
                    goal: "fix bug".into(),
                    goal_statement: "fix bug".into(),
                    criteria: Vec::new(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                    schema_version: String::new(),
                    complexity: TaskComplexity::default(),
                },
                &vec![],
            )
            .await
            .expect("critique runs");

        assert!(!result.approved);
        // All personas' issues unioned. The correctness persona's issue is present.
        assert!(result
            .issues
            .contains(&"buffer overflow on empty input".into()));
        // Score is min across personas (blocking 0.2 wins).
        assert!((result.score - 0.2).abs() < f64::EPSILON);
        // persona_breakdown has all five personas.
        assert_eq!(result.persona_breakdown.len(), 5);
        let correctness_result = result
            .persona_breakdown
            .iter()
            .find(|p| p.id == "correctness")
            .expect("correctness persona recorded");
        assert!(!correctness_result.approved);
        assert_eq!(correctness_result.score, 0.2);
        assert!(correctness_result.issues == vec!["buffer overflow on empty input"]);
    }

    #[test]
    fn ensemble_empty_personas_fallback_to_single_critic() {
        // Empty personas → single legacy call with CRITIQUE_SYSTEM_PROMPT,
        // additive fields empty.
        let mut config = AgentConfig::default();
        config.critique_personas.clear();

        let agent = Agent::builder()
            .llm(ScriptedLlm::approve_after(0))
            .tools(ToolRegistry::new())
            .config(config)
            .build()
            .expect("agent builds");

        // Blocking against the empty-personas fallback requires a sync environment.
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let result = agent
                .critique(
                    &Plan {
                        goal: "test".into(),
                        goal_statement: "test".into(),
                        criteria: Vec::new(),
                        subtasks: vec![],
                        tdd: false,
                        risks: vec![],
                        schema_version: String::new(),
                        complexity: TaskComplexity::default(),
                    },
                    &vec![],
                )
                .await
                .expect("critique runs");

            // Approved, no persona_breakdown, no injected_learning_ids.
            assert!(result.approved);
            assert_eq!(result.persona_breakdown, vec![]);
            assert_eq!(result.injected_learning_ids, Vec::<String>::new());
        });
    }

    #[tokio::test]
    async fn ensemble_union_dedup_issues() {
        // Multiple personas report the same semantic issue → union with
        // dedup. Sorted order for determinism.
        let llm = PersonaScriptedLlm::new(vec![
            PersonaResponse {
                system_prompt_contains: "correctness",
                approved: true,
                score: 0.9,
                issues: vec!["tests missing".into()],
            },
            PersonaResponse {
                system_prompt_contains: "acceptance criteria",
                approved: true,
                score: 0.8,
                issues: vec!["tests missing".into()],
            },
            PersonaResponse {
                system_prompt_contains: "boundary",
                approved: true,
                score: 0.9,
                issues: vec!["tests missing".into()],
            },
            PersonaResponse {
                system_prompt_contains: "regression",
                approved: true,
                score: 0.85,
                issues: vec!["tests missing".into()],
            },
            PersonaResponse {
                system_prompt_contains: "adversarial",
                approved: true,
                score: 0.95,
                issues: vec!["tests missing".into()],
            },
        ]);

        let mut config = AgentConfig::default();
        config.critique_personas = CritiquePersona::default_personas();
        let agent = Agent::builder()
            .llm(llm)
            .tools(ToolRegistry::new())
            .config(config)
            .build()
            .expect("agent builds");
        let result = agent
            .critique(
                &Plan {
                    goal: "test".into(),
                    goal_statement: "test".into(),
                    criteria: Vec::new(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                    schema_version: String::new(),
                    complexity: TaskComplexity::default(),
                },
                &vec![],
            )
            .await
            .expect("critique runs");

        // Approved (all personas approve), issues deduped to one.
        assert!(result.approved);
        // Five personas each reported "tests missing" → union has one entry.
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0], "tests missing");
    }

    #[tokio::test]
    async fn ensemble_parallel_dispatch_is_concurrent() {
        // Parallel dispatch: the ensemble uses tokio::JoinSet and all
        // personas run concurrently. Deterministic check: record active
        // concurrency high-water mark.
        let llm = PersonaScriptedLlm::new(vec![
            PersonaResponse {
                system_prompt_contains: "correctness",
                approved: true,
                score: 0.9,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "acceptance criteria",
                approved: true,
                score: 0.8,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "boundary",
                approved: true,
                score: 0.9,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "regression",
                approved: true,
                score: 0.85,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "adversarial",
                approved: true,
                score: 0.95,
                issues: vec![],
            },
        ]);

        let mut config = AgentConfig::default();
        config.critique_personas = CritiquePersona::default_personas();
        let agent = Agent::builder()
            .llm(llm.clone())
            .tools(ToolRegistry::new())
            .config(config)
            .build()
            .expect("agent builds");
        let result = agent
            .critique(
                &Plan {
                    goal: "concurrency check".into(),
                    goal_statement: "concurrency check".into(),
                    criteria: Vec::new(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                    schema_version: String::new(),
                    complexity: TaskComplexity::default(),
                },
                &vec![],
            )
            .await
            .expect("critique runs");

        assert!(result.approved);
        // High-water mark >= 2 (both personas ran simultaneously at some point).
        assert!(llm.max_concurrent.load(Ordering::SeqCst) >= 2);
    }

    #[tokio::test]
    async fn ensemble_all_personas_approve() {
        // All personas approve → merged approved == true, issues empty, score == 1.0.
        let llm = PersonaScriptedLlm::new(vec![
            PersonaResponse {
                system_prompt_contains: "correctness",
                approved: true,
                score: 1.0,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "acceptance criteria",
                approved: true,
                score: 0.95,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "boundary",
                approved: true,
                score: 0.9,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "regression",
                approved: true,
                score: 0.85,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "adversarial",
                approved: true,
                score: 0.95,
                issues: vec![],
            },
        ]);

        let mut config = AgentConfig::default();
        config.critique_personas = CritiquePersona::default_personas();
        let agent = Agent::builder()
            .llm(llm)
            .tools(ToolRegistry::new())
            .config(config)
            .build()
            .expect("agent builds");
        let result = agent
            .critique(
                &Plan {
                    goal: "all good".into(),
                    goal_statement: "all good".into(),
                    criteria: Vec::new(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                    schema_version: String::new(),
                    complexity: TaskComplexity::default(),
                },
                &vec![],
            )
            .await
            .expect("critique runs");

        assert!(result.approved);
        assert!(result.issues.is_empty());
        // Score is min across personas (0.85 wins).
        assert!((result.score - 0.85).abs() < f64::EPSILON);
        assert_eq!(result.persona_breakdown.len(), 5);
    }

    #[tokio::test]
    async fn ensemble_score_is_min_not_mean() {
        // Four personas score 1.0, one scores 0.2 → merged score == 0.2 (not mean).
        let llm = PersonaScriptedLlm::new(vec![
            PersonaResponse {
                system_prompt_contains: "correctness",
                approved: true,
                score: 1.0,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "acceptance criteria",
                approved: true,
                score: 1.0,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "boundary",
                approved: true,
                score: 1.0,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "regression",
                approved: true,
                score: 0.2,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "adversarial",
                approved: true,
                score: 1.0,
                issues: vec![],
            },
        ]);

        let mut config = AgentConfig::default();
        config.critique_personas = CritiquePersona::default_personas();
        let agent = Agent::builder()
            .llm(llm)
            .tools(ToolRegistry::new())
            .config(config)
            .build()
            .expect("agent builds");
        let result = agent
            .critique(
                &Plan {
                    goal: "score check".into(),
                    goal_statement: "score check".into(),
                    criteria: Vec::new(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                    schema_version: String::new(),
                    complexity: TaskComplexity::default(),
                },
                &vec![],
            )
            .await
            .expect("critique runs");

        assert!(result.approved);
        // Min score wins (0.2), not mean (0.8).
        assert!((result.score - 0.2).abs() < f64::EPSILON);
    }

    // --- Memory-in-critique tests (U4) ---

    #[tokio::test]
    async fn critique_injects_guardrail_blind_spots_and_bumps_retrieval_count() {
        // Guardrail learning in memory → appears in critic prompt under
        // "Known blind spots". Playbooks are excluded. Retrieval counters bump.
        let guardrail = LearningEntry::guardrail(
            "boundary crossing added",
            "change crosses forbidden dependency",
            "This change crosses a forbidden dependency boundary. Consider alternative approach.",
        );
        let playbook = LearningEntry::new(
            "pattern works",
            "regex pattern extraction succeeded",
            "Pattern extraction approach is validated.",
        );

        let mut config = AgentConfig::default();
        config.critique_personas = CritiquePersona::default_personas();
        let tempdir = tempfile::tempdir().expect("tempdir");

        let llm = PersonaScriptedLlm::new(vec![]);
        let concrete_mem = std::sync::Mutex::new(AgenticMemory::default());
        let mem_arc: std::sync::Arc<dyn crate::memory::Memory + Send + Sync> =
            std::sync::Arc::new(concrete_mem);
        let agent = Agent::builder()
            .llm(llm)
            .tools(ToolRegistry::new())
            .config(config)
            .memory_backend(tempdir.path(), mem_arc.clone())
            .build()
            .expect("agent builds");

        // Record learnings via the trait for test setup.
        mem_arc.record(guardrail.clone()).expect("record guardrail");
        mem_arc.record(playbook.clone()).expect("record playbook");
        mem_arc.save_to_path(tempdir.path()).expect("save memory");

        let result = agent
            .critique(
                &Plan {
                    goal: "boundary crossing".into(),
                    goal_statement: "boundary crossing".into(),
                    criteria: Vec::new(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                    schema_version: String::new(),
                    complexity: TaskComplexity::default(),
                },
                &vec![],
            )
            .await
            .expect("critique runs");

        // The guardrail learning ID is in injected_learning_ids; the playbook is not.
        assert!(result.injected_learning_ids.contains(&guardrail.id));
        assert!(!result.injected_learning_ids.contains(&playbook.id));

        // Retrieval count on the guardrail was bumped (from 0 to 1).
        // Verify via count() that the memory backend has entries.
        assert!(
            mem_arc.count() >= 2,
            "memory should have at least 2 entries"
        );
    }

    #[tokio::test]
    async fn critique_no_memory_shows_no_blind_spots_section() {
        // No memory → no blind-spots section in the prompt.
        let mut config = AgentConfig::default();
        config.critique_personas = CritiquePersona::default_personas();

        let llm = PersonaScriptedLlm::new(vec![
            PersonaResponse {
                system_prompt_contains: "correctness",
                approved: true,
                score: 0.9,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "acceptance criteria",
                approved: true,
                score: 0.9,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "boundary",
                approved: true,
                score: 0.9,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "regression",
                approved: true,
                score: 0.9,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "adversarial",
                approved: true,
                score: 0.9,
                issues: vec![],
            },
        ]);
        let agent = Agent::builder()
            .llm(llm.clone())
            .tools(ToolRegistry::new())
            .config(config)
            .build()
            .expect("agent builds");

        let _ = agent
            .critique(
                &Plan {
                    goal: "no memory".into(),
                    goal_statement: "no memory".into(),
                    criteria: Vec::new(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                    schema_version: String::new(),
                    complexity: TaskComplexity::default(),
                },
                &vec![],
            )
            .await
            .expect("critique runs");

        // Prompt received by the mock contains the shared context but no
        // "Known blind spots" section.
        let all_prompts = format!(
            "{}\n{}",
            llm.received_system_prompt(),
            llm.received_user_prompt()
        );
        assert!(!all_prompts.contains("Known blind spots"));
        assert!(all_prompts.contains("What the actor claims it did"));
    }

    #[tokio::test]
    async fn critique_playbooks_excluded_from_blind_spots() {
        // Only guardrail learnings appear in the blind-spots section; playbooks
        // are excluded from injected_learning_ids.
        let guardrail = LearningEntry::guardrail(
            "memory leak on disconnect",
            "connection not closed",
            "Always close connections in a finally block.",
        );
        let playbook = LearningEntry::new(
            "successful pattern",
            "caching worked well",
            "Use Redis for caching.",
        );

        let mut config = AgentConfig::default();
        config.critique_personas = CritiquePersona::default_personas();
        let tempdir = tempfile::tempdir().expect("tempdir");

        let llm = PersonaScriptedLlm::new(vec![PersonaResponse {
            system_prompt_contains: "correctness",
            approved: true,
            score: 0.9,
            issues: vec![],
        }]);
        let mem_arc: std::sync::Arc<dyn crate::memory::Memory + Send + Sync> =
            std::sync::Arc::new(std::sync::Mutex::new(AgenticMemory::default()));
        let mem_for_test = mem_arc.clone();
        let agent = Agent::builder()
            .llm(llm.clone())
            .tools(ToolRegistry::new())
            .config(config)
            .memory_backend(tempdir.path(), mem_arc)
            .build()
            .expect("agent builds");

        mem_for_test
            .record(guardrail.clone())
            .expect("record guardrail");
        mem_for_test
            .record(playbook.clone())
            .expect("record playbook");
        mem_for_test
            .save_to_path(tempdir.path())
            .expect("save memory");

        let _ = agent
            .critique(
                &Plan {
                    goal: "connection not closed".into(),
                    goal_statement: "connection not closed".into(),
                    criteria: Vec::new(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                    schema_version: String::new(),
                    complexity: TaskComplexity::default(),
                },
                &vec![],
            )
            .await
            .expect("critique runs");

        // The guardrail is injected; the playbook is not.
        let user_prompt = llm.received_user_prompt();
        assert!(
            user_prompt.contains("Always close connections"),
            "guardrail advice should appear in prompt"
        );
        assert!(
            !user_prompt.contains("Redis for caching"),
            "playbook should not appear in blind-spots prompt"
        );
    }

    #[tokio::test]
    async fn critique_roundtrips_with_ensemble() {
        // With ensemble active, each persona's prompt contains the blind-spots
        // section (not just one persona).
        let guardrail = LearningEntry::guardrail(
            "unchecked unwrap",
            "potential panic",
            "Always handle Result types properly.",
        );

        let mut config = AgentConfig::default();
        config.critique_personas = CritiquePersona::default_personas();
        let tempdir = tempfile::tempdir().expect("tempdir");

        let llm = PersonaScriptedLlm::new(vec![
            PersonaResponse {
                system_prompt_contains: "correctness",
                approved: true,
                score: 0.9,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "acceptance criteria",
                approved: true,
                score: 0.8,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "boundary",
                approved: true,
                score: 0.9,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "regression",
                approved: true,
                score: 0.85,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "adversarial",
                approved: true,
                score: 0.95,
                issues: vec![],
            },
        ]);
        let mem_arc: std::sync::Arc<dyn crate::memory::Memory + Send + Sync> =
            std::sync::Arc::new(std::sync::Mutex::new(AgenticMemory::default()));
        let mem_for_test = mem_arc.clone();
        let agent = Agent::builder()
            .llm(llm.clone())
            .tools(ToolRegistry::new())
            .config(config)
            .memory_backend(tempdir.path(), mem_arc)
            .build()
            .expect("agent builds");

        mem_for_test
            .record(guardrail.clone())
            .expect("record guardrail");
        mem_for_test
            .save_to_path(tempdir.path())
            .expect("save memory");

        let result = agent
            .critique(
                &Plan {
                    goal: "potential panic".into(),
                    goal_statement: "potential panic".into(),
                    criteria: Vec::new(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                    schema_version: String::new(),
                    complexity: TaskComplexity::default(),
                },
                &vec![],
            )
            .await
            .expect("critique runs");

        assert!(result.approved);
        // The guardrail was injected.
        assert!(result.injected_learning_ids.contains(&guardrail.id));
        // All five personas were invoked (each sees the blind-spots section
        // because it's appended to shared_user which all personas receive).
        assert_eq!(result.persona_breakdown.len(), 5);
    }

    #[test]
    fn usage_estimated_cost_is_nonzero() {
        let usage = Usage {
            prompt_tokens: 1_000_000,
            completion_tokens: 500_000,
            total_tokens: 1_500_000,
        };
        // gpt-4o-mini: 1M * $0.15/1M + 0.5M * $0.60/1M = $0.15 + $0.30 = $0.45
        let cost = usage.estimated_cost_usd();
        assert!(
            (cost - 0.45).abs() < 0.001,
            "expected ~$0.45, got ${cost:.4}"
        );
    }

    #[test]
    fn critique_signature_normalises_order() {
        let a = critique_signature(&["x".into(), "y".into()]);
        let b = critique_signature(&["y".into(), "x".into()]);
        assert_eq!(a, b);
    }

    #[test]
    fn check_incorporation_returns_none_when_no_issues() {
        let plan = Plan {
            goal: "g".into(),
            goal_statement: "g".into(),
            criteria: Vec::new(),
            subtasks: vec![Subtask {
                id: "s1".into(),
                description: "d".into(),
                tier: TaskTier::Cheap,
                kind: SubtaskKind::Implement,
                files: Vec::new(),
                acceptance_criteria: Vec::new(),
            }],
            tdd: false,
            risks: vec![],
            schema_version: String::new(),
            complexity: TaskComplexity::default(),
        };
        // No issues → no incorporation needed.
        assert!(check_incorporation(&plan, &plan, &[]).is_none());
    }

    #[test]
    fn check_incorporation_detects_identical_plan() {
        let plan = Plan {
            goal: "g".into(),
            goal_statement: "g".into(),
            criteria: Vec::new(),
            subtasks: vec![Subtask {
                id: "s1".into(),
                description: "d".into(),
                tier: TaskTier::Cheap,
                kind: SubtaskKind::Implement,
                files: Vec::new(),
                acceptance_criteria: Vec::new(),
            }],
            tdd: false,
            risks: vec!["r1".into()],
            schema_version: String::new(),
            complexity: TaskComplexity::default(),
        };
        let issues = vec!["fix the bug".into()];
        let gap = check_incorporation(&plan, &plan, &issues);
        assert!(
            gap.is_some(),
            "identical plans with issues should produce a gap"
        );
        assert!(gap.unwrap().contains("structurally identical"));
    }

    #[test]
    fn check_incorporation_returns_none_when_plan_changed() {
        let old = Plan {
            goal: "g".into(),
            goal_statement: "g".into(),
            criteria: Vec::new(),
            subtasks: vec![Subtask {
                id: "s1".into(),
                description: "old".into(),
                tier: TaskTier::Cheap,
                kind: SubtaskKind::Implement,
                files: Vec::new(),
                acceptance_criteria: Vec::new(),
            }],
            tdd: false,
            risks: vec![],
            schema_version: String::new(),
            complexity: TaskComplexity::default(),
        };
        let new = Plan {
            goal: "g".into(),
            goal_statement: "g".into(),
            criteria: Vec::new(),
            subtasks: vec![Subtask {
                id: "s1".into(),
                description: "new".into(),
                tier: TaskTier::Cheap,
                kind: SubtaskKind::Implement,
                files: Vec::new(),
                acceptance_criteria: Vec::new(),
            }],
            tdd: false,
            risks: vec![],
            schema_version: String::new(),
            complexity: TaskComplexity::default(),
        };
        let issues = vec!["fix the bug".into()];
        assert!(check_incorporation(&old, &new, &issues).is_none());
    }

    #[test]
    fn check_incorporation_returns_none_when_risks_changed() {
        let old = Plan {
            goal: "g".into(),
            goal_statement: "g".into(),
            criteria: Vec::new(),
            subtasks: vec![Subtask {
                id: "s1".into(),
                description: "d".into(),
                tier: TaskTier::Cheap,
                kind: SubtaskKind::Implement,
                files: Vec::new(),
                acceptance_criteria: Vec::new(),
            }],
            tdd: false,
            risks: vec![],
            schema_version: String::new(),
            complexity: TaskComplexity::default(),
        };
        let new = Plan {
            goal: "g".into(),
            goal_statement: "g".into(),
            criteria: Vec::new(),
            subtasks: vec![Subtask {
                id: "s1".into(),
                description: "d".into(),
                tier: TaskTier::Cheap,
                kind: SubtaskKind::Implement,
                files: Vec::new(),
                acceptance_criteria: Vec::new(),
            }],
            tdd: false,
            risks: vec!["new risk".into()],
            schema_version: String::new(),
            complexity: TaskComplexity::default(),
        };
        let issues = vec!["fix the bug".into()];
        // Same subtasks but risks changed → no gap.
        assert!(check_incorporation(&old, &new, &issues).is_none());
    }

    // --- Checkpoint tests ---

    fn test_comprehension() -> Comprehension {
        Comprehension {
            goal: String::new(),
            summary: String::new(),
            cited_elements: Vec::new(),
            key_findings: Vec::new(),
            risks: Vec::new(),
            usage: Usage::default(),
            retrieved_learning_ids: Vec::new(),
            complexity: TaskComplexity::default(),
        }
    }

    #[test]
    fn checkpoint_write_and_load() {
        let tmp = tempfile::tempdir().unwrap();
        let checkpoint = RunCheckpoint {
            goal: "test goal".into(),
            comprehension: test_comprehension(),
            iterations: Vec::new(),
            last_plan: None,
            last_steps: Vec::new(),
            last_critique: None,
            failure_tracker: FailureTracker::default(),
            total_usage: Usage::default(),
            converged: false,
            termination: LoopTermination::MaxIterations,
            seen_signatures: Vec::new(),
            timestamp: "2026-01-01T00:00:00Z".into(),
        };

        // Write checkpoint.
        checkpoint.write(tmp.path()).unwrap();
        assert!(RunCheckpoint::exists(tmp.path()));

        // Load checkpoint.
        let loaded = RunCheckpoint::load(tmp.path()).unwrap();
        assert_eq!(loaded.goal, "test goal");
        assert!(!loaded.converged);
        assert_eq!(loaded.termination, LoopTermination::MaxIterations);
    }

    #[test]
    fn checkpoint_cleanup() {
        let tmp = tempfile::tempdir().unwrap();
        let checkpoint = RunCheckpoint {
            goal: "test".into(),
            comprehension: test_comprehension(),
            iterations: Vec::new(),
            last_plan: None,
            last_steps: Vec::new(),
            last_critique: None,
            failure_tracker: FailureTracker::default(),
            total_usage: Usage::default(),
            converged: true,
            termination: LoopTermination::Approved,
            seen_signatures: Vec::new(),
            timestamp: "2026-01-01T00:00:00Z".into(),
        };

        checkpoint.write(tmp.path()).unwrap();
        assert!(RunCheckpoint::exists(tmp.path()));

        RunCheckpoint::cleanup(tmp.path()).unwrap();
        assert!(!RunCheckpoint::exists(tmp.path()));
    }

    #[test]
    fn failure_tracker_records_and_formats() {
        let mut tracker = FailureTracker::default();
        tracker.record(
            "subtasks: [s1(Implement)]".into(),
            "critic rejected: missing tests".into(),
            1,
            ErrorClass::Other,
        );
        assert_eq!(tracker.failures.len(), 1);
        assert_eq!(tracker.consecutive_same_approach, 1);

        // Same approach again → consecutive count increases.
        tracker.record(
            "subtasks: [s1(Implement)]".into(),
            "critic rejected: missing tests".into(),
            2,
            ErrorClass::Other,
        );
        assert_eq!(tracker.consecutive_same_approach, 2);

        // Different approach → resets consecutive count.
        tracker.record(
            "subtasks: [s1(TestAuthor), s2(Implement)]".into(),
            "critic rejected: different issue".into(),
            3,
            ErrorClass::Other,
        );
        assert_eq!(tracker.consecutive_same_approach, 1);

        // Format for prompt includes failure history.
        let formatted = tracker.format_for_prompt();
        assert!(formatted.contains("Previously Failed Approaches"));
        assert!(formatted.contains("Iteration 1"));
        assert!(formatted.contains("Iteration 2"));
        assert!(formatted.contains("Iteration 3"));
    }

    #[test]
    fn checkpoint_serializes_failure_tracker() {
        let tmp = tempfile::tempdir().unwrap();
        let mut tracker = FailureTracker::default();
        tracker.record("approach A".into(), "reason 1".into(), 1, ErrorClass::Other);
        tracker.record("approach B".into(), "reason 2".into(), 2, ErrorClass::Other);

        let checkpoint = RunCheckpoint {
            goal: "serialize test".into(),
            comprehension: test_comprehension(),
            iterations: Vec::new(),
            last_plan: None,
            last_steps: Vec::new(),
            last_critique: None,
            failure_tracker: tracker.clone(),
            total_usage: Usage::default(),
            converged: false,
            termination: LoopTermination::MaxIterations,
            seen_signatures: vec!["sig1".into()],
            timestamp: "2026-01-01T00:00:00Z".into(),
        };

        checkpoint.write(tmp.path()).unwrap();
        let loaded = RunCheckpoint::load(tmp.path()).unwrap();
        assert_eq!(loaded.failure_tracker.failures.len(), 2);
        assert_eq!(loaded.failure_tracker.failures[0].0, "approach A");
        assert_eq!(loaded.failure_tracker.failures[1].1, "reason 2");
        assert_eq!(loaded.seen_signatures, vec!["sig1"]);
    }

    #[test]
    fn classify_error_type_error() {
        let output = r#"error[E0308]: mismatched types
   --> src/main.rs:5:15
    |
5   |     let x: i32 = "hello";
    |               ^^^^^^ expected `i32`, found `&str`
"#;
        let step = StepResult {
            subtask_id: "s1".into(),
            status: StepStatus::Failed,
            output: output.to_string(),
            usage: Usage::default(),
            tool_signals: vec![],
        };

        assert_eq!(
            classify_error(&[], &[step]),
            ErrorClass::Type
        );
    }

    #[test]
    fn classify_error_compilation_error() {
        let output = r#"error[E0425]: cannot find value `x` in this scope
   --> src/main.rs:5:5
    |
5   |     println!("{}", x);
    |                     ^ not found in this scope
"#;
        let step = StepResult {
            subtask_id: "s1".into(),
            status: StepStatus::Failed,
            output: output.to_string(),
            usage: Usage::default(),
            tool_signals: vec![],
        };

        assert_eq!(
            classify_error(&[], &[step]),
            ErrorClass::Compilation
        );
    }

    #[test]
    fn classify_error_test_failure() {
        let output = r#"running 2 tests
test tests::test_foo ... FAILED
test tests::test_bar ... ok

failures:

---- tests::test_foo stdout ----
thread 'tests::test_foo' panicked at tests/test.rs:10:5:
assertion failed: `(left == right)`
  left: `1`,
 right: `2`

test result: FAILED. 1 passed; 1 failed; 0 ignored; finished in 0.01s
"#;
        let step = StepResult {
            subtask_id: "s1".into(),
            status: StepStatus::Failed,
            output: output.to_string(),
            usage: Usage::default(),
            tool_signals: vec![],
        };

        assert_eq!(
            classify_error(&[], &[step]),
            ErrorClass::Test
        );
    }

    #[test]
    fn classify_error_runtime_panic() {
        let output = r#"thread 'main' panicked at src/main.rs:10:15:
unwrap on None value
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
"#;
        let step = StepResult {
            subtask_id: "s1".into(),
            status: StepStatus::Failed,
            output: output.to_string(),
            usage: Usage::default(),
            tool_signals: vec![],
        };

        assert_eq!(
            classify_error(&[], &[step]),
            ErrorClass::Runtime
        );
    }

    #[test]
    fn classify_error_architecture() {
        let issues = vec!["Boundary violation: crate A imports from crate B".to_string()];
        assert_eq!(
            classify_error(&issues, &[]),
            ErrorClass::Architecture
        );
    }

    #[test]
    fn classify_error_spec_gap() {
        let issues = vec!["Acceptance criterion 2 not addressed: missing error handling".to_string()];
        assert_eq!(
            classify_error(&issues, &[]),
            ErrorClass::SpecGap
        );
    }

    #[test]
    fn classify_error_other() {
        let output = "unknown error occurred";
        let step = StepResult {
            subtask_id: "s1".into(),
            status: StepStatus::Failed,
            output: output.to_string(),
            usage: Usage::default(),
            tool_signals: vec![],
        };

        assert_eq!(
            classify_error(&[], &[step]),
            ErrorClass::Other
        );
    }

    // ---------------------------------------------------------------------------
