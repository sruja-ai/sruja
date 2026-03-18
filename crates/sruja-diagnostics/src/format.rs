//! Formatting diagnostics for user-facing output.

use crate::types::Diagnostic;

fn escape_github_actions_message(input: &str) -> String {
    input
        .replace('%', "%25")
        .replace('\n', "%0A")
        .replace('\r', "%0D")
}

#[must_use]
pub fn format_github_actions_annotation(d: &Diagnostic) -> String {
    let level = match d.severity {
        crate::types::Severity::Error => "error",
        crate::types::Severity::Warning => "warning",
        crate::types::Severity::Info => "notice",
    };
    let file = if d.location.file.is_empty() {
        "unknown"
    } else {
        d.location.file.as_str()
    };
    let line = d.location.line.max(1);
    let col = d.location.column.max(1);

    let msg = format!("{}: {}", d.code, d.message);
    format!(
        "::{level} file={file},line={line},col={col}::{}",
        escape_github_actions_message(&msg)
    )
}

/// Formats a diagnostic into a user-friendly string representation.
///
/// This simulates a "Rust-like" error message format, compatible with
/// the Go implementation.
///
/// # Examples
///
/// ```rust
/// use sruja_diagnostics::{Diagnostic, Severity, SourceLocation, format_diagnostic};
///
/// let diag = Diagnostic::new(
///     "E101",
///     Severity::Error,
///     "unexpected token",
///     SourceLocation::new("test.sruja".to_string(), 1, 1),
/// );
/// let formatted = format_diagnostic(&diag);
/// assert!(formatted.contains("[E101] Error: unexpected token"));
/// ```
#[must_use]
pub fn format_diagnostic(d: &Diagnostic) -> String {
    // Pre-allocate capacity to reduce allocations
    let mut output = String::with_capacity(128 + d.context.len() * 20 + d.suggestions.len() * 40);

    // Header: [E001] Error: Message
    output.push_str(&format!("[{}] {}: {}\n", d.code, d.severity, d.message));
    output.push_str(&format!("  --> {}\n", d.location));

    // Context snippet
    if !d.context.is_empty() {
        output.push('\n');
        for line in &d.context {
            output.push_str(&format!("  | {}\n", line));
        }
        output.push('\n');
    }

    // Suggestions
    if !d.suggestions.is_empty() {
        output.push_str("  = Help: ");
        output.push_str(&d.suggestions.join("\n          "));
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Diagnostic, Severity, SourceLocation};

    #[test]
    fn format_diagnostic_includes_code_severity_and_message() {
        let diag = Diagnostic::new(
            "E101",
            Severity::Error,
            "unexpected token",
            SourceLocation::new("test.sruja".to_string(), 1, 1),
        );
        let formatted = format_diagnostic(&diag);
        assert!(formatted.contains("[E101]"));
        assert!(formatted.contains("Error:"));
        assert!(formatted.contains("unexpected token"));
    }

    #[test]
    fn format_diagnostic_includes_location() {
        let diag = Diagnostic::new(
            "E102",
            Severity::Warning,
            "missing brace",
            SourceLocation::new("arch.sruja".to_string(), 5, 12),
        );
        let formatted = format_diagnostic(&diag);
        assert!(formatted.contains("-->"));
        assert!(formatted.contains("arch.sruja:5:12"));
    }

    #[test]
    fn format_diagnostic_includes_context_when_present() {
        let mut diag = Diagnostic::new(
            "E103",
            Severity::Error,
            "invalid",
            SourceLocation::new("x.sruja".to_string(), 2, 1),
        );
        diag.context = vec![
            "  A = system \"A\"".to_string(),
            "  B = container \"B\"".to_string(),
        ];
        let formatted = format_diagnostic(&diag);
        assert!(formatted.contains("  | "));
        assert!(formatted.contains("A = system"));
    }

    #[test]
    fn format_diagnostic_includes_suggestions_when_present() {
        let mut diag = Diagnostic::new(
            "E104",
            Severity::Info,
            "consider adding description",
            SourceLocation::new("y.sruja".to_string(), 1, 1),
        );
        diag.suggestions = vec!["Add description \"...\"".to_string()];
        let formatted = format_diagnostic(&diag);
        assert!(formatted.contains("= Help:"));
        assert!(formatted.contains("Add description"));
    }
}
