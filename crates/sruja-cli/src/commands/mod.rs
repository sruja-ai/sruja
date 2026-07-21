//! CLI commands module.
//!
//! Commands are split by domain: dsl, scan, intent.
//! See REFACTORING_PLAN.md for the layout.

pub mod analysis;
pub mod author;
pub mod compress_stats;
pub mod density;
pub mod dsl_domain;
pub mod eval;
pub mod intent_domain;
pub mod loop_grader;
pub mod loop_report;
pub mod scan_domain;
pub mod snippet;
pub mod utility_domain;
pub mod workflow;
pub(crate) mod workflow_aidlc;

pub use intent_domain::remediation;

pub use agent_loop::{agent_loop as run_agent_loop, AgentLoopOptions};
pub use agent_reflect::agent_reflect;
pub use auto_cmd::auto_run;
pub use intent_domain::agent::{
    agent_clear, agent_clusters, agent_curate, agent_delete, agent_history, agent_learn,
    agent_merge, agent_propose_fact, agent_record, agent_session_summary, agent_update,
};
pub use intent_domain::agent_plan::{agent_apply, agent_plan};
pub use intent_domain::agent_run::{agent_run, agent_run_to_string, AgentRunOptions};
pub use intent_domain::ai::{ai_brief, AiBriefOptions};
pub use intent_domain::evolution::{evaluate, evolution_log};
pub use intent_domain::propose::*;
pub use plan_cmd::plan_run;
pub use verify_cmd::verify_run;

pub use dsl_domain::check::{baseline, check};
pub use dsl_domain::dsl::{
    compile, diff, explain, export, fmt, import, lint, list_elements, lsp, tree, validate,
};
pub use intent_domain::focus::focus;
pub use intent_domain::ingest::ingest;
pub use scan_domain::context_graph::context_graph;
pub use scan_domain::context_score::context_score;
pub use scan_domain::discover::{
    discover_context, discover_explain, discover_questions, discover_repomap_cmd,
};
pub use scan_domain::explore::explore;
pub use scan_domain::graph_history::graph_history;
pub use utility_domain::error::CliError;

/// Re-export enrichment types for use by command handlers.
#[allow(unused_imports)]
pub use crate::enrichment::{EnrichmentArgs, EnrichmentRef};

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
pub use author::{author_evidence, author_propose};
pub use confidence::{confidence, format_confidence, ConfidenceOptions};
pub use density::density;
pub use drift_state::drift_state_print as drift_state;
pub use dsl_domain::completions::completions;
pub use dsl_domain::generate::generate_prompt;
pub use dsl_domain::watch::watch;
pub use intent_domain::intent::{intent_check, intent_propose};
pub use intent_domain::onboard::onboard;
pub use intent_domain::verify_task::{format_verify_task, verify_task, VerifyTaskOptions};
pub use scan_domain::health::health;
pub use scan_domain::impact::impact;
pub(crate) use scan_domain::impact::impact_compute_output;
pub use scan_domain::index::{query_registry, registry_dashboard, registry_index, semantic_index};
pub use scan_domain::mcp::{mcp, mcp_v2};
pub use scan_domain::review::{review, review_design};
pub use scan_domain::scan::{drift, drift_pr, quickstart, scan};
pub use scan_domain::status::status;
pub use scan_domain::sync_cmd::sync;
pub use scan_domain::why::why;
pub use utility_domain::init::init;
pub use utility_domain::run_export::run_export;
pub use utility_domain::run_show::run_show;
pub use utility_domain::version::version;

pub use workflow::{
    workflow_advance, workflow_approve, workflow_audit, workflow_capture_requirements,
    workflow_gate_check, workflow_get, workflow_init, workflow_install_rules, workflow_list,
    workflow_next_steps, workflow_record_impact, workflow_record_readiness,
    workflow_record_test_results, workflow_run, workflow_status, workflow_summary, workflow_trace,
    workflow_validate, WorkflowInitOptions,
};

pub use intent_domain::critique;
pub use intent_domain::focus;
pub use scan_domain::discover;
pub use scan_domain::scan;
pub use scan_domain::sync_cmd;
pub use utility_domain::error;
pub use utility_domain::federation;
pub use utility_domain::preflight;
#[allow(unused_imports)]
pub use utility_domain::repo_manifest;
pub use utility_domain::violation_shared;

pub use intent_domain::critique::critique;
pub use utility_domain::compliance::compliance;
pub use utility_domain::federation::{compose, publish};

pub mod agent_loop;
pub mod agent_reflect;
pub mod agent_setup;
pub mod auto_cmd;
pub mod extensions_config;
pub mod learn;
pub mod plan_cmd;
pub mod verify_cmd;
pub use learn::learn;
pub use lookup::lookup;

pub(crate) mod loop_checkpoint;
pub(crate) mod loop_events;

pub mod before;
pub mod cognitive_debt;
pub mod confidence;
pub mod drift_velocity;
pub mod explain_cmd;
pub mod lookup;
pub mod map_cmd;
pub mod trace_cmd;
pub mod what_if;

pub(crate) mod context;
pub(crate) mod context_events;
pub(crate) mod context_prune;
pub mod decision;
pub(crate) mod diagnostic_vfs;
pub(crate) mod drift_state;
pub mod event;
pub(crate) mod mcp_prompts;
pub(crate) mod mcp_resources;
pub(crate) mod memory_cmd;
pub mod requirements;

pub use context::{context_export, ContextRequest};
pub use decision::{
    create_decision_record, decision_accept, decision_link, decision_list, decision_new,
    decision_show, decision_supersede, decision_trace, list_decisions,
};
pub use event::{event_append, event_list};
pub use memory_cmd::{
    memory_archive, memory_reindex, memory_search, memory_skill_stats, memory_timeline,
};
pub use requirements::requirements_list;

// Re-export cache functions from sruja-cache crate.
pub(crate) use sruja_cache::{
    compute_all_centrality_cached, git_commit_short, scan_repo_cached, scan_repo_cached_with_opts,
    ScanCache, SCAN_CACHE_PATH,
};

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
