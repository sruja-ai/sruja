//! Proposal: a single agent's independent solution to a brainstorming problem.

use super::AgentRole;
use crate::cognition::{Plan, TaskTier};

/// A proposal from one brainstorming agent.
#[derive(Debug, Clone)]
pub struct Proposal {
    /// Unique agent identifier.
    pub agent_id: usize,
    /// The role/perspective this agent used.
    pub role: AgentRole,
    /// One-line title of the proposal.
    pub title: String,
    /// Summary of the approach.
    pub summary: String,
    /// Ordered steps in the approach.
    pub approach: Vec<String>,
    /// Identified risks or concerns.
    pub risks: Vec<String>,
    /// Agent's confidence in its own proposal (0.0–1.0).
    pub confidence: f64,
    /// The underlying plan with subtasks.
    pub plan: Plan,
}

impl Proposal {
    /// Number of implementation steps.
    pub fn step_count(&self) -> usize {
        self.approach.len()
    }

    /// Number of identified risks.
    pub fn risk_count(&self) -> usize {
        self.risks.len()
    }

    /// Estimated total complexity (sum of subtask tiers).
    pub fn complexity_score(&self) -> u32 {
        self.plan
            .subtasks
            .iter()
            .map(|s| match s.tier {
                TaskTier::Cheap => 1,
                TaskTier::Mid => 3,
                TaskTier::Premium => 10,
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognition::{Subtask, SubtaskKind};

    #[test]
    fn proposal_metrics() {
        let proposal = Proposal {
            agent_id: 1,
            role: AgentRole::Implementer,
            title: "Add caching".to_string(),
            summary: "Cache frequent queries".to_string(),
            approach: vec!["Add Redis".to_string(), "Wrap queries".to_string()],
            risks: vec!["Cache invalidation".to_string()],
            confidence: 0.85,
            plan: Plan {
                goal: "Add caching".to_string(),
                goal_statement: "Add caching".to_string(),
                criteria: Vec::new(),
                subtasks: vec![
                    Subtask {
                        id: "1.0".to_string(),
                        description: "Setup Redis".to_string(),
                        tier: TaskTier::Cheap,
                        kind: SubtaskKind::Implement,
                        files: Vec::new(),
                        acceptance_criteria: Vec::new(),
                    },
                    Subtask {
                        id: "1.1".to_string(),
                        description: "Add cache layer".to_string(),
                        tier: TaskTier::Mid,
                        kind: SubtaskKind::Implement,
                        files: Vec::new(),
                        acceptance_criteria: Vec::new(),
                    },
                ],
                tdd: false,
                risks: Vec::new(),
                schema_version: String::new(),
            },
        };

        assert_eq!(proposal.step_count(), 2);
        assert_eq!(proposal.risk_count(), 1);
        assert_eq!(proposal.complexity_score(), 4); // 1 + 3
    }
}
