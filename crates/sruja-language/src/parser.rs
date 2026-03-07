//! Parser for Sruja DSL using nom parser combinators
//!
//! This module implements a parser for the Sruja DSL using `nom` parser combinators.
//! It parses source code directly into an AST without a separate lexing phase.

mod assignments;
mod blocks;
mod elements;
mod import;
mod loops;
mod merge;
mod overview_views;
mod primitives;
mod program;
mod relations;

#[cfg(test)]
mod tests;

use primitives::line_to_byte_offset;
use program::parse_program;
use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};

use crate::ast::*;

/// Parser for Sruja DSL
pub struct Parser {
    filename: String,
}

impl Parser {
    /// Create a new parser
    pub fn new(filename: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
        }
    }

    /// Calculate the line number for a byte position in the input
    #[allow(dead_code)]
    fn line_number(&self, input: &str, pos: usize) -> u32 {
        let truncated = if pos <= input.len() {
            &input[..pos]
        } else {
            input
        };
        let line = truncated.matches('\n').count();
        line.min(u32::MAX as usize) as u32
    }

    /// Create a SourceLocation for a position
    #[allow(dead_code)]
    fn location(&self, input: &str, pos: usize) -> SourceLocation {
        SourceLocation::new(self.filename.clone(), self.line_number(input, pos), 0)
    }

    /// Parse source code into a Program AST
    pub fn parse(&self, input: &str) -> Result<Program, Vec<Diagnostic>> {
        match parse_program(input) {
            Ok((remaining, program)) => {
                let trimmed = remaining.trim();
                if !trimmed.is_empty() {
                    // Try to provide more context about what couldn't be parsed
                    let preview = if trimmed.len() > 100 {
                        format!("{}...", &trimmed[..100])
                    } else {
                        trimmed.to_string()
                    };

                    // Count lines to provide better error location
                    let lines_before_remaining = input.len() - remaining.len();
                    let line_number = input[..lines_before_remaining].matches('\n').count();
                    let line_number_u32 = line_number.min(u32::MAX as usize) as u32;

                    return Err(vec![Diagnostic::new(
                        sruja_diagnostics::codes::CODE_SYNTAX_ERROR,
                        Severity::Error,
                        format!(
                            "Unexpected input remaining at line {}: {}",
                            line_number + 1,
                            preview.replace('\n', "\\n").replace('\r', "\\r")
                        ),
                        SourceLocation::new(self.filename.clone(), line_number_u32, 0),
                    )]);
                }
                Ok(program)
            }
            Err(e) => {
                // Try to extract more information from the nom error
                let (error_msg, error_pos) = match &e {
                    nom::Err::Error(err) => {
                        let pos = input.len() - err.input.len();
                        let line = self.line_number(input, pos);
                        (
                            format!("Parse error at line {}: {:?}", line + 1, err.code),
                            line,
                        )
                    }
                    nom::Err::Failure(err) => {
                        let pos = input.len() - err.input.len();
                        let line = self.line_number(input, pos);
                        (
                            format!("Parse failure at line {}: {:?}", line + 1, err.code),
                            line,
                        )
                    }
                    nom::Err::Incomplete(_) => ("Incomplete input".to_string(), 0),
                };

                Err(vec![Diagnostic::new(
                    sruja_diagnostics::codes::CODE_SYNTAX_ERROR,
                    Severity::Error,
                    error_msg,
                    SourceLocation::new(self.filename.clone(), error_pos, 0),
                )])
            }
        }
    }

    /// Parse a specific section of DSL code incrementally
    ///
    /// This function parses only the changed portion of the DSL and merges it with the existing
    /// AST, avoiding full re-parsing of the entire document.
    ///
    /// Parameters:
    /// - `input`: The full DSL source code
    /// - `change_start`: The starting position of the change in the DSL
    /// - `change_end`: The ending position of the change in the DSL
    /// - `existing_ast`: The existing AST to merge changes into
    /// - `context_lines`: Number of lines to parse before/after the change for context
    ///
    /// Returns:
    /// - Updated AST if parsing succeeds
    /// - Diagnostic errors if parsing fails
    pub fn parse_incrementally(
        &self,
        input: &str,
        change_start: usize,
        change_end: usize,
        existing_ast: &Program,
        context_lines: usize,
    ) -> Result<IncrementalParseResult, Vec<Diagnostic>> {
        let start = std::time::Instant::now();

        // Find the line numbers for the change range (0-based)
        let start_line = input[..change_start].matches('\n').count();
        let end_line = input[..change_end].matches('\n').count();
        let total_lines = input.matches('\n').count();

        // Context window: [context_start_line, context_end_line] inclusive
        let context_start_line = start_line.saturating_sub(context_lines);
        let context_end_line = (end_line + context_lines).min(total_lines);

        // Byte offsets: start of line N = position after (N-1)th newline; end of context = start of line (context_end_line + 1)
        let context_start_pos = line_to_byte_offset(input, context_start_line);
        let context_end_pos = line_to_byte_offset(input, context_end_line + 1).min(input.len());

        let context_section = &input[context_start_pos..context_end_pos];

        // Parse the context section
        match parse_program(context_section) {
            Ok((_remaining, new_program)) => {
                let merged_ast =
                    merge::smart_merge_asts(existing_ast, &new_program, context_start_line);
                let (changed_elements, changed_ranges) =
                    merge::analyze_changes(existing_ast, &merged_ast);

                let elapsed = start.elapsed().as_millis() as u64;

                Ok(IncrementalParseResult {
                    updated_ast: merged_ast,
                    changed_elements,
                    changed_ranges,
                    parsing_time_ms: elapsed,
                })
            }
            Err(e) => {
                // Try to provide more context about the parse error
                let (error_msg, line_number) = match &e {
                    nom::Err::Error(err) => {
                        // Calculate line number within the context section
                        let context_line = context_section[..err.input.len()].matches('\n').count();
                        let absolute_line = context_start_line + context_line;
                        (
                            format!(
                                "Parse error in context section at line {}: {:?}",
                                absolute_line + 1,
                                err.code
                            ),
                            absolute_line as u32,
                        )
                    }
                    nom::Err::Failure(err) => {
                        let context_line = context_section[..err.input.len()].matches('\n').count();
                        let absolute_line = context_start_line + context_line;
                        (
                            format!(
                                "Parse failure in context section at line {}: {:?}",
                                absolute_line + 1,
                                err.code
                            ),
                            absolute_line as u32,
                        )
                    }
                    nom::Err::Incomplete(_) => (
                        "Incomplete input in context section".to_string(),
                        context_start_line as u32,
                    ),
                };

                Err(vec![Diagnostic::new(
                    sruja_diagnostics::codes::CODE_SYNTAX_ERROR,
                    Severity::Error,
                    error_msg,
                    SourceLocation::new(self.filename.clone(), line_number, 0),
                )])
            }
        }
    }
}
