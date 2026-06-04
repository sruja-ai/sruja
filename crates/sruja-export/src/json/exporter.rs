//! JSON exporter implementation
//!
//! This module provides the Exporter struct that converts Sruja Program AST to JSON.

use std::collections::HashMap;

use sruja_language::{collect_elements, Program};

use crate::json::types::*;
use crate::mermaid::views::resolve_view;

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

#[derive(Debug, Clone)]
struct ArchitectureIndexFields {
    canonical_id: Option<String>,
    aliases: Vec<String>,
    owner: Option<String>,
    domain: Option<String>,
    criticality: Option<String>,
    sources: Vec<SourceBindingDump>,
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
                sruja_version: env!("CARGO_PKG_VERSION").to_string(),
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
    fn convert_elements(
        &self,
        dump: &mut SrujaModelDump,
        elements: &HashMap<String, sruja_language::ElementDef>,
    ) {
        for (fqn, elem) in elements {
            let kind = elem.assignment.kind.to_string();
            let title = elem
                .assignment
                .title
                .clone()
                .unwrap_or_else(|| elem.assignment.name.clone());
            let description = self.extract_description(elem);
            let technology = self.extract_technology(elem);
            let doc = self.extract_doc(elem);
            let knowledge = self.extract_knowledge(elem);
            let tags = self.extract_tags(elem);
            let metadata = self.extract_metadata(elem);

            // Determine parent FQN
            let parent = fqn.rfind('.').map(|dot_idx| fqn[..dot_idx].to_string());

            // Extract architecture index fields
            let architecture_index = self.extract_architecture_index_fields(elem);

            let element_dump = ElementDump {
                id: fqn.clone(),
                kind,
                title,
                description,
                technology,
                doc,
                knowledge,
                tags,
                links: vec![],
                metadata,
                style: None,
                parent,
                canonical_id: architecture_index.canonical_id,
                aliases: architecture_index.aliases,
                owner: architecture_index.owner,
                domain: architecture_index.domain,
                criticality: architecture_index.criticality,
                sources: architecture_index.sources,
                hydration: None,
            };

            dump.elements.insert(fqn.clone(), element_dump);
        }
    }

    /// Extract architecture index fields from element body
    fn extract_architecture_index_fields(
        &self,
        elem: &sruja_language::ElementDef,
    ) -> ArchitectureIndexFields {
        elem.assignment
            .body
            .as_ref()
            .map(|body| ArchitectureIndexFields {
                canonical_id: body.canonical_id.clone(),
                aliases: body.aliases.clone(),
                owner: body.owner.clone(),
                domain: body.domain.clone(),
                criticality: body.criticality.map(|c| c.as_str().to_string()),
                sources: body
                    .sources
                    .iter()
                    .map(|s| SourceBindingDump {
                        kind: s.kind.as_str().to_string(),
                        path: s.path.clone(),
                        description: s.description.clone(),
                    })
                    .collect(),
            })
            .unwrap_or(ArchitectureIndexFields {
                canonical_id: None,
                aliases: vec![],
                owner: None,
                domain: None,
                criticality: None,
                sources: vec![],
            })
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
                kind: None,
                tags,
                metadata: HashMap::new(),
                color: None,
                line: None,
                head: None,
                tail: None,
                confidence: None,
            };

            dump.relations.push(relation_dump);
        }
    }

    /// Extract description from element body
    fn extract_description(&self, elem: &sruja_language::ElementDef) -> Option<String> {
        elem.assignment
            .body
            .as_ref()
            .and_then(|body| body.description.clone())
    }

    /// Extract technology from element body
    fn extract_technology(&self, elem: &sruja_language::ElementDef) -> Option<String> {
        elem.assignment
            .body
            .as_ref()
            .and_then(|body| body.technology.clone())
    }

    /// Extract doc path (component knowledge file) from element body
    fn extract_doc(&self, elem: &sruja_language::ElementDef) -> Option<String> {
        elem.assignment
            .body
            .as_ref()
            .and_then(|body| body.doc.clone())
    }

    /// Extract knowledge graph ID from element body
    fn extract_knowledge(&self, elem: &sruja_language::ElementDef) -> Option<String> {
        elem.assignment
            .body
            .as_ref()
            .and_then(|body| body.knowledge.clone())
    }

    /// Extract tags from element body
    fn extract_tags(&self, elem: &sruja_language::ElementDef) -> Vec<String> {
        elem.assignment.tag_refs.clone()
    }

    /// Extract metadata from element body
    fn extract_metadata(&self, elem: &sruja_language::ElementDef) -> HashMap<String, String> {
        elem.assignment
            .body
            .as_ref()
            .map(|body| {
                body.metadata
                    .iter()
                    .map(|e| (e.key.clone(), e.value.clone().unwrap_or_default()))
                    .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default()
    }

    /// Convert views from program
    fn convert_views_from_program(&self, dump: &mut SrujaModelDump, program: &Program) {
        let (elements, relations) = collect_elements(program);
        let mut view_items: Vec<&sruja_language::ViewDef> = vec![];
        for item in &program.items {
            if let sruja_language::TopLevelItem::View(view) = item {
                view_items.push(view);
            }
        }
        if view_items.is_empty() {
            return;
        }

        fn to_rule_expr(expr: &sruja_language::ViewRuleExpr) -> ViewRuleExpr {
            ViewRuleExpr {
                wildcard: expr.wildcard,
                recursive: expr.recursive,
                elements: expr.elements.clone(),
            }
        }

        fn to_rule(rule: &sruja_language::ViewRule) -> ViewRule {
            ViewRule {
                include: rule.include.as_ref().map(to_rule_expr),
                exclude: rule.exclude.as_ref().map(to_rule_expr),
            }
        }

        for view in view_items {
            let resolved = resolve_view(view, &elements, &relations);

            let mut element_ids: Vec<String> = resolved.elements.keys().cloned().collect();
            element_ids.sort();

            let columns = 4usize;
            let x_step = 220.0f64;
            let y_step = 140.0f64;
            let width = 180.0f64;
            let height = 80.0f64;

            let mut nodes_by_id: HashMap<String, NodeDump> = HashMap::new();
            for (idx, element_id) in element_ids.iter().enumerate() {
                let row = (idx / columns) as f64;
                let col = (idx % columns) as f64;
                let x = col * x_step;
                let y = row * y_step;

                let parent = element_id.rfind('.').and_then(|dot| {
                    let p = &element_id[..dot];
                    resolved.elements.contains_key(p).then(|| p.to_string())
                });
                let title = resolved
                    .elements
                    .get(element_id)
                    .and_then(|e| e.assignment.title.clone())
                    .or_else(|| {
                        resolved
                            .elements
                            .get(element_id)
                            .map(|e| e.assignment.name.clone())
                    });

                nodes_by_id.insert(
                    element_id.clone(),
                    NodeDump {
                        id: element_id.clone(),
                        element: element_id.clone(),
                        parent,
                        title,
                        x,
                        y,
                        width,
                        height,
                    },
                );
            }

            let mut edges: Vec<EdgeDump> = vec![];
            for (idx, rel) in resolved.relations.iter().enumerate() {
                let source = rel.from.as_string();
                let target = rel.to.as_string();
                if !nodes_by_id.contains_key(&source) || !nodes_by_id.contains_key(&target) {
                    continue;
                }
                edges.push(EdgeDump {
                    id: format!("e{}", idx),
                    source,
                    target,
                    relation: None,
                    title: rel.label.clone(),
                });
            }

            let mut nodes: Vec<NodeDump> = nodes_by_id.into_values().collect();
            nodes.sort_by(|a, b| a.id.cmp(&b.id));

            dump.views.insert(
                view.id.clone(),
                ViewDump {
                    id: view.id.clone(),
                    title: Some(view.title.clone()).filter(|t| !t.is_empty()),
                    description: view.description.clone(),
                    view_of: view.view_of.as_ref().map(|q| q.as_string()),
                    tags: view.tags.clone(),
                    rules: view.rules.iter().map(to_rule).collect(),
                    nodes,
                    edges,
                    layout: None,
                },
            );
        }
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
                        steps: scenario
                            .steps
                            .iter()
                            .enumerate()
                            .map(|(idx, step)| StepDump {
                                id: Some(format!("step_{}", idx)),
                                description: step.description.clone().unwrap_or_default(),
                                from: step.from.as_ref().map(|q| q.as_string()),
                                to: step.to.as_ref().map(|q| q.as_string()),
                            })
                            .collect(),
                    });
                }
                sruja_language::TopLevelItem::Flow(flow) => {
                    extensions.flows.push(FlowDump {
                        id: flow.id.clone(),
                        title: flow.title.clone(),
                        description: flow.description.clone(),
                        steps: flow
                            .steps
                            .iter()
                            .enumerate()
                            .map(|(idx, step)| StepDump {
                                id: Some(format!("step_{}", idx)),
                                description: step.description.clone().unwrap_or_default(),
                                from: step.from.as_ref().map(|q| q.as_string()),
                                to: step.to.as_ref().map(|q| q.as_string()),
                            })
                            .collect(),
                    });
                }
                sruja_language::TopLevelItem::Requirement(req) => {
                    extensions.requirements.push(RequirementDump {
                        id: req.id.clone(),
                        title: req.title.clone(),
                        r#type: Some(req.r#type.clone()),
                        description: req.description.clone(),
                        priority: req.priority.clone(),
                        status: req.status.clone(),
                        elements: vec![],
                        acceptance_criteria: req
                            .acceptance_criteria
                            .iter()
                            .map(|ac| AcceptanceCriteriaDump {
                                given: ac.given.clone(),
                                when: ac.when.clone(),
                                then: ac.then.clone(),
                            })
                            .collect(),
                        user_journey: req.user_journey.clone(),
                        scenarios: req.scenarios.clone(),
                        adrs: req.adrs.clone(),
                        affects: req.affects.clone(),
                        source: req.source.clone(),
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
                        elements: import
                            .elements
                            .iter()
                            .map(|e| match e {
                                sruja_language::ImportElement::Ident(ident) => ident.clone(),
                                sruja_language::ImportElement::Wildcard => "*".to_string(),
                            })
                            .collect(),
                        from: import.from.clone(),
                    });
                }
                sruja_language::TopLevelItem::Schema(_) => {}
                _ => {}
            }
        }

        if !extensions.scenarios.is_empty()
            || !extensions.flows.is_empty()
            || !extensions.requirements.is_empty()
            || !extensions.adrs.is_empty()
            || !extensions.policies.is_empty()
            || !extensions.imports.is_empty()
        {
            dump.sruja = Some(extensions);
        }
    }
}

impl Default for Exporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate ISO 8601 timestamp in RFC3339 format
#[cfg(target_arch = "wasm32")]
fn timestamp() -> String {
    let date = js_sys::Date::new_0();
    date.to_iso_string()
        .as_string()
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339())
}

/// Generate ISO 8601 timestamp in RFC3339 format
#[cfg(not(target_arch = "wasm32"))]
fn timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_language::Parser;

    fn parse(input: &str) -> Program {
        Parser::new("test.sruja".to_string())
            .parse(input)
            .expect("parse failed")
    }

    #[test]
    fn export_empty_program_returns_empty_json_object() {
        let program = Program::default();
        let exporter = Exporter::new();
        let json = exporter.export(&program).expect("export");
        assert_eq!(json, "{}");
    }

    #[test]
    fn export_compact_empty_program_returns_empty_object_bytes() {
        let program = Program::default();
        let exporter = Exporter::new();
        let bytes = exporter.export_compact(&program).expect("export_compact");
        assert_eq!(bytes, b"{}");
    }

    #[test]
    fn export_compact_non_empty_program_returns_valid_json() {
        let program = parse("S = system \"My System\" {}");
        let exporter = Exporter::new();
        let bytes = exporter.export_compact(&program).expect("export_compact");
        let _: serde_json::Value =
            serde_json::from_slice(&bytes).expect("export_compact must produce valid JSON");
    }

    #[test]
    fn exporter_with_extended_constructs() {
        let _ = Exporter::with_extended(true);
        let _ = Exporter::with_extended(false);
    }
}
