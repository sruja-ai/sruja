//! AST merge and change analysis for incremental parsing.

use std::collections::HashMap;

use crate::ast::{ElementDef, Program, Relation, TopLevelItem};

/// Merge two ASTs: update existing elements, add new elements/relations, apply line offset.
pub(super) fn smart_merge_asts(
    existing_ast: &Program,
    new_ast: &Program,
    context_line_offset: usize,
) -> Program {
    let mut merged_ast = existing_ast.clone();
    let mut element_map = HashMap::new();

    for item in &existing_ast.items {
        if let TopLevelItem::ElementDef(elem) = item {
            element_map.insert(elem.assignment.name.clone(), elem.assignment.name.clone());
        }
    }

    for item in &new_ast.items {
        match item {
            TopLevelItem::ElementDef(new_elem) => {
                let elem_name = &new_elem.assignment.name;
                if element_map.contains_key(elem_name) {
                    update_existing_element(&mut merged_ast, new_elem, context_line_offset);
                } else {
                    add_new_element(&mut merged_ast, new_elem, context_line_offset);
                }
            }
            TopLevelItem::Relation(new_rel) => {
                add_new_relation(&mut merged_ast, new_rel, context_line_offset);
            }
            _ => {}
        }
    }

    merged_ast
}

fn update_existing_element(
    ast: &mut Program,
    new_elem: &ElementDef,
    line_offset: usize,
) {
    let offset = line_offset as i32;
    for item in ast.items.iter_mut() {
        if let TopLevelItem::ElementDef(elem) = item {
            if elem.assignment.name == new_elem.assignment.name {
                elem.assignment = new_elem.assignment.clone();
                update_item_line_numbers(item, offset);
                return;
            }
        }
    }
}

fn add_new_element(ast: &mut Program, new_elem: &ElementDef, line_offset: usize) {
    let mut elem = new_elem.clone();
    let off = line_offset as i32;
    elem.location.line = (elem.location.line as i32 + off).max(0) as u32;
    elem.assignment.location.line = (elem.assignment.location.line as i32 + off).max(0) as u32;
    ast.items.push(TopLevelItem::ElementDef(Box::new(elem)));
}

fn add_new_relation(ast: &mut Program, new_rel: &Relation, line_offset: usize) {
    let mut rel = new_rel.clone();
    rel.location.line = (rel.location.line as i32 + line_offset as i32).max(0) as u32;
    ast.items.push(TopLevelItem::Relation(rel));
}

/// Compare two ASTs and return changed element names and affected ranges.
pub(super) fn analyze_changes(
    old_ast: &Program,
    new_ast: &Program,
) -> (Vec<String>, Vec<(usize, usize)>) {
    let mut changed_elements = Vec::new();
    let mut changed_ranges = Vec::new();

    let old_elements: HashMap<_, _> = old_ast
        .items
        .iter()
        .filter_map(|item| {
            if let TopLevelItem::ElementDef(elem) = item {
                Some((elem.assignment.name.clone(), item))
            } else {
                None
            }
        })
        .collect();

    let new_elements: HashMap<_, _> = new_ast
        .items
        .iter()
        .filter_map(|item| {
            if let TopLevelItem::ElementDef(elem) = item {
                Some((elem.assignment.name.clone(), item))
            } else {
                None
            }
        })
        .collect();

    for (name, new_item) in &new_elements {
        if let TopLevelItem::ElementDef(new_elem) = new_item {
            if let Some(old_item) = old_elements.get(name) {
                if let TopLevelItem::ElementDef(old_elem) = old_item {
                    if old_elem.assignment.title != new_elem.assignment.title
                        || old_elem.assignment.kind != new_elem.assignment.kind
                    {
                        changed_elements.push(name.clone());
                    }
                }
            } else {
                changed_elements.push(name.clone());
            }
        }
    }

    changed_ranges.push((0, 0));
    (changed_elements, changed_ranges)
}

fn update_item_line_numbers(item: &mut TopLevelItem, line_offset: i32) {
    match item {
        TopLevelItem::ElementDef(elem) => {
            elem.location.line = (elem.location.line as i32 + line_offset).max(0) as u32;
            elem.assignment.location.line =
                (elem.assignment.location.line as i32 + line_offset).max(0) as u32;
        }
        TopLevelItem::Relation(rel) => {
            rel.location.line = (rel.location.line as i32 + line_offset).max(0) as u32;
        }
        _ => {}
    }
}
