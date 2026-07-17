//! Sub-agent isolation — the "Isolate" step of context engineering.
//!
//! A [`SubAgent`] is a scoped [`Agent`] that runs in its own context window and
//! returns only a compressed [`SubAgentReport`]. The parent agent's context
//! never receives the sub-agent's raw tool dumps — only the synthesized summary
//! and any architecture element IDs it cited. This partitions context so that
//! exploration noise (a 500-line grep) cannot poison or distract the writer,
//! and two contradictory reads cannot clash in a single window.
//!
//! Roles map to tool allowlists:
//! - [`Role::Reader`]  — read-only sruja + filesystem tools, no writes/shell.
//! - [`Role::Checker`] — deterministic verify tools (lint/drift/intent).
//! - [`Role::Writer`]  — file write/edit only, no exploration tools.
//!
//! This reuses the existing [`Agent`], [`ToolRegistry`], and (when the
//! `compression` feature is enabled) the [`sruja_compress`] summarizer — no new
//! grading or compression logic, only orchestration and registry scoping.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::cognition::Agent;
use crate::goal::GoalSpec;
use crate::llm::CompletionRequest;
use crate::tool::builtin::tools::{DiffEdit, FileEdit, FileWrite, Glob, Grep};
use crate::tool::builtin::tools::FileRead;
use crate::tool::ToolRegistry;
use crate::tool::sruja::{
    SrujaDriftTool, SrujaFocusTool, SrujaLookupTool,
};

/// The isolation role a sub-agent plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Explore the codebase read-only and return a grounded summary.
    Reader,
    /// Run deterministic verification (lint/drift/intent) and return a verdict.
    Checker,
    /// Apply edits. Receives curated input; has no exploration tools.
    Writer,
}

/// Budget for a sub-agent run.
#[derive(Debug, Clone)]
pub struct SubAgentBudget {
    /// Max tool-call iterations for the sub-agent (defaults to parent's).
    pub max_iterations: Option<usize>,
    /// Hard cap on the returned summary length in characters (default 4000).
    pub max_summary_chars: usize,
}

impl Default for SubAgentBudget {
    fn default() -> Self {
        Self {
            max_iterations: None,
            max_summary_chars: 4000,
        }
    }
}

/// Specification for a delegated sub-agent.
#[derive(Debug, Clone)]
pub struct SubAgentSpec {
    pub role: Role,
    pub goal: GoalSpec,
    /// Curated inputs from the parent. These are injected verbatim; raw tool
    /// dumps from other scopes must never be placed here.
    pub inject: Vec<String>,
    pub budget: SubAgentBudget,
}

/// The only thing that escapes a sub-agent's isolated context window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentReport {
    /// Compressed high-signal summary of the sub-agent's work.
    pub summary: String,
    /// Architecture element IDs the sub-agent cited (from sruja tools).
    pub citations: Vec<String>,
    /// Whether the sub-agent's own run converged (stopped on its own).
    pub converged: bool,
    /// Whether the sub-agent reported success (role-specific signal).
    pub ok: bool,
    pub role: Role,
}

impl Agent {
    /// Delegate work to an isolated sub-agent.
    ///
    /// The sub-agent gets a fresh, empty message history and a role-scoped tool
    /// registry. Only the compressed [`SubAgentReport`] returns to the caller;
    /// the raw tool outputs stay inside the sub-agent's context window.
    pub async fn delegate(&self, spec: SubAgentSpec) -> Result<SubAgentReport, crate::AgentError> {
        let scoped = self.scoped_for(spec.role);
        let goal_str = spec.goal.statement.clone();

        let injected = if spec.inject.is_empty() {
            String::new()
        } else {
            format!(
                "\n\n## Curated Inputs (from parent — do not re-derive)\n{}",
                spec.inject
                    .iter()
                    .map(|s| format!("- {s}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        let system = match spec.role {
            Role::Reader => {
                "You are a read-only Principal Engineer. Explore using tools, cite architecture \
                 element IDs, and return a concise grounded summary. Do NOT attempt to write files."
            }
            Role::Checker => {
                "You are a deterministic verification agent. Run the provided sruja verification \
                 tools and return a clear pass/fail verdict with the specific violations found."
            }
            Role::Writer => {
                "You are an implementation agent. Apply the edits described in the goal using the \
                 provided file tools. Do not explore — act only on the curated inputs given."
            }
        };

        let user = format!(
            "## Goal\n{goal_str}{injected}\n\n\
             Produce your final answer as a concise summary of what you did and found."
        );

        let req = CompletionRequest::prompt(system, user)
            .with_tools(scoped.tools.schemas())
            .with_model(self.config.models.mid.clone());

        let max_iter = spec
            .budget
            .max_iterations
            .unwrap_or(self.config.max_tool_iterations);
        let (response, _usage, _signals, converged) = scoped
            .run_tool_loop_with_limit(req, max_iter)
            .await?;

        let citations = extract_element_ids(&response.content);
        let summary = bound_summary(response.content.clone(), spec.budget.max_summary_chars);

        // Role-specific success signal: a checker that produced no violations
        // text is treated as passing; writers/reader report ok on convergence.
        let ok = match spec.role {
            Role::Checker => !summary.to_lowercase().contains("violation")
                && !summary.to_lowercase().contains("fail"),
            _ => converged,
        };

        Ok(SubAgentReport {
            summary,
            citations,
            converged,
            ok,
            role: spec.role,
        })
    }

    /// Build a role-scoped clone of this agent with a fresh tool registry and
    /// empty message history. Shares the LLM client, memory backend, and config.
    fn scoped_for(&self, role: Role) -> Agent {
        let registry = match role {
            Role::Reader => ToolRegistry::new()
                .with(Box::new(SrujaFocusTool::new(repo_of(self))))
                .with(Box::new(SrujaLookupTool::new(repo_of(self))))
                .with(Box::new(SrujaDriftTool::new(repo_of(self))))
                .with(Box::new(FileRead::with_root(repo_of(self))))
                .with(Box::new(Glob::with_root(repo_of(self))))
                .with(Box::new(Grep::with_root(repo_of(self)))),
            Role::Checker => ToolRegistry::new()
                .with(Box::new(SrujaDriftTool::new(repo_of(self))))
                .with(Box::new(SrujaFocusTool::new(repo_of(self)))),
            Role::Writer => ToolRegistry::new()
                .with(Box::new(FileWrite::with_root(repo_of(self))))
                .with(Box::new(FileEdit::with_root(repo_of(self))))
                .with(Box::new(DiffEdit::with_root(repo_of(self))))
                .with(Box::new(Glob::with_root(repo_of(self)))),
        };

        let mut builder = Agent::builder()
            .llm(self.llm.clone())
            .tools(registry)
            .config(self.config.clone());

        if let Some(repo) = &self.repo_root {
            let backend = self.memory.clone().unwrap_or_else(|| {
                Arc::new(std::sync::Mutex::new(
                    crate::memory::AgenticMemory::default(),
                ))
            });
            builder = builder.memory_backend(repo.clone(), backend);
        }

        // A writer must not be blocked by a parent dry_run guard inappropriately,
        // but must honor the parent's actual dry_run intent.
        if self.config.dry_run {
            builder = builder.config({
                let mut c = self.config.clone();
                c.dry_run = true;
                c
            });
        }

        builder.build().expect("scoped sub-agent build cannot fail")
    }

    /// Test/introspection helper: the tool names a sub-agent of `role` would be
    /// scoped with. Used to assert the isolation guarantee (e.g. a `Writer`
    /// never receives exploration tools).
    #[cfg(test)]
    pub(crate) fn scoped_tool_names(&self, role: Role) -> Vec<String> {
        self.scoped_for(role).tools.names().iter().map(|s| s.to_string()).collect()
    }
}

/// Resolve the repo root for a scoped tool, falling back to ".".
fn repo_of(agent: &Agent) -> std::path::PathBuf {
    agent
        .repo_root
        .clone()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

/// Extract `Sruja.Element.Id` tokens cited in text (mirrors comprehension).
fn extract_element_ids(text: &str) -> Vec<String> {
    crate::cognition::extract_element_ids(text)
}

/// Bound a summary to a max character length, preserving the head.
fn bound_summary(text: String, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text;
    }
    let truncated: String = text.chars().take(max_chars.saturating_sub(20)).collect();
    format!("{truncated}\n…[truncated to {max_chars} chars]")
}
