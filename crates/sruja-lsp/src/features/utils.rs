//! Shared helpers for LSP features (word boundaries, element collection).

use sruja_language::ast::{ElementDef, Program, Relation, TopLevelItem};

/// Find word boundaries at a position in a line
pub fn word_bounds(line: &str, pos: usize) -> (usize, usize) {
    let pos = pos.min(line.len());
    let mut start = pos;
    while start > 0 && is_ident_char(line.chars().nth(start - 1).unwrap_or(' ')) {
        start -= 1;
    }
    let mut end = pos;
    while end < line.len() && is_ident_char(line.chars().nth(end).unwrap_or(' ')) {
        end += 1;
    }
    (start, end)
}

/// Character allowed in identifiers (for LSP word boundaries).
pub fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == '.'
}

/// Collect elements from a program for quick lookup
pub fn collect_elements(
    program: &Program,
) -> (std::collections::HashMap<String, ElementDef>, Vec<Relation>) {
    let mut elements = std::collections::HashMap::new();
    let mut relations = Vec::new();

    for item in &program.items {
        match item {
            TopLevelItem::ElementDef(elem) => {
                elements.insert(elem.assignment.name.clone(), (**elem).clone());
            }
            TopLevelItem::Relation(rel) => {
                relations.push(rel.clone());
            }
            _ => {}
        }
    }

    (elements, relations)
}

/// Get the last token before the cursor position
pub fn last_token(s: &str) -> String {
    let mut i = s.len();
    while i > 0 && !is_ident_char(s.chars().nth(i - 1).unwrap_or(' ')) {
        i -= 1;
    }
    if i == 0 {
        return String::new();
    }
    let mut j = i;
    while j > 0 && is_ident_char(s.chars().nth(j - 1).unwrap_or(' ')) {
        j -= 1;
    }
    s[j..i].trim().to_string()
}
