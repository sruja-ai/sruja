//! Structured goal specification — the verification contract for autonomous loops.
//!
//! A [`GoalSpec`] is richer than a plain-text goal string: it carries
//! verifiable acceptance criteria, scope hints, and constraints. The
//! deterministic verify layer checks criteria; the LLM critic uses them
//! as its rubric.
//!
//! A plain `&str` converts to a `GoalSpec` with only `statement` set —
//! fully backward-compatible with `run_loop(goal: &str, ...)`.

use serde::{Deserialize, Serialize};

/// A structured goal that doubles as the verification contract.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoalSpec {
    /// The free-text goal statement (what the agent should accomplish).
    pub statement: String,

    /// Verifiable conditions that must hold when the goal is met.
    /// Each criterion should be independently checkable.
    /// Example: "all existing tests pass", "JWT auth on /api/* endpoints"
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acceptance_criteria: Vec<String>,

    /// Files or paths the agent should focus on (scope hint, not a hard restriction).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_files: Vec<String>,

    /// Architecture element IDs (from repo.sruja) the goal relates to.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_elements: Vec<String>,

    /// Constraints on HOW the goal should be achieved (what NOT to do).
    /// Example: "do not modify the public API", "no new dependencies"
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<String>,
}

impl GoalSpec {
    /// Create a goal from a plain-text statement.
    pub fn new(statement: impl Into<String>) -> Self {
        Self {
            statement: statement.into(),
            ..Default::default()
        }
    }

    /// Add an acceptance criterion.
    pub fn with_criterion(mut self, criterion: impl Into<String>) -> Self {
        self.acceptance_criteria.push(criterion.into());
        self
    }

    /// Add a constraint.
    pub fn with_constraint(mut self, constraint: impl Into<String>) -> Self {
        self.constraints.push(constraint.into());
        self
    }

    /// Whether this goal has verifiable acceptance criteria.
    pub fn has_criteria(&self) -> bool {
        !self.acceptance_criteria.is_empty()
    }

    /// Validate that every `target_elements` ID is present in `available_elements`.
    ///
    /// - Returns `Ok(())` when `target_elements` is empty or `available_elements`
    ///   is `None` (availability unknown, skip check).
    /// - Returns `Err` with a human-readable message listing the unknown IDs.
    pub fn validate(&self, available_elements: Option<&[String]>) -> Result<(), Vec<String>> {
        if self.target_elements.is_empty() {
            return Ok(());
        }
        let available = match available_elements {
            Some(a) => a,
            None => return Ok(()),
        };
        let unknown: Vec<String> = self
            .target_elements
            .iter()
            .filter(|id| !available.iter().any(|a| a == *id))
            .cloned()
            .collect();
        if unknown.is_empty() {
            Ok(())
        } else {
            Err(unknown)
        }
    }

    /// Render the goal as a rich prompt string, embedding criteria and constraints.
    ///
    /// If only `statement` is set, returns it unchanged (backward-compatible).
    /// Otherwise, formats a structured prompt the LLM can self-check against.
    pub fn to_prompt(&self) -> String {
        if self.acceptance_criteria.is_empty() && self.constraints.is_empty() {
            return self.statement.clone();
        }

        let mut out = format!("## Goal\n{}", self.statement);

        if !self.acceptance_criteria.is_empty() {
            out.push_str("\n\n## Acceptance Criteria");
            for (i, c) in self.acceptance_criteria.iter().enumerate() {
                out.push_str(&format!("\n{}. {c}", i + 1));
            }
        }

        if !self.constraints.is_empty() {
            out.push_str("\n\n## Constraints");
            for c in &self.constraints {
                out.push_str(&format!("\n- {c}"));
            }
        }

        out
    }
}

impl From<String> for GoalSpec {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for GoalSpec {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl std::fmt::Display for GoalSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.statement)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_statement_round_trips() {
        let g = GoalSpec::new("fix the bug");
        assert_eq!(g.statement, "fix the bug");
        assert!(!g.has_criteria());
        assert_eq!(g.to_prompt(), "fix the bug");
    }

    #[test]
    fn structured_goal_renders_prompt() {
        let g = GoalSpec::new("Add JWT auth")
            .with_criterion("all tests pass")
            .with_criterion("token validation tested")
            .with_constraint("no new deps");

        let prompt = g.to_prompt();
        assert!(prompt.contains("## Goal"));
        assert!(prompt.contains("## Acceptance Criteria"));
        assert!(prompt.contains("1. all tests pass"));
        assert!(prompt.contains("2. token validation tested"));
        assert!(prompt.contains("## Constraints"));
        assert!(prompt.contains("- no new deps"));
        assert!(g.has_criteria());
    }

    #[test]
    fn from_str_converts() {
        let g: GoalSpec = "do thing".into();
        assert_eq!(g.statement, "do thing");
        assert!(g.acceptance_criteria.is_empty());
    }

    #[test]
    fn serde_round_trip() {
        let g = GoalSpec::new("test goal")
            .with_criterion("criterion 1")
            .with_constraint("constraint 1");
        let json = serde_json::to_string(&g).unwrap();
        let back: GoalSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(back.statement, "test goal");
        assert_eq!(back.acceptance_criteria, vec!["criterion 1"]);
        assert_eq!(back.constraints, vec!["constraint 1"]);
    }
}
