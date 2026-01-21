//! Diagnostics system for Sruja DSL
//!
//! This module provides structured error and warning reporting with source locations,
//! context, and suggestions. It's designed to be compatible with the Go implementation
//! while leveraging Rust's type safety and performance.

use serde::{Deserialize, Serialize};

/// Standard error codes for diagnostics
pub mod codes {
    // Syntax Errors (E1xx)
    pub const CODE_SYNTAX_ERROR: &str = "E101";
    pub const CODE_UNEXPECTED_TOKEN: &str = "E102";
    pub const CODE_MISSING_BRACE: &str = "E103";
    pub const CODE_INVALID_STRING: &str = "E104";

    // Semantic Errors (E2xx)
    pub const CODE_DUPLICATE_ID: &str = "E201";
    pub const CODE_UNDEFINED_REF: &str = "E202";
    pub const CODE_INVALID_RELATION: &str = "E203";
    pub const CODE_CYCLE_DETECTED: &str = "E204";
    pub const CODE_ORPHAN_ELEMENT: &str = "E205";
    pub const CODE_LAYER_VIOLATION: &str = "E206";

    // Validation Errors (E3xx)
    pub const CODE_INVALID_PROPERTY: &str = "E301";
    pub const CODE_MISSING_FIELD: &str = "E302";
    pub const CODE_VALIDATION_RULE_ERROR: &str = "E303";
    pub const CODE_VALIDATION_TIMEOUT: &str = "E304";
    pub const CODE_VALIDATION_PANIC: &str = "E305";
    pub const CODE_DUPLICATE_IDENTIFIER: &str = "E201"; // Alias
    pub const CODE_REFERENCE_NOT_FOUND: &str = "E202"; // Alias

    // Warnings
    pub const CODE_BEST_PRACTICE: &str = "W001";

    // Policy Errors (E4xx)
    pub const CODE_POLICY_VIOLATION: &str = "E401";
}

/// Severity of a diagnostic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "Error",
            Severity::Warning => "Warning",
            Severity::Info => "Info",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Location in a source file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl SourceLocation {
    pub fn new(file: String, line: u32, column: u32) -> Self {
        Self { file, line, column }
    }
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// A diagnostic (error, warning, or info message)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Unique error code (e.g., "E001")
    pub code: String,
    /// Severity level
    pub severity: Severity,
    /// Main error message
    pub message: String,
    /// Where the error occurred
    pub location: SourceLocation,
    /// Surrounding lines of code for context
    pub context: Vec<String>,
    /// Actionable suggestions
    pub suggestions: Vec<String>,
}

impl Diagnostic {
    pub fn new(
        code: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
        location: SourceLocation,
    ) -> Self {
        Self {
            code: code.into(),
            severity,
            message: message.into(),
            location,
            context: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn with_context(mut self, context: Vec<String>) -> Self {
        self.context = context;
        self
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}: {} at {}",
            self.code, self.severity, self.message, self.location
        )
    }
}

/// Format a diagnostic into a user-friendly string representation
/// This simulates a "Rust-like" error message format, compatible with the Go implementation.
pub fn format_diagnostic(d: &Diagnostic) -> String {
    let mut output = String::new();

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

/// Error reporter interface
pub trait ErrorReporter {
    fn report(&mut self, diagnostic: Diagnostic);
    fn has_errors(&self) -> bool;
    fn diagnostics(&self) -> &[Diagnostic];
}

/// Basic implementation of ErrorReporter
#[derive(Debug, Default)]
pub struct BasicErrorReporter {
    diagnostics: Vec<Diagnostic>,
}

impl BasicErrorReporter {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::with_capacity(8),
        }
    }
}

impl ErrorReporter for BasicErrorReporter {
    fn report(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_location_display() {
        let loc = SourceLocation::new("test.sruja".to_string(), 10, 5);
        assert_eq!(loc.to_string(), "test.sruja:10:5");
    }

    #[test]
    fn test_diagnostic_display() {
        let diag = Diagnostic::new(
            codes::CODE_SYNTAX_ERROR,
            Severity::Error,
            "test error",
            SourceLocation::new("test.sruja".to_string(), 1, 1),
        );
        assert!(diag.to_string().contains("[E101]"));
        assert!(diag.to_string().contains("Error"));
        assert!(diag.to_string().contains("test error"));
    }

    #[test]
    fn test_basic_error_reporter() {
        let mut reporter = BasicErrorReporter::new();

        assert!(!reporter.has_errors());
        assert!(reporter.diagnostics().is_empty());

        // Report Info
        reporter.report(Diagnostic::new(
            codes::CODE_ORPHAN_ELEMENT,
            Severity::Info,
            "info message",
            SourceLocation::new("test.sruja".to_string(), 1, 1),
        ));

        assert!(!reporter.has_errors());
        assert_eq!(reporter.diagnostics().len(), 1);

        // Report Error
        reporter.report(Diagnostic::new(
            codes::CODE_SYNTAX_ERROR,
            Severity::Error,
            "error message",
            SourceLocation::new("test.sruja".to_string(), 2, 1),
        ));

        assert!(reporter.has_errors());
        assert_eq!(reporter.diagnostics().len(), 2);
    }

    #[test]
    fn test_format_diagnostic() {
        let diag = Diagnostic::new(
            codes::CODE_SYNTAX_ERROR,
            Severity::Error,
            "unexpected token 'foo'",
            SourceLocation::new("test.sruja".to_string(), 5, 10),
        )
        .with_context(vec![
            "system A {".to_string(),
            "  foo".to_string(),
            "}".to_string(),
        ])
        .with_suggestions(vec![
            "Did you mean 'component'?".to_string(),
            "Did you mean 'container'?".to_string(),
        ]);

        let formatted = format_diagnostic(&diag);

        assert!(formatted.contains("[E101] Error: unexpected token 'foo'"));
        assert!(formatted.contains("--> test.sruja:5:10"));
        assert!(formatted.contains("| system A {"));
        assert!(formatted.contains("|   foo"));
        assert!(formatted.contains("| }"));
        assert!(formatted.contains("= Help: Did you mean 'component'?"));
        assert!(formatted.contains("Did you mean 'container'?"));
    }
}
