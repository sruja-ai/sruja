//! Intent classification for extracted content

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    Decision,
    Requirement,
    Constraint,
    Policy,
    Risk,
    Question,
    Tradeoff,
    ComponentMention,
    Agreement,
    Disagreement,
    Clarification,
}

impl std::fmt::Display for Intent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Intent::Decision => write!(f, "decision"),
            Intent::Requirement => write!(f, "requirement"),
            Intent::Constraint => write!(f, "constraint"),
            Intent::Policy => write!(f, "policy"),
            Intent::Risk => write!(f, "risk"),
            Intent::Question => write!(f, "question"),
            Intent::Tradeoff => write!(f, "tradeoff"),
            Intent::ComponentMention => write!(f, "component"),
            Intent::Agreement => write!(f, "agreement"),
            Intent::Disagreement => write!(f, "disagreement"),
            Intent::Clarification => write!(f, "clarification"),
        }
    }
}
