//! Database isolation best-practice rule
//!
//! Mirrors Go `DatabaseIsolationRule`:
//! - Detects "integration via database" anti-pattern where multiple services/systems
//!   access the same database.
//! - Allows shared databases when `metadata { shared "true" }` is set on the database.

use std::collections::{HashMap, HashSet};

use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
use sruja_language::{collect_elements, ElementDef, ElementKind, Program};

use crate::validator::Rule;

pub struct DatabaseIsolationRule;

impl Rule for DatabaseIsolationRule {
    fn name(&self) -> &str {
        "Database Isolation"
    }

    fn validate(&self, program: &Program) -> Vec<Diagnostic> {
        if program.items.is_empty() {
            return vec![];
        }

        let (elements, relations) = collect_elements(program);

        // Map: database FQN -> distinct consumer roots
        let mut db_usage: HashMap<String, HashSet<String>> = HashMap::new();

        for rel in &relations {
            let target_name = rel.to.as_string();
            if !is_target_database(&elements, &target_name) {
                continue;
            }

            let source_name = rel.from.as_string();
            let source_root = source_name
                .split('.')
                .next()
                .unwrap_or(&source_name)
                .to_string();

            db_usage
                .entry(target_name.clone())
                .or_default()
                .insert(source_root);
        }

        let mut diags: Vec<Diagnostic> = Vec::new();

        for (db_id, consumers) in db_usage {
            if consumers.len() <= 1 {
                continue;
            }

            if is_shared_database(&elements, &db_id) {
                continue;
            }

            let mut consumers_list: Vec<String> = consumers.into_iter().collect();
            consumers_list.sort();

            diags.push(
                Diagnostic::new(
                    sruja_diagnostics::codes::CODE_BEST_PRACTICE,
                    Severity::Warning,
                    format!(
                        "Best Practice Violation: DataStore '{}' is accessed by multiple services ({}). Prefer Database-Per-Service pattern.",
                        db_id,
                        consumers_list.join(", ")
                    ),
                    // Go uses a generic location when it can't find definition; we try to find it.
                    find_element_location(&elements, &db_id).unwrap_or_else(|| {
                        SourceLocation::new(String::from("architecture"), 0, 0)
                    }),
                )
                .with_context(vec![format!("Accessed by: {}", consumers_list.join(", "))])
                .with_suggestions(vec![
                    "Consider splitting the DataStore into service-specific databases".to_string(),
                    "Use the Database-Per-Service pattern for better service isolation".to_string(),
                    "If shared data is required, mark with `metadata { shared \"true\" }` and document the rationale".to_string(),
                ]),
            );
        }

        diags
    }
}

fn is_target_database(elements: &HashMap<String, ElementDef>, name: &str) -> bool {
    find_element(elements, name).is_some_and(|e| e.assignment.kind == ElementKind::Database)
}

fn is_shared_database(elements: &HashMap<String, ElementDef>, name: &str) -> bool {
    let Some(e) = find_element(elements, name) else {
        return false;
    };
    let Some(body) = &e.assignment.body else {
        return false;
    };
    body.metadata.iter().any(|m| {
        m.key == "shared"
            && m.value
                .as_deref()
                .map(|v| v == "true" || v == "\"true\"" || v.trim_matches('"') == "true")
                .unwrap_or(false)
    })
}

fn find_element_location(
    elements: &HashMap<String, ElementDef>,
    name: &str,
) -> Option<SourceLocation> {
    find_element(elements, name).map(|e| e.location.clone())
}

fn find_element<'a>(
    elements: &'a HashMap<String, ElementDef>,
    name: &str,
) -> Option<&'a ElementDef> {
    if let Some(e) = elements.get(name) {
        return Some(e);
    }
    let suffix = format!(".{}", name);
    elements.iter().find_map(|(fqn, e)| {
        if fqn == name || fqn.ends_with(&suffix) {
            Some(e)
        } else {
            None
        }
    })
}
