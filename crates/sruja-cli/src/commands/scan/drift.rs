#![allow(clippy::await_holding_lock)]
use std::fs;
use std::path::{Path, PathBuf};

use crate::commands::CliError;
use crate::integrations::load_repo_config;
use crate::utils::architecture_path;
use sruja_scan::scan_repo;
use sruja_scan::scan_scope::resolve_scan_scope;

use super::output::{
    apply_advisory_violation_filter, build_structural_drift_json_envelope, collect_could_not_infer,
    print_diff_text, print_drift_text, print_github_actions_output, print_pr_drift_text,
    print_violations_github_actions, PrDriftResult, PrViolation, StatusOutput,
};

pub(crate) fn should_fail_on_violations(
    fail_on: Option<&str>,
    violations: &[sruja_diff::Violation],
) -> bool {
    if let Some(criteria) = fail_on {
        let criteria_lower = criteria.to_lowercase();
        let criteria_list: Vec<&str> = criteria_lower.split(',').map(|s| s.trim()).collect();

        for criterion in criteria_list {
            match criterion {
                "all"
                    if violations
                        .iter()
                        .any(|v| matches!(v.severity, sruja_diff::Severity::Error)) =>
                {
                    return true;
                }
                "cycles" | "circular"
                    if violations.iter().any(|v| {
                        matches!(v.kind, sruja_diff::ViolationKind::CircularDependency)
                    }) =>
                {
                    return true;
                }
                "layer-violations" | "layer"
                    if violations
                        .iter()
                        .any(|v| matches!(v.kind, sruja_diff::ViolationKind::LayerViolation)) =>
                {
                    return true;
                }
                "god-modules" | "god"
                    if violations
                        .iter()
                        .any(|v| matches!(v.kind, sruja_diff::ViolationKind::GodModule)) =>
                {
                    return true;
                }
                "orphans"
                    if violations
                        .iter()
                        .any(|v| matches!(v.kind, sruja_diff::ViolationKind::OrphanComponent)) =>
                {
                    return true;
                }
                _ => {}
            }
        }
    }
    false
}

pub(crate) fn truth_status_from_baseline_compare(
    scanned: &sruja_scan::Graph,
    baseline_path: &Path,
) -> Result<sruja_diff::TruthStatus, CliError> {
    let content = fs::read_to_string(baseline_path)?;
    let parser = sruja_language::Parser::new(baseline_path.to_string_lossy().as_ref());
    let program = parser.parse(&content).map_err(|diags| {
        CliError::parse_with_diagnostics(baseline_path.to_string_lossy().to_string(), diags)
    })?;
    let proposed_graph = sruja_diff::program_to_graph(&program);
    Ok(sruja_diff::compare_graphs(scanned, &proposed_graph).truth_status)
}

pub struct DriftRequest<'a> {
    pub repo_root: &'a str,
    pub architecture_path: Option<&'a str>,
    pub format: &'a str,
    pub violations_only: bool,
    pub fail_on: Option<&'a str>,
    pub violations_baseline: Option<&'a str>,
    pub baseline_mode: Option<&'a str>,
    pub structural_only: bool,
    pub advisory: bool,
}

pub async fn drift(req: DriftRequest<'_>) -> Result<(), CliError> {
    let repo_path = Path::new(req.repo_root);

    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", req.repo_root),
        )));
    }

    let actual_graph = scan_repo(repo_path)?;
    let baseline_set = if let Some(p) = req.violations_baseline {
        let bp = if Path::new(p).is_absolute() {
            PathBuf::from(p)
        } else {
            repo_path.join(p)
        };
        if bp.exists() {
            Some(crate::commands::violation_shared::load_violations_baseline(&bp)?.fingerprints)
        } else {
            None
        }
    } else {
        None
    };

    let resolved = if req.structural_only {
        None
    } else {
        architecture_path::resolve_architecture_path(repo_path)
    };
    let effective_arch = req
        .architecture_path
        .or_else(|| resolved.as_ref().and_then(|p| p.to_str()));

    if let Some(arch_path) = effective_arch {
        let arch_file = Path::new(arch_path);
        if !arch_file.exists() {
            return Err(CliError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Architecture file not found: {}", arch_path),
            )));
        }
        let content = fs::read_to_string(arch_file)?;
        let parser = sruja_language::Parser::new(arch_path);
        let program = parser
            .parse(&content)
            .map_err(|diags| CliError::parse_with_diagnostics(arch_path.to_string(), diags))?;
        let proposed_graph = sruja_diff::program_to_graph(&program);
        let mode_str = req
            .baseline_mode
            .map(|s| s.to_string())
            .or_else(|| load_repo_config(repo_path).and_then(|c| c.baseline.mode))
            .unwrap_or_else(|| "auto".to_string());
        let mode = match mode_str.to_lowercase().as_str() {
            "overview" => sruja_diff::BaselineMode::Overview,
            "exhaustive" => sruja_diff::BaselineMode::Exhaustive,
            _ => sruja_diff::BaselineMode::Auto,
        };
        let diff_result = sruja_diff::compare_graphs_with_options(
            &actual_graph,
            &proposed_graph,
            sruja_diff::CompareOptions {
                baseline_mode: mode,
            },
        );
        let mut diff_result = diff_result;
        if let Some(ref set) = baseline_set {
            diff_result.violations = diff_result
                .violations
                .into_iter()
                .map(|mut v| {
                    let suppressed = set.contains(
                        &crate::commands::violation_shared::fingerprint_violation(&v),
                    );
                    v.suppressed = Some(suppressed);
                    v.baseline_delta =
                        Some(if suppressed { "baseline" } else { "new" }.to_string());
                    v
                })
                .filter(|v| v.suppressed != Some(true))
                .collect();
        }

        match req.format {
            "drift-state" => {
                println!(
                    "{}",
                    crate::commands::drift_state::build_drift_state_json(
                        req.repo_root,
                        &actual_graph
                    )?
                );
            }
            "json" => {
                let mut value = serde_json::to_value(&diff_result)?;
                value = crate::commands::remediation::wrap_deterministic_json(
                    value,
                    "architecture_drift",
                    "Deterministic comparison of scan evidence vs declared architecture (repo.sruja).",
                );
                println!("{}", serde_json::to_string_pretty(&value)?);
            }
            "github" | "github-actions" => {
                print_violations_github_actions(&diff_result.violations);
            }
            _ => {
                print_diff_text(&diff_result, req.violations_only);
            }
        }

        crate::commands::context_events::record_drift_compare(
            repo_path,
            diff_result.violations.len(),
            &format!("{:?}", diff_result.truth_status),
            true,
        );

        if should_fail_on_violations(req.fail_on, &diff_result.violations) {
            return Err(CliError::FailOnViolations);
        }
    } else {
        let (_, scan_scope) = resolve_scan_scope(repo_path);
        let mut drift_result = sruja_diff::detect_architectural_drift(&actual_graph);
        drift_result.scan_scope = scan_scope;
        if req.advisory {
            apply_advisory_violation_filter(&mut drift_result);
        }
        if let Some(ref set) = baseline_set {
            drift_result.violations = drift_result
                .violations
                .into_iter()
                .map(|mut v| {
                    let suppressed = set.contains(
                        &crate::commands::violation_shared::fingerprint_violation(&v),
                    );
                    v.suppressed = Some(suppressed);
                    v.baseline_delta =
                        Some(if suppressed { "baseline" } else { "new" }.to_string());
                    v
                })
                .filter(|v| v.suppressed != Some(true))
                .collect();
        }
        let could_not_infer = collect_could_not_infer(&actual_graph);

        match req.format {
            "drift-state" => {
                println!(
                    "{}",
                    crate::commands::drift_state::build_drift_state_json(
                        req.repo_root,
                        &actual_graph
                    )?
                );
            }
            "json" => {
                let envelope = build_structural_drift_json_envelope(
                    &drift_result,
                    &actual_graph,
                    &could_not_infer,
                );
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            }
            "github" | "github-actions" => {
                print_violations_github_actions(&drift_result.violations);
            }
            _ => {
                print_drift_text(
                    &drift_result,
                    Some(&actual_graph),
                    req.violations_only,
                    req.advisory,
                    &could_not_infer,
                );
            }
        }

        crate::commands::context_events::record_drift_compare(
            repo_path,
            drift_result.violations.len(),
            &format!("{:?}", drift_result.truth_status),
            false,
        );

        if should_fail_on_violations(req.fail_on, &drift_result.violations) {
            return Err(CliError::FailOnViolations);
        }
    }

    Ok(())
}

pub async fn drift_json_string(
    repo_root: &str,
    architecture_path: Option<&str>,
    violations_only: bool,
) -> Result<String, CliError> {
    let repo_path = Path::new(repo_root);

    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let actual_graph = scan_repo(repo_path)?;

    let resolved = architecture_path::resolve_architecture_path(repo_path);
    let effective_arch = architecture_path.or_else(|| resolved.as_ref().and_then(|p| p.to_str()));

    if let Some(arch_path) = effective_arch {
        let arch_file = Path::new(arch_path);
        if !arch_file.exists() {
            return Err(CliError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Architecture file not found: {}", arch_path),
            )));
        }
        let content = fs::read_to_string(arch_file)?;
        let parser = sruja_language::Parser::new(arch_path);
        let program = parser
            .parse(&content)
            .map_err(|diags| CliError::parse_with_diagnostics(arch_path.to_string(), diags))?;
        let proposed_graph = sruja_diff::program_to_graph(&program);
        let mode_str = load_repo_config(repo_path)
            .and_then(|c| c.baseline.mode)
            .unwrap_or_else(|| "auto".to_string());
        let mode = match mode_str.to_lowercase().as_str() {
            "overview" => sruja_diff::BaselineMode::Overview,
            "exhaustive" => sruja_diff::BaselineMode::Exhaustive,
            _ => sruja_diff::BaselineMode::Auto,
        };
        let diff_result = sruja_diff::compare_graphs_with_options(
            &actual_graph,
            &proposed_graph,
            sruja_diff::CompareOptions {
                baseline_mode: mode,
            },
        );

        if !violations_only {
            return Ok(serde_json::to_string_pretty(&diff_result)?);
        }

        let value = serde_json::to_value(&diff_result)?;
        let out = serde_json::json!({
            "truth_status": value.get("truth_status").cloned().unwrap_or(serde_json::Value::Null),
            "summary": value.get("summary").cloned().unwrap_or(serde_json::Value::Null),
            "violations": value.get("violations").cloned().unwrap_or(serde_json::Value::Null)
        });
        return Ok(serde_json::to_string_pretty(&out)?);
    }

    let drift_result = sruja_diff::detect_architectural_drift(&actual_graph);

    if !violations_only {
        return Ok(serde_json::to_string_pretty(&drift_result)?);
    }

    let value = serde_json::to_value(&drift_result)?;
    let out = serde_json::json!({
        "truth_status": value.get("truth_status").cloned().unwrap_or(serde_json::Value::Null),
        "health_score": value.get("health_score").cloned().unwrap_or(serde_json::Value::Null),
        "violations": value.get("violations").cloned().unwrap_or(serde_json::Value::Null)
    });
    Ok(serde_json::to_string_pretty(&out)?)
}

pub async fn status_result(repo_root: &str) -> Result<StatusOutput, CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let baseline = architecture_path::resolve_architecture_path(repo_path)
        .and_then(|p| p.to_str().map(String::from));

    let context_updated_at = std::fs::read_to_string(repo_path.join(".sruja/context.json"))
        .ok()
        .and_then(|s| {
            serde_json::from_str::<serde_json::Value>(&s)
                .ok()
                .and_then(|v| {
                    v.get("updated_at")
                        .and_then(|t| t.as_str())
                        .map(String::from)
                })
        });

    let graph = scan_repo(repo_path)?;

    // Calculate architectural velocity (recent supervision ratio)
    let velocity = (|| {
        let base = if std::process::Command::new("git")
            .args(["rev-parse", "origin/main"])
            .current_dir(repo_path)
            .status()
            .ok()?
            .success()
        {
            "origin/main"
        } else {
            "HEAD~20"
        };
        sruja_diff::architectural_velocity(repo_path, base, "HEAD", &graph).ok()
    })();

    // Calculate context score
    let context_score = (|| {
        let kg = crate::graph_store::load_or_build_graph(repo_path).ok()?;
        let age_hours = crate::utils::context::context_age_hours(repo_path);
        Some(sruja_graph::compute_context_score(&kg, graph.nodes.len(), repo_path, age_hours).score)
    })();

    if let Some(ref arch_path) = baseline {
        let content = fs::read_to_string(arch_path)?;
        let parser = sruja_language::Parser::new(arch_path);
        let program = parser
            .parse(&content)
            .map_err(|diags| CliError::parse_with_diagnostics(arch_path.clone(), diags))?;
        let proposed = sruja_diff::program_to_graph(&program);
        let diff = sruja_diff::compare_graphs(&graph, &proposed);
        let truth_status = match diff.truth_status {
            sruja_diff::TruthStatus::Reviewed => "reviewed",
            sruja_diff::TruthStatus::Drifted => "drifted",
            sruja_diff::TruthStatus::Unknown => "unknown",
        };
        let health_history = std::fs::read_to_string(repo_path.join(".sruja/health_history.json"))
            .ok()
            .and_then(|s| {
                serde_json::from_str::<serde_json::Value>(&s)
                    .ok()
                    .and_then(|v| {
                        v.get("scores")
                            .and_then(|scores| scores.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|e| {
                                        e.get("score").and_then(|s| s.as_u64()).map(|s| s as u8)
                                    })
                                    .collect::<Vec<u8>>()
                            })
                    })
            })
            .unwrap_or_default();

        let top_findings: Vec<super::output::Finding> = diff
            .violations
            .iter()
            .take(3)
            .map(|v| {
                let mut evidence: Vec<String> = v
                    .location
                    .as_ref()
                    .map(|s| vec![s.clone()])
                    .unwrap_or_default();
                for s in &v.sources {
                    evidence.push(sruja_diff::SourceRef::display_string(s));
                }
                super::output::Finding {
                    severity: format!("{:?}", v.severity).to_lowercase(),
                    kind: format!("{:?}", v.kind),
                    message: v.message.clone(),
                    evidence,
                }
            })
            .collect();

        return Ok(StatusOutput {
            baseline: Some(arch_path.clone()),
            truth_status: truth_status.to_string(),
            violations_count: diff.violations.len(),
            health_score: Some(diff.summary.health_score),
            context_updated_at,
            top_findings,
            context_score,
            health_history,
            velocity,
        });
    }

    let health_history = std::fs::read_to_string(repo_path.join(".sruja/health_history.json"))
        .ok()
        .and_then(|s| {
            serde_json::from_str::<serde_json::Value>(&s)
                .ok()
                .and_then(|v| {
                    v.get("scores")
                        .and_then(|scores| scores.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|e| {
                                    e.get("score").and_then(|s| s.as_u64()).map(|s| s as u8)
                                })
                                .collect::<Vec<u8>>()
                        })
                })
        })
        .unwrap_or_default();

    let drift = sruja_diff::detect_architectural_drift(&graph);
    let truth_status = match drift.truth_status {
        sruja_diff::TruthStatus::Reviewed => "reviewed",
        sruja_diff::TruthStatus::Drifted => "drifted",
        sruja_diff::TruthStatus::Unknown => "unknown",
    };

    let top_findings: Vec<super::output::Finding> = drift
        .violations
        .iter()
        .take(3)
        .map(|v| {
            let mut evidence: Vec<String> = v
                .location
                .as_ref()
                .map(|s| vec![s.clone()])
                .unwrap_or_default();
            for s in &v.sources {
                evidence.push(sruja_diff::SourceRef::display_string(s));
            }
            super::output::Finding {
                severity: format!("{:?}", v.severity).to_lowercase(),
                kind: format!("{:?}", v.kind),
                message: v.message.clone(),
                evidence,
            }
        })
        .collect();

    Ok(StatusOutput {
        baseline: None,
        truth_status: truth_status.to_string(),
        violations_count: drift.violations.len(),
        health_score: Some(drift.health_score),
        context_updated_at,
        top_findings,
        context_score,
        health_history,
        velocity,
    })
}

pub async fn drift_pr(
    repo_root: &str,
    base_ref: Option<&str>,
    head_ref: Option<&str>,
    format: &str,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let base = base_ref.unwrap_or("origin/main");
    let head = head_ref.unwrap_or("HEAD");

    eprintln!("🔍 PR-Scoped Drift Detection");
    eprintln!("   Base: {} | Head: {}", base, head);
    eprintln!();

    let git_ok = std::process::Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(repo_path)
        .output()
        .ok()
        .and_then(|o| o.status.success().then_some(()))
        .is_some();

    if !git_ok {
        return Err(CliError::validation(
            "Not a git repository. PR-scoped drift requires git.".to_string(),
        ));
    }

    let changed_files_output = std::process::Command::new("git")
        .args(["diff", "--name-only", &format!("{}...{}", base, head)])
        .current_dir(repo_path)
        .output()
        .map_err(|e| {
            CliError::Io(std::io::Error::other(format!(
                "Failed to get changed files: {}",
                e
            )))
        })?;

    let changed_files: Vec<String> = String::from_utf8_lossy(&changed_files_output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .collect();

    if changed_files.is_empty() {
        eprintln!("✅ No changed files detected between {} in {}", base, head);
        return Ok(());
    }

    eprintln!("📝 Changed files: {}", changed_files.len());

    let head_sha_output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()
        .ok();
    let head_sha = head_sha_output
        .and_then(|o| {
            o.status
                .success()
                .then(|| String::from_utf8_lossy(&o.stdout).trim().to_string())
        })
        .unwrap_or_default();
    let cache_dir = repo_path.join(".sruja").join("cache");
    let _ = fs::create_dir_all(&cache_dir);
    let head_cache_path = if !head_sha.is_empty() {
        cache_dir.join(format!("head_{}.json", head_sha))
    } else {
        PathBuf::new()
    };
    let head_graph = if !head_cache_path.as_os_str().is_empty() && head_cache_path.exists() {
        eprintln!(
            "📂 Using cached head graph ({})",
            &head_sha[..head_sha.len().min(8)]
        );
        let content = fs::read_to_string(&head_cache_path)?;
        serde_json::from_str(&content).map_err(CliError::Json)?
    } else {
        let g = scan_repo(repo_path)?;
        if !head_cache_path.as_os_str().is_empty() {
            if let Ok(json) = serde_json::to_string_pretty(&g) {
                let _ = fs::write(&head_cache_path, json);
            }
        }
        g
    };
    let head_drift = sruja_diff::detect_architectural_drift(&head_graph);

    // Map git diff to components (New native Phase 2 logic)
    let component_diffs = sruja_diff::map_git_diff(repo_path, base, head, &head_graph)
        .unwrap_or_else(|e| {
            eprintln!("⚠️  Warning: Failed to map git diff to components: {}", e);
            Vec::new()
        });

    let cache_filename = base.replace(['/', '.'], "_");
    let cache_path = cache_dir.join(format!("{}.json", cache_filename));
    let base_graph = if cache_path.exists() {
        let content = fs::read_to_string(&cache_path)?;
        serde_json::from_str(&content).map_err(CliError::Json)?
    } else {
        let worktree_dir =
            std::env::temp_dir().join(format!("sruja-drift-base-{}", std::process::id()));
        if worktree_dir.exists() {
            let _ = fs::remove_dir_all(&worktree_dir);
        }
        let status = std::process::Command::new("git")
            .arg("worktree")
            .arg("add")
            .arg("--detach")
            .arg(worktree_dir.as_path())
            .arg(base)
            .current_dir(repo_path)
            .status()
            .map_err(|e| {
                CliError::Io(std::io::Error::other(format!(
                    "git worktree add failed (is '{}' a valid ref?): {}",
                    base, e
                )))
            })?;
        if !status.success() {
            return Err(CliError::validation(format!(
                "Could not checkout base ref '{}'. Run a full scan on base and save to .sruja/cache/{}.json, or ensure the ref exists.",
                base, cache_filename
            )));
        }
        let base_graph = scan_repo(&worktree_dir).map_err(|e| {
            let _ = std::process::Command::new("git")
                .arg("worktree")
                .arg("remove")
                .arg("--force")
                .arg(worktree_dir.as_path())
                .current_dir(repo_path)
                .status();
            CliError::scan(e.to_string())
        })?;
        let _ = std::process::Command::new("git")
            .arg("worktree")
            .arg("remove")
            .arg("--force")
            .arg(worktree_dir.as_path())
            .current_dir(repo_path)
            .status();
        base_graph
    };

    let base_drift = sruja_diff::detect_architectural_drift(&base_graph);

    let new_violations: Vec<_> = head_drift
        .violations
        .iter()
        .filter(|hv| {
            !base_drift.violations.iter().any(|bv| {
                bv.kind == hv.kind && bv.message == hv.message && bv.location == hv.location
            })
        })
        .collect();

    let result = PrDriftResult {
        base_ref: base.to_string(),
        head_ref: head.to_string(),
        changed_files,
        base_health: base_drift.health_score,
        head_health: head_drift.health_score,
        new_violations: new_violations
            .iter()
            .map(|v| PrViolation {
                severity: format!("{:?}", v.severity),
                kind: format!("{:?}", v.kind),
                message: v.message.clone(),
                location: v.location.clone(),
                suggestion: v.suggestion.clone(),
            })
            .collect(),
        base_violations_count: base_drift.violations.len(),
        head_violations_count: head_drift.violations.len(),
        component_diffs,
    };

    match format {
        "json" => {
            let output = serde_json::to_string_pretty(&result)?;
            println!("{}", output);
        }
        "github-actions" => {
            print_github_actions_output(&result);
        }
        _ => {
            print_pr_drift_text(&result);
        }
    }

    if !result.new_violations.is_empty() {
        return Err(CliError::FailOnViolations);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::sync::Mutex;

    static DRIFT_PR_LOCK: Mutex<()> = Mutex::new(());

    fn git(repo: &std::path::Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(repo)
            .status()
            .expect("git command failed to spawn");
        assert!(status.success(), "git {:?} failed", args);
    }

    fn write_file(repo: &std::path::Path, rel: &str, content: &str) {
        let path = repo.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create dir");
        }
        std::fs::write(&path, content).expect("write file");
    }

    fn init_git_repo(repo: &std::path::Path) {
        git(repo, &["init"]);
        git(repo, &["config", "user.email", "tests@sruja.local"]);
        git(repo, &["config", "user.name", "Sruja Tests"]);
    }

    #[tokio::test]
    async fn drift_pr_runs_with_cache_and_worktree_paths() {
        let _guard = DRIFT_PR_LOCK.lock().unwrap();

        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path();
        init_git_repo(repo);
        write_file(
            repo,
            "Cargo.toml",
            r#"[package]
name = "fixture"
version = "0.1.0"
edition = "2021"
"#,
        );
        write_file(repo, "src/lib.rs", "pub fn a() -> u32 { 1 }\n");
        git(repo, &["add", "."]);
        git(repo, &["commit", "-m", "initial"]);

        write_file(repo, "src/lib.rs", "pub fn a() -> u32 { 1 }\n\n");
        git(repo, &["add", "."]);
        git(repo, &["commit", "-m", "touch"]);

        let repo_str = repo.to_str().expect("utf-8");
        super::drift_pr(repo_str, Some("HEAD~1"), Some("HEAD"), "json")
            .await
            .expect("drift_pr json");
        super::drift_pr(repo_str, Some("HEAD~1"), Some("HEAD"), "github-actions")
            .await
            .expect("drift_pr github-actions");
        super::drift_pr(repo_str, Some("HEAD~1"), Some("HEAD"), "text")
            .await
            .expect("drift_pr text");
        super::drift_pr(repo_str, Some("HEAD"), Some("HEAD"), "json")
            .await
            .expect("drift_pr no changes");
    }

    #[tokio::test]
    async fn drift_json_string_violations_only_has_expected_keys() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path();
        write_file(
            repo,
            "Cargo.toml",
            r#"[package]
name = "fixture"
version = "0.1.0"
edition = "2021"
"#,
        );
        write_file(repo, "src/lib.rs", "pub fn a() -> u32 { 1 }\n");

        let repo_str = repo.to_str().expect("utf-8");
        let json = super::drift_json_string(repo_str, None, true)
            .await
            .expect("drift_json_string");
        let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert!(v.get("truth_status").is_some());
        assert!(v.get("violations").is_some());
        assert!(v.get("health_score").is_some());
    }

    #[tokio::test]
    async fn status_result_includes_baseline_and_context_when_present() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path();
        init_git_repo(repo);
        write_file(
            repo,
            "Cargo.toml",
            r#"[package]
name = "fixture"
version = "0.1.0"
edition = "2021"
"#,
        );
        write_file(repo, "src/lib.rs", "pub fn a() -> u32 { 1 }\n");
        write_file(
            repo,
            "repo.sruja",
            r#"
system = kind "System"
container = kind "Container"

App = system "App" {
  description "App"
  Api = container "API" {
    technology "Rust"
    description "API"
  }
}
"#,
        );
        write_file(
            repo,
            ".sruja/context.json",
            r#"{ "updated_at": "2026-05-25T00:00:00Z" }"#,
        );
        write_file(
            repo,
            ".sruja/health_history.json",
            r#"{ "scores": [ { "score": 99 }, { "score": 97 } ] }"#,
        );
        git(repo, &["add", "."]);
        git(repo, &["commit", "-m", "baseline"]);

        let repo_str = repo.to_str().expect("utf-8");
        let status = super::status_result(repo_str).await.expect("status_result");
        assert!(status.baseline.is_some());
        assert_eq!(
            status.context_updated_at.as_deref(),
            Some("2026-05-25T00:00:00Z")
        );
        assert!(!status.truth_status.is_empty());
        assert!(status.health_score.is_some());
        assert_eq!(status.health_history, vec![99, 97]);
    }

    #[tokio::test]
    async fn should_fail_on_violations_matches_expected_criteria() {
        use sruja_diff::{Severity, Violation, ViolationKind};

        let violations = vec![
            Violation {
                kind: ViolationKind::CircularDependency,
                severity: Severity::Error,
                message: "cycle".to_string(),
                location: Some("a -> b".to_string()),
                suggestion: None,
                sources: vec![],
                confidence: None,
                evidence_count: Some(0),
                production_relevant: None,
                baseline_delta: None,
                suppressed: None,
                rule_id: None,
                rationale: None,
            },
            Violation {
                kind: ViolationKind::OrphanComponent,
                severity: Severity::Warning,
                message: "orphan".to_string(),
                location: Some("mod_x".to_string()),
                suggestion: None,
                sources: vec![],
                confidence: None,
                evidence_count: Some(0),
                production_relevant: None,
                baseline_delta: None,
                suppressed: None,
                rule_id: None,
                rationale: None,
            },
        ];

        assert!(super::should_fail_on_violations(Some("all"), &violations));
        assert!(super::should_fail_on_violations(
            Some("cycles"),
            &violations
        ));
        assert!(super::should_fail_on_violations(
            Some("orphans"),
            &violations
        ));
        assert!(!super::should_fail_on_violations(
            Some("layer-violations"),
            &violations
        ));
        assert!(!super::should_fail_on_violations(None, &violations));
    }

    #[tokio::test]
    async fn status_result_without_baseline_uses_drift_detection() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path();
        write_file(
            repo,
            "Cargo.toml",
            r#"[package]
name = "fixture"
version = "0.1.0"
edition = "2021"
"#,
        );
        write_file(repo, "src/lib.rs", "pub fn a() -> u32 { 1 }\n");
        write_file(
            repo,
            ".sruja/context.json",
            r#"{ "updated_at": "2026-05-25T00:00:00Z" }"#,
        );

        let repo_str = repo.to_str().expect("utf-8");
        let status = super::status_result(repo_str).await.expect("status_result");
        assert!(status.baseline.is_none());
        assert_eq!(
            status.context_updated_at.as_deref(),
            Some("2026-05-25T00:00:00Z")
        );
        assert!(!status.truth_status.is_empty());
        assert!(status.health_score.is_some());
    }

    #[tokio::test]
    async fn drift_json_string_with_architecture_violations_only_has_expected_keys() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path();
        write_file(
            repo,
            "Cargo.toml",
            r#"[package]
name = "fixture"
version = "0.1.0"
edition = "2021"
"#,
        );
        write_file(repo, "src/lib.rs", "pub fn a() -> u32 { 1 }\n");
        write_file(
            repo,
            "repo.sruja",
            r#"
system = kind "System"
App = system "App" { description "App" }
"#,
        );

        let repo_str = repo.to_str().expect("utf-8");
        let arch_path = repo.join("repo.sruja");
        let arch_str = arch_path.to_str().expect("utf-8");
        let json = super::drift_json_string(repo_str, Some(arch_str), true)
            .await
            .expect("drift_json_string");
        let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert!(v.get("truth_status").is_some());
        assert!(v.get("summary").is_some());
        assert!(v.get("violations").is_some());
    }
}
