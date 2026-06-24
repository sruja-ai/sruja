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

// Tests are defined inline below

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_col_1_indexed_first_line() {
        let input = "hello world";
        let (line, col) = line_col_1_indexed(input, 5);
        assert_eq!(line, 1);
        assert_eq!(col, 6);
    }

    #[test]
    fn test_line_col_1_indexed_second_line() {
        let input = "hello\nworld";
        let (line, col) = line_col_1_indexed(input, 7);
        assert_eq!(line, 2);
        assert_eq!(col, 2); // Position 7 is 'o' in "world" (w=1, o=2)
    }

    #[test]
    fn test_line_col_1_indexed_empty_input() {
        let input = "";
        let (line, col) = line_col_1_indexed(input, 0);
        assert_eq!(line, 1);
        assert_eq!(col, 1);
    }

    #[test]
    fn test_line_col_1_indexed_at_newline() {
        let input = "hello\nworld";
        let (line, col) = line_col_1_indexed(input, 5);
        assert_eq!(line, 1);
        assert_eq!(col, 6);
    }

    #[test]
    fn test_line_col_1_indexed_pos_exceeds_len() {
        let input = "hello";
        let (line, col) = line_col_1_indexed(input, 100);
        assert_eq!(line, 1);
        assert_eq!(col, 6);
    }

    #[test]
    fn test_context_snippet_basic() {
        let input = "line1\nline2\nline3\nline4\nline5";
        let snippet = context_snippet(input, 3, 1, 1, 1);
        assert!(snippet.iter().any(|s| s.contains("line2")));
        assert!(snippet.iter().any(|s| s.contains("line3")));
        assert!(snippet.iter().any(|s| s.contains("line4")));
    }

    #[test]
    fn test_context_snippet_with_caret() {
        let input = "hello world";
        let snippet = context_snippet(input, 1, 6, 0, 0);
        assert!(snippet.iter().any(|s| s.contains("^")));
    }

    #[test]
    fn test_context_snippet_zero_line() {
        let input = "hello";
        let snippet = context_snippet(input, 0, 1, 2, 2);
        assert!(snippet.is_empty());
    }

    #[test]
    fn test_count_char_basic() {
        assert_eq!(count_char("hello", 'l'), 2);
        assert_eq!(count_char("hello", 'o'), 1);
        assert_eq!(count_char("hello", 'x'), 0);
    }

    #[test]
    fn test_count_char_empty() {
        assert_eq!(count_char("", 'a'), 0);
    }

    #[test]
    fn test_detect_common_syntax_diagnostic_architecture_block() {
        let input = r#"architecture "My Arch" { }"#;
        let remaining = input;
        let result = detect_common_syntax_diagnostic(input, remaining);
        assert!(result.is_some());
        let (code, msg, _) = result.unwrap();
        assert_eq!(code, sruja_diagnostics::codes::CODE_SYNTAX_ERROR);
        assert!(msg.contains("architecture"));
    }

    #[test]
    fn test_detect_common_syntax_diagnostic_missing_brace() {
        let input = r#"MySystem = system "My System" {"#;
        let remaining = "";
        let result = detect_common_syntax_diagnostic(input, remaining);
        assert!(result.is_some());
        let (code, msg, _) = result.unwrap();
        assert_eq!(code, sruja_diagnostics::codes::CODE_MISSING_BRACE);
        assert!(msg.contains("Missing closing `}`"));
    }

    #[test]
    fn test_detect_common_syntax_diagnostic_missing_bracket() {
        let input = r#"tags ["a", "b""#;
        let remaining = "";
        let result = detect_common_syntax_diagnostic(input, remaining);
        assert!(result.is_some());
        let (code, msg, _) = result.unwrap();
        assert_eq!(code, sruja_diagnostics::codes::CODE_MISSING_BRACE);
        assert!(msg.contains("Missing closing `]`"));
    }

    #[test]
    fn test_detect_common_syntax_diagnostic_unterminated_string() {
        let input = r#"MySystem = system "My System"#;
        let remaining = "";
        let result = detect_common_syntax_diagnostic(input, remaining);
        assert!(result.is_some());
        let (code, msg, _) = result.unwrap();
        assert_eq!(code, sruja_diagnostics::codes::CODE_INVALID_STRING);
        assert!(msg.contains("Unterminated string"));
    }

    #[test]
    fn test_detect_common_syntax_diagnostic_equals_without_identifier() {
        let input = "= system \"My System\"";
        let remaining = "= system \"My System\"";
        let result = detect_common_syntax_diagnostic(input, remaining);
        assert!(result.is_some());
        let (code, msg, _) = result.unwrap();
        assert_eq!(code, sruja_diagnostics::codes::CODE_UNEXPECTED_TOKEN);
        assert!(msg.contains("Expected an identifier before `=`"));
    }

    #[test]
    fn test_detect_common_syntax_diagnostic_arrow_without_source() {
        let input = "-> SystemB \"calls\"";
        let remaining = "-> SystemB \"calls\"";
        let result = detect_common_syntax_diagnostic(input, remaining);
        assert!(result.is_some());
        let (code, msg, _) = result.unwrap();
        assert_eq!(code, sruja_diagnostics::codes::CODE_UNEXPECTED_TOKEN);
        assert!(msg.contains("Expected an identifier before a relationship arrow"));
    }

    #[test]
    fn test_detect_common_syntax_diagnostic_unexpected_closing_delimiter() {
        let input = "}";
        let remaining = "}";
        let result = detect_common_syntax_diagnostic(input, remaining);
        assert!(result.is_some());
        let (code, msg, _) = result.unwrap();
        assert_eq!(code, sruja_diagnostics::codes::CODE_UNEXPECTED_TOKEN);
        assert!(msg.contains("Unexpected closing delimiter"));
    }

    #[test]
    fn test_detect_common_syntax_diagnostic_missing_identifier_assignment() {
        let input = "system \"My System\"";
        let remaining = "system \"My System\"";
        let result = detect_common_syntax_diagnostic(input, remaining);
        assert!(result.is_some());
        let (code, msg, _) = result.unwrap();
        assert_eq!(code, sruja_diagnostics::codes::CODE_SYNTAX_ERROR);
        assert!(msg.contains("Missing identifier assignment"));
    }

    #[test]
    fn test_detect_common_syntax_diagnostic_relationship_without_label() {
        let input = "A -> B";
        let remaining = "A -> B";
        let result = detect_common_syntax_diagnostic(input, remaining);
        assert!(result.is_some());
        let (code, msg, _) = result.unwrap();
        assert_eq!(code, sruja_diagnostics::codes::CODE_SYNTAX_ERROR);
        assert!(msg.contains("Relationships must have a double-quoted label"));
    }

    #[test]
    fn test_detect_common_syntax_diagnostic_no_match() {
        let input = "normal code";
        let remaining = "normal code";
        let result = detect_common_syntax_diagnostic(input, remaining);
        assert!(result.is_none());
    }

    #[test]
    fn test_advance_past_current_line_basic() {
        let input = "line1\nline2\nline3";
        let result = advance_past_current_line(input, 0);
        assert_eq!(result, 6); // After "line1\n"
    }

    #[test]
    fn test_advance_past_current_line_at_end() {
        let input = "line1\nline2";
        let result = advance_past_current_line(input, 10);
        assert_eq!(result, 11); // End of input
    }

    #[test]
    fn test_advance_past_current_line_no_newline() {
        let input = "single line";
        let result = advance_past_current_line(input, 0);
        assert_eq!(result, 11);
    }

    #[test]
    fn test_advance_one_utf8_char_ascii() {
        let input = "hello";
        let result = advance_one_utf8_char(input, 0);
        assert_eq!(result, 1);
    }

    #[test]
    fn test_advance_one_utf8_char_multibyte() {
        let input = "héllo";
        let result = advance_one_utf8_char(input, 0);
        assert_eq!(result, 1); // 'h' is 1 byte
        let result = advance_one_utf8_char(input, 1);
        assert_eq!(result, 3); // 'é' is 2 bytes
    }

    #[test]
    fn test_advance_one_utf8_char_at_end() {
        let input = "hello";
        let result = advance_one_utf8_char(input, 5);
        assert_eq!(result, 5);
    }

    #[test]
    fn test_advance_one_utf8_char_empty() {
        let input = "";
        let result = advance_one_utf8_char(input, 0);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_nom_err_remaining_input_error() {
        let err = nom::Err::Error(nom::error::Error::new("hello", nom::error::ErrorKind::Tag));
        let result = nom_err_remaining_input(&err);
        assert_eq!(result, Some("hello"));
    }

    #[test]
    fn test_nom_err_remaining_input_failure() {
        let err = nom::Err::Failure(nom::error::Error::new("world", nom::error::ErrorKind::Tag));
        let result = nom_err_remaining_input(&err);
        assert_eq!(result, Some("world"));
    }

    #[test]
    fn test_nom_err_remaining_input_incomplete() {
        let err = nom::Err::Incomplete(nom::Needed::new(5));
        let result = nom_err_remaining_input(&err);
        assert!(result.is_none());
    }

    #[test]
    fn test_parser_new() {
        let parser = Parser::new("test.sruja");
        assert_eq!(parser.filename, "test.sruja");
    }

    #[test]
    fn test_parser_new_with_string() {
        let parser = Parser::new("test.sruja".to_string());
        assert_eq!(parser.filename, "test.sruja");
    }

    #[test]
    fn test_parser_line_number() {
        let parser = Parser::new("test.sruja");
        let input = "line1\nline2\nline3";
        assert_eq!(parser.line_number(input, 0), 1);
        assert_eq!(parser.line_number(input, 6), 2);
        assert_eq!(parser.line_number(input, 12), 3);
    }

    #[test]
    fn test_parser_location() {
        let parser = Parser::new("test.sruja");
        let input = "line1\nline2";
        let loc = parser.location(input, 7);
        assert_eq!(loc.file, "test.sruja");
        assert_eq!(loc.line, 2);
        assert_eq!(loc.column, 2); // Position 7 is 'i' in "line2" (l=1, i=2)
    }

    #[test]
    fn test_build_diagnostic_from_nom_err() {
        let input = "hello world";
        let err = nom::Err::Error(nom::error::Error::new("world", nom::error::ErrorKind::Tag));
        let diag = build_diagnostic_from_nom_err("test.sruja", input, &err);
        assert_eq!(diag.location.file, "test.sruja");
        assert!(diag.message.contains("Parse error"));
    }

    #[test]
    fn test_generic_parse_suggestions() {
        let suggestions = generic_parse_suggestions();
        assert_eq!(suggestions.len(), 3);
        assert!(suggestions[0].contains("missing `}`"));
        assert!(suggestions[1].contains("quoted"));
        assert!(suggestions[2].contains("assignments"));
    }

    #[test]
    fn test_parser_parse_empty_input() {
        let parser = Parser::new("test.sruja");
        let result = parser.parse("");
        assert!(result.is_ok());
        let program = result.unwrap();
        assert!(program.items.is_empty());
    }

    #[test]
    fn test_parser_parse_whitespace_only() {
        let parser = Parser::new("test.sruja");
        let result = parser.parse("   \n  \n  ");
        assert!(result.is_ok());
        let program = result.unwrap();
        assert!(program.items.is_empty());
    }

    #[test]
    fn test_parser_parse_valid_element() {
        let parser = Parser::new("test.sruja");
        let result = parser.parse(r#"MySystem = system "My System""#);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parser_parse_multiple_errors() {
        let parser = Parser::new("test.sruja");
        let input = "=\n->";
        let result = parser.parse(input);
        assert!(result.is_err());
        let diags = result.err().unwrap();
        assert!(diags.len() >= 2);
    }

    #[test]
    fn test_parser_parse_with_comments() {
        let parser = Parser::new("test.sruja");
        let input = r#"
// This is a comment
MySystem = system "My System"
/* Multi-line
   comment */
SystemA -> SystemB "Uses"
"#;
        let result = parser.parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parser_parse_incrementally() {
        let parser = Parser::new("test.sruja");
        let input = "A = system \"A\"\nB = system \"B\"\nA -> B \"uses\"\n";
        let existing = parser.parse(input).expect("initial parse");
        let edited = "A = system \"A\"\nB = system \"B Updated\"\nA -> B \"uses\"\n";
        let change_start = 22;
        let change_end = 35;
        let result = parser.parse_incrementally(edited, change_start, change_end, &existing, 2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parser_parse_incrementally_error() {
        let parser = Parser::new("test.sruja");
        let input = "A = system \"A\"\nB = system \"B\"\nA -> B \"uses\"\n";
        let existing = parser.parse(input).expect("initial parse");
        let edited = "A = system \"A\"\n= invalid\nA -> B \"uses\"\n";
        let result = parser.parse_incrementally(edited, 16, 26, &existing, 2);
        assert!(result.is_err());
    }
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
            let preview = if final_tail.chars().count() > 100 {
                format!("{}...", final_tail.chars().take(100).collect::<String>())
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
