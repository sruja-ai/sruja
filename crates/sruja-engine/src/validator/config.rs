//! Validator configuration

use std::time::Duration;

/// Maximum number of parallel validation tasks
pub(super) const DEFAULT_MAX_PARALLELISM: usize = 4;

/// Timeout for individual rule validation (in seconds)
pub(super) const DEFAULT_RULE_TIMEOUT_SECS: u64 = 30;

/// Configuration options for the validator
///
/// Used by [`super::Validator`] and [`super::ValidatorBuilder`]. Fields are
/// `pub(crate)` so the builder can mutate them without exposing them in the
/// public API.
#[derive(Debug, Clone)]
pub(crate) struct ValidatorConfig {
    /// Whether to stop validation on first error
    pub(crate) fail_fast: bool,

    /// Whether to execute rules in parallel
    pub(crate) parallel: bool,

    /// Maximum number of parallel tasks
    pub(crate) max_parallelism: usize,

    /// Timeout for individual rule validation
    pub(crate) rule_timeout: Duration,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            fail_fast: false,
            parallel: false,
            max_parallelism: DEFAULT_MAX_PARALLELISM,
            rule_timeout: Duration::from_secs(DEFAULT_RULE_TIMEOUT_SECS),
        }
    }
}
