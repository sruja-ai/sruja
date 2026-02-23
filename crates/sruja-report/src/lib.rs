//! Canonical report schema for Sruja architecture intelligence.
//!
//! This crate defines the shared report types (ComprehensiveReport, Recommendation,
//! and layer sections) so CLI and MCP emit the same JSON shape. It is DTO-only;
//! callers build these from sruja-diff, sruja-semantic, sruja-intent, and
//! sruja-runtime results.

pub mod comprehensive;

pub use comprehensive::{
    build_recommendations, ComprehensiveReport, Effort, IntentSection, Layer, Priority,
    Recommendation, RecommendationCategory, RuntimeSection, SemanticSection, StructuralSection,
};
