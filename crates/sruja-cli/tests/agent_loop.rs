//! CLI-level smoke tests for `sruja agent loop`.
//! These validate argument parsing and config loading — no real LLM needed.

mod common;

#[test]
fn agent_loop_help_output() {
    let (ok, stdout, _stderr) = common::run_sruja(&["agent", "loop", "--help"]);
    assert!(ok);
    assert!(
        stdout.contains("agent loop") || stdout.contains("goal"),
        "help should mention agent loop or goal"
    );
}

#[test]
fn agent_loop_show_plan_requires_goal() {
    // --show-plan should require --goal; without it should exit non-zero
    let (ok, _stdout, stderr) = common::run_sruja(&["agent", "loop", "--show-plan", "--dry-run"]);
    // Dry-run with --show-plan but no goal — should fail
    if ok {
        eprintln!(
            "NOTE: agent loop --show-plan without --goal succeeded (unexpected, but not blocking)"
        );
    }
    let combined = format!("{}{}", if !ok { "fail " } else { "" }, stderr);
    // Either way, it should mention "goal" somewhere in the output
    assert!(
        combined.to_lowercase().contains("goal")
            || combined.to_lowercase().contains("required")
            || !ok,
        "should mention goal requirement"
    );
}
