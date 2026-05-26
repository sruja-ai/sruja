use crate::commands::workflow_aidlc::{self, AidlcConfig, AidlcStatus};
use crate::commands::CliError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

fn workflows_dir(repo: &Path) -> PathBuf {
    repo.join(".sruja").join("workflows")
}

fn workflow_dir(repo: &Path, id: &str) -> PathBuf {
    workflows_dir(repo).join(id)
}

fn manifest_path(repo: &Path, id: &str) -> PathBuf {
    workflow_dir(repo, id).join("manifest.json")
}

fn inception_dir(repo: &Path, id: &str) -> PathBuf {
    workflow_dir(repo, id).join("inception")
}

fn construction_dir(repo: &Path, id: &str) -> PathBuf {
    workflow_dir(repo, id).join("construction")
}

fn operations_dir(repo: &Path, id: &str) -> PathBuf {
    workflow_dir(repo, id).join("operations")
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowApproval {
    pub at: String,
    pub by: String,
    pub artifact_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PhaseApprovals {
    pub inception: Option<WorkflowApproval>,
    pub construction: Option<WorkflowApproval>,
    pub operations: Option<WorkflowApproval>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowManifest {
    pub schema_version: String,
    pub id: String,
    pub title: String,
    pub phase: String,
    pub created_at: String,
    pub updated_at: String,
    pub repo_root: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_elements: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enabled_extensions: Vec<String>,
    pub phase_approvals: PhaseApprovals,
    pub strict_gates: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linked_trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linked_decision_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_design_review: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aidlc: Option<AidlcConfig>,
}

/// Options for `workflow init`.
#[derive(Debug, Clone)]
pub struct WorkflowInitOptions {
    pub with_aidlc: bool,
    pub aidlc_profile: String,
    pub install_rules: bool,
}

impl Default for WorkflowInitOptions {
    fn default() -> Self {
        Self {
            with_aidlc: false,
            aidlc_profile: "minimal".to_string(),
            install_rules: false,
        }
    }
}

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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowGateCheck {
    pub allowed: bool,
    pub phase: String,
    pub missing: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aidlc_missing: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aidlc_stage: Option<String>,
}

fn load_manifest(repo: &Path, id: &str) -> Result<WorkflowManifest, CliError> {
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

fn save_manifest(repo: &Path, manifest: &WorkflowManifest) -> Result<(), CliError> {
    let path = manifest_path(repo, &manifest.id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text =
        serde_json::to_string_pretty(manifest).map_err(|e| CliError::validation(e.to_string()))?;
    std::fs::write(&path, text)?;
    Ok(())
}

fn resolve_single_workflow_id(repo: &Path) -> Result<String, CliError> {
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

fn compute_artifact_hash(paths: &[PathBuf]) -> Result<String, CliError> {
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

fn inception_required_files(repo: &Path, id: &str, manifest: &WorkflowManifest) -> Vec<PathBuf> {
    let mut required = vec![
        inception_dir(repo, id).join("scope.md"),
        inception_dir(repo, id).join("impact.json"),
    ];
    if manifest.require_design_review.unwrap_or(false) {
        required.push(inception_dir(repo, id).join("design-review.md"));
    }
    required
}

fn construction_required_files(repo: &Path, id: &str) -> Vec<PathBuf> {
    vec![
        construction_dir(repo, id).join("task-plan.md"),
        construction_dir(repo, id).join("linked_proposal_ids.json"),
    ]
}

fn operations_required_files(repo: &Path, id: &str) -> Vec<PathBuf> {
    vec![operations_dir(repo, id).join("deploy-scope.json")]
}

fn list_missing(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .filter(|p| !p.exists())
        .map(|p| p.to_string_lossy().to_string())
        .collect()
}

fn has_phase_approval(manifest: &WorkflowManifest, phase: &str) -> bool {
    match phase {
        "inception" => manifest.phase_approvals.inception.is_some(),
        "construction" => manifest.phase_approvals.construction.is_some(),
        "operations" => manifest.phase_approvals.operations.is_some(),
        _ => false,
    }
}

fn drift_clean(repo: &Path) -> Result<bool, CliError> {
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

fn compute_gate_check(
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
            let required = construction_required_files(repo, id);
            missing.extend(list_missing(&required));

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
                    missing
                        .push("linked_proposal_ids.json must include at least one id".to_string());
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
            let required = operations_required_files(repo, id);
            missing.extend(list_missing(&required));
            let clean = drift_clean(repo)?;
            if !clean {
                missing.push("drift must be clean against repo.sruja".to_string());
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

pub fn workflow_init(
    repo_root: &str,
    title: &str,
    id: Option<&str>,
    target_elements: Vec<String>,
    strict_gates: bool,
    options: WorkflowInitOptions,
) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let wf_id = id
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()[..8].to_string());
    let dir = workflow_dir(repo, &wf_id);
    std::fs::create_dir_all(inception_dir(repo, &wf_id))?;
    std::fs::create_dir_all(construction_dir(repo, &wf_id))?;
    std::fs::create_dir_all(operations_dir(repo, &wf_id))?;

    if options.with_aidlc {
        let aidlc = AidlcConfig {
            enabled: true,
            profile: options.aidlc_profile.clone(),
            ..Default::default()
        };
        std::fs::create_dir_all(workflow_aidlc::aidlc_docs_dir(
            &inception_dir(repo, &wf_id),
            &aidlc,
        ))?;
        std::fs::create_dir_all(workflow_aidlc::aidlc_docs_dir(
            &construction_dir(repo, &wf_id),
            &aidlc,
        ))?;
        if options.install_rules {
            workflow_install_rules(repo_root)?;
        }
    }

    let mut manifest = WorkflowManifest::new(wf_id.clone(), title.to_string());
    manifest.target_elements = target_elements;
    manifest.strict_gates = strict_gates;
    manifest.repo_root = repo_root.to_string();
    if options.with_aidlc {
        manifest.aidlc = Some(AidlcConfig {
            enabled: true,
            profile: options.aidlc_profile,
            docs_root: workflow_aidlc::default_docs_root_for_manifest(),
            ..Default::default()
        });
    }
    save_manifest(repo, &manifest)?;

    println!("{}", serde_json::to_string_pretty(&manifest)?);
    println!(
        "Workflow initialized at {}",
        dir.join("manifest.json").display()
    );
    Ok(())
}

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

pub fn workflow_status(repo_root: &str, id: Option<&str>, check: bool) -> Result<(), CliError> {
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

pub fn workflow_record_impact(repo_root: &str, id: &str, depth: usize) -> Result<(), CliError> {
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
        "construction" => construction_required_files(repo, id),
        "operations" => operations_required_files(repo, id),
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

pub fn workflow_validate(repo_root: &str, id: Option<&str>) -> Result<(), CliError> {
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
    let path = workflow_aidlc::append_workflow_audit(repo, id, event, actor.unwrap_or("human"))?;
    println!("{}", path.display());
    Ok(())
}

fn find_traceability_tool_root(repo: &Path) -> Option<PathBuf> {
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
