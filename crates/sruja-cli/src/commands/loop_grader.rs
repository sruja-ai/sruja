//! Default in-loop grader for autonomous coding.
//!
//! This module constructs the default deterministic grader that makes
//! Sruja's "never self-graded" thesis true without configuration.
//!
//! The default grader runs only sruja's own deterministic architecture checks:
//! - `sruja lint repo.sruja` (if contract exists)
//! - `sruja drift --structural-only --fail-on <criteria>`
//!
//! These checks are read-only, fast, and architecture-focused. They run on
//! every iteration of the agent loop, vetoes convergence on failure, and feed
//! failures into the LLM critic for replanning.

use std::path::Path;

use sruja_agent::verify::VerifyStep;

/// The default in-loop grader: sruja's own deterministic architecture checks.
/// Runs only sruja subcommands (read-only, trusted). No arbitrary executables.
pub fn default_grader_steps(repo_path: &Path, sruja_bin: &str, fail_on: &str) -> Vec<VerifyStep> {
    let mut steps = Vec::new();

    let repo_sruja = repo_path.join("repo.sruja");

    if repo_sruja.exists() {
        steps.push(VerifyStep {
            id: "grader_lint_contract".to_string(),
            command: sruja_bin.to_string(),
            args: vec![
                "lint".to_string(),
                repo_sruja.to_string_lossy().to_string(),
                "--format".to_string(),
                "json".to_string(),
            ],
            expected: None,
        });
    }

    steps.push(VerifyStep {
        id: "grader_drift".to_string(),
        command: sruja_bin.to_string(),
        args: vec![
            "drift".to_string(),
            "-r".to_string(),
            ".".to_string(),
            "--structural-only".to_string(),
            "--fail-on".to_string(),
            fail_on.to_string(),
            "-f".to_string(),
            "json".to_string(),
        ],
        expected: None,
    });

    steps
}

/// Resolve the sruja binary path for spawning as a subprocess.
///
/// Returns the canonical path to the running executable when possible
/// (covers both installed `sruja` and `cargo run`), falling back to
/// "sruja" for PATH resolution (covers test scenarios).
pub fn resolve_sruja_binary() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.canonicalize().ok())
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "sruja".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_grader_steps_empty_repo_no_contract() {
        let steps = default_grader_steps(
            Path::new("/nonexistent"),
            "sruja",
            "cycles,layer-violations",
        );
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].id, "grader_drift");
        assert_eq!(steps[0].command, "sruja");
        assert!(steps[0].args.contains(&"--fail-on".to_string()));
        assert!(steps[0]
            .args
            .contains(&"cycles,layer-violations".to_string()));
    }

    #[test]
    fn default_grader_steps_with_contract() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path();
        std::fs::write(repo_path.join("repo.sruja"), "Test = system \"Test\" {}").unwrap();

        let steps = default_grader_steps(repo_path, "sruja", "all");
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].id, "grader_lint_contract");
        assert_eq!(steps[1].id, "grader_drift");
    }

    #[test]
    fn default_grader_steps_sruja_bin_substituted() {
        let steps = default_grader_steps(Path::new("."), "/custom/path/to/sruja", "cycles");
        assert_eq!(steps[0].command, "/custom/path/to/sruja");
        assert_eq!(steps[1].command, "/custom/path/to/sruja");
    }

    #[test]
    fn default_grader_steps_fail_on_in_drift_args() {
        let steps = default_grader_steps(Path::new("."), "sruja", "cycles,layer-violations");
        let drift_step = steps.iter().find(|s| s.id == "grader_drift").unwrap();
        let fail_on_idx = drift_step
            .args
            .iter()
            .position(|a| a == "--fail-on")
            .unwrap();
        assert_eq!(drift_step.args[fail_on_idx + 1], "cycles,layer-violations");
    }

    #[test]
    fn default_grader_steps_all_expected_none() {
        let steps = default_grader_steps(Path::new("."), "sruja", "all");
        for step in &steps {
            assert!(
                step.expected.is_none(),
                "step {} should have expected=None",
                step.id
            );
        }
    }

    #[test]
    fn resolve_sruja_binary_fallback() {
        let bin = resolve_sruja_binary();
        assert!(!bin.is_empty());
        assert!(bin.contains("sruja") || bin == "sruja");
    }
}
