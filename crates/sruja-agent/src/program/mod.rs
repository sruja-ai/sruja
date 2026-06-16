//! Declarative agent programs.
//!
//! Programs are YAML/TOML-specifiable agent behaviors — reusable playbooks
//! that define the system prompt, tool allowlist, model tiers, hooks, and
//! phase configuration for a specific task type.
//!
//! (Phase 3 — stub for now; the cognition loop and Agent API are the primary
//! interface until program loading is implemented.)

use serde::{Deserialize, Serialize};

use crate::cognition::{AgentConfig, ModelMapping};
use crate::tool::TestPathClassifier;

/// A declarative agent program spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub config: ProgramConfig,
    /// System prompt override for this program.
    pub system_prompt: Option<String>,
    /// Tool allowlist (None = all registered tools).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProgramConfig {
    #[serde(default)]
    pub tdd: Option<bool>,
    #[serde(default)]
    pub review_every_change: Option<bool>,
    #[serde(default)]
    pub spend_cap_usd: Option<f64>,
    #[serde(default)]
    pub models: Option<ModelMapping>,
    #[serde(default)]
    pub test_patterns: Option<Vec<String>>,
}

impl Program {
    /// Merge this program's config into a base [`AgentConfig`].
    pub fn apply_to(&self, mut config: AgentConfig) -> AgentConfig {
        if let Some(tdd) = self.config.tdd {
            config.tdd = tdd;
        }
        if let Some(review) = self.config.review_every_change {
            config.review_every_change = review;
        }
        if let Some(cap) = self.config.spend_cap_usd {
            config.spend_cap_usd = Some(cap);
        }
        if let Some(models) = self.config.models.clone() {
            config.models = models;
        }
        config
    }

    /// Build a test-path classifier from this program's patterns.
    pub fn classifier(&self) -> TestPathClassifier {
        match &self.config.test_patterns {
            Some(patterns) => TestPathClassifier::new(patterns.clone()),
            None => TestPathClassifier::default(),
        }
    }
}

impl Default for Program {
    fn default() -> Self {
        Self {
            name: "default".into(),
            description: "Default agent program".into(),
            config: ProgramConfig::default(),
            system_prompt: None,
            tools: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_overrides_config() {
        let program = Program {
            name: "test".into(),
            description: "test".into(),
            config: ProgramConfig {
                tdd: Some(false),
                ..Default::default()
            },
            system_prompt: None,
            tools: None,
        };
        let config = program.apply_to(AgentConfig::default());
        assert!(!config.tdd);
    }
}
