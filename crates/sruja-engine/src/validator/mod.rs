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
//!
//! # Example
//!
//! ```rust
//! use sruja_engine::Validator;
//! use sruja_language::Parser;
//!
//! let source = r#"
//! user = person "User"
//! web = system "Web App"
//! user -> web "uses"
//! "#;
//! let parser = Parser::new("example.sruja".to_string());
//! let program = parser.parse(source).unwrap();
//! let validator = Validator::with_default_rules();
//! let diagnostics = validator.validate_sync(&program);
//! ```

mod config;
mod builder;
mod core;
mod rule;

#[cfg(test)]
mod tests;

pub use core::Validator;
pub use rule::Rule;
