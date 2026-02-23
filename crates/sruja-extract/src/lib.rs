//! Architecture Extraction Engine
//!
//! This crate extracts structured architecture knowledge from conversations
//! using LLM-based extraction. No pattern matching — relies on LLM only.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sruja_graph::*;
use thiserror::Error;

pub mod intent;
pub mod llm_extract;
pub mod ratification;

pub use intent::Intent;
pub use llm_extract::extract_from_message_async;
pub use ratification::RatificationStatus;

#[derive(Debug, Error)]
pub enum ExtractError {
    #[error("Failed to parse message: {0}")]
    Parse(String),

    #[error("Invalid extraction: {0}")]
    Invalid(String),

    #[error("LLM extraction failed: {0}")]
    Llm(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub id: MessageId,
    pub session_id: SessionId,
    pub author: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Extraction {
    pub id: String,
    pub intent: Intent,
    pub confidence: f32,
    pub content: ExtractedContent,
    pub source_message_ids: Vec<MessageId>,
    /// If set, this extraction came from a thread reply; the main channel surfaces it under this root message.
    #[serde(default)]
    pub thread_root_message_id: Option<MessageId>,
    pub status: ExtractionStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtractionStatus {
    Draft,
    Confirmed,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtractedContent {
    Decision {
        title: String,
        context: String,
        decision: String,
        alternatives: Vec<String>,
        consequences: Vec<String>,
    },
    Requirement {
        title: String,
        description: String,
        priority: RequirementPriority,
    },
    Constraint {
        source: String,
        target: String,
        constraint_type: ConstraintType,
        description: String,
    },
    Policy {
        name: String,
        description: String,
        rules: Vec<String>,
    },
    Risk {
        description: String,
        severity: RiskSeverity,
        mitigation: Option<String>,
    },
    Component {
        name: String,
        kind: NodeKind,
        technology: Option<String>,
        description: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintType {
    CannotCall,
    MustUse,
    MustNotUse,
    Requires,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Extraction engine. Uses LLM only — no pattern matching.
#[derive(Clone)]
pub struct ExtractionEngine {
    ratification_keywords: Vec<String>,
}

impl Default for ExtractionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtractionEngine {
    pub fn new() -> Self {
        let ratification_keywords = vec![
            "+1".to_string(),
            "agreed".to_string(),
            "agree".to_string(),
            "sounds good".to_string(),
            "let's do it".to_string(),
            "approved".to_string(),
            "lgtm".to_string(),
            "ship it".to_string(),
            "done".to_string(),
            ":+1:".to_string(),
            "👍".to_string(),
        ];

        Self {
            ratification_keywords,
        }
    }

    /// Extract architecture intents from a message using LLM. This is the only extraction path.
    pub async fn extract_from_message_async(
        &self,
        message: &ConversationMessage,
    ) -> Result<Vec<Extraction>, ExtractError> {
        llm_extract::extract_from_message_async(self, message)
            .await
            .map_err(ExtractError::Llm)
    }

    pub fn extract_from_conversation_sync(
        &self,
        messages: &[ConversationMessage],
        extractions: &mut Vec<Extraction>,
    ) {
        self.update_ratification_status(extractions, messages);
    }

    fn update_ratification_status(
        &self,
        extractions: &mut [Extraction],
        messages: &[ConversationMessage],
    ) {
        let ratification_messages: Vec<&ConversationMessage> = messages
            .iter()
            .filter(|m| self.is_ratification(&m.content))
            .collect();

        for extraction in extractions.iter_mut() {
            if self.is_ratified(extraction, &ratification_messages) {
                extraction.status = ExtractionStatus::Confirmed;
            }
        }
    }

    fn is_ratification(&self, content: &str) -> bool {
        let content_lower = content.to_lowercase();
        self.ratification_keywords
            .iter()
            .any(|kw| content_lower.contains(kw))
    }

    fn is_ratified(
        &self,
        extraction: &Extraction,
        ratification_messages: &[&ConversationMessage],
    ) -> bool {
        if ratification_messages.is_empty() {
            return false;
        }
        let extraction_time = extraction.created_at;
        ratification_messages
            .iter()
            .any(|m| m.timestamp > extraction_time)
    }

    pub fn check_ratification(
        &self,
        extraction: &Extraction,
        messages: &[ConversationMessage],
    ) -> RatificationStatus {
        let ratifiers: Vec<&str> = messages
            .iter()
            .filter(|m| self.is_ratification(&m.content))
            .filter(|m| m.timestamp > extraction.created_at)
            .map(|m| m.author.as_str())
            .collect();

        if ratifiers.is_empty() {
            RatificationStatus::Pending
        } else {
            RatificationStatus::Ratified {
                by: ratifiers.iter().map(|s| s.to_string()).collect(),
            }
        }
    }
}

impl Extraction {
    pub fn to_decision(&self, session_id: &SessionId) -> Option<Decision> {
        match &self.content {
            ExtractedContent::Decision {
                title,
                context,
                decision,
                alternatives,
                consequences,
            } => Some(Decision {
                id: generate_id(),
                title: title.clone(),
                status: if self.status == ExtractionStatus::Confirmed {
                    DecisionStatus::Accepted
                } else {
                    DecisionStatus::Proposed
                },
                context: context.clone(),
                decision: decision.clone(),
                consequences: consequences.join("\n"),
                alternatives: alternatives.clone(),
                created_at: self.created_at,
                updated_at: Utc::now(),
                ratified_at: if self.status == ExtractionStatus::Confirmed {
                    Some(Utc::now())
                } else {
                    None
                },
                author: None,
                source: SourceReference::conversation(session_id, self.source_message_ids.clone()),
                affects: vec![],
            }),
            _ => None,
        }
    }

    pub fn to_adr_markdown(&self, adr_number: Option<u32>) -> Option<String> {
        match &self.content {
            ExtractedContent::Decision {
                title,
                context,
                decision,
                alternatives,
                consequences,
            } => {
                let num = adr_number
                    .map(|n| format!("{n:04}"))
                    .unwrap_or_else(|| "0000".to_string());
                let status = match self.status {
                    ExtractionStatus::Confirmed => "Accepted",
                    ExtractionStatus::Draft => "Proposed",
                    ExtractionStatus::Rejected => "Rejected",
                };
                let mut md = format!(
                    "# ADR-{}: {}\n\n## Status\n{}\n\n## Context\n{}\n\n## Decision\n{}\n\n",
                    num, title, status, context, decision
                );
                if !alternatives.is_empty() {
                    md.push_str("## Alternatives Considered\n\n");
                    for alt in alternatives {
                        md.push_str(&format!("- {}\n", alt));
                    }
                    md.push('\n');
                }
                if !consequences.is_empty() {
                    md.push_str("## Consequences\n\n");
                    for c in consequences {
                        md.push_str(&format!("- {}\n", c));
                    }
                }
                Some(md)
            }
            _ => None,
        }
    }

    pub fn to_node(&self, session_id: &SessionId) -> Option<ArchitectureNode> {
        match &self.content {
            ExtractedContent::Component {
                name,
                kind,
                technology,
                description,
            } => Some(ArchitectureNode {
                id: name.to_lowercase().replace(' ', "-"),
                kind: *kind,
                label: name.clone(),
                technology: technology.clone(),
                description: description.clone(),
                metadata: std::collections::HashMap::new(),
                source: SourceReference::conversation(session_id, self.source_message_ids.clone()),
                created_at: self.created_at,
                updated_at: Utc::now(),
            }),
            _ => None,
        }
    }
}
