//! Validation module for CLI
//!
//! Provides reusable validation functionality including:
//! - Running validation rules against programs
//! - Calculating architecture health scores
//! - Formatting and reporting diagnostics
//! - Supporting batch validation operations

#![allow(dead_code)]

use sruja_diagnostics::{format_diagnostic, Diagnostic, Severity};
use sruja_engine::Validator;
use sruja_language::Program;

/// Enrich diagnostics with a source snippet and caret indicator.
///
/// This is a CLI-only enhancement (no file IO): callers pass the file content.
/// We merge the generated snippet with any existing `Diagnostic.context` lines
/// (those are preserved as `note: ...` lines).
pub fn enrich_diagnostics_with_source(content: &str, diagnostics: &mut [Diagnostic]) {
    // `lines()` drops trailing empty final line, which is fine for diagnostics display.
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return;
    }

    for diag in diagnostics {
        let line_1_indexed = diag.location.line;
        if line_1_indexed == 0 {
            continue;
        }

        let idx = (line_1_indexed.saturating_sub(1)) as usize;
        if idx >= lines.len() {
            continue;
        }

        // Width for pretty alignment of line numbers (show +/-1 lines).
        let max_line = (line_1_indexed + 1).to_string();
        let width = max_line.len().max(2);

        let mut ctx: Vec<String> = Vec::with_capacity(6);

        // Previous line
        if idx > 0 {
            ctx.push(format!(
                "{:>width$} | {}",
                line_1_indexed - 1,
                lines[idx - 1],
                width = width
            ));
        }

        // Current line
        ctx.push(format!(
            "{:>width$} | {}",
            line_1_indexed,
            lines[idx],
            width = width
        ));

        // Caret line (best-effort: column is 1-indexed and counts characters approximately)
        let col_1_indexed = diag.location.column.max(1) as usize;
        let caret_pos = col_1_indexed.saturating_sub(1);
        let pad_width = " ".repeat(width);
        let caret_spaces = " ".repeat(caret_pos.min(500)); // cap to avoid absurd spacing
        ctx.push(format!("{} | {}^", pad_width, caret_spaces));

        // Next line
        if idx + 1 < lines.len() {
            ctx.push(format!(
                "{:>width$} | {}",
                line_1_indexed + 1,
                lines[idx + 1],
                width = width
            ));
        }

        // Preserve any existing context as notes (dedup conservatively).
        let old_context = std::mem::take(&mut diag.context);
        for line in old_context {
            if line.trim().is_empty() {
                continue;
            }
            let note = format!("note: {}", line);
            if !ctx.iter().any(|existing| existing == &note) {
                ctx.push(note);
            }
        }

        diag.context = ctx;
    }
}

/// Configuration for validation operations
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Whether to register default validation rules
    pub register_default_rules: bool,
    /// Whether to fail on validation errors
    pub fail_on_errors: bool,
    /// Whether to include info-level diagnostics
    pub include_info: bool,
    /// Whether to calculate a health score
    pub calculate_score: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            register_default_rules: true,
            fail_on_errors: true,
            include_info: false,
            calculate_score: false,
        }
    }
}

/// Result of a validation operation
#[allow(dead_code)]
#[derive(Debug)]
pub struct ValidationResult {
    /// Whether validation passed (no errors)
    pub passed: bool,
    /// All diagnostics found
    pub diagnostics: Vec<Diagnostic>,
    /// Health score (0-100), if calculated
    pub score: Option<u32>,
    /// Grade letter (A-F), if score calculated
    pub grade: Option<char>,
    /// Number of errors found
    pub error_count: usize,
    /// Number of warnings found
    pub warning_count: usize,
    /// Number of info messages found
    pub info_count: usize,
}

#[allow(dead_code)]
impl ValidationResult {
    /// Create a new validation result from diagnostics
    pub fn new(diagnostics: Vec<Diagnostic>, config: &ValidationConfig) -> Self {
        // Calculate score first while we still own the diagnostics
        let (score, grade) = if config.calculate_score {
            let (score, grade) = Self::calculate_score(&diagnostics);
            (Some(score), Some(grade))
        } else {
            (None, None)
        };

        // Count diagnostics by severity
        let error_count = diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        let warning_count = diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count();
        let info_count = if config.include_info {
            diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Info)
                .count()
        } else {
            0
        };

        let passed = error_count == 0;

        Self {
            passed,
            diagnostics,
            score,
            grade,
            error_count,
            warning_count,
            info_count,
        }
    }

    ///
    /// Scoring rules:
    /// - Start with 100 points
    /// - Each error: -5 points
    /// - Each warning: -2 points
    /// - Grade: A (90+), B (80+), C (70+), D (60+), F (<60)
    fn calculate_score(diagnostics: &[Diagnostic]) -> (u32, char) {
        let mut score: u32 = 100;

        for diag in diagnostics {
            let points = match diag.severity {
                Severity::Error => 5,
                Severity::Warning => 2,
                Severity::Info => 0,
                _ => 0,
            };
            score = score.saturating_sub(points);
        }

        let grade = if score >= 90 {
            'A'
        } else if score >= 80 {
            'B'
        } else if score >= 70 {
            'C'
        } else if score >= 60 {
            'D'
        } else {
            'F'
        };

        (score, grade)
    }

    /// Get only error diagnostics
    pub fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect()
    }

    /// Get only warning diagnostics
    pub fn warnings(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .collect()
    }

    /// Get only info diagnostics
    pub fn infos(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Info)
            .collect()
    }
}

/// Run validation on a program
///
/// # Arguments
/// * `program` - The program to validate
/// * `config` - Validation configuration
///
/// # Returns
/// A `ValidationResult` containing diagnostics and metadata
pub fn validate_program(program: &Program, config: &ValidationConfig) -> ValidationResult {
    let validator = if config.register_default_rules {
        Validator::with_default_rules()
    } else {
        Validator::new()
    };

    let diagnostics = validator.validate_sync(program);
    ValidationResult::new(diagnostics, config)
}

/// Validate a single file with detailed result
///
/// # Arguments
/// * `program` - The parsed program to validate
/// * `config` - Validation configuration
///
/// # Returns
/// A `ValidationResult` with full diagnostics and metadata
#[allow(dead_code)]
pub fn validate_file(program: &Program, config: &ValidationConfig) -> ValidationResult {
    validate_program(program, config)
}

/// Validate multiple programs and produce a summary
///
/// # Arguments
/// * `programs` - Slice of (file_path, program) tuples to validate
/// * `config` - Validation configuration
///
/// # Returns
/// A `BatchValidationResult` containing individual and summary results
pub fn validate_batch(
    programs: &[(&str, &Program)],
    config: &ValidationConfig,
) -> BatchValidationResult {
    let mut results = Vec::new();
    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut passed_count = 0;

    for (file_path, program) in programs {
        let result = validate_program(program, config);
        total_errors += result.error_count;
        total_warnings += result.warning_count;
        if result.passed {
            passed_count += 1;
        }
        results.push((file_path.to_string(), result));
    }

    BatchValidationResult {
        total_files: programs.len(),
        passed_files: passed_count,
        failed_files: programs.len() - passed_count,
        total_errors,
        total_warnings,
        results,
    }
}

/// Result of batch validation across multiple files
#[allow(dead_code)]
#[derive(Debug)]
pub struct BatchValidationResult {
    /// Total number of files validated
    pub total_files: usize,
    /// Number of files that passed validation
    pub passed_files: usize,
    /// Number of files that failed validation
    pub failed_files: usize,
    /// Total errors across all files
    pub total_errors: usize,
    /// Total warnings across all files
    pub total_warnings: usize,
    /// Individual validation results per file
    pub results: Vec<(String, ValidationResult)>,
}

#[allow(dead_code)]
impl BatchValidationResult {
    /// Get all failed file paths
    pub fn failed_file_paths(&self) -> Vec<&str> {
        self.results
            .iter()
            .filter(|(_, result)| !result.passed)
            .map(|(path, _)| path.as_str())
            .collect()
    }

    /// Get all passed file paths
    pub fn passed_file_paths(&self) -> Vec<&str> {
        self.results
            .iter()
            .filter(|(_, result)| result.passed)
            .map(|(path, _)| path.as_str())
            .collect()
    }
}

/// Diagnostic formatter for CLI output
pub struct DiagnosticFormatter;

#[allow(dead_code)]
impl DiagnosticFormatter {
    /// Format all diagnostics from a validation result
    ///
    /// # Arguments
    /// * `result` - The validation result containing diagnostics
    /// * `include_info` - Whether to include info-level diagnostics
    ///
    /// # Returns
    /// A vector of formatted diagnostic strings
    pub fn format_result(result: &ValidationResult, include_info: bool) -> Vec<String> {
        let mut formatted = Vec::new();

        for diag in &result.diagnostics {
            if !include_info && diag.severity == Severity::Info {
                continue;
            }
            formatted.push(format_diagnostic(diag));
        }

        formatted
    }

    /// Format a validation summary
    pub fn format_summary(result: &ValidationResult, file_name: &str) -> String {
        let mut parts = vec![file_name.to_string()];

        if let Some(score) = result.score {
            parts.push(format!("{}/100", score));
        }

        if let Some(grade) = result.grade {
            parts.push(format!("({})", grade));
        }

        parts.join(": ")
    }

    /// Format a batch validation summary
    pub fn format_batch_summary(result: &BatchValidationResult) -> String {
        format!(
            "Validation Summary:\n  Total files: {}\n  Valid: {}\n  Invalid: {}\n  Total errors: {}\n  Total warnings: {}",
            result.total_files,
            result.passed_files,
            result.failed_files,
            result.total_errors,
            result.total_warnings
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_diagnostics::{codes, SourceLocation};

    fn create_test_diagnostic(
        code: &'static str,
        severity: Severity,
        message: impl Into<String>,
    ) -> Diagnostic {
        Diagnostic::new(
            code,
            severity,
            message,
            SourceLocation::new("test.sruja".to_string(), 1, 1),
        )
    }

    #[test]
    fn test_validation_result_success() {
        let diagnostics = vec![];
        let config = ValidationConfig::default();
        let result = ValidationResult::new(diagnostics, &config);

        assert!(result.passed);
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 0);
        assert!(result.score.is_none());
    }

    #[test]
    fn test_validation_result_with_errors() {
        let diagnostics = vec![
            create_test_diagnostic(codes::CODE_DUPLICATE_ID, Severity::Error, "Duplicate ID"),
            create_test_diagnostic(codes::CODE_UNDEFINED_REF, Severity::Error, "Undefined ref"),
        ];
        let config = ValidationConfig::default();
        let result = ValidationResult::new(diagnostics, &config);

        assert!(!result.passed);
        assert_eq!(result.error_count, 2);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_validation_result_with_warnings_only() {
        let diagnostics = vec![
            create_test_diagnostic(
                codes::CODE_BEST_PRACTICE,
                Severity::Warning,
                "Best practice",
            ),
            create_test_diagnostic(
                codes::CODE_BEST_PRACTICE,
                Severity::Warning,
                "Another practice",
            ),
        ];
        let config = ValidationConfig::default();
        let result = ValidationResult::new(diagnostics, &config);

        assert!(result.passed); // Warnings don't cause failure
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 2);
    }

    #[test]
    fn test_score_calculation() {
        // 2 errors (-10) + 3 warnings (-6) = 84 -> B grade
        let diagnostics = vec![
            create_test_diagnostic(codes::CODE_DUPLICATE_ID, Severity::Error, "Error 1"),
            create_test_diagnostic(codes::CODE_UNDEFINED_REF, Severity::Error, "Error 2"),
            create_test_diagnostic(codes::CODE_BEST_PRACTICE, Severity::Warning, "Warning 1"),
            create_test_diagnostic(codes::CODE_BEST_PRACTICE, Severity::Warning, "Warning 2"),
            create_test_diagnostic(codes::CODE_BEST_PRACTICE, Severity::Warning, "Warning 3"),
        ];

        let config = ValidationConfig {
            calculate_score: true,
            ..Default::default()
        };

        let result = ValidationResult::new(diagnostics, &config);
        assert_eq!(result.score, Some(84));
        assert_eq!(result.grade, Some('B'));
    }

    #[test]
    fn test_score_grades() {
        // Test each grade boundary
        let test_cases = vec![
            (0, 100, 'A'), // No deductions
            (3, 85, 'B'),  // 3 errors = 100 - 15 = 85
            (4, 80, 'B'),  // 4 errors = 100 - 20 = 80
            (5, 75, 'C'),  // 5 errors = 100 - 25 = 75
            (10, 50, 'F'), // 10 errors = 100 - 50 = 50
        ];

        for (error_count, expected_score, expected_grade) in test_cases {
            let mut diagnostics = Vec::new();
            for i in 0..error_count {
                diagnostics.push(create_test_diagnostic(
                    codes::CODE_DUPLICATE_ID,
                    Severity::Error,
                    format!("Error {}", i),
                ));
            }

            let config = ValidationConfig {
                calculate_score: true,
                ..Default::default()
            };

            let result = ValidationResult::new(diagnostics, &config);
            assert_eq!(
                result.score,
                Some(expected_score),
                "Score mismatch for {} errors",
                error_count
            );
            assert_eq!(
                result.grade,
                Some(expected_grade),
                "Grade mismatch for {} errors",
                error_count
            );
        }
    }

    #[test]
    fn test_score_with_warnings() {
        // Test that warnings are properly deducted
        // 1 error (5 pts) + 3 warnings (6 pts) = 11 pts = 89 (A-)
        let mut diagnostics = Vec::new();

        diagnostics.push(create_test_diagnostic(
            codes::CODE_DUPLICATE_ID,
            Severity::Error,
            "Error 1",
        ));

        for i in 0..3 {
            diagnostics.push(create_test_diagnostic(
                codes::CODE_BEST_PRACTICE,
                Severity::Warning,
                format!("Warning {}", i),
            ));
        }

        let config = ValidationConfig {
            calculate_score: true,
            ..Default::default()
        };

        let result = ValidationResult::new(diagnostics, &config);
        assert_eq!(
            result.score,
            Some(89),
            "Score with 1 error and 3 warnings should be 89"
        );
        assert_eq!(result.grade, Some('B'), "Grade should be B for score 89");
    }

    #[test]
    fn test_score_only_warnings() {
        // Test that only warnings still produce a valid score
        // 10 warnings = 20 pts = 80 (B)
        let mut diagnostics = Vec::new();

        for i in 0..10 {
            diagnostics.push(create_test_diagnostic(
                codes::CODE_BEST_PRACTICE,
                Severity::Warning,
                format!("Warning {}", i),
            ));
        }

        let config = ValidationConfig {
            calculate_score: true,
            ..Default::default()
        };

        let result = ValidationResult::new(diagnostics, &config);
        assert_eq!(
            result.score,
            Some(80),
            "Score with 10 warnings should be 80"
        );
        assert_eq!(result.grade, Some('B'), "Grade should be B for score 80");
    }

    #[test]
    fn test_batch_validation() {
        // Create mock programs (using empty programs for simplicity)
        let program1 = Program::default();
        let program2 = Program::default();
        let _program3 = Program::default();

        let programs = vec![("file1.sruja", &program1), ("file2.sruja", &program2)];

        let config = ValidationConfig::default();
        let result = validate_batch(&programs, &config);

        assert_eq!(result.total_files, 2);
        // Empty programs should pass validation with no errors
        assert_eq!(result.passed_files, 2);
        assert_eq!(result.failed_files, 0);
        assert_eq!(result.total_errors, 0);
        assert_eq!(result.total_warnings, 0);
    }

    #[test]
    fn test_diagnostic_formatter() {
        let diagnostics = vec![create_test_diagnostic(
            codes::CODE_DUPLICATE_ID,
            Severity::Error,
            "Test error",
        )];

        let config = ValidationConfig::default();
        let result = ValidationResult::new(diagnostics, &config);

        let formatted = DiagnosticFormatter::format_result(&result, false);
        assert_eq!(formatted.len(), 1);
        // format_diagnostic from sruja-diagnostics emits "[code] Error: message", not "✗"
        assert!(formatted[0].contains("Error"));
        assert!(formatted[0].contains("Test error"));
    }

    #[test]
    fn test_validation_config_default() {
        let config = ValidationConfig::default();
        assert!(config.register_default_rules);
        assert!(config.fail_on_errors);
        assert!(!config.include_info);
        assert!(!config.calculate_score);
    }
}
