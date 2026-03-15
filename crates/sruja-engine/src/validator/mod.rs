//! Validator for Sruja DSL programs
//!
//! This module provides validation for Sruja architecture definitions via a
//! collection of rules. It supports synchronous and asynchronous execution,
//! configurable rules, and parallel execution.
//!
//! # Core concepts
//!
//! - **[`Rule`](rule::Rule)** – Trait for validation logic; implement to add custom rules.
//! - **[`Validator`](core::Validator)** – Orchestrates rules and runs validation.
//! - **[`ValidatorBuilder`](builder::ValidatorBuilder)** – Fluent API for configuring validators.
//!
//! # Example
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
//!
mod builder;
mod config;
mod core;
mod rule;

pub use builder::ValidatorBuilder;
pub use core::{RuleProfile, Validator};
pub use rule::Rule;
