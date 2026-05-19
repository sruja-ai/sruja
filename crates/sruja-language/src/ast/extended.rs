//! Extended AST types: SLO, feedback/causal loops, fitness, incidents.

use sruja_diagnostics::SourceLocation;

use super::{Program, QualifiedIdent, Relation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SloBlock {
    pub location: SourceLocation,
    pub availability: Option<SloAvailability>,
    pub latency: Option<SloLatency>,
    pub error_rate: Option<SloErrorRate>,
    pub throughput: Option<SloThroughput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SloAvailability {
    pub target: Option<String>,
    pub window: Option<String>,
    pub current: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SloLatency {
    pub p95: Option<String>,
    pub p99: Option<String>,
    pub window: Option<String>,
    pub current: Option<SloCurrent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SloCurrent {
    pub p95: Option<String>,
    pub p99: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SloErrorRate {
    pub target: Option<String>,
    pub window: Option<String>,
    pub current: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SloThroughput {
    pub target: Option<String>,
    pub window: Option<String>,
    pub current: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackLoopType {
    Reinforcing,
    Balancing,
}

impl FeedbackLoopType {
    pub fn to_symbol(&self) -> &str {
        match self {
            FeedbackLoopType::Reinforcing => "+",
            FeedbackLoopType::Balancing => "-",
        }
    }
}

impl std::fmt::Display for FeedbackLoopType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeedbackLoopType::Reinforcing => write!(f, "reinforcing"),
            FeedbackLoopType::Balancing => write!(f, "balancing"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CausalPolarity {
    Positive,
    Negative,
}

impl CausalPolarity {
    pub fn to_symbol(&self) -> &str {
        match self {
            CausalPolarity::Positive => "+",
            CausalPolarity::Negative => "-",
        }
    }
}

impl std::fmt::Display for CausalPolarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CausalPolarity::Positive => write!(f, "+"),
            CausalPolarity::Negative => write!(f, "-"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackLoop {
    pub location: SourceLocation,
    pub id: String,
    pub loop_type: FeedbackLoopType,
    pub loop_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub relationships: Vec<Relation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalLoopVariable {
    pub id: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalRelationship {
    pub from: String,
    pub to: String,
    pub effect: Option<String>,
    pub polarity: CausalPolarity,
    pub delay: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalLoop {
    pub location: SourceLocation,
    pub id: String,
    pub loop_type: FeedbackLoopType,
    pub loop_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub variables: Vec<CausalLoopVariable>,
    pub relationships: Vec<CausalRelationship>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FitnessDef {
    pub location: SourceLocation,
    pub id: String,
    pub target: String,
    pub measure: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalParseResult {
    pub updated_ast: Program,
    pub changed_elements: Vec<String>,
    pub changed_ranges: Vec<(usize, usize)>,
    pub parsing_time_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Incident {
    pub location: SourceLocation,
    pub id: String,
    pub title: String,
    pub date: Option<String>,
    pub severity: Option<String>,
    pub affected: Vec<QualifiedIdent>,
    pub cause: Option<String>,
    pub resolution: Option<String>,
    pub lesson: Option<String>,
}
