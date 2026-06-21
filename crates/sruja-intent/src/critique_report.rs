//! Formatting for architectural critique reports

use crate::critique::{CritiqueCategory, CritiqueFinding, CritiqueReport, CritiqueSeverity};
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

    if report.violations.is_empty() && report.context.is_empty() {
        out.push_str("─ No findings.\n");
    } else {
        if !report.violations.is_empty() {
            render_findings_section(&mut out, "Violations", &report.violations);
        }
        if !report.context.is_empty() {
            render_findings_section(&mut out, "Context", &report.context);
        }
    }

    out.push_str("╚══════════════════════════════════════════════════════════════╝\n");
    out
}

fn render_findings_section(out: &mut String, header: &str, findings: &[CritiqueFinding]) {
    out.push_str(&format!(
        "┌─ {} ──────────────────────────────────────────────────\n",
        header.bold()
    ));
    let severities = [
        CritiqueSeverity::Critical,
        CritiqueSeverity::High,
        CritiqueSeverity::Medium,
        CritiqueSeverity::Low,
    ];

    for sev in severities {
        let items: Vec<_> = findings.iter().filter(|f| f.severity == sev).collect();
        if items.is_empty() {
            continue;
        }

        let (_label_text, color) = match sev {
            CritiqueSeverity::Critical => ("Critical", "Critical".bright_red().bold().to_string()),
            CritiqueSeverity::High => ("High", "High".bright_red().to_string()),
            CritiqueSeverity::Medium => ("Medium", "Medium".bright_yellow().to_string()),
            CritiqueSeverity::Low => ("Low", "Low".bright_blue().to_string()),
        };

        out.push_str(&format!(
            "── {} ─────────────────────────────────────────────────\n",
            color
        ));
        for f in items {
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
                category_label.bold(),
                f.title.bold()
            ));
            out.push_str(&format!("     Detail: {}\n", f.detail));
            if let Some(s) = &f.suggestion {
                out.push_str(&format!("     Suggestion: {}\n", s.bright_green()));
            }
            out.push('\n');
        }
    }
}

/// Formats a critique report as JSON
pub fn format_critique_json(report: &CritiqueReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_default()
}
