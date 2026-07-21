//! `sruja agent loop` — the closed-loop autonomous coding agent.
//!
//! Drives the full cognition loop (comprehend -> plan -> execute via tools ->
//! critique -> replan until approved) against a real workspace using the
//! `sruja-agent` crate's `Agent::run_loop`.
//!
//! This is the CLI-first path that turns Sruja from a passive harness into an
//! autonomous actor graded by its own deterministic tools.
//!
//! # Configuration
//!
//! Uses the industry-standard resolution chain:
//! 1. CLI flags (highest priority)
//! 2. Environment variables (provider-specific)
//! 3. `.sruja/config.toml` (non-secrets only)
//! 4. Built-in defaults (lowest priority)
//!
//! ## Multi-Provider Support
//!
//! Configure different providers for different tasks in `.sruja/config.toml`:
//!
//! ```toml
//! [integrations]
//! default_provider = "zai"
//!
//! [integrations.providers.zai]
//! base_url = "https://api.z.ai/api/coding/paas/v4"
//! key_env = "ZAI_API_KEY"
//!
//! [agent.models]
//! cheap = { provider = "zai", model = "GLM-4-Flash" }
//! mid = { provider = "zai", model = "GLM-4.7" }
//! premium = { provider = "openrouter", model = "anthropic/claude-sonnet-4" }
//! review = { provider = "openrouter", model = "google/gemini-2.5-flash" }
//! ```
//!
//! See `config::resolve_multi_provider_config` for details.

mod calibration;
mod config;
mod output;
mod run;
mod utils;

pub use config::AgentLoopOptions;
pub use run::agent_loop;

#[cfg(test)]
mod tests {
    use super::output::print_loop_result_human;
    use super::utils::consolidate_memory;

    // ── U2: consolidate_memory ────────────────────────────────────────────

    fn make_learning_entry(
        context: &str,
        hypothesis: &str,
        retrieval_count: u32,
        success: u32,
        total: u32,
        age_days: i64,
    ) -> sruja_agent::LearningEntry {
        use chrono::{Duration, Utc};
        sruja_agent::LearningEntry {
            id: sruja_agent::generate_entry_id(),
            kind: None,
            timestamp: Utc::now() - Duration::days(age_days),
            run_id: None,
            repo: None,
            selector: None,
            context: context.to_string(),
            hypothesis: hypothesis.to_string(),
            outcome: sruja_agent::ExperimentOutcome::Failed,
            reason: None,
            guardrail_advice: String::new(),
            affected_elements: vec![],
            evidence_refs: vec![],
            confidence: None,
            tags: vec![],
            hitl_kind: None,
            related_ids: vec![],
            retrieval_count,
            task_success_after: success,
            task_total_after: total,
            category: None,
            signals_match: vec![],
            constraints: None,
            validation: vec![],
            blast_radius: None,
        }
    }

    fn make_invariant_entry(context: &str) -> sruja_agent::LearningEntry {
        let mut e = make_learning_entry(context, "invariant hyp", 10, 0, 10, 60);
        e.kind = Some(sruja_agent::LearningKind::Invariant);
        e
    }

    #[test]
    fn consolidate_archives_stale_entries() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        let mut mem = sruja_agent::AgenticMemory::default();

        mem.add_learning(make_learning_entry(
            "stale context",
            "stale hypothesis",
            1,
            1,
            1,
            365,
        ));
        mem.add_learning(make_learning_entry(
            "fresh context",
            "fresh hypothesis",
            1,
            1,
            1,
            1,
        ));
        mem.save(repo).unwrap();

        let summary = consolidate_memory(repo).unwrap();
        assert!(summary.contains("archived 1 stale"), "Summary: {summary}");
        assert!(
            summary.contains("pruned 0 low-utility"),
            "Summary: {summary}"
        );

        let loaded = sruja_agent::AgenticMemory::load(repo).unwrap();
        assert_eq!(loaded.learnings.len(), 1, "Only fresh entry should remain");
        assert_eq!(loaded.learnings[0].context, "fresh context");
    }

    #[test]
    fn consolidate_prunes_low_utility_entries() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        let mut mem = sruja_agent::AgenticMemory::default();

        mem.add_learning(make_learning_entry(
            "low utility ctx",
            "low utility hyp",
            5,
            1,
            5,
            10,
        ));
        mem.add_learning(make_learning_entry(
            "high utility ctx",
            "high utility hyp",
            0,
            0,
            0,
            10,
        ));
        mem.save(repo).unwrap();

        let summary = consolidate_memory(repo).unwrap();
        assert!(
            summary.contains("pruned 1 low-utility"),
            "Summary: {summary}"
        );

        let loaded = sruja_agent::AgenticMemory::load(repo).unwrap();
        assert_eq!(loaded.learnings.len(), 1);
        assert_eq!(loaded.learnings[0].context, "high utility ctx");
    }

    #[test]
    fn consolidate_preserves_invariant_entries() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        let mut mem = sruja_agent::AgenticMemory::default();

        mem.add_learning(make_invariant_entry("invariant must not be pruned"));
        mem.save(repo).unwrap();

        let summary = consolidate_memory(repo).unwrap();
        assert!(
            summary.contains("pruned 0"),
            "Invariants must not be pruned: {summary}"
        );

        let loaded = sruja_agent::AgenticMemory::load(repo).unwrap();
        assert_eq!(loaded.learnings.len(), 1);
    }

    #[test]
    fn consolidate_no_op_when_empty() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();

        let summary = consolidate_memory(repo).unwrap();
        assert!(summary.contains("archived 0 stale"));
        assert!(summary.contains("pruned 0 low-utility"));
        assert!(summary.contains("0 entries remain"));
    }

    // ── U3: print_loop_result_human observability ─────────────────────────

    #[test]
    fn print_loop_result_shows_applied_learnings() {
        use sruja_agent::llm::Usage;
        use sruja_agent::{Comprehension, Critique, LoopResult, LoopTermination};

        let result = LoopResult {
            goal: "test goal".to_string(),
            converged: true,
            termination: LoopTermination::Approved,
            iterations: vec![],
            final_result: sruja_agent::AgentRunResult {
                goal: "test goal".to_string(),
                comprehension: Comprehension {
                    goal: "test goal".to_string(),
                    summary: "test".to_string(),
                    cited_elements: vec![],
                    key_findings: vec![],
                    risks: vec![],
                    usage: Usage::default(),
                    retrieved_learning_ids: vec!["lrn_abc".to_string(), "lrn_def".to_string()],
                    complexity: sruja_agent::TaskComplexity::default(),
                    pre_conditions: vec![],
                },
                plan: sruja_agent::Plan {
                    goal: "test goal".to_string(),
                    goal_statement: "test goal".to_string(),
                    criteria: vec![],
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                    schema_version: String::new(),
                    complexity: sruja_agent::TaskComplexity::default(),
                },
                step_results: vec![],
                critique: Some(Critique {
                    approved: true,
                    score: 0.9,
                    issues: vec![],
                    suggestions: vec![],
                    usage: Usage::default(),
                    persona_breakdown: vec![],
                    injected_learning_ids: vec!["lrn_abc".to_string()],
                    criteria: vec![],
                    source: String::new(),
                }),
                decision: None,
                runbook: None,
                total_usage: Usage::default(),
            },
            total_usage: Usage::default(),
            grader_source: "default".to_string(),
        };

        print_loop_result_human(&result, false);
    }

    #[test]
    fn print_loop_result_shows_hint_when_no_learnings_on_failure() {
        use sruja_agent::llm::Usage;
        use sruja_agent::{Comprehension, Critique, LoopResult, LoopTermination};

        let result = LoopResult {
            goal: "test goal".to_string(),
            converged: false,
            termination: LoopTermination::MaxIterations,
            iterations: vec![],
            final_result: sruja_agent::AgentRunResult {
                goal: "test goal".to_string(),
                comprehension: Comprehension {
                    goal: "test goal".to_string(),
                    summary: "test".to_string(),
                    cited_elements: vec![],
                    key_findings: vec![],
                    risks: vec![],
                    usage: Usage::default(),
                    retrieved_learning_ids: vec![],
                    complexity: sruja_agent::TaskComplexity::default(),
                    pre_conditions: vec![],
                },
                plan: sruja_agent::Plan {
                    goal: "test goal".to_string(),
                    goal_statement: "test goal".to_string(),
                    criteria: vec![],
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                    schema_version: String::new(),
                    complexity: sruja_agent::TaskComplexity::default(),
                },
                step_results: vec![],
                critique: Some(Critique {
                    approved: false,
                    score: 0.3,
                    issues: vec!["bad".to_string()],
                    suggestions: vec![],
                    usage: Usage::default(),
                    persona_breakdown: vec![],
                    injected_learning_ids: vec![],
                    criteria: vec![],
                    source: String::new(),
                }),
                decision: None,
                runbook: None,
                total_usage: Usage::default(),
            },
            total_usage: Usage::default(),
            grader_source: "default".to_string(),
        };

        print_loop_result_human(&result, false);
    }
}
