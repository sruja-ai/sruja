//! Structural checks for context event JSON (aligns with `schemas/context_event_record.schema.json`).

use serde_json::Value;

fn assert_context_event_shape(v: &Value) {
    for key in ["schema_version", "timestamp", "kind", "outcome"] {
        assert!(
            v.get(key)
                .and_then(|x| x.as_str())
                .is_some_and(|s| !s.is_empty()),
            "missing or empty {key}: {v}"
        );
    }
    let sv = v.get("schema_version").and_then(|x| x.as_str()).unwrap();
    assert!(
        sv == "context_event/v1" || sv == "context_event/v2",
        "schema_version: {sv}"
    );
}

#[test]
fn v1_intent_shape() {
    let v: Value = serde_json::from_str(
        r#"{
        "schema_version": "context_event/v1",
        "timestamp": "2026-01-01T00:00:00Z",
        "kind": "intent_check",
        "outcome": "pass",
        "policy_fingerprint": null,
        "strict": true,
        "details": {"drift_score": 80}
    }"#,
    )
    .unwrap();
    assert_context_event_shape(&v);
}

#[test]
fn v2_decision_trace_shape() {
    let v: Value = serde_json::from_str(
        r#"{
        "schema_version": "context_event/v2",
        "timestamp": "2026-05-15T12:00:00Z",
        "kind": "context_retrieved",
        "outcome": "ok",
        "details": {},
        "trace_id": "trace-abc",
        "decision_id": "DR-2026-001",
        "run_id": "run-123",
        "actor": "agent",
        "source": "mcp",
        "tool": "sruja_get_focus_briefing",
        "elements": ["Sruja.Context"],
        "evidence_refs": ["repo.sruja"],
        "summary": "Retrieved briefing"
    }"#,
    )
    .unwrap();
    assert_context_event_shape(&v);
}
