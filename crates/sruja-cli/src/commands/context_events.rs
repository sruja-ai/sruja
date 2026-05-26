//! Append-only **context lineage** events under `.sruja/context_events.jsonl`.
//!
//! These records approximate “decision traces” for software architecture: intent checks,
//! drift runs, and merged proposals, each stamped with a fingerprint of the declared
//! architecture file in use at the time (when resolvable).
//!
//! **`context_event/v2`** adds optional trace fields (`trace_id`, `decision_id`, `actor`, …)
//! for agent workflows while remaining backward compatible with **`context_event/v1`**
//! readers that ignore unknown JSON properties.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sruja_intent::{DriftReport, Severity};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub const CONTEXT_EVENTS_SCHEMA: &str = "context_event/v1";
pub const CONTEXT_EVENTS_SCHEMA_V2: &str = "context_event/v2";

/// Event `kind` values used for decision / agent workflow lineage (v2 and tooling).
pub const DECISION_LINEAGE_KINDS: &[&str] = &[
    "decision_opened",
    "agent_plan",
    "context_retrieved",
    "evidence_cited",
    "alternative_considered",
    "human_handoff",
    "override_recorded",
    "decision_accepted",
    "decision_superseded",
    "decision_applied",
    "validation_passed",
    "validation_failed",
];

fn events_path(repo: &Path) -> std::path::PathBuf {
    repo.join(".sruja").join("context_events.jsonl")
}

/// Blake3 hex digest of the resolved default architecture file (`repo.sruja`, etc.), if present.
pub fn policy_fingerprint(repo: &Path) -> Option<String> {
    let p = crate::utils::architecture_path::resolve_architecture_path(repo)?;
    let bytes = std::fs::read(&p).ok()?;
    Some(blake3::hash(&bytes).to_hex().to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEventRecord {
    pub schema_version: String,
    pub timestamp: String,
    pub kind: String,
    pub outcome: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    #[serde(default)]
    pub details: serde_json::Value,
    // --- context_event/v2 optional lineage ---
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elements: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_refs: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    // --- context_event/v2 host and session extensions ---
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills_used: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

impl Default for ContextEventRecord {
    fn default() -> Self {
        Self {
            schema_version: CONTEXT_EVENTS_SCHEMA_V2.to_string(),
            timestamp: String::new(),
            kind: String::new(),
            outcome: String::new(),
            policy_fingerprint: None,
            strict: None,
            details: serde_json::json!({}),
            trace_id: None,
            decision_id: None,
            run_id: None,
            workflow_id: None,
            actor: None,
            source: None,
            tool: None,
            elements: None,
            subject_ids: None,
            evidence_refs: None,
            summary: None,
            host: None,
            skills_used: None,
            session_id: None,
        }
    }
}

impl ContextEventRecord {
    /// True if this row is a decision/workflow lineage kind (for focus / MCP filters).
    pub fn is_decision_lineage_kind(&self) -> bool {
        DECISION_LINEAGE_KINDS.contains(&self.kind.as_str())
    }

    /// True if `element_id` matches `elements` or appears in serialized `details`.
    pub fn touches_element(&self, element_id: &str) -> bool {
        if let Some(els) = &self.elements {
            if els
                .iter()
                .any(|e| e == element_id || element_id.starts_with(&format!("{e}.")))
            {
                return true;
            }
        }
        self.details.to_string().contains(element_id)
    }

    fn new_now(
        repo: &Path,
        schema_version: &str,
        kind: &str,
        outcome: &str,
        strict: Option<bool>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            schema_version: schema_version.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            kind: kind.to_string(),
            outcome: outcome.to_string(),
            policy_fingerprint: policy_fingerprint(repo),
            strict,
            details,
            trace_id: None,
            decision_id: None,
            run_id: None,
            workflow_id: None,
            actor: None,
            source: None,
            tool: None,
            elements: None,
            subject_ids: None,
            evidence_refs: None,
            summary: None,
            host: None,
            skills_used: None,
            session_id: None,
        }
    }

    fn new_v1_now(
        repo: &Path,
        kind: &str,
        outcome: &str,
        strict: Option<bool>,
        details: serde_json::Value,
    ) -> Self {
        Self::new_now(repo, CONTEXT_EVENTS_SCHEMA, kind, outcome, strict, details)
    }

    fn new_v2_now(repo: &Path, kind: &str, outcome: &str, details: serde_json::Value) -> Self {
        Self::new_now(repo, CONTEXT_EVENTS_SCHEMA_V2, kind, outcome, None, details)
    }
}

/// Validate a record before appending (CLI / MCP). Matches `schemas/context_event_record.schema.json`.
pub fn validate_context_event_record(r: &ContextEventRecord) -> Result<(), String> {
    if r.schema_version != CONTEXT_EVENTS_SCHEMA && r.schema_version != CONTEXT_EVENTS_SCHEMA_V2 {
        return Err(format!(
            "invalid schema_version: expected {} or {}",
            CONTEXT_EVENTS_SCHEMA, CONTEXT_EVENTS_SCHEMA_V2
        ));
    }
    if r.timestamp.trim().is_empty() {
        return Err("timestamp must be non-empty RFC3339 string".into());
    }
    if r.kind.trim().is_empty() {
        return Err("kind must be non-empty".into());
    }
    if r.outcome.trim().is_empty() {
        return Err("outcome must be non-empty".into());
    }
    if !r.details.is_object() {
        return Err(format!(
            "details must be a JSON object (got {:?}); use {{}} if empty",
            r.details
        ));
    }
    Ok(())
}

pub fn append_context_event(repo: &Path, record: ContextEventRecord) {
    let path = events_path(repo);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        if let Ok(line) = serde_json::to_string(&record) {
            let _ = writeln!(f, "{line}");
        }
    }
}

fn normalize_record_details(record: &mut ContextEventRecord) {
    if record.details.is_null() {
        record.details = serde_json::json!({});
    }
}

/// Parse and validate one JSON line from `sruja event append` or MCP.
pub fn append_context_event_from_json_line(
    repo: &Path,
    line: &str,
) -> Result<ContextEventRecord, String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Err("empty JSON line".into());
    }
    let mut record: ContextEventRecord =
        serde_json::from_str(trimmed).map_err(|e| format!("invalid JSON: {e}"))?;
    normalize_record_details(&mut record);
    validate_context_event_record(&record)?;
    append_context_event(repo, record.clone());
    Ok(record)
}

pub fn record_intent_check(repo: &Path, report: &DriftReport, strict: bool) {
    let critical = report
        .drifts
        .iter()
        .filter(|d| matches!(d.severity, Severity::Critical))
        .count();
    let outcome = if critical > 0 {
        "fail"
    } else if report.drift_score < 70 {
        "warn"
    } else {
        "pass"
    };
    let details = serde_json::json!({
        "drift_score": report.drift_score,
        "health": format!("{:?}", report.health),
        "total_drifts": report.drifts.len(),
        "critical_drifts": critical,
        "boundary_violations": report.summary.boundary_violations,
        "policy_violations": report.summary.policy_violations,
    });
    append_context_event(
        repo,
        ContextEventRecord::new_v1_now(repo, "intent_check", outcome, Some(strict), details),
    );
}

pub fn record_drift_compare(
    repo: &Path,
    violation_count: usize,
    truth_status: &str,
    compared_to_architecture: bool,
) {
    let outcome = if violation_count > 0 { "warn" } else { "pass" };
    append_context_event(
        repo,
        ContextEventRecord::new_v1_now(
            repo,
            "drift",
            outcome,
            None,
            serde_json::json!({
                "violation_count": violation_count,
                "truth_status": truth_status,
                "compared_to_architecture": compared_to_architecture,
            }),
        ),
    );
}

/// Stable id grouping related tasks on the same architectural focus area.
pub fn derive_task_stream_id(element_id: Option<&str>) -> String {
    format!("stream:{}", element_id.unwrap_or("repo"))
}

pub fn record_agent_plan(
    repo: &Path,
    run_id: &str,
    goal: &str,
    element_id: Option<&str>,
    retrieved_learning_ids: Option<&[String]>,
) {
    let task_stream_id = derive_task_stream_id(element_id);
    let details = serde_json::json!({
        "goal": goal,
        "element_id": element_id,
        "task_stream_id": task_stream_id,
        "retrieved_learning_ids": retrieved_learning_ids,
    });
    let base = ContextEventRecord::new_v2_now(repo, "agent_plan", "ok", details);
    append_context_event(
        repo,
        ContextEventRecord {
            trace_id: Some(run_id.to_string()),
            decision_id: None,
            run_id: Some(run_id.to_string()),
            workflow_id: None,
            actor: Some("sruja agent".to_string()),
            source: Some("cli".to_string()),
            tool: Some("agent_run".to_string()),
            elements: element_id.map(|id| vec![id.to_string()]),
            subject_ids: None,
            evidence_refs: None,
            summary: Some(format!("Agent plan for: {goal}")),
            ..base
        },
    );
}

/// Records agent task completion with learning utility feedback for the task stream.
pub fn record_agent_task_complete(
    repo: &Path,
    run_id: &str,
    element_id: Option<&str>,
    retrieved_learning_ids: &[String],
    success: bool,
) {
    let task_stream_id = derive_task_stream_id(element_id);
    let kind = if success {
        "validation_passed"
    } else {
        "validation_failed"
    };
    let outcome = if success { "ok" } else { "failed" };
    let details = serde_json::json!({
        "task_stream_id": task_stream_id,
        "retrieved_learning_ids": retrieved_learning_ids,
        "success": success,
    });
    let base = ContextEventRecord::new_v2_now(repo, kind, outcome, details);
    append_context_event(
        repo,
        ContextEventRecord {
            trace_id: Some(run_id.to_string()),
            run_id: Some(run_id.to_string()),
            actor: Some("sruja agent".to_string()),
            source: Some("cli".to_string()),
            tool: Some("agent_run".to_string()),
            elements: element_id.map(|id| vec![id.to_string()]),
            summary: Some(format!(
                "Agent task {} (stream {})",
                if success { "succeeded" } else { "failed" },
                task_stream_id
            )),
            ..base
        },
    );
}

/// Host compressed chat history after a validation burst; hints middleware to skip re-compress.
pub fn record_context_compressed(
    repo: &Path,
    suppress_recompress_turns: u32,
    compressed_element_ids: Option<Vec<String>>,
    summary: Option<&str>,
) {
    let details = serde_json::json!({
        "suppress_recompress_turns": suppress_recompress_turns,
        "compressed_element_ids": compressed_element_ids,
        "pair_with": "cache_friendly exports (Phase 1) and sruja_suggest_context_prune"
    });
    let base = ContextEventRecord::new_v2_now(repo, "context_compressed", "ok", details);
    append_context_event(
        repo,
        ContextEventRecord {
            summary: summary.map(str::to_string),
            tool: Some("host".to_string()),
            source: Some("mcp".to_string()),
            ..base
        },
    );
}

pub fn record_proposal_merge(repo: &Path, proposal_id: &str) {
    append_context_event(
        repo,
        ContextEventRecord::new_v1_now(
            repo,
            "proposal_merge",
            "ok",
            None,
            serde_json::json!({ "proposal_id": proposal_id }),
        ),
    );
}

#[derive(Debug, Clone, Default)]
pub struct ContextEventQuery<'a> {
    pub limit: usize,
    pub kind_filter: Option<&'a str>,
    pub details_substring: Option<&'a str>,
    pub decision_id: Option<&'a str>,
    pub trace_id: Option<&'a str>,
    pub run_id: Option<&'a str>,
    pub element_id: Option<&'a str>,
    /// When set, only kinds in [`DECISION_LINEAGE_KINDS`].
    pub decision_lineage_only: bool,
}

/// Returns up to `limit` matching events, **newest first** (tail of the log).
pub fn read_context_events_query(
    repo: &Path,
    query: ContextEventQuery<'_>,
) -> std::io::Result<Vec<ContextEventRecord>> {
    let path = events_path(repo);
    if !path.exists() || query.limit == 0 {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(&path)?;
    let mut matched: Vec<ContextEventRecord> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(ev) = serde_json::from_str::<ContextEventRecord>(line) else {
            continue;
        };
        let details_text = ev.details.to_string();
        if let Some(k) = query.kind_filter {
            if ev.kind != k {
                continue;
            }
        }
        if let Some(sub) = query.details_substring {
            if !details_text.contains(sub) {
                continue;
            }
        }
        if let Some(did) = query.decision_id {
            let in_field = ev.decision_id.as_deref() == Some(did);
            let in_details = details_text.contains(did);
            if !in_field && !in_details {
                continue;
            }
        }
        if let Some(tid) = query.trace_id {
            let in_field = ev.trace_id.as_deref() == Some(tid);
            let in_details = details_text.contains(tid);
            if !in_field && !in_details {
                continue;
            }
        }
        if let Some(rid) = query.run_id {
            let in_field = ev.run_id.as_deref() == Some(rid);
            let in_details = details_text.contains(rid);
            if !in_field && !in_details {
                continue;
            }
        }
        if let Some(eid) = query.element_id {
            if !ev.touches_element(eid) {
                continue;
            }
        }
        if query.decision_lineage_only && !ev.is_decision_lineage_kind() {
            continue;
        }
        matched.push(ev);
    }
    if matched.len() > query.limit {
        let start = matched.len() - query.limit;
        matched = matched.split_off(start);
    }
    matched.reverse();
    Ok(matched)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn record_context_compressed_writes_line() {
        let temp_dir = TempDir::new().unwrap();
        let repo = temp_dir.path();
        record_context_compressed(
            repo,
            4,
            Some(vec!["A.B".to_string()]),
            Some("host compressed"),
        );
        let raw = std::fs::read_to_string(repo.join(".sruja/context_events.jsonl")).unwrap();
        assert!(raw.contains("context_compressed"));
        assert!(raw.contains("suppress_recompress_turns"));
    }

    #[test]
    fn append_from_json_normalizes_null_details() {
        let temp_dir = TempDir::new().unwrap();
        let repo = temp_dir.path();
        let line = r#"{"schema_version":"context_event/v2","timestamp":"2026-01-01T00:00:00Z","kind":"k","outcome":"o","details":null}"#;
        append_context_event_from_json_line(repo, line).unwrap();
        let raw = std::fs::read_to_string(repo.join(".sruja/context_events.jsonl")).unwrap();
        assert!(raw.contains("\"details\":{}"));
    }

    #[test]
    fn append_from_json_rejects_non_object_details() {
        let temp_dir = TempDir::new().unwrap();
        let repo = temp_dir.path();
        let line = r#"{"schema_version":"context_event/v2","timestamp":"2026-01-01T00:00:00Z","kind":"k","outcome":"o","details":[]}"#;
        assert!(append_context_event_from_json_line(repo, line).is_err());
    }

    #[test]
    fn v1_line_deserializes_with_default_trace_fields() {
        let line = r#"{"schema_version":"context_event/v1","timestamp":"2026-01-01T00:00:00Z","kind":"drift","outcome":"pass","policy_fingerprint":null,"strict":null,"details":{"violation_count":0}}"#;
        let ev: ContextEventRecord = serde_json::from_str(line).unwrap();
        assert_eq!(ev.kind, "drift");
        assert!(ev.trace_id.is_none());
    }

    #[test]
    fn v2_round_trip() {
        let ev = ContextEventRecord {
            schema_version: CONTEXT_EVENTS_SCHEMA_V2.to_string(),
            timestamp: "2026-05-15T12:00:00Z".into(),
            kind: "context_retrieved".into(),
            outcome: "ok".into(),
            policy_fingerprint: None,
            strict: None,
            details: serde_json::json!({}),
            trace_id: Some("trace-abc".into()),
            decision_id: Some("DR-2026-001".into()),
            run_id: Some("run-123".into()),
            workflow_id: None,
            actor: Some("agent".into()),
            source: Some("mcp".into()),
            tool: Some("sruja_get_focus_briefing".into()),
            elements: Some(vec!["Sruja.Context".into()]),
            subject_ids: Some(vec![]),
            evidence_refs: Some(vec!["repo.sruja".into()]),
            summary: Some("brief".into()),
            host: None,
            skills_used: None,
            session_id: None,
        };
        validate_context_event_record(&ev).unwrap();
        let json = serde_json::to_string(&ev).unwrap();
        let back: ContextEventRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.decision_id, ev.decision_id);
        assert!(back.is_decision_lineage_kind());
    }

    #[test]
    fn agent_plan_is_decision_lineage_kind() {
        let ev = ContextEventRecord {
            schema_version: CONTEXT_EVENTS_SCHEMA_V2.to_string(),
            timestamp: "2026-05-15T12:00:00Z".into(),
            kind: "agent_plan".into(),
            outcome: "ok".into(),
            policy_fingerprint: None,
            strict: None,
            details: serde_json::json!({}),
            trace_id: Some("run-1".into()),
            decision_id: None,
            run_id: Some("run-1".into()),
            workflow_id: None,
            actor: Some("sruja agent".into()),
            source: Some("cli".into()),
            tool: Some("agent_run".into()),
            elements: Some(vec!["MySystem.Api".into()]),
            subject_ids: None,
            evidence_refs: None,
            summary: Some("plan".into()),
            host: None,
            skills_used: None,
            session_id: None,
        };
        assert!(ev.is_decision_lineage_kind());
    }

    #[test]
    fn record_agent_plan_uses_v2_schema() {
        let temp_dir = TempDir::new().unwrap();
        let repo = temp_dir.path();

        record_agent_plan(repo, "run-123", "do a thing", Some("MySystem.Api"), None);
        let events = read_context_events_query(
            repo,
            ContextEventQuery {
                limit: 5,
                kind_filter: Some("agent_plan"),
                details_substring: None,
                decision_id: None,
                trace_id: None,
                run_id: None,
                element_id: None,
                decision_lineage_only: false,
            },
        )
        .unwrap();
        assert!(events[0]
            .details
            .get("task_stream_id")
            .and_then(|v| v.as_str())
            .is_some());
        let raw = std::fs::read_to_string(repo.join(".sruja/context_events.jsonl")).unwrap();
        let line = raw.lines().last().unwrap();
        let ev: ContextEventRecord = serde_json::from_str(line).unwrap();
        assert_eq!(ev.schema_version, CONTEXT_EVENTS_SCHEMA_V2);
        assert_eq!(ev.kind, "agent_plan");
        assert_eq!(ev.trace_id.as_deref(), Some("run-123"));
    }

    #[test]
    fn touches_element_checks_vec() {
        let ev = ContextEventRecord {
            schema_version: CONTEXT_EVENTS_SCHEMA_V2.to_string(),
            timestamp: "t".into(),
            kind: "context_retrieved".into(),
            outcome: "ok".into(),
            policy_fingerprint: None,
            strict: None,
            details: serde_json::json!({}),
            trace_id: None,
            decision_id: None,
            run_id: None,
            workflow_id: None,
            actor: None,
            source: None,
            tool: None,
            elements: Some(vec!["MySystem.Api".into()]),
            subject_ids: None,
            evidence_refs: None,
            summary: None,
            host: None,
            skills_used: None,
            session_id: None,
        };
        assert!(ev.touches_element("MySystem.Api"));
        assert!(ev.touches_element("MySystem.Api.Handler"));
    }

    #[test]
    fn query_filters_by_run_id() {
        let temp_dir = TempDir::new().unwrap();
        let repo = temp_dir.path();

        append_context_event(
            repo,
            ContextEventRecord {
                schema_version: CONTEXT_EVENTS_SCHEMA_V2.to_string(),
                timestamp: "2026-05-15T12:00:00Z".into(),
                kind: "context_retrieved".into(),
                outcome: "ok".into(),
                policy_fingerprint: None,
                strict: None,
                details: serde_json::json!({ "k": "v" }),
                trace_id: None,
                decision_id: None,
                run_id: Some("run_a".into()),
                workflow_id: None,
                actor: None,
                source: None,
                tool: None,
                elements: None,
                subject_ids: None,
                evidence_refs: None,
                summary: None,
                host: None,
                skills_used: None,
                session_id: None,
            },
        );
        append_context_event(
            repo,
            ContextEventRecord {
                schema_version: CONTEXT_EVENTS_SCHEMA_V2.to_string(),
                timestamp: "2026-05-15T12:01:00Z".into(),
                kind: "context_retrieved".into(),
                outcome: "ok".into(),
                policy_fingerprint: None,
                strict: None,
                details: serde_json::json!({ "k": "v" }),
                trace_id: None,
                decision_id: None,
                run_id: Some("run_b".into()),
                workflow_id: None,
                actor: None,
                source: None,
                tool: None,
                elements: None,
                subject_ids: None,
                evidence_refs: None,
                summary: None,
                host: None,
                skills_used: None,
                session_id: None,
            },
        );

        let events = read_context_events_query(
            repo,
            ContextEventQuery {
                limit: 50,
                kind_filter: None,
                details_substring: None,
                decision_id: None,
                trace_id: None,
                run_id: Some("run_b"),
                element_id: None,
                decision_lineage_only: false,
            },
        )
        .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].run_id.as_deref(), Some("run_b"));
    }
}
