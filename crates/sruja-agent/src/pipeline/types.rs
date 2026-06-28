use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Identifies a role in the pipeline. Used for prompt loading, model routing,
/// and lesson scoping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineRole {
    Analyzer,
    Prober,
    Confirmer,
    Fixer,
    Auditor,
    ReTester,
    Judge,
}

impl std::fmt::Display for PipelineRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Analyzer => write!(f, "analyzer"),
            Self::Prober => write!(f, "prober"),
            Self::Confirmer => write!(f, "confirmer"),
            Self::Fixer => write!(f, "fixer"),
            Self::Auditor => write!(f, "auditor"),
            Self::ReTester => write!(f, "retester"),
            Self::Judge => write!(f, "judge"),
        }
    }
}

impl PipelineRole {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "analyzer" | "self_review" | "analyzer_self_review" => Some(Self::Analyzer),
            "prober" => Some(Self::Prober),
            "confirmer" => Some(Self::Confirmer),
            "fixer" => Some(Self::Fixer),
            "auditor" => Some(Self::Auditor),
            "retester" => Some(Self::ReTester),
            "judge" => Some(Self::Judge),
            _ => None,
        }
    }
}

/// A stage definition loaded from `.sruja/pipeline.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageDef {
    pub id: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub parallel: bool,
    pub model: String,
    #[serde(default)]
    pub prompt_file: Option<String>,
    #[serde(default)]
    pub phase_1_verify: bool,
}

fn default_enabled() -> bool {
    true
}

// ---------------------------------------------------------------------------
// Artifacts
// ---------------------------------------------------------------------------

/// A gap between intended design and actual implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gap {
    pub id: String,
    pub area: String,
    pub description: String,
    pub severity: String,
    pub evidence: Vec<String>,
    #[serde(default = "default_substantiated")]
    pub substantiated: bool,
}

fn default_substantiated() -> bool {
    true
}

/// Report from the Analyzer stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapReport {
    pub gaps: Vec<Gap>,
    pub summary: String,
    pub cycle: usize,
}

/// A bug or test case gap found by the Prober.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bug {
    pub id: String,
    pub gap_id: String,
    pub area: String,
    pub description: String,
    pub severity: String,
    pub test_case: String,
    pub evidence: Vec<String>,
}

/// Report from the Prober (test cases).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugReport {
    pub bugs: Vec<Bug>,
    pub summary: String,
    pub cycle: usize,
}

/// Fix status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FixStatus {
    Resolved,
    Failed,
    Blocked,
}

/// Result from the Fixer stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixReport {
    pub bug_id: String,
    pub status: FixStatus,
    pub fix_description: String,
    pub modified_files: Vec<String>,
    pub verify_output: Vec<String>,
    pub root_cause: String,
}

/// Review verdict.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditVerdict {
    Approved,
    RequestChanges,
    Rejected,
}

/// Result from the Auditor stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResult {
    pub fix_index: usize,
    pub bug_id: String,
    pub verdict: AuditVerdict,
    pub issues: Vec<String>,
    pub approves: bool,
}

/// Re-test verdict.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetestVerdict {
    Resolved,
    Incomplete,
    Regression,
}

/// Result from the ReTester stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetestResult {
    pub bug_id: String,
    pub verdict: RetestVerdict,
    pub details: String,
    pub tester_role: PipelineRole,
}

/// Verify step report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyReport {
    pub all_passed: bool,
    pub failures: Vec<String>,
    pub details: String,
}

/// Scorecard from the Judge stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scorecard {
    pub functional_correctness: u8,
    pub code_quality: u8,
    pub test_coverage: u8,
    pub ux_quality: u8,
    pub cost_efficiency: u8,
    pub evidence: Vec<String>,
    pub total: f64,
    pub summary: String,
    pub improved_from_previous: bool,
}

impl Scorecard {
    pub fn zero() -> Self {
        Self {
            functional_correctness: 0,
            code_quality: 0,
            test_coverage: 0,
            ux_quality: 0,
            cost_efficiency: 0,
            evidence: vec![],
            total: 0.0,
            summary: String::new(),
            improved_from_previous: false,
        }
    }
}

/// A lesson recorded when a reviewer rejects work.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub role: PipelineRole,
    pub cycle: usize,
    pub what_wrong: String,
    pub correction: String,
}

// ---------------------------------------------------------------------------
// Envelope types
// ---------------------------------------------------------------------------

/// Artifact passed between pipeline stages.
#[derive(Debug, Clone)]
pub enum PipelineArtifact {
    GapReport(GapReport),
    BugReport(BugReport),
    FixReport(FixReport),
    AuditResult(AuditResult),
    RetestResult(RetestResult),
    VerifyReport(VerifyReport),
    Scorecard(Scorecard),
}

/// Result of a single pipeline stage execution.
#[derive(Debug, Clone)]
pub struct StageResult {
    pub stage_id: String,
    pub role: PipelineRole,
    pub success: bool,
    pub artifact: Option<PipelineArtifact>,
    pub duration: Duration,
    pub errors: Vec<String>,
}

/// Final result of a full pipeline run.
#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub scorecard: Option<Scorecard>,
    pub stages: Vec<StageResult>,
    pub cycles: usize,
    pub converged: bool,
    pub reason: String,
    pub lessons_recorded: usize,
}

/// Errors from pipeline execution.
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("agent error: {0}")]
    Agent(#[from] crate::AgentError),
    #[error("manifest error: {0}")]
    Manifest(String),
    #[error("no enabled stages in pipeline config")]
    NoStages,
    #[error("stage {stage} failed: {message}")]
    Stage {
        stage: String,
        message: String,
    },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}
