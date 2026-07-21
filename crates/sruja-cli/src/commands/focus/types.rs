use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct MemoryHit {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hitl_kind: Option<String>,
    pub outcome: String,
    pub match_reason: String,
    pub timestamp: String,
    pub hypothesis: String,
    pub guardrail_advice: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TemporalContextBrief {
    pub base_ref: String,
    pub head_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture_relative_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_fingerprint_head: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_fingerprint_base: Option<String>,
    pub diff_mapped_component_ids: Vec<String>,
    pub touches_focus_target: bool,
}

#[derive(Debug, Serialize)]
pub struct FocusBriefing {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_drift_violations: Vec<sruja_diff::Violation>,
    pub anti_patterns: Vec<String>,
    pub boundaries: Vec<BoundaryInfo>,
    pub ai_instructions: Vec<String>,
    pub target: FocusTarget,
    pub blast_radius: BlastRadius,
    pub reasoned_traces: Vec<ReasonedTrace>,
    pub decisions: Vec<LinkedDecision>,
    pub external_context: Vec<ExternalContextRef>,
    pub hotspot_status: HotspotStatus,
    pub pointer_traces: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub memory_hits: Vec<MemoryHit>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub memory_truncated: bool,
    pub context_score: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporal: Option<TemporalContextBrief>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrichment: Option<crate::integrations::EnrichmentResult>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decision_trace_events: Vec<crate::commands::context_events::ContextEventRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decision_records: Vec<crate::commands::decision::DecisionListItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub linked_requirements: Vec<LinkedRequirementSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub surfaced_learning_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_session: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ask_plan: Option<sruja_agent::AskPlan>,
}

#[derive(Debug, Clone)]
pub struct SurfacedLearnings {
    pub hits: Vec<MemoryHit>,
    pub ids: Vec<String>,
    pub truncated: bool,
    pub anti_patterns: Vec<String>,
    pub pointer_traces: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ReasonedTrace {
    pub node_id: String,
    pub node_label: String,
    pub direction: String,
    pub reasoning: String,
    pub decision_ref: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FocusForAiOutput {
    pub artifact_kind: String,
    pub schema_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    pub repo: String,
    pub target: FocusForAiTarget,
    pub briefing: FocusBriefing,
    pub suggested_next_steps: Vec<SuggestedCommand>,
}

#[derive(Debug, Serialize)]
pub struct FocusForAiTarget {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,
    pub resolved_element_id: String,
}

#[derive(Debug, Serialize)]
pub struct SuggestedCommand {
    pub purpose: String,
    pub argv: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FocusTarget {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub technology: Option<String>,
    pub system: Option<String>,
    pub gotchas: Vec<String>,
    pub operational_constraints: Vec<String>,
    pub runbooks: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BlastRadius {
    pub total_affected: usize,
    pub upstream: Vec<AffectedNode>,
    pub downstream: Vec<AffectedNode>,
}

#[derive(Debug, Serialize)]
pub struct AffectedNode {
    pub id: String,
    pub label: String,
    pub depth: usize,
    pub relationship: String,
}

#[derive(Debug, Serialize)]
pub struct LinkedDecision {
    pub id: String,
    pub title: String,
    pub status: String,
    pub summary: String,
}

#[derive(Debug, Serialize)]
pub struct LinkedRequirementSummary {
    pub id: String,
    pub title: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub acceptance_criteria: Vec<AcceptanceCriteriaSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub affects: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub adrs: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scenarios: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AcceptanceCriteriaSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub then: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BoundaryInfo {
    pub from: String,
    pub to: String,
    pub allowed: bool,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct ExternalContextRef {
    pub file: String,
    pub category: String,
    pub excerpt: String,
}

#[derive(Debug, Serialize)]
pub struct HotspotStatus {
    pub is_hotspot: bool,
    pub role: String,
    pub in_degree: usize,
    pub out_degree: usize,
}

pub(super) fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        ".".repeat(max_len)
    } else {
        let keep = max_len.saturating_sub(3);
        let prefix: String = s.chars().take(keep).collect();
        format!("{}...", prefix)
    }
}
