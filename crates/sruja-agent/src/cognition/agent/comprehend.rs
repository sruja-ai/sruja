use super::*;

use crate::cognition::prompts;

impl Agent {
    // --- Phase 0: Comprehension (read-only, grounded) ---

    /// Deeply understand a goal using available tools, then produce a
    /// grounded summary citing architecture element IDs.
    ///
    /// If memory is enabled, relevant past learnings are injected into the
    /// context — the agent learns from its own history.
    pub async fn comprehend(
        &self,
        goal: &crate::goal::GoalSpec,
    ) -> Result<Comprehension, AgentError> {
        self.guard.set_phase(Phase::Comprehend);
        self.hooks.on_phase_change(Phase::Comprehend).await;

        let goal_str = goal.statement.as_str();

        // Retrieve relevant memories (token-budget capped).
        let (memory_context, retrieved_learning_ids) = if let Some(ref mem) = self.memory {
            let learnings = mem.search(goal_str, 5, None);
            if learnings.is_empty() {
                (String::new(), Vec::new())
            } else {
                let ids: Vec<String> = learnings.iter().map(|l| l.id.clone()).collect();
                let entries: Vec<String> = learnings
                    .iter()
                    .map(|l| {
                        let kind = l.kind.map(|k| format!("{k:?}")).unwrap_or_default();
                        let utility = l
                            .utility_ratio()
                            .map(|u| format!("{:.0}%", u * 100.0))
                            .unwrap_or_default();
                        format!(
                            "- [{kind}] {} (utility: {utility}, retrieved {} times)\n  Advice: {}",
                            l.context, l.retrieval_count, l.guardrail_advice
                        )
                    })
                    .collect();
                (
                    format!(
                        "\n\n## Past Learnings (from previous runs)\n\
                         The following lessons were learned from earlier tasks. \
                         Use them to avoid repeating mistakes and replicate successes:\n{}",
                        entries.join("\n")
                    ),
                    ids,
                )
            }
        } else {
            (String::new(), Vec::new())
        };

        let hints = if self.config.system_hints.is_empty() {
            String::new()
        } else {
            format!(
                "\n\n## Additional Instructions\n{}",
                self.config
                    .system_hints
                    .iter()
                    .map(|h| format!("- {h}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        // Retrieve error frequency history for this repo.
        let (error_history, pre_conditions) = if let Some(ref mem) = self.memory {
            if let Some(repo_path) = &self.repo_root {
                let repo_path_str = repo_path.display().to_string();
                if let Ok(frequencies) = mem.search_error_history(&repo_path_str) {
                    if frequencies.is_empty() {
                        (String::new(), Vec::new())
                    } else {
                        let total: usize = frequencies.iter().map(|f| f.count).sum();
                        let mut percentages = Vec::new();
                        let mut preconds = Vec::new();
                        for f in &frequencies {
                            let pct = if total > 0 {
                                (f.count as f64 / total as f64 * 100.0) as u32
                            } else {
                                0
                            };
                            let (advice, precond) = match f.error_class {
                                ErrorClass::Compilation => (
                                    "(run cargo check first)",
                                    if pct >= 20 {
                                        Some("Run `cargo check` before editing — high rate of compilation errors in this repo.".to_string())
                                    } else {
                                        None
                                    },
                                ),
                                ErrorClass::Type => (
                                    "(check type annotations before tests)",
                                    if pct >= 20 {
                                        Some("Check type annotations and trait bounds carefully — type errors are common here.".to_string())
                                    } else {
                                        None
                                    },
                                ),
                                ErrorClass::Test => (
                                    "(verify logic against acceptance criteria)",
                                    if pct >= 20 {
                                        Some("Verify test assertions against acceptance criteria before implementing.".to_string())
                                    } else {
                                        None
                                    },
                                ),
                                ErrorClass::Runtime => (
                                    "(check for unwrap/None, bounds)",
                                    if pct >= 20 {
                                        Some("Check for unwrap/None and bounds — runtime panics are frequent in this repo.".to_string())
                                    } else {
                                        None
                                    },
                                ),
                                ErrorClass::Lint => (
                                    "(run cargo clippy)",
                                    if pct >= 20 {
                                        Some("Run `cargo clippy --fix` after changes.".to_string())
                                    } else {
                                        None
                                    },
                                ),
                                ErrorClass::Architecture => (
                                    "(check boundary crossings)",
                                    if pct >= 20 {
                                        Some("Run `sruja drift` before verification — boundary violations are common.".to_string())
                                    } else {
                                        None
                                    },
                                ),
                                ErrorClass::SpecGap => (
                                    "(verify all criteria are addressed)",
                                    if pct >= 20 {
                                        Some("Verify all acceptance criteria are addressed before submitting.".to_string())
                                    } else {
                                        None
                                    },
                                ),
                                ErrorClass::Other => ("(investigate carefully)", None),
                            };
                            percentages.push(format!("{}% {:?} {}", pct, f.error_class, advice));
                            if let Some(pc) = precond {
                                preconds.push(pc);
                            }
                        }
                        let history = format!(
                            "\n\n## Error History for This Repo\n\
                             This repo's past agent runs had these failure patterns:\n\
                             - {}\n\
                             Focus your attention accordingly.",
                            percentages.join("\n- ")
                        );
                        (history, preconds)
                    }
                } else {
                    (String::new(), Vec::new())
                }
            } else {
                (String::new(), Vec::new())
            }
        } else {
            (String::new(), Vec::new())
        };

        let system = format!("{}{memory_context}{error_history}{hints}", prompts::COMPREHENSION_SYSTEM_PROMPT);

        let preloaded_section = if self.preloaded_files.is_empty() {
            String::new()
        } else {
            let mut sections = Vec::new();
            for (path, content) in &self.preloaded_files {
                sections.push(format!("### {path}\n```\n{content}\n```"));
            }
            format!(
                "\n\n## Pre-loaded Target Files\n\
                 The following files have been provided for your reference. \
                 Do NOT call file_read for these — the content is already here.\n\n{}",
                sections.join("\n\n")
            )
        };

        // Include pre-loaded architecture context if available
        let arch_context_section = if self.preloaded_arch_context.is_empty() {
            String::new()
        } else {
            self.preloaded_arch_context.clone()
        };

        let user = format!(
            "## Goal\n{goal_str}\n\n\
             ## Instructions\n\
             Use the available tools to explore the codebase. \
             Cite architecture element IDs in your findings. \
             Produce a concise, grounded understanding.{preloaded_section}{arch_context_section}"
        );

        // Delegate exploration to an isolated Reader sub-agent.
        // The Reader gets a fresh context window with only read-only tools,
        // preventing exploration noise from poisoning later phases.
        tracing::info!("comprehend: delegating exploration to Reader sub-agent");
        let report = self
            .delegate(crate::cognition::subagent::SubAgentSpec {
                role: crate::cognition::subagent::Role::Reader,
                goal: goal.clone(),
                inject: Vec::new(),
                budget: crate::cognition::subagent::SubAgentBudget {
                    max_iterations: Some(self.config.max_tool_iterations),
                    max_summary_chars: 8000,
                },
                system_prompt: Some(system),
                user_prompt: Some(user),
            })
            .await?;

        let cited_elements = extract_element_ids(&report.summary);

        let complexity = classify_task_complexity(
            goal_str,
            &goal.target_files,
            &goal.target_elements,
        );
        tracing::info!(?complexity, "comprehend: classified task complexity");

        let summary = report.summary;
        let final_usage = crate::llm::Usage::default();

        Ok(Comprehension {
            goal: goal.to_string(),
            summary,
            cited_elements,
            key_findings: Vec::new(),
            risks: Vec::new(),
            usage: final_usage,
            retrieved_learning_ids,
            complexity,
            pre_conditions,
        })
    }
}
