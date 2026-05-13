//! Tests for the diagnostics crate.

use crate::codes;
use crate::format::format_diagnostic;
use crate::reporter::{BasicErrorReporter, ErrorReporter};
use crate::types::{Diagnostic, Severity, SourceLocation};

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
fn test_basic_error_reporter_empty() {
    let reporter = BasicErrorReporter::new();
    assert!(reporter.is_empty());
    assert_eq!(reporter.len(), 0);
    assert!(!reporter.has_errors());
    assert!(reporter.diagnostics().is_empty());
}

#[test]
fn test_basic_error_reporter_with_diagnostics() {
    let mut reporter = BasicErrorReporter::new();

    let info_loc = SourceLocation::new("test.sruja".to_string(), 1, 1);
    reporter.report_owned(Diagnostic::new(
        codes::CODE_ORPHAN_ELEMENT,
        Severity::Info,
        "info message",
        info_loc,
    ));
    assert!(!reporter.has_errors());
    assert_eq!(reporter.diagnostics().len(), 1);
    assert_eq!(reporter.len(), 1);
    assert!(!reporter.is_empty());

    let error_loc = SourceLocation::new("test.sruja".to_string(), 2, 1);
    reporter.report_owned(Diagnostic::new(
        codes::CODE_SYNTAX_ERROR,
        Severity::Error,
        "error message",
        error_loc,
    ));
    assert!(reporter.has_errors());
    assert_eq!(reporter.diagnostics().len(), 2);
    assert_eq!(reporter.len(), 2);
}

#[test]
fn test_basic_error_reporter_clear() {
    let mut reporter = BasicErrorReporter::new();
    let loc = SourceLocation::new("test.sruja".to_string(), 1, 1);
    reporter.report_owned(Diagnostic::new(
        codes::CODE_SYNTAX_ERROR,
        Severity::Error,
        "error message",
        loc,
    ));
    reporter.clear();
    assert!(reporter.is_empty());
    assert!(!reporter.has_errors());
    assert_eq!(reporter.len(), 0);
}

#[test]
fn test_format_diagnostic_full() {
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

#[test]
fn test_format_diagnostic_minimal() {
    let diag = Diagnostic::new(
        codes::CODE_SYNTAX_ERROR,
        Severity::Error,
        "simple error",
        SourceLocation::new("test.sruja".to_string(), 1, 1),
    );
    let formatted = format_diagnostic(&diag);
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
    assert!(reporter.has_errors());
}

#[test]
fn test_basic_error_reporter_only_warnings() {
    let mut reporter = BasicErrorReporter::new();
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
    assert!(!reporter.has_errors());
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
    assert_eq!(codes::CODE_NESTING_VIOLATION, "E207");
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
    assert_eq!(loc.to_string(), ":1:1");
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
    for i in 0..20 {
        let diag = Diagnostic::new(
            "E001",
            Severity::Error,
            format!("Error {}", i),
            SourceLocation::new("test.sruja".to_string(), i as u32, i as u32),
        );
        reporter.report_owned(diag);
    }
    assert_eq!(reporter.len(), 20);
}
