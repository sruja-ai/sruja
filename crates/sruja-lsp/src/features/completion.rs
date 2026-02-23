//! Completion (autocomplete) support.

use crate::workspace::Document;
use sruja_language::ast::Program;
use tower_lsp::lsp_types::*;

use super::utils::{collect_elements, last_token};

/// Get completion items at a position
pub fn get_completion(
    doc: &Document,
    program: &Program,
    line: usize,
    character: usize,
) -> Vec<CompletionItem> {
    let line_text = match doc.get_line(line) {
        Some(l) => l,
        None => return Vec::new(),
    };

    if character > line_text.len() {
        return Vec::new();
    }

    let before = &line_text[..character];
    let token: String = last_token(before);

    let mut items = Vec::new();

    let keywords = vec![
        "system",
        "container",
        "component",
        "datastore",
        "database",
        "queue",
        "person",
        "adr",
        "requirement",
        "policy",
        "scenario",
        "story",
        "flow",
        "description",
        "technology",
        "tags",
        "metadata",
        "relation",
    ];

    for keyword in keywords {
        if token.is_empty() || keyword.to_lowercase().starts_with(&token.to_lowercase()) {
            items.push(CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                ..Default::default()
            });
        }
    }

    let (elements, _) = collect_elements(program);
    let mut seen = std::collections::HashSet::new();

    for fqn in elements.keys() {
        if let Some(short) = fqn.split('.').next_back() {
            if !seen.contains(short) {
                seen.insert(short.to_string());
                if token.is_empty() || short.to_lowercase().starts_with(&token.to_lowercase()) {
                    items.push(CompletionItem {
                        label: short.to_string(),
                        kind: Some(CompletionItemKind::TEXT),
                        ..Default::default()
                    });
                }
            }
        }

        if !seen.contains(fqn) {
            seen.insert(fqn.clone());
            if token.is_empty() || fqn.to_lowercase().starts_with(&token.to_lowercase()) {
                items.push(CompletionItem {
                    label: fqn.clone(),
                    kind: Some(CompletionItemKind::TEXT),
                    ..Default::default()
                });
            }
        }
    }

    if before.contains("->") {
        let verbs = vec!["reads", "writes", "calls", "uses", "publishes"];
        for verb in verbs {
            items.push(CompletionItem {
                label: verb.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                ..Default::default()
            });
        }
    }

    items
}
