//! Append-only **context lineage** events under `.sruja/context_events.jsonl`.
//!
//! These records approximate “decision traces” for software architecture: intent checks,
//! drift runs, and merged proposals, each stamped with a fingerprint of the declared
//! architecture file in use at the time (when resolvable).

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sruja_intent::{DriftReport, Severity};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub const CONTEXT_EVENTS_SCHEMA: &str = "context_event/v1";

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
    pub details: serde_json::Value,
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
        ContextEventRecord {
            schema_version: CONTEXT_EVENTS_SCHEMA.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            kind: "intent_check".to_string(),
            outcome: outcome.to_string(),
            policy_fingerprint: policy_fingerprint(repo),
            strict: Some(strict),
            details,
        },
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
        ContextEventRecord {
            schema_version: CONTEXT_EVENTS_SCHEMA.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            kind: "drift".to_string(),
            outcome: outcome.to_string(),
            policy_fingerprint: policy_fingerprint(repo),
            strict: None,
            details: serde_json::json!({
                "violation_count": violation_count,
                "truth_status": truth_status,
                "compared_to_architecture": compared_to_architecture,
            }),
        },
    );
}

pub fn record_proposal_merge(repo: &Path, proposal_id: &str) {
    append_context_event(
        repo,
        ContextEventRecord {
            schema_version: CONTEXT_EVENTS_SCHEMA.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            kind: "proposal_merge".to_string(),
            outcome: "ok".to_string(),
            policy_fingerprint: policy_fingerprint(repo),
            strict: None,
            details: serde_json::json!({ "proposal_id": proposal_id }),
        },
    );
}

/// Returns up to `limit` matching events, **newest first** (tail of the log).
pub fn read_context_events(
    repo: &Path,
    limit: usize,
    kind_filter: Option<&str>,
    details_substring: Option<&str>,
) -> std::io::Result<Vec<ContextEventRecord>> {
    let path = events_path(repo);
    if !path.exists() || limit == 0 {
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
        if let Some(k) = kind_filter {
            if ev.kind != k {
                continue;
            }
        }
        if let Some(sub) = details_substring {
            if !ev.details.to_string().contains(sub) {
                continue;
            }
        }
        matched.push(ev);
    }
    if matched.len() > limit {
        let start = matched.len() - limit;
        matched = matched.split_off(start);
    }
    matched.reverse();
    Ok(matched)
}
