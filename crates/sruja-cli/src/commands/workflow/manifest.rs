use std::path::{Path, PathBuf};

use crate::commands::workflow_aidlc::{self, AidlcConfig};
use crate::commands::CliError;

use super::types::*;

impl WorkflowManifest {
    pub fn aidlc_config(&self) -> AidlcConfig {
        let mut cfg = self.aidlc.clone().unwrap_or_default();
        if cfg.docs_root.is_empty() {
            cfg.docs_root = workflow_aidlc::default_docs_root_for_manifest();
        }
        cfg
    }

    pub fn new(id: String, title: String) -> Self {
        let now = now_iso();
        Self {
            schema_version: "workflow/v2".to_string(),
            id,
            title,
            phase: "inception".to_string(),
            created_at: now.clone(),
            updated_at: now,
            repo_root: ".".to_string(),
            target_elements: Vec::new(),
            enabled_extensions: Vec::new(),
            phase_approvals: PhaseApprovals::default(),
            strict_gates: true,
            linked_trace_id: None,
            linked_decision_id: None,
            require_design_review: None,
            aidlc: None,
            profile: "minimal".to_string(),
        }
    }
}

pub(super) fn load_manifest(repo: &Path, id: &str) -> Result<WorkflowManifest, CliError> {
    let path = manifest_path(repo, id);
    let text = std::fs::read_to_string(&path).map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("{}: {}", path.display(), e),
        ))
    })?;
    let manifest = serde_json::from_str(&text).map_err(|e| CliError::validation(e.to_string()))?;
    Ok(manifest)
}

pub(super) fn save_manifest(repo: &Path, manifest: &WorkflowManifest) -> Result<(), CliError> {
    let path = manifest_path(repo, &manifest.id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text =
        serde_json::to_string_pretty(manifest).map_err(|e| CliError::validation(e.to_string()))?;
    std::fs::write(&path, text)?;
    Ok(())
}

pub(super) fn resolve_single_workflow_id(repo: &Path) -> Result<String, CliError> {
    let dir = workflows_dir(repo);
    if !dir.exists() {
        return Err(CliError::validation(
            "No workflows found under .sruja/workflows".to_string(),
        ));
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
    match ids.len() {
        0 => Err(CliError::validation(
            "No workflows found under .sruja/workflows".to_string(),
        )),
        1 => Ok(ids[0].clone()),
        _ => Err(CliError::validation(format!(
            "Multiple workflows found ({}). Pass --id. IDs: {}",
            ids.len(),
            ids.join(", ")
        ))),
    }
}

pub(super) fn compute_artifact_hash(paths: &[PathBuf]) -> Result<String, CliError> {
    let mut hasher = blake3::Hasher::new();
    for p in paths {
        hasher.update(p.to_string_lossy().as_bytes());
        hasher.update(b"\n");
        let bytes = std::fs::read(p)?;
        hasher.update(&bytes);
        hasher.update(b"\n");
    }
    Ok(hasher.finalize().to_hex().to_string())
}

pub(super) fn inception_required_files(
    repo: &Path,
    id: &str,
    manifest: &WorkflowManifest,
) -> Vec<PathBuf> {
    if manifest.profile == "e2e" {
        return vec![
            inception_dir(repo, id).join("requirements.md"),
            inception_dir(repo, id).join("scope.md"),
            inception_dir(repo, id).join("impact.json"),
            inception_dir(repo, id).join("design-review.md"),
        ];
    }
    let mut required = vec![
        inception_dir(repo, id).join("scope.md"),
        inception_dir(repo, id).join("impact.json"),
    ];
    if manifest.require_design_review.unwrap_or(false) {
        required.push(inception_dir(repo, id).join("design-review.md"));
    }
    required
}

pub(super) fn construction_required_files(
    repo: &Path,
    id: &str,
    manifest: &WorkflowManifest,
) -> Vec<PathBuf> {
    if manifest.profile == "e2e" {
        return vec![
            construction_dir(repo, id).join("task-plan.md"),
            construction_dir(repo, id).join("linked_proposal_ids.json"),
            construction_dir(repo, id).join("test-plan.md"),
            construction_dir(repo, id).join("test-results.json"),
        ];
    }
    vec![
        construction_dir(repo, id).join("task-plan.md"),
        construction_dir(repo, id).join("linked_proposal_ids.json"),
    ]
}

pub(super) fn operations_required_files(
    repo: &Path,
    id: &str,
    manifest: &WorkflowManifest,
) -> Vec<PathBuf> {
    if manifest.profile == "e2e" {
        return vec![
            operations_dir(repo, id).join("deploy-scope.json"),
            operations_dir(repo, id).join("readiness.json"),
            operations_dir(repo, id).join("rollback-plan.md"),
        ];
    }
    vec![operations_dir(repo, id).join("deploy-scope.json")]
}

pub(super) fn list_missing(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .filter(|p| !p.exists())
        .map(|p| p.to_string_lossy().to_string())
        .collect()
}

pub(super) fn has_phase_approval(manifest: &WorkflowManifest, phase: &str) -> bool {
    match phase {
        "inception" => manifest.phase_approvals.inception.is_some(),
        "construction" => manifest.phase_approvals.construction.is_some(),
        "operations" => manifest.phase_approvals.operations.is_some(),
        _ => false,
    }
}

pub(super) fn drift_clean(repo: &Path) -> Result<bool, CliError> {
    let sruja_file = repo.join("repo.sruja");
    if !sruja_file.exists() {
        return Ok(true);
    }
    let content = std::fs::read_to_string(&sruja_file)?;
    let parser = sruja_language::Parser::new(sruja_file.to_string_lossy().to_string());
    let program = parser.parse(&content).map_err(|diags| {
        CliError::parse_with_diagnostics(sruja_file.to_string_lossy().to_string(), diags)
    })?;
    let declared = sruja_diff::program_to_graph(&program);
    let actual = sruja_scan::scan_repo(repo)?;
    let diff = sruja_diff::compare_graphs(&actual, &declared);
    Ok(diff
        .violations
        .iter()
        .all(|v| !matches!(v.severity, sruja_diff::Severity::Error)))
}

pub(super) fn compute_gate_check(
    repo: &Path,
    manifest: &WorkflowManifest,
) -> Result<WorkflowGateCheck, CliError> {
    let aidlc = manifest.aidlc_config();
    let aidlc_status = if aidlc.enabled {
        Some(workflow_aidlc::build_aidlc_status(
            repo,
            &manifest.id,
            &manifest.phase,
            &aidlc,
        ))
    } else {
        None
    };
    let aidlc_missing = aidlc_status
        .as_ref()
        .map(|s| s.missing.clone())
        .unwrap_or_default();
    let aidlc_stage = aidlc_status
        .as_ref()
        .and_then(|s| s.state.current_stage.clone());

    if !manifest.strict_gates {
        return Ok(WorkflowGateCheck {
            allowed: true,
            phase: manifest.phase.clone(),
            missing: Vec::new(),
            aidlc_missing,
            aidlc_stage,
        });
    }

    let id = manifest.id.as_str();
    let mut missing: Vec<String> = Vec::new();

    match manifest.phase.as_str() {
        "inception" => {
            let required = inception_required_files(repo, id, manifest);
            missing.extend(list_missing(&required));
        }
        "construction" => {
            if !has_phase_approval(manifest, "inception") {
                missing.push("inception not approved".to_string());
            }
            let required = construction_required_files(repo, id, manifest);
            missing.extend(list_missing(&required));

            if manifest.profile == "e2e" {
                let test_results_path = construction_dir(repo, id).join("test-results.json");
                if test_results_path.exists() {
                    if let Ok(text) = std::fs::read_to_string(&test_results_path) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                            if val["all_passed"].as_bool() != Some(true) {
                                missing.push("construction/test-results.json is not successful (all_passed is false)".to_string());
                            }
                        } else {
                            missing.push(
                                "construction/test-results.json has invalid JSON".to_string(),
                            );
                        }
                    } else {
                        missing.push("construction/test-results.json could not be read".to_string());
                    }
                }
            }

            let linked_path = construction_dir(repo, id).join("linked_proposal_ids.json");
            if linked_path.exists() {
                let text = std::fs::read_to_string(&linked_path)?;
                let ids: Vec<String> = serde_json::from_str(&text).map_err(|e| {
                    CliError::validation(format!(
                        "Invalid JSON in {}: {}",
                        linked_path.to_string_lossy(),
                        e
                    ))
                })?;
                if ids.is_empty() {
                    missing.push("linked_proposal_ids.json must include at least one id".to_string());
                } else {
                    let proposals = sruja_diff::Proposal::load_all(repo)
                        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
                    let mut ok_any = false;
                    let mut not_found: Vec<String> = Vec::new();
                    let mut not_ok: Vec<String> = Vec::new();
                    for pid in ids {
                        let Some(p) = proposals.iter().find(|p| p.id == pid) else {
                            not_found.push(pid);
                            continue;
                        };
                        if matches!(
                            p.status,
                            sruja_diff::ProposalStatus::Pending
                                | sruja_diff::ProposalStatus::Approved
                        ) {
                            ok_any = true;
                            continue;
                        }
                        not_ok.push(format!("{} ({:?})", p.id, p.status));
                    }
                    if !ok_any {
                        if !not_found.is_empty() {
                            not_found.sort();
                            not_found.dedup();
                            missing.push(format!(
                                "linked proposals not found: {}",
                                not_found.join(", ")
                            ));
                        }
                        if !not_ok.is_empty() {
                            not_ok.sort();
                            not_ok.dedup();
                            missing.push(format!(
                                "linked proposals not Pending|Approved: {}",
                                not_ok.join(", ")
                            ));
                        }
                        missing.push("no linked proposal is Pending or Approved".to_string());
                    }
                }
            }
        }
        "operations" => {
            if !has_phase_approval(manifest, "construction") {
                missing.push("construction not approved".to_string());
            }
            let required = operations_required_files(repo, id, manifest);
            missing.extend(list_missing(&required));
            let clean = drift_clean(repo)?;
            if !clean {
                missing.push("drift must be clean against repo.sruja".to_string());
            }

            if manifest.profile == "e2e" {
                let readiness_path = operations_dir(repo, id).join("readiness.json");
                if readiness_path.exists() {
                    if let Ok(text) = std::fs::read_to_string(&readiness_path) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                            if val["all_ready"].as_bool() != Some(true) {
                                missing.push(
                                    "operations/readiness.json is not ready (all_ready is false)"
                                        .to_string(),
                                );
                            }
                        } else {
                            missing.push("operations/readiness.json has invalid JSON".to_string());
                        }
                    } else {
                        missing.push("operations/readiness.json could not be read".to_string());
                    }
                }
            }
        }
        other => {
            return Err(CliError::validation(format!(
                "Unknown workflow phase: {}",
                other
            )));
        }
    }

    missing.extend(aidlc_missing.clone());
    missing.sort();
    missing.dedup();

    Ok(WorkflowGateCheck {
        allowed: missing.is_empty(),
        phase: manifest.phase.clone(),
        missing,
        aidlc_missing,
        aidlc_stage,
    })
}
