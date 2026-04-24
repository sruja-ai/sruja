//! Intent vs Reality Comparison
//!
//! Compares declared architectural intent (ADRs, .sruja files, design docs) against
//! actual implementation to detect boundary drift, intent violations, and
//! undocumented architectural changes.

pub mod compare;
pub mod model;
pub mod parser;
pub mod report;
pub mod critique;
pub mod behavioral_drift;
pub mod critique_report;

pub use compare::{Drift, DriftDetector, DriftHealth, DriftKind, DriftReport, Evidence, Severity};
pub use model::{
    BoundaryRule, BoundaryRuleType, DeclaredBoundary, DeclaredComponent, DeclaredConstraint,
    DeclaredPolicy, DeclaredRelationship, IntentModel, IntentSourceInfo, SourceReference,
};
pub use parser::{AdrParser, AdrStatus, ParsedAdr};
pub use report::{IntentReport, IntentViolation};
pub use critique::{
    CritiqueCategory, CritiqueEngine, CritiqueFinding, CritiqueReport, CritiqueRequest,
    CritiqueSeverity, RiskLevel,
};
pub use critique_report::{format_critique_json, format_critique_text};

use std::path::Path;

pub struct IntentContext {
    models: Vec<IntentModel>,
    schema: sruja_language::DomainSchema,
}

impl IntentContext {
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            schema: sruja_language::DomainSchema::architecture(),
        }
    }

    pub fn schema(&self) -> &sruja_language::DomainSchema {
        &self.schema
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
                let content = std::fs::read_to_string(path).map_err(IntentError::Io)?;
                let parser = sruja_language::Parser::new(path.to_string_lossy().to_string());
                if let Ok(program) = parser.parse(&content) {
                    // Extract schema if present
                    for item in &program.items {
                        if let sruja_language::TopLevelItem::Schema(s) = item {
                            self.schema = sruja_language::DomainSchema::from_ast(s);
                        }
                    }

                    if let Ok(model) = IntentModel::from_sruja_content(&content, path) {
                        models.push(model);
                    }
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
