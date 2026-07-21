use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::commands::workflow_aidlc::AidlcConfig;

pub(super) fn workflows_dir(repo: &Path) -> PathBuf {
    repo.join(".sruja").join("workflows")
}

pub(super) fn workflow_dir(repo: &Path, id: &str) -> PathBuf {
    workflows_dir(repo).join(id)
}

pub(super) fn manifest_path(repo: &Path, id: &str) -> PathBuf {
    workflow_dir(repo, id).join("manifest.json")
}

pub(super) fn inception_dir(repo: &Path, id: &str) -> PathBuf {
    workflow_dir(repo, id).join("inception")
}

pub(super) fn construction_dir(repo: &Path, id: &str) -> PathBuf {
    workflow_dir(repo, id).join("construction")
}

pub(super) fn operations_dir(repo: &Path, id: &str) -> PathBuf {
    workflow_dir(repo, id).join("operations")
}

pub(super) fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

pub(super) fn default_profile() -> String {
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

pub(super) fn requirements_template() -> &'static str {
    "# Requirements\n\n## Overview\nBrief description of the user request or requirements.\n\n## User Stories\n- **As a** ... **I want to** ... **So that** ...\n\n## Acceptance Criteria\n- [ ] AC-1: ...\n- [ ] AC-2: ...\n\n## Non-Functional Requirements\n- Performance: ...\n- Security: ...\n"
}

pub(super) fn scope_template() -> &'static str {
    "# Scope\n\n## What\nDescription of the proposed changes.\n\n## Why\nRationale for the changes.\n"
}

pub(super) fn design_review_template() -> &'static str {
    "# Design Review\n\n## Architectural Alignment\nSummary of structural reviews and checks.\n\n## Impact & Blast Radius\nSummary of affected architecture nodes.\n\n## Findings\n- [ ] ...\n"
}

pub(super) fn task_plan_template() -> &'static str {
    "# Task Plan\n\n## Implementation Order\n1. ...\n2. ...\n\n## Verification Steps\n- [ ] ...\n"
}

pub(super) fn test_plan_template() -> &'static str {
    "# Test Plan\n\n## Strategy\nOverview of the verification strategy.\n\n## Test Cases\n- [ ] TC-1: ...\n"
}

pub(super) fn rollback_plan_template() -> &'static str {
    "# Rollback Plan\n\n## Triggers\nWhen to perform a rollback.\n\n## Execution Steps\n1. ...\n"
}
