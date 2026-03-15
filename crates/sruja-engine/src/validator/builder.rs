#! // Builder for creating configured [`Validator`] instances
//!
//! The builder provides a fluent API for configuring all aspects of the
//! validator, including rule registration, execution mode, and performance tuning.
//!
//! # Examples
//!
//! ```
//! use std::time::Duration;
//! use sruja_engine::validator::ValidatorBuilder;
//!
//! let validator = ValidatorBuilder::new()
//!     .with_default_rules()
//!     .with_parallel(true)
//!     .with_max_parallelism(4)
//!     .with_rule_timeout(Duration::from_secs(30))
//!     .build();
//! ```
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
    ///
    /// Returns a builder with a validator that has no rules registered.
    pub fn new() -> Self {
        Self {
            validator: Validator::new(),
        }
    }

    /// Register all default validation rules
    ///
    /// This includes all currently implemented validation rules:
    /// UniqueId, ValidRef, Orphan, Cycle, LayerViolation, Simplicity,
    /// ScenarioValidation, DatabaseIsolation, PublicInterfaceDocumentation,
    /// SloValidation, PropertiesValidation, and GovernanceValidation.
    pub fn with_default_rules(mut self) -> Self {
        self.validator = self.validator.with_registered_default_rules();
        self
    }

    /// Register a custom validation rule
    ///
    /// # Arguments
    ///
    /// * `rule` - The rule to register (wrapped in an Arc)
    ///
    /// Returns the builder for method chaining.
    pub fn with_rule(mut self, rule: Arc<dyn Rule>) -> Self {
        self.validator.register_rule(rule);
        self
    }

    /// Exclude a specific rule from validation
    ///
    /// # Arguments
    ///
    /// * `rule_name` - The name of the rule to exclude (anything that can be converted to String)
    ///
    /// Returns the builder for method chaining.
    pub fn exclude_rule(mut self, rule_name: impl Into<String>) -> Self {
        let rule_name = rule_name.into();
        self.validator.excluded_rules.insert(rule_name.clone());
        self.validator.remove_rule_by_name(&rule_name);
        self
    }

    /// Enable or disable fail-fast mode
    ///
    /// When fail-fast is enabled, validation stops after the first rule that produces diagnostics.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable fail-fast mode
    ///
    /// Returns the builder for method chaining.
    pub fn with_fail_fast(mut self, enabled: bool) -> Self {
        self.validator.config.fail_fast = enabled;
        self
    }

    /// Enable or disable parallel rule execution
    ///
    /// When parallel execution is enabled, rules are executed concurrently using a thread pool.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable parallel execution
    ///
    /// Returns the builder for method chaining.
    pub fn with_parallel(mut self, enabled: bool) -> Self {
        self.validator.config.parallel = enabled;
        self
    }

    /// Set the maximum number of parallel validation tasks
    ///
    /// # Arguments
    ///
    /// * `max_parallelism` - The maximum number of parallel tasks (will be clamped to at least 1)
    ///
    /// Returns the builder for method chaining.
    pub fn with_max_parallelism(mut self, max_parallelism: usize) -> Self {
        self.validator.config.max_parallelism = max_parallelism.max(1);
        self
    }

    /// Set the timeout for individual rule validation
    ///
    /// # Arguments
    ///
    /// * `timeout` - The duration after which rule validation will time out
    ///
    /// Returns the builder for method chaining.
    pub fn with_rule_timeout(mut self, timeout: Duration) -> Self {
        self.validator.config.rule_timeout = timeout;
        self
    }

    /// Build the configured validator
    ///
    /// Consumes the builder and returns the configured [`Validator`] instance.
    ///
    /// # Returns
    ///
    /// The configured validator ready for use.
    pub fn build(self) -> Validator {
        self.validator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::UniqueIdRule;
    use std::time::Duration;

    #[test]
    fn test_builder_new() {
        let builder = ValidatorBuilder::new();
        assert_eq!(builder.validator.rule_count(), 0);
        assert!(!builder.validator.config.fail_fast);
        assert!(!builder.validator.config.parallel);
        assert_eq!(builder.validator.config.max_parallelism, 4); // Default value from ValidatorConfig::default()
    }

    #[test]
    fn test_builder_with_default_rules() {
        let builder = ValidatorBuilder::new().with_default_rules();
        assert!(builder.validator.rule_count() > 0);
        assert!(builder.validator.has_rule("Unique IDs"));
    }

    #[test]
    fn test_builder_with_rule() {
        let rule = Arc::new(UniqueIdRule);
        let builder = ValidatorBuilder::new().with_rule(rule);
        assert_eq!(builder.validator.rule_count(), 1);
        assert!(builder.validator.has_rule("Unique IDs"));
    }

    #[test]
    fn test_builder_exclude_rule() {
        let builder = ValidatorBuilder::new()
            .with_default_rules()
            .exclude_rule("Unique IDs");
        assert!(!builder.validator.has_rule("Unique IDs"));
        // Should still have other rules
        assert!(builder.validator.rule_count() > 0);
    }

    #[test]
    fn test_builder_with_fail_fast() {
        let builder = ValidatorBuilder::new().with_fail_fast(true);
        assert!(builder.validator.config.fail_fast);
    }

    #[test]
    fn test_builder_with_parallel() {
        let builder = ValidatorBuilder::new().with_parallel(true);
        assert!(builder.validator.config.parallel);
    }

    #[test]
    fn test_builder_with_max_parallelism() {
        let builder = ValidatorBuilder::new().with_max_parallelism(4);
        assert_eq!(builder.validator.config.max_parallelism, 4);

        // Test clamping to minimum of 1
        let builder = ValidatorBuilder::new().with_max_parallelism(0);
        assert_eq!(builder.validator.config.max_parallelism, 1);
    }

    #[test]
    fn test_builder_with_rule_timeout() {
        let timeout = Duration::from_secs(30);
        let builder = ValidatorBuilder::new().with_rule_timeout(timeout);
        assert_eq!(builder.validator.config.rule_timeout, timeout);
    }

    #[test]
    fn test_builder_build() {
        let validator = ValidatorBuilder::new().with_default_rules().build();
        assert!(validator.rule_count() > 0);
    }
}
