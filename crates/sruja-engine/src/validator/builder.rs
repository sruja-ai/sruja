//! Builder for creating configured [`Validator`] instances

use std::sync::Arc;
use std::time::Duration;

use super::core::Validator;
use super::rule::Rule;

/// Builder for creating configured [`Validator`] instances
///
/// The builder provides a fluent API for configuring all aspects of the
/// validator, including rule registration, execution mode, and performance tuning.
pub struct ValidatorBuilder {
    validator: Validator,
}

impl ValidatorBuilder {
    /// Create a new validator builder
    pub(super) fn new() -> Self {
        Self {
            validator: Validator::new(),
        }
    }

    /// Register all default validation rules
    pub fn with_default_rules(mut self) -> Self {
        self.validator = self.validator.with_registered_default_rules();
        self
    }

    /// Register a custom validation rule
    pub fn with_rule(mut self, rule: Arc<dyn Rule>) -> Self {
        self.validator.register_rule(rule);
        self
    }

    /// Exclude a specific rule from validation
    pub fn exclude_rule(mut self, rule_name: impl Into<String>) -> Self {
        let rule_name = rule_name.into();
        self.validator.excluded_rules.insert(rule_name.clone());
        self.validator.remove_rule_by_name(&rule_name);
        self
    }

    /// Enable or disable fail-fast mode
    pub fn with_fail_fast(mut self, enabled: bool) -> Self {
        self.validator.config.fail_fast = enabled;
        self
    }

    /// Enable or disable parallel rule execution
    pub fn with_parallel(mut self, enabled: bool) -> Self {
        self.validator.config.parallel = enabled;
        self
    }

    /// Set the maximum number of parallel validation tasks
    pub fn with_max_parallelism(mut self, max_parallelism: usize) -> Self {
        self.validator.config.max_parallelism = max_parallelism.max(1);
        self
    }

    /// Set the timeout for individual rule validation
    pub fn with_rule_timeout(mut self, timeout: Duration) -> Self {
        self.validator.config.rule_timeout = timeout;
        self
    }

    /// Build the configured validator
    pub fn build(self) -> Validator {
        self.validator
    }
}
