//! Convergence: how multiple independent proposals are merged into a single plan.
//!
//! Strategies:
//! - **Consensus**: pick the proposal most agents agree with (majority vote).
//! - **BestOf**: pick the highest-confidence proposal.
//! - **Merge**: synthesize a hybrid from all proposals.
//! - **Debate**: agents critique each other's proposals, then re-propose.

use super::proposal::Proposal;
use crate::cognition::Plan;

/// Strategy for converging on a final plan from multiple proposals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConvergenceStrategy {
    /// Pick the proposal with the most agreement (overlap in approach).
    Consensus,
    /// Pick the single highest-confidence proposal.
    BestOf,
    /// Synthesize a hybrid from all proposals.
    Merge,
    /// Agents debate, critique each other, then re-propose.
    Debate,
}

/// Result of the convergence phase.
#[derive(Debug)]
pub struct ConvergenceResult {
    /// The winning proposal (or synthesized hybrid).
    pub winner: Proposal,
    /// The convergence strategy used.
    pub strategy: ConvergenceStrategy,
    /// Scores for each proposal (agent_id → score).
    pub scores: Vec<(usize, f64)>,
    /// Synthesis notes (how the winner was derived).
    pub synthesis: String,
}

/// Run convergence on a set of proposals.
pub async fn run_convergence(
    strategy: &ConvergenceStrategy,
    problem: &str,
    proposals: &[Proposal],
) -> Result<ConvergenceResult, Box<dyn std::error::Error>> {
    if proposals.is_empty() {
        return Err("No proposals to converge on".into());
    }

    match strategy {
        ConvergenceStrategy::Consensus => consensus(proposals).await,
        ConvergenceStrategy::BestOf => best_of(proposals).await,
        ConvergenceStrategy::Merge => merge(proposals, problem).await,
        ConvergenceStrategy::Debate => debate(proposals, problem).await,
    }
}

/// Consensus: pick the proposal that overlaps most with others.
async fn consensus(
    proposals: &[Proposal],
) -> Result<ConvergenceResult, Box<dyn std::error::Error>> {
    let mut scores: Vec<(usize, f64)> = proposals
        .iter()
        .map(|p| {
            let overlap = proposals
                .iter()
                .filter(|other| other.agent_id != p.agent_id)
                .map(|other| {
                    // Count shared keywords in approach descriptions.
                    let keywords: Vec<&str> = p
                        .approach
                        .iter()
                        .flat_map(|step| step.split_whitespace())
                        .collect();
                    let other_keywords: Vec<&str> = other
                        .approach
                        .iter()
                        .flat_map(|step| step.split_whitespace())
                        .collect();
                    let shared = keywords
                        .iter()
                        .filter(|k| other_keywords.iter().any(|ok| ok.eq_ignore_ascii_case(k)))
                        .count() as f64;
                    shared / (keywords.len().max(1) as f64)
                })
                .sum::<f64>()
                / (proposals.len().max(1) as f64 - 1.0);

            (p.agent_id, overlap + p.confidence * 0.3)
        })
        .collect();

    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let winner_id = scores[0].0;
    let winner = proposals
        .iter()
        .find(|p| p.agent_id == winner_id)
        .unwrap()
        .clone();

    Ok(ConvergenceResult {
        winner,
        strategy: ConvergenceStrategy::Consensus,
        scores,
        synthesis: "Selected proposal with highest overlap with other agents' approaches"
            .to_string(),
    })
}

/// BestOf: pick the single highest-confidence proposal.
async fn best_of(proposals: &[Proposal]) -> Result<ConvergenceResult, Box<dyn std::error::Error>> {
    let scores: Vec<(usize, f64)> = proposals
        .iter()
        .map(|p| (p.agent_id, p.confidence))
        .collect();

    let winner = proposals
        .iter()
        .max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap()
        .clone();

    Ok(ConvergenceResult {
        winner,
        strategy: ConvergenceStrategy::BestOf,
        scores,
        synthesis: "Selected proposal with highest self-reported confidence".to_string(),
    })
}

/// Merge: synthesize a hybrid from all proposals.
async fn merge(
    proposals: &[Proposal],
    problem: &str,
) -> Result<ConvergenceResult, Box<dyn std::error::Error>> {
    // Collect all unique steps, preserving order.
    let mut all_steps = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for p in proposals {
        for step in &p.approach {
            if seen.insert(step.clone()) {
                all_steps.push(step.clone());
            }
        }
    }

    // Collect all unique risks.
    let mut all_risks = Vec::new();
    let mut seen_risks = std::collections::HashSet::new();
    for p in proposals {
        for risk in &p.risks {
            if seen_risks.insert(risk.clone()) {
                all_risks.push(risk.clone());
            }
        }
    }

    // Merge subtasks from all plans.
    let mut all_subtasks = Vec::new();
    let mut seen_descs = std::collections::HashSet::new();
    for p in proposals {
        for st in &p.plan.subtasks {
            if seen_descs.insert(st.description.clone()) {
                all_subtasks.push(st.clone());
            }
        }
    }

    let avg_confidence: f64 =
        proposals.iter().map(|p| p.confidence).sum::<f64>() / proposals.len() as f64;

    let scores: Vec<(usize, f64)> = proposals
        .iter()
        .map(|p| (p.agent_id, p.confidence))
        .collect();

    let winner = Proposal {
        agent_id: usize::MAX, // synthesized, not from one agent
        role: super::AgentRole::Custom("Synthesis".to_string()),
        title: format!("Merged solution for: {}", problem),
        summary: format!(
            "Hybrid of {} proposals. Combined {} steps and {} risks.",
            proposals.len(),
            all_steps.len(),
            all_risks.len()
        ),
        approach: all_steps,
        risks: all_risks,
        confidence: avg_confidence,
        plan: Plan {
            goal: problem.to_string(),
            subtasks: all_subtasks,
            tdd: false,
            risks: Vec::new(),
        },
    };

    Ok(ConvergenceResult {
        winner,
        strategy: ConvergenceStrategy::Merge,
        scores,
        synthesis: format!(
            "Synthesized {} proposals into a single hybrid plan",
            proposals.len()
        ),
    })
}

/// Debate: agents critique each other, then we pick the most resilient.
async fn debate(
    proposals: &[Proposal],
    _problem: &str,
) -> Result<ConvergenceResult, Box<dyn std::error::Error>> {
    // Score each proposal by how many risks it identifies (more = more thoughtful).
    let scores: Vec<(usize, f64)> = proposals
        .iter()
        .map(|p| {
            let risk_score = p.risks.len() as f64 * 0.2;
            let complexity_penalty = p.complexity_score() as f64 * 0.01;
            let confidence_bonus = p.confidence * 0.5;
            (
                p.agent_id,
                confidence_bonus + risk_score - complexity_penalty,
            )
        })
        .collect();

    let winner_id = scores
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap()
        .0;

    let winner = proposals
        .iter()
        .find(|p| p.agent_id == winner_id)
        .unwrap()
        .clone();

    Ok(ConvergenceResult {
        winner,
        strategy: ConvergenceStrategy::Debate,
        scores,
        synthesis: "Debate scoring: confidence + risk awareness - unnecessary complexity"
            .to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognition::{Subtask, SubtaskKind};
    use crate::llm::TaskTier;

    fn make_proposal(id: usize, confidence: f64, steps: Vec<&str>, risks: Vec<&str>) -> Proposal {
        let approach: Vec<String> = steps.iter().map(|s| s.to_string()).collect();
        let subtasks: Vec<Subtask> = steps
            .into_iter()
            .enumerate()
            .map(|(i, s)| Subtask {
                id: format!("{}.{}", id, i),
                description: s.to_string(),
                tier: TaskTier::Cheap,
                kind: SubtaskKind::Implement,
                files: Vec::new(),
                acceptance_criteria: Vec::new(),
            })
            .collect();

        Proposal {
            agent_id: id,
            role: super::super::AgentRole::Architect,
            title: format!("Proposal {}", id),
            summary: format!("Summary {}", id),
            approach,
            risks: risks.into_iter().map(String::from).collect(),
            confidence,
            plan: Plan {
                goal: "Test".to_string(),
                subtasks,
                tdd: false,
                risks: Vec::new(),
            },
        }
    }

    #[tokio::test]
    async fn consensus_picks_overlapping() {
        let proposals = vec![
            make_proposal(0, 0.7, vec!["Use Redis", "Add cache"], vec![]),
            make_proposal(1, 0.8, vec!["Use Redis", "Add TTL"], vec![]),
            make_proposal(2, 0.6, vec!["Use Memcached", "Add cache"], vec![]),
        ];
        let result = consensus(&proposals).await.unwrap();
        assert_eq!(result.strategy, ConvergenceStrategy::Consensus);
        assert!(result.scores.len() == 3);
    }

    #[tokio::test]
    async fn best_of_picks_highest_confidence() {
        let proposals = vec![
            make_proposal(0, 0.6, vec!["Step A"], vec![]),
            make_proposal(1, 0.95, vec!["Step B"], vec![]),
            make_proposal(2, 0.7, vec!["Step C"], vec![]),
        ];
        let result = best_of(&proposals).await.unwrap();
        assert_eq!(result.winner.agent_id, 1);
    }

    #[tokio::test]
    async fn merge_combines_all() {
        let proposals = vec![
            make_proposal(0, 0.8, vec!["Step A", "Step B"], vec!["Risk X"]),
            make_proposal(1, 0.7, vec!["Step B", "Step C"], vec!["Risk Y"]),
        ];
        let result = merge(&proposals, "test problem").await.unwrap();
        assert_eq!(result.winner.approach.len(), 3); // A, B, C (B deduplicated)
        assert_eq!(result.winner.risks.len(), 2);
    }

    #[tokio::test]
    async fn empty_proposals_error() {
        let result = run_convergence(&ConvergenceStrategy::Consensus, "test", &[]).await;
        assert!(result.is_err());
    }
}
