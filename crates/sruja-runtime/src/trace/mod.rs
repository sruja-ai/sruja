//! Trace types and processing for runtime analysis.

mod processor;

pub use processor::{ExecutionEdge, ExecutionGraph, ExecutionGraphProcessor};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single execution trace (span) from runtime instrumentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub id: String,
    pub name: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub attributes: Vec<TraceAttribute>,
    pub children: Vec<ExecutionTrace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAttribute {
    pub key: String,
    pub value: TraceValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum TraceValue {
    String(String),
    Int(i64),
    Double(f64),
    Bool(bool),
}
