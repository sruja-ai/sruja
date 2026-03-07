//! CLI commands module.
//!
//! Commands are split by domain: dsl, scan, analyze, runtime, intent.
//! See REFACTORING_PLAN.md for the layout.

pub mod ai;
mod analyze;
mod dsl;
mod error;
mod intent;
pub mod llm;
mod scan;
pub mod timeline;
mod version;

pub use analyze::{analyze, complexity, semantic_analyze};
pub use dsl::{
    compile, diff, explain, export, fmt, import, lint, list_elements, lsp, tree, validate,
};
pub use error::CliError;
pub use intent::{intent_check, intent_propose};
pub use scan::{drift, drift_pr, quickstart, scan, smart_coverage, why};
pub use version::version;
mod context;
mod runtime;

pub use context::context_export;
pub use runtime::runtime_analyze;
