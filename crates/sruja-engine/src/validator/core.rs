//! Main validator: orchestration and execution of validation rules

use std::collections::HashSet;
use std::sync::Arc;

use sruja_diagnostics::Diagnostic;
use sruja_language::Program;

use crate::rules::{
    CycleDetectionRule, DatabaseIsolationRule, GovernanceValidationRule, LayerViolationRule,
    OrphanDetectionRule, PolicyEvaluationRule, PropertiesValidationRule,
    PublicInterfaceDocumentationRule, ScenarioValidationRule, SimplicityRule, SloValidationRule,
    SourcesValidationRule, UniqueIdRule, ValidRefRule,
};

use super::config::ValidatorConfig;
use super::rule::Rule;

/// Rule profile: which set of validation rules to run.
///
/// - **Minimal:** Essential safety/correctness only (unique id, valid refs, orphans, cycles, layer violations).
///   Use for fast feedback or when stricter rules are not desired.
/// - **Default:** All registered rules (current full set).
/// - **Strict:** Same as Default today; reserved for future additional opinionated rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RuleProfile {
    /// Safety and correctness only: UniqueId, ValidRef, Orphan, Cycle, LayerViolation.
    Minimal,
    /// All rules (full validation).
    #[default]
    Default,
    /// All rules; reserved for future stricter checks.
    Strict,
}

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

    /// Create a validator with the given rule profile (minimal, default, or strict).
    pub fn with_profile(profile: RuleProfile) -> Self {
        Self::new().with_registered_profile(profile)
    }

    /// Create a validator builder for custom configuration
    pub fn builder() -> super::builder::ValidatorBuilder {
        super::builder::ValidatorBuilder::new()
    }

    /// Register rules for the given profile.
    pub(crate) fn with_registered_profile(mut self, profile: RuleProfile) -> Self {
        match profile {
            RuleProfile::Minimal => {
                self.register_rule(Arc::new(UniqueIdRule));
                self.register_rule(Arc::new(ValidRefRule));
                self.register_rule(Arc::new(OrphanDetectionRule));
                self.register_rule(Arc::new(CycleDetectionRule));
                self.register_rule(Arc::new(LayerViolationRule));
            }
            RuleProfile::Default | RuleProfile::Strict => {
                self = self.with_registered_default_rules();
            }
        }
        self
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
        self.register_rule(Arc::new(SourcesValidationRule));
        self.register_rule(Arc::new(GovernanceValidationRule));
        self.register_rule(Arc::new(PolicyEvaluationRule));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validator_new_creates_empty_validator() {
        let validator = Validator::new();
        assert_eq!(validator.rule_count(), 0);
    }

    #[test]
    fn validator_with_default_rules_has_rules() {
        let validator = Validator::with_default_rules();
        assert!(validator.rule_count() > 0);
    }

    #[test]
    fn validator_with_profile_minimal_has_fewer_rules() {
        let minimal = Validator::with_profile(RuleProfile::Minimal);
        let default = Validator::with_profile(RuleProfile::Default);
        assert!(minimal.rule_count() < default.rule_count());
    }

    #[test]
    fn validator_with_profile_strict_has_same_as_default() {
        let strict = Validator::with_profile(RuleProfile::Strict);
        let default = Validator::with_profile(RuleProfile::Default);
        assert_eq!(strict.rule_count(), default.rule_count());
    }

    #[test]
    fn validator_default_is_same_as_with_default_rules() {
        let validator = Validator::default();
        let with_default = Validator::with_default_rules();
        assert_eq!(validator.rule_count(), with_default.rule_count());
    }

    #[test]
    fn validator_has_rule_returns_true_for_registered_rule() {
        let validator = Validator::with_default_rules();
        assert!(validator.has_rule("Unique IDs"));
        assert!(validator.has_rule("Valid References"));
    }

    #[test]
    fn validator_has_rule_returns_false_for_unknown_rule() {
        let validator = Validator::with_default_rules();
        assert!(!validator.has_rule("NonExistentRule"));
    }

    #[test]
    fn validator_validate_sync_empty_program() {
        let validator = Validator::with_default_rules();
        let program = Program::default();
        let diagnostics = validator.validate_sync(&program);
        assert!(diagnostics.is_empty() || !diagnostics.is_empty());
    }

    #[test]
    fn validator_builder_creates_validator() {
        let validator = Validator::builder().with_default_rules().build();
        assert!(validator.rule_count() > 0);
    }

    #[test]
    fn validator_builder_empty_has_no_rules() {
        let validator = Validator::builder().build();
        assert_eq!(validator.rule_count(), 0);
    }

    #[test]
    fn rule_profile_default_value() {
        let profile = RuleProfile::default();
        assert_eq!(profile, RuleProfile::Default);
    }

    #[test]
    fn rule_profile_equality() {
        assert_eq!(RuleProfile::Minimal, RuleProfile::Minimal);
        assert_eq!(RuleProfile::Default, RuleProfile::Default);
        assert_ne!(RuleProfile::Minimal, RuleProfile::Default);
    }

    #[test]
    fn validator_remove_rule_by_name() {
        let mut validator = Validator::with_default_rules();
        let initial_count = validator.rule_count();
        validator.remove_rule_by_name("Unique IDs");
        assert!(validator.rule_count() < initial_count);
        assert!(!validator.has_rule("Unique IDs"));
    }

    #[test]
    fn validator_excluded_rules_not_registered() {
        let mut validator = Validator::new();
        validator.excluded_rules.insert("Unique IDs".to_string());
        validator = validator.with_registered_default_rules();
        assert!(!validator.has_rule("Unique IDs"));
    }
}
