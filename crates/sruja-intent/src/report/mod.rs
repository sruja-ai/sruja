//! Intent Report Types
//!
//! Reporting structures for intent vs reality comparison.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentReport {
    pub intent_source: String,
    pub reality_source: String,
    pub drift_score: u8,
    pub health: String,
    pub violations: Vec<IntentViolation>,
    pub summary: IntentReportSummary,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentReportSummary {
    pub components_declared: usize,
    pub components_discovered: usize,
    pub relationships_declared: usize,
    pub relationships_discovered: usize,
    pub undocumented_count: usize,
    pub missing_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentViolation {
    pub kind: String,
    pub severity: String,
    pub description: String,
    pub component: Option<String>,
    pub evidence: Vec<String>,
    pub suggestion: Option<String>,
}

impl IntentReport {
    pub fn from_drift_report(report: &crate::compare::DriftReport) -> Self {
        let violations: Vec<IntentViolation> = report
            .drifts
            .iter()
            .map(|d| IntentViolation {
                kind: format!("{}", d.kind),
                severity: format!("{}", d.severity),
                description: d.description.clone(),
                component: d.evidence.first().map(|e| e.detail.clone()),
                evidence: d.evidence.iter().map(|e| e.detail.clone()).collect(),
                suggestion: d.suggestion.clone(),
            })
            .collect();

        let suggestions: Vec<String> = report
            .drifts
            .iter()
            .filter_map(|d| d.suggestion.clone())
            .take(10)
            .collect();

        Self {
            intent_source: report.intent_source.clone(),
            reality_source: report.reality_source.clone(),
            drift_score: report.drift_score,
            health: format!("{}", report.health),
            violations,
            summary: IntentReportSummary {
                components_declared: report.summary.total_components_declared,
                components_discovered: report.summary.total_components_discovered,
                relationships_declared: 0,
                relationships_discovered: 0,
                undocumented_count: report.summary.undocumented_components,
                missing_count: report.summary.missing_components,
            },
            suggestions,
        }
    }

    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str("# Intent vs Reality Report\n\n");

        md.push_str(&format!("**Intent Source:** {}\n\n", self.intent_source));
        md.push_str(&format!("**Reality Source:** {}\n\n", self.reality_source));
        md.push_str(&format!(
            "**Drift Score:** {}/100 ({})\n\n",
            self.drift_score, self.health
        ));

        md.push_str("## Summary\n\n");
        md.push_str(&format!(
            "- Components declared: {}\n",
            self.summary.components_declared
        ));
        md.push_str(&format!(
            "- Components discovered: {}\n",
            self.summary.components_discovered
        ));
        md.push_str(&format!(
            "- Undocumented: {}\n",
            self.summary.undocumented_count
        ));
        md.push_str(&format!("- Missing: {}\n\n", self.summary.missing_count));

        if !self.violations.is_empty() {
            md.push_str("## Violations\n\n");
            for v in &self.violations {
                md.push_str(&format!("### {} [{}]\n\n", v.kind, v.severity));
                md.push_str(&format!("{}\n\n", v.description));
                if let Some(ref suggestion) = v.suggestion {
                    md.push_str(&format!("**Suggestion:** {}\n\n", suggestion));
                }
            }
        }

        if !self.suggestions.is_empty() {
            md.push_str("## Suggestions\n\n");
            for (i, s) in self.suggestions.iter().enumerate() {
                md.push_str(&format!("{}. {}\n", i + 1, s));
            }
        }

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compare::{
        Drift, DriftHealth, DriftKind, DriftReport, DriftSummary, Evidence, Severity,
    };

    fn minimal_drift_report(
        drifts: Vec<Drift>,
        drift_score: u8,
        health: DriftHealth,
    ) -> DriftReport {
        DriftReport {
            intent_source: "arch.sruja".to_string(),
            reality_source: "repo/".to_string(),
            drifts,
            drift_score,
            health,
            summary: DriftSummary {
                total_components_declared: 2,
                total_components_discovered: 3,
                undocumented_components: 1,
                missing_components: 0,
                undocumented_relationships: 0,
                boundary_violations: 0,
                policy_violations: 0,
                schema_violations: 0,
                taxonomy_mismatches: 0,
            },
        }
    }

    #[test]
    fn from_drift_report_empty_drifts() {
        let report = minimal_drift_report(vec![], 100, DriftHealth::Healthy);
        let intent_report = IntentReport::from_drift_report(&report);
        assert_eq!(intent_report.intent_source, "arch.sruja");
        assert_eq!(intent_report.drift_score, 100);
        assert!(intent_report.violations.is_empty());
        assert_eq!(intent_report.summary.components_declared, 2);
    }

    #[test]
    fn from_drift_report_with_violation() {
        let drifts = vec![Drift {
            kind: DriftKind::UndocumentedComponent,
            severity: Severity::Medium,
            description: "Component X not in docs".to_string(),
            evidence: vec![Evidence {
                source: "scan".to_string(),
                location: Some("src/x.rs".to_string()),
                detail: "Discovered node".to_string(),
            }],
            intent_ref: None,
            suggestion: Some("Add to docs".to_string()),
        }];
        let report = minimal_drift_report(drifts, 80, DriftHealth::MinorDrift);
        let intent_report = IntentReport::from_drift_report(&report);
        assert_eq!(intent_report.violations.len(), 1);
        assert!(intent_report.violations[0].kind.contains("Undocumented"));
        assert_eq!(intent_report.suggestions.len(), 1);
    }

    #[test]
    fn to_markdown_includes_sections() {
        let report = minimal_drift_report(vec![], 90, DriftHealth::Healthy);
        let intent_report = IntentReport::from_drift_report(&report);
        let md = intent_report.to_markdown();
        assert!(md.contains("# Intent vs Reality Report"));
        assert!(md.contains("**Intent Source:**"));
        assert!(md.contains("## Summary"));
        assert!(md.contains("Components declared: 2"));
    }

    #[test]
    fn to_markdown_includes_violations_and_suggestions() {
        let drifts = vec![Drift {
            kind: DriftKind::MissingComponent,
            severity: Severity::High,
            description: "Y missing in repo".to_string(),
            evidence: vec![],
            intent_ref: None,
            suggestion: Some("Implement Y".to_string()),
        }];
        let report = minimal_drift_report(drifts, 70, DriftHealth::SignificantDrift);
        let intent_report = IntentReport::from_drift_report(&report);
        let md = intent_report.to_markdown();
        assert!(md.contains("## Violations"));
        assert!(md.contains("## Suggestions"));
    }
}
