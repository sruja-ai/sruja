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

fn default_profile() -> String {
    "minimal".to_string()
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
    #[serde(default = "default_profile")]
    pub profile: String,
}

/// Options for `workflow init`.
#[derive(Debug, Clone)]
pub struct WorkflowInitOptions {
    pub with_aidlc: bool,
    pub aidlc_profile: String,
    pub install_rules: bool,
    pub profile: String,
    pub template: Option<String>,
}

impl Default for WorkflowInitOptions {
    fn default() -> Self {
        Self {
            with_aidlc: false,
            aidlc_profile: "minimal".to_string(),
            install_rules: false,
            profile: "minimal".to_string(),
            template: None,
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
            profile: "minimal".to_string(),
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

fn construction_required_files(repo: &Path, id: &str, manifest: &WorkflowManifest) -> Vec<PathBuf> {
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

fn operations_required_files(repo: &Path, id: &str, manifest: &WorkflowManifest) -> Vec<PathBuf> {
    if manifest.profile == "e2e" {
        return vec![
            operations_dir(repo, id).join("deploy-scope.json"),
            operations_dir(repo, id).join("readiness.json"),
            operations_dir(repo, id).join("rollback-plan.md"),
        ];
    }
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
                        missing
                            .push("construction/test-results.json could not be read".to_string());
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

fn requirements_template() -> &'static str {
    "# Requirements\n\n## Overview\nBrief description of the user request or requirements.\n\n## User Stories\n- **As a** ... **I want to** ... **So that** ...\n\n## Acceptance Criteria\n- [ ] AC-1: ...\n- [ ] AC-2: ...\n\n## Non-Functional Requirements\n- Performance: ...\n- Security: ...\n"
}

fn scope_template() -> &'static str {
    "# Scope\n\n## What\nDescription of the proposed changes.\n\n## Why\nRationale for the changes.\n"
}

fn design_review_template() -> &'static str {
    "# Design Review\n\n## Architectural Alignment\nSummary of structural reviews and checks.\n\n## Impact & Blast Radius\nSummary of affected architecture nodes.\n\n## Findings\n- [ ] ...\n"
}

fn task_plan_template() -> &'static str {
    "# Task Plan\n\n## Implementation Order\n1. ...\n2. ...\n\n## Verification Steps\n- [ ] ...\n"
}

fn test_plan_template() -> &'static str {
    "# Test Plan\n\n## Strategy\nOverview of the verification strategy.\n\n## Test Cases\n- [ ] TC-1: ...\n"
}

fn rollback_plan_template() -> &'static str {
    "# Rollback Plan\n\n## Triggers\nWhen to perform a rollback.\n\n## Execution Steps\n1. ...\n"
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
    let inc_dir = inception_dir(repo, &wf_id);
    let con_dir = construction_dir(repo, &wf_id);
    let op_dir = operations_dir(repo, &wf_id);

    std::fs::create_dir_all(&inc_dir)?;
    std::fs::create_dir_all(&con_dir)?;
    std::fs::create_dir_all(&op_dir)?;

    let template = options
        .template
        .clone()
        .unwrap_or_else(|| "minimal".to_string());
    let profile = if options.profile != "minimal" {
        options.profile.clone()
    } else {
        match template.as_str() {
            "e2e" => "e2e".to_string(),
            _ => "minimal".to_string(),
        }
    };

    // Scaffolding based on template
    match template.as_str() {
        "e2e" => {
            std::fs::write(inc_dir.join("requirements.md"), requirements_template())?;
            std::fs::write(inc_dir.join("scope.md"), scope_template())?;
            std::fs::write(
                inc_dir.join("impact.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_impact/v1",
                    "workflow_id": wf_id,
                    "targets": []
                }))?,
            )?;
            std::fs::write(inc_dir.join("design-review.md"), design_review_template())?;

            std::fs::write(con_dir.join("task-plan.md"), task_plan_template())?;
            std::fs::write(con_dir.join("linked_proposal_ids.json"), "[]")?;
            std::fs::write(con_dir.join("test-plan.md"), test_plan_template())?;
            std::fs::write(
                con_dir.join("test-results.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_test_results/v1",
                    "workflow_id": wf_id,
                    "recorded_at": now_iso(),
                    "profile": "coding",
                    "all_passed": false,
                    "steps": []
                }))?,
            )?;

            std::fs::write(
                op_dir.join("deploy-scope.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_deploy_scope/v1",
                    "resources": []
                }))?,
            )?;
            std::fs::write(
                op_dir.join("readiness.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_readiness/v1",
                    "workflow_id": wf_id,
                    "recorded_at": now_iso(),
                    "checks": {
                        "drift_clean": false,
                        "lint_clean": false,
                        "intent_aligned": false,
                        "tests_passed": false,
                        "rollback_plan": false,
                        "deploy_scope_defined": false
                    },
                    "all_ready": false
                }))?,
            )?;
            std::fs::write(op_dir.join("rollback-plan.md"), rollback_plan_template())?;
        }
        "feature" => {
            std::fs::write(inc_dir.join("requirements.md"), requirements_template())?;
            std::fs::write(inc_dir.join("scope.md"), scope_template())?;
            std::fs::write(
                inc_dir.join("impact.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_impact/v1",
                    "workflow_id": wf_id,
                    "targets": []
                }))?,
            )?;

            std::fs::write(con_dir.join("task-plan.md"), task_plan_template())?;
            std::fs::write(con_dir.join("linked_proposal_ids.json"), "[]")?;
            std::fs::write(
                con_dir.join("test-results.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_test_results/v1",
                    "workflow_id": wf_id,
                    "recorded_at": now_iso(),
                    "profile": "coding",
                    "all_passed": false,
                    "steps": []
                }))?,
            )?;

            std::fs::write(
                op_dir.join("deploy-scope.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_deploy_scope/v1",
                    "resources": []
                }))?,
            )?;
        }
        "bugfix" => {
            std::fs::write(inc_dir.join("scope.md"), scope_template())?;
            std::fs::write(
                inc_dir.join("impact.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_impact/v1",
                    "workflow_id": wf_id,
                    "targets": []
                }))?,
            )?;

            std::fs::write(con_dir.join("task-plan.md"), task_plan_template())?;
            std::fs::write(con_dir.join("linked_proposal_ids.json"), "[]")?;
            std::fs::write(
                con_dir.join("test-results.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_test_results/v1",
                    "workflow_id": wf_id,
                    "recorded_at": now_iso(),
                    "profile": "coding",
                    "all_passed": false,
                    "steps": []
                }))?,
            )?;

            std::fs::write(
                op_dir.join("deploy-scope.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_deploy_scope/v1",
                    "resources": []
                }))?,
            )?;
        }
        _ => {
            // "minimal"
            std::fs::write(inc_dir.join("scope.md"), scope_template())?;
            std::fs::write(
                inc_dir.join("impact.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_impact/v1",
                    "workflow_id": wf_id,
                    "targets": []
                }))?,
            )?;

            std::fs::write(con_dir.join("task-plan.md"), task_plan_template())?;
            std::fs::write(con_dir.join("linked_proposal_ids.json"), "[]")?;

            std::fs::write(
                op_dir.join("deploy-scope.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_deploy_scope/v1",
                    "resources": []
                }))?,
            )?;
        }
    }

    if options.with_aidlc {
        let aidlc = AidlcConfig {
            enabled: true,
            profile: options.aidlc_profile.clone(),
            ..Default::default()
        };
        std::fs::create_dir_all(workflow_aidlc::aidlc_docs_dir(&inc_dir, &aidlc))?;
        std::fs::create_dir_all(workflow_aidlc::aidlc_docs_dir(&con_dir, &aidlc))?;
        if options.install_rules {
            workflow_install_rules(repo_root)?;
        }
    }

    let mut manifest = WorkflowManifest::new(wf_id.clone(), title.to_string());
    manifest.target_elements = target_elements;
    manifest.strict_gates = strict_gates;
    manifest.repo_root = repo_root.to_string();
    manifest.profile = profile;
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

pub fn workflow_capture_requirements(
    repo_root: &str,
    id: Option<&str>,
    from_issue: Option<&str>,
    enrich_cmd: Option<&str>,
) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let wf_id = if let Some(i) = id {
        i.to_string()
    } else {
        resolve_single_workflow_id(repo)?
    };

    let manifest = load_manifest(repo, &wf_id)?;
    let requirements_path = inception_dir(repo, &wf_id).join("requirements.md");

    let mut current_content = if requirements_path.exists() {
        std::fs::read_to_string(&requirements_path)?
    } else {
        requirements_template().to_string()
    };

    if let Some(issue_url) = from_issue {
        current_content.push_str(&format!(
            "\n\n## Ingested from Issue\nCaptured from: {}\n\n- Ingested on: {}\n",
            issue_url,
            now_iso()
        ));
    }

    if let Some(cmd) = enrich_cmd {
        // Run external enrichment command
        // We pass the current requirements content as JSON on stdin
        let input_json = serde_json::json!({
            "workflow_id": wf_id,
            "title": manifest.title,
            "profile": manifest.profile,
            "current_requirements": current_content,
        });
        let payload =
            serde_json::to_vec(&input_json).map_err(|e| CliError::validation(e.to_string()))?;
        let limits = crate::integrations::EnrichmentLimits {
            timeout_ms: 15000,
            max_bytes: 20000,
        };
        match crate::integrations::run_cmd_enrichment(cmd, &payload, limits) {
            Ok(narrative) => {
                current_content = narrative;
            }
            Err(e) => {
                return Err(CliError::validation(format!(
                    "Enrichment command failed: {}",
                    e
                )));
            }
        }
    }

    std::fs::write(&requirements_path, &current_content)?;

    // Append audit event
    let event_msg = if from_issue.is_some() && enrich_cmd.is_some() {
        "Captured and enriched requirements"
    } else if from_issue.is_some() {
        "Captured requirements from issue"
    } else if enrich_cmd.is_some() {
        "Enriched requirements"
    } else {
        "Scaffolded requirements"
    };

    let audit_path = workflow_aidlc::append_workflow_audit(repo, &wf_id, event_msg, "human")?;

    println!(
        "Requirements captured/updated at {}",
        requirements_path.display()
    );
    println!("Appended audit log at {}", audit_path.display());

    Ok(())
}

pub async fn workflow_record_test_results(
    repo_root: &str,
    id: Option<&str>,
    profile: Option<&str>,
    from_file: Option<&str>,
) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let wf_id = if let Some(i) = id {
        i.to_string()
    } else {
        resolve_single_workflow_id(repo)?
    };

    let _manifest = load_manifest(repo, &wf_id)?;
    let test_results_path = construction_dir(repo, &wf_id).join("test-results.json");

    let payload = if let Some(file_path) = from_file {
        let text = std::fs::read_to_string(file_path)?;
        let val: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| CliError::validation(format!("Invalid JSON in test-results: {}", e)))?;
        val
    } else {
        let prof = profile.unwrap_or("coding");
        let verify_opts = crate::commands::VerifyTaskOptions {
            repo: repo_root,
            profile: prof,
            file: None,
            max_runtime_ms: None,
        };
        let out = crate::commands::verify_task(verify_opts).await?;
        serde_json::to_value(&out)?
    };

    if let Some(parent) = test_results_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&test_results_path, serde_json::to_string_pretty(&payload)?)?;

    // Determine all_passed from payload
    let all_passed = payload["all_passed"].as_bool().unwrap_or(false);

    // Append audit event
    let event_msg = format!(
        "Recorded test results (profile={:?}, all_passed={})",
        profile.unwrap_or("coding"),
        all_passed
    );
    let audit_path = workflow_aidlc::append_workflow_audit(repo, &wf_id, &event_msg, "human")?;

    println!("Test results recorded at {}", test_results_path.display());
    println!("Appended audit log at {}", audit_path.display());

    Ok(())
}

pub fn workflow_record_readiness(repo_root: &str, id: Option<&str>) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let wf_id = if let Some(i) = id {
        i.to_string()
    } else {
        resolve_single_workflow_id(repo)?
    };

    let _manifest = load_manifest(repo, &wf_id)?;
    let readiness_path = operations_dir(repo, &wf_id).join("readiness.json");

    let drift_val = drift_clean(repo)?;
    let lint_val = if repo.join("repo.sruja").exists() {
        let content = std::fs::read_to_string(repo.join("repo.sruja"))?;
        let parser = sruja_language::Parser::new("repo.sruja".to_string());
        parser.parse(&content).is_ok()
    } else {
        true
    };

    let test_results_path = construction_dir(repo, &wf_id).join("test-results.json");
    let tests_val = if test_results_path.exists() {
        if let Ok(text) = std::fs::read_to_string(&test_results_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                val["all_passed"].as_bool().unwrap_or(false)
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    let rollback_path = operations_dir(repo, &wf_id).join("rollback-plan.md");
    let rollback_val = rollback_path.exists()
        && std::fs::metadata(&rollback_path)
            .map(|m| m.len() > 10)
            .unwrap_or(false);

    let deploy_scope_path = operations_dir(repo, &wf_id).join("deploy-scope.json");
    let deploy_val = deploy_scope_path.exists()
        && std::fs::metadata(&deploy_scope_path)
            .map(|m| m.len() > 5)
            .unwrap_or(false);

    let all_ready = drift_val && lint_val && tests_val && rollback_val && deploy_val;

    let payload = serde_json::json!({
        "schema_version": "workflow_readiness/v1",
        "workflow_id": wf_id,
        "recorded_at": now_iso(),
        "checks": {
            "drift_clean": drift_val,
            "lint_clean": lint_val,
            "intent_aligned": true,
            "tests_passed": tests_val,
            "rollback_plan": rollback_val,
            "deploy_scope_defined": deploy_val
        },
        "all_ready": all_ready
    });

    if let Some(parent) = readiness_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&readiness_path, serde_json::to_string_pretty(&payload)?)?;

    // Append audit event
    let event_msg = format!("Recorded operations readiness (all_ready={})", all_ready);
    let audit_path = workflow_aidlc::append_workflow_audit(repo, &wf_id, &event_msg, "human")?;

    println!("Readiness recorded at {}", readiness_path.display());
    println!("Appended audit log at {}", audit_path.display());

    Ok(())
}

pub fn workflow_summary(repo_root: &str, id: Option<&str>, format: &str) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let wf_id = if let Some(i) = id {
        i.to_string()
    } else {
        resolve_single_workflow_id(repo)?
    };

    let manifest = load_manifest(repo, &wf_id)?;
    let gate = compute_gate_check(repo, &manifest)?;

    // Calculate completeness & health score
    let mut total_checks = 0;
    let mut passed_checks = 0;

    let inc_reqs = inception_required_files(repo, &wf_id, &manifest);
    total_checks += inc_reqs.len();
    passed_checks += inc_reqs.len() - list_missing(&inc_reqs).len();

    let con_reqs = construction_required_files(repo, &wf_id, &manifest);
    total_checks += con_reqs.len();
    passed_checks += con_reqs.len() - list_missing(&con_reqs).len();

    let op_reqs = operations_required_files(repo, &wf_id, &manifest);
    total_checks += op_reqs.len();
    passed_checks += op_reqs.len() - list_missing(&op_reqs).len();

    let test_results_path = construction_dir(repo, &wf_id).join("test-results.json");
    let mut tests_pass = false;
    if test_results_path.exists() {
        total_checks += 1;
        if let Ok(text) = std::fs::read_to_string(&test_results_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                if val["all_passed"].as_bool() == Some(true) {
                    tests_pass = true;
                    passed_checks += 1;
                }
            }
        }
    }

    let readiness_path = operations_dir(repo, &wf_id).join("readiness.json");
    let mut ready_pass = false;
    if readiness_path.exists() {
        total_checks += 1;
        if let Ok(text) = std::fs::read_to_string(&readiness_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                if val["all_ready"].as_bool() == Some(true) {
                    ready_pass = true;
                    passed_checks += 1;
                }
            }
        }
    }

    let health_score = if total_checks > 0 {
        ((passed_checks as f32 / total_checks as f32) * 100.0) as u8
    } else {
        100
    };

    if format == "json" {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "workflow_summary/v1",
                "workflow_id": wf_id,
                "title": manifest.title,
                "profile": manifest.profile,
                "phase": manifest.phase,
                "health_score": health_score,
                "gate_allowed": gate.allowed,
                "missing": gate.missing,
                "inception_approved": manifest.phase_approvals.inception.is_some(),
                "construction_approved": manifest.phase_approvals.construction.is_some(),
                "operations_approved": manifest.phase_approvals.operations.is_some(),
            }))?
        );
        return Ok(());
    }

    // Text format - beautiful terminal dashboard
    println!("============================================================");
    println!("                    SRUJA WORKFLOW DASHBOARD                ");
    println!("============================================================");
    println!("Workflow ID   : {}", wf_id);
    println!("Title         : {}", manifest.title);
    println!("Profile       : {}", manifest.profile.to_uppercase());
    println!("Health Score  : {}/100", health_score);
    println!(
        "Gate Allowed  : {}",
        if gate.allowed { "✔ YES" } else { "✘ NO" }
    );
    println!("------------------------------------------------------------");

    // Phase progression timeline
    print!("Timeline      : ");
    let p = manifest.phase.as_str();
    if p == "inception" {
        print!("[INCEPTION] ===> construction ===> operations");
    } else if p == "construction" {
        print!("inception ===> [CONSTRUCTION] ===> operations");
    } else {
        print!("inception ===> construction ===> [OPERATIONS]");
    }
    println!("\n------------------------------------------------------------");

    // Artifacts check
    println!("Phase Artifacts Status:");
    println!("  [Inception]");
    for path in inc_reqs {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        println!("    {} {}", if path.exists() { "✔" } else { "✘" }, name);
    }
    println!("  [Construction]");
    for path in con_reqs {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        println!("    {} {}", if path.exists() { "✔" } else { "✘" }, name);
    }
    println!("  [Operations]");
    for path in op_reqs {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        println!("    {} {}", if path.exists() { "✔" } else { "✘" }, name);
    }
    println!("------------------------------------------------------------");

    // Linked proposals
    let linked_path = construction_dir(repo, &wf_id).join("linked_proposal_ids.json");
    if linked_path.exists() {
        if let Ok(text) = std::fs::read_to_string(&linked_path) {
            if let Ok(ids) = serde_json::from_str::<Vec<String>>(&text) {
                if !ids.is_empty() {
                    println!("Linked Proposals:");
                    for pid in ids {
                        println!("  - {}", pid);
                    }
                    println!("------------------------------------------------------------");
                }
            }
        }
    }

    // Test Results Summary
    if test_results_path.exists() {
        println!(
            "Test Results  : {}",
            if tests_pass {
                "✔ PASSED"
            } else {
                "✘ FAILED / UNRECORDED"
            }
        );
        println!("------------------------------------------------------------");
    }

    // Readiness Summary
    if readiness_path.exists() {
        println!(
            "Readiness     : {}",
            if ready_pass {
                "✔ READY"
            } else {
                "✘ NOT READY"
            }
        );
        println!("------------------------------------------------------------");
    }

    // Last 3 Audit Events
    let audit_path = workflow_dir(repo, &wf_id).join("audit.jsonl");
    if audit_path.exists() {
        if let Ok(text) = std::fs::read_to_string(&audit_path) {
            let lines: Vec<&str> = text.lines().rev().take(3).collect();
            if !lines.is_empty() {
                println!("Audit Trail (Last 3):");
                for line in lines {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                        let at_str = val["at"].as_str().unwrap_or("");
                        let at = at_str.get(..19).unwrap_or(at_str);
                        let actor = val["actor"].as_str().unwrap_or("");
                        let event = val["event"].as_str().unwrap_or("");
                        println!("  [{}] {} - {}", at, actor, event);
                    }
                }
                println!("------------------------------------------------------------");
            }
        }
    }

    Ok(())
}

pub fn workflow_summary_json_value(
    repo_root: &str,
    id: &str,
) -> Result<serde_json::Value, CliError> {
    let repo = Path::new(repo_root);
    let manifest = load_manifest(repo, id)?;
    let gate = compute_gate_check(repo, &manifest)?;

    // Calculate completeness & health score
    let mut total_checks = 0;
    let mut passed_checks = 0;

    let inc_reqs = inception_required_files(repo, id, &manifest);
    total_checks += inc_reqs.len();
    passed_checks += inc_reqs.len() - list_missing(&inc_reqs).len();

    let con_reqs = construction_required_files(repo, id, &manifest);
    total_checks += con_reqs.len();
    passed_checks += con_reqs.len() - list_missing(&con_reqs).len();

    let op_reqs = operations_required_files(repo, id, &manifest);
    total_checks += op_reqs.len();
    passed_checks += op_reqs.len() - list_missing(&op_reqs).len();

    let test_results_path = construction_dir(repo, id).join("test-results.json");
    let mut tests_pass = false;
    if test_results_path.exists() {
        total_checks += 1;
        if let Ok(text) = std::fs::read_to_string(&test_results_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                if val["all_passed"].as_bool() == Some(true) {
                    tests_pass = true;
                    passed_checks += 1;
                }
            }
        }
    }

    let readiness_path = operations_dir(repo, id).join("readiness.json");
    let mut ready_pass = false;
    if readiness_path.exists() {
        total_checks += 1;
        if let Ok(text) = std::fs::read_to_string(&readiness_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                if val["all_ready"].as_bool() == Some(true) {
                    ready_pass = true;
                    passed_checks += 1;
                }
            }
        }
    }

    let health_score = if total_checks > 0 {
        ((passed_checks as f32 / total_checks as f32) * 100.0) as u8
    } else {
        100
    };

    Ok(serde_json::json!({
        "schema_version": "workflow_summary/v1",
        "workflow_id": id,
        "title": manifest.title,
        "profile": manifest.profile,
        "phase": manifest.phase,
        "health_score": health_score,
        "gate_allowed": gate.allowed,
        "missing": gate.missing,
        "inception_approved": manifest.phase_approvals.inception.is_some(),
        "construction_approved": manifest.phase_approvals.construction.is_some(),
        "operations_approved": manifest.phase_approvals.operations.is_some(),
        "tests_passed": tests_pass,
        "readiness_passed": ready_pass,
    }))
}

pub fn workflow_next_steps_json_value(
    repo_root: &str,
    wf_id: &str,
) -> Result<serde_json::Value, CliError> {
    let repo = Path::new(repo_root);
    let manifest = load_manifest(repo, wf_id)?;
    let gate = compute_gate_check(repo, &manifest)?;

    let mut steps = Vec::new();
    let mut recommendations = Vec::new();

    let phase = manifest.phase.as_str();
    let profile = manifest.profile.as_str();

    match phase {
        "inception" => {
            // Requirements for e2e
            if profile == "e2e" {
                let req_path = inception_dir(repo, wf_id).join("requirements.md");
                if !req_path.exists() {
                    steps.push("Create requirements.md by running `sruja workflow capture-requirements` or creating `.sruja/workflows/<id>/inception/requirements.md`".to_string());
                    recommendations.push(
                        "Traceable requirements are mandatory for the e2e profile.".to_string(),
                    );
                }
            }

            let scope_path = inception_dir(repo, wf_id).join("scope.md");
            if !scope_path.exists() {
                steps.push(
                    "Scaffold scope.md by creating `.sruja/workflows/<id>/inception/scope.md`"
                        .to_string(),
                );
            }

            let impact_path = inception_dir(repo, wf_id).join("impact.json");
            if !impact_path.exists() {
                steps.push("Generate impact.json for your target elements under `.sruja/workflows/<id>/inception/impact.json`".to_string());
            }

            let dr_path = inception_dir(repo, wf_id).join("design-review.md");
            if !dr_path.exists() {
                steps.push(
                    "Perform design review by running `sruja workflow design-review`".to_string(),
                );
            } else {
                // If design review exists, load blockers
                let dr_json = workflow_dir(repo, wf_id).join("design-review.json");
                if dr_json.exists() {
                    if let Ok(text) = std::fs::read_to_string(&dr_json) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(blockers) = val["blockers"].as_array() {
                                for b in blockers {
                                    if let Some(b_str) = b.as_str() {
                                        steps.push(format!(
                                            "Resolve design review blocker: {}",
                                            b_str
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if steps.is_empty() && manifest.phase_approvals.inception.is_none() {
                steps.push("Approve the inception phase by running `sruja workflow approve` or updating phase_approvals in manifest.".to_string());
                recommendations.push(
                    "Once approved, transition the phase in manifest.json to 'construction'."
                        .to_string(),
                );
            }
        }
        "construction" => {
            let tp_path = construction_dir(repo, wf_id).join("task-plan.md");
            if !tp_path.exists() {
                steps.push("Create task-plan.md under `.sruja/workflows/<id>/construction/task-plan.md` to detail implementation steps.".to_string());
            }

            // Check if any proposals exist or are linked
            let linked_path = construction_dir(repo, wf_id).join("linked_proposal_ids.json");
            let mut has_linked = false;
            if linked_path.exists() {
                if let Ok(text) = std::fs::read_to_string(&linked_path) {
                    if let Ok(ids) = serde_json::from_str::<Vec<String>>(&text) {
                        has_linked = !ids.is_empty();
                    }
                }
            }
            if !has_linked {
                steps.push("Propose architecture topology changes or link an existing proposal by listing it in `.sruja/workflows/<id>/construction/linked_proposal_ids.json`".to_string());
            }

            // Test results
            let test_results_path = construction_dir(repo, wf_id).join("test-results.json");
            if !test_results_path.exists() {
                steps.push("Run and record test verification results using `sruja workflow record-test-results`".to_string());
            } else {
                if let Ok(text) = std::fs::read_to_string(&test_results_path) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                        if val["all_passed"].as_bool() != Some(true) {
                            steps.push("Fix test failures in verification suite and re-record test results".to_string());
                        }
                    }
                }
            }

            if steps.is_empty() && manifest.phase_approvals.construction.is_none() {
                steps.push("Approve the construction phase by running `sruja workflow approve` or updating phase_approvals in manifest.".to_string());
                recommendations.push(
                    "Once approved, transition the phase in manifest.json to 'operations'."
                        .to_string(),
                );
            }
        }
        "operations" => {
            let ds_path = operations_dir(repo, wf_id).join("deploy-scope.json");
            if !ds_path.exists() {
                steps.push("Define deployment scope under `.sruja/workflows/<id>/operations/deploy-scope.json`".to_string());
            }

            // Readiness
            let readiness_path = operations_dir(repo, wf_id).join("readiness.json");
            if !readiness_path.exists() {
                steps.push("Perform operations readiness checks and record results using `sruja workflow record-readiness`".to_string());
            } else {
                if let Ok(text) = std::fs::read_to_string(&readiness_path) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                        if val["all_ready"].as_bool() != Some(true) {
                            steps.push("Resolve readiness blockers (such as architectural drift, lint failures, or failing tests) and re-record readiness".to_string());
                        }
                    }
                }
            }

            if steps.is_empty() && manifest.phase_approvals.operations.is_none() {
                steps.push("Approve operations to complete the workflow by running `sruja workflow approve`.".to_string());
            }
        }
        _ => {
            steps.push(format!("Unknown phase: {}", phase));
        }
    }

    if !gate.allowed {
        recommendations.push(format!(
            "Currently gated! Missing requirements to proceed: {}",
            gate.missing.join(", ")
        ));
    }

    Ok(serde_json::json!({
        "schema_version": "workflow_next_steps/v1",
        "workflow_id": wf_id,
        "current_phase": phase,
        "profile": profile,
        "gate_allowed": gate.allowed,
        "next_steps": steps,
        "recommendations": recommendations,
    }))
}
