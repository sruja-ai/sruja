//! Sync command: refresh evidence (discover → .sruja/context.json) and run drift.

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use super::discover::discover_context_json_from_graph;
use super::violation_shared::*;
use super::CliError;
use crate::utils::{architecture_path, colors};
use sruja_diff::Violation;
use sruja_scan::scan_repo;
use std::collections::HashSet;

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

/// Read git HEAD commit (short) if repo is a git work tree; otherwise None.
fn git_commit_short(repo_path: &Path) -> Option<String> {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(repo_path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
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

    let (truth_status, violations, health_score) = if let Some(ref baseline_file) = baseline_path {
        let content = fs::read_to_string(baseline_file)?;
        let parser = sruja_language::Parser::new(baseline_file.to_string_lossy().as_ref());
        let program = parser.parse(&content).map_err(|diags| {
            CliError::parse_with_diagnostics(baseline_file.to_string_lossy().to_string(), diags)
        })?;
        let proposed_graph = sruja_diff::program_to_graph(&program);
        let diff = sruja_diff::compare_graphs(&graph, &proposed_graph);
        let truth = match diff.truth_status {
            sruja_diff::TruthStatus::Reviewed => "reviewed",
            sruja_diff::TruthStatus::Drifted => "drifted",
            sruja_diff::TruthStatus::Unknown => "unknown",
        };
        (
            truth.to_string(),
            diff.violations,
            Some(diff.summary.health_score),
        )
    } else {
        let drift = sruja_diff::detect_architectural_drift(&graph);
        (
            "unknown".to_string(),
            drift.violations,
            Some(drift.health_score),
        )
    };

    // Write versioned context.json with evidence + truth state (plan: updated_at, git_commit, baseline_path, truth_status).
    value["updated_at"] = serde_json::Value::String(iso8601_now());
    value["schema_version"] = serde_json::Value::Number(CONTEXT_SCHEMA_VERSION.into());
    value["truth_status"] = serde_json::Value::String(truth_status.clone());
    value["baseline_path"] = baseline
        .as_ref()
        .map(|s| serde_json::Value::String(s.clone()))
        .unwrap_or(serde_json::Value::Null);
    let git_commit = git_commit_short(repo_path);
    value["git_commit"] = git_commit
        .as_ref()
        .map(|s| serde_json::Value::String(s.clone()))
        .unwrap_or(serde_json::Value::Null);

    // Add normalized violations with shared metadata, split by baseline suppression if baseline file exists.
    let violations: Vec<Violation> = violations
        .into_iter()
        .map(|mut v| {
            v.production_relevant = Some(true);
            if v.evidence_count.is_none() {
                v.evidence_count = Some(v.sources.len());
            }
            v
        })
        .collect();
    let baseline_fp_path = repo_path.join(".sruja").join("violations.baseline.json");
    let baseline_set: Option<HashSet<String>> = if baseline_fp_path.exists() {
        Some(super::violation_shared::load_violations_baseline(&baseline_fp_path)?.fingerprints)
    } else {
        None
    };
    let (active, suppressed): (Vec<Violation>, Vec<Violation>) = if let Some(ref set) = baseline_set
    {
        violations
            .into_iter()
            .map(|mut v| {
                let sup = set.contains(&fingerprint_violation(&v));
                v.suppressed = Some(sup);
                v.baseline_delta = Some(if sup { "baseline" } else { "new" }.to_string());
                v
            })
            .partition(|v| v.suppressed != Some(true))
    } else {
        (violations, Vec::new())
    };

    let active_summ: Vec<ViolationSummary> = active.iter().map(summarize_violation).collect();
    let suppressed_summ: Vec<ViolationSummary> =
        suppressed.iter().map(summarize_violation).collect();

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

    let graph_json =
        serde_json::to_string(&graph).map_err(|e| CliError::validation(e.to_string()))?;
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
        &truth_status,
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
        health_score,
        context_path: context_path.clone(),
    };

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

            if let Some(score) = health_score {
                eprintln!("Health score: {}", colors::health_bar(score, 20));
            }

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
