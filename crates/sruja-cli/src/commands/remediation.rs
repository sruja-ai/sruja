//! Deterministic remediation playbooks (80% layer): map findings to safe `sruja` steps.
//!
//! LLM enrichment may suggest narrative; these steps are always reproducible from facts.

use serde_json::Value;

const MAX_STEPS: usize = 8;

#[derive(Debug, Clone)]
pub struct PlaybookStep {
    pub id: String,
    pub kind: String,
    pub argv: Vec<String>,
    pub expected: Option<String>,
}

/// Build architecture-bounded next steps from drift/intent JSON facts.
pub fn plan_remediation_steps(drift_json: &Value, intent_json: &Value) -> Vec<PlaybookStep> {
    let mut steps = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for v in collect_violation_records(drift_json) {
        push_unique(&mut steps, &mut seen_ids, step_for_violation(&v));
    }
    for v in collect_violation_records(intent_json) {
        push_unique(&mut steps, &mut seen_ids, step_for_intent_violation(&v));
    }

    if steps.is_empty() {
        push_unique(
            &mut steps,
            &mut seen_ids,
            PlaybookStep {
                id: "step_sync_truth".to_string(),
                kind: "sruja_cmd".to_string(),
                argv: vec![
                    "sruja".to_string(),
                    "sync".to_string(),
                    "-r".to_string(),
                    ".".to_string(),
                ],
                expected: Some("Refresh scan evidence and graph artifacts".to_string()),
            },
        );
    }

    steps.truncate(MAX_STEPS);
    steps
}

#[derive(Debug, Clone)]
struct ViolationRecord {
    kind: String,
    rule_id: Option<String>,
    severity: String,
}

fn collect_violation_records(json: &Value) -> Vec<ViolationRecord> {
    let Some(arr) = json.get("violations").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    arr.iter()
        .map(|item| {
            let kind = item
                .get("kind")
                .and_then(|k| k.as_str())
                .map(str::to_string)
                .or_else(|| {
                    item.get("kind")
                        .and_then(|k| k.as_str().map(|s| s.to_string()))
                })
                .unwrap_or_else(|| "unknown".to_string());
            let severity = item
                .get("severity")
                .and_then(|s| s.as_str())
                .map(str::to_string)
                .unwrap_or_else(|| "unknown".to_string());
            let rule_id = item
                .get("rule_id")
                .and_then(|r| r.as_str())
                .map(str::to_string);
            ViolationRecord {
                kind,
                rule_id,
                severity,
            }
        })
        .collect()
}

fn push_unique(
    steps: &mut Vec<PlaybookStep>,
    seen: &mut std::collections::HashSet<String>,
    step: PlaybookStep,
) {
    if seen.insert(step.id.clone()) {
        steps.push(step);
    }
}

fn step_for_violation(v: &ViolationRecord) -> PlaybookStep {
    let kind_norm = v.kind.to_lowercase().replace(' ', "");
    let rule = v.rule_id.as_deref().unwrap_or("");

    if kind_norm.contains("undocumented") || rule.contains("DOC") {
        return PlaybookStep {
            id: "step_propose_baseline".to_string(),
            kind: "sruja_cmd".to_string(),
            argv: vec![
                "sruja".to_string(),
                "propose".to_string(),
                "create".to_string(),
                "-r".to_string(),
                ".".to_string(),
                "--title".to_string(),
                "Align repo.sruja with discovered components".to_string(),
            ],
            expected: Some(
                "Create a reviewable proposal for undocumented architecture".to_string(),
            ),
        };
    }

    if kind_norm.contains("layer") || kind_norm.contains("boundary") || rule.contains("LAYER") {
        return PlaybookStep {
            id: "step_review_boundaries".to_string(),
            kind: "sruja_cmd".to_string(),
            argv: vec![
                "sruja".to_string(),
                "review".to_string(),
                "-r".to_string(),
                ".".to_string(),
                "-f".to_string(),
                "json".to_string(),
            ],
            expected: Some("Capture boundary/layer remediation actions".to_string()),
        };
    }

    if kind_norm.contains("circular") || kind_norm.contains("cycle") || rule.contains("CYCLE") {
        return PlaybookStep {
            id: "step_impact_cycle".to_string(),
            kind: "sruja_cmd".to_string(),
            argv: vec![
                "sruja".to_string(),
                "impact".to_string(),
                "-r".to_string(),
                ".".to_string(),
                "-f".to_string(),
                "json".to_string(),
            ],
            expected: Some("Assess blast radius before breaking dependency cycles".to_string()),
        };
    }

    if kind_norm.contains("unproposed") || rule.contains("PROPOSAL") {
        return PlaybookStep {
            id: "step_propose_change".to_string(),
            kind: "sruja_cmd".to_string(),
            argv: vec![
                "sruja".to_string(),
                "propose".to_string(),
                "create".to_string(),
                "-r".to_string(),
                ".".to_string(),
            ],
            expected: Some("Record architectural change via proposal workflow".to_string()),
        };
    }

    if kind_norm.contains("policy") || rule.contains("POLICY") {
        return PlaybookStep {
            id: "step_compliance".to_string(),
            kind: "sruja_cmd".to_string(),
            argv: vec![
                "sruja".to_string(),
                "compliance".to_string(),
                "-r".to_string(),
                ".".to_string(),
                "-f".to_string(),
                "json".to_string(),
            ],
            expected: Some("List policy violations with remediation hints".to_string()),
        };
    }

    PlaybookStep {
        id: format!(
            "step_drift_kind_{}",
            kind_norm.chars().take(24).collect::<String>()
        ),
        kind: "sruja_cmd".to_string(),
        argv: vec![
            "sruja".to_string(),
            "drift".to_string(),
            "-r".to_string(),
            ".".to_string(),
            "-f".to_string(),
            "json".to_string(),
        ],
        expected: Some(format!(
            "Re-inspect drift for kind={} severity={}",
            v.kind, v.severity
        )),
    }
}

fn step_for_intent_violation(v: &ViolationRecord) -> PlaybookStep {
    let mut step = step_for_violation(v);
    let is_generic_fallback = step.id.starts_with("step_drift_kind_");
    step.id = format!("step_intent_{}", step.id);

    if is_generic_fallback {
        step.argv = vec![
            "sruja".to_string(),
            "intent".to_string(),
            "check".to_string(),
            "-r".to_string(),
            ".".to_string(),
            "-f".to_string(),
            "json".to_string(),
        ];
        step.expected = Some("Intent vs reality report for architectural alignment".to_string());
    } else {
        step.expected = Some(format!(
            "Intent check for: {}",
            step.expected.unwrap_or_default()
        ));
    }
    step
}

/// Attach standard JSON metadata for deterministic CLI/MCP outputs.
pub fn wrap_deterministic_json(mut value: Value, metric_type: &str, description: &str) -> Value {
    if let Value::Object(ref mut map) = value {
        map.insert(
            "artifact_kind".to_string(),
            Value::String("deterministic_fact".to_string()),
        );
        map.insert(
            "metric_type".to_string(),
            Value::String(metric_type.to_string()),
        );
        map.insert(
            "metric_description".to_string(),
            Value::String(description.to_string()),
        );
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposes_propose_step_for_undocumented() {
        let drift = serde_json::json!({
            "violations": [{
                "kind": "UndocumentedComponent",
                "severity": "Warning",
                "rule_id": "SRUJA-DOC-001"
            }]
        });
        let steps = plan_remediation_steps(&drift, &Value::Null);
        assert!(steps.iter().any(|s| s.id == "step_propose_baseline"));
    }

    #[test]
    fn wrap_adds_artifact_kind() {
        let v = wrap_deterministic_json(serde_json::json!({"score": 1}), "drift", "desc");
        assert_eq!(
            v.get("artifact_kind").and_then(|x| x.as_str()),
            Some("deterministic_fact")
        );
    }

    #[test]
    fn intent_violation_preserves_specific_step_argv() {
        let intent = serde_json::json!({
            "violations": [{
                "kind": "UndocumentedComponent",
                "severity": "Warning",
                "rule_id": "SRUJA-DOC-001"
            }]
        });
        let steps = plan_remediation_steps(&Value::Null, &intent);
        let propose = steps
            .iter()
            .find(|s| s.id == "step_intent_step_propose_baseline");
        assert!(
            propose.is_some(),
            "should have a propose step, got: {:?}",
            steps.iter().map(|s| &s.id).collect::<Vec<_>>()
        );
        let propose = propose.unwrap();
        assert!(
            propose.argv.contains(&"propose".to_string()),
            "propose step should preserve sruja propose argv, got: {:?}",
            propose.argv
        );
        assert!(propose.id.starts_with("step_intent_"));
    }

    #[test]
    fn intent_violation_generic_fallback_uses_intent_check() {
        let intent = serde_json::json!({
            "violations": [{
                "kind": "SomethingUnknown",
                "severity": "Warning"
            }]
        });
        let steps = plan_remediation_steps(&Value::Null, &intent);
        assert_eq!(steps.len(), 1);
        assert!(steps[0].id.starts_with("step_intent_step_drift_kind_"));
        assert!(steps[0].argv.contains(&"intent".to_string()));
        assert!(steps[0].argv.contains(&"check".to_string()));
    }
}
