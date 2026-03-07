//! Intent Model Types
//!
//! Normalized representation of architectural intent from various sources
//! (ADRs, .sruja files, design docs).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::parser::ParsedAdr;
use crate::IntentError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentSourceType {
    SrujaFile,
    AdrFile,
    DesignDoc,
    CodeComments,
    Manual,
}

#[derive(Debug, Clone)]
pub struct IntentModel {
    pub source: IntentSourceInfo,
    pub components: Vec<DeclaredComponent>,
    pub relationships: Vec<DeclaredRelationship>,
    pub boundaries: Vec<DeclaredBoundary>,
    pub policies: Vec<DeclaredPolicy>,
    pub constraints: Vec<DeclaredConstraint>,
}

#[derive(Debug, Clone)]
pub struct IntentSourceInfo {
    pub source_type: IntentSourceType,
    pub path: PathBuf,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclaredComponent {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub source_ref: SourceReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclaredRelationship {
    pub source: String,
    pub target: String,
    pub kind: String,
    pub label: Option<String>,
    pub source_ref: SourceReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclaredBoundary {
    pub name: String,
    pub inside: Vec<String>,
    pub allowed_connections: Vec<AllowedConnection>,
    pub rules: Vec<BoundaryRule>,
    pub source_ref: SourceReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowedConnection {
    pub target_boundary: String,
    pub via: ConnectionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    ApiCall,
    EventBus,
    Database,
    DirectCall,
    MessageQueue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryRule {
    pub rule_type: BoundaryRuleType,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryRuleType {
    NoDirectDatabaseAccess,
    ApiOnly,
    EventBusOnly,
    NoSharedState,
    SingleResponsibility,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclaredPolicy {
    pub name: String,
    pub description: String,
    pub scope: Vec<String>,
    pub rules: Vec<PolicyRule>,
    pub source_ref: SourceReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub description: String,
    pub constraint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclaredConstraint {
    pub name: String,
    pub description: String,
    pub applies_to: Vec<String>,
    pub constraint_type: ConstraintType,
    pub source_ref: SourceReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    MustUse { technology: String },
    MustNotUse { technology: String },
    MustImplement { interface: String },
    MaxDependencyCount { count: usize },
    MinTestCoverage { percent: f32 },
    Custom { rule: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceReference {
    pub file: String,
    pub line: Option<u32>,
    pub element: Option<String>,
}

impl IntentModel {
    pub fn new(source: IntentSourceInfo) -> Self {
        Self {
            source,
            components: Vec::new(),
            relationships: Vec::new(),
            boundaries: Vec::new(),
            policies: Vec::new(),
            constraints: Vec::new(),
        }
    }

    pub fn from_adr(adr: ParsedAdr) -> Self {
        let source = IntentSourceInfo {
            source_type: IntentSourceType::AdrFile,
            path: adr.path.clone(),
            name: adr.title.clone(),
        };

        let mut model = Self::new(source);

        for impl_text in &adr.implications {
            if let Some(ref component) = impl_text.component {
                model.components.push(DeclaredComponent {
                    id: component.clone(),
                    kind: "service".to_string(),
                    label: component.clone(),
                    description: Some(impl_text.raw_text.clone()),
                    technology: None,
                    source_ref: SourceReference {
                        file: adr.path.to_string_lossy().to_string(),
                        line: None,
                        element: Some(component.clone()),
                    },
                });
            }

            if let Some(ref policy) = impl_text.new_policy {
                model.policies.push(DeclaredPolicy {
                    name: format!("Policy from {}", adr.title),
                    description: policy.clone(),
                    scope: vec![],
                    rules: vec![],
                    source_ref: SourceReference {
                        file: adr.path.to_string_lossy().to_string(),
                        line: None,
                        element: None,
                    },
                });
            }
        }

        model
    }

    pub fn from_sruja_file(path: &Path) -> Result<Self, IntentError> {
        let content = std::fs::read_to_string(path).map_err(IntentError::Io)?;

        let parser = sruja_language::Parser::new(path.to_string_lossy().to_string());
        let program = parser
            .parse(&content)
            .map_err(|e| IntentError::Dsl(format!("{:?}", e)))?;

        let source = IntentSourceInfo {
            source_type: IntentSourceType::SrujaFile,
            path: path.to_path_buf(),
            name: path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string()),
        };

        let mut model = Self::new(source);

        let (elements, _) = sruja_language::collect_elements(&program);

        for (fqn, element) in &elements {
            let assignment = &element.assignment;
            let label = assignment
                .title
                .clone()
                .unwrap_or_else(|| assignment.name.clone());

            let description = assignment.body.as_ref().and_then(|b| {
                b.items.iter().find_map(|item| {
                    if let sruja_language::ElementDefBodyItem::Description(d) = item {
                        Some(d.clone())
                    } else {
                        None
                    }
                })
            });

            let technology = assignment.body.as_ref().and_then(|b| {
                b.items.iter().find_map(|item| {
                    if let sruja_language::ElementDefBodyItem::Technology(t) = item {
                        Some(t.clone())
                    } else {
                        None
                    }
                })
            });

            model.components.push(DeclaredComponent {
                id: fqn.clone(),
                kind: format!("{:?}", assignment.kind).to_lowercase(),
                label,
                description,
                technology,
                source_ref: SourceReference {
                    file: path.to_string_lossy().to_string(),
                    line: Some(element.location.line),
                    element: Some(fqn.clone()),
                },
            });
        }

        let (_, relations) = sruja_language::collect_elements(&program);
        for rel in &relations {
            model.relationships.push(DeclaredRelationship {
                source: rel.from.as_string(),
                target: rel.to.as_string(),
                kind: "depends_on".to_string(),
                label: rel.label.clone(),
                source_ref: SourceReference {
                    file: path.to_string_lossy().to_string(),
                    line: None,
                    element: None,
                },
            });
        }

        Ok(model)
    }

    pub fn merge(&mut self, other: IntentModel) {
        self.components.extend(other.components);
        self.relationships.extend(other.relationships);
        self.boundaries.extend(other.boundaries);
        self.policies.extend(other.policies);
        self.constraints.extend(other.constraints);
    }

    pub fn component_ids(&self) -> Vec<&str> {
        self.components.iter().map(|c| c.id.as_str()).collect()
    }

    pub fn relationship_pairs(&self) -> Vec<(&str, &str)> {
        self.relationships
            .iter()
            .map(|r| (r.source.as_str(), r.target.as_str()))
            .collect()
    }

    pub fn find_component(&self, id: &str) -> Option<&DeclaredComponent> {
        self.components.iter().find(|c| c.id == id)
    }
}

impl Default for IntentModel {
    fn default() -> Self {
        Self::new(IntentSourceInfo {
            source_type: IntentSourceType::Manual,
            path: PathBuf::new(),
            name: "Default".to_string(),
        })
    }
}
