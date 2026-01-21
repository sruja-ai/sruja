//! JSON exporter implementation
//!
//! This module provides the Exporter struct that converts Sruja Program AST to JSON.

use std::collections::HashMap;
use std::time::SystemTime;

use sruja_language::{Program, collect_elements, build_qualified_id};

use crate::json::types::*;

/// Result type for export operations
pub type ExportResult<T> = Result<T, ExportError>;

/// Export error type
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Export failed: {0}")]
    Export(String),
}

/// JSON exporter for Sruja architectures
pub struct Exporter {
    /// Include computed views with layout
    pub extended: bool,
}

impl Exporter {
    /// Create a new exporter
    pub fn new() -> Self {
        Self { extended: false }
    }

    /// Create a new exporter with extended mode
    pub fn with_extended(extended: bool) -> Self {
        Self { extended }
    }

    /// Export a Program to JSON string with 2-space indentation
    ///
    /// Returns empty JSON "{}" for nil programs without error for backward compatibility.
    pub fn export(&self, program: &Program) -> ExportResult<String> {
        if program.items.is_empty() {
            return Ok("{}".to_string());
        }

        let dump = self.to_model_dump(program);
        let json = serde_json::to_string_pretty(&dump)?;
        Ok(json)
    }

    /// Export without indentation for smaller payload size
    ///
    /// Returns empty JSON "{}" for nil programs without error for backward compatibility.
    pub fn export_compact(&self, program: &Program) -> ExportResult<Vec<u8>> {
        if program.items.is_empty() {
            return Ok(b"{}".to_vec());
        }

        let dump = self.to_model_dump(program);
        let json = serde_json::to_vec(&dump)?;
        Ok(json)
    }

    /// Convert Program (AST) to SrujaModelDump
    ///
    /// Handles empty programs gracefully by returning an empty model dump.
    pub fn to_model_dump(&self, program: &Program) -> SrujaModelDump {
        let model_name = if program.items.is_empty() {
            "Untitled"
        } else {
            "Model"
        };

        let project_id = model_name.to_string();
        let project = ProjectDump {
            id: project_id.clone(),
            name: model_name.to_string(),
        };

        let estimated_capacity = program.items.len() * 2;
        let capacity = estimated_capacity.max(16);

        let mut dump = SrujaModelDump {
            stage: Some("parsed".to_string()),
            project_id: Some(project_id.clone()),
            project: Some(project),
            globals: Some(GlobalsDump {
                predicates: HashMap::new(),
                dynamic_predicates: HashMap::new(),
                styles: HashMap::new(),
            }),
            imports: HashMap::new(),
            deployments: Some(DeploymentsDump {
                elements: HashMap::new(),
                relations: HashMap::new(),
            }),
            specification: SpecificationDump::default(),
            elements: HashMap::with_capacity(capacity),
            relations: Vec::with_capacity(capacity / 2),
            views: HashMap::new(),
            sruja: Some(SrujaExtensions::default()),
            metadata: Some(ModelMetadata {
                name: model_name.to_string(),
                version: "1.0.0".to_string(),
                generated: timestamp(),
                sruja_version: "2.0.0".to_string(),
                layout_data: HashMap::new(),
            }),
        };

        // Convert elements and relations
        let (elements, relations) = collect_elements(program);
        self.convert_elements(&mut dump, &elements);
        self.convert_relations(&mut dump, &relations);

        // Convert views, scenarios, flows, requirements, ADRs, policies
        self.convert_views_from_program(&mut dump, program);
        self.build_sruja_extensions(&mut dump, program);

        dump
    }

    /// Convert elements to ElementDump
    fn convert_elements(&self, dump: &mut SrujaModelDump, elements: &HashMap<String, sruja_language::ElementDef>) {
        for (fqn, elem) in elements {
            let kind = elem.assignment.kind.to_string();
            let title = elem.assignment.name.clone();
            let description = self.extract_description(&elem);
            let technology = self.extract_technology(&elem);
            let tags = self.extract_tags(&elem);
            let metadata = self.extract_metadata(&elem);
            
            // Determine parent FQN
            let parent = if let Some(dot_idx) = fqn.rfind('.') {
                Some(fqn[..dot_idx].to_string())
            } else {
                None
            };

            let element_dump = ElementDump {
                id: fqn.clone(),
                kind,
                title,
                description,
                technology,
                tags,
                links: vec![], // TODO: Extract links from metadata
                metadata,
                style: None, // TODO: Extract style
                parent,
            };

            dump.elements.insert(fqn.clone(), element_dump);
        }
    }

    /// Convert relations to RelationDump
    fn convert_relations(&self, dump: &mut SrujaModelDump, relations: &[sruja_language::Relation]) {
        for (idx, rel) in relations.iter().enumerate() {
            let from_fqn = rel.from.as_string();
            let to_fqn = rel.to.as_string();
            
            let id = format!("rel_{}", idx);
            let title = rel.label.clone().or_else(|| rel.description.clone());
            let technology = rel.technology.clone();
            let tags = rel.tags.clone();

            let relation_dump = RelationDump {
                id,
                source: FqnRefDump::new(from_fqn),
                target: FqnRefDump::new(to_fqn),
                title,
                description: rel.description.clone(),
                technology,
                kind: None, // TODO: Extract kind if available
                tags,
                metadata: HashMap::new(), // TODO: Extract metadata
                color: None,
                line: None,
                head: None,
                tail: None,
            };

            dump.relations.push(relation_dump);
        }
    }

    /// Extract description from element body
    fn extract_description(&self, elem: &sruja_language::ElementDef) -> Option<String> {
        elem.assignment.body.as_ref()
            .and_then(|body| body.description.clone())
    }

    /// Extract technology from element body
    fn extract_technology(&self, elem: &sruja_language::ElementDef) -> Option<String> {
        elem.assignment.body.as_ref()
            .and_then(|body| body.technology.clone())
    }

    /// Extract tags from element body
    fn extract_tags(&self, elem: &sruja_language::ElementDef) -> Vec<String> {
        elem.assignment.tag_refs.clone()
    }

    /// Extract metadata from element body
    fn extract_metadata(&self, elem: &sruja_language::ElementDef) -> HashMap<String, String> {
        elem.assignment.body.as_ref()
            .map(|body| {
                body.metadata.iter().map(|e| {
                    (e.key.clone(), e.value.clone().unwrap_or_default())
                }).collect::<HashMap<_, _>>()
            })
            .unwrap_or_default()
    }

    /// Convert views from program
    fn convert_views_from_program(&self, dump: &mut SrujaModelDump, program: &Program) {
        // TODO: Implement view conversion
        // This will need to process ViewDef items from the program
    }

    /// Build Sruja extensions (scenarios, flows, requirements, ADRs, policies)
    fn build_sruja_extensions(&self, dump: &mut SrujaModelDump, program: &Program) {
        let mut extensions = SrujaExtensions::default();

        for item in &program.items {
            match item {
                sruja_language::TopLevelItem::Scenario(scenario) => {
                    extensions.scenarios.push(ScenarioDump {
                        id: scenario.id.clone(),
                        title: scenario.title.clone(),
                        description: scenario.description.clone(),
                        steps: scenario.steps.iter().enumerate().map(|(idx, step)| {
                            StepDump {
                                id: Some(format!("step_{}", idx)),
                                description: step.description.clone().unwrap_or_default(),
                                from: step.from.as_ref().map(|q| q.as_string()),
                                to: step.to.as_ref().map(|q| q.as_string()),
                            }
                        }).collect(),
                    });
                }
                sruja_language::TopLevelItem::Flow(flow) => {
                    extensions.flows.push(FlowDump {
                        id: flow.id.clone(),
                        title: flow.title.clone(),
                        description: flow.description.clone(),
                        steps: flow.steps.iter().enumerate().map(|(idx, step)| {
                            StepDump {
                                id: Some(format!("step_{}", idx)),
                                description: step.description.clone().unwrap_or_default(),
                                from: step.from.as_ref().map(|q| q.as_string()),
                                to: step.to.as_ref().map(|q| q.as_string()),
                            }
                        }).collect(),
                    });
                }
                sruja_language::TopLevelItem::Requirement(req) => {
                    extensions.requirements.push(RequirementDump {
                        id: req.id.clone(),
                        title: req.title.clone(),
                        r#type: Some(req.r#type.clone()),
                        description: req.description.clone(),
                        priority: None,
                        status: None,
                        elements: vec![],
                    });
                }
                sruja_language::TopLevelItem::Adr(adr) => {
                    extensions.adrs.push(AdrDump {
                        id: adr.id.clone(),
                        title: adr.title.clone(),
                        status: adr.status.clone(),
                        context: adr.context.clone(),
                        decision: adr.decision.clone(),
                        consequences: adr.consequences.clone(),
                        date: None,
                        author: None,
                    });
                }
                sruja_language::TopLevelItem::Policy(policy) => {
                    extensions.policies.push(PolicyDump {
                        id: policy.id.clone(),
                        title: policy.title.clone(),
                        category: Some(policy.category.clone()),
                        enforcement: Some(policy.enforcement.clone()),
                        description: policy.description.clone(),
                        elements: vec![],
                    });
                }
                sruja_language::TopLevelItem::Import(import) => {
                    extensions.imports.push(ImportDump {
                        elements: import.elements.iter().map(|e| match e {
                            sruja_language::ImportElement::Ident(ident) => ident.clone(),
                            sruja_language::ImportElement::Wildcard => "*".to_string(),
                        }).collect(),
                        from: import.from.clone(),
                    });
                }
                _ => {}
            }
        }

        if !extensions.scenarios.is_empty() 
            || !extensions.flows.is_empty()
            || !extensions.requirements.is_empty()
            || !extensions.adrs.is_empty()
            || !extensions.policies.is_empty()
            || !extensions.imports.is_empty() {
            dump.sruja = Some(extensions);
        }
    }
}

impl Default for Exporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate ISO 8601 timestamp
fn timestamp() -> String {
    // Simple RFC3339 format - in production, use chrono crate
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| {
            // For now, return a placeholder. In production, use chrono::DateTime::from()
            // to format as RFC3339 properly
            format!("1970-01-01T00:00:00Z")
        })
        .unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z"))
}
