//! Generic allowlisted verification step runner.
//!
//! This module provides the execution engine used by the agent loop's
//! independent grader (cognition). It runs a sequence of allowlisted
//! commands and returns structured results. The caller controls fail-fast
//! behavior via `VerifyOptions.continue_on_error`.
//!
//! The CLI has its own kind-dispatching layer (`run_verification_steps_in_repo`
//! in `agent_run.rs`) that maps `AgentStep { kind: "sruja_cmd" | "verify_cmd" }`
//! to the appropriate runner. That CLI layer handles the dual-allowlist security
//! boundary (sruja subcommands vs. general executables) and delegates actual
//! process execution here or to `sruja_cmd` respectively.

use serde::{Deserialize, Serialize};

/// A single verification step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyStep {
    pub id: String,
    /// Executable name (must be in the allowlist).
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Expected substring in stdout/stderr for success (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
}

/// Result of a single verification step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResult {
    pub step_id: String,
    pub status: VerifyStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub elapsed_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerifyStatus {
    Ok,
    Failed,
    Skipped,
}

impl VerifyStatus {
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Ok | Self::Skipped)
    }
}

/// Options for running a batch of verification steps.
#[derive(Debug, Clone)]
pub struct VerifyOptions {
    /// If true, continue running after a failure instead of stopping.
    pub continue_on_error: bool,
    /// Timeout per step in milliseconds.
    pub timeout_ms: u64,
    /// Allowlisted executables.
    pub allowed_executables: Vec<String>,
}

impl Default for VerifyOptions {
    fn default() -> Self {
        Self {
            continue_on_error: false,
            timeout_ms: 60_000,
            allowed_executables: vec![
                "cargo".into(),
                "npm".into(),
                "just".into(),
                "make".into(),
                "git".into(),
            ],
        }
    }
}

/// Run a batch of verification steps sequentially.
///
/// This is the canonical implementation — all call sites in sruja-cli should
/// delegate here instead of re-implementing the loop.
pub async fn run_verification_steps(
    steps: &[VerifyStep],
    opts: &VerifyOptions,
    workdir: &std::path::Path,
) -> Vec<VerifyResult> {
    let mut results = Vec::with_capacity(steps.len());

    for step in steps {
        if !opts.allowed_executables.iter().any(|a| a == &step.command) {
            results.push(VerifyResult {
                step_id: step.id.clone(),
                status: VerifyStatus::Skipped,
                exit_code: None,
                stdout: String::new(),
                stderr: format!("'{}' not in allowlist", step.command),
                elapsed_ms: 0,
            });
            if !opts.continue_on_error {
                break;
            }
            continue;
        }

        let result = run_one(step, opts.timeout_ms, workdir).await;
        let is_fail = result.status == VerifyStatus::Failed;
        results.push(result);

        if is_fail && !opts.continue_on_error {
            break;
        }
    }

    results
}

async fn run_one(step: &VerifyStep, timeout_ms: u64, workdir: &std::path::Path) -> VerifyResult {
    let start = std::time::Instant::now();
    let mut cmd = tokio::process::Command::new(&step.command);
    cmd.args(&step.args).current_dir(workdir);

    let output = match tokio::time::timeout(
        std::time::Duration::from_millis(timeout_ms),
        cmd.output(),
    )
    .await
    {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => {
            return VerifyResult {
                step_id: step.id.clone(),
                status: VerifyStatus::Failed,
                exit_code: None,
                stdout: String::new(),
                stderr: format!("spawn failed: {e}"),
                elapsed_ms: start.elapsed().as_millis(),
            };
        }
        Err(_) => {
            return VerifyResult {
                step_id: step.id.clone(),
                status: VerifyStatus::Failed,
                exit_code: None,
                stdout: String::new(),
                stderr: format!("timed out after {timeout_ms}ms"),
                elapsed_ms: start.elapsed().as_millis(),
            };
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code();

    let passed = match (&step.expected, exit_code) {
        (Some(expected), _) => stdout.contains(expected) || stderr.contains(expected),
        (None, Some(0)) => true,
        (None, _) => false,
    };

    VerifyResult {
        step_id: step.id.clone(),
        status: if passed {
            VerifyStatus::Ok
        } else {
            VerifyStatus::Failed
        },
        exit_code,
        stdout,
        stderr,
        elapsed_ms: start.elapsed().as_millis(),
    }
}

/// Whether all results passed (ok or skipped).
pub fn all_passed(results: &[VerifyResult]) -> bool {
    !results.is_empty() && results.iter().all(|r| r.status.is_pass())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_passed_logic() {
        let ok = vec![VerifyResult {
            step_id: "a".into(),
            status: VerifyStatus::Ok,
            exit_code: Some(0),
            stdout: String::new(),
            stderr: String::new(),
            elapsed_ms: 1,
        }];
        assert!(all_passed(&ok));

        let mixed = vec![
            VerifyResult {
                step_id: "a".into(),
                status: VerifyStatus::Ok,
                exit_code: Some(0),
                stdout: String::new(),
                stderr: String::new(),
                elapsed_ms: 1,
            },
            VerifyResult {
                step_id: "b".into(),
                status: VerifyStatus::Failed,
                exit_code: Some(1),
                stdout: String::new(),
                stderr: String::new(),
                elapsed_ms: 1,
            },
        ];
        assert!(!all_passed(&mixed));
    }
}
