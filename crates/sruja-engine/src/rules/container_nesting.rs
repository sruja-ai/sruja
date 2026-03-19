//! Container/Component Nesting Rule
//!
//! Enforces that containers and components must be nested within a system.
//! This follows the C4 model hierarchy where:
//! - Systems contain containers
//! - Containers contain components
//! - Components cannot exist at top-level

use sruja_diagnostics::{Diagnostic, Severity};
use sruja_language::{ElementKind, Program};

use crate::validator::Rule;

pub struct ContainerNestingRule;

impl Rule for ContainerNestingRule {
    fn name(&self) -> &str {
        "Container Nesting"
    }

    fn validate(&self, program: &Program) -> Vec<Diagnostic> {
        let mut diags: Vec<Diagnostic> = Vec::new();

        for item in &program.items {
            if let sruja_language::TopLevelItem::ElementDef(elem) = item {
                let kind = &elem.assignment.kind;
                if requires_nesting(kind) {
                    diags.push(
                        Diagnostic::new(
                            sruja_diagnostics::codes::CODE_NESTING_VIOLATION,
                            Severity::Error,
                            format!(
                                "{} `{}` must be nested inside a system. Top-level {} declarations are not allowed.",
                                kind,
                                elem.assignment.name,
                                kind
                            ),
                            elem.assignment.location.clone(),
                        )
                        .with_suggestions(vec![format!(
                            "Move `{}` inside a system block:\n  MySystem = system \"...\" {{\n    {} = {} \"...\" {{ ... }}\n  }}",
                            elem.assignment.name,
                            elem.assignment.name,
                            kind
                        )]),
                    );
                }
            }
        }

        diags
    }
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
    use super::*;
    use sruja_language::Parser;

    fn validate(input: &str) -> Vec<Diagnostic> {
        let program = Parser::new("test.sruja".to_string())
            .parse(input)
            .expect("parse");
        ContainerNestingRule.validate(&program)
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
