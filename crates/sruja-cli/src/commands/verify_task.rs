//! Verification task: run a profile of verification steps and report results.
//!
//! Extracted from `agent_run.rs` to provide a skill-agnostic verification command
//! that any host (Cursor, CI, MCP) can use without knowing Sruja internals.
//!
//! Usage:
//! ```bash
//! sruja verify-task --profile coding -r .
//! sruja verify-task --profile bugfix --file src/auth.rs -r .
//! sruja verify-task --profile review -r .
//! sruja verify-task --profile arch -r .
//! ```

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::commands::CliError;
use crate::integrations::{load_repo_config, VerifyProfileConfig};

use super::agent_run::{
    load_allowlists, run_allowlisted_process, run_sruja_cmd, AgentStep, StepObservation,
};

/// Schema version for verify-task output.
pub const VERIFY_TASK_SCHEMA: &str = "verify_task/v2";

/// Verification profile determines which steps to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifyProfile {
    /// `lint repo.sruja` + `just check` (or `make check`) + `drift` (if arch paths touched)
    Coding,
    /// `focus --file` + `just check` (or `make check`) + `intent check`
    Bugfix,
    /// `review -f json` + `intent check` + `drift`
    Review,
    /// `lint repo.sruja` + `drift` + `intent check` + `review -f json`
    Arch,
}

impl std::str::FromStr for VerifyProfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "coding" => Ok(VerifyProfile::Coding),
            "bugfix" => Ok(VerifyProfile::Bugfix),
            "review" => Ok(VerifyProfile::Review),
            "arch" => Ok(VerifyProfile::Arch),
            other => Err(format!(
                "Unknown profile '{}'. Valid: coding, bugfix, review, arch",
                other
            )),
        }
    }
}

impl std::fmt::Display for VerifyProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifyProfile::Coding => write!(f, "coding"),
            VerifyProfile::Bugfix => write!(f, "bugfix"),
            VerifyProfile::Review => write!(f, "review"),
            VerifyProfile::Arch => write!(f, "arch"),
        }
    }
}

/// Output of a verification task run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct VerifyTaskOutput {
    pub schema_version: String,
    pub profile: String,
    pub repo: String,
    pub all_passed: bool,
    pub steps: Vec<StepObservation>,
    pub elapsed_ms: u128,
    pub provenance: VerifyProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_pack: Option<VerifyEvidencePack>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_memory: Option<crate::utils::agent_memory_signal::AgentMemorySignal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct VerifyProvenance {
    pub sruja_version: String,
    pub os: String,
    pub arch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_commit: Option<String>,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct VerifyEvidencePack {
    pub output_dir: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
}

/// Options for running a verification task.
pub struct VerifyTaskOptions<'a> {
    pub repo: &'a str,
    pub profile: &'a str,
    pub file: Option<&'a str>,
    pub max_runtime_ms: Option<u64>,
    pub evidence_pack: bool,
    pub evidence_pack_dir: Option<&'a str>,
}

fn git_head_commit(repo_path: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!s.is_empty()).then_some(s)
}

fn config_hash(repo_path: &Path) -> Option<String> {
    let p = repo_path.join(".sruja").join("config.toml");
    let txt = std::fs::read(&p).ok()?;
    Some(blake3::hash(&txt).to_hex().to_string())
}

fn default_evidence_pack_dir(repo_path: &Path) -> PathBuf {
    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    repo_path.join(".sruja").join("evidence-packs").join(ts)
}

fn binary_in_path(name: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    for dir in std::env::split_paths(&paths) {
        if dir.join(name).exists() {
            return true;
        }
        #[cfg(windows)]
        {
            if dir.join(format!("{name}.exe")).exists()
                || dir.join(format!("{name}.cmd")).exists()
                || dir.join(format!("{name}.bat")).exists()
            {
                return true;
            }
        }
    }
    false
}

fn write_evidence_pack(
    repo_path: &Path,
    output: &VerifyTaskOutput,
) -> Result<VerifyEvidencePack, CliError> {
    let dir = output
        .evidence_pack
        .as_ref()
        .map(|p| PathBuf::from(&p.output_dir))
        .unwrap_or_else(|| default_evidence_pack_dir(repo_path));
    std::fs::create_dir_all(&dir)?;

    let mut files = Vec::new();

    let verify_path = dir.join("verify-task.json");
    std::fs::write(
        &verify_path,
        serde_json::to_string_pretty(output).map_err(|e| CliError::validation(e.to_string()))?,
    )?;
    files.push(verify_path.display().to_string());

    for step in &output.steps {
        let filename = match step.step_id.as_str() {
            "drift_check" => Some("drift.json"),
            "intent_check" => Some("intent.json"),
            "review" => Some("review.json"),
            _ => None,
        };
        let Some(name) = filename else { continue };
        let trimmed = step.stdout.trim();
        if trimmed.is_empty() {
            continue;
        }
        let p = dir.join(name);
        std::fs::write(&p, trimmed)?;
        files.push(p.display().to_string());
    }

    Ok(VerifyEvidencePack {
        output_dir: dir.display().to_string(),
        files,
    })
}

/// Build verification steps for a given profile.
fn build_verification_steps(
    profile: VerifyProfile,
    repo_path: &Path,
    file: Option<&str>,
    profile_config: Option<&VerifyProfileConfig>,
) -> Vec<AgentStep> {
    // If custom steps are defined in config, use those
    if let Some(cfg) = profile_config {
        if let Some(ref steps) = cfg.steps {
            return build_steps_from_config(steps, profile, repo_path, file);
        }
    }

    let mut steps = Vec::new();
    let has_makefile = repo_path.join("Makefile").exists() || repo_path.join("makefile").exists();
    let has_justfile = repo_path.join("justfile").exists() || repo_path.join("Justfile").exists();
    let has_build_system = has_makefile || has_justfile;
    let _ = has_build_system; // Reserved for future conditional step logic

    match profile {
        VerifyProfile::Coding => {
            // lint repo.sruja (if it exists)
            if repo_path.join("repo.sruja").exists() {
                steps.push(AgentStep {
                    id: "lint_repo_sruja".to_string(),
                    kind: "sruja_cmd".to_string(),
                    argv: vec![
                        "sruja".to_string(),
                        "lint".to_string(),
                        "repo.sruja".to_string(),
                        "--format".to_string(),
                        "json".to_string(),
                    ],
                    expected: Some("repo.sruja parses and lints cleanly".to_string()),
                });
            }
            // make/just check (or sruja check fallback)
            if has_justfile && binary_in_path("just") {
                steps.push(AgentStep {
                    id: "just_check".to_string(),
                    kind: "verify_cmd".to_string(),
                    argv: vec!["just".to_string(), "check".to_string()],
                    expected: Some("fmt + lint + test pass".to_string()),
                });
            } else if has_makefile && binary_in_path("make") {
                steps.push(AgentStep {
                    id: "make_check".to_string(),
                    kind: "verify_cmd".to_string(),
                    argv: vec!["make".to_string(), "check".to_string()],
                    expected: Some("fmt + lint + test pass".to_string()),
                });
            } else {
                steps.push(AgentStep {
                    id: "sruja_check".to_string(),
                    kind: "sruja_cmd".to_string(),
                    argv: vec![
                        "sruja".to_string(),
                        "check".to_string(),
                        "-r".to_string(),
                        ".".to_string(),
                        "-f".to_string(),
                        "github-actions".to_string(),
                    ],
                    expected: Some(
                        "CI-style drift check passes (no Makefile/justfile found)".to_string(),
                    ),
                });
            }
            // drift check
            steps.push(AgentStep {
                id: "drift_check".to_string(),
                kind: "sruja_cmd".to_string(),
                argv: vec![
                    "sruja".to_string(),
                    "drift".to_string(),
                    "-r".to_string(),
                    ".".to_string(),
                    "-f".to_string(),
                    "json".to_string(),
                ],
                expected: Some("No new architectural drift".to_string()),
            });
        }
        VerifyProfile::Bugfix => {
            // focus on the file
            if let Some(f) = file {
                steps.push(AgentStep {
                    id: "focus_file".to_string(),
                    kind: "sruja_cmd".to_string(),
                    argv: vec![
                        "sruja".to_string(),
                        "focus".to_string(),
                        "--file".to_string(),
                        f.to_string(),
                        "-r".to_string(),
                        ".".to_string(),
                        "-f".to_string(),
                        "json".to_string(),
                    ],
                    expected: Some("Focus briefing generated for the bugfix target".to_string()),
                });
            }
            // make/just check (or sruja check fallback)
            if has_justfile {
                steps.push(AgentStep {
                    id: "just_check".to_string(),
                    kind: "verify_cmd".to_string(),
                    argv: vec!["just".to_string(), "check".to_string()],
                    expected: Some("fmt + lint + test pass".to_string()),
                });
            } else if has_makefile {
                steps.push(AgentStep {
                    id: "make_check".to_string(),
                    kind: "verify_cmd".to_string(),
                    argv: vec!["make".to_string(), "check".to_string()],
                    expected: Some("fmt + lint + test pass".to_string()),
                });
            } else {
                steps.push(AgentStep {
                    id: "sruja_check".to_string(),
                    kind: "sruja_cmd".to_string(),
                    argv: vec![
                        "sruja".to_string(),
                        "check".to_string(),
                        "-r".to_string(),
                        ".".to_string(),
                        "-f".to_string(),
                        "github-actions".to_string(),
                    ],
                    expected: Some(
                        "CI-style drift check passes (no Makefile/justfile found)".to_string(),
                    ),
                });
            }
            // intent check
            steps.push(AgentStep {
                id: "intent_check".to_string(),
                kind: "sruja_cmd".to_string(),
                argv: vec![
                    "sruja".to_string(),
                    "intent".to_string(),
                    "check".to_string(),
                    "-r".to_string(),
                    ".".to_string(),
                    "-f".to_string(),
                    "json".to_string(),
                ],
                expected: Some("Intent vs reality check passes".to_string()),
            });
        }
        VerifyProfile::Review => {
            // review
            steps.push(AgentStep {
                id: "review".to_string(),
                kind: "sruja_cmd".to_string(),
                argv: vec![
                    "sruja".to_string(),
                    "review".to_string(),
                    "-r".to_string(),
                    ".".to_string(),
                    "-f".to_string(),
                    "json".to_string(),
                ],
                expected: Some("Review suggestions captured".to_string()),
            });
            // intent check
            steps.push(AgentStep {
                id: "intent_check".to_string(),
                kind: "sruja_cmd".to_string(),
                argv: vec![
                    "sruja".to_string(),
                    "intent".to_string(),
                    "check".to_string(),
                    "-r".to_string(),
                    ".".to_string(),
                    "-f".to_string(),
                    "json".to_string(),
                ],
                expected: Some("Intent vs reality check passes".to_string()),
            });
            // drift
            steps.push(AgentStep {
                id: "drift_check".to_string(),
                kind: "sruja_cmd".to_string(),
                argv: vec![
                    "sruja".to_string(),
                    "drift".to_string(),
                    "-r".to_string(),
                    ".".to_string(),
                    "-f".to_string(),
                    "json".to_string(),
                ],
                expected: Some("No new architectural drift".to_string()),
            });
        }
        VerifyProfile::Arch => {
            // lint repo.sruja
            if repo_path.join("repo.sruja").exists() {
                steps.push(AgentStep {
                    id: "lint_repo_sruja".to_string(),
                    kind: "sruja_cmd".to_string(),
                    argv: vec![
                        "sruja".to_string(),
                        "lint".to_string(),
                        "repo.sruja".to_string(),
                        "--format".to_string(),
                        "json".to_string(),
                    ],
                    expected: Some("repo.sruja parses and lints cleanly".to_string()),
                });
            }
            // drift
            steps.push(AgentStep {
                id: "drift_check".to_string(),
                kind: "sruja_cmd".to_string(),
                argv: vec![
                    "sruja".to_string(),
                    "drift".to_string(),
                    "-r".to_string(),
                    ".".to_string(),
                    "-f".to_string(),
                    "json".to_string(),
                ],
                expected: Some("No new architectural drift".to_string()),
            });
            // intent check
            steps.push(AgentStep {
                id: "intent_check".to_string(),
                kind: "sruja_cmd".to_string(),
                argv: vec![
                    "sruja".to_string(),
                    "intent".to_string(),
                    "check".to_string(),
                    "-r".to_string(),
                    ".".to_string(),
                    "-f".to_string(),
                    "json".to_string(),
                ],
                expected: Some("Intent vs reality check passes".to_string()),
            });
            // review
            steps.push(AgentStep {
                id: "review".to_string(),
                kind: "sruja_cmd".to_string(),
                argv: vec![
                    "sruja".to_string(),
                    "review".to_string(),
                    "-r".to_string(),
                    ".".to_string(),
                    "-f".to_string(),
                    "json".to_string(),
                ],
                expected: Some("Review suggestions captured".to_string()),
            });
        }
    }

    steps
}

/// Build steps from config-defined step names.
fn build_steps_from_config(
    step_names: &[String],
    _profile: VerifyProfile,
    repo_path: &Path,
    file: Option<&str>,
) -> Vec<AgentStep> {
    let mut steps = Vec::new();
    for name in step_names {
        match name.as_str() {
            "lint" => {
                if repo_path.join("repo.sruja").exists() {
                    steps.push(AgentStep {
                        id: "lint_repo_sruja".to_string(),
                        kind: "sruja_cmd".to_string(),
                        argv: vec![
                            "sruja".to_string(),
                            "lint".to_string(),
                            "repo.sruja".to_string(),
                            "--format".to_string(),
                            "json".to_string(),
                        ],
                        expected: Some("repo.sruja lints cleanly".to_string()),
                    });
                }
            }
            "check" => {
                if repo_path.join("justfile").exists() || repo_path.join("Justfile").exists() {
                    steps.push(AgentStep {
                        id: "just_check".to_string(),
                        kind: "verify_cmd".to_string(),
                        argv: vec!["just".to_string(), "check".to_string()],
                        expected: Some("just check passes".to_string()),
                    });
                } else if repo_path.join("Makefile").exists() || repo_path.join("makefile").exists()
                {
                    steps.push(AgentStep {
                        id: "make_check".to_string(),
                        kind: "verify_cmd".to_string(),
                        argv: vec!["make".to_string(), "check".to_string()],
                        expected: Some("make check passes".to_string()),
                    });
                } else {
                    steps.push(AgentStep {
                        id: "sruja_check".to_string(),
                        kind: "sruja_cmd".to_string(),
                        argv: vec![
                            "sruja".to_string(),
                            "check".to_string(),
                            "-r".to_string(),
                            ".".to_string(),
                            "-f".to_string(),
                            "github-actions".to_string(),
                        ],
                        expected: Some("sruja check passes".to_string()),
                    });
                }
            }
            "drift" | "drift-if-arch" => {
                steps.push(AgentStep {
                    id: "drift_check".to_string(),
                    kind: "sruja_cmd".to_string(),
                    argv: vec![
                        "sruja".to_string(),
                        "drift".to_string(),
                        "-r".to_string(),
                        ".".to_string(),
                        "-f".to_string(),
                        "json".to_string(),
                    ],
                    expected: Some("No architectural drift".to_string()),
                });
            }
            "intent" => {
                steps.push(AgentStep {
                    id: "intent_check".to_string(),
                    kind: "sruja_cmd".to_string(),
                    argv: vec![
                        "sruja".to_string(),
                        "intent".to_string(),
                        "check".to_string(),
                        "-r".to_string(),
                        ".".to_string(),
                        "-f".to_string(),
                        "json".to_string(),
                    ],
                    expected: Some("Intent check passes".to_string()),
                });
            }
            "review" => {
                steps.push(AgentStep {
                    id: "review".to_string(),
                    kind: "sruja_cmd".to_string(),
                    argv: vec![
                        "sruja".to_string(),
                        "review".to_string(),
                        "-r".to_string(),
                        ".".to_string(),
                        "-f".to_string(),
                        "json".to_string(),
                    ],
                    expected: Some("Review passes".to_string()),
                });
            }
            "focus" => {
                if let Some(f) = file {
                    steps.push(AgentStep {
                        id: "focus_file".to_string(),
                        kind: "sruja_cmd".to_string(),
                        argv: vec![
                            "sruja".to_string(),
                            "focus".to_string(),
                            "--file".to_string(),
                            f.to_string(),
                            "-r".to_string(),
                            ".".to_string(),
                            "-f".to_string(),
                            "json".to_string(),
                        ],
                        expected: Some("Focus briefing generated".to_string()),
                    });
                }
            }
            other => {
                eprintln!("warning: unknown verify step '{}', skipping", other);
            }
        }
    }
    steps
}

/// Run verification steps and return results.
async fn run_verify_steps(
    repo_path: &Path,
    steps: &[AgentStep],
    max_runtime_ms: u64,
    allowed_sruja: &[String],
    allowed_execs: &[String],
) -> Result<Vec<StepObservation>, CliError> {
    let mut results = Vec::new();

    for step in steps {
        let obs = match step.kind.as_str() {
            "sruja_cmd" => {
                run_sruja_cmd(repo_path, &step.argv, max_runtime_ms, allowed_sruja).await?
            }
            "verify_cmd" => {
                run_allowlisted_process(repo_path, &step.argv, max_runtime_ms, allowed_execs)
                    .await?
            }
            _ => StepObservation {
                step_id: step.id.clone(),
                status: "skipped".to_string(),
                exit_code: None,
                stdout: "".to_string(),
                stderr: format!("Unknown verification kind: {}", step.kind),
                elapsed_ms: 0,
            },
        };

        results.push(obs);

        // Fail-fast: stop on first error
        if results.last().unwrap().status == "error" {
            break;
        }
    }

    Ok(results)
}

/// Run verification and return structured output.
pub async fn verify_task(options: VerifyTaskOptions<'_>) -> Result<VerifyTaskOutput, CliError> {
    let repo_path = Path::new(options.repo);
    if !repo_path.exists() {
        return Err(CliError::validation(format!(
            "Repository not found: {}",
            options.repo
        )));
    }

    let profile = options
        .profile
        .parse::<VerifyProfile>()
        .map_err(CliError::validation)?;

    let start = std::time::Instant::now();

    let (allowed_sruja, allowed_execs, _) = load_allowlists(repo_path);
    let max_runtime_ms = options.max_runtime_ms.unwrap_or(30_000);

    // Get profile-specific config if available
    let cfg = load_repo_config(repo_path);
    let profile_config = cfg.as_ref().and_then(|c| match profile {
        VerifyProfile::Coding => c.verify.coding.as_ref(),
        VerifyProfile::Bugfix => c.verify.bugfix.as_ref(),
        VerifyProfile::Review => c.verify.review.as_ref(),
        VerifyProfile::Arch => c.verify.arch.as_ref(),
    });

    let steps = build_verification_steps(profile, repo_path, options.file, profile_config);
    let results = run_verify_steps(
        repo_path,
        &steps,
        max_runtime_ms,
        &allowed_sruja,
        &allowed_execs,
    )
    .await?;

    let all_passed = results
        .iter()
        .all(|r| matches!(r.status.as_str(), "ok" | "skipped"));
    let elapsed_ms = start.elapsed().as_millis();
    let agent_memory = crate::utils::agent_memory_signal::read_agent_memory_signal(repo_path)?;

    let mut out = VerifyTaskOutput {
        schema_version: VERIFY_TASK_SCHEMA.to_string(),
        profile: profile.to_string(),
        repo: options.repo.to_string(),
        all_passed,
        steps: results,
        elapsed_ms,
        provenance: VerifyProvenance {
            sruja_version: env!("CARGO_PKG_VERSION").to_string(),
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            config_hash: config_hash(repo_path),
            repo_commit: git_head_commit(repo_path),
            generated_at: chrono::Utc::now().to_rfc3339(),
        },
        evidence_pack: None,
        agent_memory,
    };

    if options.evidence_pack || options.evidence_pack_dir.is_some() {
        let dir = options
            .evidence_pack_dir
            .map(PathBuf::from)
            .unwrap_or_else(|| default_evidence_pack_dir(repo_path));
        out.evidence_pack = Some(VerifyEvidencePack {
            output_dir: dir.display().to_string(),
            files: Vec::new(),
        });
        let pack = write_evidence_pack(repo_path, &out)?;
        out.evidence_pack = Some(pack);
    }

    Ok(out)
}

/// Format verification output for display.
pub fn format_verify_task(output: &VerifyTaskOutput, format: &str) -> String {
    match format {
        "json" => serde_json::to_string_pretty(output).unwrap_or_default(),
        _ => {
            let mut lines = Vec::new();
            lines.push(format!("Verification Profile: {}", output.profile));
            lines.push(format!("Repository: {}", output.repo));
            lines.push(format!(
                "All Passed: {}",
                if output.all_passed { "yes" } else { "no" }
            ));
            lines.push(format!("Elapsed: {}ms", output.elapsed_ms));
            lines.push(String::new());

            for step in &output.steps {
                let icon = match step.status.as_str() {
                    "ok" => "[OK]",
                    "skipped" => "[SKIP]",
                    _ => "[FAIL]",
                };
                lines.push(format!("{} {} ({}ms)", icon, step.step_id, step.elapsed_ms));
                if step.status == "error" && !step.stderr.is_empty() {
                    lines.push(format!(
                        "  stderr: {}",
                        step.stderr.lines().next().unwrap_or("")
                    ));
                }
            }

            if let Some(ref memory) = output.agent_memory {
                if memory.is_stale {
                    lines.push(String::new());
                    lines.push(format!(
                        "[WARN] Agent memory adoption low ({} learnings). Record guardrails when Sruja catches a miss.",
                        memory.learnings_count
                    ));
                }
            }

            lines.join("\n")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_from_str_valid() {
        assert_eq!(
            "coding".parse::<VerifyProfile>().unwrap(),
            VerifyProfile::Coding
        );
        assert_eq!(
            "bugfix".parse::<VerifyProfile>().unwrap(),
            VerifyProfile::Bugfix
        );
        assert_eq!(
            "review".parse::<VerifyProfile>().unwrap(),
            VerifyProfile::Review
        );
        assert_eq!(
            "arch".parse::<VerifyProfile>().unwrap(),
            VerifyProfile::Arch
        );
    }

    #[test]
    fn profile_from_str_invalid() {
        assert!("invalid".parse::<VerifyProfile>().is_err());
    }

    #[test]
    fn profile_display() {
        assert_eq!(VerifyProfile::Coding.to_string(), "coding");
        assert_eq!(VerifyProfile::Bugfix.to_string(), "bugfix");
    }

    #[test]
    fn coding_profile_has_steps() {
        let steps = build_verification_steps(VerifyProfile::Coding, Path::new("."), None, None);
        assert!(!steps.is_empty());
        // Should have at least check and drift
        assert!(steps.len() >= 2);
    }

    #[test]
    fn bugfix_profile_with_file() {
        let steps = build_verification_steps(
            VerifyProfile::Bugfix,
            Path::new("."),
            Some("src/main.rs"),
            None,
        );
        assert!(!steps.is_empty());
        // Should have focus, check, intent
        assert!(steps.len() >= 3);
    }
}
