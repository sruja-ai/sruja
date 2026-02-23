//! Document symbols (outline) support.

use crate::workspace::Document;
use sruja_language::ast::{ElementKind, Program};
use tower_lsp::lsp_types::*;

use super::utils::collect_elements;

/// Get document symbols
pub fn get_document_symbols(doc: &Document, program: &Program) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();
    let (elements, _) = collect_elements(program);

    for (fqn, elem) in &elements {
        let kind = match elem.assignment.kind {
            ElementKind::System => SymbolKind::CLASS,
            ElementKind::Container => SymbolKind::MODULE,
            ElementKind::Component => SymbolKind::FUNCTION,
            ElementKind::Person => SymbolKind::OBJECT,
            ElementKind::Database | ElementKind::DataStore => SymbolKind::STRUCT,
            ElementKind::Queue => SymbolKind::OBJECT,
            _ => SymbolKind::OBJECT,
        };

        let title = elem
            .assignment
            .title
            .clone()
            .unwrap_or_else(|| elem.assignment.name.clone());

        let mut line = 0;
        for (line_idx, line_text) in doc.lines().iter().enumerate() {
            if line_text.contains(&elem.assignment.name) {
                line = line_idx;
                break;
            }
        }

        #[allow(deprecated)]
        symbols.push(DocumentSymbol {
            name: fqn.clone(),
            detail: Some(title),
            kind,
            range: Range {
                start: Position {
                    line: line as u32,
                    character: 0,
                },
                end: Position {
                    line: line as u32,
                    character: 0,
                },
            },
            selection_range: Range {
                start: Position {
                    line: line as u32,
                    character: 0,
                },
                end: Position {
                    line: line as u32,
                    character: 0,
                },
            },
            children: None,
            tags: None,
            deprecated: None,
        });
    }

    symbols
}
