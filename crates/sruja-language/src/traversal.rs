//! AST traversal helpers
//!
//! This module provides helper functions for traversing the AST and collecting
//! elements, relations, and other structures needed for validation and processing.

use std::collections::HashMap;

use crate::ast::*;

/// Collect all element definitions from a program keyed by fully qualified name (FQN)
/// Returns a map of FQN -> ElementDef and a vector of all relations
pub fn collect_elements(program: &Program) -> (HashMap<String, ElementDef>, Vec<Relation>) {
    let estimated_elements = program.items.len() * 4;
    let capacity = estimated_elements.max(16);
    
    let mut elements: HashMap<String, ElementDef> = HashMap::with_capacity(capacity);
    let mut relations: Vec<Relation> = Vec::with_capacity(capacity / 2);

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
                relations.push(rel.clone());
            }
            _ => {}
        }
    }

    // Iterative traversal
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
                        relations.push(rel.clone());
                    }
                    _ => {}
                }
            }
        }
    }

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

/// Collect all relations from a program with their scope
pub fn collect_all_relations(program: &Program) -> Vec<Relation> {
    let (_elements, relations) = collect_elements(program);
    
    // Add top-level relations
    let mut all_relations = relations;
    
    for item in &program.items {
        if let TopLevelItem::Relation(rel) = item {
            all_relations.push(rel.clone());
        }
    }
    
    all_relations
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
