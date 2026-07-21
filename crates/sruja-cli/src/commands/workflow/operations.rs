use std::path::{Path, PathBuf};

use crate::commands::workflow_aidlc::{self, AidlcStatus};
use crate::commands::CliError;

use super::manifest::*;
use super::types::*;

pub fn workflow_list(repo_root: &str) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let dir = workflows_dir(repo);
    if !dir.exists() {
        println!("(no workflows)");
        return Ok(());
    }

    let mut ids: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let id = entry.file_name().to_string_lossy().to_string();
            if manifest_path(repo, &id).exists() {
                ids.push(id);
            }
        }
    }
    ids.sort();
    ids.dedup();

    if ids.is_empty() {
        println!("(no workflows)");
        return Ok(());
    }

    for id in ids {
        let manifest = load_manifest(repo, &id)?;
        println!(
            "{}  phase={}  strict_gates={}  title={}",
            manifest.id, manifest.phase, manifest.strict_gates, manifest.title
        );
    }
    Ok(())
}

pub fn workflow_status(
    repo_root: &str,
    id: Option<&str>,
    check: bool,
) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let wf_id = if let Some(id) = id {
        id.to_string()
    } else {
        resolve_single_workflow_id(repo)?
    };
    let manifest = load_manifest(repo, &wf_id)?;
    let gate = compute_gate_check(repo, &manifest)?;
    let aidlc: AidlcStatus =
        workflow_aidlc::build_aidlc_status(repo, &wf_id, &manifest.phase, &manifest.aidlc_config());
    let extensions_enabled = crate::commands::extensions_config::enabled_extension_ids(repo);

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": "workflow_status/v2",
            "workflow": manifest,
            "gate": gate,
            "aidlc": aidlc,
            "extensions_enabled": extensions_enabled,
        }))?
    );

    if check && !gate.allowed {
        return Err(CliError::validation(format!(
            "Workflow gate failed (phase={}): {}",
            gate.phase,
            gate.missing.join(", ")
        )));
    }
    Ok(())
}

pub fn workflow_gate_check(repo_root: &str, id: &str) -> Result<WorkflowGateCheck, CliError> {
    let repo = Path::new(repo_root);
    if !manifest_path(repo, id).exists() {
        return Ok(WorkflowGateCheck {
            allowed: true,
            phase: "none".to_string(),
            missing: Vec::new(),
            aidlc_missing: Vec::new(),
            aidlc_stage: None,
        });
    }
    let manifest = load_manifest(repo, id)?;
    compute_gate_check(repo, &manifest)
}

pub fn workflow_get(repo_root: &str, id: &str) -> Result<WorkflowManifest, CliError> {
    let repo = Path::new(repo_root);
    load_manifest(repo, id)
}

pub fn workflow_record_impact(
    repo_root: &str,
    id: &str,
    depth: usize,
) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let manifest = load_manifest(repo, id)?;
    if manifest.target_elements.is_empty() {
        return Err(CliError::validation(
            "manifest.target_elements must be non-empty to record impact".to_string(),
        ));
    }

    let mut targets: Vec<serde_json::Value> = Vec::new();
    for t in &manifest.target_elements {
        let out = crate::commands::impact_compute_output(repo, t, depth)?;
        targets.push(serde_json::to_value(out)?);
    }
    let payload = serde_json::json!({
        "schema_version": "workflow_impact/v1",
        "workflow_id": manifest.id,
        "created_at": now_iso(),
        "targets": targets,
    });
    let out_path = inception_dir(repo, id).join("impact.json");
    std::fs::write(&out_path, serde_json::to_string_pretty(&payload)?)?;
    println!("{}", out_path.display());
    Ok(())
}

pub fn workflow_approve(
    repo_root: &str,
    id: &str,
    phase: &str,
    by: Option<&str>,
) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let mut manifest = load_manifest(repo, id)?;

    let required_paths: Vec<PathBuf> = match phase {
        "inception" => inception_required_files(repo, id, &manifest),
        "construction" => construction_required_files(repo, id, &manifest),
        "operations" => operations_required_files(repo, id, &manifest),
        other => return Err(CliError::validation(format!("Unknown phase: {}", other))),
    };

    let missing = list_missing(&required_paths);
    if !missing.is_empty() {
        return Err(CliError::validation(format!(
            "Cannot approve phase {}: missing artifacts: {}",
            phase,
            missing.join(", ")
        )));
    }

    if manifest.strict_gates {
        if phase == "construction" && !has_phase_approval(&manifest, "inception") {
            return Err(CliError::validation(
                "Cannot approve construction: inception not approved".to_string(),
            ));
        }
        if phase == "operations" && !has_phase_approval(&manifest, "construction") {
            return Err(CliError::validation(
                "Cannot approve operations: construction not approved".to_string(),
            ));
        }
    }

    let approval = WorkflowApproval {
        at: now_iso(),
        by: by.unwrap_or("human").to_string(),
        artifact_blake3: compute_artifact_hash(&required_paths)?,
    };

    match phase {
        "inception" => manifest.phase_approvals.inception = Some(approval),
        "construction" => manifest.phase_approvals.construction = Some(approval),
        "operations" => manifest.phase_approvals.operations = Some(approval),
        _ => {}
    }
    manifest.updated_at = now_iso();
    save_manifest(repo, &manifest)?;

    let record = crate::commands::context_events::ContextEventRecord {
        schema_version: crate::commands::context_events::CONTEXT_EVENTS_SCHEMA_V2.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        kind: "decision_accepted".to_string(),
        outcome: "ok".to_string(),
        policy_fingerprint: crate::commands::context_events::policy_fingerprint(repo),
        strict: None,
        details: serde_json::json!({
            "phase": phase,
            "workflow_id": manifest.id,
        }),
        trace_id: manifest.linked_trace_id.clone(),
        decision_id: manifest.linked_decision_id.clone(),
        run_id: None,
        workflow_id: Some(manifest.id.clone()),
        actor: Some("human".to_string()),
        source: Some("cli".to_string()),
        tool: Some("workflow approve".to_string()),
        elements: Some(manifest.target_elements.clone()),
        subject_ids: None,
        evidence_refs: None,
        summary: Some(format!("Approved workflow phase {}", phase)),
        ..Default::default()
    };
    crate::commands::context_events::append_context_event(repo, record);

    println!("Approved workflow {} phase {}", manifest.id, phase);
    Ok(())
}

pub fn workflow_advance(repo_root: &str, id: &str) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let mut manifest = load_manifest(repo, id)?;
    if manifest.strict_gates && !has_phase_approval(&manifest, &manifest.phase) {
        return Err(CliError::validation(format!(
            "Cannot advance: current phase '{}' is not approved",
            manifest.phase
        )));
    }

    let next = match manifest.phase.as_str() {
        "inception" => "construction",
        "construction" => "operations",
        "operations" => {
            return Err(CliError::validation(
                "Cannot advance: already in operations".to_string(),
            ))
        }
        other => {
            return Err(CliError::validation(format!(
                "Unknown workflow phase: {}",
                other
            )))
        }
    };
    manifest.phase = next.to_string();
    manifest.updated_at = now_iso();
    save_manifest(repo, &manifest)?;
    println!(
        "Advanced workflow {} to phase {}",
        manifest.id, manifest.phase
    );
    Ok(())
}

pub fn workflow_install_rules(repo_root: &str) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let installed = workflow_aidlc::install_aidlc_rules(repo)?;
    for p in installed {
        println!("Installed AIDLC rules: {p}");
    }
    Ok(())
}

pub fn workflow_validate(
    repo_root: &str,
    id: Option<&str>,
) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let wf_id = if let Some(id) = id {
        id.to_string()
    } else {
        resolve_single_workflow_id(repo)?
    };
    let manifest = load_manifest(repo, &wf_id)?;
    let gate = compute_gate_check(repo, &manifest)?;
    let out = serde_json::json!({
        "schema_version": "workflow_validate/v1",
        "workflow_id": wf_id,
        "allowed": gate.allowed,
        "missing": gate.missing,
        "aidlc_missing": gate.aidlc_missing,
        "aidlc_stage": gate.aidlc_stage,
    });
    println!("{}", serde_json::to_string_pretty(&out)?);
    if !gate.allowed {
        return Err(CliError::validation(format!(
            "Workflow validation failed: {}",
            gate.missing.join(", ")
        )));
    }
    Ok(())
}

pub fn workflow_audit(
    repo_root: &str,
    id: &str,
    event: &str,
    actor: Option<&str>,
) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let _ = load_manifest(repo, id)?;
    let path =
        workflow_aidlc::append_workflow_audit(repo, id, event, actor.unwrap_or("human"))?;
    println!("{}", path.display());
    Ok(())
}
