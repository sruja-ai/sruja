use std::fs;
use std::path::Path;

use crate::commands::CliError;

pub(crate) fn should_fail_on_violations(
    fail_on: Option<&str>,
    violations: &[sruja_diff::Violation],
) -> bool {
    if let Some(criteria) = fail_on {
        let criteria_lower = criteria.to_lowercase();
        let criteria_list: Vec<&str> = criteria_lower.split(',').map(|s| s.trim()).collect();

        for criterion in criteria_list {
            match criterion {
                "all"
                    if violations
                        .iter()
                        .any(|v| matches!(v.severity, sruja_diff::Severity::Error)) =>
                {
                    return true;
                }
                "cycles" | "circular"
                    if violations.iter().any(|v| {
                        matches!(v.kind, sruja_diff::ViolationKind::CircularDependency)
                            && matches!(v.severity, sruja_diff::Severity::Error)
                    }) =>
                {
                    return true;
                }
                "layer-violations" | "layer"
                    if violations.iter().any(|v| {
                        matches!(v.kind, sruja_diff::ViolationKind::LayerViolation)
                            && matches!(v.severity, sruja_diff::Severity::Error)
                    }) =>
                {
                    return true;
                }
                "god-modules" | "god"
                    if violations.iter().any(|v| {
                        matches!(v.kind, sruja_diff::ViolationKind::GodModule)
                            && matches!(v.severity, sruja_diff::Severity::Error)
                    }) =>
                {
                    return true;
                }
                "orphans"
                    if violations.iter().any(|v| {
                        matches!(v.kind, sruja_diff::ViolationKind::OrphanComponent)
                            && matches!(v.severity, sruja_diff::Severity::Error)
                    }) =>
                {
                    return true;
                }
                _ => {}
            }
        }
    }
    false
}

pub(crate) fn truth_status_from_baseline_compare(
    scanned: &sruja_scan::Graph,
    baseline_path: &Path,
) -> Result<sruja_diff::TruthStatus, CliError> {
    let content = fs::read_to_string(baseline_path)?;
    let parser = sruja_language::Parser::new(baseline_path.to_string_lossy().as_ref());
    let program = parser.parse(&content).map_err(|diags| {
        CliError::parse_with_diagnostics(baseline_path.to_string_lossy().to_string(), diags)
    })?;
    let proposed_graph = sruja_diff::program_to_graph(&program);
    Ok(sruja_diff::compare_graphs(scanned, &proposed_graph).truth_status)
}
