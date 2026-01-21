//! Validator for Sruja architectures
//!
//! The validator runs a collection of validation rules concurrently to check
//! architectures for correctness, best practices, and potential issues.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::timeout;

use sruja_diagnostics::{Diagnostic, SourceLocation};
use sruja_language::Program;

/// Trait for validation rules
pub trait Rule: Send + Sync {
    /// Get the human-readable name of the validation rule
    fn name(&self) -> &str;
    
    /// Validate the program and return diagnostics
    /// Returns an empty vector if no issues are found
    fn validate(&self, program: &Program) -> Vec<Diagnostic>;
}

/// Validator manages and executes validation rules
pub struct Validator {
    rules: Vec<Arc<dyn Rule>>,
    options: ValidatorOptions,
}

/// Configuration options for the validator
#[derive(Debug, Clone)]
pub struct ValidatorOptions {
    /// Maximum time allowed for all validation rules to complete
    pub timeout: Duration,
    /// Maximum number of concurrent validation rules
    pub concurrency: usize,
}

impl Default for ValidatorOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            concurrency: 10,
        }
    }
}

impl Validator {
    /// Create a new validator with default options
    pub fn new() -> Self {
        Self::with_options(ValidatorOptions::default())
    }

    /// Create a new validator with custom options
    pub fn with_options(options: ValidatorOptions) -> Self {
        Self {
            rules: Vec::new(),
            options,
        }
    }

    /// Add a validation rule
    pub fn register_rule<R: Rule + 'static>(&mut self, rule: R) {
        self.rules.push(Arc::new(rule));
    }

    /// Register default validation rules
    pub fn register_default_rules(&mut self) {
        self.register_rule(crate::rules::UniqueIdRule);
        self.register_rule(crate::rules::ValidRefRule);
        self.register_rule(crate::rules::CycleDetectionRule);
        self.register_rule(crate::rules::OrphanDetectionRule);
        // Add more default rules as we migrate them
    }

    /// Validate a program with all registered rules
    /// Returns a vector of diagnostics (errors and warnings)
    pub async fn validate(&self, program: &Program) -> Vec<Diagnostic> {
        if self.rules.is_empty() {
            return Vec::new();
        }

        let program = Arc::new(program.clone());
        let semaphore = Arc::new(Semaphore::new(self.options.concurrency));
        let timeout_duration = self.options.timeout;

        let mut handles = Vec::new();

        for rule in &self.rules {
            let rule = Arc::clone(rule);
            let rule_name = rule.name().to_string();
            let program_clone = Arc::clone(&program);
            let semaphore_clone = Arc::clone(&semaphore);

            let handle = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();
                let start = Instant::now();

                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    rule.validate(&program_clone)
                }));

                match result {
                    Ok(diagnostics) => (rule_name, diagnostics, start.elapsed()),
                    Err(_) => (
                        rule_name.clone(),
                        vec![Diagnostic::new(
                            sruja_diagnostics::codes::CODE_VALIDATION_PANIC,
                            sruja_diagnostics::Severity::Error,
                            format!("Rule '{}' panicked during execution", rule_name),
                            SourceLocation::new(String::new(), 0, 0),
                        )],
                        start.elapsed(),
                    ),
                }
            });

            handles.push(handle);
        }

        // Wait for all rules to complete or timeout
        let validation_result = timeout(timeout_duration, async {
            let mut all_diagnostics = Vec::new();
            for handle in handles {
                if let Ok((_rule_name, diagnostics, _elapsed)) = handle.await {
                    all_diagnostics.extend(diagnostics);
                }
            }
            all_diagnostics
        })
        .await;

        match validation_result {
            Ok(diagnostics) => diagnostics,
            Err(_) => {
                vec![Diagnostic::new(
                    sruja_diagnostics::codes::CODE_VALIDATION_TIMEOUT,
                    sruja_diagnostics::Severity::Error,
                    format!(
                        "Validation timed out after {} seconds",
                        timeout_duration.as_secs()
                    ),
                    SourceLocation::new(String::new(), 0, 0),
                )]
            }
        }
    }

    /// Validate synchronously (runs rules sequentially)
    /// Useful for simple validation without async runtime
    pub fn validate_sync(&self, program: &Program) -> Vec<Diagnostic> {
        if self.rules.is_empty() {
            return Vec::new();
        }

        let mut all_diagnostics = Vec::new();

        for rule in &self.rules {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                rule.validate(program)
            }));

            match result {
                Ok(diagnostics) => {
                    all_diagnostics.extend(diagnostics);
                }
                Err(_) => {
                    all_diagnostics.push(Diagnostic::new(
                        sruja_diagnostics::codes::CODE_VALIDATION_PANIC,
                        sruja_diagnostics::Severity::Error,
                        format!("Rule '{}' panicked during execution", rule.name()),
                        SourceLocation::new(String::new(), 0, 0),
                    ));
                }
            }
        }

        all_diagnostics
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_with_no_rules() {
        let validator = Validator::new();
        let program = Program::default();
        let diagnostics = validator.validate_sync(&program);
        assert!(diagnostics.is_empty());
    }
}
