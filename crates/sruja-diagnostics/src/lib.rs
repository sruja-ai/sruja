//! Diagnostics system for Sruja DSL.
//!
//! This module provides structured error and warning reporting with source locations,
//! context, and suggestions. It's designed to be compatible with the Go implementation
//! while leveraging Rust's type safety and performance.
//!
//! # Examples
//!
//! ```rust
//! use sruja_diagnostics::{Diagnostic, Severity, SourceLocation, BasicErrorReporter};
//!
//! let mut reporter = BasicErrorReporter::new();
//! let diag = Diagnostic::new(
//!     "E101",
//!     Severity::Error,
//!     "unexpected token",
//!     SourceLocation::new("test.sruja".to_string(), 1, 1),
//! );
//! reporter.report(diag);
//! ```

use serde::{Deserialize, Serialize};

/// Standard error codes for diagnostics.
///
/// These codes follow a hierarchical naming scheme:
/// - E1xx: Syntax errors
/// - E2xx: Semantic errors
/// - E3xx: Validation errors
/// - E4xx: Policy errors
/// - W001: Best practice warnings
pub mod codes {
    // Syntax Errors (E1xx)
    /// Generic syntax error
    pub const CODE_SYNTAX_ERROR: &str = "E101";
    /// Token was not expected in this context
    pub const CODE_UNEXPECTED_TOKEN: &str = "E102";
    /// Missing closing brace or bracket
    pub const CODE_MISSING_BRACE: &str = "E103";
    /// Invalid or malformed string literal
    pub const CODE_INVALID_STRING: &str = "E104";

    // Semantic Errors (E2xx)
    /// Duplicate identifier found in scope
    pub const CODE_DUPLICATE_ID: &str = "E201";
    /// Reference to undefined identifier
    pub const CODE_UNDEFINED_REF: &str = "E202";
    /// Invalid relationship between elements
    pub const CODE_INVALID_RELATION: &str = "E203";
    /// Cycle detected in dependency graph
    pub const CODE_CYCLE_DETECTED: &str = "E204";
    /// Element has no parent or connection
    pub const CODE_ORPHAN_ELEMENT: &str = "E205";
    /// Layer architecture constraint violation
    pub const CODE_LAYER_VIOLATION: &str = "E206";

    // Validation Errors (E3xx)
    /// Property value fails validation
    pub const CODE_INVALID_PROPERTY: &str = "E301";
    /// Required field is missing
    pub const CODE_MISSING_FIELD: &str = "E302";
    /// Custom validation rule failed
    pub const CODE_VALIDATION_RULE_ERROR: &str = "E303";
    /// Validation operation timed out
    pub const CODE_VALIDATION_TIMEOUT: &str = "E304";
    /// Validation logic panicked
    pub const CODE_VALIDATION_PANIC: &str = "E305";
    /// Alias for duplicate identifier
    pub const CODE_DUPLICATE_IDENTIFIER: &str = "E201";
    /// Alias for reference not found
    pub const CODE_REFERENCE_NOT_FOUND: &str = "E202";

    // Warnings
    /// Best practice suggestion
    pub const CODE_BEST_PRACTICE: &str = "W001";

    // Policy Errors (E4xx)
    /// Policy constraint violation
    pub const CODE_POLICY_VIOLATION: &str = "E401";
}

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub enum Severity {
    /// Error that prevents compilation or execution
    Error,
    /// Warning about potential issues
    Warning,
    /// Informational message
    Info,
}

impl Severity {
    /// Returns the string representation of the severity.
    #[inline]
    #[must_use]
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

/// Represents a location in a source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SourceLocation {
    /// Path to the source file
    pub file: String,
    /// Line number (1-indexed)
    pub line: u32,
    /// Column number (1-indexed)
    pub column: u32,
}

impl SourceLocation {
    /// Creates a new source location.
    #[must_use]
    pub fn new(file: String, line: u32, column: u32) -> Self {
        Self { file, line, column }
    }
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// A diagnostic message (error, warning, or info).
///
/// Diagnostics contain all information needed to display rich error messages
/// to users, including source location, context, and actionable suggestions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Diagnostic {
    /// Unique error code (e.g., "E001", "W001")
    pub code: String,
    /// Severity level of the diagnostic
    pub severity: Severity,
    /// Main error/warning message
    pub message: String,
    /// Source location where the diagnostic occurred
    pub location: SourceLocation,
    /// Surrounding lines of code for context
    pub context: Vec<String>,
    /// Actionable suggestions for fixing the issue
    pub suggestions: Vec<String>,
}

impl Diagnostic {
    /// Creates a new diagnostic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
    ///
    /// let diag = Diagnostic::new(
    ///     "E101",
    ///     Severity::Error,
    ///     "unexpected token",
    ///     SourceLocation::new("test.sruja".to_string(), 1, 1),
    /// );
    /// ```
    #[must_use]
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

    /// Adds context lines to the diagnostic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
    ///
    /// let diag = Diagnostic::new(
    ///     "E101",
    ///     Severity::Error,
    ///     "unexpected token",
    ///     SourceLocation::new("test.sruja".to_string(), 1, 1),
    /// ).with_context(vec![
    ///     "system A {".to_string(),
    ///     "  foo".to_string(),
    ///     "}".to_string(),
    /// ]);
    /// ```
    #[must_use]
    pub fn with_context(mut self, context: Vec<String>) -> Self {
        self.context = context;
        self
    }

    /// Adds suggestions to the diagnostic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
    ///
    /// let diag = Diagnostic::new(
    ///     "E101",
    ///     Severity::Error,
    ///     "unexpected token",
    ///     SourceLocation::new("test.sruja".to_string(), 1, 1),
    /// ).with_suggestions(vec![
    ///     "Did you mean 'component'?".to_string(),
    /// ]);
    /// ```
    #[must_use]
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

/// Error reporter trait for collecting and reporting diagnostics.
///
/// Implementations of this trait can collect diagnostics and provide
/// information about whether errors have been reported.
pub trait ErrorReporter {
    /// Reports a diagnostic.
    fn report(&mut self, diagnostic: Diagnostic);

    /// Returns `true` if any error-level diagnostics have been reported.
    #[must_use]
    fn has_errors(&self) -> bool;

    /// Returns a slice of all reported diagnostics.
    #[must_use]
    fn diagnostics(&self) -> &[Diagnostic];
}

/// Basic in-memory implementation of [`ErrorReporter`].
///
/// This reporter collects all diagnostics in memory and provides
/// simple methods to query the collected diagnostics.
#[derive(Debug, Default)]
pub struct BasicErrorReporter {
    diagnostics: Vec<Diagnostic>,
}

impl BasicErrorReporter {
    /// Creates a new empty error reporter.
    ///
    /// Pre-allocates space for 8 diagnostics to reduce allocations
    /// in common use cases.
    #[must_use]
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::with_capacity(8),
        }
    }

    /// Clears all reported diagnostics.
    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }

    /// Returns the number of diagnostics reported.
    #[must_use]
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Returns `true` if no diagnostics have been reported.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

impl ErrorReporter for BasicErrorReporter {
    fn report(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    #[inline]
    fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    #[inline]
    fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
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

    #[test]
    fn test_source_location_display() {
        let loc = SourceLocation::new("test.sruja".to_string(), 10, 5);

        // Act & Assert
        assert_eq!(loc.to_string(), "test.sruja:10:5");
    }

    #[test]
    fn test_diagnostic_display() {
        // Arrange
        let diag = Diagnostic::new(
            codes::CODE_SYNTAX_ERROR,
            Severity::Error,
            "test error",
            SourceLocation::new("test.sruja".to_string(), 1, 1),
        );

        // Act & Assert
        assert!(diag.to_string().contains("[E101]"));
        assert!(diag.to_string().contains("Error"));
        assert!(diag.to_string().contains("test error"));
    }

    #[test]
    fn test_basic_error_reporter_empty() {
        // Arrange
        let reporter = BasicErrorReporter::new();

        // Assert
        assert!(reporter.is_empty());
        assert_eq!(reporter.len(), 0);
        assert!(!reporter.has_errors());
        assert!(reporter.diagnostics().is_empty());
    }

    #[test]
    fn test_basic_error_reporter_with_diagnostics() {
        // Arrange
        let mut reporter = BasicErrorReporter::new();

        // Act - Report Info
        reporter.report(Diagnostic::new(
            codes::CODE_ORPHAN_ELEMENT,
            Severity::Info,
            "info message",
            SourceLocation::new("test.sruja".to_string(), 1, 1),
        ));

        // Assert
        assert!(!reporter.has_errors());
        assert_eq!(reporter.diagnostics().len(), 1);
        assert_eq!(reporter.len(), 1);
        assert!(!reporter.is_empty());

        // Act - Report Error
        reporter.report(Diagnostic::new(
            codes::CODE_SYNTAX_ERROR,
            Severity::Error,
            "error message",
            SourceLocation::new("test.sruja".to_string(), 2, 1),
        ));

        // Assert
        assert!(reporter.has_errors());
        assert_eq!(reporter.diagnostics().len(), 2);
        assert_eq!(reporter.len(), 2);
    }

    #[test]
    fn test_basic_error_reporter_clear() {
        // Arrange
        let mut reporter = BasicErrorReporter::new();
        reporter.report(Diagnostic::new(
            codes::CODE_SYNTAX_ERROR,
            Severity::Error,
            "error message",
            SourceLocation::new("test.sruja".to_string(), 1, 1),
        ));

        // Act
        reporter.clear();

        // Assert
        assert!(reporter.is_empty());
        assert!(!reporter.has_errors());
        assert_eq!(reporter.len(), 0);
    }

    #[test]
    fn test_format_diagnostic_full() {
        // Arrange
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

        // Act
        let formatted = format_diagnostic(&diag);

        // Assert
        assert!(formatted.contains("[E101] Error: unexpected token 'foo'"));
        assert!(formatted.contains("--> test.sruja:5:10"));
        assert!(formatted.contains("| system A {"));
        assert!(formatted.contains("|   foo"));
        assert!(formatted.contains("| }"));
        assert!(formatted.contains("= Help: Did you mean 'component'?"));
        assert!(formatted.contains("Did you mean 'container'?"));
    }

    #[test]
    fn test_format_diagnostic_minimal() {
        // Arrange
        let diag = Diagnostic::new(
            codes::CODE_SYNTAX_ERROR,
            Severity::Error,
            "simple error",
            SourceLocation::new("test.sruja".to_string(), 1, 1),
        );

        // Act
        let formatted = format_diagnostic(&diag);

        // Assert - Should not contain context or suggestions sections
        assert!(formatted.contains("[E101] Error: simple error"));
        assert!(formatted.contains("--> test.sruja:1:1"));
        assert!(!formatted.contains("| "));
        assert!(!formatted.contains("= Help:"));
    }

    #[test]
    fn test_severity_as_str() {
        assert_eq!(Severity::Error.as_str(), "Error");
        assert_eq!(Severity::Warning.as_str(), "Warning");
        assert_eq!(Severity::Info.as_str(), "Info");
    }
}
