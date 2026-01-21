//! JSON export types
//!
//! This module defines the JSON structures for exporting Sruja models.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sruja Model Dump - root JSON structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrujaModelDump {
    #[serde(rename = "_stage", skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>, // "parsed" | "computed" | "layouted"
    #[serde(rename = "projectId", skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectDump>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub globals: Option<GlobalsDump>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub imports: HashMap<String, Vec<ElementDump>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployments: Option<DeploymentsDump>,
    pub specification: SpecificationDump,
    pub elements: HashMap<String, ElementDump>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<RelationDump>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub views: HashMap<String, ViewDump>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sruja: Option<SrujaExtensions>,
    #[serde(rename = "_metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ModelMetadata>,
}

/// Project dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDump {
    pub id: String,
    pub name: String,
}

/// Globals dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalsDump {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub predicates: HashMap<String, serde_json::Value>,
    #[serde(rename = "dynamicPredicates", skip_serializing_if = "HashMap::is_empty")]
    pub dynamic_predicates: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub styles: HashMap<String, serde_json::Value>,
}

/// Deployments dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentsDump {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub elements: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub relations: HashMap<String, serde_json::Value>,
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub version: String,
    pub generated: String,
    #[serde(rename = "srujaVersion")]
    pub sruja_version: String,
    #[serde(rename = "layout", skip_serializing_if = "HashMap::is_empty")]
    pub layout_data: HashMap<String, LayoutData>,
}

/// Layout data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutData {
    pub x: i32,
    pub y: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
}

/// Specification dump
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpecificationDump {
    pub elements: HashMap<String, ElementKindDump>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub relationships: HashMap<String, RelationshipKindDump>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub tags: HashMap<String, TagDump>,
    #[serde(rename = "customColors", skip_serializing_if = "HashMap::is_empty")]
    pub colors: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectDump>,
}

/// Element kind dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementKindDump {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<StyleDump>,
}

/// Relationship kind dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipKindDump {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
}

/// Tag dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagDump {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// Element dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementDump {
    pub id: String, // FQN
    pub kind: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<LinkDump>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<StyleDump>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>, // Parent FQN
}

/// Link dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkDump {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Style dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleDump {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<i32>,
}

/// FQN reference dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FqnRefDump {
    pub model: String, // FQN string
}

impl FqnRefDump {
    pub fn new(fqn: String) -> Self {
        Self { model: fqn }
    }
}

/// Relation dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationDump {
    pub id: String,
    pub source: FqnRefDump,
    pub target: FqnRefDump,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<String>, // "solid", "dashed", "dotted"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<String>, // arrow type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tail: Option<String>, // arrow type
}

/// View dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewDump {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "viewOf", skip_serializing_if = "Option::is_none")]
    pub view_of: Option<String>, // FQN of scoped element
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<ViewRule>,
    pub nodes: Vec<NodeDump>,
    pub edges: Vec<EdgeDump>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<ViewLayoutDump>,
}

/// View layout dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewLayoutDump {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub positions: HashMap<String, ViewPositionDump>,
}

/// View position dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewPositionDump {
    pub x: f64,
    pub y: f64,
}

/// View rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<ViewRuleExpr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<ViewRuleExpr>,
}

/// View rule expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewRuleExpr {
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub wildcard: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub recursive: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<String>,
}

/// Node dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDump {
    pub id: String,
    pub element: String, // FQN reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Edge dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDump {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation: Option<String>, // Relation ID reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Sruja extensions (governance layer)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SrujaExtensions {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub requirements: Vec<RequirementDump>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub adrs: Vec<AdrDump>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub policies: Vec<PolicyDump>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<ConstraintDump>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub conventions: Vec<ConventionDump>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scenarios: Vec<ScenarioDump>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub flows: Vec<FlowDump>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deployments: Vec<DeploymentDump>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<ImportDump>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub slos: Vec<SloDump>,
}

/// Requirement dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementDump {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<String>, // FQN references
}

/// ADR dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdrDump {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consequences: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

/// Policy dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDump {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforcement: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<String>,
}

/// Constraint dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintDump {
    pub id: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

/// Convention dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConventionDump {
    pub id: String,
    pub description: String,
}

/// Scenario dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioDump {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<StepDump>,
}

/// Step dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDump {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

/// Flow dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowDump {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<StepDump>,
}

/// Deployment dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentDump {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>, // "node", "region", "zone", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<DeploymentDump>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub instances: Vec<String>, // Element FQNs deployed here
}

/// Import dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDump {
    pub elements: Vec<String>,
    pub from: String,
}

/// SLO dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloDump {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throughput: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_budget: Option<f64>,
    #[serde(rename = "recoveryObjective", skip_serializing_if = "Option::is_none")]
    pub recovery_objective: Option<String>,
    #[serde(rename = "recoveryPoint", skip_serializing_if = "Option::is_none")]
    pub recovery_point: Option<String>,
}
