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
mod impact;
mod init;
mod intent;
mod knowledge;
mod mcp;
mod review;
mod scan;
mod sources;
mod status;
mod sync_cmd;
mod version;
mod watch;
mod why;

pub use check::{baseline, check};
pub use compliance::compliance;
pub use discover::{discover_context, discover_explain, discover_questions, discover_repomap_cmd};
pub use dsl::{
    compile, diff, explain, export, fmt, import, lint, list_elements, lsp, tree, validate,
};
pub use error::CliError;

pub fn parse_sruja_file<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<(String, sruja_language::ast::Program), CliError> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)?;
    let parser = sruja_language::Parser::new(path.to_string_lossy().to_string());

    match parser.parse(&content) {
        Ok(program) => Ok((content, program)),
        Err(mut diagnostics) => {
            crate::modules::validation::enrich_diagnostics_with_source(&content, &mut diagnostics);
            for diag in &diagnostics {
                eprintln!("{}", sruja_diagnostics::format_diagnostic(diag));
            }
            Err(CliError::Parse {
                file: path.to_string_lossy().to_string(),
                message: format!("Parsing failed with {} errors", diagnostics.len()),
                diagnostics,
            })
        }
    }
}
pub use federation::{compose, publish};
pub use generate::generate_prompt;
pub use impact::impact;
pub use init::init;
pub use intent::{intent_check, intent_propose};
pub use knowledge::knowledge;
pub use mcp::mcp;
pub use review::review;
pub use scan::{drift, drift_pr, quickstart, scan};
pub use sources::sources;
pub use status::status;
pub use sync_cmd::sync;
pub use version::version;
pub use watch::watch;
pub use why::why;
mod context;
mod runtime;

pub use context::context_export;
pub use runtime::runtime_analyze;

pub(crate) fn scan_repo_cached(repo_path: &std::path::Path) -> Result<sruja_scan::Graph, CliError> {
    let graph_path = repo_path.join(".sruja").join("graph.json");
    if graph_path.exists() {
        let content = std::fs::read_to_string(&graph_path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(sruja_scan::scan_repo(repo_path)?)
    }
}
