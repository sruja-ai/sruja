//! Configurable multi-agent role pipeline.
//!
//! Drives a sequence of role-specialized agent stages — analyzer, prober,
//! confirmer, fixer, auditor, judge — that pass structured artifacts to each
//! other. Every role, model, budget, and verify step is configured in
//! `.sruja/pipeline.toml` — nothing is hardcoded.
//!
//! ## Architecture
//!
//! Each stage runs a fresh [`crate::cognition::Agent::run_loop`] with a
//! role-specific system prompt (loaded from `.sruja/agents/*.md` or built-in
//! defaults). Stages marked as `parallel` spawn two agents on different models
//! simultaneously (e.g. Prober-A on ZAI GLM-5.1, Prober-B on Xiaomi Mimo)
//! to catch model-specific blind spots and avoid rate-limit contention.
//!
//! The pipeline terminates when:
//! - Score threshold met with zero critical/high open bugs (converged)
//! - Score plateaus across cycles (no improvement)
//! - Budget exhausted (max cycles or per-stage limits)

pub mod area;
pub mod budget;
pub mod config;
pub mod lessons;
pub mod live_report;
pub mod orchestrator;
pub mod types;

mod stages;

pub use area::AreaPartitioner;
pub use budget::{BudgetTracker, ConvergenceResult, PipelineBudgets};
pub use config::PipelineManifest;
pub use lessons::LessonStore;
pub use live_report::LiveReport;
pub use orchestrator::PipelineOrchestrator;
pub use types::{
    AuditResult, AuditVerdict, Bug, BugReport, FixReport, FixStatus, Gap, GapReport, Lesson,
    PipelineArtifact, PipelineError, PipelineResult, PipelineRole, Scorecard, StageDef,
    StageResult, VerifyReport,
};
