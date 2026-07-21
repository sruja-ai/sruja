use std::path::Path;

use crate::commands::focus::types::{FocusBriefing, MemoryHit, SurfacedLearnings};
use crate::commands::context::types::TokenBudget;
use crate::enrichment::EnrichmentRef;
use crate::integrations::{load_repo_config, EnrichmentResult, DEFAULT_ENRICHMENT_PROMPT_TEMPLATE};
use sruja_agent::{
    calibration, AgenticMemory, AskInput, AskPlan, ExperimentOutcome, MemoryError, TargetHints,
    Thresholds,
};

pub fn surface_agent_learnings(
    repo_path: &Path,
    target_id: &str,
    record_retrievals: bool,
) -> Result<SurfacedLearnings, MemoryError> {
    let mut memory = AgenticMemory::load(repo_path)?;

    let repo_cfg = load_repo_config(repo_path);
    let auto_prune = repo_cfg
        .as_ref()
        .and_then(|c| c.agent.auto_prune)
        .unwrap_or(false);

    if auto_prune {
        let last_pruned = crate::commands::context_events::read_context_events_query(
            repo_path,
            crate::commands::context_events::ContextEventQuery {
                limit: 1,
                kind_filter: Some("memory_pruned"),
                details_substring: None,
                decision_id: None,
                trace_id: None,
                run_id: None,
                element_id: None,
                decision_lineage_only: false,
            },
        )
        .ok()
        .and_then(|events| events.first().cloned());

        let run_prune = match last_pruned {
            None => true,
            Some(ev) => {
                if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(&ev.timestamp) {
                    let duration =
                        chrono::Utc::now().signed_duration_since(ts.with_timezone(&chrono::Utc));
                    duration.num_hours() >= 24
                } else {
                    true
                }
            }
        };

        if run_prune {
            let archived = memory.auto_archive_stale(0.15, 30);
            let pruned_ids: Vec<String> = archived.iter().map(|e| e.id.clone()).collect();
            if !pruned_ids.is_empty() {
                let _ = memory.save(repo_path);
            }
            let details = serde_json::json!({
                "pruned_count": pruned_ids.len(),
                "pruned_ids": pruned_ids,
            });
            let record = crate::commands::context_events::ContextEventRecord {
                schema_version: crate::commands::context_events::CONTEXT_EVENTS_SCHEMA_V2
                    .to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                kind: "memory_pruned".to_string(),
                outcome: "ok".to_string(),
                details,
                ..Default::default()
            };
            crate::commands::context_events::append_context_event(repo_path, record);
        }
    }

    let mut relevant = memory.find_relevant(target_id);
    relevant.sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| a.id.cmp(&b.id)));

    let ce_cfg = load_repo_config(repo_path)
        .map(|c| c.context_engineering)
        .unwrap_or_default();
    let max_items = ce_cfg.bm25_max_results_focus.unwrap_or(10).max(1);

    let mut budget = TokenBudget::new(800);
    let mut hits = Vec::new();
    let mut ids = Vec::new();
    let mut anti_patterns = Vec::new();
    let mut pointer_traces = Vec::new();
    let mut truncated = false;

    for entry in relevant.into_iter().take(max_items) {
        let match_reason = if entry.affected_elements.iter().any(|e| {
            e == target_id
                || target_id.starts_with(&format!("{}.", e))
                || e.starts_with(&format!("{}.", target_id))
        }) {
            "affected_elements"
        } else if entry
            .context
            .to_lowercase()
            .contains(&target_id.to_lowercase())
        {
            "context_keyword"
        } else {
            "unknown"
        };

        let hit_str = format!(
            "{} {} {} {}",
            entry.id,
            entry.hypothesis,
            entry.guardrail_advice,
            entry.reason.clone().unwrap_or_default()
        );
        if budget
            .used_tokens
            .saturating_add(TokenBudget::estimate_tokens(&hit_str))
            > budget.max_tokens
        {
            truncated = true;
            break;
        }
        budget.used_tokens = budget
            .used_tokens
            .saturating_add(TokenBudget::estimate_tokens(&hit_str));

        let outcome = match entry.outcome {
            ExperimentOutcome::Success => "success",
            ExperimentOutcome::Failed => "failed",
        }
        .to_string();
        let kind = entry.kind.map(|k| format!("{k:?}").to_lowercase());

        ids.push(entry.id.clone());
        hits.push(MemoryHit {
            id: entry.id.clone(),
            kind,
            hitl_kind: entry.hitl_kind.clone(),
            outcome,
            match_reason: match_reason.to_string(),
            timestamp: entry.timestamp.to_rfc3339(),
            hypothesis: entry.hypothesis.clone(),
            guardrail_advice: entry.guardrail_advice.clone(),
        });

        anti_patterns.push(entry.guardrail_advice.clone());
        if let Some(reason) = &entry.reason {
            pointer_traces.push(format!(
                "Failed hypothesis: {} ({})",
                entry.hypothesis, reason
            ));
        } else {
            pointer_traces.push(format!("Prior learning: {}", entry.hypothesis));
        }
    }

    if record_retrievals && !ids.is_empty() {
        let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        memory.record_retrievals(&refs);
        memory.save(repo_path)?;
    }

    Ok(SurfacedLearnings {
        hits,
        ids,
        truncated,
        anti_patterns,
        pointer_traces,
    })
}

pub fn load_ask_thresholds(repo_path: &Path) -> Thresholds {
    let mut t = Thresholds::default();
    if let Some(cfg) = load_repo_config(repo_path) {
        if let Some(ask) = cfg.ask {
            if let Some(v) = ask.blast_ask {
                t.blast_ask = v;
            }
            if let Some(v) = ask.confidence_floor {
                t.confidence_floor = v;
            }
            if let Some(v) = ask.confidence_flag {
                t.confidence_flag = v;
            }
            if let Some(v) = ask.trust_default {
                t.trust_default = v;
            }
        }
    }
    t
}

pub fn compute_ask_plan(
    kind: &str,
    label: &str,
    blast_total: usize,
    confidence: Option<u8>,
    memory_hits: &[MemoryHit],
    thresholds: &Thresholds,
) -> AskPlan {
    let has_precedent = memory_hits
        .iter()
        .any(|h| h.hitl_kind.as_deref() == Some("precedent"));
    let reversibility = calibration::infer_reversibility(TargetHints { kind, label });
    let input = AskInput {
        reversibility,
        blast_radius: blast_total.min(u16::MAX as usize) as u16,
        confidence,
        trust_level: None,
        has_precedent,
        policy_says_ask: false,
    };
    calibration::decide(&input, thresholds)
}

pub(super) fn build_focus_enrichment(
    repo_path: &Path,
    briefing: &FocusBriefing,
    enrich: &EnrichmentRef<'_>,
) -> Option<EnrichmentResult> {
    let payload = serde_json::json!({
        "schema_version": "focus_enrichment_input/v1",
        "repo": repo_path.display().to_string(),
        "briefing": briefing,
    });
    crate::integrations::build_enrichment(
        repo_path,
        &payload,
        enrich,
        "You are a careful repo assistant. Never fabricate.",
        DEFAULT_ENRICHMENT_PROMPT_TEMPLATE,
    )
}
