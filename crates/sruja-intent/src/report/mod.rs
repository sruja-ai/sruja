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
