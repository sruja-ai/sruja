//! Main validator: orchestration and execution of validation rules

use std::collections::HashSet;
use std::sync::Arc;

use sruja_diagnostics::Diagnostic;
use sruja_language::Program;

use crate::rules::{
    CycleDetectionRule, DatabaseIsolationRule, GovernanceValidationRule, LayerViolationRule,
    OrphanDetectionRule, PropertiesValidationRule, PublicInterfaceDocumentationRule,
    ScenarioValidationRule, SimplicityRule, SloValidationRule, UniqueIdRule, ValidRefRule,
};

use super::config::ValidatorConfig;
use super::rule::Rule;

/// Main validator that orchestrates multiple validation rules against a program
///
/// The validator manages a collection of validation rules and executes them
/// against Sruja DSL programs, collecting and aggregating diagnostics from
/// all rules.
///
/// # Thread Safety
///
/// The validator is thread-safe and can be shared across threads, allowing
/// for parallel validation of multiple programs.
#[derive(Clone)]
pub struct Validator {
    /// Registered validation rules
    rules: Vec<Arc<dyn Rule>>,

    /// Rule names to exclude from validation
    pub(crate) excluded_rules: HashSet<String>,

    /// Configuration for validation behavior
    pub(crate) config: ValidatorConfig,
}

impl Validator {
    /// Create a new validator with no rules registered
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            excluded_rules: HashSet::new(),
            config: ValidatorConfig::default(),
        }
    }

    /// Create a validator with all default rules registered
    pub fn with_default_rules() -> Self {
        Self::new().with_registered_default_rules()
    }

    /// Create a validator builder for custom configuration
    pub fn builder() -> super::builder::ValidatorBuilder {
        super::builder::ValidatorBuilder::new()
    }

    /// Register all default validation rules (used by builder and with_default_rules).
    pub(crate) fn with_registered_default_rules(mut self) -> Self {
        self.register_rule(Arc::new(UniqueIdRule));
        self.register_rule(Arc::new(ValidRefRule));
        self.register_rule(Arc::new(OrphanDetectionRule));
        self.register_rule(Arc::new(CycleDetectionRule));
        self.register_rule(Arc::new(LayerViolationRule));
        self.register_rule(Arc::new(SimplicityRule));
        self.register_rule(Arc::new(ScenarioValidationRule));
        self.register_rule(Arc::new(DatabaseIsolationRule));
        self.register_rule(Arc::new(PublicInterfaceDocumentationRule));
        self.register_rule(Arc::new(SloValidationRule));
        self.register_rule(Arc::new(PropertiesValidationRule));
        self.register_rule(Arc::new(GovernanceValidationRule));
        self
    }

    /// Register a validation rule
    pub fn register_rule(&mut self, rule: Arc<dyn Rule>) {
        if !self.excluded_rules.contains(rule.name()) {
            self.rules.push(rule);
        }
    }

    /// Validate a program synchronously and return all diagnostics
    pub fn validate_sync(&self, program: &Program) -> Vec<Diagnostic> {
        let mut all_diagnostics = Vec::new();

        for rule in &self.rules {
            if self.config.fail_fast && !all_diagnostics.is_empty() {
                break;
            }

            let mut diagnostics = rule.validate(program);
            all_diagnostics.append(&mut diagnostics);
        }

        all_diagnostics
    }

    /// Validate a program asynchronously and return all diagnostics
    ///
    /// Only available when the "async" feature is enabled.
    #[cfg(feature = "async")]
    pub async fn validate(&self, program: Arc<Program>) -> Vec<Diagnostic> {
        if !self.config.parallel || self.rules.len() <= 1 {
            return self.validate_sync(&program);
        }

        let mut tasks = Vec::new();
        let rule_timeout = self.config.rule_timeout;

        for rule in self.rules.clone() {
            let program_clone = Arc::clone(&program);
            let task = tokio::spawn(async move {
                tokio::time::timeout(
                    rule_timeout,
                    tokio::task::spawn_blocking(move || rule.validate(&program_clone)),
                )
                .await
            });
            tasks.push(task);
        }

        let mut all_diagnostics = Vec::new();

        for task in tasks {
            match task.await {
                Ok(Ok(Ok(mut diagnostics))) => {
                    all_diagnostics.append(&mut diagnostics);
                }
                Ok(Ok(Err(e))) => {
                    eprintln!("Rule execution task panicked: {}", e);
                }
                Ok(Err(_)) => {
                    eprintln!(
                        "Rule execution timed out after {:?}",
                        self.config.rule_timeout
                    );
                }
                Err(e) => {
                    eprintln!("Failed to join validation task: {}", e);
                }
            }

            if self.config.fail_fast && !all_diagnostics.is_empty() {
                break;
            }
        }

        all_diagnostics
    }

    /// Get the number of registered rules
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Check if a specific rule is registered
    pub fn has_rule(&self, rule_name: &str) -> bool {
        self.rules.iter().any(|r| r.name() == rule_name)
    }

    /// Remove a rule by name (used by builder when excluding rules)
    pub(crate) fn remove_rule_by_name(&mut self, rule_name: &str) {
        self.rules.retain(|r| r.name() != rule_name);
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::with_default_rules()
    }
}
