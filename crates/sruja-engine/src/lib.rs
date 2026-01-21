//! Sruja Validation Engine
//!
//! This crate provides validation rules and a validator for Sruja architectures.
//! It checks for correctness, best practices, and potential issues.

pub mod rules;
pub mod validator;

pub use validator::{Validator, ValidatorOptions, Rule};
pub use rules::*;
