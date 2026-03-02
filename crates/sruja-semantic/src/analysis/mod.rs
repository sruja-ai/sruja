//! Semantic analysis: coupling detection, reports.

mod coupling;
mod report;

pub use coupling::{
    SemanticCoupling, SemanticCouplingAnalyzer, SemanticCouplingReport, SemanticCouplingType,
    SemanticHub,
};
pub use report::{SemanticReport, SemanticSummary, VocabularyLeak};
