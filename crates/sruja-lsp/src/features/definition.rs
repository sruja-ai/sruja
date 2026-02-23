//! Go-to-definition support.

use crate::workspace::Document;
use sruja_language::ast::Program;
use tower_lsp::lsp_types::*;

use super::utils::collect_elements;

/// Find definition location for an identifier
pub fn find_definition(doc: &Document, program: &Program, id: &str) -> Option<Location> {
    let (elements, _) = collect_elements(program);

    if elements.contains_key(id) {
        for (line_idx, line) in doc.lines().iter().enumerate() {
            let trimmed = line.trim();
            let keywords = vec![
                "system",
                "container",
                "component",
                "datastore",
                "queue",
                "person",
            ];
            for keyword in keywords {
                if let Some(stripped) = trimmed.strip_prefix(keyword) {
                    let rest = stripped.trim();
                    if rest.starts_with(id) {
                        if let Some(col) = line.find(id) {
                            return Some(Location {
                                uri: doc.uri().clone(),
                                range: Range {
                                    start: Position {
                                        line: line_idx as u32,
                                        character: col as u32,
                                    },
                                    end: Position {
                                        line: line_idx as u32,
                                        character: (col + id.len()) as u32,
                                    },
                                },
                            });
                        }
                    }
                }
            }
        }
    }

    None
}
