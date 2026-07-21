use std::path::Path;

use crate::commands::workflow_aidlc;
use crate::commands::CliError;

use super::manifest::load_manifest;
use super::types::*;

fn find_traceability_tool_root(repo: &Path) -> Option<std::path::PathBuf> {
    workflow_aidlc::resolve_vendored_aidlc_rules(repo).and_then(|rules| {
        rules
            .parent()
            .map(|aw| aw.join("scripts").join("aidlc-traceability"))
            .filter(|p| p.is_dir())
    })
}

pub fn workflow_trace(
    repo_root: &str,
    id: &str,
    format: &str,
    check_only: bool,
) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let manifest = load_manifest(repo, id)?;
    let aidlc = manifest.aidlc_config();
    let docs = workflow_aidlc::aidlc_docs_dir(&inception_dir(repo, id), &aidlc);
    let input = if docs.is_dir() {
        docs.clone()
    } else {
        repo.join("aidlc-docs")
    };
    if !input.is_dir() {
        return Err(CliError::validation(format!(
            "No aidlc-docs directory at {} or repo aidlc-docs/",
            docs.display()
        )));
    }

    let Some(tool_root) = find_traceability_tool_root(repo) else {
        return Err(CliError::validation(
            "aidlc-traceability not found; vendor aidlc-workflows or set SRUJA_AIDLC_RULES"
                .to_string(),
        ));
    };

    if check_only {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "schema_version": "workflow_trace_check/v1",
                "input": input.display().to_string(),
                "tool_root": tool_root.display().to_string(),
            })
        );
        return Ok(());
    }

    let mut cmd = std::process::Command::new("python3");
    cmd.arg("-m")
        .arg("traceability")
        .arg("generate")
        .arg("--input")
        .arg(&input)
        .arg("--format")
        .arg(format)
        .arg("--no-ai")
        .current_dir(&tool_root)
        .env(
            "PYTHONPATH",
            tool_root.join("src").to_string_lossy().to_string(),
        );
    let status = cmd.status().map_err(CliError::Io)?;
    if !status.success() {
        return Err(CliError::validation(
            "traceability generate failed (install aidlc-traceability deps in that package)"
                .to_string(),
        ));
    }
    println!("Traceability report generated from {}", input.display());
    Ok(())
}

pub fn workflow_run(
    repo_root: &str,
    id: &str,
    vision: &Path,
    dry_run: bool,
) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let _manifest = load_manifest(repo, id)?;
    if !vision.is_file() {
        return Err(CliError::validation(format!(
            "vision file not found: {}",
            vision.display()
        )));
    }
    let evaluator = workflow_aidlc::resolve_vendored_aidlc_rules(repo).and_then(|rules| {
        let mut cur = rules;
        for _ in 0..6 {
            let run_py = cur.join("scripts").join("aidlc-evaluator").join("run.py");
            if run_py.is_file() {
                return Some(run_py);
            }
            if !cur.pop() {
                break;
            }
        }
        None
    });

    if dry_run {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "workflow_run_plan/v1",
                "workflow_id": id,
                "vision": vision.display().to_string(),
                "evaluator": evaluator.as_ref().map(|p| p.display().to_string()),
                "pre_hooks": ["sruja sync", "workflow record-impact", "workflow install-rules"],
                "post_hooks": ["workflow status --check", "sruja drift"],
            }))?
        );
        return Ok(());
    }

    let Some(run_py) = evaluator else {
        return Err(CliError::validation(
            "aidlc-evaluator run.py not found; use editor-driven AIDLC or set SRUJA_AIDLC_RULES"
                .to_string(),
        ));
    };

    let status = std::process::Command::new("python3")
        .arg(&run_py)
        .arg("full")
        .arg("--vision")
        .arg(vision)
        .current_dir(run_py.parent().unwrap().parent().unwrap())
        .status()
        .map_err(CliError::Io)?;
    if !status.success() {
        return Err(CliError::validation(
            "aidlc-evaluator run failed".to_string(),
        ));
    }
    Ok(())
}
