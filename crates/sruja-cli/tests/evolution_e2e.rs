//! E2E tests for sruja evaluate and sruja evolution commands.

mod common;
use common::{create_test_repo, run_sruja, write_file};

const FITNESS_SRUJA: &str = r#"
fitness AccuracyTarget {
  target "success_rate > 99.0%"
  measure "sh test_script.sh"
}
"#;

#[test]
fn test_evaluate_command_basic() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", FITNESS_SRUJA);

    // Create the mock script
    write_file(
        repo.path(),
        "test_script.sh",
        "#!/bin/sh\necho \"success_rate: 99.5%\"\nexit 0\n",
    );

    let path_str = repo.path().join("arch.sruja").to_str().unwrap().to_string();

    // Run evaluate
    let (success, stdout, stderr) = run_sruja(&["evaluate", "-a", &path_str]);
    assert!(success, "evaluate should succeed: stderr={}", stderr);
    assert!(
        stdout.contains("AccuracyTarget"),
        "should evaluate AccuracyTarget"
    );
    assert!(
        stdout.contains("Result: [PASS]"),
        "should pass the fitness check"
    );

    // Also assert that evolution log works
    let repo_str = repo.path().to_str().unwrap();
    let (log_success, log_stdout, log_stderr) = run_sruja(&["evolution", "log", "-r", repo_str]);
    assert!(
        log_success,
        "evolution log should succeed: stderr={}",
        log_stderr
    );
    assert!(
        log_stdout.contains("AccuracyTarget"),
        "log should contain the evaluated fitness ID"
    );
    assert!(
        log_stdout.contains("PASS"),
        "log should record success state"
    );
}

#[test]
fn test_evaluate_command_no_fitness() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "arch.sruja",
        "system = kind \"System\"\nS = system \"S\" { description \"x\" }",
    );
    let path_str = repo.path().join("arch.sruja").to_str().unwrap().to_string();

    let (success, stdout, stderr) = run_sruja(&["evaluate", "-a", &path_str]);
    assert!(
        success,
        "evaluate should succeed gracefully: stderr={}",
        stderr
    );
    assert!(
        stdout.contains("No fitness functions declared"),
        "should warn about no fitness functions"
    );
}

#[test]
fn test_evolution_log_empty() {
    let repo = create_test_repo();
    let repo_str = repo.path().to_str().unwrap();

    let (success, stdout, stderr) = run_sruja(&["evolution", "log", "-r", repo_str]);
    assert!(
        success,
        "evolution log should succeed gracefully: stderr={}",
        stderr
    );
    assert!(
        stdout.contains("No evolution history found"),
        "should report empty log"
    );
}
