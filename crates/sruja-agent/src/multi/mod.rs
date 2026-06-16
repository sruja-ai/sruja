//! Multi-agent brainstorming: multiple agents work independently on a problem,
//! then converge on the best solution.
//!
//! ## Architecture
//!
//! A `BrainstormSession` spawns N agents with different perspectives (roles).
//! Each agent independently comprehends → plans → critiques. After all agents
//! finish, a convergence phase merges proposals, resolves conflicts, and picks
//! the best approach (or synthesizes a hybrid).
//!
//! ## Usage
//!
//! ```no_run
//! # use std::sync::Arc;
//! # use sruja_agent::multi::*;
//! # use sruja_agent::llm::OpenAiClient;
//! # use sruja_agent::tool::ToolRegistry;
//! # async fn demo() -> Result<(), Box<dyn std::error::Error>> {
//! let llm = OpenAiClient::from_env()?;
//! let tools = ToolRegistry::new();
//!
//! let session = BrainstormSession::builder()
//!     .llm(Arc::new(llm))
//!     .tools(tools)
//!     .agent_count(3)
//!     .roles(vec![
//!         AgentRole::Architect,
//!         AgentRole::Implementer,
//!         AgentRole::Reviewer,
//!     ])
//!     .build()?;
//!
//! let result = session.brainstorm("How should we refactor the API layer?").await?;
//! println!("Winning proposal: {}", result.convergence.winner.title);
//! # Ok(())
//! # }
//! ```

pub mod converge;
pub mod proposal;

use crate::cognition::{Agent, AgentConfig, Comprehension, Plan, TaskTier};
use crate::llm::{CompletionRequest, LlmClient};
use crate::tool::ToolRegistry;
use converge::{ConvergenceResult, ConvergenceStrategy};
use proposal::Proposal;
use std::sync::Arc;

/// Role assigned to a brainstorming agent — determines perspective and bias.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentRole {
    /// Focuses on architecture, boundaries, and long-term consequences.
    Architect,
    /// Focuses on implementation details, pragmatism, and delivery speed.
    Implementer,
    /// Focuses on risks, edge cases, and what could go wrong.
    Reviewer,
    /// Focuses on user experience and product outcomes.
    Product,
    /// Focuses on performance, scalability, and operational concerns.
    SRE,
    /// Custom role with a user-defined perspective.
    Custom(String),
}

impl AgentRole {
    /// System prompt fragment that biases the agent's perspective.
    pub fn perspective_prompt(&self) -> &str {
        match self {
            Self::Architect => {
                "You are an Architect. Focus on boundaries, long-term consequences, \
                 architectural integrity, and system-wide impact. Challenge proposals \
                 that create coupling or violate layering."
            }
            Self::Implementer => {
                "You are an Implementer. Focus on pragmatism, delivery speed, code clarity, \
                 and what can be shipped today. Challenge proposals that are over-engineered \
                 or require too many steps."
            }
            Self::Reviewer => {
                "You are a Reviewer. Focus on risks, edge cases, failure modes, and what \
                 could go wrong. Challenge proposals that skip testing or have unclear error handling."
            }
            Self::Product => {
                "You are a Product thinker. Focus on user value, simplicity, and outcomes. \
                 Challenge proposals that add complexity without user benefit."
            }
            Self::SRE => {
                "You are an SRE. Focus on operational concerns: monitoring, rollback, \
                 performance, and failure recovery. Challenge proposals that are hard to \
                 debug or deploy."
            }
            Self::Custom(perspective) => perspective,
        }
    }
}

/// Configuration for a brainstorming session.
#[derive(Debug, Clone)]
pub struct BrainstormConfig {
    /// Number of independent agents to spawn.
    pub agent_count: usize,
    /// Roles for each agent. If fewer than `agent_count`, agents are assigned
    /// roles cyclically.
    pub roles: Vec<AgentRole>,
    /// Convergence strategy to use after all agents finish.
    pub convergence: ConvergenceStrategy,
    /// Maximum total tokens across all agents (cost cap).
    pub max_total_tokens: Option<u64>,
    /// Task complexity tier for model routing.
    pub tier: TaskTier,
}

impl Default for BrainstormConfig {
    fn default() -> Self {
        Self {
            agent_count: 3,
            roles: vec![
                AgentRole::Architect,
                AgentRole::Implementer,
                AgentRole::Reviewer,
            ],
            convergence: ConvergenceStrategy::Consensus,
            max_total_tokens: None,
            tier: TaskTier::Mid,
        }
    }
}

/// Builder for a brainstorming session.
pub struct BrainstormSessionBuilder {
    llm: Option<Arc<dyn LlmClient>>,
    tools: Option<ToolRegistry>,
    config: BrainstormConfig,
}

/// A brainstorming session with multiple independent agents.
pub struct BrainstormSession {
    llm: Arc<dyn LlmClient>,
    config: BrainstormConfig,
}

impl BrainstormSessionBuilder {
    pub fn new() -> Self {
        Self {
            llm: None,
            tools: None,
            config: BrainstormConfig::default(),
        }
    }

    pub fn llm(mut self, llm: Arc<dyn LlmClient>) -> Self {
        self.llm = Some(llm);
        self
    }

    pub fn tools(mut self, tools: ToolRegistry) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn agent_count(mut self, count: usize) -> Self {
        self.config.agent_count = count;
        self
    }

    pub fn roles(mut self, roles: Vec<AgentRole>) -> Self {
        self.config.roles = roles;
        self
    }

    pub fn convergence(mut self, strategy: ConvergenceStrategy) -> Self {
        self.config.convergence = strategy;
        self
    }

    pub fn tier(mut self, tier: TaskTier) -> Self {
        self.config.tier = tier;
        self
    }

    pub fn max_total_tokens(mut self, max: u64) -> Self {
        self.config.max_total_tokens = Some(max);
        self
    }

    pub fn build(self) -> Result<BrainstormSession, String> {
        let llm = self.llm.ok_or("LLM client is required")?;

        Ok(BrainstormSession {
            llm,
            config: self.config,
        })
    }
}

impl BrainstormSession {
    pub fn builder() -> BrainstormSessionBuilder {
        BrainstormSessionBuilder::new()
    }

    /// Run the brainstorming session: spawn N agents independently, then converge.
    pub async fn brainstorm(
        &self,
        problem: &str,
    ) -> Result<BrainstormResult, Box<dyn std::error::Error>> {
        let mut proposals = Vec::new();

        // Phase 1: Independent brainstorming — each agent works alone.
        for i in 0..self.config.agent_count {
            let role = self
                .config
                .roles
                .get(i % self.config.roles.len())
                .cloned()
                .unwrap_or(AgentRole::Custom("General".to_string()));

            let proposal = self.run_agent(i, &role, problem).await?;
            proposals.push(proposal);
        }

        // Phase 2: Convergence — merge, review, and pick the best.
        let convergence =
            converge::run_convergence(&self.config.convergence, problem, &proposals).await?;

        Ok(BrainstormResult {
            proposals,
            convergence,
        })
    }

    /// Run a single agent with a specific role.
    async fn run_agent(
        &self,
        agent_id: usize,
        role: &AgentRole,
        problem: &str,
    ) -> Result<Proposal, Box<dyn std::error::Error>> {
        let config = AgentConfig {
            models: crate::cognition::ModelMapping::default(),
            review_every_change: false,
            ..Default::default()
        };

        let agent = Agent::builder()
            .llm(Arc::clone(&self.llm))
            .tools(ToolRegistry::new())
            .config(config)
            .build()?;

        // Comprehend with role-specific perspective.
        let system_prompt = format!(
            "{}\n\nYou are Agent #{} in a brainstorming session. \
             Independently analyze this problem and propose a solution.",
            role.perspective_prompt(),
            agent_id
        );

        let comprehension = agent
            .comprehend_with_context(problem, &system_prompt)
            .await?;

        // Plan from this agent's perspective.
        let plan = agent
            .plan_from_comprehension(problem, &comprehension)
            .await?;

        // Build the proposal.
        Ok(Proposal {
            agent_id,
            role: role.clone(),
            title: plan.goal.clone(),
            summary: comprehension.summary.clone(),
            approach: plan
                .subtasks
                .iter()
                .map(|s| s.description.clone())
                .collect(),
            risks: plan
                .subtasks
                .iter()
                .filter(|s| matches!(s.tier, TaskTier::Premium))
                .map(|s| s.description.clone())
                .collect(),
            confidence: 0.8, // default confidence
            plan,
        })
    }
}

/// Result of a brainstorming session.
#[derive(Debug)]
pub struct BrainstormResult {
    /// All proposals from independent agents.
    pub proposals: Vec<Proposal>,
    /// The convergence result (winning proposal + synthesis).
    pub convergence: ConvergenceResult,
}

/// Trait for a brainstormable agent — extends Agent with brainstorming capability.
pub trait Brainstormable {
    fn comprehend_with_context(
        &self,
        query: &str,
        system_context: &str,
    ) -> impl std::future::Future<Output = Result<Comprehension, Box<dyn std::error::Error>>>;

    fn plan_from_comprehension(
        &self,
        goal: &str,
        comprehension: &Comprehension,
    ) -> impl std::future::Future<Output = Result<Plan, Box<dyn std::error::Error>>>;
}

impl Brainstormable for Agent {
    async fn comprehend_with_context(
        &self,
        query: &str,
        system_context: &str,
    ) -> Result<Comprehension, Box<dyn std::error::Error>> {
        // Use the agent's LLM with custom system prompt.
        let req = CompletionRequest::prompt(system_context, query).with_model("gpt-4o");

        let (response, _usage) = self
            .run_tool_loop(req)
            .await
            .map_err(|e| format!("Comprehension failed: {}", e))?;

        Ok(Comprehension {
            goal: query.to_string(),
            summary: response.content.clone(),
            cited_elements: Vec::new(),
            key_findings: Vec::new(),
            risks: Vec::new(),
            usage: crate::llm::Usage::default(),
        })
    }

    async fn plan_from_comprehension(
        &self,
        goal: &str,
        comprehension: &Comprehension,
    ) -> Result<Plan, Box<dyn std::error::Error>> {
        let user = format!(
            "## Goal\n{}\n\n## Context\n{}\n\n\
             Break this into complexity-tagged subtasks (cheap/mid/premium).",
            goal, comprehension.summary
        );

        let req = CompletionRequest::prompt(crate::cognition::PLAN_SYSTEM_PROMPT, &user)
            .with_model("gpt-4o");

        let (response, _usage) = self
            .run_tool_loop(req)
            .await
            .map_err(|e| format!("Planning failed: {}", e))?;

        let plan = crate::cognition::parse_plan_from_response(&response.content, goal, false);
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_role_prompts() {
        assert!(AgentRole::Architect
            .perspective_prompt()
            .contains("boundaries"));
        assert!(AgentRole::Implementer
            .perspective_prompt()
            .contains("pragmatism"));
        assert!(AgentRole::Reviewer.perspective_prompt().contains("risks"));
        assert!(AgentRole::Product
            .perspective_prompt()
            .contains("user value"));
        assert!(AgentRole::SRE.perspective_prompt().contains("operational"));
    }

    #[test]
    fn custom_role() {
        let role = AgentRole::Custom("Security expert".to_string());
        assert!(role.perspective_prompt().contains("Security expert"));
    }

    #[test]
    fn default_config() {
        let config = BrainstormConfig::default();
        assert_eq!(config.agent_count, 3);
        assert_eq!(config.roles.len(), 3);
    }

    #[test]
    fn proposal_fields() {
        let proposal = Proposal {
            agent_id: 0,
            role: AgentRole::Architect,
            title: "Refactor API".to_string(),
            summary: "Split into layers".to_string(),
            approach: vec!["Step 1".to_string()],
            risks: vec!["Risk 1".to_string()],
            confidence: 0.9,
            plan: Plan {
                goal: "Refactor API".to_string(),
                subtasks: Vec::new(),
                tdd: false,
                risks: Vec::new(),
            },
        };
        assert_eq!(proposal.agent_id, 0);
        assert_eq!(proposal.role, AgentRole::Architect);
    }
}
