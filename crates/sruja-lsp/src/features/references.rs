//! Find references support.

use crate::workspace::Document;
use sruja_language::ast::Program;
use tower_lsp::lsp_types::*;

use super::utils::word_bounds;

/// Find all references to an identifier
pub fn find_references(doc: &Document, _program: &Program, id: &str) -> Vec<Location> {
    let mut locations = Vec::new();

    for (line_idx, line) in doc.lines().iter().enumerate() {
        let mut search_pos = 0;
        while let Some(pos) = line[search_pos..].find(id) {
            let abs_pos = search_pos + pos;
            let (start, end) = word_bounds(line, abs_pos);
            if line[start..end] == *id {
                locations.push(Location {
                    uri: doc.uri().clone(),
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: start as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: end as u32,
                        },
                    },
                });
            }
            search_pos = abs_pos + 1;
        }
    }

    locations
}
