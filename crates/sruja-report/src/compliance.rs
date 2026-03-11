//! Compliance report: single status combining structural drift, intent drift, and policy violations.

use serde::{Deserialize, Serialize};
use sruja_diff::Violation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
}

/// One intent or policy drift entry (DTO for serialization; built by CLI from sruja_intent::Drift).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftEntry {
    pub kind: String,
    pub severity: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

/// One policy violation entry (DTO; built from graph PolicyViolation or intent Drift).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolationEntry {
    pub policy_name: String,
    pub message: String,
    pub source: String,
    pub target: String,
}

/// Aggregated compliance report for CI and tooling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub status: ComplianceStatus,
    pub health_score: u8,
    pub structural_violations: Vec<Violation>,
    pub drift_entries: Vec<DriftEntry>,
    pub policy_violations: Vec<PolicyViolationEntry>,
    pub boundary_violations_count: u32,
    pub remediation_checklist: Vec<String>,
}

impl ComplianceReport {
    /// Build from structural violations and drift entries. Status is NonCompliant if there are
    /// any structural errors or any policy/boundary violations.
    pub fn from_parts(
        structural_violations: Vec<Violation>,
        drift_entries: Vec<DriftEntry>,
        policy_violations: Vec<PolicyViolationEntry>,
        boundary_violations_count: u32,
        health_score: u8,
    ) -> Self {
        let has_structural = !structural_violations.is_empty();
        let has_policy = !policy_violations.is_empty();
        let has_boundary = boundary_violations_count > 0;

        let status = if has_structural || has_policy || has_boundary {
            ComplianceStatus::NonCompliant
        } else {
            ComplianceStatus::Compliant
        };

        let mut remediation_checklist: Vec<String> = Vec::new();
        if !structural_violations.is_empty() {
            remediation_checklist.push(format!(
                "Fix {} structural violation(s) (cycles, layers, god modules, orphans)",
                structural_violations.len()
            ));
        }
        if !policy_violations.is_empty() {
            remediation_checklist.push(format!(
                "Resolve {} policy violation(s)",
                policy_violations.len()
            ));
        }
        if boundary_violations_count > 0 {
            remediation_checklist.push(format!(
                "Fix {} boundary violation(s)",
                boundary_violations_count
            ));
        }
        if !drift_entries.is_empty()
            && policy_violations.is_empty()
            && boundary_violations_count == 0
        {
            remediation_checklist.push(format!(
                "Address {} intent drift(s) (undocumented/missing components or relationships)",
                drift_entries.len()
            ));
        }

        Self {
            status,
            health_score,
            structural_violations,
            drift_entries,
            policy_violations,
            boundary_violations_count,
            remediation_checklist,
        }
    }
}
