use sruja_diff::{Severity, Violation, ViolationKind};
use sruja_language::ast::Program;
use std::collections::HashMap;

/// Architecture health score result.
pub struct HealthScore {
    pub score: u8,
    pub deductions: Vec<Deduction>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Deduction {
    pub message: String,
    pub points: u8,
}

/// Calculate health score from a list of violations and the source program.
pub fn calculate_health(violations: &[Violation], program: &Program) -> HealthScore {
    let mut score = 100i16;
    let mut deductions = Vec::new();

    // 1. Violations based deductions (grouped and capped)
    let mut deductions_by_kind: HashMap<ViolationKind, Vec<(u8, String)>> = HashMap::new();

    for v in violations {
        let (points, msg) = match (v.kind, v.severity) {
            (ViolationKind::CircularDependency, _) => {
                (15, format!("Circular dependency: {}", v.message))
            }
            (ViolationKind::LayerViolation, Severity::Error) => {
                (10, format!("Layer violation (Error): {}", v.message))
            }
            (ViolationKind::LayerViolation, Severity::Warning) => {
                (5, format!("Layer violation (Warning): {}", v.message))
            }
            (ViolationKind::GodModule, _) => (10, format!("God module: {}", v.message)),
            (ViolationKind::OrphanComponent, _) => (5, format!("Orphan component: {}", v.message)),
            (_, Severity::Error) => (5, format!("Error: {}", v.message)),
            (_, Severity::Warning) => (2, format!("Warning: {}", v.message)),
            _ => (0, String::new()),
        };

        if points > 0 {
            deductions_by_kind
                .entry(v.kind)
                .or_default()
                .push((points, msg));
        }
    }

    // Apply capped deductions per kind
    for (kind, kind_deductions) in deductions_by_kind {
        let mut kind_points = 0;
        let mut kind_msgs = Vec::new();
        for (pts, msg) in kind_deductions {
            kind_points += pts as i16;
            kind_msgs.push(msg);
        }

        // Cap at 25 points per violation kind to avoid flooring to zero too quickly
        let capped_points = if kind_points > 25 {
            25
        } else {
            kind_points as u8
        };
        score -= capped_points as i16;

        let message = if kind_msgs.len() > 3 {
            format!(
                "{:?} ({} items, capped at {} pts)",
                kind,
                kind_msgs.len(),
                capped_points
            )
        } else {
            kind_msgs.join(", ")
        };

        deductions.push(Deduction {
            message,
            points: capped_points,
        });
    }

    // 2. DSL completeness deductions (descriptions, technology)
    let (elements, _relations) = sruja_language::collect_elements(program);
    let mut missing_desc = Vec::new();
    let mut missing_tech = Vec::new();

    for (fqn, elem) in &elements {
        let body = elem.assignment.body.as_ref();

        if body.and_then(|b| b.description.as_ref()).is_none() {
            missing_desc.push(fqn.clone());
        }

        if elem.assignment.kind.to_string() == "container"
            && body.and_then(|b| b.technology.as_ref()).is_none()
        {
            missing_tech.push(fqn.clone());
        }
    }

    if !missing_desc.is_empty() {
        let pts = (missing_desc.len() * 2).min(20) as u8;
        score -= pts as i16;
        deductions.push(Deduction {
            message: format!(
                "Missing descriptions ({} elements): {}",
                missing_desc.len(),
                missing_desc
                    .iter()
                    .take(3)
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            points: pts,
        });
    }

    if !missing_tech.is_empty() {
        let pts = (missing_tech.len() * 2).min(20) as u8;
        score -= pts as i16;
        deductions.push(Deduction {
            message: format!(
                "Missing technology for containers ({} elements): {}",
                missing_tech.len(),
                missing_tech
                    .iter()
                    .take(3)
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            points: pts,
        });
    }

    // Clamp score to 0-100
    let final_score = if score < 0 {
        0
    } else if score > 100 {
        100
    } else {
        score as u8
    };

    HealthScore {
        score: final_score,
        deductions,
    }
}
