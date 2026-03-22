//! LSP diagnostics conversion
//!
//! This module converts Sruja diagnostics to LSP diagnostic format.

use sruja_diagnostics::{Diagnostic as SrujaDiagnostic, Severity};
use tower_lsp::lsp_types::*;

/// Convert Sruja diagnostics to LSP diagnostics
pub fn convert_diagnostics_to_lsp(
    diags: &[SrujaDiagnostic],
) -> Vec<tower_lsp::lsp_types::Diagnostic> {
    diags
        .iter()
        .map(|d| {
            let severity = match d.severity {
                Severity::Warning => DiagnosticSeverity::WARNING,
                Severity::Info => DiagnosticSeverity::INFORMATION,
                Severity::Error => DiagnosticSeverity::ERROR,
                _ => DiagnosticSeverity::HINT,
            };

            // Calculate end position for better range highlighting
            let start_line = d.location.line.saturating_sub(1);
            let start_char = d.location.column.saturating_sub(1);
            let end_line = start_line;
            let mut end_char = start_char + 1; // Default to single character

            // Try to estimate token length for better highlighting using context
            if !d.context.is_empty() {
                let line_idx = if d.context.len() > 1 { 1 } else { 0 };
                if let Some(line_text) = d.context.get(line_idx) {
                    let start_col_usize = start_char as usize;
                    let chars: Vec<char> = line_text.chars().collect();
                    if start_col_usize < chars.len() {
                        let mut estimated_end = start_col_usize;
                        // Find word boundary or next delimiter
                        while estimated_end < chars.len() && estimated_end < start_col_usize + 50 {
                            let c = chars[estimated_end];
                            if matches!(
                                c,
                                ' ' | '\t'
                                    | '{'
                                    | '}'
                                    | ':'
                                    | ','
                                    | ';'
                                    | '\n'
                                    | '['
                                    | ']'
                                    | '('
                                    | ')'
                            ) {
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
                code_description: None,
                source: Some("sruja".to_string()),
                message: d.message.clone(),
                related_information: None,
                tags: None,
                data: None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_diagnostics::{Diagnostic as SrujaDiagnostic, Severity, SourceLocation};

    fn create_test_diagnostic(
        severity: Severity,
        line: u32,
        column: u32,
        message: &str,
        code: &str,
        context: Vec<String>,
    ) -> SrujaDiagnostic {
        SrujaDiagnostic::new(
            code.to_string(),
            severity,
            message.to_string(),
            SourceLocation::new("test.sruja".to_string(), line, column),
        )
        .with_context(context)
    }

    #[test]
    fn test_convert_diagnostics_to_lsp_error() {
        let diags = vec![create_test_diagnostic(
            Severity::Error,
            1,
            5,
            "Error message",
            "E001",
            vec!["line of code".to_string()],
        )];

        let lsp_diags = convert_diagnostics_to_lsp(&diags);
        assert_eq!(lsp_diags.len(), 1);

        let lsp_diag = &lsp_diags[0];
        assert_eq!(lsp_diag.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(lsp_diag.message, "Error message");
        assert_eq!(lsp_diag.range.start.line, 0);
        assert_eq!(lsp_diag.range.start.character, 4);
        assert_eq!(lsp_diag.source, Some("sruja".to_string()));
    }

    #[test]
    fn test_convert_diagnostics_to_lsp_warning() {
        let diags = vec![create_test_diagnostic(
            Severity::Warning,
            2,
            10,
            "Warning message",
            "W001",
            vec![],
        )];

        let lsp_diags = convert_diagnostics_to_lsp(&diags);
        assert_eq!(lsp_diags.len(), 1);

        let lsp_diag = &lsp_diags[0];
        assert_eq!(lsp_diag.severity, Some(DiagnosticSeverity::WARNING));
        assert_eq!(lsp_diag.message, "Warning message");
        assert_eq!(lsp_diag.range.start.line, 1);
        assert_eq!(lsp_diag.range.start.character, 9);
    }

    #[test]
    fn test_convert_diagnostics_to_lsp_info() {
        let diags = vec![create_test_diagnostic(
            Severity::Info,
            3,
            0,
            "Info message",
            "I001",
            vec![],
        )];

        let lsp_diags = convert_diagnostics_to_lsp(&diags);
        assert_eq!(lsp_diags.len(), 1);

        let lsp_diag = &lsp_diags[0];
        assert_eq!(lsp_diag.severity, Some(DiagnosticSeverity::INFORMATION));
        assert_eq!(lsp_diag.message, "Info message");
        assert_eq!(lsp_diag.range.start.line, 2);
        assert_eq!(lsp_diag.range.start.character, 0);
    }

    #[test]
    fn test_convert_diagnostics_to_lsp_multiple() {
        let diags = vec![
            create_test_diagnostic(Severity::Error, 1, 5, "Error 1", "E001", vec![]),
            create_test_diagnostic(Severity::Warning, 2, 10, "Warning 1", "W001", vec![]),
            create_test_diagnostic(Severity::Info, 3, 0, "Info 1", "I001", vec![]),
        ];

        let lsp_diags = convert_diagnostics_to_lsp(&diags);
        assert_eq!(lsp_diags.len(), 3);

        assert_eq!(lsp_diags[0].severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(lsp_diags[1].severity, Some(DiagnosticSeverity::WARNING));
        assert_eq!(lsp_diags[2].severity, Some(DiagnosticSeverity::INFORMATION));
    }

    #[test]
    fn test_convert_diagnostics_to_lsp_empty() {
        let diags: Vec<SrujaDiagnostic> = vec![];
        let lsp_diags = convert_diagnostics_to_lsp(&diags);
        assert_eq!(lsp_diags.len(), 0);
    }

    #[test]
    fn test_convert_diagnostics_to_lsp_with_context() {
        let diags = vec![create_test_diagnostic(
            Severity::Error,
            1,
            5,
            "Invalid identifier",
            "E001",
            vec!["  app = system \"My App\" {}".to_string()],
        )];

        let lsp_diags = convert_diagnostics_to_lsp(&diags);
        assert_eq!(lsp_diags.len(), 1);

        let lsp_diag = &lsp_diags[0];
        // With context, the end position should be calculated better
        assert!(lsp_diag.range.end.character >= lsp_diag.range.start.character);
    }

    #[test]
    fn test_convert_diagnostics_to_lsp_range_calculation() {
        let diags = vec![create_test_diagnostic(
            Severity::Error,
            1,
            5,
            "Error",
            "E001",
            vec!["    variable_name = value".to_string()],
        )];

        let lsp_diags = convert_diagnostics_to_lsp(&diags);
        assert_eq!(lsp_diags.len(), 1);

        let lsp_diag = &lsp_diags[0];
        // Range should be calculated to highlight the relevant token
        assert!(lsp_diag.range.end.character > lsp_diag.range.start.character);
    }

    #[test]
    fn test_convert_diagnostics_to_lsp_code() {
        let diags = vec![create_test_diagnostic(
            Severity::Error,
            1,
            0,
            "Test",
            "CUSTOM_CODE_123",
            vec![],
        )];

        let lsp_diags = convert_diagnostics_to_lsp(&diags);
        assert_eq!(lsp_diags.len(), 1);

        let lsp_diag = &lsp_diags[0];
        assert!(lsp_diag.code.is_some());
        match &lsp_diag.code {
            Some(NumberOrString::String(code)) => assert_eq!(code, "CUSTOM_CODE_123"),
            Some(NumberOrString::Number(n)) => panic!("Expected String code, got Number: {}", n),
            None => panic!("Expected String code, got None"),
        }
    }

    #[test]
    fn test_convert_diagnostics_to_lsp_source() {
        let diags = vec![create_test_diagnostic(
            Severity::Error,
            1,
            0,
            "Test",
            "E001",
            vec![],
        )];

        let lsp_diags = convert_diagnostics_to_lsp(&diags);
        assert_eq!(lsp_diags[0].source, Some("sruja".to_string()));
    }

    #[test]
    fn test_convert_diagnostics_edge_case_large_line_column() {
        let diags = vec![create_test_diagnostic(
            Severity::Error,
            9999,
            9999,
            "Edge case",
            "E001",
            vec![],
        )];

        let lsp_diags = convert_diagnostics_to_lsp(&diags);
        assert_eq!(lsp_diags.len(), 1);
        assert_eq!(lsp_diags[0].range.start.line, 9998);
        assert_eq!(lsp_diags[0].range.start.character, 9998);
    }

    #[test]
    fn test_convert_diagnostics_zero_line_column() {
        let diags = vec![create_test_diagnostic(
            Severity::Error,
            0,
            0,
            "Start",
            "E001",
            vec![],
        )];

        let lsp_diags = convert_diagnostics_to_lsp(&diags);
        assert_eq!(lsp_diags.len(), 1);
        assert_eq!(lsp_diags[0].range.start.line, 0);
        assert_eq!(lsp_diags[0].range.start.character, 0);
    }

    #[test]
    fn test_convert_diagnostics_multiline_context() {
        let diags = vec![create_test_diagnostic(
            Severity::Error,
            5,
            10,
            "Error with context",
            "E001",
            vec![
                "  1 | app = system \"App\" {".to_string(),
                "  2 |   invalid_token_here".to_string(),
                "  3 | }".to_string(),
            ],
        )];

        let lsp_diags = convert_diagnostics_to_lsp(&diags);
        assert_eq!(lsp_diags.len(), 1);

        let lsp_diag = &lsp_diags[0];
        assert!(lsp_diag.range.end.character > lsp_diag.range.start.character);
    }
}
