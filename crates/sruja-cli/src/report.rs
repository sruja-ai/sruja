//! Compliance report DTOs inlined into sruja-cli (previously in a separate reporting crate).

use serde::{Deserialize, Serialize};
use sruja_diff::Violation;

/// Compliance status indicating whether the architecture conforms to expectations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    /// The architecture is fully compliant with no violations
    Compliant,
    /// The architecture has one or more violations (structural, policy, or boundary)
    NonCompliant,
}

/// One intent or policy drift entry (DTO for serialization).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftEntry {
    pub kind: String,
    pub severity: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

/// One policy violation entry (DTO).
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
    /// Build from structural violations and drift entries.
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

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_diff::{Severity, ViolationKind};

    fn sample_violation(message: &str) -> Violation {
        Violation {
            kind: ViolationKind::CircularDependency,
            severity: Severity::Error,
            message: message.to_string(),
            location: None,
            suggestion: None,
            sources: Vec::new(),
            confidence: None,
            evidence_count: None,
            production_relevant: None,
            baseline_delta: None,
            suppressed: None,
            rule_id: None,
            rationale: None,
        }
    }

    #[test]
    fn from_parts_returns_compliant_when_no_structural_policy_or_boundary() {
        let report = ComplianceReport::from_parts(Vec::new(), Vec::new(), Vec::new(), 0, 100);
        assert_eq!(report.status, ComplianceStatus::Compliant);
        assert!(report.remediation_checklist.is_empty());
    }

    #[test]
    fn from_parts_flags_non_compliant_for_structural_violations_and_adds_checklist_item() {
        let report = ComplianceReport::from_parts(
            vec![sample_violation("cycle")],
            Vec::new(),
            Vec::new(),
            0,
            80,
        );
        assert_eq!(report.status, ComplianceStatus::NonCompliant);
        assert!(report
            .remediation_checklist
            .iter()
            .any(|s| s.contains("Fix 1 structural violation")));
    }

    #[test]
    fn from_parts_flags_non_compliant_for_policy_violations_and_adds_checklist_item() {
        let report = ComplianceReport::from_parts(
            Vec::new(),
            vec![DriftEntry {
                kind: "PolicyViolation".to_string(),
                severity: "Error".to_string(),
                description: "nope".to_string(),
                suggestion: None,
            }],
            vec![PolicyViolationEntry {
                policy_name: "NoProdDb".to_string(),
                message: "db access".to_string(),
                source: "A".to_string(),
                target: "B".to_string(),
            }],
            0,
            90,
        );
        assert_eq!(report.status, ComplianceStatus::NonCompliant);
        assert!(report
            .remediation_checklist
            .iter()
            .any(|s| s.contains("Resolve 1 policy violation")));
        assert!(!report
            .remediation_checklist
            .iter()
            .any(|s| s.contains("Address 1 intent drift")));
    }

    #[test]
    fn from_parts_flags_non_compliant_for_boundary_violations_and_adds_checklist_item() {
        let report = ComplianceReport::from_parts(Vec::new(), Vec::new(), Vec::new(), 2, 95);
        assert_eq!(report.status, ComplianceStatus::NonCompliant);
        assert!(report
            .remediation_checklist
            .iter()
            .any(|s| s.contains("Fix 2 boundary violation")));
    }

    #[test]
    fn from_parts_adds_intent_drift_checklist_item_when_only_intent_drifts_exist() {
        let report = ComplianceReport::from_parts(
            Vec::new(),
            vec![DriftEntry {
                kind: "UndocumentedComponent".to_string(),
                severity: "Warning".to_string(),
                description: "missing docs".to_string(),
                suggestion: Some("document it".to_string()),
            }],
            Vec::new(),
            0,
            100,
        );
        assert_eq!(report.status, ComplianceStatus::Compliant);
        assert!(report
            .remediation_checklist
            .iter()
            .any(|s| s.contains("Address 1 intent drift")));
    }
}
