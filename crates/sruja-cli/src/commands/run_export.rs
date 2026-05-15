use crate::commands::context_events::{read_context_events_query, ContextEventQuery};
use crate::commands::CliError;
use crate::utils::run_snapshots::run_dir;
use std::fs;
use std::path::{Path, PathBuf};

fn agent_run_dir(repo_root: &Path, run_id: &str) -> PathBuf {
    repo_root
        .join(".sruja")
        .join("agent")
        .join("runs")
        .join(run_id)
}

fn export_dir(repo_root: &Path, run_id: &str) -> PathBuf {
    repo_root.join(".sruja").join("run_exports").join(run_id)
}

fn copy_dir_recursive(src: &Path, dst: &Path, copied: &mut Vec<String>) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let dst_path = dst.join(&name);
        if path.is_dir() {
            copy_dir_recursive(&path, &dst_path, copied)?;
        } else {
            fs::create_dir_all(dst)?;
            fs::copy(&path, &dst_path)?;
            copied.push(dst_path.to_string_lossy().to_string());
        }
    }
    Ok(())
}

pub async fn run_export(
    repo: &str,
    run_id: &str,
    out: Option<&str>,
    events_limit: usize,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let out_dir = out
        .map(PathBuf::from)
        .unwrap_or_else(|| export_dir(repo_path, run_id));
    fs::create_dir_all(&out_dir)?;

    let mut copied: Vec<String> = Vec::new();

    let snapshots_src = run_dir(repo_path, run_id);
    if snapshots_src.exists() {
        let dst = out_dir.join("runs");
        copy_dir_recursive(&snapshots_src, &dst, &mut copied).map_err(CliError::Io)?;
    }

    let agent_src = agent_run_dir(repo_path, run_id);
    if agent_src.exists() {
        let dst = out_dir.join("agent_runs");
        copy_dir_recursive(&agent_src, &dst, &mut copied).map_err(CliError::Io)?;
    }

    let events = read_context_events_query(
        repo_path,
        ContextEventQuery {
            limit: events_limit,
            kind_filter: None,
            details_substring: None,
            decision_id: None,
            trace_id: None,
            run_id: Some(run_id),
            element_id: None,
            decision_lineage_only: false,
        },
    )
    .unwrap_or_default();

    fs::write(
        out_dir.join("context_events.json"),
        serde_json::to_string_pretty(&events).unwrap_or_default(),
    )
    .map_err(CliError::Io)?;

    let manifest = serde_json::json!({
        "schema_version": "run_export/v1",
        "repo": repo,
        "run_id": run_id,
        "out_dir": out_dir.to_string_lossy(),
        "included": {
            "runs_dir": snapshots_src.exists(),
            "agent_runs_dir": agent_src.exists(),
            "context_events_count": events.len(),
        },
        "copied_files": copied,
    });
    fs::write(
        out_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap_or_default(),
    )
    .map_err(CliError::Io)?;

    println!("{}", out_dir.display());
    Ok(())
}
