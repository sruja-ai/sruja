//! CLI commands module.
//!
//! Commands are split by domain: dsl, scan, analyze, runtime, intent.
//! See REFACTORING_PLAN.md for the layout.

mod error;
mod version;
mod dsl;
mod scan;
mod analyze;
mod runtime;
mod intent;
mod stakeholder;
pub mod timeline;
pub mod llm;

pub use error::CliError;
pub use version::version;
pub use dsl::{
    compile, diff, explain, export, fmt, import, lint, list_elements, lsp, tree, validate,
};
pub use scan::{drift, drift_pr, quickstart, scan, why};
pub use analyze::{analyze, complexity, semantic_analyze};
pub use runtime::runtime_analyze;
pub use intent::{intent_check, intent_propose};
pub use stakeholder::{cto, devops, product, security, sre};
mod context;

pub use context::context_export;
