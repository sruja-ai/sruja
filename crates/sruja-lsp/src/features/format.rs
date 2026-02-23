//! Document formatting support.

use crate::workspace::Document;
use sruja_language::ast::Program;
use tower_lsp::lsp_types::*;

/// Format document using DSL printer
pub fn format_document(doc: &Document, program: &Program) -> Option<Vec<TextEdit>> {
    use sruja_export::dsl::DslPrinter;

    let printer = DslPrinter::new();
    let formatted = printer.print(program);

    let text_edits: Vec<TextEdit> = vec![TextEdit {
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: doc.lines().len() as u32,
                character: 0,
            },
        },
        new_text: formatted,
    }];
    Some(text_edits)
}
