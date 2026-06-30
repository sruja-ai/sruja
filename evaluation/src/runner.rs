//! Core task runner — orchestrates eval tasks with git worktree isolation,
//! memory lifecycle management, and structured result capture.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// Whether to clear memory before running (baseline) or accumulate learnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    /// Clear agent memory before each task (establish baseline with no learned context)
    Baseline,
    /// Keep memory across tasks so learnings from earlier tasks feed later ones
    WithMemory,
}

/// Options for `run_tasks`.
pub struct RunOptions {
    pub mode: RunMode,
    pub sruja_bin: PathBuf,
    pub repo_root: PathBuf,
    pub tag: Option<String>,
    pub max_iterations: usize,
    pub task_filter: Vec<String>,
    pub dry_run: bool,
}

/// Options for `retry_failed`.
pub struct RetryOptions {
    pub run_id: Option<String>,
    pub sruja_bin: PathBuf,
    pub repo_root: PathBuf,
    pub tag: Option<String>,
    pub max_iterations: usize,
    pub dry_run: bool,
}

/// Per-task result captured from the eval run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub instance_id: String,
    pub category: String,
    pub difficulty: u8,
    pub passed: bool,
    pub status: String, // "passed", "failed", "skipped", "error"
    pub duration_ms: u64,
    pub failure_reason: Option<String>,
    pub error_class: Option<String>,
    pub iterations: usize,
    pub converge_iterations: Option<usize>,
}

/// Top-level result for a run (aggregated across all tasks).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub run_id: String,
    pub tag: String,
    pub timestamp: String,
    pub mode: String,
    pub max_iterations: usize,
    pub task_filter: Vec<String>,
    pub total_tasks: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub error_count: usize,
    pub pass_rate: f64,
    pub total_duration_ms: u64,
    pub tasks: Vec<TaskResult>,
}

/// Parsed task instance metadata from instance.toml.
#[derive(Debug, Clone, Deserialize)]
struct TaskInstanceMeta {
    instance_id: String,
    category: String,
    difficulty: u8,
    #[allow(dead_code)]
    profile: String,
    #[allow(dead_code)]
    base_commit: String,
    #[serde(default)]
    fail_to_pass: Vec<String>,
    #[serde(default)]
    pass_to_pass: Vec<String>,
}

/// Generate a unique run ID.
fn generate_run_id() -> String {
    format!("run_{}", Utc::now().format("%Y%m%d_%H%M%S"))
}

/// Find the sruja binary path.
fn resolve_sruja_bin(custom: &Path) -> PathBuf {
    if custom.exists() {
        return custom.to_path_buf();
    }

    // Check a few likely locations
    let candidates = [
        custom.to_path_buf(),
        PathBuf::from("target/release/sruja"),
        PathBuf::from("target/debug/sruja"),
    ];

    // Check PATH first via `which` on Unix
    if let Ok(output) = Command::new("which").arg("sruja").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return PathBuf::from(path);
            }
        }
    }

    // Fall back to candidates
    for c in &candidates {
        if c.exists() {
            return c.to_path_buf();
        }
    }

    custom.to_path_buf() // return original even if not found (will error at run time)
}

/// Load all task instances from the evaluation/tasks directory.
fn load_task_instances(tasks_dir: &Path) -> Result<Vec<TaskInstanceMeta>, String> {
    if !tasks_dir.exists() {
        return Err(format!(
            "Tasks directory not found: {}",
            tasks_dir.display()
        ));
    }

    let mut instances = Vec::new();
    let entries =
        std::fs::read_dir(tasks_dir).map_err(|e| format!("Failed to read tasks directory: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
        let task_dir = entry.path();
        if !task_dir.is_dir() {
            continue;
        }

        let instance_path = task_dir.join("instance.toml");
        if !instance_path.exists() {
            continue;
        }

        let content = std::fs::read_to_string(&instance_path)
            .map_err(|e| format!("Failed to read {}: {e}", instance_path.display()))?;
        let meta: TaskInstanceMeta = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {e}", instance_path.display()))?;

        instances.push(meta);
    }

    // Sort by instance_id for deterministic ordering
    instances.sort_by(|a, b| a.instance_id.cmp(&b.instance_id));

    Ok(instances)
}

/// Create a git worktree at the specified base commit.
fn create_worktree(repo_root: &Path, task_id: &str, base_commit: &str) -> Result<PathBuf, String> {
    let worktree_dir = repo_root
        .join("target")
        .join("eval-worktrees")
        .join(task_id);

    // Clean up any existing worktree first
    let _ = Command::new("git")
        .args(["worktree", "remove", "--force"])
        .arg(&worktree_dir)
        .current_dir(repo_root)
        .output();

    let _ = std::fs::remove_dir_all(&worktree_dir);

    // Create parent directory
    std::fs::create_dir_all(worktree_dir.parent().unwrap())
        .map_err(|e| format!("Failed to create worktree parent: {e}"))?;

    // Create the worktree
    let output = Command::new("git")
        .args(["worktree", "add", "--force"])
        .arg(&worktree_dir)
        .arg(base_commit)
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("Failed to create worktree: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git worktree add failed: {stderr}"));
    }

    Ok(worktree_dir)
}

/// Remove a git worktree.
fn remove_worktree(repo_root: &Path, worktree_dir: &Path) {
    let _ = Command::new("git")
        .args(["worktree", "remove", "--force"])
        .arg(worktree_dir)
        .current_dir(repo_root)
        .output();
    let _ = std::fs::remove_dir_all(worktree_dir);
}

/// Get the base_commit for a task instance from its instance.toml.
fn get_base_commit(tasks_dir: &Path, instance_id: &str) -> Result<String, String> {
    let instance_path = tasks_dir.join(instance_id).join("instance.toml");
    let content =
        std::fs::read_to_string(&instance_path).map_err(|e| format!("Failed to read: {e}"))?;
    let meta: TaskInstanceMeta =
        toml::from_str(&content).map_err(|e| format!("Failed to parse: {e}"))?;
    Ok(meta.base_commit)
}

/// Classify a failure reason into an error class using simple keyword matching.
fn classify_failure(_status: &str, stderr: &str) -> (String, String) {
    let lower = stderr.to_lowercase();

    let error_class = if lower.contains("error[e0")
        || lower.contains("compilation")
        || lower.contains("does not compile")
    {
        "compilation"
    } else if lower.contains("mismatched types")
        || lower.contains("trait bound")
        || lower.contains("lifetime")
    {
        "type"
    } else if lower.contains("test result: failed")
        || lower.contains("assertion failed")
        || lower.contains("test ... failed")
    {
        "test"
    } else if lower.contains("panicked")
        || lower.contains("unwrap on none")
        || lower.contains("index out of bounds")
    {
        "runtime"
    } else if lower.contains("boundary")
        || lower.contains("drift")
        || lower.contains("architecture violation")
    {
        "architecture"
    } else if lower.contains("not addressed")
        || lower.contains("incomplete")
        || lower.contains("missing criterion")
    {
        "spec_gap"
    } else if lower.contains("warning") || lower.contains("lint") || lower.contains("clippy") {
        "lint"
    } else if lower.contains("timeout")
        || lower.contains("rate limit")
        || lower.contains("llm error")
    {
        "infrastructure"
    } else {
        "other"
    };

    let reason = if lower.len() > 200 {
        format!("{}...", &lower[..200])
    } else {
        stderr.to_string()
    };

    (error_class.to_string(), reason)
}

/// Build a LearningEntry from a failed task result.
fn build_learning_from_failure(
    task: &TaskResult,
    _task_dir: &Path,
    run_id: &str,
) -> sruja_agent::LearningEntry {
    let hypothesis = if let Some(ref cls) = task.error_class {
        format!(
            "Task '{}' ({} difficulty {}) failed due to {}. {}",
            task.instance_id,
            task.category,
            task.difficulty,
            cls,
            task.failure_reason.as_deref().unwrap_or("unknown reason")
        )
    } else {
        format!(
            "Task '{}' ({} difficulty {}) failed.",
            task.instance_id, task.category, task.difficulty
        )
    };

    let guardrail = if let Some(ref cls) = task.error_class {
        match cls.as_str() {
            "compilation" => "Before running the agent for a coding task, run `cargo check` to establish the baseline compilation state. This prevents compilation errors from being misattributed to agent changes.",
            "test" => "After implementing changes, always run `cargo test` to verify. If tests fail, check the test output for assertion details before replanning.",
            "type" => "When the agent encounters type errors, check type annotations and trait bounds first. These are often caused by incomplete struct definitions or missing trait implementations.",
            "runtime" => "Check for unwrap() calls and index access without bounds checking. Replace with proper error handling before running tests.",
            "architecture" => "Run `sruja drift -r .` before and after changes to verify no architecture boundary violations were introduced.",
            "spec_gap" => "Review the acceptance criteria from the problem statement. Ensure all criteria are addressed before considering the task complete.",
            "lint" => "Run `cargo clippy -- -D warnings` after implementation to catch style issues before verification.",
            _ => "When a task fails, review the agent trajectory in .sruja/runs/<run_id>/loop.json for detailed failure analysis.",
        }
        .to_string()
    } else {
        "Review the agent trajectory for detailed failure analysis.".to_string()
    };

    sruja_agent::LearningEntry::guardrail(
        format!(
            "eval task: {} (category={} difficulty={})",
            task.instance_id, task.category, task.difficulty
        ),
        hypothesis,
        guardrail,
    )
    .with_run_id(run_id)
    .with_repo(".")
    .with_selector(task.instance_id.clone())
    .with_confidence(if task.difficulty <= 2 {
        "high"
    } else if task.difficulty <= 3 {
        "medium"
    } else {
        "low"
    })
    .with_tags(vec![
        "eval".to_string(),
        task.category.clone(),
        format!("difficulty_{}", task.difficulty),
        task.error_class
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
    ])
}

/// Build a LearningEntry from a successful task result.
fn build_learning_from_success(task: &TaskResult, run_id: &str) -> sruja_agent::LearningEntry {
    sruja_agent::LearningEntry::playbook(
        format!(
            "eval task: {} (category={} difficulty={})",
            task.instance_id, task.category, task.difficulty
        ),
        format!(
            "Task '{}' passed with {} iterations. Current routing approach is effective for this task category.",
            task.instance_id,
            task.iterations
        ),
        format!(
            "For {} tasks at difficulty {}, the current agent configuration works. Use the standard comprehension -> plan -> execute -> critique -> reflect pipeline.",
            task.category, task.difficulty
        ),
    )
    .with_run_id(run_id)
    .with_repo(".")
    .with_selector(task.instance_id.clone())
    .with_confidence("medium")
}

/// Run all eval tasks.
pub async fn run_tasks(options: &RunOptions) -> Result<(), Box<dyn std::error::Error>> {
    let sruja_bin = resolve_sruja_bin(&options.sruja_bin);
    let tasks_dir = options.repo_root.join("evaluation").join("tasks");
    let results_dir = options.repo_root.join("evaluation").join("results");

    let all_instances = load_task_instances(&tasks_dir)?;
    let instances: Vec<&TaskInstanceMeta> = if options.task_filter.is_empty() {
        all_instances.iter().collect()
    } else {
        all_instances
            .iter()
            .filter(|i| options.task_filter.contains(&i.instance_id))
            .collect()
    };

    if instances.is_empty() {
        eprintln!("No tasks to run.");
        return Ok(());
    }

    let run_id = generate_run_id();
    let tag = options.tag.clone().unwrap_or_else(|| run_id.clone());
    let mode_str = match options.mode {
        RunMode::Baseline => "baseline",
        RunMode::WithMemory => "with-memory",
    };

    eprintln!("═══ Eval Runner ═══");
    eprintln!("  Run ID: {run_id}");
    eprintln!("  Tag: {tag}");
    eprintln!("  Mode: {mode_str}");
    eprintln!("  Sruja binary: {}", sruja_bin.display());
    eprintln!(
        "  Tasks: {} (filtered from {})",
        instances.len(),
        all_instances.len()
    );
    eprintln!("  Max iterations: {}", options.max_iterations);
    eprintln!();

    // Handle memory lifecycle
    let memory_path = sruja_agent::AgenticMemory::get_path(&options.repo_root);
    match options.mode {
        RunMode::Baseline => {
            eprintln!("[memory] Clearing agent memory for baseline run...");
            eprintln!("  (backup saved at {}.bak)", memory_path.display());
            if memory_path.exists() {
                let backup = format!("{}.bak", memory_path.display());
                let _ = std::fs::copy(&memory_path, &backup);
            }
            let _ = sruja_agent::AgenticMemory::clear(&options.repo_root);
        }
        RunMode::WithMemory => {
            if memory_path.exists() {
                let count = sruja_agent::AgenticMemory::load(&options.repo_root)
                    .map(|m| m.learnings.len())
                    .unwrap_or(0);
                eprintln!("[memory] Accumulating learnings ({count} existing entries)");
            } else {
                eprintln!("[memory] No existing memory — starting fresh");
            }
        }
    }
    eprintln!();

    let start = Instant::now();
    let mut tasks: Vec<TaskResult> = Vec::new();

    for instance in &instances {
        eprintln!("─── Task: {} ───", instance.instance_id);
        eprintln!(
            "  Category: {} | Difficulty: {}",
            instance.category, instance.difficulty
        );
        eprintln!("  Fail-to-pass: {:?}", instance.fail_to_pass);
        eprintln!("  Pass-to-pass: {:?}", instance.pass_to_pass);

        if options.dry_run {
            eprintln!("  DRY RUN — skipping");
            tasks.push(TaskResult {
                instance_id: instance.instance_id.clone(),
                category: instance.category.clone(),
                difficulty: instance.difficulty,
                passed: false,
                status: "skipped".to_string(),
                duration_ms: 0,
                failure_reason: Some("dry run".to_string()),
                error_class: None,
                iterations: 0,
                converge_iterations: None,
            });
            continue;
        }

        // Get base commit from instance.toml
        let base_commit = match get_base_commit(&tasks_dir, &instance.instance_id) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("  ERROR: {e}");
                tasks.push(TaskResult {
                    instance_id: instance.instance_id.clone(),
                    category: instance.category.clone(),
                    difficulty: instance.difficulty,
                    passed: false,
                    status: "error".to_string(),
                    duration_ms: 0,
                    failure_reason: Some(format!("Failed to get base commit: {e}")),
                    error_class: Some("infrastructure".to_string()),
                    iterations: 0,
                    converge_iterations: None,
                });
                continue;
            }
        };

        // Create worktree
        let worktree =
            match create_worktree(&options.repo_root, &instance.instance_id, &base_commit) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("  ERROR: {e}");
                    tasks.push(TaskResult {
                        instance_id: instance.instance_id.clone(),
                        category: instance.category.clone(),
                        difficulty: instance.difficulty,
                        passed: false,
                        status: "error".to_string(),
                        duration_ms: 0,
                        failure_reason: Some(format!("Worktree creation failed: {e}")),
                        error_class: Some("infrastructure".to_string()),
                        iterations: 0,
                        converge_iterations: None,
                    });
                    continue;
                }
            };

        let task_start = Instant::now();

        // Copy agent_memory.json to worktree if using memory
        if options.mode == RunMode::WithMemory && memory_path.exists() {
            let worktree_memory_dir = worktree.join(".sruja");
            let _ = std::fs::create_dir_all(&worktree_memory_dir);
            let _ = std::fs::copy(&memory_path, worktree_memory_dir.join("agent_memory.json"));
        }

        // Run the sruja eval command
        let output = Command::new(&sruja_bin)
            .args(["eval", "run", "--instance", &instance.instance_id, "--repo"])
            .arg(&worktree)
            .args([
                "--max-iterations",
                &options.max_iterations.to_string(),
                "--format",
                "json",
            ])
            .output()
            .map_err(|e| format!("Failed to run sruja eval: {e}"))?;

        let duration_ms = task_start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Try to parse the JSON result from stdout
        let (passed, status, failure_reason) =
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                let s = json
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let is_pass = s == "passed";
                (is_pass, s, None)
            } else {
                // Fall back to exit code + stderr analysis
                let is_pass = output.status.success();
                let stderr_trimmed = stderr.trim();
                let reason = if stderr_trimmed.is_empty() {
                    None
                } else {
                    Some(stderr_trimmed.to_string())
                };
                (
                    is_pass,
                    if is_pass { "passed" } else { "failed" }.to_string(),
                    reason,
                )
            };

        let (error_class, failure_reason) = if passed {
            (None, failure_reason)
        } else {
            let (cls, reason) = classify_failure(&status, &stderr);
            (Some(cls), Some(reason))
        };

        let task_result = TaskResult {
            instance_id: instance.instance_id.clone(),
            category: instance.category.clone(),
            difficulty: instance.difficulty,
            passed,
            status: status.clone(),
            duration_ms,
            failure_reason,
            error_class,
            iterations: options.max_iterations,
            converge_iterations: None,
        };

        eprintln!(
            "  Result: {} ({})",
            if passed { "PASS" } else { "FAIL" },
            status
        );
        eprintln!("  Duration: {}ms", duration_ms);

        // If in with-memory mode, extract learnings from this task and persist
        if options.mode == RunMode::WithMemory {
            let mut memory =
                sruja_agent::AgenticMemory::load(&options.repo_root).unwrap_or_default();

            let learning = if task_result.passed {
                build_learning_from_success(&task_result, &run_id)
            } else {
                build_learning_from_failure(&task_result, &tasks_dir, &run_id)
            };
            memory.add_learning(learning);
            if let Err(e) = memory.save(&options.repo_root) {
                eprintln!("  Warning: failed to save memory: {e}");
            }
        }

        // Move task_result into tasks Vec
        tasks.push(task_result);

        // Clean up worktree
        remove_worktree(&options.repo_root, &worktree);
        eprintln!();
    }

    // Build aggregate result
    let total_duration_ms = start.elapsed().as_millis() as u64;
    let passed_count = tasks.iter().filter(|t| t.passed).count();
    let failed_count = tasks
        .iter()
        .filter(|t| !t.passed && t.status == "failed")
        .count();
    let skipped_count = tasks.iter().filter(|t| t.status == "skipped").count();
    let error_count = tasks.iter().filter(|t| t.status == "error").count();
    let pass_rate = if tasks.is_empty() {
        0.0
    } else {
        passed_count as f64 / tasks.len() as f64 * 100.0
    };

    let run_result = RunResult {
        run_id: run_id.clone(),
        tag: tag.clone(),
        timestamp: Utc::now().to_rfc3339(),
        mode: mode_str.to_string(),
        max_iterations: options.max_iterations,
        task_filter: options.task_filter.clone(),
        total_tasks: tasks.len(),
        passed: passed_count,
        failed: failed_count,
        skipped: skipped_count,
        error_count,
        pass_rate,
        total_duration_ms,
        tasks,
    };

    // Save results
    let run_dir = results_dir.join(&run_id);
    std::fs::create_dir_all(&run_dir).map_err(|e| format!("Failed to create results dir: {e}"))?;

    let result_path = run_dir.join("results.json");
    let result_json = serde_json::to_string_pretty(&run_result)
        .map_err(|e| format!("Failed to serialize results: {e}"))?;
    std::fs::write(&result_path, &result_json)
        .map_err(|e| format!("Failed to write results: {e}"))?;

    // Also write a tag symlink or reference file
    let tag_path = results_dir.join(format!("{}.tag", tag));
    std::fs::write(&tag_path, &run_id)
        .map_err(|e| format!("Failed to write tag reference: {e}"))?;

    // Print summary
    eprintln!("═══ Run Complete ═══");
    eprintln!("  Run ID: {run_id}");
    eprintln!("  Tag: {tag}");
    eprintln!("  Mode: {mode_str}");
    eprintln!(
        "  Pass rate: {pass_rate:.1}% ({passed_count}/{})",
        run_result.total_tasks
    );
    eprintln!("  Failed: {failed_count}");
    eprintln!("  Skipped: {skipped_count}");
    eprintln!("  Errors: {error_count}");
    eprintln!("  Total duration: {}ms", total_duration_ms);
    eprintln!();
    eprintln!("  Results saved to: {}", result_path.display());

    // Print JSON results to stdout for piping
    println!("{result_json}");

    Ok(())
}

/// Retry only failed tasks from a previous run.
pub async fn retry_failed(options: &RetryOptions) -> Result<(), Box<dyn std::error::Error>> {
    let results_dir = options.repo_root.join("evaluation").join("results");

    // Find the run directory
    let run_dir = if let Some(ref run_id) = options.run_id {
        results_dir.join(run_id)
    } else {
        // Find latest run
        let mut entries: Vec<_> = std::fs::read_dir(&results_dir)
            .map_err(|e| format!("Failed to read results dir: {e}"))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().join("results.json").exists())
            .collect();
        entries.sort_by_key(|e| e.path().metadata().ok().and_then(|m| m.modified().ok()));
        match entries.last() {
            Some(e) => e.path(),
            None => {
                eprintln!("No previous run found in {}", results_dir.display());
                return Ok(());
            }
        }
    };

    let result_path = run_dir.join("results.json");
    let content = std::fs::read_to_string(&result_path)?;
    let prev_result: RunResult = serde_json::from_str(&content)?;

    let failed_tasks: Vec<String> = prev_result
        .tasks
        .iter()
        .filter(|t| !t.passed)
        .map(|t| t.instance_id.clone())
        .collect();

    if failed_tasks.is_empty() {
        eprintln!("No failed tasks to retry.");
        return Ok(());
    }

    eprintln!(
        "Retrying {} failed tasks from run {}",
        failed_tasks.len(),
        prev_result.run_id
    );
    for t in &failed_tasks {
        eprintln!("  - {t}");
    }
    eprintln!();

    // Run only the failed tasks with with-memory mode and higher max iterations
    run_tasks(&RunOptions {
        mode: RunMode::WithMemory,
        sruja_bin: options.sruja_bin.clone(),
        repo_root: options.repo_root.clone(),
        tag: options.tag.clone(),
        max_iterations: options.max_iterations,
        task_filter: failed_tasks,
        dry_run: options.dry_run,
    })
    .await
}
