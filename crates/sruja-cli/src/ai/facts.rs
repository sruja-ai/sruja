//! Fact type and confidence update rules (deterministic feedback application).

use serde::{Deserialize, Serialize};

use super::schemas::EvidenceEntry;

/// Fact status in the memory loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactStatus {
    Candidate,
    Confirmed,
    Disputed,
    Deprecated,
}

impl std::fmt::Display for FactStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FactStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            FactStatus::Candidate => "candidate",
            FactStatus::Confirmed => "confirmed",
            FactStatus::Disputed => "disputed",
            FactStatus::Deprecated => "deprecated",
        }
    }
}

/// Fact type for categorization. Part of public API for fact schema.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactType {
    Flow,
    Boundary,
    Dependency,
    Decision,
    Risk,
    Ownership,
}

impl std::fmt::Display for FactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FactType::Flow => write!(f, "flow"),
            FactType::Boundary => write!(f, "boundary"),
            FactType::Dependency => write!(f, "dependency"),
            FactType::Decision => write!(f, "decision"),
            FactType::Risk => write!(f, "risk"),
            FactType::Ownership => write!(f, "ownership"),
        }
    }
}

/// In-memory fact used by the AI module (matches plan schema).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub fact_id: String,
    pub statement: String,
    #[serde(rename = "fact_type")]
    pub kind: String, // flow|boundary|... (string for JSONL flexibility)
    pub status: String, // candidate|confirmed|disputed|deprecated
    pub confidence: f64,
    pub source: String, // scan|llm|user
    pub repo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<EvidenceEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_validated_sha: Option<String>,
}

/// Verdict from user feedback.
#[derive(Debug, Clone, Copy)]
pub enum Verdict {
    Correct,
    Wrong,
    Partial,
}

impl Verdict {
    #[allow(dead_code)]
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Correct => "correct",
            Verdict::Wrong => "wrong",
            Verdict::Partial => "partial",
        }
    }
}

/// Apply one feedback verdict to a fact's confidence and status.
/// Returns updated confidence and new status. Does not mutate.
pub fn apply_verdict(confidence: f64, status: &str, verdict: Verdict) -> (f64, String) {
    let (new_conf, new_status) = match verdict {
        Verdict::Correct => (
            (confidence + 0.15).min(1.0),
            FactStatus::Confirmed.as_str().to_string(),
        ),
        Verdict::Wrong => (
            (confidence - 0.35).max(0.0),
            FactStatus::Disputed.as_str().to_string(),
        ),
        Verdict::Partial => {
            let c = (confidence - 0.10).max(0.0);
            let s = if c < 0.4 {
                FactStatus::Disputed.as_str().to_string()
            } else {
                status.to_string()
            };
            (c, s)
        }
    };
    (new_conf, new_status)
}

/// After applying wrong twice, deprecate if confidence < 0.25.
/// Caller should track consecutive wrong count per fact if desired.
pub fn should_deprecate(confidence: f64, consecutive_wrong: u32) -> bool {
    consecutive_wrong >= 2 && confidence < 0.25
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_verdict_correct_increases_confidence_caps_at_one() {
        let (conf, status) = apply_verdict(0.9, "candidate", Verdict::Correct);
        assert!((conf - 1.0).abs() < 1e-9);
        assert_eq!(status, "confirmed");
        let (conf2, _) = apply_verdict(0.5, "candidate", Verdict::Correct);
        assert!((conf2 - 0.65).abs() < 1e-9);
    }

    #[test]
    fn apply_verdict_wrong_decreases_confidence_floor_zero() {
        let (conf, status) = apply_verdict(0.4, "candidate", Verdict::Wrong);
        assert!((conf - 0.05).abs() < 1e-9);
        assert_eq!(status, "disputed");
        let (conf2, _) = apply_verdict(0.2, "candidate", Verdict::Wrong);
        assert!((conf2 - 0.0).abs() < 1e-9);
    }

    #[test]
    fn apply_verdict_partial_below_04_becomes_disputed() {
        let (conf, status) = apply_verdict(0.35, "candidate", Verdict::Partial);
        assert!(conf < 0.4);
        assert_eq!(status, "disputed");
    }

    #[test]
    fn should_deprecate_requires_two_wrong_and_low_confidence() {
        assert!(!should_deprecate(0.5, 1));
        assert!(!should_deprecate(0.2, 1));
        assert!(should_deprecate(0.2, 2));
        assert!(should_deprecate(0.24, 2));
        assert!(!should_deprecate(0.25, 2));
    }
}
