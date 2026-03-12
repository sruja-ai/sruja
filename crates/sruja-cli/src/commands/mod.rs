//! CLI commands module.
//!
//! Commands are split by domain: dsl, scan, analyze, runtime, intent.
//! See REFACTORING_PLAN.md for the layout.

mod analyze;
mod compliance;
mod discover;
mod dsl;
mod error;
mod generate;
mod intent;
mod scan;
mod version;

pub use analyze::{analyze, complexity, semantic_analyze};
pub use compliance::compliance;
pub use discover::{discover_context, discover_questions};
pub use dsl::{
    compile, diff, explain, export, fmt, import, lint, list_elements, lsp, tree, validate,
};
pub use error::CliError;
pub use generate::generate_prompt;
pub use intent::{intent_check, intent_propose};
pub use scan::{drift, drift_pr, quickstart, scan, smart_coverage, why};
pub use version::version;
mod context;
mod runtime;

pub use context::context_export;
pub use runtime::runtime_analyze;
