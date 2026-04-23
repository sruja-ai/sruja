//! Container/Component Nesting Rule
//!
//! Enforces that containers and components must be nested within a system.
//! This follows the C4 model hierarchy where:
//! - Systems contain containers
//! - Containers contain components
//! - Components cannot exist at top-level

use crate::DomainSchema;
use sruja_diagnostics::{Diagnostic, Severity};
use sruja_language::{ElementKind, Program};

use crate::validator::Rule;

pub struct ContainerNestingRule;

impl Rule for ContainerNestingRule {
    fn name(&self) -> &str {
        "Container Nesting"
    }

    fn validate(&self, program: &Program, schema: &DomainSchema) -> Vec<Diagnostic> {
        let mut diags: Vec<Diagnostic> = Vec::new();

        for item in &program.items {
            if let sruja_language::TopLevelItem::ElementDef(elem) = item {
                let kind = &elem.assignment.kind;
                // If it's a top-level element, its "parent" kind is essentially "root" or None.
                // In our DomainSchema, we can represent this by checking if it's allowed at top level.
                // For now, we follow the logic that certain kinds MUST be nested.
                if requires_nesting(kind) {
                    diags.push(
                        Diagnostic::new(
                            sruja_diagnostics::codes::CODE_NESTING_VIOLATION,
                            Severity::Error,
                            format!(
                                "{} `{}` must be nested. Top-level {} declarations are not allowed in this schema.",
                                kind,
                                elem.assignment.name,
                                kind
                            ),
                            elem.assignment.location.clone(),
                        )
                        .with_suggestions(vec![format!(
                            "Move `{}` inside an allowed parent block.",
                            elem.assignment.name,
                        )]),
                    );
                }
                
                // Also check nested elements recursively
                if let Some(body) = &elem.assignment.body {
                    diags.append(&mut validate_nested_nesting(kind, &body.items, schema));
                }
            }
        }

        diags
    }
}

fn validate_nested_nesting(parent_kind: &sruja_language::ElementKind, items: &[sruja_language::ElementDefBodyItem], schema: &DomainSchema) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let parent_kind_str = parent_kind.to_string();

    for item in items {
        if let sruja_language::ElementDefBodyItem::ElementDef(elem) = item {
            let child_kind = &elem.assignment.kind;
            let child_kind_str = child_kind.to_string();

            if !schema.is_nesting_allowed(&parent_kind_str, &child_kind_str) {
                diags.push(
                    Diagnostic::new(
                        sruja_diagnostics::codes::CODE_NESTING_VIOLATION,
                        Severity::Error,
                        format!(
                            "{} `{}` cannot be nested inside {}. This nesting is not allowed in the current schema.",
                            child_kind,
                            elem.assignment.name,
                            parent_kind
                        ),
                        elem.assignment.location.clone(),
                    )
                );
            }

            // Recurse
            if let Some(body) = &elem.assignment.body {
                diags.append(&mut validate_nested_nesting(child_kind, &body.items, schema));
            }
        }
    }
    diags
}

fn requires_nesting(kind: &ElementKind) -> bool {
    matches!(
        kind,
        ElementKind::Container
            | ElementKind::Component
            | ElementKind::Database
            | ElementKind::Queue
            | ElementKind::DataStore
    )
}

#[cfg(test)]
mod tests {
    use crate::DomainSchema;
    use super::*;
    use sruja_language::Parser;

    fn validate(input: &str) -> Vec<Diagnostic> {
        let program = Parser::new("test.sruja".to_string())
            .parse(input)
            .expect("parse");
        ContainerNestingRule.validate(&program, &DomainSchema::architecture())
    }

    #[test]
    fn reports_top_level_container() {
        let diags = validate(
            r#"
system = kind "System"
container = kind "Container"

MyContainer = container "My Container" {
  technology "Rust"
  description "A container"
}
"#,
        );
        assert!(diags.iter().any(|d| d.message.contains("must be nested")));
    }

    #[test]
    fn reports_top_level_component() {
        let diags = validate(
            r#"
system = kind "System"
component = kind "Component"

MyComponent = component "My Component" {
  description "A component"
}
"#,
        );
        assert!(diags.iter().any(|d| d.message.contains("must be nested")));
    }

    #[test]
    fn allows_nested_container() {
        let diags = validate(
            r#"
system = kind "System"
container = kind "Container"

MySystem = system "My System" {
  description "A system"

  MyContainer = container "My Container" {
    technology "Rust"
    description "A container"
  }
}
"#,
        );
        assert!(!diags.iter().any(|d| d.message.contains("must be nested")));
    }

    #[test]
    fn allows_top_level_system() {
        let diags = validate(
            r#"
system = kind "System"

MySystem = system "My System" {
  description "A system"
}
"#,
        );
        assert!(!diags.iter().any(|d| d.message.contains("must be nested")));
    }

    #[test]
    fn allows_top_level_person() {
        let diags = validate(
            r#"
person = kind "Person"

User = person "User" {
  description "A user"
}
"#,
        );
        assert!(!diags.iter().any(|d| d.message.contains("must be nested")));
    }
}
