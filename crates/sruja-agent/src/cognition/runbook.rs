//! Runbooks — "if this breaks at 3AM, do X."
//!
//! Runbooks are generated alongside every change so that when production
//! breaks, the human on-call has an immediate, actionable guide.
//!
//! ## Format
//!
//! ```markdown
//! # Runbook: API Layer Refactor
//!
//! **Trigger:** API returns 500 errors after deployment
//! **Elements:** Sruja.API.Handler, Sruja.API.Router
//! **Created:** 2024-01-15
//!
//! ## Symptoms
//! - HTTP 500 on /api/v1/users
//! - Error logs show "handler not found"
//!
//! ## Diagnosis
//! 1. Check if the new handler is registered
//! 2. Verify the route table
//! 3. Check for drift
//!
//! ## Resolution
//! 1. [step-by-step fix]
//!
//! ## Rollback
//! 1. [how to undo the change]
//!
//! ## Verification
//! 1. [how to confirm it's fixed]
//! ```

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Severity of a runbook trigger.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunbookSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl std::fmt::Display for RunbookSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "Critical"),
            Self::High => write!(f, "High"),
            Self::Medium => write!(f, "Medium"),
            Self::Low => write!(f, "Low"),
        }
    }
}

/// A runbook for handling failures related to a specific change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runbook {
    pub id: String,
    pub title: String,
    pub severity: RunbookSeverity,
    pub date: String,
    /// Architecture element IDs this runbook covers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<String>,
    /// What would trigger this runbook (e.g. "API returns 500 after deploy").
    pub trigger: String,
    /// Observable symptoms.
    pub symptoms: Vec<String>,
    /// Step-by-step diagnosis procedure.
    pub diagnosis: Vec<String>,
    /// Step-by-step resolution procedure.
    pub resolution: Vec<String>,
    /// How to roll back the change.
    pub rollback: Vec<String>,
    /// How to verify the fix worked.
    pub verification: Vec<String>,
}

impl Runbook {
    /// Create a new runbook with defaults.
    pub fn new(title: impl Into<String>, trigger: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: format!("rb_{}", now.timestamp_millis()),
            title: title.into(),
            severity: RunbookSeverity::High,
            date: now.format("%Y-%m-%d").to_string(),
            elements: Vec::new(),
            trigger: trigger.into(),
            symptoms: Vec::new(),
            diagnosis: Vec::new(),
            resolution: Vec::new(),
            rollback: Vec::new(),
            verification: Vec::new(),
        }
    }

    /// Builder-style: set severity.
    pub fn with_severity(mut self, severity: RunbookSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Builder-style: set affected elements.
    pub fn with_elements(mut self, elements: Vec<String>) -> Self {
        self.elements = elements;
        self
    }

    /// Builder-style: add a symptom.
    pub fn with_symptom(mut self, symptom: impl Into<String>) -> Self {
        self.symptoms.push(symptom.into());
        self
    }

    /// Builder-style: add a diagnosis step.
    pub fn with_diagnosis_step(mut self, step: impl Into<String>) -> Self {
        self.diagnosis.push(step.into());
        self
    }

    /// Builder-style: add a resolution step.
    pub fn with_resolution_step(mut self, step: impl Into<String>) -> Self {
        self.resolution.push(step.into());
        self
    }

    /// Builder-style: add a rollback step.
    pub fn with_rollback_step(mut self, step: impl Into<String>) -> Self {
        self.rollback.push(step.into());
        self
    }

    /// Builder-style: add a verification step.
    pub fn with_verification_step(mut self, step: impl Into<String>) -> Self {
        self.verification.push(step.into());
        self
    }

    /// Render as markdown.
    pub fn to_markdown(&self) -> String {
        let mut md = format!(
            "# Runbook: {}\n\n\
             **Trigger:** {}\n\
             **Severity:** {}\n\
             **Date:** {}\n",
            self.title, self.trigger, self.severity, self.date,
        );

        if !self.elements.is_empty() {
            md.push_str(&format!("**Elements:** {}\n", self.elements.join(", ")));
        }

        if !self.symptoms.is_empty() {
            md.push_str("\n## Symptoms\n");
            for s in &self.symptoms {
                md.push_str(&format!("- {s}\n"));
            }
        }

        if !self.diagnosis.is_empty() {
            md.push_str("\n## Diagnosis\n");
            for (i, step) in self.diagnosis.iter().enumerate() {
                md.push_str(&format!("{}. {step}\n", i + 1));
            }
        }

        if !self.resolution.is_empty() {
            md.push_str("\n## Resolution\n");
            for (i, step) in self.resolution.iter().enumerate() {
                md.push_str(&format!("{}. {step}\n", i + 1));
            }
        }

        if !self.rollback.is_empty() {
            md.push_str("\n## Rollback\n");
            for (i, step) in self.rollback.iter().enumerate() {
                md.push_str(&format!("{}. {step}\n", i + 1));
            }
        }

        if !self.verification.is_empty() {
            md.push_str("\n## Verification\n");
            for (i, step) in self.verification.iter().enumerate() {
                md.push_str(&format!("{}. {step}\n", i + 1));
            }
        }

        md
    }

    /// Filename for this runbook (e.g. `rb_1234567890-api-layer-refactor.md`).
    pub fn filename(&self) -> String {
        let slug = slugify(&self.title);
        format!("{}-{}.md", self.id, slug)
    }
}

/// Render a list of runbooks as a single markdown document.
pub fn render_runbooks(runbooks: &[Runbook]) -> String {
    runbooks
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
    fn runbook_markdown() {
        let rb = Runbook::new("API Layer Refactor", "API returns 500 after deploy")
            .with_severity(RunbookSeverity::Critical)
            .with_symptom("HTTP 500 on /api/v1/users")
            .with_diagnosis_step("Check if the new handler is registered")
            .with_resolution_step("Revert to the previous handler")
            .with_rollback_step("git revert HEAD")
            .with_verification_step("Run cargo test");

        let md = rb.to_markdown();
        assert!(md.contains("# Runbook: API Layer Refactor"));
        assert!(md.contains("**Severity:** Critical"));
        assert!(md.contains("HTTP 500 on /api/v1/users"));
        assert!(md.contains("git revert HEAD"));
    }

    #[test]
    fn filename_is_safe() {
        let rb = Runbook::new("Add Caching!", "");
        let filename = rb.filename();
        assert!(!filename.contains(' '));
        assert!(!filename.contains('!'));
        assert!(filename.ends_with(".md"));
    }
}
