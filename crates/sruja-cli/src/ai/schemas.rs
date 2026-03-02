//! JSON schemas for memory store: interactions, feedback, evidence.
//! Matches ARCHITECTURE_EXPLAINER_MEMORY_IMPLEMENTATION_PLAN.md.

use serde::{Deserialize, Serialize};

/// Evidence entry for a fact (file path + optional line + why).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEntry {
    pub kind: String, // "file"
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_hint: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub why_relevant: Option<String>,
}

/// One interaction (one answer) in interactions.jsonl.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InteractionRecord {
    pub answer_id: String,
    pub question: String,
    pub response_markdown: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub used_fact_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub new_fact_ids: Vec<String>,
    pub confidence: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
    pub created_at: String,
}

/// One feedback record in feedback.jsonl.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeedbackRecord {
    pub feedback_id: String,
    pub answer_id: String,
    pub fact_id: String,
    pub verdict: String, // correct|wrong|partial
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub actor: String, // "user"
    pub created_at: String,
}
