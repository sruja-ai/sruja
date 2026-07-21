//! Session lifecycle tracking with per-model cost attribution.
//!
//! Tracks session status, token/cost usage, and per-model attribution across
//! LLM calls within a session. Inspired by cognee's `SessionRecord` pattern.
//!
//! # Usage
//!
//! ```rust,no_run
//! use sruja_memory::session::{SessionTracker, SessionRecord};
//!
//! let tracker = SessionTracker::new("my-session".into(), "user-1".into());
//!
//! // Record LLM calls
//! tracker.record_llm_call("gpt-4", 1000, 500, 0.03).unwrap();
//! tracker.record_llm_call("claude-3", 800, 400, 0.02).unwrap();
//!
//! // Get session summary
//! let record = tracker.current_session();
//! assert_eq!(record.tokens_in, 1800);
//! assert_eq!(record.tokens_out, 900);
//! assert!((record.cost_usd - 0.05).abs() < 0.001);
//! ```

use crate::MemoryStoreError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Default timeout after which a session is considered abandoned.
const SESSION_ABANDON_AFTER_SECS: u64 = 1800; // 30 minutes

/// Per-model usage record within a session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelUsage {
    /// Model name (e.g., "gpt-4", "claude-3-opus").
    pub model: String,
    /// Total prompt tokens across all calls with this model.
    pub tokens_in: u32,
    /// Total completion tokens across all calls with this model.
    pub tokens_out: u32,
    /// Total estimated cost in USD for this model.
    pub cost_usd: f64,
    /// Number of LLM calls made with this model.
    pub call_count: u32,
}

/// Session record tracking status, usage, and per-model attribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    /// Unique session identifier.
    pub session_id: String,
    /// User/agent identifier.
    pub user_id: String,
    /// Session status.
    pub status: SessionStatus,
    /// When the session started.
    pub started_at: String,
    /// When the session last had activity.
    pub last_activity_at: String,
    /// When the session ended (if completed).
    pub ended_at: Option<String>,
    /// Total prompt tokens across all models.
    pub tokens_in: u32,
    /// Total completion tokens across all models.
    pub tokens_out: u32,
    /// Total estimated cost in USD.
    pub cost_usd: f64,
    /// Number of errors encountered.
    pub error_count: u32,
    /// Per-model usage breakdown.
    pub model_usage: HashMap<String, ModelUsage>,
}

/// Session status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Session is actively running.
    Running,
    /// Session completed successfully.
    Completed,
    /// Session failed with errors.
    Failed,
    /// Session was abandoned (no activity for >30 minutes).
    /// Computed at read time, not stored.
    Abandoned,
}

/// In-memory session tracker with thread-safe access.
///
/// Tracks the current session's usage and persists to disk periodically.
pub struct SessionTracker {
    /// Current session record.
    session: RwLock<SessionRecord>,
    /// Last activity time (for computing abandoned status).
    last_activity: RwLock<Instant>,
}

impl SessionTracker {
    /// Create a new session tracker.
    pub fn new(session_id: String, user_id: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        let start = Instant::now();
        Self {
            session: RwLock::new(SessionRecord {
                session_id,
                user_id,
                status: SessionStatus::Running,
                started_at: now.clone(),
                last_activity_at: now,
                ended_at: None,
                tokens_in: 0,
                tokens_out: 0,
                cost_usd: 0.0,
                error_count: 0,
                model_usage: HashMap::new(),
            }),
            last_activity: RwLock::new(start),
        }
    }

    /// Record an LLM call with token usage and cost.
    ///
    /// This is the primary entry point for tracking usage. Call this after
    /// each LLM completion to accumulate per-model statistics.
    pub fn record_llm_call(
        &self,
        model: &str,
        tokens_in: u32,
        tokens_out: u32,
        cost_usd: f64,
    ) -> Result<(), MemoryStoreError> {
        let mut session = self
            .session
            .write()
            .map_err(|e| MemoryStoreError::LockPoisoned(e.to_string()))?;
        let mut last_activity = self
            .last_activity
            .write()
            .map_err(|e| MemoryStoreError::LockPoisoned(e.to_string()))?;

        // Update totals
        session.tokens_in += tokens_in;
        session.tokens_out += tokens_out;
        session.cost_usd += cost_usd;
        session.last_activity_at = chrono::Utc::now().to_rfc3339();

        // Update per-model breakdown
        let model_entry = session
            .model_usage
            .entry(model.to_string())
            .or_insert_with(|| ModelUsage {
                model: model.to_string(),
                tokens_in: 0,
                tokens_out: 0,
                cost_usd: 0.0,
                call_count: 0,
            });
        model_entry.tokens_in += tokens_in;
        model_entry.tokens_out += tokens_out;
        model_entry.cost_usd += cost_usd;
        model_entry.call_count += 1;

        // Update last activity
        *last_activity = Instant::now();

        Ok(())
    }

    /// Record an error in the session.
    pub fn record_error(&self) {
        let mut session = self.session.write().unwrap();
        session.error_count += 1;
        session.last_activity_at = chrono::Utc::now().to_rfc3339();
    }

    /// Mark the session as completed.
    pub fn complete(&self) {
        let mut session = self.session.write().unwrap();
        session.status = SessionStatus::Completed;
        session.ended_at = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Mark the session as failed.
    pub fn fail(&self) {
        let mut session = self.session.write().unwrap();
        session.status = SessionStatus::Failed;
        session.ended_at = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Get the current session record.
    ///
    /// Computes `Abandoned` status at read time if the session has been
    /// inactive for more than `SESSION_ABANDON_AFTER_SECS`.
    pub fn current_session(&self) -> SessionRecord {
        let mut session = self.session.read().unwrap().clone();
        let last_activity = *self.last_activity.read().unwrap();

        // Compute abandoned status at read time
        if session.status == SessionStatus::Running
            && last_activity.elapsed() > Duration::from_secs(SESSION_ABANDON_AFTER_SECS)
        {
            session.status = SessionStatus::Abandoned;
        }

        session
    }

    /// Get the total cost of this session in USD.
    ///
    /// This is a lightweight accessor that reads only the cost field,
    /// avoiding the overhead of cloning the full session record.
    pub fn total_cost(&self) -> f64 {
        self.session.read().unwrap().cost_usd
    }

    /// Get the effective status (computes abandoned at read time).
    pub fn effective_status(&self) -> SessionStatus {
        let session = self.session.read().unwrap();
        let last_activity = *self.last_activity.read().unwrap();

        if session.status != SessionStatus::Running {
            return session.status;
        }

        if last_activity.elapsed() > Duration::from_secs(SESSION_ABANDON_AFTER_SECS) {
            SessionStatus::Abandoned
        } else {
            SessionStatus::Running
        }
    }

    /// Get a summary of per-model usage.
    pub fn model_usage_summary(&self) -> Vec<ModelUsage> {
        let session = self.session.read().unwrap();
        session.model_usage.values().cloned().collect()
    }

    /// Get total token counts.
    pub fn totals(&self) -> (u32, u32, f64) {
        let session = self.session.read().unwrap();
        (session.tokens_in, session.tokens_out, session.cost_usd)
    }

    /// Check if the session is still active.
    pub fn is_active(&self) -> bool {
        self.effective_status() == SessionStatus::Running
    }
}

/// Estimate cost for a given model and token counts.
///
/// Uses a simple char-based heuristic when model-specific pricing is not available.
/// For known models, uses approximate per-token pricing.
pub fn estimate_cost(model: &str, tokens_in: u32, tokens_out: u32) -> f64 {
    let (prompt_per_mtok, completion_per_mtok) = match model {
        m if m.contains("gpt-4o") => (2.50, 10.00),
        m if m.contains("gpt-4-turbo") => (10.00, 30.00),
        m if m.contains("gpt-4") => (30.00, 60.00),
        m if m.contains("gpt-3.5") => (0.50, 1.50),
        m if m.contains("claude-3-opus") => (15.00, 75.00),
        m if m.contains("claude-3-sonnet") => (3.00, 15.00),
        m if m.contains("claude-3-haiku") => (0.25, 1.25),
        m if m.contains("claude") => (3.00, 15.00),
        m if m.contains("gemini-pro") => (0.50, 1.50),
        m if m.contains("gemini") => (0.50, 1.50),
        _ => {
            // Fallback: char-based heuristic (len/4 = tokens)
            let total_chars = (tokens_in + tokens_out) * 4; // approximate
            return total_chars as f64 / 1_000_000.0 * 10.0; // $10/MTok average
        }
    };

    let prompt_cost = (tokens_in as f64 / 1_000_000.0) * prompt_per_mtok;
    let completion_cost = (tokens_out as f64 / 1_000_000.0) * completion_per_mtok;
    prompt_cost + completion_cost
}

/// Return a static greeting string identifying the `sruja-memory` session module.
///
/// Useful as a lightweight health-check or smoke-test probe to confirm the
/// session module is linked and callable.
pub fn greet() -> &'static str {
    "Hello from sruja-memory session!"
}

impl SessionRecord {
    /// Format this session record as a human-readable summary.
    ///
    /// Produces a multi-line report covering session metadata, totals,
    /// error count, and a per-model usage breakdown.
    pub fn summary(&self) -> String {
        let mut out = String::new();

        // Header
        out.push_str(&format!(
            "Session: {} | User: {} | Status: {:?}\n",
            self.session_id, self.user_id, self.status
        ));
        out.push_str(&format!(
            "Started: {} | Last activity: {}\n",
            self.started_at, self.last_activity_at
        ));
        if let Some(ref ended) = self.ended_at {
            out.push_str(&format!("Ended: {ended}\n"));
        }

        // Totals
        out.push_str("---\n");
        out.push_str(&format!(
            "Tokens: {} in / {} out ({} total)\n",
            self.tokens_in,
            self.tokens_out,
            self.tokens_in + self.tokens_out
        ));
        out.push_str(&format!("Cost:   ${:.6}\n", self.cost_usd));
        out.push_str(&format!("Errors: {}\n", self.error_count));

        // Per-model breakdown
        if !self.model_usage.is_empty() {
            out.push_str("---\nModels:\n");
            let mut models: Vec<&ModelUsage> = self.model_usage.values().collect();
            models.sort_by(|a, b| b.cost_usd.partial_cmp(&a.cost_usd).unwrap());
            for m in &models {
                out.push_str(&format!(
                    "  {:<24} {:>6} calls | {:>8} in / {:>8} out | ${:.6}\n",
                    m.model, m.call_count, m.tokens_in, m.tokens_out, m.cost_usd
                ));
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_tracker_basic() {
        let tracker = SessionTracker::new("test-session".into(), "user-1".into());

        assert!(tracker.is_active());
        assert_eq!(tracker.effective_status(), SessionStatus::Running);

        // Record some LLM calls
        tracker.record_llm_call("gpt-4", 1000, 500, 0.03).unwrap();
        tracker.record_llm_call("claude-3", 800, 400, 0.02).unwrap();

        let (tokens_in, tokens_out, cost) = tracker.totals();
        assert_eq!(tokens_in, 1800);
        assert_eq!(tokens_out, 900);
        assert!((cost - 0.05).abs() < 0.001);

        // Check model breakdown
        let models = tracker.model_usage_summary();
        assert_eq!(models.len(), 2);
    }

    #[test]
    fn test_session_completion() {
        let tracker = SessionTracker::new("test-session".into(), "user-1".into());
        tracker.complete();

        assert!(!tracker.is_active());
        assert_eq!(tracker.effective_status(), SessionStatus::Completed);
    }

    #[test]
    fn test_session_failure() {
        let tracker = SessionTracker::new("test-session".into(), "user-1".into());
        tracker.record_error();
        tracker.fail();

        let session = tracker.current_session();
        assert_eq!(session.error_count, 1);
        assert_eq!(session.status, SessionStatus::Failed);
    }

    #[test]
    fn test_session_summary() {
        let tracker = SessionTracker::new("test-session".into(), "user-1".into());
        tracker.record_llm_call("gpt-4", 1000, 500, 0.03).unwrap();
        tracker.record_llm_call("claude-3", 800, 400, 0.02).unwrap();
        tracker.record_error();
        tracker.complete();

        let record = tracker.current_session();
        let summary = record.summary();

        // Verify key pieces are present
        assert!(summary.contains("test-session"), "missing session id");
        assert!(summary.contains("user-1"), "missing user id");
        assert!(summary.contains("Completed"), "missing status");
        assert!(summary.contains("Errors: 1"), "missing error count");
        assert!(summary.contains("gpt-4"), "missing gpt-4 model");
        assert!(summary.contains("claude-3"), "missing claude-3 model");
        assert!(summary.contains("Tokens:"), "missing token totals");
        assert!(summary.contains("Cost:"), "missing cost line");
        assert!(summary.contains("Ended:"), "missing ended line");
    }

    #[test]
    fn test_cost_estimation() {
        // GPT-4o
        let cost = estimate_cost("gpt-4o", 1000, 500);
        assert!((cost - 0.0075).abs() < 0.001); // ~$0.0075

        // Claude-3 Opus
        let cost = estimate_cost("claude-3-opus", 1000, 500);
        assert!((cost - 0.0525).abs() < 0.001); // ~$0.0525
    }
}
