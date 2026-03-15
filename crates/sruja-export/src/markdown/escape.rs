//! Markdown escaping for safe embedding of user content.
//!
//! Escapes characters that would break Markdown structure or inline formatting
//! (backslash, backtick, square brackets). Headings also escape `#` so user
//! titles do not create unintended sub-headings.

/// Escape text used in a Markdown heading (e.g. `### Title`).
/// Escapes `\`, `` ` ``, `[`, `]`, and `#` so the heading renders as plain text.
pub fn escape_heading(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '`' => out.push_str("\\`"),
            '[' => out.push_str("\\["),
            ']' => out.push_str("\\]"),
            '#' => out.push_str("\\#"),
            _ => out.push(c),
        }
    }
    out
}

/// Escape text used in Markdown body (paragraphs, list items).
/// Escapes `\`, `` ` ``, `[`, `]` to avoid broken links and inline code.
pub fn escape_inline(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '`' => out.push_str("\\`"),
            '[' => out.push_str("\\["),
            ']' => out.push_str("\\]"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_heading_escapes_special_chars() {
        assert_eq!(escape_heading("Hello"), "Hello");
        assert_eq!(escape_heading("C# API"), "C\\# API");
        assert_eq!(escape_heading("See [link]"), "See \\[link\\]");
        assert_eq!(escape_heading("Use `code`"), "Use \\`code\\`");
        assert_eq!(escape_heading("Path\\to\\file"), "Path\\\\to\\\\file");
    }

    #[test]
    fn escape_inline_does_not_escape_hash() {
        assert_eq!(escape_inline("C#"), "C#");
        assert_eq!(escape_inline("[a] and `b`"), "\\[a\\] and \\`b\\`");
    }
}
