//! AST traversal helpers
//!
//! This module provides helper functions for traversing the AST and collecting
//! elements, relations, and other structures needed for validation and processing.

use std::collections::HashMap;

use crate::ast::*;
use sruja_diagnostics::SourceLocation;

/// A relation paired with the scope (parent FQN) it was declared within.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationWithScope {
    pub relation: Relation,
    /// Parent FQN for nested relations; empty for top-level.
    pub scope: String,
}

/// Collect all element definitions from a program keyed by fully qualified name (FQN)
/// Returns a map of FQN -> ElementDef and a vector of all relations
///
/// This is a two-pass approach:
/// 1. First pass: Collect all elements with their FQDNs
/// 2. Second pass: Resolve relation endpoints to FQDNs based on scope
pub fn collect_elements(program: &Program) -> (HashMap<String, ElementDef>, Vec<Relation>) {
    let estimated_elements = program.items.len() * 4;
    let capacity = estimated_elements.max(16);

    let mut elements: HashMap<String, ElementDef> = HashMap::with_capacity(capacity);
    let mut relations_with_scope: Vec<(Relation, String)> = Vec::with_capacity(capacity / 2);

    // Use iterative traversal with explicit stack
    struct Frame {
        elem: ElementDef,
        parent: String,
    }

    let mut stack: Vec<Frame> = Vec::new();

    // Initialize with top-level elements
    for item in &program.items {
        match item {
            TopLevelItem::ElementDef(elem) => {
                stack.push(Frame {
                    elem: (**elem).clone(),
                    parent: String::new(),
                });
            }
            TopLevelItem::Relation(rel) => {
                relations_with_scope.push((rel.clone(), String::new()));
            }
            TopLevelItem::Requirement(req) => {
                let elem_def = ElementDef {
                    location: req.location.clone(),
                    assignment: ElementAssignment {
                        location: req.location.clone(),
                        name: req.id.clone(),
                        kind: ElementKind::Requirement,
                        sub_kind: Some(req.r#type.clone()),
                        title: Some(req.title.clone()),
                        tag_refs: req.tags.clone(),
                        body: None,
                    },
                };
                let fqn = req.id.clone();
                elements.insert(fqn, elem_def);
            }
            _ => {}
        }
    }

    // First pass: Collect all elements (iterative traversal)
    while let Some(frame) = stack.pop() {
        let elem = frame.elem;
        let id = elem.assignment.name.clone();

        if id.is_empty() {
            continue;
        }

        let fqn = build_qualified_id(&frame.parent, &id);
        elements.insert(fqn.clone(), elem.clone());

        // Process body items
        if let Some(body) = &elem.assignment.body {
            for item in &body.items {
                match item {
                    ElementDefBodyItem::ElementDef(nested_elem) => {
                        stack.push(Frame {
                            elem: (**nested_elem).clone(),
                            parent: fqn.clone(),
                        });
                    }
                    ElementDefBodyItem::Relation(rel) => {
                        relations_with_scope.push((rel.clone(), fqn.clone()));
                    }
                    _ => {}
                }
            }
        }
    }

    // Second pass: Resolve relation endpoints to FQDNs
    let relations: Vec<Relation> = relations_with_scope
        .into_iter()
        .map(|(rel, scope)| resolve_relation_fqns(rel, &scope, &elements))
        .collect();

    (elements, relations)
}

/// Build a fully qualified identifier from parent and child
pub fn build_qualified_id(parent: &str, id: &str) -> String {
    if parent.is_empty() {
        id.to_string()
    } else {
        format!("{}.{}", parent, id)
    }
}

/// Collect all relations from a program (no scope information).
///
/// All relation endpoints are resolved to fully qualified names.
pub fn collect_all_relations(program: &Program) -> Vec<Relation> {
    let (_elements, relations) = collect_elements(program);
    relations
}

/// Collect all relations from a program with their scope (parent FQN).
///
/// This mirrors the Go engine behavior where nested relations inherit scope.
pub fn collect_relations_with_scope(program: &Program) -> Vec<RelationWithScope> {
    let mut out: Vec<RelationWithScope> = Vec::new();

    // Use iterative traversal with explicit stack.
    #[derive(Clone)]
    struct Frame {
        elem: ElementDef,
        parent: String,
    }

    let mut stack: Vec<Frame> = Vec::new();

    // Top-level items
    for item in &program.items {
        match item {
            TopLevelItem::Relation(rel) => out.push(RelationWithScope {
                relation: rel.clone(),
                scope: String::new(),
            }),
            TopLevelItem::ElementDef(elem) => stack.push(Frame {
                elem: (**elem).clone(),
                parent: String::new(),
            }),
            _ => {}
        }
    }

    while let Some(frame) = stack.pop() {
        let elem = frame.elem;
        let id = elem.assignment.name.clone();
        if id.is_empty() {
            continue;
        }

        let fqn = build_qualified_id(&frame.parent, &id);

        if let Some(body) = &elem.assignment.body {
            for item in &body.items {
                match item {
                    ElementDefBodyItem::Relation(rel) => out.push(RelationWithScope {
                        relation: rel.clone(),
                        scope: fqn.clone(),
                    }),
                    ElementDefBodyItem::ElementDef(nested) => stack.push(Frame {
                        elem: (**nested).clone(),
                        parent: fqn.clone(),
                    }),
                    _ => {}
                }
            }
        }
    }

    out
}

/// Resolve relation endpoints to fully qualified names based on scope
///
/// When a relation is defined inside an element body, the endpoint names
/// are relative to that scope. This function resolves them to FQDNs by:
/// 1. Checking if the name already exists as a fully qualified name
/// 2. Checking if the name exists in the elements map
/// 3. Prepending the scope if neither match, and checking if the scoped name exists
/// 4. Returning the original name if no match is found (for external references)
pub fn resolve_relation_fqns(
    rel: Relation,
    scope: &str,
    elements: &HashMap<String, ElementDef>,
) -> Relation {
    let resolve = |ident: &QualifiedIdent| -> String {
        let name = ident.as_string();

        // If it's already a fully qualified name that exists, use it
        if elements.contains_key(&name) {
            return name;
        }

        // If it's a simple name that exists at the top level, use it
        if !name.contains('.') && elements.contains_key(&name) {
            return name;
        }

        // Try resolving with the scope
        if !scope.is_empty() {
            let scoped = format!("{}.{}", scope, name);
            if elements.contains_key(&scoped) {
                return scoped;
            }
        }

        // Return the original name (might be an external reference or error)
        name
    };

    let from_fqn = resolve(&rel.from);
    let to_fqn = resolve(&rel.to);

    Relation {
        from: QualifiedIdent::qualified(from_fqn.split('.').map(|s| s.to_string()).collect()),
        to: QualifiedIdent::qualified(to_fqn.split('.').map(|s| s.to_string()).collect()),
        ..rel
    }
}

/// Get element location from various AST types
pub trait HasLocation {
    fn location(&self) -> &SourceLocation;
}

impl HasLocation for ElementDef {
    fn location(&self) -> &SourceLocation {
        &self.location
    }
}

impl HasLocation for Relation {
    fn location(&self) -> &SourceLocation {
        &self.location
    }
}

impl HasLocation for ImportStatement {
    fn location(&self) -> &SourceLocation {
        &self.location
    }
}

impl HasLocation for Scenario {
    fn location(&self) -> &SourceLocation {
        &self.location
    }
}

impl HasLocation for Flow {
    fn location(&self) -> &SourceLocation {
        &self.location
    }
}

impl HasLocation for Requirement {
    fn location(&self) -> &SourceLocation {
        &self.location
    }
}

impl HasLocation for Adr {
    fn location(&self) -> &SourceLocation {
        &self.location
    }
}

impl HasLocation for Policy {
    fn location(&self) -> &SourceLocation {
        &self.location
    }
}

impl HasLocation for ViewDef {
    fn location(&self) -> &SourceLocation {
        &self.location
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    fn parse_test_input(input: &str) -> Program {
        let parser = Parser::new("test.sruja".to_string());
        parser.parse(input).expect("Should parse successfully")
    }

    #[test]
    fn test_build_qualified_id_top_level() {
        let result = build_qualified_id("", "SystemA");
        assert_eq!(result, "SystemA");
    }

    #[test]
    fn test_build_qualified_id_nested() {
        let result = build_qualified_id("SystemA", "ContainerB");
        assert_eq!(result, "SystemA.ContainerB");
    }

    #[test]
    fn test_build_qualified_id_deeply_nested() {
        let result = build_qualified_id("SystemA.ContainerB", "ComponentC");
        assert_eq!(result, "SystemA.ContainerB.ComponentC");
    }

    #[test]
    fn test_collect_elements_simple_system() {
        let input = r#"
A = system "System A" {
    description "A test system"
}
"#;
        let program = parse_test_input(input);
        let (elements, relations) = collect_elements(&program);

        assert_eq!(elements.len(), 1);
        assert!(elements.contains_key("A"));
        assert_eq!(relations.len(), 0);
    }

    #[test]
    fn test_collect_elements_nested_elements() {
        let input = r#"
A = system "System A" {
    B = container "Container B" {
        C = component "Component C"
    }
}
"#;
        let program = parse_test_input(input);
        let (elements, relations) = collect_elements(&program);

        assert_eq!(elements.len(), 3);
        assert!(elements.contains_key("A"));
        assert!(elements.contains_key("A.B"));
        assert!(elements.contains_key("A.B.C"));
        assert_eq!(relations.len(), 0);
    }

    #[test]
    fn test_collect_elements_with_relations() {
        let input = r#"
A = system "System A"
B = system "System B"
A -> B "calls"
"#;
        let program = parse_test_input(input);
        let (elements, relations) = collect_elements(&program);

        assert_eq!(elements.len(), 2);
        assert_eq!(relations.len(), 1);
        assert_eq!(relations[0].from.as_string(), "A");
        assert_eq!(relations[0].to.as_string(), "B");
    }

    #[test]
    fn test_collect_elements_nested_relations() {
        let input = r#"
A = system "System A" {
    B = container "Container B" {
        C = component "Component C"
    }
    D = container "Container D"
    B -> D "calls"
}
"#;
        let program = parse_test_input(input);
        let (elements, relations) = collect_elements(&program);

        assert_eq!(elements.len(), 4);
        assert_eq!(relations.len(), 1);
        assert_eq!(relations[0].from.as_string(), "A.B");
        assert_eq!(relations[0].to.as_string(), "A.D");
    }

    #[test]
    fn test_collect_elements_with_requirements() {
        let input = r#"
requirement REQ-001 "User Authentication" {
    type "functional"
}
"#;
        let program = parse_test_input(input);
        let (elements, relations) = collect_elements(&program);

        assert_eq!(elements.len(), 1);
        assert!(elements.contains_key("REQ-001"));
        assert_eq!(relations.len(), 0);
    }

    #[test]
    fn test_collect_all_relations() {
        let input = r#"
system A "System A"
system B "System B"
A -> B "relation1"
B -> A "relation2"
"#;
        let program = parse_test_input(input);
        let relations = collect_all_relations(&program);

        assert_eq!(relations.len(), 2);
        assert_eq!(relations[0].label.as_deref(), Some("relation1"));
        assert_eq!(relations[1].label.as_deref(), Some("relation2"));
    }

    #[test]
    fn test_collect_relations_with_scope_top_level() {
        let input = r#"
A = system "System A"
B = system "System B"
A -> B "calls"
"#;
        let program = parse_test_input(input);
        let relations_with_scope = collect_relations_with_scope(&program);

        assert_eq!(relations_with_scope.len(), 1);
        assert!(relations_with_scope[0].scope.is_empty());
        assert_eq!(relations_with_scope[0].relation.from.as_string(), "A");
        assert_eq!(relations_with_scope[0].relation.to.as_string(), "B");
    }

    #[test]
    fn test_collect_relations_with_scope_nested() {
        let input = r#"
A = system "System A" {
    B = container "Container B"
    C = container "Container C"
    B -> C "calls"
}
"#;
        let program = parse_test_input(input);
        let relations_with_scope = collect_relations_with_scope(&program);

        assert_eq!(relations_with_scope.len(), 1);
        assert_eq!(relations_with_scope[0].scope, "A");
        assert_eq!(relations_with_scope[0].relation.from.as_string(), "B");
        assert_eq!(relations_with_scope[0].relation.to.as_string(), "C");
    }

    #[test]
    fn test_resolve_relation_fqns_simple() {
        let input = r#"
system A "System A"
system B "System B"
A -> B "calls"
"#;
        let program = parse_test_input(input);
        let (elements, _) = collect_elements(&program);

        let rel = Relation {
            from: QualifiedIdent::simple("A".to_string()),
            to: QualifiedIdent::simple("B".to_string()),
            label: Some("calls".to_string()),
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            description: None,
            technology: None,
            tags: vec![],
        };

        let resolved = resolve_relation_fqns(rel, "", &elements);

        assert_eq!(resolved.from.as_string(), "A");
        assert_eq!(resolved.to.as_string(), "B");
    }

    #[test]
    fn test_resolve_relation_fqns_nested() {
        let input = r#"
A = system "System A" {
    B = container "Container B"
    C = container "Container C"
}
"#;
        let program = parse_test_input(input);
        let (elements, _) = collect_elements(&program);

        let rel = Relation {
            from: QualifiedIdent::simple("B".to_string()),
            to: QualifiedIdent::simple("C".to_string()),
            label: Some("calls".to_string()),
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            description: None,
            technology: None,
            tags: vec![],
        };

        let resolved = resolve_relation_fqns(rel, "A", &elements);

        assert_eq!(resolved.from.as_string(), "A.B");
        assert_eq!(resolved.to.as_string(), "A.C");
    }

    #[test]
    fn test_resolve_relation_fqns_external_reference() {
        let input = r#"
A = system "System A" {
    B = container "Container B"
}
C = system "System C"
"#;
        let program = parse_test_input(input);
        let (elements, _) = collect_elements(&program);

        let rel = Relation {
            from: QualifiedIdent::simple("B".to_string()),
            to: QualifiedIdent::simple("C".to_string()),
            label: Some("calls".to_string()),
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            description: None,
            technology: None,
            tags: vec![],
        };

        let resolved = resolve_relation_fqns(rel, "A", &elements);

        assert_eq!(resolved.from.as_string(), "A.B");
        assert_eq!(resolved.to.as_string(), "C");
    }

    #[test]
    fn test_resolve_relation_fqns_fully_qualified() {
        let input = r#"
A = system "System A" {
    B = container "Container B"
}
C = system "System C"
"#;
        let program = parse_test_input(input);
        let (elements, _) = collect_elements(&program);

        let rel = Relation {
            from: QualifiedIdent::simple("B".to_string()),
            to: QualifiedIdent::qualified(vec!["C".to_string()]),
            label: Some("calls".to_string()),
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            description: None,
            technology: None,
            tags: vec![],
        };

        let resolved = resolve_relation_fqns(rel, "A", &elements);

        assert_eq!(resolved.from.as_string(), "A.B");
        assert_eq!(resolved.to.as_string(), "C");
    }

    #[test]
    fn test_has_location_element_def() {
        let location = SourceLocation::new("test.sruja".to_string(), 10, 5);
        let elem = ElementDef {
            location: location.clone(),
            assignment: ElementAssignment {
                location: location.clone(),
                name: "Test".to_string(),
                kind: ElementKind::System,
                sub_kind: None,
                title: None,
                tag_refs: vec![],
                body: None,
            },
        };

        assert_eq!(elem.location(), &location);
    }

    #[test]
    fn test_has_location_relation() {
        let location = SourceLocation::new("test.sruja".to_string(), 15, 10);
        let rel = Relation {
            from: QualifiedIdent::simple("A".to_string()),
            to: QualifiedIdent::simple("B".to_string()),
            label: Some("calls".to_string()),
            location: location.clone(),
            description: None,
            technology: None,
            tags: vec![],
        };

        assert_eq!(rel.location(), &location);
    }

    #[test]
    fn test_has_location_import() {
        let location = SourceLocation::new("test.sruja".to_string(), 20, 1);
        let import = ImportStatement {
            location: location.clone(),
            elements: vec![ImportElement::Ident("SystemA".to_string())],
            from: "other.sruja".to_string(),
        };

        assert_eq!(import.location(), &location);
    }

    #[test]
    fn test_collect_elements_with_metadata() {
        let input = r#"
A = system "System A" {
    metadata {
        tags ["production", "critical"]
    }
}
"#;
        let program = parse_test_input(input);
        let (elements, relations) = collect_elements(&program);

        assert_eq!(elements.len(), 1);
        assert!(elements.contains_key("A"));
        assert_eq!(relations.len(), 0);

        let elem = &elements["A"];
        if let Some(body) = &elem.assignment.body {
            assert!(!body.metadata.is_empty(), "Element should have metadata");
            assert!(
                body.metadata.iter().any(|m| m.key == "tags"),
                "Should have tags metadata"
            );
        }
    }

    #[test]
    fn test_collect_elements_multiple_top_level() {
        let input = r#"
A = system "System A"
B = system "System B"
C = system "System C"
"#;
        let program = parse_test_input(input);
        let (elements, relations) = collect_elements(&program);

        assert_eq!(elements.len(), 3);
        assert!(elements.contains_key("A"));
        assert!(elements.contains_key("B"));
        assert!(elements.contains_key("C"));
        assert_eq!(relations.len(), 0);
    }

    #[test]
    fn test_relation_with_scope_cross_system() {
        let input = r#"
A = system "System A" {
    B = container "Container B"
}
C = system "System C" {
    D = container "Container D"
}
A.B -> C.D "cross system call"
"#;
        let program = parse_test_input(input);
        let relations = collect_all_relations(&program);

        assert_eq!(relations.len(), 1);
        assert_eq!(relations[0].from.as_string(), "A.B");
        assert_eq!(relations[0].to.as_string(), "C.D");
    }
}
