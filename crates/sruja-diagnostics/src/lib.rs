//! Diagnostics system for Sruja DSL.
//!
//! This module provides structured error and warning reporting with source locations,
//! context, and suggestions. It's designed to be compatible with the Go implementation
//! while leveraging Rust's type safety and performance.
//!
//! # Examples
//!
//! ```rust
//! use sruja_diagnostics::{Diagnostic, Severity, SourceLocation, BasicErrorReporter, ErrorReporter};
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

    /// Creates a new diagnostic from a reference (for reuse).
    ///
    /// This is useful when you need to use the same diagnostic
    /// multiple times, such as in tests.
    #[must_use]
    pub fn new_from_ref(
        code: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
        location: &SourceLocation,
    ) -> Self {
        Self {
            code: code.into(),
            severity,
            message: message.into(),
            location: location.clone(),
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

    /// Reports a diagnostic from an owned value.
    ///
    /// This is useful for tests where you create diagnostics
    /// inline and want to pass them directly.
    fn report_owned(&mut self, diagnostic: Diagnostic) {
        self.report(diagnostic);
    }

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

    fn report_owned(&mut self, diagnostic: Diagnostic) {
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
        let mut reporter = BasicErrorReporter::new();

        // Act - Report Info
        let info_loc = SourceLocation::new("test.sruja".to_string(), 1, 1);
        let info_diag = Diagnostic::new(
            codes::CODE_ORPHAN_ELEMENT,
            Severity::Info,
            "info message",
            info_loc,
        );
        reporter.report_owned(info_diag);

        // Assert
        assert!(!reporter.has_errors());
        assert_eq!(reporter.diagnostics().len(), 1);
        assert_eq!(reporter.len(), 1);
        assert!(!reporter.is_empty());

        // Act - Report Error
        let error_loc = SourceLocation::new("test.sruja".to_string(), 2, 1);
        let error_diag = Diagnostic::new(
            codes::CODE_SYNTAX_ERROR,
            Severity::Error,
            "error message",
            error_loc,
        );
        reporter.report_owned(error_diag);

        // Assert
        assert!(reporter.has_errors());
        assert_eq!(reporter.diagnostics().len(), 2);
        assert_eq!(reporter.len(), 2);
    }

    #[test]
    fn test_basic_error_reporter_clear() {
        let mut reporter = BasicErrorReporter::new();

        // Arrange
        let loc = SourceLocation::new("test.sruja".to_string(), 1, 1);
        let diag = Diagnostic::new(
            codes::CODE_SYNTAX_ERROR,
            Severity::Error,
            "error message",
            loc,
        );
        reporter.report_owned(diag);

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

    // Additional comprehensive tests for sruja-diagnostics

    #[test]
    fn test_source_location_new() {
        let loc = SourceLocation::new("file.sruja".to_string(), 10, 20);
        assert_eq!(loc.file, "file.sruja");
        assert_eq!(loc.line, 10);
        assert_eq!(loc.column, 20);
    }

    #[test]
    fn test_source_location_zero_values() {
        let loc = SourceLocation::new("test.sruja".to_string(), 0, 0);
        assert_eq!(loc.line, 0);
        assert_eq!(loc.column, 0);
    }

    #[test]
    fn test_source_location_equality() {
        let loc1 = SourceLocation::new("file.sruja".to_string(), 10, 20);
        let loc2 = SourceLocation::new("file.sruja".to_string(), 10, 20);
        assert_eq!(loc1, loc2);
    }

    #[test]
    fn test_source_location_inequality() {
        let loc1 = SourceLocation::new("file1.sruja".to_string(), 10, 20);
        let loc2 = SourceLocation::new("file2.sruja".to_string(), 10, 20);
        assert_ne!(loc1, loc2);
    }

    #[test]
    fn test_source_location_clone() {
        let loc1 = SourceLocation::new("file.sruja".to_string(), 10, 20);
        let loc2 = loc1.clone();
        assert_eq!(loc1, loc2);
    }

    #[test]
    fn test_severity_equality() {
        assert_eq!(Severity::Error, Severity::Error);
        assert_ne!(Severity::Error, Severity::Warning);
        assert_ne!(Severity::Warning, Severity::Info);
    }

    #[test]
    fn test_severity_copy() {
        let sev1 = Severity::Error;
        let sev2 = sev1;
        assert_eq!(sev1, sev2);
    }

    #[test]
    fn test_diagnostic_new() {
        let loc = SourceLocation::new("test.sruja".to_string(), 5, 10);
        let diag = Diagnostic::new(
            codes::CODE_DUPLICATE_ID,
            Severity::Error,
            "Duplicate identifier",
            loc.clone(),
        );

        assert_eq!(diag.code, codes::CODE_DUPLICATE_ID);
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "Duplicate identifier");
        assert_eq!(diag.location, loc);
        assert!(diag.context.is_empty());
        assert!(diag.suggestions.is_empty());
    }

    #[test]
    fn test_diagnostic_with_context() {
        let loc = SourceLocation::new("test.sruja".to_string(), 5, 10);
        let diag = Diagnostic::new(
            codes::CODE_SYNTAX_ERROR,
            Severity::Error,
            "Syntax error",
            loc.clone(),
        )
        .with_context(vec![
            "system A {".to_string(),
            "  invalid".to_string(),
            "}".to_string(),
        ]);

        assert_eq!(diag.context.len(), 3);
        assert_eq!(diag.context[0], "system A {");
    }

    #[test]
    fn test_diagnostic_with_suggestions() {
        let loc = SourceLocation::new("test.sruja".to_string(), 5, 10);
        let diag = Diagnostic::new(
            codes::CODE_REFERENCE_NOT_FOUND,
            Severity::Error,
            "Reference not found",
            loc.clone(),
        )
        .with_suggestions(vec![
            "Check if identifier is defined".to_string(),
            "Verify spelling".to_string(),
        ]);

        assert_eq!(diag.suggestions.len(), 2);
        assert_eq!(diag.suggestions[0], "Check if identifier is defined");
    }

    #[test]
    fn test_diagnostic_chain_methods() {
        let loc = SourceLocation::new("test.sruja".to_string(), 5, 10);
        let diag = Diagnostic::new(
            codes::CODE_CYCLE_DETECTED,
            Severity::Error,
            "Cycle detected",
            loc.clone(),
        )
        .with_context(vec!["line 1".to_string()])
        .with_suggestions(vec!["suggestion".to_string()]);

        assert_eq!(diag.context.len(), 1);
        assert_eq!(diag.suggestions.len(), 1);
        assert_eq!(diag.code, codes::CODE_CYCLE_DETECTED);
    }

    #[test]
    fn test_diagnostic_clone() {
        let loc = SourceLocation::new("test.sruja".to_string(), 5, 10);
        let diag1 = Diagnostic::new(
            codes::CODE_INVALID_PROPERTY,
            Severity::Warning,
            "Invalid property",
            loc.clone(),
        );
        let diag2 = diag1.clone();
        assert_eq!(diag1, diag2);
    }

    #[test]
    fn test_diagnostic_equality() {
        let loc1 = SourceLocation::new("test.sruja".to_string(), 5, 10);
        let loc2 = SourceLocation::new("test.sruja".to_string(), 5, 10);
        let diag1 = Diagnostic::new("E001", Severity::Error, "Error", loc1);
        let diag2 = Diagnostic::new("E001", Severity::Error, "Error", loc2);
        assert_eq!(diag1, diag2);
    }

    #[test]
    fn test_basic_error_reporter_with_multiple_diagnostics() {
        let mut reporter = BasicErrorReporter::new();

        // Pre-create diagnostics to avoid any borrow checker issues
        let loc1 = SourceLocation::new("test.sruja".to_string(), 1, 1);
        let diag1 = Diagnostic::new(codes::CODE_SYNTAX_ERROR, Severity::Error, "Error 1", loc1);
        let loc2 = SourceLocation::new("test.sruja".to_string(), 2, 1);
        let diag2 = Diagnostic::new(
            codes::CODE_DUPLICATE_ID,
            Severity::Warning,
            "Warning 1",
            loc2,
        );
        let loc3 = SourceLocation::new("test.sruja".to_string(), 3, 1);
        let diag3 = Diagnostic::new(codes::CODE_ORPHAN_ELEMENT, Severity::Info, "Info 1", loc3);

        reporter.report_owned(diag1);
        reporter.report_owned(diag2);
        reporter.report_owned(diag3);

        assert_eq!(reporter.len(), 3);
        assert!(reporter.has_errors()); // Has 1 error
    }

    #[test]
    fn test_basic_error_reporter_only_warnings() {
        let mut reporter = BasicErrorReporter::new();

        // Pre-create diagnostics to avoid any borrow checker issues
        let loc1 = SourceLocation::new("test.sruja".to_string(), 1, 1);
        let diag1 = Diagnostic::new(
            codes::CODE_BEST_PRACTICE,
            Severity::Warning,
            "Warning 1",
            loc1,
        );
        let loc2 = SourceLocation::new("test.sruja".to_string(), 2, 1);
        let diag2 = Diagnostic::new(
            codes::CODE_BEST_PRACTICE,
            Severity::Warning,
            "Warning 2",
            loc2,
        );

        reporter.report_owned(diag1);
        reporter.report_owned(diag2);

        assert_eq!(reporter.len(), 2);
        assert!(!reporter.has_errors()); // No errors, only warnings
    }

    #[test]
    fn test_basic_error_reporter_diagnostics_slice() {
        let mut reporter = BasicErrorReporter::new();

        let loc = SourceLocation::new("test.sruja".to_string(), 1, 1);
        let diag1 = Diagnostic::new(codes::CODE_SYNTAX_ERROR, Severity::Error, "Error", loc);
        reporter.report(diag1.clone());

        let diagnostics = reporter.diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0], diag1);
    }

    #[test]
    fn test_format_diagnostic_with_only_context() {
        let loc = SourceLocation::new("test.sruja".to_string(), 1, 1);
        let diag = Diagnostic::new(
            codes::CODE_SYNTAX_ERROR,
            Severity::Error,
            "Error",
            loc.clone(),
        )
        .with_context(vec!["line 1".to_string(), "line 2".to_string()]);

        let formatted = format_diagnostic(&diag);
        assert!(formatted.contains("| line 1"));
        assert!(formatted.contains("| line 2"));
        assert!(!formatted.contains("= Help:"));
    }

    #[test]
    fn test_format_diagnostic_with_only_suggestions() {
        let loc = SourceLocation::new("test.sruja".to_string(), 1, 1);
        let diag = Diagnostic::new(
            codes::CODE_SYNTAX_ERROR,
            Severity::Error,
            "Error",
            loc.clone(),
        )
        .with_suggestions(vec![
            "Suggestion 1".to_string(),
            "Suggestion 2".to_string(),
            "Suggestion 3".to_string(),
        ]);

        let formatted = format_diagnostic(&diag);
        assert!(!formatted.contains("| "));
        assert!(formatted.contains("= Help:"));
        assert!(formatted.contains("Suggestion 1"));
        assert!(formatted.contains("Suggestion 2"));
        assert!(formatted.contains("Suggestion 3"));
    }

    #[test]
    fn test_format_diagnostic_warning() {
        let loc = SourceLocation::new("test.sruja".to_string(), 1, 1);
        let diag = Diagnostic::new(
            codes::CODE_BEST_PRACTICE,
            Severity::Warning,
            "Best practice warning",
            loc.clone(),
        );

        let formatted = format_diagnostic(&diag);
        assert!(formatted.contains("[W001] Warning:"));
    }

    #[test]
    fn test_format_diagnostic_info() {
        let loc = SourceLocation::new("test.sruja".to_string(), 1, 1);
        let diag = Diagnostic::new("I001", Severity::Info, "Info message", loc.clone());

        let formatted = format_diagnostic(&diag);
        assert!(formatted.contains("[I001] Info:"));
    }

    #[test]
    fn test_error_codes_values() {
        // Test that error codes have expected values
        assert_eq!(codes::CODE_SYNTAX_ERROR, "E101");
        assert_eq!(codes::CODE_UNEXPECTED_TOKEN, "E102");
        assert_eq!(codes::CODE_MISSING_BRACE, "E103");
        assert_eq!(codes::CODE_INVALID_STRING, "E104");
        assert_eq!(codes::CODE_DUPLICATE_ID, "E201");
        assert_eq!(codes::CODE_UNDEFINED_REF, "E202");
        assert_eq!(codes::CODE_INVALID_RELATION, "E203");
        assert_eq!(codes::CODE_CYCLE_DETECTED, "E204");
        assert_eq!(codes::CODE_ORPHAN_ELEMENT, "E205");
        assert_eq!(codes::CODE_LAYER_VIOLATION, "E206");
        assert_eq!(codes::CODE_INVALID_PROPERTY, "E301");
        assert_eq!(codes::CODE_MISSING_FIELD, "E302");
        assert_eq!(codes::CODE_VALIDATION_RULE_ERROR, "E303");
        assert_eq!(codes::CODE_VALIDATION_TIMEOUT, "E304");
        assert_eq!(codes::CODE_VALIDATION_PANIC, "E305");
        assert_eq!(codes::CODE_DUPLICATE_IDENTIFIER, "E201");
        assert_eq!(codes::CODE_REFERENCE_NOT_FOUND, "E202");
        assert_eq!(codes::CODE_BEST_PRACTICE, "W001");
        assert_eq!(codes::CODE_POLICY_VIOLATION, "E401");
    }

    #[test]
    fn test_diagnostic_display_format() {
        let loc = SourceLocation::new("file.sruja".to_string(), 10, 5);
        let diag = Diagnostic::new("E001", Severity::Error, "Test error", loc.clone());
        let display = diag.to_string();

        // Verify format: [E001] Error: Test error at file.sruja:10:5
        assert!(display.contains("[E001]"));
        assert!(display.contains("Error:"));
        assert!(display.contains("Test error"));
        assert!(display.contains("file.sruja:10:5"));
    }

    #[test]
    fn test_basic_error_reporter_default() {
        let reporter = BasicErrorReporter::default();
        assert!(reporter.is_empty());
        assert_eq!(reporter.len(), 0);
    }

    #[test]
    fn test_diagnostic_with_long_message() {
        let long_message = "This is a very long error message that should still be formatted correctly without any issues in the output format and should be displayed properly to the user".to_string();
        let loc = SourceLocation::new("test.sruja".to_string(), 1, 1);
        let diag = Diagnostic::new("E001", Severity::Error, &long_message, loc.clone());

        let formatted = format_diagnostic(&diag);
        assert!(formatted.contains("[E001] Error:"));
        assert!(formatted.contains(&long_message));
    }

    #[test]
    fn test_diagnostic_with_unicode() {
        let loc = SourceLocation::new("tëst.sruja".to_string(), 1, 1);
        let diag = Diagnostic::new(
            "E001",
            Severity::Error,
            "Error with émojis 🎉 and 特殊字符",
            loc.clone(),
        );

        let formatted = format_diagnostic(&diag);
        assert!(formatted.contains("émojis 🎉"));
        assert!(formatted.contains("特殊字符"));
        assert!(formatted.contains("tëst.sruja"));
    }

    #[test]
    fn test_source_location_with_empty_file() {
        let loc = SourceLocation::new("".to_string(), 1, 1);
        assert!(loc.file.is_empty());
        let display = loc.to_string();
        assert_eq!(display, ":1:1");
    }

    #[test]
    fn test_format_diagnostic_empty_context_and_suggestions() {
        let loc = SourceLocation::new("test.sruja".to_string(), 1, 1);
        let diag = Diagnostic::new(
            codes::CODE_SYNTAX_ERROR,
            Severity::Error,
            "Error",
            loc.clone(),
        )
        .with_context(vec![])
        .with_suggestions(vec![]);

        let formatted = format_diagnostic(&diag);
        // Should have header and location, but no context or suggestions
        assert!(formatted.contains("[E101] Error:"));
        assert!(formatted.contains("--> test.sruja:1:1"));
        assert!(!formatted.contains("| "));
        assert!(!formatted.contains("= Help:"));
    }

    #[test]
    fn test_diagnostic_multiline_message() {
        let message = "Error on multiple\nlines\nshould be handled";
        let loc = SourceLocation::new("test.sruja".to_string(), 1, 1);
        let diag = Diagnostic::new("E001", Severity::Error, message, loc.clone());

        let formatted = format_diagnostic(&diag);
        assert!(formatted.contains("Error on multiple"));
    }

    #[test]
    fn test_basic_error_reporter_capacity() {
        let mut reporter = BasicErrorReporter::new();
        // Pre-allocates capacity for 8 diagnostics, but can hold more
        for i in 0..20 {
            let diag = Diagnostic::new(
                "E001",
                Severity::Error,
                &format!("Error {}", i),
                SourceLocation::new("test.sruja".to_string(), i as u32, i as u32),
            );
            reporter.report_owned(diag);
        }
        assert_eq!(reporter.len(), 20);
    }
}
