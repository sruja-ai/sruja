//! AST traversal helpers
//!
//! This module provides helper functions for traversing AST and collecting
//! elements, relations, and other structures needed for validation and processing.

use std::collections::HashMap;

use crate::ast::*;
use sruja_diagnostics::SourceLocation;

/// A relation paired with a scope (parent FQN) it was declared within.
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
                    elem: elem.clone(),
                    parent: String::new(),
                });
            }
            TopLevelItem::Relation(rel) => {
                relations_with_scope.push((rel.clone(), String::new()));
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
                            elem: nested_elem.clone(),
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
                elem: elem.clone(),
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
                        elem: nested.clone(),
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
/// When a relation is defined inside an element body, endpoint names
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
