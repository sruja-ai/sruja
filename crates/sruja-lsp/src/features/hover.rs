//! Hover support for elements and relations.

use crate::workspace::Document;
use sruja_language::ast::Program;
use tower_lsp::lsp_types::*;

use super::utils::{collect_elements, is_ident_char, word_bounds};

/// Find element information for hover
pub fn find_element_hover(program: &Program, id: &str) -> Option<(String, String)> {
    let (elements, _) = collect_elements(program);

    if let Some(elem) = elements.get(id) {
        let kind = elem.assignment.kind.to_string();
        let title = elem
            .assignment
            .title
            .clone()
            .unwrap_or_else(|| elem.assignment.name.clone());
        return Some((kind, title));
    }

    for (fqn, elem) in &elements {
        if fqn.ends_with(&format!(".{}", id)) || fqn == id {
            let kind = elem.assignment.kind.to_string();
            let title = elem
                .assignment
                .title
                .clone()
                .unwrap_or_else(|| elem.assignment.name.clone());
            return Some((kind, title));
        }
    }

    None
}

/// Find relation information for hover
pub fn find_relation_hover(program: &Program, from: &str, to: &str) -> Option<(String, String)> {
    let (_elements, relations) = collect_elements(program);

    for rel in relations {
        let rel_from = rel.from.as_string();
        let rel_to = rel.to.as_string();

        if (rel_from == from || rel_from.ends_with(&format!(".{}", from)))
            && (rel_to == to || rel_to.ends_with(&format!(".{}", to)))
        {
            let verb = String::new();
            let label = rel.label.clone().unwrap_or_default();
            return Some((verb, label));
        }
    }

    None
}

/// Get hover information at a position
pub fn get_hover(
    doc: &Document,
    program: &Program,
    line: usize,
    character: usize,
) -> Option<Hover> {
    let line_text = doc.get_line(line)?;
    if character > line_text.len() {
        return None;
    }

    let (start, end) = word_bounds(line_text, character);
    let word: &str = line_text[start..end].trim();

    if !word.is_empty() {
        if let Some((kind, title)) = find_element_hover(program, word) {
            let content = format!("**{}** `{}`\n{}", kind, word, title);
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: content,
                }),
                range: Some(Range {
                    start: Position {
                        line: line as u32,
                        character: start as u32,
                    },
                    end: Position {
                        line: line as u32,
                        character: end as u32,
                    },
                }),
            });
        }
    }

    if let Some(arrow_idx) = line_text.find("->") {
        if character >= arrow_idx && character < arrow_idx + 2 {
            let (left_start, left_end) = word_bounds(line_text, arrow_idx);
            let left = line_text[left_start..left_end].trim();

            let mut right_pos = arrow_idx + 2;
            while right_pos < line_text.len()
                && !is_ident_char(line_text.chars().nth(right_pos).unwrap_or(' '))
            {
                right_pos += 1;
            }
            let (right_start, right_end) = word_bounds(line_text, right_pos);
            let right = line_text[right_start..right_end].trim();

            if !left.is_empty() && !right.is_empty() {
                if let Some((verb, label)) = find_relation_hover(program, left, right) {
                    let mut parts = Vec::new();
                    if !verb.is_empty() {
                        parts.push(verb);
                    }
                    if !label.is_empty() {
                        parts.push(label);
                    }
                    let content =
                        format!("**Relation** `{} -> {}`\n{}", left, right, parts.join(" "));
                    return Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: content,
                        }),
                        range: Some(Range {
                            start: Position {
                                line: line as u32,
                                character: arrow_idx as u32,
                            },
                            end: Position {
                                line: line as u32,
                                character: (arrow_idx + 2) as u32,
                            },
                        }),
                    });
                }
            }
        }
    }

    None
}
