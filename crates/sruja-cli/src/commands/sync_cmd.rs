//! Sync command: refresh evidence (discover → .sruja/context.json) and run drift.

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use super::discover::discover_context_json_from_graph;
use super::violation_shared::*;
use super::CliError;
use crate::utils::{architecture_path, colors};
use sruja_scan::scan_repo;

pub(crate) struct RepoWriteLock {
    path: std::path::PathBuf,
}

impl Drop for RepoWriteLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub(crate) async fn acquire_repo_write_lock(repo_path: &Path) -> Result<RepoWriteLock, CliError> {
    let lock_path = repo_path.join(".sruja").join("write.lock");
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let started = SystemTime::now();
    let stale_after = Duration::from_secs(10 * 60);
    let timeout = Duration::from_secs(30);
    let poll = Duration::from_millis(100);

    loop {
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(mut f) => {
                use std::io::Write;
                let pid = std::process::id();
                let now = chrono::Utc::now().to_rfc3339();
                let _ = writeln!(f, "pid={pid}\nstarted_at={now}");
                let _ = f.sync_all();
                return Ok(RepoWriteLock { path: lock_path });
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if let Ok(meta) = std::fs::metadata(&lock_path) {
                    if let Ok(modified) = meta.modified() {
                        if SystemTime::now()
                            .duration_since(modified)
                            .unwrap_or_default()
                            > stale_after
                        {
                            let _ = std::fs::remove_file(&lock_path);
                            continue;
                        }
                    }
                }

                if started.elapsed().unwrap_or_default() > timeout {
                    return Err(CliError::validation(format!(
                        "Timed out waiting for Sruja write lock: {}",
                        lock_path.display()
                    )));
                }
                tokio::time::sleep(poll).await;
            }
            Err(e) => return Err(CliError::Io(e)),
        }
    }
}

pub(crate) fn atomic_write_file(path: &Path, contents: &[u8]) -> Result<(), CliError> {
    use std::io::Write;

    let parent = path.parent().ok_or_else(|| {
        CliError::validation(format!(
            "Invalid path (no parent directory): {}",
            path.display()
        ))
    })?;
    fs::create_dir_all(parent)?;

    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("file");
    let unique = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let tmp_path = parent.join(format!(
        ".{file_name}.tmp.{}.{}",
        std::process::id(),
        unique
    ));

    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp_path)?;
    f.write_all(contents)?;
    f.sync_all()?;

    #[cfg(windows)]
    {
        let _ = std::fs::remove_file(path);
    }

    std::fs::rename(&tmp_path, path)?;
    Ok(())
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SyncOutput {
    truth_status: String,
    baseline: Option<String>,
    violations_count: usize,
    health_score: Option<u8>,
    context_path: String,
}

fn iso8601_now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct ScoreHistoryEntry {
    pub date: String,
    pub score: u8,
    pub commit: Option<String>,
}

/// Append a score history entry to .sruja/score_history.jsonl
fn append_score_history(
    repo_path: &Path,
    score: u8,
    commit: Option<String>,
) -> Result<(), CliError> {
    let path = repo_path.join(".sruja/score_history.jsonl");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let entry = ScoreHistoryEntry {
        date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        score,
        commit,
    };

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    use std::io::Write;
    writeln!(file, "{}", serde_json::to_string(&entry)?)?;

    // Keep only last 90 entries
    trim_score_history(&path, 90)?;

    Ok(())
}

fn trim_score_history(path: &Path, max_count: usize) -> Result<(), CliError> {
    if !path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();

    if lines.len() <= max_count {
        return Ok(());
    }

    let keep = &lines[lines.len() - max_count..];
    std::fs::write(path, keep.join("\n") + "\n")?;

    Ok(())
}

/// Check if the cached context is fresh: git commit in context.json matches current HEAD.
/// If true, sync can be skipped to avoid expensive re-scans.
pub fn is_context_fresh(repo_path: &Path) -> bool {
    let context_path = repo_path.join(".sruja/context.json");
    let scan_cache_path = repo_path.join(".sruja/cache/scan.json");

    // Both context.json and scan cache must exist for freshness.
    if !context_path.exists() || !scan_cache_path.exists() {
        return false;
    }

    let content = match fs::read_to_string(&context_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let ctx: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let cached_commit = match ctx.get("git_commit").and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return false,
    };

    // If no current HEAD available, cannot verify freshness.
    let current_commit = match crate::commands::git_commit_short(repo_path) {
        Some(c) => c,
        None => return false,
    };

    current_commit == cached_commit
}

/// Context.json schema version for machine consumers.
const CONTEXT_SCHEMA_VERSION: u32 = 1;

/// Refresh evidence and drift: write .sruja/context.json (with timestamp, git_commit, baseline_path, truth_status), then run drift.
pub async fn sync(repo_root: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let dot_sruja = repo_path.join(".sruja");
    if !dot_sruja.exists() {
        fs::create_dir_all(&dot_sruja).map_err(|e| {
            CliError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to create {}: {}", dot_sruja.display(), e),
            ))
        })?;
    }

    // Scan once and reuse for context + drift/baseline-compare + explain/evidence to avoid redundant work.
    let graph = scan_repo(repo_path).map_err(|e| CliError::scan(e.to_string()))?;
    let mut value =
        super::discover::discover_explanation_value_from_graph(repo_root, repo_path, &graph)
            .or_else(|_| {
                let ctx = discover_context_json_from_graph(repo_root, repo_path, &graph)?;
                serde_json::to_value(&ctx).map_err(|e| CliError::validation(e.to_string()))
            })?;

    let baseline_path = architecture_path::resolve_architecture_path(repo_path);
    let baseline = baseline_path
        .as_ref()
        .and_then(|p| p.to_str().map(String::from));

    // Use shared analysis pipeline for the compare step, reusing the graph from scan.
    let analysis = crate::commands::analysis::run_analysis_with_graph(
        repo_path,
        &crate::commands::analysis::AnalysisOptions::default(),
        Some(graph.clone()),
    )?;
    let truth_status = &analysis.truth_status;
    let health_score = analysis.health_score;

    // Write versioned context.json with evidence + truth state (plan: updated_at, git_commit, baseline_path, truth_status).
    value["updated_at"] = serde_json::Value::String(iso8601_now());
    value["schema_version"] = serde_json::Value::Number(CONTEXT_SCHEMA_VERSION.into());
    value["truth_status"] = serde_json::Value::String(truth_status.clone());
    value["baseline_path"] = baseline
        .as_ref()
        .map(|s| serde_json::Value::String(s.clone()))
        .unwrap_or(serde_json::Value::Null);
    let git_commit = crate::commands::git_commit_short(repo_path);
    value["git_commit"] = git_commit
        .as_ref()
        .map(|s: &String| serde_json::Value::String(s.clone()))
        .unwrap_or(serde_json::Value::Null);

    // Use violations from shared analysis pipeline.
    let active_summ: Vec<ViolationSummary> = analysis
        .active_violations
        .iter()
        .map(summarize_violation)
        .collect();
    let suppressed_summ: Vec<ViolationSummary> = analysis
        .suppressed_violations
        .iter()
        .map(summarize_violation)
        .collect();

    value["violations"] =
        serde_json::to_value(&active_summ).map_err(|e| CliError::validation(e.to_string()))?;
    value["suppressed_violations"] =
        serde_json::to_value(&suppressed_summ).map_err(|e| CliError::validation(e.to_string()))?;
    value["suppressed_count"] = serde_json::Value::Number((suppressed_summ.len() as u64).into());

    let path = dot_sruja.join("context.json");
    let context_path = path.display().to_string();
    let cache_dir = dot_sruja.join("cache");
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)?;
    }
    let graph_path = cache_dir.join("scan.json");
    let author_evidence_path = dot_sruja.join("author_evidence.json");
    let _lock = acquire_repo_write_lock(repo_path).await?;

    let context_json =
        serde_json::to_string_pretty(&value).map_err(|e| CliError::validation(e.to_string()))?;
    atomic_write_file(&path, context_json.as_bytes()).map_err(|e| match e {
        CliError::Io(io) => CliError::Io(std::io::Error::new(
            io.kind(),
            format!("Failed to write {}: {}", path.display(), io),
        )),
        other => other,
    })?;

    let graph_json = {
        let cache = crate::commands::ScanCache {
            git_commit: git_commit.clone().unwrap_or_default(),
            graph: graph.clone(),
        };
        serde_json::to_string(&cache).map_err(|e| CliError::validation(e.to_string()))?
    };
    atomic_write_file(&graph_path, graph_json.as_bytes()).map_err(|e| match e {
        CliError::Io(io) => CliError::Io(std::io::Error::new(
            io.kind(),
            format!("Failed to write {}: {}", graph_path.display(), io),
        )),
        other => other,
    })?;

    let author_evidence = super::author::build_author_evidence_from_graph(
        repo_root,
        repo_path,
        &graph,
        truth_status,
        git_commit,
    )?;
    let author_evidence_json = serde_json::to_string_pretty(&author_evidence)
        .map_err(|e| CliError::validation(e.to_string()))?;
    atomic_write_file(&author_evidence_path, author_evidence_json.as_bytes()).map_err(
        |e| match e {
            CliError::Io(io) => CliError::Io(std::io::Error::new(
                io.kind(),
                format!("Failed to write {}: {}", author_evidence_path.display(), io),
            )),
            other => other,
        },
    )?;

    let output = SyncOutput {
        truth_status: truth_status.clone(),
        baseline: baseline.clone(),
        violations_count: active_summ.len(),
        health_score: Some(health_score),
        context_path: context_path.clone(),
    };

    // Append score history if health_score is available
    if let Err(e) = append_score_history(
        repo_path,
        health_score,
        crate::commands::git_commit_short(repo_path),
    ) {
        eprintln!("Warning: Failed to save score history: {}", e);
    }

    match format {
        "json" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&output)
                    .map_err(|e| CliError::validation(e.to_string()))?
            );
        }
        "quiet" => {}
        _ => {
            eprintln!("Wrote {}", colors::info(context_path));
            eprintln!("Wrote {}", colors::info(graph_path.display()));
            eprintln!("Wrote {}", colors::info(author_evidence_path.display()));
            if let Some(ref base) = baseline {
                eprintln!("Baseline: {}", base);
            } else {
                eprintln!("{}", colors::warning("No baseline (repo.sruja not found)"));
            }

            let status_color = match truth_status.as_str() {
                "reviewed" => colors::success(&truth_status),
                "drifted" => colors::error(&truth_status),
                _ => colors::warning(&truth_status),
            };
            eprintln!(
                "Truth: {} ({} violation(s))",
                status_color,
                active_summ.len()
            );

            eprintln!("Health score: {}", colors::health_bar(health_score, 20));

            if !active_summ.is_empty() {
                eprintln!();
                eprintln!("{}", colors::style("Violations:").bold());
                for v in &active_summ {
                    eprintln!(
                        "  {} {}: {} {}",
                        colors::severity_icon(&v.severity),
                        colors::style(&v.kind).bold(),
                        v.message,
                        colors::dim(v.location.as_deref().unwrap_or(""))
                    );
                }
            }
        }
    }

    Ok(())
}
