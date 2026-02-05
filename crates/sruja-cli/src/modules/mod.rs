//! CLI modules
//!
//! This module contains reusable, modular components for CLI operations.

pub mod file_operations;
pub mod validation;

// Re-export only what the rest of the CLI uses (commands.rs uses collect_sruja_files).
pub use file_operations::collect_sruja_files;
