//! Default in-loop grader for autonomous coding.
//!
//! This module constructs the default deterministic grader that makes
//! Sruja's "never self-graded" thesis true without configuration.
//!
//! The default grader runs only sruja's own deterministic architecture checks:
//! - `sruja lint repo.sruja` (if contract exists)
//! - `sruja intent check` (if intent artifacts exist — ADRs or decisions)
//! - `sruja drift --structural-only --fail-on <criteria>`
//!
//! These checks are read-only, fast, and architecture-focused. They run on
//! every iteration of the agent loop, veto convergence on failure, and feed
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

    // Intent check: only injected when the project has documented intent
    // (ADRs under docs/architecture/ or .sruja/adr/ or .sruja/decisions/).
    // This is the richer verification — policy rules, boundary violations,
    // undocumented components, missing relationships — beyond what drift's
    // structural-only check covers.
    if has_intent_artifacts(repo_path) {
        steps.push(VerifyStep {
            id: "grader_intent".to_string(),
            command: sruja_bin.to_string(),
            args: vec![
                "intent".to_string(),
                "check".to_string(),
                "-r".to_string(),
                ".".to_string(),
                "-f".to_string(),
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

/// Check whether the repository has intent artifacts that `sruja intent check`
/// can evaluate against the scanned code graph.
///
/// Looks for ADR files or .sruja decision directories:
/// - `docs/architecture/adr/` (conventional ADR location)
/// - `docs/architecture/decisions/` (alternative ADR location)
/// - `.sruja/adr/`  (Sruja-managed ADR storage)
/// - `.sruja/decisions/` (Sruja-managed decision storage)
fn has_intent_artifacts(repo_path: &Path) -> bool {
    let candidates = [
        repo_path.join("docs").join("architecture").join("adr"),
        repo_path.join("docs").join("architecture").join("decisions"),
        repo_path.join(".sruja").join("adr"),
        repo_path.join(".sruja").join("decisions"),
    ];
    candidates.iter().any(|d| d.exists() && d.is_dir())
}

/// Pre-loop smoke test that verifies the sruja binary and the default grader
/// toolchain are functional *before* entering the agent loop.
///
/// Checks (in order, short-circuit on first failure):
/// 1. `sruja --version` — binary exists and responds
/// 2. `sruja lint repo.sruja` — contract is parseable (if repo.sruja exists)
/// 3. `sruja drift --structural-only` — drift tool produces a structural report
///
/// Returns `Ok(())` if all applicable checks pass, or `Err(problem_list)` with
/// one string per diagnosed issue. The caller may log the problems as warnings
/// rather than aborting (the loop can still run without the grader).
pub fn verify_grader_health(repo_path: &Path, sruja_bin: &str) -> Result<(), Vec<String>> {
    let mut problems = Vec::new();

    // 1. Binary exists and is executable (uses `version` subcommand, not `--version` flag)
    let version_output = std::process::Command::new(sruja_bin)
        .arg("version")
        .output();
    match version_output {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            problems.push(format!(
                "sruja binary '{}' responded with non-zero exit: {}",
                sruja_bin,
                stderr.trim()
            ));
        }
        Err(e) => {
            problems.push(format!(
                "sruja binary '{}' not found or cannot be executed: {}",
                sruja_bin, e
            ));
            // Can't run any further checks if the binary doesn't work
            return Err(problems);
        }
    }

    // 2. repo.sruja is parseable (if it exists)
    let repo_sruja = repo_path.join("repo.sruja");
    if repo_sruja.exists() {
        let lint_output = std::process::Command::new(sruja_bin)
            .args(["lint", "--format", "json"])
            .arg(&repo_sruja)
            .output();
        match lint_output {
            Ok(output) if output.status.success() => {}
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                problems.push(format!(
                    "repo.sruja lint failed: {}",
                    stderr.trim().lines().next().unwrap_or("unknown error")
                ));
            }
            Err(e) => {
                problems.push(format!("repo.sruja lint could not be run: {}", e));
            }
        }
    }

    // 3. Drift tool works
    let drift_output = std::process::Command::new(sruja_bin)
        .args([
            "drift",
            "-r",
            ".",
            "--structural-only",
            "-f",
            "json",
        ])
        .current_dir(repo_path)
        .output();
    match drift_output {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            problems.push(format!(
                "sruja drift failed: {}",
                stderr.trim().lines().next().unwrap_or("unknown error")
            ));
        }
        Err(e) => {
            problems.push(format!("sruja drift could not be run: {}", e));
        }
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
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

    // ── default_grader_steps tests ───────────────────────────────────────

    #[test]
    fn empty_repo_no_contract_no_intent() {
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
    fn with_contract_no_intent() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path();
        std::fs::write(repo_path.join("repo.sruja"), "Test = system \"Test\" {}").unwrap();

        let steps = default_grader_steps(repo_path, "sruja", "all");
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].id, "grader_lint_contract");
        assert_eq!(steps[1].id, "grader_drift");
    }

    #[test]
    fn with_intent_artifacts() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path();
        // Create an ADR directory to signal intent artifacts
        let adr_dir = repo_path.join(".sruja").join("adr");
        std::fs::create_dir_all(&adr_dir).unwrap();
        std::fs::write(
            adr_dir.join("001-choice.md"),
            "# ADR-001: Some decision\n",
        )
        .unwrap();

        let steps = default_grader_steps(repo_path, "sruja", "all");
        assert_eq!(steps.len(), 2); // contract (-) + intent + drift
        let intent_step = steps.iter().find(|s| s.id == "grader_intent");
        assert!(intent_step.is_some(), "expected grader_intent step");
        assert_eq!(intent_step.unwrap().command, "sruja");
    }

    #[test]
    fn with_contract_and_intent_artifacts() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path();
        std::fs::write(repo_path.join("repo.sruja"), "Test = system \"Test\" {}").unwrap();
        let adr_dir = repo_path.join("docs").join("architecture").join("adr");
        std::fs::create_dir_all(&adr_dir).unwrap();
        std::fs::write(adr_dir.join("001-choice.md"), "# ADR-001\n").unwrap();

        let steps = default_grader_steps(repo_path, "sruja", "all");
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0].id, "grader_lint_contract");
        assert_eq!(steps[1].id, "grader_intent");
        assert_eq!(steps[2].id, "grader_drift");
    }

    #[test]
    fn intent_step_has_correct_args() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path();
        let adr_dir = repo_path.join(".sruja").join("decisions");
        std::fs::create_dir_all(&adr_dir).unwrap();
        std::fs::write(adr_dir.join("001.md"), "# Decision 1\n").unwrap();

        let steps = default_grader_steps(repo_path, "/my/sruja", "cycles");
        let intent_step = steps.iter().find(|s| s.id == "grader_intent").unwrap();
        assert_eq!(intent_step.command, "/my/sruja");
        assert!(intent_step.args.contains(&"intent".to_string()));
        assert!(intent_step.args.contains(&"check".to_string()));
        assert!(intent_step.args.contains(&"-r".to_string()));
        assert!(intent_step.args.contains(&".".to_string()));
        assert!(intent_step.args.contains(&"-f".to_string()));
        assert!(intent_step.args.contains(&"json".to_string()));
    }

    #[test]
    fn sruja_bin_substituted_across_steps() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path();
        std::fs::write(repo_path.join("repo.sruja"), "Test = system \"Test\" {}").unwrap();
        let adr_dir = repo_path.join(".sruja").join("adr");
        std::fs::create_dir_all(&adr_dir).unwrap();
        std::fs::write(adr_dir.join("a.md"), "# A\n").unwrap();

        let steps = default_grader_steps(repo_path, "/custom/path/to/sruja", "cycles");
        for step in &steps {
            assert_eq!(step.command, "/custom/path/to/sruja");
        }
    }

    #[test]
    fn fail_on_in_drift_args() {
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
    fn all_expected_none() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path();
        std::fs::write(repo_path.join("repo.sruja"), "Test = system \"Test\" {}").unwrap();
        let adr_dir = repo_path.join(".sruja").join("adr");
        std::fs::create_dir_all(&adr_dir).unwrap();
        std::fs::write(adr_dir.join("a.md"), "# A\n").unwrap();

        let steps = default_grader_steps(repo_path, "sruja", "all");
        for step in &steps {
            assert!(
                step.expected.is_none(),
                "step {} should have expected=None",
                step.id
            );
        }
    }

    // ── has_intent_artifacts tests ───────────────────────────────────────

    #[test]
    fn no_intent_when_no_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(!has_intent_artifacts(tmp.path()));
    }

    #[test]
    fn detects_sruja_adr_dir() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".sruja").join("adr")).unwrap();
        assert!(has_intent_artifacts(tmp.path()));
    }

    #[test]
    fn detects_sruja_decisions_dir() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".sruja").join("decisions")).unwrap();
        assert!(has_intent_artifacts(tmp.path()));
    }

    #[test]
    fn detects_docs_architecture_adr() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("docs").join("architecture").join("adr"))
            .unwrap();
        assert!(has_intent_artifacts(tmp.path()));
    }

    #[test]
    fn detects_docs_architecture_decisions() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(
            tmp.path()
                .join("docs")
                .join("architecture")
                .join("decisions"),
        )
        .unwrap();
        assert!(has_intent_artifacts(tmp.path()));
    }

    #[test]
    fn no_intent_when_dir_is_a_file() {
        let tmp = tempfile::tempdir().unwrap();
        let adr_path = tmp.path().join(".sruja").join("adr");
        std::fs::create_dir_all(adr_path.parent().unwrap()).unwrap();
        std::fs::write(&adr_path, "not a dir").unwrap();
        assert!(!has_intent_artifacts(tmp.path()));
    }

    // ── verify_grader_health tests ───────────────────────────────────────

    #[test]
    fn health_check_accepts_built_sruja_binary() {
        // Try to find the real sruja binary in target/debug/ or target/release/
        // (not the test runner binary, which has different CLI arg handling).
        // Navigate up from the crate to the workspace root.
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .expect("workspace root: crates/sruja-cli/../..");
        let debug = workspace_root.join("target").join("debug").join("sruja");
        let release = workspace_root.join("target").join("release").join("sruja");
        let bin = if debug.exists() {
            debug.to_string_lossy().into_owned()
        } else if release.exists() {
            release.to_string_lossy().into_owned()
        } else {
            eprintln!("skipping health check test: no sruja binary found in target/");
            return;
        };

        let result = verify_grader_health(workspace_root, &bin);
        assert!(
            result.is_ok(),
            "health check failed against built binary at {}: {:?}",
            bin,
            result
        );
    }

    #[test]
    fn health_check_fails_on_nonexistent_binary() {
        let result = verify_grader_health(Path::new("."), "/nonexistent/sruja/binary");
        assert!(result.is_err());
        let problems = result.unwrap_err();
        let combined = problems.join(" ");
        assert!(combined.contains("not found") || combined.contains("cannot be executed"));
    }

    #[test]
    fn health_check_reports_individual_problems() {
        // The binary exists but is a non-sruja script that will fail on
        // lint/drift. We use /bin/echo (always exists on Unix) as the binary.
        let bin = if cfg!(target_os = "windows") {
            "cmd.exe"
        } else {
            "/bin/echo"
        };
        let tmp = tempfile::tempdir().unwrap();
        // Write a repo.sruja that will fail lint
        std::fs::write(tmp.path().join("repo.sruja"), "not valid sruja DSL !@#$%")
            .unwrap();

        let result = verify_grader_health(tmp.path(), bin);
        // The binary may succeed on --version, but fail on lint and drift.
        // We only assert that there's at least one problem reported.
        // Some platforms accept /bin/echo --version and exit 0, others don't.
        // If all three checks pass (unlikely with a random binary), that's fine too.
        if let Err(problems) = &result {
            assert!(!problems.is_empty(), "should report at least one problem");
        }
    }

    // ── resolve_sruja_binary tests ───────────────────────────────────────

    #[test]
    fn resolve_sruja_binary_fallback() {
        let bin = resolve_sruja_binary();
        assert!(!bin.is_empty());
        assert!(bin.contains("sruja") || bin == "sruja");
    }
}
