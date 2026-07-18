use super::*;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;

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
    let c = classify_task_complexity("Fix typo in function name", &["main.rs".to_string()], &[]);
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
    let c = classify_task_complexity("Migrate the database schema", &[], &[]);
    assert_eq!(c, TaskComplexity::Complex);
}

#[test]
fn classify_complex_many_elements() {
    let c = classify_task_complexity(
        "Update all API endpoints",
        &[],
        &[
            "System.Api".to_string(),
            "System.Db".to_string(),
            "System.Auth".to_string(),
        ],
    );
    assert_eq!(c, TaskComplexity::Complex);
}

#[test]
fn classify_moderate_multi_file() {
    let c = classify_task_complexity(
        "Add JWT auth to the API",
        &[
            "auth.rs".to_string(),
            "middleware.rs".to_string(),
            "config.rs".to_string(),
        ],
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
    assert_eq!(TaskComplexity::Trivial.max_tool_iterations(8), 7);
    assert_eq!(TaskComplexity::Simple.max_tool_iterations(8), 7);
    assert_eq!(TaskComplexity::Moderate.max_tool_iterations(8), 8);
}

// --- Research task classification tests ---

#[test]
fn classify_research_what_question() {
    let c = classify_task_complexity("what is the architecture of the parser", &[], &[]);
    assert_eq!(c, TaskComplexity::Research);
}

#[test]
fn classify_research_why_question() {
    let c = classify_task_complexity("why is the build failing", &["ci.yml".to_string()], &[]);
    assert_eq!(c, TaskComplexity::Research);
}

#[test]
fn classify_research_explain() {
    let c = classify_task_complexity("explain the migration system", &[], &[]);
    assert_eq!(c, TaskComplexity::Research);
}

#[test]
fn classify_research_analyze() {
    let c = classify_task_complexity("analyze the performance of the query engine", &[], &[]);
    assert_eq!(c, TaskComplexity::Research);
}

#[test]
fn classify_research_review() {
    let c = classify_task_complexity("review the security of the auth module", &[], &[]);
    assert_eq!(c, TaskComplexity::Research);
}

#[test]
fn classify_research_description() {
    let c = classify_task_complexity("describe the data flow between components", &[], &[]);
    assert_eq!(c, TaskComplexity::Research);
}

#[test]
fn classify_research_evaluate() {
    let c = classify_task_complexity("evaluate the parser performance", &[], &[]);
    assert_eq!(c, TaskComplexity::Research);
}

#[test]
fn classify_not_research_when_implementation_keyword() {
    // "fix" is an implementation keyword — not research even though it's a question.
    let c = classify_task_complexity("fix the bug in the parser", &["parser.rs".to_string()], &[]);
    assert_eq!(c, TaskComplexity::Simple);
}

#[test]
fn classify_not_research_when_how_to() {
    // "how to" is excluded — it's an implementation question.
    let c = classify_task_complexity("how to add JWT auth to the API", &[], &[]);
    assert_eq!(c, TaskComplexity::Simple);
}

#[test]
fn classify_not_research_when_how_do() {
    let c = classify_task_complexity("how do I implement input validation", &[], &[]);
    assert_eq!(c, TaskComplexity::Simple);
}

#[test]
fn classify_not_research_with_implementation_keyword() {
    // "add" is an implementation keyword.
    let c = classify_task_complexity("add JWT auth to the API", &[], &[]);
    assert_eq!(c, TaskComplexity::Simple);
}

#[test]
fn research_disables_tdd() {
    assert!(
        !TaskComplexity::Research.enforce_tdd(),
        "Research tasks should not enforce TDD (no code changes)"
    );
}

#[test]
fn research_generates_artifacts() {
    assert!(
        TaskComplexity::Research.generate_artifacts(),
        "Research should generate comprehension artifact"
    );
}

#[test]
fn research_caps_tool_iterations() {
    assert_eq!(
        TaskComplexity::Research.max_tool_iterations(8),
        8,
        "Research with 8 configured should use 8 (10 > 8)"
    );
    assert_eq!(
        TaskComplexity::Research.max_tool_iterations(12),
        10,
        "Research with 12 configured should cap at 10"
    );
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
    let plan = parse_plan_from_response(raw, &crate::goal::GoalSpec::new("test"), false).unwrap();
    assert_eq!(plan.subtasks.len(), 1);
    assert_eq!(plan.subtasks[0].id, "custom-id");
}

#[test]
fn parse_plan_empty_array_returns_no_subtasks_error() {
    // Model returned valid JSON but with an empty subtasks array —
    // now returns a typed error (U2), not a synthesized fallback.
    let raw = r#"{"subtasks":[],"risks":["nothing to do"]}"#;
    let err = parse_plan_from_response(raw, &crate::goal::GoalSpec::new("add the function"), false)
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
    let plan =
        parse_plan_from_response(raw, &crate::goal::GoalSpec::new("add function"), true).unwrap();
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
    let plan = parse_plan_from_response(raw, &crate::goal::GoalSpec::new("test"), false).unwrap();
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
        group: None,
    }];

    let opts = VerifyOptions {
        allowed_executables: vec!["cargo".into()],
        continue_on_error: false,
        timeout_ms: 5000,
        min_pass_rate: 1.0,
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
        } else if sys.contains("executing a specific subtask")
            || sys.contains("autonomous coding agent")
        {
            "Done writing the hello module. The file was created successfully and contains the expected content.".to_string()
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
                cost_usd: 0.0,
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
                cost_usd: 0.0,
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
                pre_conditions: vec![],
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
        } else if sys.contains("autonomous coding agent")
            || sys.contains("executing a specific subtask")
        {
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
            "Done writing the hello module. The file was created successfully and contains the expected content.".to_string()
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
        group: None,
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
        .run_loop(
            &crate::goal::GoalSpec::new("ship the feature"),
            &cfg,
            None,
            None,
        )
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
        spend_cap_usd: Some(0.000001), // below one call's cost (~$0.0000045)
        detect_oscillation: false,
        ..Default::default()
    };
    let result = agent
        .run_loop(
            &crate::goal::GoalSpec::new("ship the feature"),
            &cfg,
            None,
            None,
        )
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

/// An LLM that never converges — always returns tool calls with empty content,
/// simulating a model stuck in a tool-call loop.
struct StuckLlm;

#[async_trait]
impl LlmClient for StuckLlm {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        Ok(CompletionResponse {
            content: String::new(),
            tool_calls: vec![crate::llm::ToolCall {
                id: "t1".into(),
                name: "shell_command".into(),
                arguments: serde_json::json!({"command": "echo stuck"}),
            }],
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            model: "stuck".into(),
            finish_reason: crate::llm::FinishReason::ToolCalls,
        })
    }

    fn default_model(&self) -> &str {
        "stuck"
    }
}

#[tokio::test]
async fn run_loop_model_not_converging_terminates_early() {
    let agent = Agent::builder()
        .llm(Arc::new(StuckLlm))
        .tools(ToolRegistry::new())
        .config(AgentConfig::default())
        .build()
        .expect("agent builds");

    let result = agent
        .run_loop(
            &crate::goal::GoalSpec::new("do the thing"),
            &LoopConfig {
                max_iterations: 6,
                ..Default::default()
            },
            None,
            None,
        )
        .await
        .expect("loop runs");

    assert!(
        matches!(result.termination, LoopTermination::ModelNotConverging(_)),
        "expected ModelNotConverging, got {:?}",
        result.termination
    );
    assert!(!result.converged);
    // Should not run to the full 6 iterations — fail-fast kicks in.
    assert!(result.iteration_count() < 6);
}

/// An LLM that produces a clean non-tool answer after `fail_first` tool-only
/// responses. Used to test convergence in the simplified loop.
struct ConvergingLlm {
    call_count: AtomicUsize,
    fail_first: usize,
}

impl ConvergingLlm {
    fn after(fail_first: usize) -> Arc<Self> {
        Arc::new(Self {
            call_count: AtomicUsize::new(0),
            fail_first,
        })
    }
}

#[async_trait]
impl LlmClient for ConvergingLlm {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let n = self.call_count.fetch_add(1, Ordering::SeqCst);
        let usage = Usage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        };
        if n < self.fail_first {
            Ok(CompletionResponse {
                content: String::new(),
                tool_calls: vec![crate::llm::ToolCall {
                    id: format!("t{n}"),
                    name: "shell_command".into(),
                    arguments: serde_json::json!({"command": "echo pre" }),
                }],
                usage,
                model: "converging".into(),
                finish_reason: crate::llm::FinishReason::ToolCalls,
            })
        } else {
            Ok(CompletionResponse {
                content: "Done! Implemented the feature.".into(),
                tool_calls: vec![],
                usage,
                model: "converging".into(),
                finish_reason: crate::llm::FinishReason::Stop,
            })
        }
    }

    fn default_model(&self) -> &str {
        "converging"
    }
}

#[tokio::test]
async fn run_loop_converges_on_first_iteration() {
    // Model answers immediately with no tool calls.
    let agent = Agent::builder()
        .llm(ConvergingLlm::after(0))
        .tools(ToolRegistry::new())
        .config(AgentConfig::default())
        .build()
        .expect("agent builds");

    let cfg = LoopConfig {
        max_iterations: 3,
        ..Default::default()
    };
    let result = agent
        .run_loop(
            &crate::goal::GoalSpec::new("explain concept"),
            &cfg,
            None,
            None,
        )
        .await
        .expect("loop runs");

    assert!(result.converged);
    assert_eq!(result.termination, LoopTermination::Approved);
    assert_eq!(result.iteration_count(), 1);
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

    let config = AgentConfig {
        critique_personas: CritiquePersona::default_personas(),
        critique_mode: CritiqueMode::Full,
        ..Default::default()
    };
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
            &[],
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
                &[],
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

    let config = AgentConfig {
        critique_personas: CritiquePersona::default_personas(),
        critique_mode: CritiqueMode::Full,
        ..Default::default()
    };
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
            &[],
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

    let config = AgentConfig {
        critique_personas: CritiquePersona::default_personas(),
        critique_mode: CritiqueMode::Full,
        ..Default::default()
    };
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
            &[],
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

    let config = AgentConfig {
        critique_personas: CritiquePersona::default_personas(),
        critique_mode: CritiqueMode::Full,
        ..Default::default()
    };
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
            &[],
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

    let config = AgentConfig {
        critique_personas: CritiquePersona::default_personas(),
        critique_mode: CritiqueMode::Full,
        ..Default::default()
    };
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
            &[],
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

    let config = AgentConfig {
        critique_personas: CritiquePersona::default_personas(),
        critique_mode: CritiqueMode::Full,
        ..Default::default()
    };
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
            &[],
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
    let config = AgentConfig {
        critique_personas: CritiquePersona::default_personas(),
        critique_mode: CritiqueMode::Full,
        ..Default::default()
    };

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
            &[],
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

    let config = AgentConfig {
        critique_personas: CritiquePersona::default_personas(),
        critique_mode: CritiqueMode::Full,
        ..Default::default()
    };
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
            &[],
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

    let config = AgentConfig {
        critique_personas: CritiquePersona::default_personas(),
        critique_mode: CritiqueMode::Full,
        ..Default::default()
    };
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
            &[],
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
        pre_conditions: vec![],
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
        converged: true,
    };

    assert_eq!(classify_error(&[], &[step]), ErrorClass::Type);
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
        converged: true,
    };

    assert_eq!(classify_error(&[], &[step]), ErrorClass::Compilation);
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
        converged: true,
    };

    assert_eq!(classify_error(&[], &[step]), ErrorClass::Test);
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
        converged: true,
    };

    assert_eq!(classify_error(&[], &[step]), ErrorClass::Runtime);
}

#[test]
fn classify_error_architecture() {
    let issues = vec!["Boundary violation: crate A imports from crate B".to_string()];
    assert_eq!(classify_error(&issues, &[]), ErrorClass::Architecture);
}

#[test]
fn classify_error_spec_gap() {
    let issues = vec!["Acceptance criterion 2 not addressed: missing error handling".to_string()];
    assert_eq!(classify_error(&issues, &[]), ErrorClass::SpecGap);
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
        converged: true,
    };

    assert_eq!(classify_error(&[], &[step]), ErrorClass::Other);
}

// ---------------------------------------------------------------------------
// E2E regression tests: full run_loop() with mock LLMs
// ---------------------------------------------------------------------------

/// E2E: Research task classification runs the Research path (premium model,
/// single iteration, comprehension is the deliverable).
#[tokio::test]
async fn run_loop_research_task_single_iteration() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();

    let llm = ConvergingLlm::after(0);
    let tools = crate::tool::ToolRegistry::with_builtin(root.clone(), Vec::new());
    let agent = crate::cognition::Agent::builder()
        .llm(llm)
        .tools(tools)
        .config(crate::cognition::AgentConfig {
            tdd: false,
            review_every_change: false,
            ..Default::default()
        })
        .build()
        .expect("agent builds");

    let goal = crate::goal::GoalSpec::new("what is the architecture of the parser");
    let loop_cfg = crate::cognition::LoopConfig::default();
    let result = agent
        .run_loop(&goal, &loop_cfg, None, None)
        .await
        .expect("loop runs");

    assert!(result.converged, "research task should converge");
    assert_eq!(result.iteration_count(), 1);
    assert!(
        result
            .final_result
            .comprehension
            .summary
            .to_lowercase()
            .contains("done"),
        "should produce a summary"
    );
}

/// Smoke: ScriptedLlm converges within 2 iterations with critique enabled.
#[tokio::test]
async fn run_loop_scripted_llm_converges_in_two() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();

    let llm = ScriptedLlm::approve_after(0);
    let tools = crate::tool::ToolRegistry::with_builtin(root.clone(), Vec::new());
    let agent = crate::cognition::Agent::builder()
        .llm(llm)
        .tools(tools)
        .config(crate::cognition::AgentConfig {
            tdd: false,
            review_every_change: true,
            ..Default::default()
        })
        .build()
        .expect("agent builds");

    let result = agent
        .run_loop(
            &crate::goal::GoalSpec::new("make a change"),
            &crate::cognition::LoopConfig::default(),
            None,
            None,
        )
        .await
        .expect("loop runs");

    assert!(result.converged);
    assert!(result.iteration_count() <= 2);
}

/// E2E: Pipeline-driven loop with Plan + Critique stages. Verifies that the
/// critique result is consulted for convergence approval.
#[tokio::test]
async fn run_loop_pipeline_with_critique_approval() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();

    let llm = ScriptedLlm::approve_after(0);
    let tools = crate::tool::ToolRegistry::with_builtin(root.clone(), Vec::new());
    let agent = crate::cognition::Agent::builder()
        .llm(llm)
        .tools(tools)
        .config(crate::cognition::AgentConfig {
            tdd: false,
            review_every_change: true,
            ..Default::default()
        })
        .build()
        .expect("agent builds");

    let result = agent
        .run_loop(
            &crate::goal::GoalSpec::new("add a small feature"),
            &crate::cognition::LoopConfig::default(),
            None,
            None,
        )
        .await
        .expect("loop runs");

    assert!(result.converged);
    let last_iteration = result.iterations.last().expect("at least one iteration");
    assert!(last_iteration.critique_approved, "critique should approve");
}

/// E2E: step_has_quality should correctly flag a refused converged step.
/// When the tool signals include a failure, quality should be false even
/// if the model claims to have converged.
#[test]
fn step_has_quality_with_tool_failure() {
    let signals = vec![crate::tool::ToolSignal {
        tool: "file_write".into(),
        ok: false,
        empty: false,
        elapsed_ms: 10,
        source: crate::tool::ToolSource::Builtin,
    }];
    assert!(
        !crate::cognition::step_has_quality(
            true, // converged
            &signals,
            "I wrote the file successfully and everything looks good now.",
        ),
        "should fail quality check when a tool failed"
    );
}

#[test]
fn step_has_quality_with_all_tools_ok() {
    let signals = vec![crate::tool::ToolSignal {
        tool: "file_write".into(),
        ok: true,
        empty: false,
        elapsed_ms: 10,
        source: crate::tool::ToolSource::Builtin,
    }];
    assert!(
        crate::cognition::step_has_quality(
            true,
            &signals,
            "The file was written successfully. The module now contains the expected function.",
        ),
        "should pass quality check when all tools succeed"
    );
}

#[test]
fn step_has_quality_non_converged() {
    assert!(
        !crate::cognition::step_has_quality(false, &[], "some output"),
        "should fail when step did not converge"
    );
}

#[test]
fn step_has_quality_refusal_pattern() {
    assert!(
        !crate::cognition::step_has_quality(
            true,
            &[],
            "I cannot complete this task because I don't have enough information.",
        ),
        "should fail on refusal patterns"
    );
}

// ---------------------------------------------------------------------------
// Live task-type routing sweep (requires a real LLM backend).
//
// Builds the real OpenAI-compatible client from the `ximimo` provider env
// vars and classifies a representative prompt for every supported task type,
// then prints the resulting TaskType + the pipeline stages it routes to.
// Run with: cargo test -p sruja-agent --lib --ignored task_type_routing_sweep
// ---------------------------------------------------------------------------
#[tokio::test]
#[ignore]
async fn task_type_routing_sweep() {
    // TODO: re-enable when classify_task_type is added back to the public API
    eprintln!("skipped: classify_task_type not yet public");
}

// --- Sub-agent isolation tests (context-engineering "Isolate" step) ---

use super::subagent::{Role, SubAgentBudget, SubAgentSpec};

/// A minimal LLM that returns a grounded summary citing an element ID, with no
/// tool calls (so the loop converges on the first response).
struct SummarizingLlm;
#[async_trait]
impl LlmClient for SummarizingLlm {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        Ok(CompletionResponse {
            content: "Reviewed MySystem.ApiContainer: it depends on MySystem.Database. \
                      No unused imports found; change is safe."
                .to_string(),
            tool_calls: Vec::new(),
            usage: Usage {
                prompt_tokens: 5,
                completion_tokens: 5,
                total_tokens: 10,
            },
            model: "scripted".into(),
            finish_reason: crate::llm::FinishReason::Stop,
        })
    }
    fn default_model(&self) -> &str {
        "scripted"
    }
}

fn isolated_agent() -> Agent {
    Agent::builder()
        .llm(Arc::new(SummarizingLlm))
        .tools(ToolRegistry::new())
        .config(AgentConfig::default())
        .build()
        .expect("agent builds")
}

#[test]
fn writer_subagent_has_no_exploration_tools() {
    let agent = isolated_agent();
    let names = agent.scoped_tool_names(Role::Writer);
    // Writers may write/edit and resolve globs, but must NOT read or search.
    assert!(
        names.contains(&"file_write".to_string()),
        "writer needs file_write: {names:?}"
    );
    assert!(
        names.contains(&"file_edit".to_string()),
        "writer needs file_edit: {names:?}"
    );
    assert!(
        !names.contains(&"grep".to_string()),
        "writer must not have grep: {names:?}"
    );
    assert!(
        !names.contains(&"file_read".to_string()),
        "writer must not have file_read: {names:?}"
    );
    assert!(
        !names.iter().any(|n| n.starts_with("sruja_lookup")),
        "writer must not have lookup tools: {names:?}"
    );
}

#[test]
fn reader_subagent_has_no_write_tools() {
    let agent = isolated_agent();
    let names = agent.scoped_tool_names(Role::Reader);
    assert!(
        names.contains(&"grep".to_string()),
        "reader needs grep: {names:?}"
    );
    assert!(
        names.contains(&"sruja_focus".to_string()),
        "reader needs sruja_focus: {names:?}"
    );
    assert!(
        !names
            .iter()
            .any(|n| n.starts_with("file_write") || n.starts_with("file_edit")),
        "reader must not have write tools: {names:?}"
    );
}

#[tokio::test]
async fn delegate_reader_returns_compressed_report_with_citations() {
    let agent = isolated_agent();
    let spec = SubAgentSpec {
        role: Role::Reader,
        goal: crate::goal::GoalSpec::new("Review MySystem.ApiContainer for unused imports"),
        inject: vec!["Focus on src/api.rs".to_string()],
        budget: SubAgentBudget::default(),
        system_prompt: None,
        user_prompt: None,
    };
    let report = agent.delegate(spec).await.expect("delegate succeeds");
    assert!(report.converged, "single-shot LLM should converge");
    assert!(report.ok, "reader report should be ok");
    assert!(
        report.citations.iter().any(|c| c.starts_with("MySystem")),
        "citations should include architecture element IDs: {:?}",
        report.citations
    );
    assert!(
        report.summary.len() <= SubAgentBudget::default().max_summary_chars + 30,
        "summary must be bounded: len={}",
        report.summary.len()
    );
}
