use super::*;

// --- Verify veto test ---

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

// --- Loop spine tests ---

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

// --- ACT phase tests ---

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

    assert!(result.converged, "expected convergence");
    assert_eq!(result.iteration_count(), 1);

    let written = std::fs::read_to_string(root.join("src/hello.rs")).expect("file was written");
    assert!(written.contains("hello from the loop"));
}

#[tokio::test]
async fn run_loop_convergence_vetoed_by_deterministic_verifier() {
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
    assert!(result
        .iterations
        .iter()
        .all(|i| !i.verify_failed.is_empty()));
}

// --- Guardrail tests ---

#[tokio::test]
async fn run_loop_terminates_on_spend_cap() {
    let llm = ScriptedLlm::approve_after(usize::MAX);
    let agent = loop_test_agent(llm);
    let cfg = LoopConfig {
        max_iterations: 5,
        spend_cap_usd: Some(0.000001),
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
    assert!(result.iteration_count() < 5);
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
    assert!(result.iteration_count() < 6);
}

#[tokio::test]
async fn run_loop_converges_on_first_iteration() {
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

// --- E2E regression tests ---

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
            true,
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
