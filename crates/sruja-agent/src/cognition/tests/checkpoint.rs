use super::*;

// --- Checkpoint tests ---

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

    checkpoint.write(tmp.path()).unwrap();
    assert!(RunCheckpoint::exists(tmp.path()));

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

    tracker.record(
        "subtasks: [s1(Implement)]".into(),
        "critic rejected: missing tests".into(),
        2,
        ErrorClass::Other,
    );
    assert_eq!(tracker.consecutive_same_approach, 2);

    tracker.record(
        "subtasks: [s1(TestAuthor), s2(Implement)]".into(),
        "critic rejected: different issue".into(),
        3,
        ErrorClass::Other,
    );
    assert_eq!(tracker.consecutive_same_approach, 1);

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

// --- Error classification tests ---

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
