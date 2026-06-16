//! Decision records — the "WHY" behind every change.
//!
//! When a human Debugs a production issue at 3AM, they need to know WHY
//! the AI made a particular decision. Decision records persist this reasoning
//! alongside the code.
//!
//! ## Format
//!
//! Decision records are markdown files stored in `.sruja/decisions/`:
//!
//! ```markdown
//! # Decision: Refactor API Layer
//!
//! **Status:** Accepted
//! **Date:** 2024-01-15
//! **Agent Run:** run_12345
//! **Elements:** Sruja.API.Handler, Sruja.API.Router
//!
//! ## Context
//! The API layer was accumulating technical debt...
//!
//! ## Decision
//! Refactor to use the new handler pattern...
//!
//! ## Consequences
//! - Positive: Cleaner separation of concerns
//! - Negative: Temporary increase in file count
//! - Risk: May affect existing integrations
//!
//! ## Alternatives Considered
//! 1. Keep monolithic handler — rejected because...
//! 2. Split into microservices — rejected because...
//! ```

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Status of a decision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    Proposed,
    Accepted,
    Deprecated,
    Superseded,
    Rejected,
}

impl std::fmt::Display for DecisionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Proposed => write!(f, "Proposed"),
            Self::Accepted => write!(f, "Accepted"),
            Self::Deprecated => write!(f, "Deprecated"),
            Self::Superseded => write!(f, "Superseded"),
            Self::Rejected => write!(f, "Rejected"),
        }
    }
}

/// A decision record explaining WHY a change was made.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub id: String,
    pub title: String,
    pub status: DecisionStatus,
    pub date: String,
    pub run_id: Option<String>,
    /// Architecture element IDs this decision affects.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<String>,
    /// The situation that required a decision.
    pub context: String,
    /// What was decided.
    pub decision: String,
    /// What follows from this decision.
    pub consequences: Vec<String>,
    /// Other options that were considered and why they were rejected.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternatives: Vec<String>,
}

impl DecisionRecord {
    /// Create a new decision record with defaults.
    pub fn new(
        title: impl Into<String>,
        context: impl Into<String>,
        decision: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: format!("dec_{}", now.timestamp_millis()),
            title: title.into(),
            status: DecisionStatus::Proposed,
            date: now.format("%Y-%m-%d").to_string(),
            run_id: None,
            elements: Vec::new(),
            context: context.into(),
            decision: decision.into(),
            consequences: Vec::new(),
            alternatives: Vec::new(),
        }
    }

    /// Builder-style: set the status.
    pub fn with_status(mut self, status: DecisionStatus) -> Self {
        self.status = status;
        self
    }

    /// Builder-style: set the run ID.
    pub fn with_run_id(mut self, run_id: impl Into<String>) -> Self {
        self.run_id = Some(run_id.into());
        self
    }

    /// Builder-style: set affected elements.
    pub fn with_elements(mut self, elements: Vec<String>) -> Self {
        self.elements = elements;
        self
    }

    /// Builder-style: add a consequence.
    pub fn with_consequence(mut self, consequence: impl Into<String>) -> Self {
        self.consequences.push(consequence.into());
        self
    }

    /// Builder-style: add an alternative.
    pub fn with_alternative(mut self, alternative: impl Into<String>) -> Self {
        self.alternatives.push(alternative.into());
        self
    }

    /// Render as markdown.
    pub fn to_markdown(&self) -> String {
        let mut md = format!(
            "# Decision: {}\n\n\
             **Status:** {}\n\
             **Date:** {}\n",
            self.title, self.status, self.date,
        );

        if let Some(run_id) = &self.run_id {
            md.push_str(&format!("**Agent Run:** {run_id}\n"));
        }

        if !self.elements.is_empty() {
            md.push_str(&format!("**Elements:** {}\n", self.elements.join(", ")));
        }

        md.push_str(&format!(
            "\n## Context\n{}\n\n\
             ## Decision\n{}\n",
            self.context, self.decision,
        ));

        if !self.consequences.is_empty() {
            md.push_str("## Consequences\n");
            for c in &self.consequences {
                md.push_str(&format!("- {c}\n"));
            }
        }

        if !self.alternatives.is_empty() {
            md.push_str("\n## Alternatives Considered\n");
            for (i, a) in self.alternatives.iter().enumerate() {
                md.push_str(&format!("{}. {a}\n", i + 1));
            }
        }

        md
    }

    /// Filename for this decision record (e.g. `dec_1234567890-refactor-api-layer.md`).
    pub fn filename(&self) -> String {
        let slug = slugify(&self.title);
        format!("{}-{}.md", self.id, slug)
    }
}

/// Render a list of decision records as a single markdown document.
pub fn render_decisions(records: &[DecisionRecord]) -> String {
    records
        .iter()
        .map(|r| r.to_markdown())
        .collect::<Vec<_>>()
        .join("\n---\n\n")
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
        .replace(' ', "-")
        .chars()
        .take(50)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_record_markdown() {
        let record = DecisionRecord::new(
            "Refactor API Layer",
            "The API layer was accumulating tech debt",
            "Refactor to use the new handler pattern",
        )
        .with_status(DecisionStatus::Accepted)
        .with_elements(vec!["Sruja.API.Handler".into()])
        .with_consequence("Cleaner separation of concerns")
        .with_alternative("Keep monolithic handler — rejected because tech debt");

        let md = record.to_markdown();
        assert!(md.contains("# Decision: Refactor API Layer"));
        assert!(md.contains("**Status:** Accepted"));
        assert!(md.contains("Sruja.API.Handler"));
        assert!(md.contains("Cleaner separation of concerns"));
    }

    #[test]
    fn filename_is_safe() {
        let record = DecisionRecord::new("Add Caching Layer!", "", "");
        let filename = record.filename();
        assert!(!filename.contains(' '));
        assert!(!filename.contains('!'));
        assert!(filename.ends_with(".md"));
    }
}
