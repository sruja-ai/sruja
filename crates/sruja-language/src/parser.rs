//! Parser for Sruja DSL using nom parser combinators
//!
//! This module implements a parser for the Sruja DSL using `nom` parser combinators.
//! It parses source code directly into an AST without a separate lexing phase.

mod assignments;
mod blocks;
mod deployment;
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

fn line_col_1_indexed(input: &str, pos: usize) -> (u32, u32) {
    let pos = pos.min(input.len());
    let prefix = &input[..pos];

    let line = prefix.matches('\n').count().saturating_add(1);
    let line_u32 = (line.min(u32::MAX as usize)) as u32;

    let col_start = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let col_chars = input[col_start..pos].chars().count().saturating_add(1);
    let col_u32 = (col_chars.min(u32::MAX as usize)) as u32;

    (line_u32, col_u32)
}

fn context_snippet(
    input: &str,
    line: u32,
    column: u32,
    before: usize,
    after: usize,
) -> Vec<String> {
    if line == 0 {
        return Vec::new();
    }
    let target = line as usize;
    let start = target.saturating_sub(before).max(1);
    let end = target.saturating_add(after);

    let mut out: Vec<String> = Vec::new();
    for (idx, text) in input.lines().enumerate() {
        let ln = idx + 1;
        if ln < start || ln > end {
            continue;
        }
        out.push(format!("{:>4} | {}", ln, text));
        if ln == target {
            let caret_spaces = " ".repeat(column.saturating_sub(1) as usize);
            out.push(format!("     | {}^", caret_spaces));
        }
    }
    out
}

fn generic_parse_suggestions() -> Vec<String> {
    vec![
        "Check for a missing `}` or `]` near the marked location".to_string(),
        "Ensure strings are quoted (prefer double quotes)".to_string(),
        "Ensure assignments use `=` (e.g. `A = system \"A\"`)".to_string(),
    ]
}

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
        let pos = pos.min(input.len());
        let truncated = &input[..pos];
        let line = truncated.matches('\n').count().saturating_add(1);
        line.min(u32::MAX as usize) as u32
    }

    /// Create a SourceLocation for a position
    #[allow(dead_code)]
    fn location(&self, input: &str, pos: usize) -> SourceLocation {
        let (line, col) = line_col_1_indexed(input, pos);
        SourceLocation::new(self.filename.clone(), line, col)
    }

    /// Parse source code into a Program AST
    pub fn parse(&self, input: &str) -> Result<Program, Vec<Diagnostic>> {
        match parse_program(input) {
            Ok((remaining, program)) => {
                let trimmed = remaining.trim();
                if !trimmed.is_empty() {
                    let preview = if trimmed.len() > 100 {
                        format!("{}...", &trimmed[..100])
                    } else {
                        trimmed.to_string()
                    };

                    let pos = input.len().saturating_sub(remaining.len());
                    let (line, col) = line_col_1_indexed(input, pos);

                    let mut d = Diagnostic::new(
                        sruja_diagnostics::codes::CODE_SYNTAX_ERROR,
                        Severity::Error,
                        format!(
                            "Unexpected input at {}:{}: {}",
                            line,
                            col,
                            preview.replace('\n', "\\n").replace('\r', "\\r")
                        ),
                        SourceLocation::new(self.filename.clone(), line, col),
                    )
                    .with_context(context_snippet(input, line, col, 2, 2))
                    .with_suggestions(generic_parse_suggestions());

                    return Err(vec![{
                        d.context.retain(|s| !s.trim().is_empty());
                        d
                    }]);
                }
                Ok(program)
            }
            Err(e) => {
                let (error_msg, line, col) = match &e {
                    nom::Err::Error(err) => {
                        let pos = input.len().saturating_sub(err.input.len());
                        let (line, col) = line_col_1_indexed(input, pos);
                        let remaining_preview = input
                            .get(pos..)
                            .unwrap_or("")
                            .chars()
                            .take(80)
                            .collect::<String>()
                            .replace('\n', "\\n")
                            .replace('\r', "\\r");
                        (
                            format!("Parse error ({:?}) near: {}", err.code, remaining_preview),
                            line,
                            col,
                        )
                    }
                    nom::Err::Failure(err) => {
                        let pos = input.len().saturating_sub(err.input.len());
                        let (line, col) = line_col_1_indexed(input, pos);
                        let remaining_preview = input
                            .get(pos..)
                            .unwrap_or("")
                            .chars()
                            .take(80)
                            .collect::<String>()
                            .replace('\n', "\\n")
                            .replace('\r', "\\r");
                        (
                            format!("Parse failure ({:?}) near: {}", err.code, remaining_preview),
                            line,
                            col,
                        )
                    }
                    nom::Err::Incomplete(_) => ("Incomplete input".to_string(), 1, 1),
                };

                Err(vec![Diagnostic::new(
                    sruja_diagnostics::codes::CODE_SYNTAX_ERROR,
                    Severity::Error,
                    error_msg,
                    SourceLocation::new(self.filename.clone(), line, col),
                )
                .with_context(context_snippet(input, line, col, 2, 2))
                .with_suggestions(generic_parse_suggestions())])
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
                let (error_msg, line_number, column_number) = match &e {
                    nom::Err::Error(err) => {
                        let pos = context_section.len().saturating_sub(err.input.len());
                        let (context_line, context_col) = line_col_1_indexed(context_section, pos);
                        let absolute_line = context_start_line as u32 + context_line;
                        (
                            format!(
                                "Parse error in context section at {}:{} ({:?})",
                                absolute_line, context_col, err.code
                            ),
                            absolute_line,
                            context_col,
                        )
                    }
                    nom::Err::Failure(err) => {
                        let pos = context_section.len().saturating_sub(err.input.len());
                        let (context_line, context_col) = line_col_1_indexed(context_section, pos);
                        let absolute_line = context_start_line as u32 + context_line;
                        (
                            format!(
                                "Parse failure in context section at {}:{} ({:?})",
                                absolute_line, context_col, err.code
                            ),
                            absolute_line,
                            context_col,
                        )
                    }
                    nom::Err::Incomplete(_) => (
                        "Incomplete input in context section".to_string(),
                        context_start_line.saturating_add(1) as u32,
                        1,
                    ),
                };

                Err(vec![Diagnostic::new(
                    sruja_diagnostics::codes::CODE_SYNTAX_ERROR,
                    Severity::Error,
                    error_msg,
                    SourceLocation::new(self.filename.clone(), line_number, column_number),
                )
                .with_context(context_snippet(input, line_number, column_number, 2, 2))
                .with_suggestions(generic_parse_suggestions())])
            }
        }
    }
}
