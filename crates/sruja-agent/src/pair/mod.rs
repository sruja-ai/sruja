//! Pair programming: two agents (or human+AI) work together on the same task.
//!
//! ## Architecture
//!
//! A `PairSession` manages a Driver and Navigator:
//! - **Driver**: writes the code, executes tools, makes implementation decisions.
//! - **Navigator**: reviews in real-time, catches issues, suggests improvements.
//!
//! They communicate through a `Channel` (shared context) and can swap roles.
//! The navigator sees every change the driver makes and can interrupt with feedback.
//!
//! ## Usage
//!
//! ```no_run
//! # use std::sync::Arc;
//! # use sruja_agent::pair::*;
//! # use sruja_agent::llm::OpenAiClient;
//! # use sruja_agent::tool::ToolRegistry;
//! # async fn demo() -> Result<(), Box<dyn std::error::Error>> {
//! let driver_llm = OpenAiClient::from_env()?;
//! let navigator_llm = OpenAiClient::from_env()?;
//! let tools = ToolRegistry::new();
//!
//! let session = PairSession::builder()
//!     .driver_llm(Arc::new(driver_llm))
//!     .navigator_llm(Arc::new(navigator_llm))
//!     .tools(tools)
//!     .build()?;
//!
//! let result = session.work("Implement a rate limiter").await?;
//! println!("Result: {:?}", result.outcome);
//! # Ok(())
//! # }
//! ```

pub mod channel;

use crate::cognition::{Agent, AgentConfig, TaskTier};
use crate::llm::LlmClient;
use crate::tool::ToolRegistry;
use channel::{Channel, ChannelMessage};
use std::sync::Arc;

/// Role in a pair programming session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairRole {
    /// Writes code, executes tools, makes implementation decisions.
    Driver,
    /// Reviews in real-time, catches issues, suggests improvements.
    Navigator,
}

/// Configuration for a pair programming session.
#[derive(Debug, Clone)]
pub struct PairConfig {
    /// How many rounds of driver/navigator before swapping.
    pub swap_interval: usize,
    /// Maximum total rounds before giving up.
    pub max_rounds: usize,
    /// Whether the navigator can override the driver.
    pub navigator_can_override: bool,
    /// Task complexity tier.
    pub tier: TaskTier,
}

impl Default for PairConfig {
    fn default() -> Self {
        Self {
            swap_interval: 3,
            max_rounds: 20,
            navigator_can_override: true,
            tier: TaskTier::Mid,
        }
    }
}

/// Builder for a pair programming session.
pub struct PairSessionBuilder {
    driver_llm: Option<Arc<dyn LlmClient>>,
    navigator_llm: Option<Arc<dyn LlmClient>>,
    tools: Option<ToolRegistry>,
    config: PairConfig,
}

/// A pair programming session with a driver and navigator.
pub struct PairSession {
    driver_llm: Arc<dyn LlmClient>,
    navigator_llm: Arc<dyn LlmClient>,
    config: PairConfig,
}

impl Default for PairSessionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PairSessionBuilder {
    pub fn new() -> Self {
        Self {
            driver_llm: None,
            navigator_llm: None,
            tools: None,
            config: PairConfig::default(),
        }
    }

    pub fn driver_llm(mut self, llm: Arc<dyn LlmClient>) -> Self {
        self.driver_llm = Some(llm);
        self
    }

    pub fn navigator_llm(mut self, llm: Arc<dyn LlmClient>) -> Self {
        self.navigator_llm = Some(llm);
        self
    }

    pub fn tools(mut self, tools: ToolRegistry) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn config(mut self, config: PairConfig) -> Self {
        self.config = config;
        self
    }

    pub fn swap_interval(mut self, interval: usize) -> Self {
        self.config.swap_interval = interval;
        self
    }

    pub fn max_rounds(mut self, max: usize) -> Self {
        self.config.max_rounds = max;
        self
    }

    pub fn build(self) -> Result<PairSession, String> {
        let driver_llm = self.driver_llm.ok_or("Driver LLM is required")?;
        let navigator_llm = self.navigator_llm.ok_or("Navigator LLM is required")?;

        Ok(PairSession {
            driver_llm,
            navigator_llm,
            config: self.config,
        })
    }
}

impl PairSession {
    pub fn builder() -> PairSessionBuilder {
        PairSessionBuilder::new()
    }

    /// Run the pair programming session.
    pub async fn work(&self, task: &str) -> Result<PairResult, Box<dyn std::error::Error>> {
        let channel = Channel::new();
        let mut round = 0;
        let mut current_role = PairRole::Driver;
        let driver_agent = self.create_agent(&self.driver_llm).await?;
        let navigator_agent = self.create_agent(&self.navigator_llm).await?;

        // Phase 1: Both agents comprehend the task.
        let comprehension = driver_agent.comprehend(task).await?;
        let nav_comprehension = navigator_agent.comprehend(task).await?;

        // Navigator sends initial observations.
        channel.send(ChannelMessage::Observation {
            agent: PairRole::Navigator,
            content: format!(
                "I've analyzed the task. Key concerns: {}",
                nav_comprehension.summary
            ),
        });

        // Phase 2: Driver plans, navigator reviews.
        let plan = driver_agent.plan(task, &comprehension).await?;
        channel.send(ChannelMessage::PlanReview {
            agent: PairRole::Navigator,
            feedback: format!(
                "Plan has {} subtasks. Suggest ordering by risk: tackle risky parts first.",
                plan.subtasks.len()
            ),
            approved: true,
        });

        // Phase 3: Iterative implementation rounds.
        let mut iterations = Vec::new();
        let mut all_results = Vec::new();

        loop {
            round += 1;
            if round > self.config.max_rounds {
                break;
            }

            // Check for role swap.
            if round % self.config.swap_interval == 0 {
                current_role = match current_role {
                    PairRole::Driver => PairRole::Navigator,
                    PairRole::Navigator => PairRole::Driver,
                };
                channel.send(ChannelMessage::RoleSwap {
                    from: match current_role {
                        PairRole::Driver => PairRole::Navigator,
                        PairRole::Navigator => PairRole::Driver,
                    },
                    to: current_role.clone(),
                });
            }

            match current_role {
                PairRole::Driver => {
                    // Driver executes the next subtask.
                    let step = plan.subtasks.get(round % plan.subtasks.len().max(1));
                    if let Some(subtask) = step {
                        let result = driver_agent.execute_step(subtask, &comprehension).await?;
                        all_results.push(result.clone());

                        channel.send(ChannelMessage::Change {
                            agent: PairRole::Driver,
                            description: subtask.description.clone(),
                            files_affected: result.files_affected.clone(),
                        });

                        // Navigator reviews the change.
                        let review = navigator_agent
                            .review_change(&subtask.description, &result.output)
                            .await?;

                        channel.send(ChannelMessage::Review {
                            agent: PairRole::Navigator,
                            approved: review.approved,
                            feedback: review.feedback.clone(),
                        });

                        if !review.approved && self.config.navigator_can_override {
                            // Navigator suggests a fix.
                            let fix = navigator_agent.suggest_fix(&review).await?;
                            channel.send(ChannelMessage::Suggestion {
                                agent: PairRole::Navigator,
                                suggestion: fix,
                            });
                        }

                        iterations.push(RoundIteration {
                            round,
                            role: PairRole::Driver,
                            action: subtask.description.clone(),
                            approved: review.approved,
                        });
                    } else {
                        break; // All subtasks done.
                    }
                }
                PairRole::Navigator => {
                    // Navigator drives a cleanup/review pass.
                    let cleanup = navigator_agent.suggest_cleanup(&all_results).await?;
                    channel.send(ChannelMessage::Cleanup {
                        agent: PairRole::Navigator,
                        suggestions: cleanup,
                    });

                    iterations.push(RoundIteration {
                        round,
                        role: PairRole::Navigator,
                        action: "Cleanup pass".to_string(),
                        approved: true,
                    });
                }
            }

            // Check if all subtasks are covered.
            if round >= plan.subtasks.len() {
                break;
            }
        }

        Ok(PairResult {
            task: task.to_string(),
            rounds: round,
            iterations,
            channel,
            outcome: PairOutcome::Success,
        })
    }

    async fn create_agent(
        &self,
        llm: &Arc<dyn LlmClient>,
    ) -> Result<Agent, Box<dyn std::error::Error>> {
        let config = AgentConfig {
            models: crate::cognition::ModelMapping::default(),
            review_every_change: false,
            ..Default::default()
        };

        Agent::builder()
            .llm(Arc::clone(llm))
            .tools(ToolRegistry::new())
            .config(config)
            .build()
            .map_err(|e| e.into())
    }
}

/// Result of a single round in the pair session.
#[derive(Debug, Clone)]
pub struct RoundIteration {
    pub round: usize,
    pub role: PairRole,
    pub action: String,
    pub approved: bool,
}

/// Review result from the navigator.
#[derive(Debug, Clone)]
pub struct ReviewResult {
    pub approved: bool,
    pub feedback: String,
}

/// Step execution result from the driver.
#[derive(Debug, Clone)]
pub struct StepResult {
    pub output: String,
    pub files_affected: Vec<String>,
}

/// Outcome of a pair programming session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairOutcome {
    Success,
    Stalled,
    MaxRoundsExceeded,
}

/// Result of a pair programming session.
#[derive(Debug)]
pub struct PairResult {
    pub task: String,
    pub rounds: usize,
    pub iterations: Vec<RoundIteration>,
    pub channel: Channel,
    pub outcome: PairOutcome,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_role_equality() {
        assert_eq!(PairRole::Driver, PairRole::Driver);
        assert_ne!(PairRole::Driver, PairRole::Navigator);
    }

    #[test]
    fn default_config() {
        let config = PairConfig::default();
        assert_eq!(config.swap_interval, 3);
        assert_eq!(config.max_rounds, 20);
        assert!(config.navigator_can_override);
    }

    #[test]
    fn round_iteration() {
        let iter = RoundIteration {
            round: 1,
            role: PairRole::Driver,
            action: "Implement rate limiter".to_string(),
            approved: true,
        };
        assert_eq!(iter.round, 1);
        assert!(iter.approved);
    }
}
