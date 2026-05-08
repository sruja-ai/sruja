//! AST structures for Sruja DSL
//!
//! This module defines the Abstract Syntax Tree structures that represent
//! parsed Sruja DSL code.

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
    /// Schema definition
    Schema(SchemaBlock),
    /// Operational incident definition
    Incident(Incident),
    /// Fitness function definition for self-healing and optimization
    Fitness(FitnessDef),
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

/// Error type for parsing `NodeKind` from string
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("Unknown NodeKind: '{0}'")]
pub struct ParseNodeKindError(pub String);

/// Error type for parsing `EdgeKind` from string
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("Unknown EdgeKind: '{0}'")]
pub struct ParseEdgeKindError(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// A top-level system or subsystem that groups related services.
    System,
    /// A deployable unit of functionality that can run independently.
    Service,
    /// A grouping of components within a service.
    Container,
    /// A modular unit of code with a specific responsibility.
    Component,
    /// A data storage system.
    Database,
    /// A message queue or streaming platform.
    Queue,
    /// An external API or service outside the system boundary.
    ExternalApi,
    /// A user-facing application or interface.
    Frontend,
    /// A generic module or package.
    Module,
    /// Custom node kind
    Custom(String),
}

impl NodeKind {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeKind::System => "system",
            NodeKind::Service => "service",
            NodeKind::Container => "container",
            NodeKind::Component => "component",
            NodeKind::Database => "database",
            NodeKind::Queue => "queue",
            NodeKind::ExternalApi => "external_api",
            NodeKind::Frontend => "frontend",
            NodeKind::Module => "module",
            NodeKind::Custom(_) => "custom",
        }
    }

    pub fn kind_str(&self) -> &str {
        match self {
            NodeKind::Custom(s) => s.as_str(),
            _ => self.as_str(),
        }
    }

    pub fn to_string_kind(&self) -> String {
        match self {
            NodeKind::Custom(s) => s.clone(),
            _ => self.as_str().to_string(),
        }
    }
}

impl std::fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for NodeKind {
    type Err = ParseNodeKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "system" => Ok(NodeKind::System),
            "service" => Ok(NodeKind::Service),
            "container" => Ok(NodeKind::Container),
            "component" => Ok(NodeKind::Component),
            "database" => Ok(NodeKind::Database),
            "queue" => Ok(NodeKind::Queue),
            "external_api" | "externalapi" => Ok(NodeKind::ExternalApi),
            "frontend" => Ok(NodeKind::Frontend),
            "module" => Ok(NodeKind::Module),
            _ => Err(ParseNodeKindError(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// Indicates that the source node depends on the target node for functionality.
    DependsOn,
    /// Indicates that the source node makes calls to the target node.
    Calls,
    /// Indicates that the source node reads data from the target node.
    ReadsFrom,
    /// Indicates that the source node writes data to the target node.
    WritesTo,
    /// Indicates that the source node publishes events to the target node.
    PublishesTo,
    /// Indicates that the source node subscribes to events from the target node.
    SubscribesTo,
    /// Indicates that the source node owns or manages the target node.
    Owns,
    /// Indicates that the source node contains or encompasses the target node.
    Contains,
    /// Indicates that the source node uses the target node in some way.
    Uses,
    /// Custom edge kind
    Custom(String),
}

impl EdgeKind {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeKind::DependsOn => "depends_on",
            EdgeKind::Calls => "calls",
            EdgeKind::ReadsFrom => "reads_from",
            EdgeKind::WritesTo => "writes_to",
            EdgeKind::PublishesTo => "publishes_to",
            EdgeKind::SubscribesTo => "subscribes_to",
            EdgeKind::Owns => "owns",
            EdgeKind::Contains => "contains",
            EdgeKind::Uses => "uses",
            EdgeKind::Custom(_) => "custom",
        }
    }

    pub fn kind_str(&self) -> &str {
        match self {
            EdgeKind::Custom(s) => s.as_str(),
            _ => self.as_str(),
        }
    }

    pub fn to_string_kind(&self) -> String {
        match self {
            EdgeKind::Custom(s) => s.clone(),
            _ => self.as_str().to_string(),
        }
    }
}

impl std::fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for EdgeKind {
    type Err = ParseEdgeKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "depends_on" => Ok(EdgeKind::DependsOn),
            "calls" => Ok(EdgeKind::Calls),
            "reads_from" => Ok(EdgeKind::ReadsFrom),
            "writes_to" => Ok(EdgeKind::WritesTo),
            "publishes_to" => Ok(EdgeKind::PublishesTo),
            "subscribes_to" => Ok(EdgeKind::SubscribesTo),
            "owns" => Ok(EdgeKind::Owns),
            "contains" => Ok(EdgeKind::Contains),
            "uses" => Ok(EdgeKind::Uses),
            _ => Err(ParseEdgeKindError(s.to_string())),
        }
    }
}

/// Criticality level for an element
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Criticality {
    Low,
    Medium,
    High,
    Critical,
}

impl Criticality {
    #[must_use]
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
            _ => Err(format!("Unknown criticality: {s}")),
        }
    }
}

/// Source kind for external resource bindings
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
    #[must_use]
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

    #[must_use]
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    /// Gotchas/tribal knowledge about this element
    pub gotchas: Vec<String>,
    /// Operational constraints
    pub operational_constraints: Vec<String>,
    /// Paths to runbooks
    pub runbooks: Vec<String>,
    /// Canonical ID for cross-system reference
    pub canonical_id: Option<String>,
    pub aliases: Vec<String>,
    pub owner: Option<String>,
    pub domain: Option<String>,
    pub criticality: Option<Criticality>,
    pub sources: Vec<SourceBinding>,
    pub state_machines: Vec<StateMachine>,
    pub contracts: Vec<Contract>,
    /// Fitness functions for self-healing and evolutionary tracking
    pub fitness_functions: Vec<FitnessDef>,
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
    /// State machine definition
    StateMachine(StateMachine),
    /// API contract definition
    Contract(Contract),
    /// Gotcha string
    Gotcha(String),
    /// Operational constraint string
    OperationalConstraint(String),
    /// Runbook path
    Runbook(String),
    /// Fitness function definition
    Fitness(FitnessDef),
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

/// State machine definition (nested inside element body)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateMachine {
    pub location: SourceLocation,
    pub name: String,
    pub initial_state: String,
    pub terminal_states: Vec<String>,
    pub transitions: Vec<StateTransition>,
    pub description: Option<String>,
}

/// A single state transition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateTransition {
    pub location: SourceLocation,
    pub from: String,
    pub to: String,
    pub event: String,          // The trigger event name
    pub guard: Option<String>,  // Pre-condition expression (string)
    pub action: Option<String>, // Side-effect to execute
    pub description: Option<String>,
}

/// API contract definition (nested inside element body)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contract {
    pub location: SourceLocation,
    pub name: String, // e.g. "POST /payments"
    pub description: Option<String>,
    pub inputs: Vec<ContractField>,
    pub outputs: Vec<ContractField>,
    pub errors: Vec<ContractError>,
    pub constraints: Vec<String>,
}

/// A field in contract input/output
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractField {
    pub name: String,
    pub spec: String, // Free-text type/validation spec
}

/// An error response in a contract
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractError {
    pub code: String, // "400", "409", etc.
    pub description: String,
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
    pub affects: Vec<String>,
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

/// Schema definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaBlock {
    pub location: SourceLocation,
    pub name: String,
    pub node_kinds: Vec<String>,
    pub edge_kinds: Vec<String>,
    pub nesting: Vec<NestingRule>,
}

/// Nesting rule in a schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestingRule {
    pub parent: String,
    pub child: String,
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

/// Fitness function definition for self-healing and optimization
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FitnessDef {
    pub location: SourceLocation,
    pub id: String,
    pub target: String,
    pub measure: String,
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

    #[test]
    fn test_node_kind_parsing_and_display() {
        use std::str::FromStr;

        assert_eq!(NodeKind::from_str("system").unwrap(), NodeKind::System);
        assert_eq!(NodeKind::from_str("service").unwrap(), NodeKind::Service);
        assert_eq!(
            NodeKind::from_str("container").unwrap(),
            NodeKind::Container
        );
        assert_eq!(
            NodeKind::from_str("component").unwrap(),
            NodeKind::Component
        );
        assert_eq!(NodeKind::from_str("database").unwrap(), NodeKind::Database);
        assert_eq!(NodeKind::from_str("queue").unwrap(), NodeKind::Queue);
        assert_eq!(
            NodeKind::from_str("external_api").unwrap(),
            NodeKind::ExternalApi
        );
        assert_eq!(NodeKind::from_str("frontend").unwrap(), NodeKind::Frontend);
        assert_eq!(NodeKind::from_str("module").unwrap(), NodeKind::Module);
        assert!(NodeKind::from_str("invalid_kind").is_err());

        assert_eq!(NodeKind::System.as_str(), "system");
        assert_eq!(format!("{}", NodeKind::Database), "database");
        assert_eq!(
            NodeKind::Custom("custom_node".to_string()).kind_str(),
            "custom_node"
        );
        assert_eq!(NodeKind::System.kind_str(), "system");
        assert_eq!(
            NodeKind::Custom("custom_node".to_string()).to_string_kind(),
            "custom_node"
        );
        assert_eq!(NodeKind::System.to_string_kind(), "system");
    }

    #[test]
    fn test_edge_kind_parsing_and_display() {
        use std::str::FromStr;

        assert_eq!(
            EdgeKind::from_str("depends_on").unwrap(),
            EdgeKind::DependsOn
        );
        assert_eq!(EdgeKind::from_str("calls").unwrap(), EdgeKind::Calls);
        assert_eq!(
            EdgeKind::from_str("reads_from").unwrap(),
            EdgeKind::ReadsFrom
        );
        assert_eq!(EdgeKind::from_str("writes_to").unwrap(), EdgeKind::WritesTo);
        assert_eq!(
            EdgeKind::from_str("publishes_to").unwrap(),
            EdgeKind::PublishesTo
        );
        assert_eq!(
            EdgeKind::from_str("subscribes_to").unwrap(),
            EdgeKind::SubscribesTo
        );
        assert_eq!(EdgeKind::from_str("owns").unwrap(), EdgeKind::Owns);
        assert_eq!(EdgeKind::from_str("contains").unwrap(), EdgeKind::Contains);
        assert_eq!(EdgeKind::from_str("uses").unwrap(), EdgeKind::Uses);
        assert!(EdgeKind::from_str("invalid_edge").is_err());

        assert_eq!(EdgeKind::Calls.as_str(), "calls");
        assert_eq!(format!("{}", EdgeKind::ReadsFrom), "reads_from");
        assert_eq!(
            EdgeKind::Custom("custom_edge".to_string()).kind_str(),
            "custom_edge"
        );
        assert_eq!(EdgeKind::Calls.kind_str(), "calls");
        assert_eq!(
            EdgeKind::Custom("custom_edge".to_string()).to_string_kind(),
            "custom_edge"
        );
        assert_eq!(EdgeKind::Calls.to_string_kind(), "calls");
    }

    #[test]
    fn test_criticality_parsing_and_display() {
        use std::str::FromStr;

        assert_eq!(Criticality::from_str("low").unwrap(), Criticality::Low);
        assert_eq!(
            Criticality::from_str("medium").unwrap(),
            Criticality::Medium
        );
        assert_eq!(Criticality::from_str("med").unwrap(), Criticality::Medium);
        assert_eq!(Criticality::from_str("high").unwrap(), Criticality::High);
        assert_eq!(
            Criticality::from_str("critical").unwrap(),
            Criticality::Critical
        );
        assert!(Criticality::from_str("invalid_criticality").is_err());

        assert_eq!(Criticality::Low.as_str(), "low");
        assert_eq!(format!("{}", Criticality::High), "high");
    }

    #[test]
    fn test_source_kind_parsing_and_display() {
        use std::str::FromStr;

        assert_eq!(SourceKind::parse("openapi"), SourceKind::OpenApi);
        assert_eq!(SourceKind::parse("asyncapi"), SourceKind::AsyncApi);
        assert_eq!(SourceKind::parse("kubernetes"), SourceKind::Kubernetes);
        assert_eq!(SourceKind::parse("k8s"), SourceKind::Kubernetes);
        assert_eq!(SourceKind::parse("dockerfile"), SourceKind::Dockerfile);
        assert_eq!(SourceKind::parse("docker"), SourceKind::Dockerfile);
        assert_eq!(SourceKind::parse("terraform"), SourceKind::Terraform);
        assert_eq!(SourceKind::parse("tf"), SourceKind::Terraform);
        assert_eq!(SourceKind::parse("docs"), SourceKind::Docs);
        assert_eq!(SourceKind::parse("doc"), SourceKind::Docs);
        assert_eq!(SourceKind::parse("readme"), SourceKind::Readme);
        assert_eq!(SourceKind::parse("proto"), SourceKind::Proto);
        assert_eq!(SourceKind::parse("protobuf"), SourceKind::Proto);
        assert_eq!(SourceKind::parse("config"), SourceKind::Config);
        assert_eq!(SourceKind::parse("graphql"), SourceKind::GraphQL);
        assert_eq!(SourceKind::parse("gql"), SourceKind::GraphQL);
        assert_eq!(SourceKind::parse("helm"), SourceKind::Helm);
        assert_eq!(
            SourceKind::parse("custom_kind"),
            SourceKind::Custom("custom_kind".to_string())
        );

        assert_eq!(SourceKind::OpenApi.as_str(), "openapi");
        assert_eq!(format!("{}", SourceKind::Kubernetes), "kubernetes");
        assert_eq!(
            format!("{}", SourceKind::Custom("custom".to_string())),
            "custom"
        );

        assert_eq!(
            SourceKind::from_str("openapi").unwrap(),
            SourceKind::OpenApi
        );
    }
}

/// Operational incident definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Incident {
    pub location: SourceLocation,
    pub id: String,
    pub title: String,
    pub date: Option<String>,
    pub severity: Option<String>,
    pub affected: Vec<QualifiedIdent>,
    pub cause: Option<String>,
    pub resolution: Option<String>,
    pub lesson: Option<String>,
}
