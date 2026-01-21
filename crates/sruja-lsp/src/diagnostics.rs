//! LSP diagnostics conversion
//!
//! This module converts Sruja diagnostics to LSP diagnostic format.

use tower_lsp::lsp_types::*;
use sruja_diagnostics::{Diagnostic as SrujaDiagnostic, Severity};

/// Convert Sruja diagnostics to LSP diagnostics
pub fn convert_diagnostics_to_lsp(diags: &[SrujaDiagnostic]) -> Vec<tower_lsp::lsp_types::Diagnostic> {
    diags.iter().map(|d| {
        let severity = match d.severity {
            Severity::Warning => DiagnosticSeverity::WARNING,
            Severity::Info => DiagnosticSeverity::INFORMATION,
            Severity::Error => DiagnosticSeverity::ERROR,
        };

        // Calculate end position for better range highlighting
        let start_line = d.location.line.saturating_sub(1);
        let start_char = d.location.column.saturating_sub(1);
        let mut end_line = start_line;
        let mut end_char = start_char + 1; // Default to single character

        // Try to estimate token length for better highlighting using context
        if !d.context.is_empty() {
            let line_idx = if d.context.len() > 1 { 1 } else { 0 };
            if let Some(line_text) = d.context.get(line_idx) {
                let start_col_usize = start_char as usize;
                if start_col_usize < line_text.len() {
                    let mut estimated_end = start_col_usize;
                    // Find word boundary or next delimiter
                    while estimated_end < line_text.len() && estimated_end < start_col_usize + 50 {
                        let c = line_text.chars().nth(estimated_end).unwrap_or(' ');
                        if matches!(c, ' ' | '\t' | '{' | '}' | ':' | ',' | ';' | '\n' | '[' | ']' | '(' | ')') {
                            break;
                        }
                        estimated_end += 1;
                    }
                    if estimated_end > start_col_usize {
                        end_char = estimated_end as u32;
                    }
                }
            }
        }

        tower_lsp::lsp_types::Diagnostic {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_char,
                },
                end: Position {
                    line: end_line,
                    character: end_char,
                },
            },
            severity: Some(severity),
            code: Some(tower_lsp::lsp_types::NumberOrString::String(d.code.clone())),
            source: Some("sruja".to_string()),
            message: d.message.clone(),
            related_information: None,
            tags: None,
            data: None,
        }
    }).collect()
}
