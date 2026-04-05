// We need this dummy SEARCH block since the file might be empty or I need to read it first.
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
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub category: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub enforcement: String,
    pub scope: Vec<String>,
    pub rules: Vec<PolicyRule>,
    pub source_ref: SourceReference,
}

/// Content of a policy rule: either a structured rule (evaluated by rule engine) or a legacy phrase (compatibility shim).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyRuleContent {
    DenyEdge {
        from: PolicySelector,
        to: PolicySelector,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        except: Vec<PolicyEdgeException>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        suggestions: Vec<String>,
    },
    RequireTags {
        selector: PolicySelector,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tags: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        except: Vec<PolicySelector>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        suggestions: Vec<String>,
    },
    RequireMetadata {
        selector: PolicySelector,
        key: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        except: Vec<PolicySelector>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        suggestions: Vec<String>,
    },
    RequireSlo {
        selector: PolicySelector,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        except: Vec<PolicySelector>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        suggestions: Vec<String>,
    },
    /// Legacy: free-text constraint; only "X must not call Y" / "X cannot call Y" are parsed (compatibility shim).
    Phrase(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PolicySelector {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub meta: Vec<PolicyMetaSelector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyMetaSelector {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEdgeException {
    pub from: PolicySelector,
    pub to: PolicySelector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub description: String,
    /// Legacy constraint text; used when content is Phrase or for violation message.
    pub constraint: String,
    /// Structured rule content. When present, evaluation uses this instead of parsing constraint.
    #[serde(default)]
    pub content: Option<PolicyRuleContent>,
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
                    category: String::new(),
                    enforcement: String::new(),
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

    /// Load intent model from .sruja file path.
    pub fn from_sruja_file(path: &Path) -> Result<Self, IntentError> {
        let content = std::fs::read_to_string(path).map_err(IntentError::Io)?;
        Self::from_sruja_content(&content, path)
    }

    /// Load intent model from .sruja content string (e.g. for tests). `path` is used for source_ref.
    pub fn from_sruja_content(content: &str, path: &Path) -> Result<Self, IntentError> {
        let parser = sruja_language::Parser::new(path.to_string_lossy().to_string());
        let program = parser
            .parse(content)
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
        model.fill_from_program(&program, path)?;

        Ok(model)
    }

    fn fill_from_program(
        &mut self,
        program: &sruja_language::Program,
        path: &Path,
    ) -> Result<(), IntentError> {
        let (elements, _) = sruja_language::collect_elements(program);

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

            self.components.push(DeclaredComponent {
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

        let (_, relations) = sruja_language::collect_elements(program);
        for rel in &relations {
            self.relationships.push(DeclaredRelationship {
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

        for item in &program.items {
            if let sruja_language::TopLevelItem::Policy(p) = item {
                let rules: Vec<PolicyRule> = if p.rules.is_empty() {
                    // Legacy: no structured rules; one synthetic rule from description (compatibility shim).
                    let constraint_text = p
                        .description
                        .clone()
                        .unwrap_or_else(|| format!("{} / {}", p.category, p.enforcement));
                    vec![PolicyRule {
                        description: p.title.clone(),
                        constraint: constraint_text,
                        content: None,
                    }]
                } else {
                    // Structured rules from DSL; no phrase parsing for evaluation.
                    p.rules
                        .iter()
                        .map(|r| {
                            let (constraint, content) = match r {
                                sruja_language::PolicyRuleAst::DenyEdge {
                                    from,
                                    to,
                                    except,
                                    message,
                                    suggestions,
                                } => (
                                    "deny edge".to_string(),
                                    Some(PolicyRuleContent::DenyEdge {
                                        from: selector_from_ast(from),
                                        to: selector_from_ast(to),
                                        except: except
                                            .iter()
                                            .map(|e| PolicyEdgeException {
                                                from: selector_from_ast(&e.from),
                                                to: selector_from_ast(&e.to),
                                            })
                                            .collect(),
                                        message: message.clone(),
                                        suggestions: suggestions.clone(),
                                    }),
                                ),
                                sruja_language::PolicyRuleAst::RequireTags {
                                    selector,
                                    tags,
                                    except,
                                    message,
                                    suggestions,
                                } => (
                                    "require tags".to_string(),
                                    Some(PolicyRuleContent::RequireTags {
                                        selector: selector_from_ast(selector),
                                        tags: tags.clone(),
                                        except: except.iter().map(selector_from_ast).collect(),
                                        message: message.clone(),
                                        suggestions: suggestions.clone(),
                                    }),
                                ),
                                sruja_language::PolicyRuleAst::RequireMetadata {
                                    selector,
                                    key,
                                    value,
                                    except,
                                    message,
                                    suggestions,
                                } => (
                                    "require metadata".to_string(),
                                    Some(PolicyRuleContent::RequireMetadata {
                                        selector: selector_from_ast(selector),
                                        key: key.clone(),
                                        value: value.clone(),
                                        except: except.iter().map(selector_from_ast).collect(),
                                        message: message.clone(),
                                        suggestions: suggestions.clone(),
                                    }),
                                ),
                                sruja_language::PolicyRuleAst::RequireSlo {
                                    selector,
                                    except,
                                    message,
                                    suggestions,
                                } => (
                                    "require slo".to_string(),
                                    Some(PolicyRuleContent::RequireSlo {
                                        selector: selector_from_ast(selector),
                                        except: except.iter().map(selector_from_ast).collect(),
                                        message: message.clone(),
                                        suggestions: suggestions.clone(),
                                    }),
                                ),
                            };
                            PolicyRule {
                                description: p.title.clone(),
                                constraint,
                                content,
                            }
                        })
                        .collect()
                };
                self.policies.push(DeclaredPolicy {
                    name: p.id.clone(),
                    description: p.title.clone(),
                    category: p.category.clone(),
                    enforcement: p.enforcement.clone(),
                    scope: vec![],
                    rules,
                    source_ref: SourceReference {
                        file: path.to_string_lossy().to_string(),
                        line: Some(p.location.line),
                        element: Some(p.id.clone()),
                    },
                });
            }
        }

        Ok(())
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

fn selector_from_ast(ast: &sruja_language::PolicySelectorAst) -> PolicySelector {
    PolicySelector {
        kind: ast.kind.clone(),
        id: ast.id.clone(),
        tags: ast.tags.clone(),
        technology: ast.technology.clone(),
        meta: ast
            .meta
            .iter()
            .map(|m| PolicyMetaSelector {
                key: m.key.clone(),
                value: m.value.clone(),
            })
            .collect(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ParsedAdr, StructuralImplication as Implication};
    use std::path::PathBuf;

    #[test]
    fn test_intent_model_default() {
        let model = IntentModel::default();
        assert_eq!(model.source.name, "Default");
        assert_eq!(model.source.source_type, IntentSourceType::Manual);
        assert!(model.components.is_empty());
    }

    #[test]
    fn test_intent_model_merge() {
        let mut model1 = IntentModel::default();
        model1.components.push(DeclaredComponent {
            id: "C1".to_string(),
            kind: "service".to_string(),
            label: "C1".to_string(),
            description: None,
            technology: None,
            source_ref: SourceReference { file: "".to_string(), line: None, element: None },
        });

        let mut model2 = IntentModel::default();
        model2.components.push(DeclaredComponent {
            id: "C2".to_string(),
            kind: "service".to_string(),
            label: "C2".to_string(),
            description: None,
            technology: None,
            source_ref: SourceReference { file: "".to_string(), line: None, element: None },
        });
        model2.relationships.push(DeclaredRelationship {
            source: "C1".to_string(),
            target: "C2".to_string(),
            kind: "depends_on".to_string(),
            label: None,
            source_ref: SourceReference { file: "".to_string(), line: None, element: None },
        });

        model1.merge(model2);
        assert_eq!(model1.components.len(), 2);
        assert_eq!(model1.relationships.len(), 1);
        
        let ids = model1.component_ids();
        assert!(ids.contains(&"C1"));
        assert!(ids.contains(&"C2"));

        let pairs = model1.relationship_pairs();
        assert_eq!(pairs, vec![("C1", "C2")]);

        assert!(model1.find_component("C1").is_some());
        assert!(model1.find_component("C3").is_none());
    }

    #[test]
    fn test_intent_model_from_adr() {
        let adr = ParsedAdr {
            path: PathBuf::from("doc/adr/0001-test.md"),
            number: Some(1),
            title: "Test ADR".to_string(),
            status: crate::parser::AdrStatus::Accepted,
            date: None,
            context: "Ctx".to_string(),
            decision: "Dec".to_string(),
            consequences: "Cons".to_string(),
            tags: vec![],
            implications: vec![
                Implication {
                    raw_text: "Component API introduced".to_string(),
                    component: Some("API".to_string()),
                    new_policy: None,
                    boundary_change: None,
                    constraint: None,
                },
                Implication {
                    raw_text: "Policy: No cycles".to_string(),
                    component: None,
                    new_policy: Some("No cycles allowed".to_string()),
                    boundary_change: None,
                    constraint: None,
                }
            ],
        };

        let model = IntentModel::from_adr(adr);
        assert_eq!(model.source.name, "Test ADR");
        assert_eq!(model.source.source_type, IntentSourceType::AdrFile);
        assert_eq!(model.components.len(), 1);
        assert_eq!(model.components[0].id, "API");
        assert_eq!(model.policies.len(), 1);
        assert_eq!(model.policies[0].description, "No cycles allowed");
    }

    #[test]
    fn test_intent_model_from_sruja_content() {
        let content = r#"
            system = kind "System"
            App = system "App"
            DB = system "DB"
            App -> DB "reads"
        "#;

        let model = IntentModel::from_sruja_content(content, Path::new("test.sruja")).unwrap();
        assert_eq!(model.source.name, "test.sruja");
        assert_eq!(model.components.len(), 2);
        assert_eq!(model.relationships.len(), 1);
        
        let app = model.find_component("App").unwrap();
        assert_eq!(app.id, "App");
        
        let rel = &model.relationships[0];
        assert_eq!(rel.source, "App");
        assert_eq!(rel.target, "DB");
    }
}
