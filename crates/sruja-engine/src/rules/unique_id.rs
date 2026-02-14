//! Unique ID validation rule
//!
//! Ensures all element IDs are unique within architecture.

use std::collections::HashMap;

use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
use sruja_language::{ElementDefBodyItem, Program, TopLevelItem};

use crate::validator::Rule;

/// Rule that checks for duplicate element IDs
pub struct UniqueIdRule;

impl Rule for UniqueIdRule {
    fn name(&self) -> &str {
        "Unique IDs"
    }

    fn validate(&self, program: &Program) -> Vec<Diagnostic> {
        let mut seen_ids: HashMap<String, SourceLocation> = HashMap::with_capacity(100);
        let mut diagnostics = Vec::new();

        // Helper to check if an ID is duplicate
        let mut check_id = |id: &str, loc: &SourceLocation| {
            if id.is_empty() {
                return;
            }

            if let Some(existing) = seen_ids.get(id) {
                let msg = format!(
                    "Duplicate identifier '{}'. First defined at line {}:{}",
                    id, existing.line, existing.column
                );

                let suggestions = vec![
                    format!(
                        "Rename this element to a unique identifier (e.g., '{}2' or '{}_v2')",
                        id, id
                    ),
                    "Element IDs must be unique within the architecture".to_string(),
                ];

                diagnostics.push(
                    Diagnostic::new(
                        sruja_diagnostics::codes::CODE_DUPLICATE_ID,
                        Severity::Error,
                        msg,
                        loc.clone(),
                    )
                    .with_suggestions(suggestions),
                );
            } else {
                seen_ids.insert(id.to_string(), loc.clone());
            }
        };

        // Check top-level elements for duplicates
        for item in &program.items {
            if let TopLevelItem::ElementDef(elem) = item {
                check_id(&elem.assignment.name, &elem.location);

                // Check nested elements
                if let Some(body) = &elem.assignment.body {
                    check_nested_elements(body, &elem.assignment.name, &mut check_id);
                }
            }
        }

        diagnostics
    }
}

/// Recursively check nested elements for duplicate IDs
fn check_nested_elements<F>(
    body: &sruja_language::ElementDefBody,
    parent_fqn: &str,
    check_id: &mut F,
) where
    F: FnMut(&str, &SourceLocation),
{
    for item in &body.items {
        if let ElementDefBodyItem::ElementDef(elem) = item {
            let fqn = format!("{}.{}", parent_fqn, elem.assignment.name);
            check_id(&fqn, &elem.location);

            // Recursively check deeper nested elements
            if let Some(nested_body) = &elem.assignment.body {
                check_nested_elements(nested_body, &fqn, check_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_diagnostics::SourceLocation;
    use sruja_language::ElementDefBodyItem;
    use sruja_language::ElementKind;
    use sruja_language::{ElementAssignment, ElementDef, ElementDefBody, TopLevelItem};

    fn create_element(name: &str, line: u32, column: u32) -> ElementDef {
        ElementDef {
            location: SourceLocation::new("test.sruja".to_string(), line, column),
            assignment: ElementAssignment {
                location: SourceLocation::new("test.sruja".to_string(), line, column),
                name: name.to_string(),
                kind: ElementKind::System,
                sub_kind: None,
                title: None,
                tag_refs: vec![],
                body: None,
            },
        }
    }

    #[test]
    fn test_unique_id_no_duplicates() {
        let program = Program {
            items: vec![
                TopLevelItem::ElementDef(Box::new(create_element("SystemA", 1, 1))),
                TopLevelItem::ElementDef(Box::new(create_element("SystemB", 5, 1))),
                TopLevelItem::ElementDef(Box::new(create_element("SystemC", 10, 1))),
            ],
        };

        let rule = UniqueIdRule;
        let diagnostics = rule.validate(&program);

        assert!(
            diagnostics.is_empty(),
            "Should not report any diagnostics for unique IDs"
        );
    }

    #[test]
    fn test_unique_id_duplicate_top_level() {
        let program = Program {
            items: vec![
                TopLevelItem::ElementDef(Box::new(create_element("SystemA", 1, 1))),
                TopLevelItem::ElementDef(Box::new(create_element("SystemA", 5, 1))),
                TopLevelItem::ElementDef(Box::new(create_element("SystemB", 10, 1))),
            ],
        };

        let rule = UniqueIdRule;
        let diagnostics = rule.validate(&program);

        assert_eq!(diagnostics.len(), 1, "Should report one duplicate");
        assert_eq!(
            diagnostics[0].code,
            sruja_diagnostics::codes::CODE_DUPLICATE_ID
        );
        assert_eq!(diagnostics[0].severity, Severity::Error);
        assert!(diagnostics[0]
            .message
            .contains("Duplicate identifier 'SystemA'"));
        assert!(diagnostics[0].message.contains("First defined at line 1:1"));
    }

    #[test]
    fn test_unique_id_duplicate_nested() {
        let nested_elem1 = ElementDef {
            location: SourceLocation::new("test.sruja".to_string(), 3, 5),
            assignment: ElementAssignment {
                location: SourceLocation::new("test.sruja".to_string(), 3, 5),
                name: "ComponentX".to_string(),
                kind: ElementKind::Component,
                sub_kind: None,
                title: None,
                tag_refs: vec![],
                body: None,
            },
        };

        let nested_elem2 = ElementDef {
            location: SourceLocation::new("test.sruja".to_string(), 8, 5),
            assignment: ElementAssignment {
                location: SourceLocation::new("test.sruja".to_string(), 8, 5),
                name: "ComponentX".to_string(),
                kind: ElementKind::Component,
                sub_kind: None,
                title: None,
                tag_refs: vec![],
                body: None,
            },
        };

        let container = ElementDef {
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            assignment: ElementAssignment {
                location: SourceLocation::new("test.sruja".to_string(), 1, 1),
                name: "ContainerA".to_string(),
                kind: ElementKind::Container,
                sub_kind: None,
                title: None,
                tag_refs: vec![],
                body: Some(ElementDefBody {
                    items: vec![
                        ElementDefBodyItem::ElementDef(Box::new(nested_elem1)),
                        ElementDefBodyItem::ElementDef(Box::new(nested_elem2)),
                    ],
                    ..Default::default()
                }),
            },
        };

        let program = Program {
            items: vec![TopLevelItem::ElementDef(Box::new(container))],
        };

        let rule = UniqueIdRule;
        let diagnostics = rule.validate(&program);

        assert_eq!(
            diagnostics.len(),
            1,
            "Should report one duplicate nested ID"
        );
        assert!(diagnostics[0].message.contains("ContainerA.ComponentX"));
        assert!(diagnostics[0].message.contains("First defined at line 3:5"));
    }

    #[test]
    fn test_unique_id_mixed_duplicates() {
        let nested_elem1 = ElementDef {
            location: SourceLocation::new("test.sruja".to_string(), 3, 5),
            assignment: ElementAssignment {
                location: SourceLocation::new("test.sruja".to_string(), 3, 5),
                name: "ContainerA".to_string(),
                kind: ElementKind::Container,
                sub_kind: None,
                title: None,
                tag_refs: vec![],
                body: None,
            },
        };

        let system1 = ElementDef {
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            assignment: ElementAssignment {
                location: SourceLocation::new("test.sruja".to_string(), 1, 1),
                name: "SystemA".to_string(),
                kind: ElementKind::System,
                sub_kind: None,
                title: None,
                tag_refs: vec![],
                body: Some(ElementDefBody {
                    items: vec![ElementDefBodyItem::ElementDef(Box::new(nested_elem1))],
                    ..Default::default()
                }),
            },
        };

        let system2 = ElementDef {
            location: SourceLocation::new("test.sruja".to_string(), 8, 1),
            assignment: ElementAssignment {
                location: SourceLocation::new("test.sruja".to_string(), 8, 1),
                name: "SystemA".to_string(),
                kind: ElementKind::System,
                sub_kind: None,
                title: None,
                tag_refs: vec![],
                body: None,
            },
        };

        let program = Program {
            items: vec![
                TopLevelItem::ElementDef(Box::new(system1)),
                TopLevelItem::ElementDef(Box::new(create_element("SystemB", 5, 1))),
                TopLevelItem::ElementDef(Box::new(system2)),
            ],
        };

        let rule = UniqueIdRule;
        let diagnostics = rule.validate(&program);

        assert_eq!(diagnostics.len(), 1, "Should report one duplicate");
        assert!(diagnostics[0]
            .message
            .contains("Duplicate identifier 'SystemA'"));
    }

    #[test]
    fn test_unique_id_empty_name_ignored() {
        let elem = ElementDef {
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            assignment: ElementAssignment {
                location: SourceLocation::new("test.sruja".to_string(), 1, 1),
                name: "".to_string(),
                kind: ElementKind::System,
                sub_kind: None,
                title: None,
                tag_refs: vec![],
                body: None,
            },
        };

        let program = Program {
            items: vec![TopLevelItem::ElementDef(Box::new(elem))],
        };

        let rule = UniqueIdRule;
        let diagnostics = rule.validate(&program);

        assert!(
            diagnostics.is_empty(),
            "Should not report diagnostics for empty names"
        );
    }

    #[test]
    fn test_unique_id_multiple_duplicates() {
        let program = Program {
            items: vec![
                TopLevelItem::ElementDef(Box::new(create_element("SystemA", 1, 1))),
                TopLevelItem::ElementDef(Box::new(create_element("SystemA", 3, 1))),
                TopLevelItem::ElementDef(Box::new(create_element("SystemA", 5, 1))),
                TopLevelItem::ElementDef(Box::new(create_element("SystemB", 7, 1))),
                TopLevelItem::ElementDef(Box::new(create_element("SystemB", 9, 1))),
            ],
        };

        let rule = UniqueIdRule;
        let diagnostics = rule.validate(&program);

        assert_eq!(
            diagnostics.len(),
            3,
            "Should report three duplicates (2 for SystemA, 1 for SystemB)"
        );

        let system_a_duplicates: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.message.contains("SystemA"))
            .collect();
        assert_eq!(
            system_a_duplicates.len(),
            2,
            "Should report 2 duplicates for SystemA"
        );

        let system_b_duplicates: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.message.contains("SystemB"))
            .collect();
        assert_eq!(
            system_b_duplicates.len(),
            1,
            "Should report 1 duplicate for SystemB"
        );
    }

    #[test]
    fn test_unique_id_rule_name() {
        let rule = UniqueIdRule;
        assert_eq!(rule.name(), "Unique IDs");
    }

    #[test]
    fn test_unique_id_with_suggestions() {
        let program = Program {
            items: vec![
                TopLevelItem::ElementDef(Box::new(create_element("MySystem", 1, 1))),
                TopLevelItem::ElementDef(Box::new(create_element("MySystem", 5, 1))),
            ],
        };

        let rule = UniqueIdRule;
        let diagnostics = rule.validate(&program);

        assert_eq!(diagnostics.len(), 1);
        assert!(
            !diagnostics[0].suggestions.is_empty(),
            "Should have suggestions"
        );
        assert!(diagnostics[0]
            .suggestions
            .iter()
            .any(|s| s.contains("MySystem2")));
        assert!(diagnostics[0]
            .suggestions
            .iter()
            .any(|s| s.contains("MySystem_v2")));
    }

    #[test]
    fn test_unique_id_deeply_nested_no_duplicates() {
        let component1 = ElementDef {
            location: SourceLocation::new("test.sruja".to_string(), 7, 13),
            assignment: ElementAssignment {
                location: SourceLocation::new("test.sruja".to_string(), 7, 13),
                name: "Database".to_string(),
                kind: ElementKind::Database,
                sub_kind: None,
                title: None,
                tag_refs: vec![],
                body: None,
            },
        };

        let component2 = ElementDef {
            location: SourceLocation::new("test.sruja".to_string(), 9, 13),
            assignment: ElementAssignment {
                location: SourceLocation::new("test.sruja".to_string(), 9, 13),
                name: "Database".to_string(),
                kind: ElementKind::Database,
                sub_kind: None,
                title: None,
                tag_refs: vec![],
                body: None,
            },
        };

        let container1 = ElementDef {
            location: SourceLocation::new("test.sruja".to_string(), 3, 5),
            assignment: ElementAssignment {
                location: SourceLocation::new("test.sruja".to_string(), 3, 5),
                name: "API".to_string(),
                kind: ElementKind::Container,
                sub_kind: None,
                title: None,
                tag_refs: vec![],
                body: Some(ElementDefBody {
                    items: vec![ElementDefBodyItem::ElementDef(Box::new(component1))],
                    ..Default::default()
                }),
            },
        };

        let container2 = ElementDef {
            location: SourceLocation::new("test.sruja".to_string(), 5, 5),
            assignment: ElementAssignment {
                location: SourceLocation::new("test.sruja".to_string(), 5, 5),
                name: "Worker".to_string(),
                kind: ElementKind::Container,
                sub_kind: None,
                title: None,
                tag_refs: vec![],
                body: Some(ElementDefBody {
                    items: vec![ElementDefBodyItem::ElementDef(Box::new(component2))],
                    ..Default::default()
                }),
            },
        };

        let system = ElementDef {
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            assignment: ElementAssignment {
                location: SourceLocation::new("test.sruja".to_string(), 1, 1),
                name: "MainSystem".to_string(),
                kind: ElementKind::System,
                sub_kind: None,
                title: None,
                tag_refs: vec![],
                body: Some(ElementDefBody {
                    items: vec![
                        ElementDefBodyItem::ElementDef(Box::new(container1)),
                        ElementDefBodyItem::ElementDef(Box::new(container2)),
                    ],
                    ..Default::default()
                }),
            },
        };

        let program = Program {
            items: vec![TopLevelItem::ElementDef(Box::new(system))],
        };

        let rule = UniqueIdRule;
        let diagnostics = rule.validate(&program);

        // These elements have different FQNs (MainSystem.API.Database vs MainSystem.Worker.Database)
        // so they should NOT be reported as duplicates
        assert_eq!(
            diagnostics.len(),
            0,
            "Should not report duplicate for elements with different FQNs"
        );
    }
}
