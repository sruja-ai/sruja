use sruja_diff::{Violation, ViolationKind, Severity};
use sruja_language::ast::Program;

/// Architecture health score result.
pub struct HealthScore {
    pub score: u8,
    pub deductions: Vec<Deduction>,
}

pub struct Deduction {
    pub message: String,
    pub points: u8,
}

/// Calculate health score from a list of violations and the source program.
pub fn calculate_health(violations: &[Violation], program: &Program) -> HealthScore {
    let mut score = 100i16;
    let mut deductions = Vec::new();

    // 1. Violations based deductions
    for v in violations {
        let (points, msg) = match (v.kind, v.severity) {
            (ViolationKind::CircularDependency, _) => (15, format!("Circular dependency: {}", v.message)),
            (ViolationKind::LayerViolation, Severity::Error) => (10, format!("Layer violation (Error): {}", v.message)),
            (ViolationKind::LayerViolation, Severity::Warning) => (5, format!("Layer violation (Warning): {}", v.message)),
            (ViolationKind::GodModule, _) => (10, format!("God module detected: {}", v.message)),
            (ViolationKind::OrphanComponent, _) => (5, format!("Orphan component: {}", v.message)),
            (_, Severity::Error) => (5, format!("Error: {}", v.message)),
            (_, Severity::Warning) => (2, format!("Warning: {}", v.message)),
            _ => (0, String::new()),
        };

        if points > 0 {
            score -= points;
            deductions.push(Deduction { message: msg, points: points as u8 });
        }
    }

    // 2. DSL completeness deductions (descriptions, SLOs)
    let (elements, _relations) = sruja_language::collect_elements(program);
    for (fqn, elem) in &elements {
        let body = elem.assignment.body.as_ref();
        
        // Missing description
        if body.and_then(|b| b.description.as_ref()).is_none() {
            score -= 2;
            deductions.push(Deduction { 
                message: format!("Missing description for {}", fqn), 
                points: 2 
            });
        }

        // Missing technology for containers
        if elem.assignment.kind.to_string() == "container" && body.and_then(|b| b.technology.as_ref()).is_none() {
            score -= 2;
            deductions.push(Deduction { 
                message: format!("Missing technology for container {}", fqn), 
                points: 2 
            });
        }
    }

    // Clamp score to 0-100
    let final_score = if score < 0 { 0 } else if score > 100 { 100 } else { score as u8 };

    HealthScore {
        score: final_score,
        deductions,
    }
}
