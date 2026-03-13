//! Sruja Validation Engine
//!
//! This crate provides validation rules and a validator for Sruja architectures.
//! It checks for correctness, best practices, and potential issues.

pub mod rules;
pub mod utils;
pub mod validator;

// Re-export key public types
pub use validator::{Rule, RuleProfile, Validator};

// Re-export common utilities for convenience
pub use utils::{
    element_exists, extract_tags, find_element, has_tag, resolve_layer, ElementFinder,
};
