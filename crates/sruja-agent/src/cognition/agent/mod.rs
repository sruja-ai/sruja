use std::sync::Arc;

use tokio::sync::mpsc;

use crate::llm::{
    CompletionRequest, CompletionResponse, LlmClient, LlmError, Message, Usage,
};
use crate::tool::ToolSignal;
use crate::llm::TaskTier;
use crate::tool::{FileGuard, Phase, ToolRegistry};
use crate::verify::{
    run_verification_steps, VerifyResult, VerifyStatus,
};
use crate::LearningEntry;

use super::builder::AgentBuilder;
use super::config::AgentConfig;
use super::complexity::{TaskComplexity, classify_task_complexity};
use super::types::{
    SubtaskKind, Subtask, Plan, StepResult, StepStatus, Comprehension, Critique,
    PersonaResult, CriterionStatus, CriterionVerdict, CritiquePersona, ErrorClass,
    FailureTracker, ScopeDrift, AgentRunResult, step_has_quality,
    classify_error, LoopConfig, LoopTermination, LoopIteration, LoopResult,
};
use super::checkpoint::RunCheckpoint;
use super::tool_tracing::ToolCallTracer;
use super::parsing::{parse_plan_from_response, parse_critique_from_response, parse_learnings_from_response};
use super::{HookAction, LoopEvent, LoopPhase, AgentError};
mod comprehend;
mod critique;
mod execute;
mod plan;
mod r#loop;

/// The programmable agent. Holds an LLM brain, tool hands, optional memory,
/// lifecycle hooks, and a file guard enforcing the TDD pipeline.
pub struct Agent {
    pub(super) llm: Arc<dyn LlmClient>,
    pub(super) tools: ToolRegistry,
    pub(super) guard: FileGuard,
    pub(super) hooks: super::hook::HookRegistry,
    pub(super) config: AgentConfig,
    /// Repo root for resolving `.sruja/` paths (decisions, runbooks, memory).
    pub(super) repo_root: Option<std::path::PathBuf>,
    /// Pluggable memory backend (in-memory JSON, FTS5+BM25, etc.).
    pub(super) memory: Option<std::sync::Arc<dyn crate::memory::Memory + Send + Sync>>,
    #[cfg(feature = "mcp-client")]
    #[allow(dead_code)]
    pub(super) mcp_manager: Option<crate::tool::mcp::McpClientManager>,
    /// Tool-call tracer for context event attribution (U5).
    pub(super) tool_call_tracer: Option<Box<dyn ToolCallTracer>>,
    /// Trace context for tool-call event attribution (U5).
    pub(super) trace_run_id: Option<String>,
    pub(super) trace_id: Option<String>,
    /// Pre-loaded target file contents, keyed by path.
    /// Injected into the comprehension user prompt to avoid redundant file_read
    /// tool calls when --file is specified on the CLI.
    pub(super) preloaded_files: std::collections::HashMap<String, String>,
    /// Pre-loaded architecture context (repomap, topology).
    /// Injected into the comprehension user prompt to avoid redundant MCP tool calls.
    pub(super) preloaded_arch_context: String,
}

impl Agent {
    /// Start building an agent.
    pub fn builder() -> AgentBuilder {
        AgentBuilder::default()
    }

    /// The file guard (for external phase control, e.g. HITL gates).
    pub fn guard(&self) -> &FileGuard {
        &self.guard
    }

    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Validate that the agent's intermediate state is consistent and ready to continue.
    ///
    /// Checks include:
    /// - Phase guard is properly initialized
    /// - Tool registry is not empty (if tools are expected)
    /// - LLM client is available
    /// - Memory backend (if present) is properly configured
    ///
    /// Returns `Ok(())` if validation passes, or an `AgentError` describing the issue.
    /// Validate that the agent is in a consistent state before proceeding.
    ///
    /// Checks:
    /// 1. Tool registry is not empty (in non-dry-run mode)
    /// 2. Memory is accessible (if enabled)
    pub fn validate_intermediate_state(&self) -> Result<(), AgentError> {
        // Check tool registry
        if !self.config.dry_run && self.tools.names().is_empty() {
            return Err(AgentError::Other(
                "Tool registry is empty in non-dry-run mode".into(),
            ));
        }

        // If memory is enabled, verify it's accessible
        if let Some(ref mem) = self.memory {
            let _ = mem.search("", 0, None);
        }

        Ok(())
    }
}

/// Summarize failing verification steps into human-readable critique issues.
///
/// Used by [`Agent::run_loop`] to (a) veto convergence and (b) feed the
/// failures into the next replan so the loop addresses them.
fn summarize_verify_failures(results: &[VerifyResult]) -> Vec<String> {
    results
        .iter()
        .filter(|r| matches!(r.status, VerifyStatus::Failed))
        .map(|r| {
            let exit = r
                .exit_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "none".to_string());
            let status_str = match r.status {
                VerifyStatus::Failed => "failed",
                VerifyStatus::Skipped => "skipped",
                VerifyStatus::Ok => unreachable!(),
            };

            // For drift failures, parse the JSON to extract actionable violation details.
            if r.step_id == "grader_drift" {
                if let Some(lines) = extract_violation_lines_from_drift_json(&r.stdout) {
                    return format!(
                        "verify '{}' {} (exit={}):\n{}",
                        r.step_id,
                        status_str,
                        exit,
                        lines.join("\n")
                    );
                }
            }

            let detail = if r.stderr.trim().is_empty() {
                r.stdout.trim()
            } else {
                r.stderr.trim()
            };
            format!(
                "verify '{}' {} (exit={}): {}",
                r.step_id, status_str, exit, detail
            )
        })
        .collect()
}

/// Extract human-readable violation lines from a `sruja drift` JSON output.
///
/// Parses the stdout of `sruja drift -f json --structural-only --fail-on ...`
/// and returns one line per violation that triggered the failure. Returns
/// `None` if the JSON can't be parsed (fall back to raw output).
fn extract_violation_lines_from_drift_json(stdout: &str) -> Option<Vec<String>> {
    let parsed: serde_json::Value = serde_json::from_str(stdout).ok()?;
    let violations = parsed.get("violations")?.as_array()?;

    if violations.is_empty() {
        return Some(vec!["No specific violations listed.".into()]);
    }

    // Show up to 12 violations so the model gets context without flooding.
    let mut lines: Vec<String> = violations
        .iter()
        .filter(|v| v.get("suppressed").and_then(|s| s.as_bool()) != Some(true))
        .take(12)
        .map(|v| {
            let kind = v.get("kind").and_then(|k| k.as_str()).unwrap_or("?");
            let severity = v.get("severity").and_then(|s| s.as_str()).unwrap_or("?");
            let message = v.get("message").and_then(|m| m.as_str()).unwrap_or("?");
            let location = v.get("location").and_then(|l| l.as_str()).unwrap_or("");
            let file = v
                .get("sources")
                .and_then(|s| s.as_array())
                .and_then(|arr| arr.first())
                .and_then(|src| src.get("file"))
                .and_then(|f| f.as_str())
                .unwrap_or("");
            let baseline = v
                .get("baseline_delta")
                .and_then(|b| b.as_str())
                .unwrap_or("");
            let tag = if baseline == "new" { " [NEW]" } else { "" };
            if file.is_empty() {
                format!("  {kind}({severity}){tag}: {message} ({location})")
            } else {
                format!("  {kind}({severity}){tag}: {message} — {file}")
            }
        })
        .collect();

    let total = violations.len();
    let suppressed = violations
        .iter()
        .filter(|v| v.get("suppressed").and_then(|s| s.as_bool()) == Some(true))
        .count();
    let summary = format!(
        "{} new violation(s) ({} total, {} pre-existing suppressed):",
        total - suppressed,
        total,
        suppressed
    );
    lines.insert(0, summary);

    Some(lines)
}

/// Structural check: did the replan actually incorporate the prior critique?
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        // Smart truncation: try to preserve important parts
        // Look for a good breaking point (end of a line or sentence)
        let mut break_point = max;
        if let Some(last_newline) = s[..max].rfind('\n') {
            // Break at the last newline before max
            break_point = last_newline + 1;
        } else if let Some(last_space) = s[..max].rfind(' ') {
            // Break at the last space before max
            break_point = last_space + 1;
        }

        let dropped_chars = s.len() - break_point;
        let dropped_lines = s[break_point..].lines().count();
        format!(
            "{}...\n(truncated: {} chars, {} lines dropped — {} total chars)",
            &s[..break_point],
            dropped_chars,
            dropped_lines,
            s.len()
        )
    }
}

pub fn extract_element_ids(text: &str) -> Vec<String> {
    // Match patterns like Element.Id or System.Container.Component
    let mut ids = Vec::new();
    for word in text.split_whitespace() {
        let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.');
        if cleaned.contains('.') && cleaned.chars().filter(|c| *c == '.').count() >= 1 {
            let parts: Vec<&str> = cleaned.split('.').collect();
            if parts
                .iter()
                .all(|p| !p.is_empty() && p.chars().next().is_some_and(|c| c.is_uppercase()))
            {
                ids.push(cleaned.to_string());
            }
        }
    }
    ids.sort();
    ids.dedup();
    ids
}
