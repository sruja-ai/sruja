//! Architecture Explainer + Memory: evidence-based answers, persistence, feedback loop.
//!
//! Pipeline: scan → context (graph + memory) → LLM → parse envelope → store facts/interactions.

mod commit_select;
mod context;
mod facts;
mod memory;
mod prompt;
mod schemas;

pub use commit_select::{score_commits, CommitCandidate};
pub use context::build_context;
pub use facts::{apply_verdict, should_deprecate, Fact, Verdict};
pub use memory::{
    append_fact, append_feedback, append_interaction, load_facts, load_feedback,
    load_interactions, load_state, save_state, write_facts,
};
pub use prompt::{explain_user_prompt, parse_envelope, EXPLAIN_SYSTEM};
pub use schemas::{EvidenceEntry, FeedbackRecord, InteractionRecord};
