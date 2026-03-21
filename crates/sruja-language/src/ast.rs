//! AST structures for Sruja DSL
//!
//! This module defines the Abstract Syntax Tree structures that represent
//! parsed Sruja DSL code.

use serde::{Deserialize, Serialize};
use sruja_diagnostics::SourceLocation;
use std::collections::HashMap;

/// Main program structure containing all parsed items
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub items: Vec<TopLevelItem>,
}

impl Program {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn with_items(mut self, items: Vec<TopLevelItem>) -> Self {
        self.items = items;
        self
    }

    pub fn push_item(&mut self, item: TopLevelItem) {
        self.items.push(item);
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

/// Top-level items that can appear in a Sruja file
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopLevelItem {
    /// Element definition: person, system, container, component, etc.
    ElementDef(Box<ElementDef>),
    /// Relation between elements
    Relation(Relation),
    /// Import statement
    Import(ImportStatement),
    /// Scenario definition
    Scenario(Scenario),
    /// Flow definition
    Flow(Flow),
    /// Requirement definition
    Requirement(Requirement),
    /// ADR (Architecture Decision Record)
    Adr(Adr),
    /// Policy definition
    Policy(Policy),
    /// View definition
    View(ViewDef),
    /// Overview block
    Overview(OverviewBlock),
    /// Deployment node
    Deployment(DeploymentNode),
    /// Constraints block
    Constraints(ConstraintsBlock),
    /// Conventions block
    Conventions(ConventionsBlock),
    /// Extend element
    Extend(ExtendElement),
    /// Style declaration
    Style(StyleDecl),
    /// Element kind definition
    KindDef(ElementKindDef),
    /// Tag definition
    TagDef(TagDef),
    /// Feedback loop definition
    FeedbackLoop(FeedbackLoop),
    /// Causal loop definition
    CausalLoop(CausalLoop),
}

/// Element definition (person, system, container, component, database, queue)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementDef {
    pub location: SourceLocation,
    pub assignment: ElementAssignment,
}

/// Element assignment: Name = Kind [SubKind] [Label] [#tags...] [Body]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementAssignment {
    pub location: SourceLocation,
    pub name: String,
    pub kind: ElementKind,
    pub sub_kind: Option<String>,
    pub title: Option<String>,
    pub tag_refs: Vec<String>,
    pub body: Option<ElementDefBody>,
}

impl ElementAssignment {
    /// Create an element assignment with name and kind (for tests and examples).
    #[must_use]
    pub fn new(name: impl Into<String>, kind: ElementKind) -> Self {
        Self {
            location: SourceLocation::new(String::new(), 0, 0),
            name: name.into(),
            kind,
            sub_kind: None,
            title: None,
            tag_refs: Vec::new(),
            body: None,
        }
    }
}

/// Element kinds
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElementKind {
    Person,
    Role,
    System,
    Container,
    Component,
    Database,
    Queue,
    ExternalSystem,
    DataStore,
    // Governance / behavioral kinds (parsed as ElementDefs in Go model)
    Policy,
    Requirement,
    Adr,
    Flow,
    Scenario,
    Story,
    /// Custom kinds (specification-defined)
    Custom(String),
}

impl std::fmt::Display for ElementKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementKind::Person => write!(f, "person"),
            ElementKind::Role => write!(f, "role"),
            ElementKind::System => write!(f, "system"),
            ElementKind::Container => write!(f, "container"),
            ElementKind::Component => write!(f, "component"),
            ElementKind::Database => write!(f, "database"),
            ElementKind::Queue => write!(f, "queue"),
            ElementKind::ExternalSystem => write!(f, "externalSystem"),
            ElementKind::DataStore => write!(f, "datastore"),
            ElementKind::Policy => write!(f, "policy"),
            ElementKind::Requirement => write!(f, "requirement"),
            ElementKind::Adr => write!(f, "adr"),
            ElementKind::Flow => write!(f, "flow"),
            ElementKind::Scenario => write!(f, "scenario"),
            ElementKind::Story => write!(f, "story"),
            ElementKind::Custom(k) => write!(f, "{}", k),
        }
    }
}

/// Criticality level for an element
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Criticality {
    Low,
    Medium,
    High,
    Critical,
}

impl Criticality {
    pub fn as_str(&self) -> &'static str {
        match self {
            Criticality::Low => "low",
            Criticality::Medium => "medium",
            Criticality::High => "high",
            Criticality::Critical => "critical",
        }
    }
}

impl std::fmt::Display for Criticality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Criticality {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Criticality::Low),
            "medium" | "med" => Ok(Criticality::Medium),
            "high" => Ok(Criticality::High),
            "critical" => Ok(Criticality::Critical),
            _ => Err(format!("Unknown criticality: {}", s)),
        }
    }
}

/// Source kind for external resource bindings
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    OpenApi,
    AsyncApi,
    Kubernetes,
    Dockerfile,
    Terraform,
    Docs,
    Readme,
    Proto,
    Config,
    GraphQL,
    Helm,
    Custom(String),
}

impl SourceKind {
    pub fn as_str(&self) -> &str {
        match self {
            SourceKind::OpenApi => "openapi",
            SourceKind::AsyncApi => "asyncapi",
            SourceKind::Kubernetes => "kubernetes",
            SourceKind::Dockerfile => "dockerfile",
            SourceKind::Terraform => "terraform",
            SourceKind::Docs => "docs",
            SourceKind::Readme => "readme",
            SourceKind::Proto => "proto",
            SourceKind::Config => "config",
            SourceKind::GraphQL => "graphql",
            SourceKind::Helm => "helm",
            SourceKind::Custom(s) => s,
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "openapi" => SourceKind::OpenApi,
            "asyncapi" => SourceKind::AsyncApi,
            "kubernetes" | "k8s" => SourceKind::Kubernetes,
            "dockerfile" | "docker" => SourceKind::Dockerfile,
            "terraform" | "tf" => SourceKind::Terraform,
            "docs" | "doc" => SourceKind::Docs,
            "readme" => SourceKind::Readme,
            "proto" | "protobuf" => SourceKind::Proto,
            "config" => SourceKind::Config,
            "graphql" | "gql" => SourceKind::GraphQL,
            "helm" => SourceKind::Helm,
            _ => SourceKind::Custom(s.to_string()),
        }
    }
}

impl std::str::FromStr for SourceKind {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SourceKind::parse(s))
    }
}

impl std::fmt::Display for SourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Source binding linking element to external resource
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceBinding {
    pub kind: SourceKind,
    pub path: String,
    pub description: Option<String>,
}

/// Element definition body containing nested items
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ElementDefBody {
    pub description: Option<String>,
    pub technology: Option<String>,
    /// Path to component knowledge markdown (relative to workspace root).
    pub doc: Option<String>,
    /// ID linking to code knowledge graph (.sruja/knowledge/<id>.json)
    pub knowledge: Option<String>,
    pub metadata: Vec<MetaEntry>,
    pub constraints: Vec<ConstraintEntry>,
    pub conventions: Vec<ConventionEntry>,
    pub style: Option<StyleBlock>,
    pub scale: Option<ScaleBlock>,
    pub slo: Option<SloBlock>,
    pub items: Vec<ElementDefBodyItem>,
    /// Canonical ID for cross-system reference
    pub canonical_id: Option<String>,
    /// Aliases found in codebase (alternative names for this element)
    pub aliases: Vec<String>,
    /// Owner team or individual
    pub owner: Option<String>,
    /// Business domain
    pub domain: Option<String>,
    /// Criticality level
    pub criticality: Option<Criticality>,
    /// Source bindings to external resources
    pub sources: Vec<SourceBinding>,
}

/// Items that can appear in an element body
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementDefBodyItem {
    ElementDef(Box<ElementDef>),
    Relation(Relation),
    Step(ScenarioStep),
    Description(String),
    Technology(String),
    /// Doc path to component knowledge file (e.g. ".sruja/knowledge/<id>.md").
    Doc(String),
    /// ID linking to code knowledge graph (.sruja/knowledge/<id>.json)
    Knowledge(String),
    Metadata(MetadataBlock),
    Constraints(ConstraintsBlock),
    Conventions(ConventionsBlock),
    Style(StyleDecl),
    Scale(ScaleBlock),
    Slo(Box<SloBlock>),
    /// Tags on an element (parsed but not stored in ElementDefBody; consumed to avoid parse failures)
    Tags(Vec<String>),
    /// Canonical ID for cross-system reference
    CanonicalId(String),
    /// Aliases found in codebase
    Aliases(Vec<String>),
    /// Owner team or individual
    Owner(String),
    /// Business domain
    Domain(String),
    /// Criticality level
    Criticality(Criticality),
    /// Source binding to external resource
    Source(SourceBinding),
}

/// System element (specialized ElementDef)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct System {
    pub location: SourceLocation,
    pub id: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub items: Vec<SystemItem>,
}

/// Items that can appear in a system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemItem {
    Container(Container),
    Component(Component),
    Relation(Relation),
}

/// Container element
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub location: SourceLocation,
    pub id: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub items: Vec<ContainerItem>,
}

/// Items that can appear in a container
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerItem {
    Component(Component),
    Relation(Relation),
}

/// Component element
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub location: SourceLocation,
    pub id: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub technology: Option<String>,
}

/// Relation between elements
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relation {
    pub location: SourceLocation,
    pub from: QualifiedIdent,
    pub to: QualifiedIdent,
    pub label: Option<String>,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub tags: Vec<String>,
}

/// Qualified identifier (e.g., "System.Container" or just "System")
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QualifiedIdent {
    pub parts: Vec<String>,
}

impl QualifiedIdent {
    pub fn simple(ident: String) -> Self {
        Self { parts: vec![ident] }
    }

    pub fn qualified(parts: Vec<String>) -> Self {
        Self { parts }
    }

    pub fn as_string(&self) -> String {
        self.parts.join(".")
    }
}

impl std::fmt::Display for QualifiedIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

/// Import statement
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportStatement {
    pub location: SourceLocation,
    pub elements: Vec<ImportElement>,
    pub from: String,
}

/// Import element (identifier or wildcard)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportElement {
    Ident(String),
    Wildcard,
}

/// Scenario definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
    pub location: SourceLocation,
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub steps: Vec<ScenarioStep>,
}

/// Scenario step
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioStep {
    pub from: Option<QualifiedIdent>,
    pub to: Option<QualifiedIdent>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub order: Option<usize>,
}

/// Flow definition (alias for Scenario)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flow {
    pub location: SourceLocation,
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub steps: Vec<ScenarioStep>,
}

/// Requirement definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    pub location: SourceLocation,
    pub id: String,
    pub title: String,
    pub r#type: String, // functional, performance, security, constraint
    pub description: Option<String>,
    pub tags: Vec<String>,
}

/// ADR (Architecture Decision Record)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adr {
    pub location: SourceLocation,
    pub id: String,
    pub title: String,
    pub status: Option<String>,
    pub context: Option<String>,
    pub decision: Option<String>,
    pub consequences: Option<String>,
}

/// Policy definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    pub location: SourceLocation,
    pub id: String,
    pub title: String,
    pub category: String,
    pub enforcement: String,
    pub description: Option<String>,
    pub rules: Vec<PolicyRuleAst>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PolicySelectorAst {
    pub kind: Option<String>,
    pub id: Option<String>,
    pub tags: Vec<String>,
    pub technology: Option<String>,
    pub meta: Vec<PolicyMetaSelectorAst>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyMetaSelectorAst {
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyEdgeExceptionAst {
    pub from: PolicySelectorAst,
    pub to: PolicySelectorAst,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyRuleAst {
    DenyEdge {
        from: PolicySelectorAst,
        to: PolicySelectorAst,
        except: Vec<PolicyEdgeExceptionAst>,
        message: Option<String>,
        suggestions: Vec<String>,
    },
    RequireTags {
        selector: PolicySelectorAst,
        tags: Vec<String>,
        except: Vec<PolicySelectorAst>,
        message: Option<String>,
        suggestions: Vec<String>,
    },
    RequireMetadata {
        selector: PolicySelectorAst,
        key: String,
        value: Option<String>,
        except: Vec<PolicySelectorAst>,
        message: Option<String>,
        suggestions: Vec<String>,
    },
    RequireSlo {
        selector: PolicySelectorAst,
        except: Vec<PolicySelectorAst>,
        message: Option<String>,
        suggestions: Vec<String>,
    },
}

/// View definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewDef {
    pub location: SourceLocation,
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub view_of: Option<QualifiedIdent>,
    pub tags: Vec<String>,
    pub rules: Vec<ViewRule>,
}

/// View rule
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewRule {
    pub include: Option<ViewRuleExpr>,
    pub exclude: Option<ViewRuleExpr>,
}

/// View rule expression
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewRuleExpr {
    pub wildcard: bool,
    pub recursive: bool,
    pub elements: Vec<String>,
}

/// Overview block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverviewBlock {
    pub location: SourceLocation,
    pub summary: Option<String>,
    pub audience: Option<String>,
    pub scope: Option<String>,
    pub goals: Vec<String>,
    pub non_goals: Vec<String>,
    pub risks: Vec<String>,
}

/// Deployment node
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeploymentNode {
    pub location: SourceLocation,
    pub id: String,
    pub label: Option<String>,
    pub technology: Option<String>,
    pub children: Vec<DeploymentNode>,
}

/// Constraints block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintsBlock {
    pub location: SourceLocation,
    pub entries: Vec<ConstraintEntry>,
}

/// Constraint entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintEntry {
    pub key: String,
    pub value: String,
}

/// Conventions block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConventionsBlock {
    pub location: SourceLocation,
    pub entries: Vec<ConventionEntry>,
}

/// Convention entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConventionEntry {
    pub key: String,
    pub value: String,
}

/// Extend element
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendElement {
    pub location: SourceLocation,
    pub target: QualifiedIdent,
    pub assignments: Vec<ElementAssignment>,
}

/// Style declaration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleDecl {
    pub location: SourceLocation,
    pub selector: String,
    pub properties: HashMap<String, String>,
}

/// Element kind definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementKindDef {
    pub location: SourceLocation,
    pub kind: ElementKind,
    pub title: Option<String>,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub style: Option<StyleBlock>,
}

/// Tag definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagDef {
    pub location: SourceLocation,
    pub id: String,
    pub color: Option<String>,
}

/// Metadata block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataBlock {
    pub location: SourceLocation,
    pub entries: Vec<MetaEntry>,
}

/// Metadata entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaEntry {
    pub key: String,
    pub value: Option<String>,
}

/// Style block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleBlock {
    pub location: SourceLocation,
    pub properties: HashMap<String, String>,
}

/// Scale block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaleBlock {
    pub location: SourceLocation,
    pub min: Option<usize>,
    pub max: Option<usize>,
    pub metric: Option<String>,
}

/// SLO block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SloBlock {
    pub location: SourceLocation,
    pub availability: Option<SloAvailability>,
    pub latency: Option<SloLatency>,
    pub error_rate: Option<SloErrorRate>,
    pub throughput: Option<SloThroughput>,
}

/// SLO Availability
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SloAvailability {
    pub target: Option<String>,
    pub window: Option<String>,
    pub current: Option<String>,
}

/// SLO Latency
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SloLatency {
    pub p95: Option<String>,
    pub p99: Option<String>,
    pub window: Option<String>,
    pub current: Option<SloCurrent>,
}

/// SLO Current
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SloCurrent {
    pub p95: Option<String>,
    pub p99: Option<String>,
}

/// SLO Error Rate
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SloErrorRate {
    pub target: Option<String>,
    pub window: Option<String>,
    pub current: Option<String>,
}

/// SLO Throughput
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SloThroughput {
    pub target: Option<String>,
    pub window: Option<String>,
    pub current: Option<String>,
}

/// Feedback loop type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackLoopType {
    Reinforcing,
    Balancing,
}

impl FeedbackLoopType {
    pub fn to_symbol(&self) -> &str {
        match self {
            FeedbackLoopType::Reinforcing => "+",
            FeedbackLoopType::Balancing => "-",
        }
    }
}

impl std::fmt::Display for FeedbackLoopType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeedbackLoopType::Reinforcing => write!(f, "reinforcing"),
            FeedbackLoopType::Balancing => write!(f, "balancing"),
        }
    }
}

/// Causal polarity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CausalPolarity {
    Positive,
    Negative,
}

impl CausalPolarity {
    pub fn to_symbol(&self) -> &str {
        match self {
            CausalPolarity::Positive => "+",
            CausalPolarity::Negative => "-",
        }
    }
}

impl std::fmt::Display for CausalPolarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CausalPolarity::Positive => write!(f, "+"),
            CausalPolarity::Negative => write!(f, "-"),
        }
    }
}

/// Feedback loop definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackLoop {
    pub location: SourceLocation,
    pub id: String,
    pub loop_type: FeedbackLoopType,
    pub loop_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub relationships: Vec<Relation>,
}

/// Causal loop variable
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalLoopVariable {
    pub id: String,
    pub label: Option<String>,
}

/// Causal relationship
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalRelationship {
    pub from: String,
    pub to: String,
    pub effect: Option<String>,
    pub polarity: CausalPolarity,
    pub delay: Option<String>,
}

/// Causal loop definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalLoop {
    pub location: SourceLocation,
    pub id: String,
    pub loop_type: FeedbackLoopType,
    pub loop_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub variables: Vec<CausalLoopVariable>,
    pub relationships: Vec<CausalRelationship>,
}

/// Result of an incremental parse: updated AST plus change metadata and timing.
/// Used when re-parsing only a context window around an edit and merging with the existing AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalParseResult {
    /// The merged/updated program AST.
    pub updated_ast: Program,
    /// Names of elements that were added or modified.
    pub changed_elements: Vec<String>,
    /// Line ranges that were affected (start_line, end_line) for incremental UI updates.
    pub changed_ranges: Vec<(usize, usize)>,
    /// Parsing time in milliseconds (for metrics and adaptive debouncing).
    pub parsing_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Program tests

    #[test]
    fn test_program_new_creates_empty_program() {
        let program = Program::new();
        assert!(program.items.is_empty());
    }

    #[test]
    fn test_program_default_creates_empty_program() {
        let program = Program::default();
        assert!(program.items.is_empty());
    }

    #[test]
    fn test_program_with_items() {
        let items = vec![TopLevelItem::ElementDef(Box::new(ElementDef {
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            assignment: ElementAssignment::new("test", ElementKind::System),
        }))];
        let program = Program::new().with_items(items.clone());
        assert_eq!(program.items.len(), 1);
        assert_eq!(program.items, items);
    }

    #[test]
    fn test_program_push_item() {
        let mut program = Program::new();
        assert_eq!(program.items.len(), 0);

        let item = TopLevelItem::ElementDef(Box::new(ElementDef {
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            assignment: ElementAssignment::new("test", ElementKind::System),
        }));
        program.push_item(item.clone());

        assert_eq!(program.items.len(), 1);
        assert_eq!(program.items[0], item);
    }

    // ElementAssignment tests

    #[test]
    fn test_element_assignment_new_creates_with_defaults() {
        let assignment = ElementAssignment::new("my_system", ElementKind::System);
        assert_eq!(assignment.name, "my_system");
        assert_eq!(assignment.kind, ElementKind::System);
        assert!(assignment.sub_kind.is_none());
        assert!(assignment.title.is_none());
        assert!(assignment.tag_refs.is_empty());
        assert!(assignment.body.is_none());
        // Check default location
        assert_eq!(assignment.location.file, "");
        assert_eq!(assignment.location.line, 0);
        assert_eq!(assignment.location.column, 0);
    }

    #[test]
    fn test_element_assignment_new_with_different_kinds() {
        let kinds = vec![
            (ElementKind::Person, "user"),
            (ElementKind::System, "system"),
            (ElementKind::Container, "web"),
            (ElementKind::Component, "auth"),
            (ElementKind::Database, "db"),
            (ElementKind::Queue, "queue"),
            (ElementKind::ExternalSystem, "external"),
            (ElementKind::DataStore, "datastore"),
            (ElementKind::Policy, "policy"),
            (ElementKind::Requirement, "req"),
            (ElementKind::Adr, "adr"),
            (ElementKind::Flow, "flow"),
            (ElementKind::Scenario, "scenario"),
            (ElementKind::Story, "story"),
            (ElementKind::Custom("custom_kind".to_string()), "custom"),
        ];

        for (kind, name) in kinds {
            let assignment = ElementAssignment::new(name, kind.clone());
            assert_eq!(assignment.name, name);
            assert_eq!(assignment.kind, kind);
        }
    }

    // ElementKind Display tests

    #[test]
    fn test_element_kind_display_standard_kinds() {
        assert_eq!(format!("{}", ElementKind::Person), "person");
        assert_eq!(format!("{}", ElementKind::Role), "role");
        assert_eq!(format!("{}", ElementKind::System), "system");
        assert_eq!(format!("{}", ElementKind::Container), "container");
        assert_eq!(format!("{}", ElementKind::Component), "component");
        assert_eq!(format!("{}", ElementKind::Database), "database");
        assert_eq!(format!("{}", ElementKind::Queue), "queue");
        assert_eq!(format!("{}", ElementKind::ExternalSystem), "externalSystem");
        assert_eq!(format!("{}", ElementKind::DataStore), "datastore");
    }

    #[test]
    fn test_element_kind_display_governance_kinds() {
        assert_eq!(format!("{}", ElementKind::Policy), "policy");
        assert_eq!(format!("{}", ElementKind::Requirement), "requirement");
        assert_eq!(format!("{}", ElementKind::Adr), "adr");
        assert_eq!(format!("{}", ElementKind::Flow), "flow");
        assert_eq!(format!("{}", ElementKind::Scenario), "scenario");
        assert_eq!(format!("{}", ElementKind::Story), "story");
    }

    #[test]
    fn test_element_kind_display_custom_kind() {
        let custom_kind = ElementKind::Custom("MyCustomKind".to_string());
        assert_eq!(format!("{}", custom_kind), "MyCustomKind");
    }

    // QualifiedIdent tests

    #[test]
    fn test_qualified_ident_simple() {
        let ident = QualifiedIdent::simple("test".to_string());
        assert_eq!(ident.parts, vec!["test"]);
        assert_eq!(ident.as_string(), "test");
    }

    #[test]
    fn test_qualified_ident_qualified() {
        let ident = QualifiedIdent::qualified(vec![
            "module".to_string(),
            "submodule".to_string(),
            "item".to_string(),
        ]);
        assert_eq!(ident.parts, vec!["module", "submodule", "item"]);
        assert_eq!(ident.as_string(), "module.submodule.item");
    }

    #[test]
    fn test_qualified_ident_as_string_single_part() {
        let ident = QualifiedIdent::simple("single".to_string());
        assert_eq!(ident.as_string(), "single");
    }

    #[test]
    fn test_qualified_ident_as_string_multiple_parts() {
        let ident =
            QualifiedIdent::qualified(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        assert_eq!(ident.as_string(), "a.b.c");
    }

    #[test]
    fn test_qualified_ident_display() {
        let ident = QualifiedIdent::qualified(vec!["foo".to_string(), "bar".to_string()]);
        assert_eq!(format!("{}", ident), "foo.bar");
    }

    // FeedbackLoopType tests

    #[test]
    fn test_feedback_loop_type_to_symbol_reinforcing() {
        let loop_type = FeedbackLoopType::Reinforcing;
        assert_eq!(loop_type.to_symbol(), "+");
    }

    #[test]
    fn test_feedback_loop_type_to_symbol_balancing() {
        let loop_type = FeedbackLoopType::Balancing;
        assert_eq!(loop_type.to_symbol(), "-");
    }

    #[test]
    fn test_feedback_loop_type_display_reinforcing() {
        let loop_type = FeedbackLoopType::Reinforcing;
        assert_eq!(format!("{}", loop_type), "reinforcing");
    }

    #[test]
    fn test_feedback_loop_type_display_balancing() {
        let loop_type = FeedbackLoopType::Balancing;
        assert_eq!(format!("{}", loop_type), "balancing");
    }

    // CausalPolarity tests

    #[test]
    fn test_causal_polarity_to_symbol_positive() {
        let polarity = CausalPolarity::Positive;
        assert_eq!(polarity.to_symbol(), "+");
    }

    #[test]
    fn test_causal_polarity_to_symbol_negative() {
        let polarity = CausalPolarity::Negative;
        assert_eq!(polarity.to_symbol(), "-");
    }

    #[test]
    fn test_causal_polarity_display_positive() {
        let polarity = CausalPolarity::Positive;
        assert_eq!(format!("{}", polarity), "+");
    }

    #[test]
    fn test_causal_polarity_display_negative() {
        let polarity = CausalPolarity::Negative;
        assert_eq!(format!("{}", polarity), "-");
    }

    // Comprehensive test combining multiple structures

    #[test]
    fn test_comprehensive_ast_construction() {
        let mut program = Program::new();

        // Add a system element
        let system_elem = ElementDef {
            location: SourceLocation::new("example.sruja".to_string(), 1, 1),
            assignment: ElementAssignment::new("MyApp", ElementKind::System),
        };
        program.push_item(TopLevelItem::ElementDef(Box::new(system_elem)));

        // Add a container element
        let container_elem = ElementDef {
            location: SourceLocation::new("example.sruja".to_string(), 5, 1),
            assignment: ElementAssignment::new("WebServer", ElementKind::Container),
        };
        program.push_item(TopLevelItem::ElementDef(Box::new(container_elem)));

        // Add a relation
        let relation = TopLevelItem::Relation(Relation {
            location: SourceLocation::new("example.sruja".to_string(), 10, 1),
            from: QualifiedIdent::simple("MyApp".to_string()),
            to: QualifiedIdent::simple("WebServer".to_string()),
            label: Some("HTTPS".to_string()),
            description: None,
            technology: None,
            tags: Vec::new(),
        });
        program.push_item(relation);

        assert_eq!(program.items.len(), 3);
    }

    #[test]
    fn test_element_assignment_cloning() {
        let assignment = ElementAssignment::new("original", ElementKind::System);
        let cloned = assignment.clone();

        assert_eq!(assignment.name, cloned.name);
        assert_eq!(assignment.kind, cloned.kind);
    }

    #[test]
    fn test_program_builder_pattern() {
        let program = Program::new().with_items(vec![
            TopLevelItem::ElementDef(Box::new(ElementDef {
                location: SourceLocation::new("test.sruja".to_string(), 1, 1),
                assignment: ElementAssignment::new("A", ElementKind::System),
            })),
            TopLevelItem::ElementDef(Box::new(ElementDef {
                location: SourceLocation::new("test.sruja".to_string(), 2, 1),
                assignment: ElementAssignment::new("B", ElementKind::System),
            })),
        ]);

        assert_eq!(program.items.len(), 2);
    }
}
