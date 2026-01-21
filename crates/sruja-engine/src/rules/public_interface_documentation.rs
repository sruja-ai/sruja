//! Public interface documentation best-practice rule
//!
//! Mirrors Go `PublicInterfaceDocumentationRule`:
//! - Find all `person` IDs.
//! - Find all elements accessed by a person via relations (respecting nested scope).
//! - For accessed `system` and `container` elements:
//!   - warn if missing description
//!   - for containers, warn if missing technology

use std::collections::{HashMap, HashSet};

use sruja_diagnostics::{Diagnostic, Severity};
use sruja_language::{
    collect_elements, collect_relations_with_scope, build_qualified_id, ElementDef, ElementKind,
    Program,
};

use crate::validator::Rule;

pub struct PublicInterfaceDocumentationRule;

impl Rule for PublicInterfaceDocumentationRule {
    fn name(&self) -> &str {
        "Public Interface Documentation"
    }

    fn validate(&self, program: &Program) -> Vec<Diagnostic> {
        if program.items.is_empty() {
            return vec![];
        }

        let (elements, _relations) = collect_elements(program);

        // 1) Collect all person IDs (leaf IDs) from all elements.
        let mut person_ids: HashSet<String> = HashSet::new();
        for (_fqn, elem) in &elements {
            if elem.assignment.kind == ElementKind::Person {
                if !elem.assignment.name.is_empty() {
                    person_ids.insert(elem.assignment.name.clone());
                }
            }
        }

        if person_ids.is_empty() {
            return vec![];
        }

        // 2) Collect accessed items by scanning scoped relations for person -> target.
        let mut accessed: HashSet<String> = HashSet::new();
        let relations_with_scope = collect_relations_with_scope(program);

        for rs in relations_with_scope {
            let rel = rs.relation;
            let from_name = rel.from.as_string();
            if !person_ids.contains(&from_name) {
                continue;
            }

            let to_name = rel.to.as_string();
            // If relation is nested and target is unqualified, qualify it with scope.
            let full_to = if !rs.scope.is_empty() && !to_name.contains('.') {
                build_qualified_id(&rs.scope, &to_name)
            } else {
                to_name.clone()
            };

            accessed.insert(full_to);
            accessed.insert(to_name);
        }

        if accessed.is_empty() {
            return vec![];
        }

        // 3) Check accessed systems/containers for documentation.
        let mut diags: Vec<Diagnostic> = Vec::new();
        for target in accessed {
            if let Some(elem) = find_element(&elements, &target) {
                let kind = elem.assignment.kind;
                if kind != ElementKind::System && kind != ElementKind::Container {
                    continue;
                }

                let (desc, tech) = extract_desc_tech(elem);
                let id = elem.assignment.name.clone();

                if desc.trim().is_empty() {
                    let msg = match kind {
                        ElementKind::System => format!(
                            "Public API Documentation: System '{}' is used by humans but lacks a description.",
                            id
                        ),
                        ElementKind::Container => format!(
                            "Public Interface: Container '{}' is used by humans but lacks a description.",
                            id
                        ),
                        _ => unreachable!(),
                    };

                    diags.push(
                        Diagnostic::new(
                            sruja_diagnostics::codes::CODE_BEST_PRACTICE,
                            Severity::Warning,
                            msg,
                            elem.location.clone(),
                        )
                        .with_suggestions(vec![
                            format!(
                                "Add a description to {} '{}'",
                                kind.to_string(),
                                id
                            ),
                            "Public interfaces should be well-documented for API consumers".to_string(),
                        ]),
                    );
                }

                if kind == ElementKind::Container && tech.trim().is_empty() {
                    diags.push(
                        Diagnostic::new(
                            sruja_diagnostics::codes::CODE_BEST_PRACTICE,
                            Severity::Warning,
                            format!(
                                "Public Interface: Container '{}' is used by humans but lacks technology specification.",
                                id
                            ),
                            elem.location.clone(),
                        )
                        .with_suggestions(vec![
                            format!("Add technology to Container '{}' (e.g., 'Go', 'React')", id),
                            "Technology helps API consumers understand implementation details".to_string(),
                        ]),
                    );
                }
            }
        }

        diags
    }
}

fn find_element<'a>(elements: &'a HashMap<String, ElementDef>, name: &str) -> Option<&'a ElementDef> {
    if let Some(e) = elements.get(name) {
        return Some(e);
    }
    let suffix = format!(".{}", name);
    elements
        .iter()
        .find_map(|(k, e)| if k == name || k.ends_with(&suffix) { Some(e) } else { None })
}

fn extract_desc_tech(elem: &ElementDef) -> (String, String) {
    let mut desc = String::new();
    let mut tech = String::new();

    if let Some(body) = &elem.assignment.body {
        if let Some(d) = &body.description {
            desc = d.clone();
        }
        if let Some(t) = &body.technology {
            tech = t.clone();
        }

        // Fallback (parity): description/technology can also appear as items.
        if desc.is_empty() || tech.is_empty() {
            for item in &body.items {
                match item {
                    sruja_language::ElementDefBodyItem::Description(d) if desc.is_empty() => {
                        desc = d.clone();
                    }
                    sruja_language::ElementDefBodyItem::Technology(t) if tech.is_empty() => {
                        tech = t.clone();
                    }
                    _ => {}
                }
            }
        }
    }

    (desc, tech)
}

