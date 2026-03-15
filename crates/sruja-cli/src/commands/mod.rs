//! CLI commands module.
//!
//! Commands are split by domain: dsl, scan, runtime, intent.
//! See REFACTORING_PLAN.md for the layout.

mod check;
mod compliance;
mod discover;
mod dsl;
mod error;
mod federation;
mod generate;
mod init;
mod intent;
mod knowledge;
mod review;
mod scan;
mod status;
mod sync_cmd;
mod version;

pub use check::check;
pub use compliance::compliance;
pub use discover::{discover_context, discover_questions};
pub use dsl::{
    compile, diff, explain, export, fmt, import, lint, list_elements, lsp, tree, validate,
};
pub use error::CliError;
pub use federation::{compose, publish};
pub use generate::generate_prompt;
pub use init::init;
pub use intent::{intent_check, intent_propose};
pub use knowledge::knowledge;
pub use review::review;
pub use scan::{drift, drift_pr, quickstart, scan};
pub use status::status;
pub use sync_cmd::sync;
pub use version::version;
mod context;
mod runtime;

pub use context::context_export;
pub use runtime::runtime_analyze;
