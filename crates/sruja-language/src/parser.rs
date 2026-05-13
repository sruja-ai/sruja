//! Parser for Sruja DSL using nom parser combinators
//!
//! This module implements a parser for the Sruja DSL using `nom` parser combinators.
//! It parses source code directly into an AST without a separate lexing phase.

mod assignments;
mod blocks;
mod contracts;
mod deployment;
mod elements;
mod import;
mod loops;
mod merge;
mod overview_views;
mod primitives;
mod program;
mod relations;
mod schema;
mod state_machine;

#[cfg(test)]
mod tests;

use primitives::{line_to_byte_offset, ws};
use program::{parse_program, parse_top_level_item};
use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};

use crate::ast::*;

fn line_col_1_indexed(input: &str, pos: usize) -> (u32, u32) {
    let pos = pos.min(input.len());
    let prefix = &input[..pos];

    let line = prefix.matches('\n').count().saturating_add(1);
    let line_u32 = if line > u32::MAX as usize {
        u32::MAX
    } else {
        line as u32
    };

    let col_start = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let col_chars = input[col_start..pos].chars().count().saturating_add(1);
    let col_u32 = if col_chars > u32::MAX as usize {
        u32::MAX
    } else {
        col_chars as u32
    };

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

fn count_char(input: &str, ch: char) -> usize {
    input.chars().filter(|c| *c == ch).count()
}

fn detect_common_syntax_diagnostic(
    input: &str,
    remaining: &str,
) -> Option<(&'static str, String, Vec<String>)> {
    let remaining_trimmed = remaining.trim_start();

    if input.trim_start().starts_with("architecture") {
        return Some((
            sruja_diagnostics::codes::CODE_SYNTAX_ERROR,
            "Top-level `architecture \"...\" { ... }` blocks are not supported. Keep declarations at the top level (no wrapper).".to_string(),
            vec![
                "Remove the outer `architecture \"...\" {` and matching `}`".to_string(),
                "Put `MySystem = system \"...\" { ... }` and `A -> B \"...\"` directly in the file".to_string(),
            ],
        ));
    }

    if count_char(input, '{') > count_char(input, '}') {
        return Some((
            sruja_diagnostics::codes::CODE_MISSING_BRACE,
            "Missing closing `}`".to_string(),
            vec![
                "Add a matching `}` for the last opened `{`".to_string(),
                "If you intended a single-line element, remove the `{ ... }` block".to_string(),
            ],
        ));
    }

    if count_char(input, '[') > count_char(input, ']') {
        return Some((
            sruja_diagnostics::codes::CODE_MISSING_BRACE,
            "Missing closing `]`".to_string(),
            vec![
                "Add a matching `]` for the last opened `[`".to_string(),
                "Check list syntax: tags [\"a\", \"b\"]".to_string(),
            ],
        ));
    }

    if count_char(input, '"') % 2 == 1 || count_char(input, '\'') % 2 == 1 {
        return Some((
            sruja_diagnostics::codes::CODE_INVALID_STRING,
            "Unterminated string literal (missing closing quote)".to_string(),
            vec![
                "Close the string with a matching quote".to_string(),
                "Prefer double quotes for strings: \"...\"".to_string(),
            ],
        ));
    }

    if remaining_trimmed.starts_with('=') {
        return Some((
            sruja_diagnostics::codes::CODE_UNEXPECTED_TOKEN,
            "Expected an identifier before `=` (e.g. `MySystem = system \"My System\"`)"
                .to_string(),
            vec![
                "Add a name on the left side: `App = system \"App\" { ... }`".to_string(),
                "If this line is accidental, delete it".to_string(),
            ],
        ));
    }

    if remaining_trimmed.starts_with("->")
        || remaining_trimmed.starts_with("<-")
        || remaining_trimmed.starts_with("<->")
    {
        return Some((
            sruja_diagnostics::codes::CODE_UNEXPECTED_TOKEN,
            "Expected an identifier before a relationship arrow (e.g. `A -> B \"calls\"`)"
                .to_string(),
            vec![
                "Add a source element id before the arrow".to_string(),
                "If you meant a comment, prefix with `//`".to_string(),
            ],
        ));
    }

    if remaining_trimmed.starts_with('}') || remaining_trimmed.starts_with(']') {
        return Some((
            sruja_diagnostics::codes::CODE_UNEXPECTED_TOKEN,
            "Unexpected closing delimiter".to_string(),
            vec![
                "Remove the extra closing delimiter".to_string(),
                "Check for a missing opening `{` or `[` earlier in the file".to_string(),
            ],
        ));
    }

    let trimmed_all = remaining_trimmed.trim();
    if trimmed_all.starts_with("system")
        || trimmed_all.starts_with("container")
        || trimmed_all.starts_with("component")
        || trimmed_all.starts_with("person")
        || trimmed_all.starts_with("database")
        || trimmed_all.starts_with("queue")
    {
        let kw = trimmed_all.split_whitespace().next().unwrap_or("element");
        return Some((
            sruja_diagnostics::codes::CODE_SYNTAX_ERROR,
            format!("Missing identifier assignment before keyword `{}`. Elements must be defined as `ID = {} \"Label\"`.", kw, kw),
            vec![
                format!("Add an identifier and `=` before the keyword, e.g. `MyElement = {} \"Label\"`", kw),
                "Element IDs must be PascalCase".to_string(),
            ],
        ));
    }

    if trimmed_all.contains("->") && !trimmed_all.contains('"') {
        return Some((
            sruja_diagnostics::codes::CODE_SYNTAX_ERROR,
            "Relationships must have a double-quoted label, e.g., `A -> B \"label\"`".to_string(),
            vec![
                "Add a double-quoted label after the target component: `A -> B \"calls\"`"
                    .to_string(),
                "Single quotes are not supported; use double quotes: \"label\"".to_string(),
            ],
        ));
    }

    None
}

fn nom_err_remaining_input<'a>(err: &'a nom::Err<nom::error::Error<&'a str>>) -> Option<&'a str> {
    match err {
        nom::Err::Error(e) | nom::Err::Failure(e) => Some(e.input),
        nom::Err::Incomplete(_) => None,
    }
}

/// After a failed top-level parse, skip to the next line so later statements can still be parsed.
fn advance_past_current_line(input: &str, fail_pos: usize) -> usize {
    let fail_pos = fail_pos.min(input.len());
    if let Some(rel) = input[fail_pos..].find('\n') {
        (fail_pos + rel + 1).min(input.len())
    } else {
        input.len()
    }
}

fn advance_one_utf8_char(input: &str, pos: usize) -> usize {
    let pos = pos.min(input.len());
    input[pos..]
        .chars()
        .next()
        .map(|ch| pos.saturating_add(ch.len_utf8()))
        .unwrap_or(input.len())
        .min(input.len())
}

fn build_diagnostic_from_nom_err(
    filename: &str,
    input: &str,
    err: &nom::Err<nom::error::Error<&str>>,
) -> Diagnostic {
    let (pos, error_msg, line, col) = match err {
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
                pos,
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
                pos,
                format!("Parse failure ({:?}) near: {}", err.code, remaining_preview),
                line,
                col,
            )
        }
        nom::Err::Incomplete(_) => {
            let pos = input.len();
            let (line, col) = line_col_1_indexed(input, pos);
            (pos, "Incomplete input".to_string(), line, col)
        }
    };

    let remaining = input.get(pos..).unwrap_or("");
    let (code, message, suggestions) =
        detect_common_syntax_diagnostic(input, remaining).unwrap_or((
            sruja_diagnostics::codes::CODE_SYNTAX_ERROR,
            error_msg,
            generic_parse_suggestions(),
        ));

    Diagnostic::new(
        code,
        Severity::Error,
        message,
        SourceLocation::new(filename.to_string(), line, col),
    )
    .with_context(context_snippet(input, line, col, 2, 2))
    .with_suggestions(suggestions)
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

    /// Parse source code into a Program AST.
    ///
    /// On syntax errors, collects diagnostics for recoverable failures (skipping to the next
    /// line after each bad statement) so a single run can surface multiple issues.
    pub fn parse(&self, input: &str) -> Result<Program, Vec<Diagnostic>> {
        let mut items = Vec::new();
        let mut errors: Vec<Diagnostic> = Vec::new();
        let mut remaining = input;
        let max_steps = input.len().saturating_add(64);
        let mut steps = 0usize;

        while steps < max_steps {
            steps += 1;
            let (tail, _) = match ws(remaining) {
                Ok(v) => v,
                Err(_) => break,
            };
            if tail.is_empty() {
                remaining = tail;
                break;
            }

            match parse_top_level_item(tail) {
                Ok((next, item)) => {
                    items.push(item);
                    remaining = next;
                }
                Err(e) => {
                    let fail_pos = match nom_err_remaining_input(&e) {
                        Some(inp) => input.len().saturating_sub(inp.len()),
                        None => input.len().saturating_sub(tail.len()),
                    };
                    let mut d = build_diagnostic_from_nom_err(&self.filename, input, &e);
                    d.context.retain(|s| !s.trim().is_empty());
                    errors.push(d);

                    let after_line = advance_past_current_line(input, fail_pos);
                    let next_pos = if after_line > fail_pos {
                        after_line
                    } else {
                        advance_one_utf8_char(input, fail_pos)
                    };
                    remaining = input.get(next_pos..).unwrap_or("");
                }
            }
        }

        let mut program = Program::with_items(Program::new(), items);

        let (final_tail, _) = match ws(remaining) {
            Ok(v) => v,
            Err(_) => (remaining, ()),
        };
        if !final_tail.trim().is_empty() {
            let preview = if final_tail.len() > 100 {
                format!("{}...", &final_tail[..100])
            } else {
                final_tail.to_string()
            };

            let pos = input.len().saturating_sub(final_tail.len());
            let (line, col) = line_col_1_indexed(input, pos);

            let (code, message, suggestions) = detect_common_syntax_diagnostic(input, final_tail)
                .unwrap_or((
                    sruja_diagnostics::codes::CODE_SYNTAX_ERROR,
                    format!(
                        "Unexpected input at {}:{}: {}",
                        line,
                        col,
                        preview.replace('\n', "\\n").replace('\r', "\\r")
                    ),
                    generic_parse_suggestions(),
                ));

            let mut d = Diagnostic::new(
                code,
                Severity::Error,
                message,
                SourceLocation::new(self.filename.clone(), line, col),
            )
            .with_context(context_snippet(input, line, col, 2, 2))
            .with_suggestions(suggestions);
            d.context.retain(|s| !s.trim().is_empty());
            errors.push(d);
        }

        if errors.is_empty() {
            crate::traversal::populate_locations(&mut program, input, &self.filename);
            Ok(program)
        } else {
            Err(errors)
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
        #[cfg(not(target_arch = "wasm32"))]
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

                #[cfg(not(target_arch = "wasm32"))]
                let elapsed = start.elapsed().as_millis() as u64;
                #[cfg(target_arch = "wasm32")]
                let elapsed = 0;

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
