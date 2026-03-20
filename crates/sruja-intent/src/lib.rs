//! Intent vs Reality Comparison
//!
//! Compares declared architectural intent (ADRs, .sruja files, design docs) against
//! actual implementation to detect boundary drift, intent violations, and
//! undocumented architectural changes.

pub mod compare;
pub mod model;
pub mod parser;
pub mod report;

pub use compare::{Drift, DriftDetector, DriftHealth, DriftKind, DriftReport, Severity};
pub use model::{
    DeclaredBoundary, DeclaredComponent, DeclaredConstraint, DeclaredPolicy, DeclaredRelationship,
    IntentModel, IntentSourceInfo, SourceReference,
};
pub use parser::{AdrParser, AdrStatus, ParsedAdr};
pub use report::{IntentReport, IntentViolation};

use std::path::Path;

pub struct IntentContext {
    models: Vec<IntentModel>,
}

impl IntentContext {
    pub fn new() -> Self {
        Self { models: Vec::new() }
    }

    pub fn load_from_directory(&mut self, dir: &Path) -> Result<Vec<IntentModel>, IntentError> {
        let mut models = Vec::new();

        let adr_dir = dir.join("adr").join("decisions");
        if adr_dir.exists() {
            let parser = AdrParser::new();
            let adrs = parser.parse_dir(&adr_dir)?;
            for adr in adrs {
                models.push(IntentModel::from_adr(adr));
            }
        }

        for entry in walkdir::WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "sruja") {
                if let Ok(model) = IntentModel::from_sruja_file(path) {
                    models.push(model);
                }
            }
        }

        self.models.extend(models.clone());
        Ok(models)
    }

    pub fn models(&self) -> &[IntentModel] {
        &self.models
    }
}

impl Default for IntentContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IntentError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid ADR format: {0}")]
    InvalidAdr(String),

    #[error("DSL error: {0}")]
    Dsl(String),
}
