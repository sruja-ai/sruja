//! Formatting for architectural critique reports

use crate::critique::{CritiqueCategory, CritiqueReport, CritiqueSeverity};
use colored::*;

/// Formats a critique report for terminal output
pub fn format_critique_text(report: &CritiqueReport) -> String {
    let mut out = String::new();

    let risk_icon = match report.risk_level {
        crate::critique::RiskLevel::Clear => "✅",
        crate::critique::RiskLevel::Caution => "🟡",
        crate::critique::RiskLevel::Warning => "⚠️",
        crate::critique::RiskLevel::Danger => "🔴",
    };

    out.push_str("╔══════════════════════════════════════════════════════════════╗\n");
    out.push_str(&format!(
        "║  ARCHITECTURAL CRITIQUE — Risk Level: {} {:<14} ║\n",
        risk_icon,
        format!("{:?}", report.risk_level).to_uppercase()
    ));
    out.push_str("╠══════════════════════════════════════════════════════════════╣\n\n");

    out.push_str(&format!("📋 Summary: {}\n", report.summary));
    out.push_str(&format!(
        "   Blast radius: {} downstream consumers\n\n",
        report.blast_radius.downstream_consumers
    ));

    // Group findings by severity
    let severities = [
        CritiqueSeverity::Critical,
        CritiqueSeverity::High,
        CritiqueSeverity::Medium,
        CritiqueSeverity::Low,
    ];

    for sev in severities {
        let findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.severity == sev)
            .collect();
        if findings.is_empty() {
            continue;
        }

        let (header, color_fn): (&str, fn(&str) -> ColoredString) = match sev {
            CritiqueSeverity::Critical => ("Critical", |s| s.red().bold()),
            CritiqueSeverity::High => ("High", |s| s.red()),
            CritiqueSeverity::Medium => ("Medium", |s| s.yellow()),
            CritiqueSeverity::Low => ("Low", |s| s.blue()),
        };

        out.push_str(&format!(
            "── {} ─────────────────────────────────────────────────\n",
            color_fn(header)
        ));
        for f in findings {
            let icon = match f.severity {
                CritiqueSeverity::Critical => "🔴",
                CritiqueSeverity::High => "🟠",
                CritiqueSeverity::Medium => "🟡",
                CritiqueSeverity::Low => "🔵",
            };

            let category_label = match f.category {
                CritiqueCategory::PolicyViolation => "Policy Violation",
                CritiqueCategory::HistoricalPatternMatch => "Historical Match",
                CritiqueCategory::ConstraintBreach => "Constraint Breach",
                CritiqueCategory::BlastRadius => "Blast Radius",
                CritiqueCategory::BehavioralContractDrift => "Behavioral Drift",
                CritiqueCategory::GotchaWarning => "Gotcha",
                CritiqueCategory::UnproposedChange => "Unproposed Change",
            };

            out.push_str(&format!(
                "  {} {}: {}\n",
                icon,
                color_fn(category_label),
                f.title.bold()
            ));
            out.push_str(&format!("     Detail: {}\n", f.detail));
            if let Some(s) = &f.suggestion {
                out.push_str(&format!("     Suggestion: {}\n", s.green()));
            }
            out.push('\n');
        }
    }

    out.push_str("╚══════════════════════════════════════════════════════════════╝\n");
    out
}

/// Formats a critique report as JSON
pub fn format_critique_json(report: &CritiqueReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_default()
}
