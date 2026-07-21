use crate::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("No results found")]
    NoResults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub question: String,
    pub answer: String,
    pub evidence: Vec<Evidence>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonedWhyResult {
    pub question: String,
    pub target_id: String,
    pub target_label: String,
    pub steps: Vec<ReasonedWhyStep>,
    pub final_answer: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonedWhyStep {
    pub step_index: usize,
    pub node_id: String,
    pub node_label: String,
    pub direction: String,
    pub relationship: String,
    pub reasoning: String,
    pub decision_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmGuidedWhyResult {
    pub question: String,
    pub target_id: String,
    pub target_label: String,
    pub steps: Vec<LlmGuidedWhyStep>,
    pub summary: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmGuidedWhyStep {
    pub step_index: usize,
    pub node_id: String,
    pub node_label: String,
    pub direction: String,
    pub relationship: String,
    pub relevance_score: String,
    pub llm_reasoning: String,
    pub decision_ref: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub kind: EvidenceKind,
    pub reference: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceKind {
    Decision,
    Policy,
    Requirement,
    Node,
    Edge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub policy_id: String,
    pub policy_name: String,
    pub edge_id: String,
    pub source: String,
    pub target: String,
    pub message: String,
    pub severity: PolicySeverity,
}
