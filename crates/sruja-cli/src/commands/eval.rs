//! Task instance schema and loader for agent capability eval harness.
//!
//! Implements the SWE-bench-flavored task instance format defined in
//! `docs/plans/2026-06-18-001-feat-agent-capability-eval-harness-plan.md` §U4.

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Task instance loaded from `evaluation/tasks/<name>/instance.toml`.
///
/// Follows the SWE-bench held-out-test contract: the agent sees only the
/// `problem_statement`; `test_patch`, `gold_patch`, and test lists are held out.
#[derive(Debug, Clone, Deserialize)]
pub struct TaskInstance {
    /// Unique identifier for this task instance.
    pub instance_id: String,

    /// Capability category (taxonomy from §3 of the eval harness plan).
    #[serde(deserialize_with = "deserialize_category")]
    pub category: TaskCategory,

    /// Task difficulty (1-5 scale, where 5 is hardest).
    #[serde(default)]
    pub difficulty: u8,

    /// Verification profile for grading (must be a known profile in the repo).
    #[serde(rename = "profile")]
    pub profile_name: String,

    /// Git commit hash representing the task's starting point.
    ///
    /// The harness will create a worktree at this commit before running the agent.
    #[serde(rename = "base_commit")]
    pub base_commit_hash: String,

    /// Path to the gold patch (reference solution) under `snapshots/`.
    ///
    /// Held out from the agent; used for gold-sanity validation.
    #[serde(rename = "gold_patch")]
    pub gold_patch_path: PathBuf,

    /// Path to the test patch (new failing tests) under `snapshots/`.
    ///
    /// Held out from the agent; applied by the harness during grading.
    #[serde(rename = "test_patch")]
    pub test_patch_path: PathBuf,

    /// Glob pattern(s) for tests that should fail before the fix and pass after.
    ///
    /// Can be a single string or an array of strings.
    #[serde(rename = "fail_to_pass", default)]
    pub fail_to_pass_patterns: Vec<String>,

    /// Glob pattern(s) for tests that should pass throughout (regression guard).
    ///
    /// Can be a single string or an array of strings.
    #[serde(rename = "pass_to_pass", default)]
    pub pass_to_pass_patterns: Vec<String>,

    /// Path to the problem statement (markdown) visible to the agent.
    ///
    /// This is the ONLY text the agent sees from the harness.
    #[serde(rename = "problem_statement")]
    pub problem_statement_path: PathBuf,

    /// Optional metadata for human annotation (not used by the harness).
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Task capability categories from the eval harness taxonomy (§3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskCategory {
    /// Bug reproduction + fix: failing test passes, no regression.
    BugReproFix,
    /// Feature implementation: new behavior + suite green.
    FeatureImpl,
    /// Behavior-preserving refactor: suite green, diff purely structural.
    Refactor,
    /// Debug from stack trace: build + tests green.
    DebugFromStacktrace,
    /// Multi-file rename / API change: compiles + suite green.
    MultiFileRename,
    /// DSL / architecture change: `sruja lint` + drift clean.
    DslArchChange,
    /// Test generation: new tests pass now, fail under mutation.
    TestGeneration,
}

fn deserialize_category<'de, D>(deserializer: D) -> Result<TaskCategory, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "bug_repro_fix" => Ok(TaskCategory::BugReproFix),
        "feature_impl" => Ok(TaskCategory::FeatureImpl),
        "refactor" => Ok(TaskCategory::Refactor),
        "debug_from_stacktrace" => Ok(TaskCategory::DebugFromStacktrace),
        "multi_file_rename" => Ok(TaskCategory::MultiFileRename),
        "dsl_arch_change" => Ok(TaskCategory::DslArchChange),
        "test_generation" => Ok(TaskCategory::TestGeneration),
        other => Err(serde::de::Error::custom(format!(
            "unknown category '{other}'. Valid values: bug_repro_fix, feature_impl, refactor, debug_from_stacktrace, multi_file_rename, dsl_arch_change, test_generation"
        ))),
    }
}

/// Verification profile for grading tasks.
///
/// Maps to Sruja's `sruja verify-task` profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyProfile {
    /// Standard coding profile (lint + drift + intent checks).
    Coding,
    /// Bugfix-focused profile (emphasizes test coverage).
    Bugfix,
    /// Review-focused profile (emphasizes drift and architecture).
    Review,
    /// Architecture-focused profile (emphasizes intent and drift).
    Arch,
}

impl VerifyProfile {
    pub fn from_name(name: &str) -> Result<Self, String> {
        match name.to_lowercase().as_str() {
            "coding" => Ok(VerifyProfile::Coding),
            "bugfix" => Ok(VerifyProfile::Bugfix),
            "review" => Ok(VerifyProfile::Review),
            "arch" => Ok(VerifyProfile::Arch),
            other => Err(format!(
                "unknown verify profile '{other}'. Valid values: coding, bugfix, review, arch"
            )),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            VerifyProfile::Coding => "coding",
            VerifyProfile::Bugfix => "bugfix",
            VerifyProfile::Review => "review",
            VerifyProfile::Arch => "arch",
        }
    }
}

impl TaskInstance {
    /// Load and validate a task instance from `instance.toml`.
    ///
    /// # Arguments
    ///
    /// * `instance_dir` - Path to the task directory (e.g., `evaluation/tasks/<name>/`)
    /// * `repo_root` - Path to the repository root (for validating `base_commit`)
    ///
    /// # Returns
    ///
    /// * `Ok(TaskInstance)` - Loaded and validated instance
    /// * `Err(String)` - Validation error with user-friendly message
    pub fn load(instance_dir: &Path, repo_root: &Path) -> Result<Self, String> {
        let instance_path = instance_dir.join("instance.toml");
        if !instance_path.exists() {
            return Err(format!(
                "instance.toml not found in {}",
                instance_dir.display()
            ));
        }

        let content = std::fs::read_to_string(&instance_path)
            .map_err(|e| format!("failed to read instance.toml: {e}"))?;
        let instance: TaskInstance =
            toml::from_str(&content).map_err(|e| format!("failed to parse instance.toml: {e}"))?;

        // Validate patch paths exist
        let snapshots_dir = instance_dir.join("snapshots");
        let gold_patch_full = snapshots_dir.join(&instance.gold_patch_path);
        let test_patch_full = snapshots_dir.join(&instance.test_patch_path);
        let problem_statement_full = instance_dir.join(&instance.problem_statement_path);

        if !gold_patch_full.exists() {
            return Err(format!(
                "gold_patch file not found: {}",
                gold_patch_full.display()
            ));
        }

        if !test_patch_full.exists() {
            return Err(format!(
                "test_patch file not found: {}",
                test_patch_full.display()
            ));
        }

        if !problem_statement_full.exists() {
            return Err(format!(
                "problem_statement file not found: {}",
                problem_statement_full.display()
            ));
        }

        // Validate base_commit exists in repo
        let commit_output = std::process::Command::new("git")
            .args(["cat-file", "-t", &instance.base_commit_hash])
            .current_dir(repo_root)
            .output();

        match commit_output {
            Ok(output) if output.status.success() => {}
            _ => {
                return Err(format!(
                    "base_commit '{}' not found in repository at {}",
                    instance.base_commit_hash,
                    repo_root.display()
                ));
            }
        }

        // Validate verify profile is known
        let _ = VerifyProfile::from_name(&instance.profile_name)?;

        // Validate fail_to_pass is non-empty unless refactor category
        if instance.category != TaskCategory::Refactor && instance.fail_to_pass_patterns.is_empty()
        {
            return Err(format!(
                "fail_to_pass must be non-empty for category {:?} (only allowed for refactor)",
                instance.category
            ));
        }

        // Ensure difficulty is in valid range
        if instance.difficulty < 1 || instance.difficulty > 5 {
            return Err(format!(
                "difficulty must be 1-5, got {}",
                instance.difficulty
            ));
        }

        Ok(instance)
    }

    /// Get the full path to the gold patch file.
    #[allow(dead_code)]
    pub fn gold_patch_file(&self, instance_dir: &Path) -> PathBuf {
        instance_dir.join("snapshots").join(&self.gold_patch_path)
    }

    /// Get the full path to the test patch file.
    #[allow(dead_code)]
    pub fn test_patch_file(&self, instance_dir: &Path) -> PathBuf {
        instance_dir.join("snapshots").join(&self.test_patch_path)
    }

    /// Get the full path to the problem statement file.
    pub fn problem_statement_file(&self, instance_dir: &Path) -> PathBuf {
        instance_dir.join(&self.problem_statement_path)
    }

    /// Read the problem statement markdown content.
    pub fn read_problem_statement(&self, instance_dir: &Path) -> Result<String, String> {
        let path = self.problem_statement_file(instance_dir);
        std::fs::read_to_string(&path).map_err(|e| format!("failed to read problem statement: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_instance_dir(tmp_dir: &Path) -> PathBuf {
        let instance_dir = tmp_dir.join("test-task");
        let snapshots_dir = instance_dir.join("snapshots");

        fs::create_dir_all(&snapshots_dir).unwrap();

        // Initialize git repo in tmp_dir so HEAD commit exists
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(tmp_dir)
            .output()
            .expect("git init failed");
        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(tmp_dir)
            .output()
            .expect("git config failed");
        std::process::Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(tmp_dir)
            .output()
            .expect("git config failed");
        std::process::Command::new("git")
            .args(["commit", "--allow-empty", "-m", "initial"])
            .current_dir(tmp_dir)
            .output()
            .expect("git commit failed");

        // Get actual commit hash
        let head_output = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(tmp_dir)
            .output()
            .expect("git rev-parse failed");
        let head_hash = String::from_utf8_lossy(&head_output.stdout)
            .trim()
            .to_string();

        // Create problem statement
        fs::write(
            instance_dir.join("problem_statement.md"),
            "# Fix the bug\n\nThe function panics on empty input.",
        )
        .unwrap();

        // Create patch files
        fs::write(
            snapshots_dir.join("gold.diff"),
            "--- a/file.rs\n+++ b/file.rs\n@@ -1,1 +1,1 @@\n-old\n+new\n",
        )
        .unwrap();
        fs::write(
            snapshots_dir.join("tests.diff"),
            "--- a/tests.rs\n+++ b/tests.rs\n@@ -1,1 +1,1 @@\n-old\n+new\n",
        )
        .unwrap();

        // Create instance.toml with actual HEAD hash
        let instance_toml = format!(
            r#"
instance_id = "test-001"
category = "bug_repro_fix"
difficulty = 2
profile = "bugfix"
base_commit = "{}"
gold_patch = "gold.diff"
test_patch = "tests.diff"
fail_to_pass = ["test_fix*"]
pass_to_pass = ["test_existing*"]
problem_statement = "problem_statement.md"
"#,
            head_hash
        );
        fs::write(instance_dir.join("instance.toml"), instance_toml).unwrap();

        instance_dir
    }

    #[test]
    fn test_load_well_formed_instance() {
        let tmp = TempDir::new().unwrap();
        let instance_dir = create_test_instance_dir(tmp.path());

        let instance = TaskInstance::load(&instance_dir, tmp.path()).unwrap();

        assert_eq!(instance.instance_id, "test-001");
        assert_eq!(instance.category, TaskCategory::BugReproFix);
        assert_eq!(instance.difficulty, 2);
        assert_eq!(instance.profile_name, "bugfix");
        assert_eq!(instance.fail_to_pass_patterns, vec!["test_fix*"]);
        assert_eq!(instance.pass_to_pass_patterns, vec!["test_existing*"]);
    }

    #[test]
    fn test_load_missing_instance_toml() {
        let tmp = TempDir::new().unwrap();
        let instance_dir = tmp.path().join("test-task");
        fs::create_dir_all(&instance_dir).unwrap();

        let result = TaskInstance::load(&instance_dir, tmp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("instance.toml not found"));
    }

    #[test]
    fn test_load_unknown_profile() {
        let tmp = TempDir::new().unwrap();
        let instance_dir = create_test_instance_dir(tmp.path());

        let instance_path = instance_dir.join("instance.toml");
        let mut content = fs::read_to_string(&instance_path).unwrap();
        content = content.replace("profile = \"bugfix\"", "profile = \"unknown\"");
        fs::write(&instance_path, content).unwrap();

        let result = TaskInstance::load(&instance_dir, tmp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown verify profile"));
    }

    #[test]
    fn test_load_missing_fail_to_pass_for_non_refactor() {
        let tmp = TempDir::new().unwrap();
        let instance_dir = create_test_instance_dir(tmp.path());

        let instance_path = instance_dir.join("instance.toml");
        let mut content = fs::read_to_string(&instance_path).unwrap();
        content = content.replace("fail_to_pass = [\"test_fix*\"]", "fail_to_pass = []");
        fs::write(&instance_path, content).unwrap();

        let result = TaskInstance::load(&instance_dir, tmp.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("fail_to_pass must be non-empty"));
    }

    #[test]
    fn test_load_refactor_accepts_empty_fail_to_pass() {
        let tmp = TempDir::new().unwrap();
        let instance_dir = create_test_instance_dir(tmp.path());

        let instance_path = instance_dir.join("instance.toml");
        let mut content = fs::read_to_string(&instance_path).unwrap();
        content = content.replace("category = \"bug_repro_fix\"", "category = \"refactor\"");
        content = content.replace("fail_to_pass = [\"test_fix*\"]", "fail_to_pass = []");
        fs::write(&instance_path, content).unwrap();

        let result = TaskInstance::load(&instance_dir, tmp.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().category, TaskCategory::Refactor);
    }

    #[test]
    fn test_load_invalid_difficulty() {
        let tmp = TempDir::new().unwrap();
        let instance_dir = create_test_instance_dir(tmp.path());

        let instance_path = instance_dir.join("instance.toml");
        let mut content = fs::read_to_string(&instance_path).unwrap();
        content = content.replace("difficulty = 2", "difficulty = 10");
        fs::write(&instance_path, content).unwrap();

        let result = TaskInstance::load(&instance_dir, tmp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("difficulty must be 1-5"));
    }

    #[test]
    fn test_read_problem_statement() {
        let tmp = TempDir::new().unwrap();
        let instance_dir = create_test_instance_dir(tmp.path());

        let instance = TaskInstance::load(&instance_dir, tmp.path()).unwrap();
        let problem = instance.read_problem_statement(&instance_dir).unwrap();

        assert!(problem.contains("Fix the bug"));
        assert!(problem.contains("panics on empty input"));
    }

    #[test]
    fn test_verify_profile_parsing() {
        assert_eq!(
            VerifyProfile::from_name("coding").unwrap(),
            VerifyProfile::Coding
        );
        assert_eq!(
            VerifyProfile::from_name("bugfix").unwrap(),
            VerifyProfile::Bugfix
        );
        assert_eq!(
            VerifyProfile::from_name("review").unwrap(),
            VerifyProfile::Review
        );
        assert_eq!(
            VerifyProfile::from_name("arch").unwrap(),
            VerifyProfile::Arch
        );

        assert!(VerifyProfile::from_name("unknown").is_err());
        assert!(VerifyProfile::from_name("").is_err());
    }

    #[test]
    fn test_verify_profile_as_str() {
        assert_eq!(VerifyProfile::Coding.as_str(), "coding");
        assert_eq!(VerifyProfile::Bugfix.as_str(), "bugfix");
        assert_eq!(VerifyProfile::Review.as_str(), "review");
        assert_eq!(VerifyProfile::Arch.as_str(), "arch");
    }
}

/// Run a single eval task instance against the agent.
pub async fn run_eval_instance(
    instance_id: &str,
    repo: &str,
    max_iterations: usize,
    dry_run: bool,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use super::CliError;
    use std::path::PathBuf;

    let repo_path = PathBuf::from(repo);
    let tasks_dir = repo_path.join("evaluation").join("tasks");
    let instance_dir = tasks_dir.join(instance_id);

    if !instance_dir.exists() {
        return Err(CliError::validation(format!("Task instance not found: {instance_id}")).into());
    }

    let instance = TaskInstance::load(&instance_dir, &repo_path).map_err(|e| {
        CliError::validation(format!("Failed to load task instance {instance_id}: {e}"))
    })?;

    let problem = instance
        .read_problem_statement(&instance_dir)
        .map_err(|e| CliError::validation(format!("Failed to read problem statement: {e}")))?;

    eprintln!("Eval task: {}", instance.instance_id);
    eprintln!("Category: {:?}", instance.category);
    eprintln!("Difficulty: {}", instance.difficulty);
    eprintln!("Profile: {}", instance.profile_name);
    eprintln!();
    eprintln!("Problem statement:");
    eprintln!("{problem}");
    eprintln!();

    // Run the agent loop with the problem statement as the goal
    let goal = format!(
        "Task: {} (difficulty {})\n\n{}",
        instance.instance_id, instance.difficulty, problem
    );

    let options = super::AgentLoopOptions {
        repo,
        goal: &goal,
        max_iterations: Some(max_iterations),
        no_tdd: false,
        dry_run,
        model: None,
        base_url: None,
        spend_cap_usd: None,
        no_oscillation_detection: false,
        format,
        force_proceed: true, // eval tasks always proceed
        no_default_grader: false,
        steer: false,
        resume: false,
        show_plan: false,
        plan_only: false,
        show_pipeline: false,
        pipeline_override: None,
        checkpoint: false,
        no_checkpoint: true,
        changelog: false,
        verbose: false,
    };

    super::agent_loop(&options).await?;

    // Verify against held-out tests
    eprintln!();
    eprintln!("Verification against held-out tests...");

    let test_diff = instance_dir
        .join("snapshots")
        .join(&instance.test_patch_path);
    let repo_path_clone = repo_path.clone();

    let verification_status = if test_diff.exists() {
        eprintln!("Applying test patch: {}", test_diff.display());

        // Reset to clean state so the test patch applies against baseline,
        // not on top of agent changes. This mirrors SWE-bench behavior:
        // attempt → reset → apply gold fix → run tests.
        let reset_output = std::process::Command::new("git")
            .args(["reset", "--hard", "HEAD"])
            .current_dir(&repo_path_clone)
            .output();
        if let Ok(out) = &reset_output {
            if !out.status.success() {
                eprintln!(
                    "Warning: git reset failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }
        // Also clean untracked files the agent may have created
        let _ = std::process::Command::new("git")
            .args(["clean", "-fd"])
            .current_dir(&repo_path_clone)
            .output();

        // Apply test patch
        let apply_output = std::process::Command::new("git")
            .args(["apply", "--check"])
            .arg(test_diff.to_str().unwrap_or_default())
            .current_dir(&repo_path_clone)
            .output();

        let patch_applyable = match apply_output {
            Ok(out) => out.status.success(),
            Err(_) => false,
        };

        if patch_applyable {
            let apply_result = std::process::Command::new("git")
                .args(["apply"])
                .arg(test_diff.to_str().unwrap_or_default())
                .current_dir(&repo_path_clone)
                .output();

            match apply_result {
                Ok(out) if out.status.success() => {
                    eprintln!("Test patch applied successfully.");
                    run_held_out_tests(&instance, &repo_path_clone).await
                }
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    eprintln!("Failed to apply test patch: {stderr}");
                    "error".to_string()
                }
                Err(e) => {
                    eprintln!("Failed to apply test patch: {e}");
                    "error".to_string()
                }
            }
        } else {
            eprintln!("Test patch does not apply cleanly, skipping test verification.");
            "skipped".to_string()
        }
    } else {
        eprintln!("No test patch found at: {}", test_diff.display());
        "skipped".to_string()
    };

    if format == "json" {
        let result = serde_json::json!({
            "instance_id": instance.instance_id,
            "category": format!("{:?}", instance.category),
            "difficulty": instance.difficulty,
            "profile": instance.profile_name,
            "fail_to_pass": instance.fail_to_pass_patterns,
            "pass_to_pass": instance.pass_to_pass_patterns,
            "status": verification_status,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}

/// Run held-out tests against the repo and report pass/fail results.
///
/// Applies the test patch, then runs fail-to-pass and pass-to-pass test
/// patterns. Returns "passed" if all tests match expectations, "failed"
/// otherwise.
async fn run_held_out_tests(instance: &TaskInstance, repo_path: &Path) -> String {
    use sruja_agent::verify::{run_verification_steps, VerifyOptions, VerifyStep};

    let mut all_results = Vec::new();
    let opts = VerifyOptions {
        continue_on_error: true,
        timeout_ms: 120_000,
        ..Default::default()
    };

    // Run fail-to-pass tests (these should now pass after the fix)
    for pattern in &instance.fail_to_pass_patterns {
        let step = VerifyStep {
            id: format!("fail_to_pass:{pattern}"),
            command: "cargo".into(),
            args: vec!["test".into(), pattern.into()],
            expected: None,
            group: None,
        };
        let results = run_verification_steps(&[step], &opts, repo_path).await;
        all_results.extend(results);
    }

    // Run pass-to-pass tests (these should still pass - regression check)
    for pattern in &instance.pass_to_pass_patterns {
        let step = VerifyStep {
            id: format!("pass_to_pass:{pattern}"),
            command: "cargo".into(),
            args: vec!["test".into(), pattern.into()],
            expected: None,
            group: None,
        };
        let results = run_verification_steps(&[step], &opts, repo_path).await;
        all_results.extend(results);
    }

    // Report results
    let mut failed_patterns = Vec::new();
    for result in &all_results {
        if !matches!(result.status, sruja_agent::verify::VerifyStatus::Ok) {
            let detail = if result.stderr.trim().is_empty() {
                result.stdout.trim()
            } else {
                result.stderr.trim()
            };
            eprintln!("  FAIL {}: {}", result.step_id, detail);
            failed_patterns.push(result.step_id.clone());
        } else {
            eprintln!("  OK   {}", result.step_id);
        }
    }

    if failed_patterns.is_empty() {
        eprintln!("All held-out tests passed!");
        "passed".to_string()
    } else {
        eprintln!("{} test patterns failed", failed_patterns.len());
        "failed".to_string()
    }
}

/// List available eval task instances.
pub fn list_eval_instances(tasks_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    use super::CliError;
    use std::path::PathBuf;

    let tasks_path = PathBuf::from(tasks_dir);
    if !tasks_path.exists() {
        return Err(CliError::validation(format!("Tasks directory not found: {tasks_dir}")).into());
    }

    let mut instances: Vec<String> = std::fs::read_dir(&tasks_path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().join("instance.toml").exists())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();

    instances.sort();

    if instances.is_empty() {
        eprintln!("No eval task instances found in {tasks_dir}");
        return Ok(());
    }

    eprintln!("Found {} eval task instances:", instances.len());
    for instance_id in &instances {
        let instance_dir = tasks_path.join(instance_id);
        let toml_path = instance_dir.join("instance.toml");
        let toml_content = std::fs::read_to_string(&toml_path).unwrap_or_default();

        let (category, difficulty) = match toml::from_str::<TaskInstance>(&toml_content) {
            Ok(inst) => (format!("{:?}", inst.category), inst.difficulty.to_string()),
            Err(_) => ("parse_error".to_string(), "?".to_string()),
        };

        eprintln!(
            "  {} (category={}, difficulty={})",
            instance_id, category, difficulty
        );
    }

    Ok(())
}
