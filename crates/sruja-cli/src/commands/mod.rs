//! CLI commands module.
//!
//! Commands are split by domain: dsl, scan, intent.
//! See REFACTORING_PLAN.md for the layout.

pub mod dsl_domain;
pub mod intent_domain;
pub mod scan_domain;
pub mod utility_domain;

pub use intent_domain::remediation;

pub use intent_domain::agent::{agent_clear, agent_clusters, agent_history, agent_record};
pub use intent_domain::agent_plan::{agent_apply, agent_plan};
pub use intent_domain::agent_run::{agent_run, agent_run_to_string, AgentRunOptions};
pub use intent_domain::ai::{ai_brief, AiBriefOptions};
pub use intent_domain::evolution::{evaluate, evolution_log};
pub use intent_domain::propose::*;

pub use dsl_domain::check::{baseline, check};
pub use dsl_domain::dsl::{
    compile, diff, explain, export, fmt, import, lint, list_elements, lsp, tree, validate,
    ExportOptions,
};
pub use intent_domain::focus::focus;
pub use intent_domain::ingest::ingest;
pub use scan_domain::context_graph::context_graph;
pub use scan_domain::context_score::context_score;
pub use scan_domain::discover::{
    discover_context, discover_explain, discover_questions, discover_repomap_cmd,
};
pub use utility_domain::error::CliError;

#[derive(Debug, Clone, Copy)]
pub struct LlmConfig<'a> {
    pub provider: Option<&'a str>,
    pub model: Option<&'a str>,
    pub base_url: Option<&'a str>,
}

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
pub use dsl_domain::completions::completions;
pub use dsl_domain::generate::generate_prompt;
pub use dsl_domain::watch::watch;
pub use intent_domain::intent::{intent_check, intent_propose};
pub use intent_domain::onboard::onboard;
pub use scan_domain::health::health;
pub use scan_domain::impact::impact;
pub use scan_domain::index::{query_registry, registry_dashboard, registry_index, semantic_index};
pub use scan_domain::mcp::mcp;
pub use scan_domain::review::review;
pub use scan_domain::scan::{drift, drift_pr, quickstart, scan};
pub use scan_domain::status::status;
pub use scan_domain::sync_cmd::sync;
pub use scan_domain::why::why;
pub use utility_domain::init::init;
pub use utility_domain::run_export::run_export;
pub use utility_domain::run_show::run_show;
pub use utility_domain::version::version;

pub use dsl_domain::check;
pub use intent_domain::critique;
pub use intent_domain::focus;
pub use scan_domain::discover;
pub use scan_domain::scan;
pub use scan_domain::sync_cmd;
pub use utility_domain::error;
pub use utility_domain::federation;
pub use utility_domain::preflight;
pub use utility_domain::violation_shared;

pub use intent_domain::critique::critique;
pub use utility_domain::compliance::compliance;
pub use utility_domain::federation::{compose, publish};

pub mod learn;
pub use learn::learn;

pub(crate) mod context;
pub(crate) mod context_events;
pub(crate) mod context_prune;
pub mod decision;
pub(crate) mod diagnostic_vfs;
pub(crate) mod drift_state;
pub mod event;
pub(crate) mod mcp_prompts;
pub(crate) mod mcp_resources;

pub use context::{context_export, sync_ide_rules, sync_ide_rules_check, ContextRequest};
pub use decision::{
    create_decision_record, decision_accept, decision_link, decision_list, decision_new,
    decision_show, decision_supersede, decision_trace, list_decisions,
};
pub use event::{event_append, event_list};

pub(crate) fn scan_repo_cached(repo_path: &std::path::Path) -> Result<sruja_scan::Graph, CliError> {
    scan_repo_cached_with_opts(repo_path, false)
}

pub(crate) fn scan_repo_cached_with_opts(
    repo_path: &std::path::Path,
    incremental: bool,
) -> Result<sruja_scan::Graph, CliError> {
    let graph_path = repo_path.join(".sruja").join("graph.json");

    if !incremental && graph_path.exists() {
        let content = std::fs::read_to_string(&graph_path)?;
        if let Ok(graph) = serde_json::from_str::<sruja_scan::Graph>(&content) {
            return Ok(graph);
        }
    }

    let graph = if incremental {
        sruja_scan::scan_repo_incremental(repo_path)?
    } else {
        sruja_scan::scan_repo(repo_path)?
    };

    let dir = repo_path.join(".sruja");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    let content = serde_json::to_string_pretty(&graph)?;
    let _ = std::fs::write(graph_path, content);

    Ok(graph)
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
