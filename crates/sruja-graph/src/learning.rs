//! Shared learning types for agent memory and knowledge graph.
//!
//! These types are the single source of truth for learning entries across
//! sruja-agent (memory CRUD, curation, MaTTS) and sruja-graph (knowledge
//! graph storage, traversal queries).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during memory operations.
#[derive(Debug, Error)]
pub enum MemoryError {
    /// An I/O error occurred while reading or writing the memory file.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// A serialization or deserialization error occurred.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// No learning entry exists for the given id.
    #[error("learning not found: {0}")]
    NotFound(String),
    /// Merge or update requested with invalid or duplicate ids.
    #[error("invalid learning ids: {0}")]
    InvalidIds(String),
}

/// The outcome of an architectural experiment.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentOutcome {
    /// The experiment was successful (e.g., the change improved the architecture).
    #[serde(rename = "success")]
    #[default]
    Success,
    /// The experiment failed (e.g., the change caused a regression or drift).
    #[serde(rename = "failed")]
    Failed,
}

impl std::fmt::Display for ExperimentOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExperimentOutcome::Success => write!(f, "success"),
            ExperimentOutcome::Failed => write!(f, "failed"),
        }
    }
}

/// Evolution intent category, borrowed from Evolver's Gene schema.
///
/// Each category biases how a learning is used in the agent loop:
/// - `Repair`: fix a known failure pattern
/// - `Optimize`: improve performance or efficiency
/// - `Innovate`: add new capabilities
/// - `Explore`: investigate unknown territory
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LearningCategory {
    Repair,
    Optimize,
    Innovate,
    Explore,
}

/// A signal pattern that activates a learning entry.
///
/// Modeled after Evolver's `signals_match` on Genes — allows a learning
/// to specify *when* it should be retrieved based on extracted session signals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalPattern {
    /// Signal name or prefix to match (e.g., "log_error", "perf_bottleneck").
    pub signal: String,
    /// Boost multiplier when this signal matches the current context (1.0 = default).
    #[serde(default = "default_signal_weight")]
    pub weight: f64,
}

fn default_signal_weight() -> f64 {
    1.0
}

/// Constraints bounding a learning's applicability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConstraints {
    /// Maximum number of files a change derived from this learning should touch.
    #[serde(default = "default_max_files")]
    pub max_files: usize,
    /// File paths/patterns that should never be touched when applying this learning.
    #[serde(default)]
    pub forbidden_paths: Vec<String>,
}

fn default_max_files() -> usize {
    20
}

impl Default for LearningConstraints {
    fn default() -> Self {
        Self {
            max_files: 20,
            forbidden_paths: vec![".git".to_string(), "node_modules".to_string()],
        }
    }
}

/// Blast radius of a change — how many files/lines were affected.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlastRadius {
    /// Number of files changed.
    #[serde(default)]
    pub files: usize,
    /// Number of lines changed (added + removed).
    #[serde(default)]
    pub lines: usize,
}

/// Standard signal names extracted from session context.
///
/// These match Evolver's `SIGNAL_PROFILES` and are produced by the
/// [`SignalExtractor`](crate::signal::SignalExtractor).
pub mod signals {
    /// An error was logged in the session.
    pub const LOG_ERROR: &str = "log_error";
    /// A specific error signature was detected (carries detail after `:`).
    pub const ERR_SIG: &str = "errsig";
    /// A performance bottleneck was detected.
    pub const PERF_BOTTLENECK: &str = "perf_bottleneck";
    /// A capability gap was detected.
    pub const CAPABILITY_GAP: &str = "capability_gap";
    /// A user feature request was detected.
    pub const USER_FEATURE_REQUEST: &str = "user_feature_request";
    /// A user improvement suggestion was detected.
    pub const USER_IMPROVEMENT_SUGGESTION: &str = "user_improvement_suggestion";
    /// A recurring error pattern was found.
    pub const RECURRING_ERROR: &str = "recurring_error";
    /// The system appears stagnant (plateau).
    pub const STAGNATION_DETECTED: &str = "evolution_stagnation_detected";
    /// High consecutive repair count detected.
    pub const REPAIR_LOOP_DETECTED: &str = "repair_loop_detected";
    /// Consecutive failures detected.
    pub const FAILURE_LOOP_DETECTED: &str = "failure_loop_detected";
    /// A tool bypass (shell exec outside tool layer) was detected.
    pub const TOOL_BYPASS: &str = "tool_bypass";
}

/// A single learning entry representing a learned architectural lesson.
///
/// Inspired by the Zettelkasten method: each entry is an atomic note with
/// auto-generated tags and bidirectional links to related entries, enabling
/// thematic clustering and associative retrieval.
///
/// This is the canonical type used by both the agent memory system
/// (CRUD, curation, MaTTS) and the knowledge graph (traversal queries).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEntry {
    /// Stable identifier for cross-referencing between entries.
    #[serde(default = "generate_entry_id")]
    pub id: String,
    /// Optional classification for the type of learning.
    ///
    /// This is intentionally optional for backward compatibility with existing
    /// `.sruja/agent_memory.json` files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<LearningKind>,
    /// When this learning was recorded.
    pub timestamp: DateTime<Utc>,
    /// Optional run ID that produced this learning (for traceability).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    /// Optional repository identifier/path (useful for federated multi-repo memory).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// Optional selector string that led to this learning (file/element_id/query/diff).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// The context in which the experiment was performed (e.g., "Refactoring API layer").
    pub context: String,
    /// The hypothesis being tested (e.g., "Moving logic to sruja-engine will reduce CLI bloat").
    pub hypothesis: String,
    /// Whether the experiment succeeded or failed.
    #[serde(default)]
    pub outcome: ExperimentOutcome,
    /// The reason for the outcome (optional).
    pub reason: Option<String>,
    /// Actionable advice for future agents to avoid repeating mistakes or to replicate success.
    pub guardrail_advice: String,
    /// IDs of architectural elements affected by this experiment.
    pub affected_elements: Vec<String>,
    /// Optional evidence references (files, ADR IDs, commands, etc.) that ground this learning.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    /// Optional confidence label (high/medium/low) aligned with Sruja task confidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
    /// Auto-generated thematic tags extracted from context, hypothesis, and guardrail.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Human-in-the-loop classification: precedent, exception, correction, guardrail (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hitl_kind: Option<String>,
    /// IDs of related learning entries (bidirectional Zettelkasten links).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_ids: Vec<String>,
    /// How often this entry was retrieved for a task (focus, agent run, MCP).
    #[serde(default)]
    pub retrieval_count: u32,
    /// Tasks that succeeded after this entry was retrieved.
    #[serde(default)]
    pub task_success_after: u32,
    /// Total tasks where this entry was retrieved (denominator for utility).
    #[serde(default)]
    pub task_total_after: u32,
    /// Evolution category: what kind of change this learning guides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<LearningCategory>,
    /// Signal patterns that trigger this learning's retrieval.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signals_match: Vec<SignalPattern>,
    /// Constraints bounding this learning's applicability (blast radius, forbidden paths).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraints: Option<LearningConstraints>,
    /// Validation commands or preconditions to verify before applying this learning.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validation: Vec<String>,
    /// Measured blast radius of the change that produced this learning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blast_radius: Option<BlastRadius>,
}

/// Partial update for an existing [`LearningEntry`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearningPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<LearningKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hypothesis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<ExperimentOutcome>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guardrail_advice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub affected_elements: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_refs: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hitl_kind: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<LearningCategory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signals_match: Option<Vec<SignalPattern>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraints: Option<Option<LearningConstraints>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blast_radius: Option<Option<BlastRadius>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LearningKind {
    /// A guardrail: "what not to try again".
    Guardrail,
    /// A playbook: "what worked, do this again".
    Playbook,
    /// An invariant: "must always hold".
    Invariant,
}

pub fn generate_entry_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("learn_{}_{}", chrono::Utc::now().timestamp_millis(), seq)
}

impl LearningEntry {
    /// Success rate after retrieval (`None` if never retrieved for a completed task).
    pub fn utility_ratio(&self) -> Option<f64> {
        if self.task_total_after == 0 {
            None
        } else {
            Some(self.task_success_after as f64 / self.task_total_after as f64)
        }
    }

    /// Decay score in [0.0, 1.0] based on age and retrieval frequency.
    ///
    /// Entries that are old and rarely retrieved decay toward 0.0.
    /// Recent or frequently-retrieved entries stay near 1.0.
    /// Uses a half-life of 90 days, boosted by retrieval count.
    pub fn decay_score(&self) -> f64 {
        let age = Utc::now().signed_duration_since(self.timestamp);
        let age_days = age.num_days().max(0) as f64;
        let half_life = 90.0_f64;
        // Retrieval frequency extends effective freshness.
        let retrieval_boost = (self.retrieval_count as f64).ln_1p() * 15.0;
        let effective_age = (age_days - retrieval_boost).max(0.0);
        2.0_f64.powf(-effective_age / half_life)
    }

    /// Age of this entry in days.
    pub fn age_days(&self) -> i64 {
        Utc::now()
            .signed_duration_since(self.timestamp)
            .num_days()
            .max(0)
    }

    /// Checks if this learning entry is relevant to a specific architectural element.
    pub fn is_relevant_to(&self, element_id: &str) -> bool {
        let element_id_lower = element_id.to_lowercase();

        // 1. Check affected elements (direct match or hierarchy match)
        let element_match = self.affected_elements.iter().any(|e| {
            e == element_id
                || element_id.starts_with(&format!("{}.", e))
                || e.starts_with(&format!("{}.", element_id))
        });

        if element_match {
            return true;
        }

        // 2. Check context for keywords
        self.context.to_lowercase().contains(&element_id_lower)
    }

    /// Create a new learning entry with all bookkeeping fields defaulted.
    ///
    /// Prefer the specific constructors ([`playbook`](Self::playbook),
    /// [`guardrail`](Self::guardrail), [`invariant`](Self::invariant)) which
    /// also set the correct `kind` and `outcome`.
    pub fn new(
        context: impl Into<String>,
        hypothesis: impl Into<String>,
        guardrail_advice: impl Into<String>,
    ) -> Self {
        Self {
            id: generate_entry_id(),
            kind: None,
            timestamp: Utc::now(),
            run_id: None,
            repo: None,
            selector: None,
            context: context.into(),
            hypothesis: hypothesis.into(),
            outcome: ExperimentOutcome::default(),
            reason: None,
            guardrail_advice: guardrail_advice.into(),
            affected_elements: Vec::new(),
            evidence_refs: Vec::new(),
            confidence: None,
            tags: Vec::new(),
            hitl_kind: None,
            related_ids: Vec::new(),
            retrieval_count: 0,
            task_success_after: 0,
            task_total_after: 0,
            category: None,
            signals_match: Vec::new(),
            constraints: None,
            validation: Vec::new(),
            blast_radius: None,
        }
    }

    /// A successful playbook: "what worked, do this again".
    pub fn playbook(
        context: impl Into<String>,
        hypothesis: impl Into<String>,
        guardrail_advice: impl Into<String>,
    ) -> Self {
        Self::new(context, hypothesis, guardrail_advice)
            .with_kind(LearningKind::Playbook)
            .with_outcome(ExperimentOutcome::Success)
    }

    /// A guardrail: "what not to try again".
    pub fn guardrail(
        context: impl Into<String>,
        hypothesis: impl Into<String>,
        guardrail_advice: impl Into<String>,
    ) -> Self {
        Self::new(context, hypothesis, guardrail_advice)
            .with_kind(LearningKind::Guardrail)
            .with_outcome(ExperimentOutcome::Failed)
    }

    /// An invariant: "must always hold".
    pub fn invariant(
        context: impl Into<String>,
        hypothesis: impl Into<String>,
        guardrail_advice: impl Into<String>,
    ) -> Self {
        Self::new(context, hypothesis, guardrail_advice).with_kind(LearningKind::Invariant)
    }

    /// Builder-style: set the learning kind.
    pub fn with_kind(mut self, kind: LearningKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Builder-style: set the experiment outcome.
    pub fn with_outcome(mut self, outcome: ExperimentOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    /// Builder-style: attach affected element ids.
    pub fn with_elements(mut self, elements: Vec<String>) -> Self {
        self.affected_elements = elements;
        self
    }

    /// Builder-style: attach evidence references.
    pub fn with_evidence(mut self, refs: Vec<String>) -> Self {
        self.evidence_refs = refs;
        self
    }

    /// Builder-style: set tags.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Builder-style: set the run id.
    pub fn with_run_id(mut self, run_id: impl Into<String>) -> Self {
        self.run_id = Some(run_id.into());
        self
    }

    /// Builder-style: set the repo path.
    pub fn with_repo(mut self, repo: impl Into<String>) -> Self {
        self.repo = Some(repo.into());
        self
    }

    /// Builder-style: set the selector.
    pub fn with_selector(mut self, selector: impl Into<String>) -> Self {
        self.selector = Some(selector.into());
        self
    }

    /// Builder-style: set the HITL kind, validated via [`parse_hitl_kind`].
    pub fn with_hitl_kind(mut self, kind: &str) -> Result<Self, MemoryError> {
        self.hitl_kind = parse_hitl_kind(kind)?;
        Ok(self)
    }

    /// Builder-style: set confidence label.
    pub fn with_confidence(mut self, confidence: impl Into<String>) -> Self {
        self.confidence = Some(confidence.into());
        self
    }

    /// Builder-style: set the evolution category.
    pub fn with_category(mut self, category: LearningCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Builder-style: set signal patterns that trigger this learning.
    pub fn with_signals(mut self, signals: Vec<SignalPattern>) -> Self {
        self.signals_match = signals;
        self
    }

    /// Builder-style: add a single signal pattern.
    pub fn with_signal(mut self, signal: SignalPattern) -> Self {
        self.signals_match.push(signal);
        self
    }

    /// Builder-style: set constraints.
    pub fn with_constraints(mut self, constraints: LearningConstraints) -> Self {
        self.constraints = Some(constraints);
        self
    }

    /// Builder-style: add validation commands.
    pub fn with_validation(mut self, validation: Vec<String>) -> Self {
        self.validation = validation;
        self
    }

    /// Builder-style: set blast radius.
    pub fn with_blast_radius(mut self, blast_radius: BlastRadius) -> Self {
        self.blast_radius = Some(blast_radius);
        self
    }
}

/// Parse a human-in-the-loop kind string into a normalized value.
///
/// Accepts: `precedent`, `exception`, `correction`, `guardrail` (case-insensitive).
/// Empty string yields `None`. Anything else is an error.
pub fn parse_hitl_kind(s: &str) -> Result<Option<String>, MemoryError> {
    let trimmed = s.trim().to_lowercase();
    match trimmed.as_str() {
        "" => Ok(None),
        "precedent" | "exception" | "correction" | "guardrail" => Ok(Some(trimmed)),
        other => Err(MemoryError::InvalidIds(format!(
            "invalid hitl_kind '{other}': expected precedent|exception|correction|guardrail"
        ))),
    }
}
