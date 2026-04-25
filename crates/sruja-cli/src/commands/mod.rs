//! CLI commands module.
//!
//! Commands are split by domain: dsl, scan, intent.
//! See REFACTORING_PLAN.md for the layout.

mod agent;
mod check;
mod completions;
pub mod compliance;
mod context_graph;
mod context_score;
pub mod critique;
mod discover;
mod dsl;
mod error;
mod federation;
mod focus;
mod generate;
mod health;
mod impact;
mod index;
mod ingest;
mod init;
mod intent;
mod mcp;
mod preflight;
mod propose;
mod review;
mod scan;
mod status;
mod sync_cmd;
mod version;
pub mod violation_shared;
mod watch;
mod why;

pub use agent::{agent_clear, agent_history, agent_record};
pub use propose::*;

pub use check::{baseline, check};
pub use compliance::compliance;
pub use context_graph::context_graph;
pub use context_score::context_score;
pub use critique::critique;
pub use discover::{discover_context, discover_explain, discover_questions, discover_repomap_cmd};
pub use dsl::{
    compile, diff, explain, export, fmt, import, lint, list_elements, lsp, tree, validate,
    ExportOptions,
};
pub use error::CliError;
pub use focus::focus;
pub use ingest::ingest;

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
                help: Some("Run 'sruja lint <file>' for detailed validation output.".into()),
                fix: Some("Fix the syntax errors in the file, then re-run 'sruja lint'.".into()),
            })
        }
    }
}
pub use completions::completions;
pub use federation::{compose, publish};
pub use generate::generate_prompt;
pub use health::health;
pub use impact::impact;
pub use index::{query_registry, registry_dashboard, registry_index, semantic_index};
pub use init::init;
pub use intent::{intent_check, intent_propose};
pub use mcp::mcp;
pub use review::review;
pub use scan::{drift, drift_pr, quickstart, scan};
pub use status::status;
pub use sync_cmd::sync;
pub use version::version;
pub use watch::watch;
pub use why::why;
mod context;

pub use context::{context_export, ContextRequest};

pub(crate) fn scan_repo_cached(repo_path: &std::path::Path) -> Result<sruja_scan::Graph, CliError> {
    let graph_path = repo_path.join(".sruja").join("graph.json");

    if graph_path.exists() {
        let content = std::fs::read_to_string(&graph_path)?;
        if let Ok(graph) = serde_json::from_str::<sruja_scan::Graph>(&content) {
            return Ok(graph);
        }
    }

    Ok(sruja_scan::scan_repo(repo_path)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn parse_succeeds_on_minimal_valid_file() {
        let dir = tempdir().expect("temp");
        let file = dir.path().join("ok.sruja");
        fs::write(
            &file,
            r#"MySystem = system "My System" {
  description "A deployable system"
}
"#,
        )
        .expect("write");
        let (content, program) = parse_sruja_file(&file).expect("parse");
        assert!(content.contains("My System"));
        assert!(!program.items.is_empty());
    }

    #[test]
    fn parse_fails_on_invalid_file() {
        let dir = tempdir().expect("temp");
        let file = dir.path().join("bad.sruja");
        fs::write(&file, "invalid {").expect("write");
        let err = parse_sruja_file(&file).expect_err("expected error");
        match err {
            CliError::Parse { file: f, .. } => {
                assert!(f.ends_with("bad.sruja"));
            }
            other => panic!("expected Parse error, got {:?}", other),
        }
    }
}
