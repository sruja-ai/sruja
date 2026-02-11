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
    build_qualified_id, collect_elements, collect_relations_with_scope, ElementDef, ElementKind,
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
        for elem in elements.values() {
            if elem.assignment.kind == ElementKind::Person && !elem.assignment.name.is_empty() {
                person_ids.insert(elem.assignment.name.clone());
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
                let kind = elem.assignment.kind.clone();
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
                            format!("Add a description to {} '{}'", kind.to_string(), id),
                            "Public interfaces should be well-documented for API consumers"
                                .to_string(),
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

fn find_element<'a>(
    elements: &'a HashMap<String, ElementDef>,
    name: &str,
) -> Option<&'a ElementDef> {
    if let Some(e) = elements.get(name) {
        return Some(e);
    }
    let suffix = format!(".{}", name);
    elements.iter().find_map(|(k, e)| {
        if k == name || k.ends_with(&suffix) {
            Some(e)
        } else {
            None
        }
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_language::Parser;

    fn validate_program(input: &str) -> Vec<Diagnostic> {
        let parser = Parser::new("test.sruja".to_string());
        let program = match parser.parse(input) {
            Ok(p) => p,
            Err(_) => return vec![],
        };
        let rule = PublicInterfaceDocumentationRule;
        rule.validate(&program)
    }

    #[test]
    fn empty_program_returns_no_diagnostics() {
        let diags = validate_program("");
        assert!(diags.is_empty());
    }

    #[test]
    fn no_persons_returns_no_diagnostics() {
        let input = r#"
api = system "API"
db = container "DB"
api -> db "uses"
"#;
        let diags = validate_program(input);
        assert!(diags.is_empty());
    }

    #[test]
    fn person_uses_system_with_description_passes() {
        let input = r#"
user = person "User"
api = system "API" {
    description "Main API"
}
user -> api "uses"
"#;
        let diags = validate_program(input);
        assert!(diags.is_empty());
    }

    #[test]
    fn person_uses_system_without_description_warns() {
        let input = r#"
user = person "User"
api = system "API"

user -> api "uses"
"#;
        let diags = validate_program(input);
        assert!(!diags.is_empty());
        assert!(diags.iter().any(|d| d.message.contains("lacks a description")));
    }

    #[test]
    fn person_uses_container_without_technology_warns() {
        let input = r#"
user = person "User"
web = container "Web App" {
    description "Frontend"
}
user -> web "uses"
"#;
        let diags = validate_program(input);
        assert!(!diags.is_empty());
        assert!(diags.iter().any(|d| d.message.contains("technology")));
    }

    #[test]
    fn person_uses_container_with_tech_and_desc_passes() {
        let input = r#"
user = person "User"
web = container "Web App" {
    description "Frontend"
    technology "React"
}
user -> web "uses"
"#;
        let diags = validate_program(input);
        assert!(diags.is_empty());
    }
}
