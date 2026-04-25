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
            TopLevelItem::CausalLoop(cl) => {
                let loc = cl.location.clone();
                let elem_def = ElementDef {
                    location: loc.clone(),
                    assignment: ElementAssignment {
                        location: loc,
                        name: cl.id.clone(),
                        kind: ElementKind::Custom("causal_loop".to_string()),
                        sub_kind: None,
                        title: Some(cl.title.clone()),
                        tag_refs: vec![],
                        body: None,
                    },
                };
                elements.insert(cl.id.clone(), elem_def);
            }
            TopLevelItem::FeedbackLoop(fl) => {
                let loc = fl.location.clone();
                let elem_def = ElementDef {
                    location: loc.clone(),
                    assignment: ElementAssignment {
                        location: loc,
                        name: fl.id.clone(),
                        kind: ElementKind::Custom("feedback".to_string()),
                        sub_kind: None,
                        title: Some(fl.title.clone()),
                        tag_refs: vec![],
                        body: None,
                    },
                };
                elements.insert(fl.id.clone(), elem_def);
            }
            TopLevelItem::Adr(adr) => {
                let elem_def = ElementDef {
                    location: adr.location.clone(),
                    assignment: ElementAssignment {
                        location: adr.location.clone(),
                        name: adr.id.clone(),
                        kind: ElementKind::Adr,
                        sub_kind: None,
                        title: Some(adr.title.clone()),
                        tag_refs: vec![],
                        body: None,
                    },
                };
                elements.insert(adr.id.clone(), elem_def);
            }
            TopLevelItem::Flow(flow) => {
                let elem_def = ElementDef {
                    location: flow.location.clone(),
                    assignment: ElementAssignment {
                        location: flow.location.clone(),
                        name: flow.id.clone(),
                        kind: ElementKind::Flow,
                        sub_kind: None,
                        title: Some(flow.title.clone()),
                        tag_refs: vec![],
                        body: None,
                    },
                };
                elements.insert(flow.id.clone(), elem_def);
            }
            TopLevelItem::Schema(_schema) => {}
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
            TopLevelItem::FeedbackLoop(fl) => {
                for rel in &fl.relationships {
                    out.push(RelationWithScope {
                        relation: rel.clone(),
                        scope: fl.id.clone(),
                    });
                }
            }
            TopLevelItem::CausalLoop(cl) => {
                let loc = cl.location.clone();
                for cr in &cl.relationships {
                    let relation = Relation {
                        location: loc.clone(),
                        from: QualifiedIdent::simple(cr.from.clone()),
                        to: QualifiedIdent::simple(cr.to.clone()),
                        label: cr.effect.clone(),
                        description: None,
                        technology: None,
                        tags: vec![],
                    };
                    out.push(RelationWithScope {
                        relation,
                        scope: cl.id.clone(),
                    });
                }
            }
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

/// Returns true if the character is valid in a DSL identifier (alphanumeric, _, ., -).
fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '.' || c == '-'
}

/// Find the (0-based) line and character of the first definition of an identifier in source.
/// A definition is a line where the identifier appears at word boundary followed by optional
/// whitespace and '='. Used when the parser does not set source locations (e.g. all 0,0).
pub fn find_definition_line(source: &str, identifier: &str) -> Option<(u32, u32)> {
    if identifier.is_empty() {
        return None;
    }
    for (line_idx, line) in source.lines().enumerate() {
        let mut search_start = 0;
        while let Some(rel_pos) = line[search_start..].find(identifier) {
            let pos = search_start + rel_pos;
            let before_ok = pos == 0
                || !line
                    .chars()
                    .nth(pos.saturating_sub(1))
                    .is_some_and(is_ident_char);
            let after_end = pos + identifier.len();
            let rest = line.get(after_end..).unwrap_or("");
            let after_ok = rest.trim_start().starts_with('=')
                && rest.chars().next().is_none_or(|c| !is_ident_char(c));
            if before_ok && after_ok {
                return Some((line_idx as u32, pos as u32));
            }
            search_start = pos + 1;
        }
    }
    None
}

fn is_unset_location(loc: &SourceLocation) -> bool {
    loc.line == 0 && loc.column == 0
}

pub fn populate_locations(program: &mut Program, source: &str, filename: &str) {
    for item in &mut program.items {
        match item {
            TopLevelItem::ElementDef(elem) => populate_element_locations(elem, source, filename),
            TopLevelItem::Relation(rel) => populate_relation_location(rel, source, filename),
            TopLevelItem::Requirement(req) if is_unset_location(&req.location) => {
                if let Some((line, col)) = find_definition_line(source, &req.id) {
                    req.location = SourceLocation::new(filename.to_string(), line + 1, col + 1);
                }
            }
            TopLevelItem::View(view) if is_unset_location(&view.location) => {
                if let Some((line, col)) = find_view_definition_line(source, &view.id) {
                    view.location = SourceLocation::new(filename.to_string(), line + 1, col + 1);
                }
            }
            TopLevelItem::Schema(schema) if is_unset_location(&schema.location) => {
                if let Some((line, col)) = find_schema_definition_line(source, &schema.name) {
                    schema.location = SourceLocation::new(filename.to_string(), line + 1, col + 1);
                }
            }
            _ => {}
        }
    }
}

fn populate_element_locations(elem: &mut ElementDef, source: &str, filename: &str) {
    if is_unset_location(&elem.location) {
        if let Some((line, col)) = find_definition_line(source, &elem.assignment.name) {
            elem.location = SourceLocation::new(filename.to_string(), line + 1, col + 1);
            if is_unset_location(&elem.assignment.location) {
                elem.assignment.location = elem.location.clone();
            }
        }
    }

    if let Some(body) = &mut elem.assignment.body {
        for item in &mut body.items {
            match item {
                ElementDefBodyItem::ElementDef(nested) => {
                    populate_element_locations(nested, source, filename);
                }
                ElementDefBodyItem::Relation(rel) => {
                    populate_relation_location(rel, source, filename);
                }
                _ => {}
            }
        }
    }
}

fn populate_relation_location(rel: &mut Relation, source: &str, filename: &str) {
    if !is_unset_location(&rel.location) {
        return;
    }

    let from = rel.from.as_string();
    let to = rel.to.as_string();
    let pattern = format!("{} -> {}", from, to);

    for (line_idx, line) in source.lines().enumerate() {
        if let Some(pos) = line.find(&pattern) {
            rel.location = SourceLocation::new(
                filename.to_string(),
                (line_idx + 1) as u32,
                (pos + 1) as u32,
            );
            return;
        }
    }
}

fn find_view_definition_line(source: &str, identifier: &str) -> Option<(u32, u32)> {
    for (line_idx, line) in source.lines().enumerate() {
        if line.contains("view") && line.contains(identifier) {
            if let Some(pos) = line.find(identifier) {
                return Some((line_idx as u32, pos as u32));
            }
        }
    }
    None
}

fn find_schema_definition_line(source: &str, identifier: &str) -> Option<(u32, u32)> {
    for (line_idx, line) in source.lines().enumerate() {
        if line.contains("schema") && line.contains(identifier) {
            if let Some(pos) = line.find(identifier) {
                return Some((line_idx as u32, pos as u32));
            }
        }
    }
    None
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

impl HasLocation for SchemaBlock {
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
    fn test_find_definition_line() {
        let source = r#"
A = system "System A"
B = system "System B"
A -> B "calls"
"#;
        assert_eq!(find_definition_line(source, "A"), Some((1, 0)));
        assert_eq!(find_definition_line(source, "B"), Some((2, 0)));
        assert_eq!(find_definition_line(source, "X"), None);
        // Should not match "A" inside "A ->"
        let src2 = "A = system \"A\"\nA -> B \"x\"\n";
        assert_eq!(find_definition_line(src2, "A"), Some((0, 0)));
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
A = system "System A"
B = system "System B"
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
A = system "System A"
B = system "System B"
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
