//! Skills filtering and loading module (simplified version)
//!
//! Provides intelligent filtering of Rust skills based on context,
//! project analysis, and developer experience level.

pub mod context;
pub mod filter;
pub mod loader;
pub mod suggest;

pub use filter::{OutputFormat, SkillFilter};
pub use loader::load_filtered_skills;
pub use suggest::suggest_rules;
