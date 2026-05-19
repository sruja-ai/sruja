//! Core AST types: program, elements, relations, governance blocks.

use sruja_diagnostics::SourceLocation;
use std::collections::HashMap;

use super::extended::{CausalLoop, FeedbackLoop, FitnessDef, Incident, SloBlock};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopLevelItem {
    ElementDef(Box<ElementDef>),
    Relation(Relation),
    Import(ImportStatement),
    Scenario(Scenario),
    Flow(Flow),
    Requirement(Requirement),
    Adr(Adr),
    Policy(Policy),
    View(ViewDef),
    Overview(OverviewBlock),
    Deployment(DeploymentNode),
    Constraints(ConstraintsBlock),
    Conventions(ConventionsBlock),
    Extend(ExtendElement),
    Style(StyleDecl),
    KindDef(ElementKindDef),
    TagDef(TagDef),
    FeedbackLoop(FeedbackLoop),
    CausalLoop(CausalLoop),
    Schema(SchemaBlock),
    Incident(Incident),
    Fitness(FitnessDef),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementDef {
    pub location: SourceLocation,
    pub assignment: ElementAssignment,
}

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
    Policy,
    Requirement,
    Adr,
    Flow,
    Scenario,
    Story,
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
            ElementKind::Custom(k) => write!(f, "{k}"),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("Unknown NodeKind: '{0}'")]
pub struct ParseNodeKindError(pub String);

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("Unknown EdgeKind: '{0}'")]
pub struct ParseEdgeKindError(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct NodeKind(pub String);

impl NodeKind {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn kind_str(&self) -> &str {
        &self.0
    }

    pub fn to_string_kind(&self) -> String {
        self.0.clone()
    }

    pub fn is_custom(&self) -> bool {
        !matches!(
            self.as_str(),
            "system"
                | "service"
                | "container"
                | "component"
                | "database"
                | "queue"
                | "external_api"
                | "frontend"
                | "module"
        )
    }

    pub const SYSTEM: &'static str = "system";
    pub const SERVICE: &'static str = "service";
    pub const CONTAINER: &'static str = "container";
    pub const COMPONENT: &'static str = "component";
    pub const DATABASE: &'static str = "database";
    pub const QUEUE: &'static str = "queue";
    pub const EXTERNAL_API: &'static str = "external_api";
    pub const FRONTEND: &'static str = "frontend";
    pub const MODULE: &'static str = "module";
}

impl std::fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for NodeKind {
    type Err = ParseNodeKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(NodeKind(s.to_string()))
    }
}

impl PartialEq<&str> for NodeKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<str> for NodeKind {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<String> for NodeKind {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl From<&str> for NodeKind {
    fn from(s: &str) -> Self {
        NodeKind(s.to_string())
    }
}

impl From<String> for NodeKind {
    fn from(s: String) -> Self {
        NodeKind(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct EdgeKind(pub String);

impl EdgeKind {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn kind_str(&self) -> &str {
        &self.0
    }

    pub fn to_string_kind(&self) -> String {
        self.0.clone()
    }

    pub fn is_custom(&self) -> bool {
        !matches!(
            self.as_str(),
            "depends_on"
                | "calls"
                | "reads_from"
                | "writes_to"
                | "publishes_to"
                | "subscribes_to"
                | "owns"
                | "contains"
                | "uses"
        )
    }

    pub const DEPENDS_ON: &'static str = "depends_on";
    pub const CALLS: &'static str = "calls";
    pub const READS_FROM: &'static str = "reads_from";
    pub const WRITES_TO: &'static str = "writes_to";
    pub const PUBLISHES_TO: &'static str = "publishes_to";
    pub const SUBSCRIBES_TO: &'static str = "subscribes_to";
    pub const OWNS: &'static str = "owns";
    pub const CONTAINS: &'static str = "contains";
    pub const USES: &'static str = "uses";
}

impl std::fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for EdgeKind {
    type Err = ParseEdgeKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(EdgeKind(s.to_string()))
    }
}

impl PartialEq<&str> for EdgeKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<str> for EdgeKind {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<String> for EdgeKind {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl From<&str> for EdgeKind {
    fn from(s: &str) -> Self {
        EdgeKind(s.to_string())
    }
}

impl From<String> for EdgeKind {
    fn from(s: String) -> Self {
        EdgeKind(s)
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SourceBinding {
    pub kind: SourceKind,
    pub path: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ElementDefBody {
    pub description: Option<String>,
    pub technology: Option<String>,
    pub doc: Option<String>,
    pub knowledge: Option<String>,
    pub metadata: Vec<MetaEntry>,
    pub constraints: Vec<ConstraintEntry>,
    pub conventions: Vec<ConventionEntry>,
    pub style: Option<StyleBlock>,
    pub scale: Option<ScaleBlock>,
    pub slo: Option<SloBlock>,
    pub items: Vec<ElementDefBodyItem>,
    pub gotchas: Vec<String>,
    pub operational_constraints: Vec<String>,
    pub runbooks: Vec<String>,
    pub canonical_id: Option<String>,
    pub aliases: Vec<String>,
    pub owner: Option<String>,
    pub domain: Option<String>,
    pub criticality: Option<Criticality>,
    pub sources: Vec<SourceBinding>,
    pub state_machines: Vec<StateMachine>,
    pub contracts: Vec<Contract>,
    pub fitness_functions: Vec<FitnessDef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementDefBodyItem {
    ElementDef(Box<ElementDef>),
    Relation(Relation),
    Step(ScenarioStep),
    Description(String),
    Technology(String),
    Doc(String),
    Knowledge(String),
    Metadata(MetadataBlock),
    Constraints(ConstraintsBlock),
    Conventions(ConventionsBlock),
    Style(StyleDecl),
    Scale(ScaleBlock),
    Slo(Box<SloBlock>),
    Tags(Vec<String>),
    CanonicalId(String),
    Aliases(Vec<String>),
    Owner(String),
    Domain(String),
    Criticality(Criticality),
    Source(SourceBinding),
    StateMachine(StateMachine),
    Contract(Contract),
    Gotcha(String),
    OperationalConstraint(String),
    Runbook(String),
    Fitness(FitnessDef),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct System {
    pub location: SourceLocation,
    pub id: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub items: Vec<SystemItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemItem {
    Container(Container),
    Component(Component),
    Relation(Relation),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub location: SourceLocation,
    pub id: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub items: Vec<ContainerItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerItem {
    Component(Component),
    Relation(Relation),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub location: SourceLocation,
    pub id: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub technology: Option<String>,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportStatement {
    pub location: SourceLocation,
    pub elements: Vec<ImportElement>,
    pub from: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportElement {
    Ident(String),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
    pub location: SourceLocation,
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub steps: Vec<ScenarioStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioStep {
    pub from: Option<QualifiedIdent>,
    pub to: Option<QualifiedIdent>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub order: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flow {
    pub location: SourceLocation,
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub steps: Vec<ScenarioStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    pub location: SourceLocation,
    pub id: String,
    pub title: String,
    pub r#type: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateMachine {
    pub location: SourceLocation,
    pub name: String,
    pub initial_state: String,
    pub terminal_states: Vec<String>,
    pub transitions: Vec<StateTransition>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateTransition {
    pub location: SourceLocation,
    pub from: String,
    pub to: String,
    pub event: String,
    pub guard: Option<String>,
    pub action: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contract {
    pub location: SourceLocation,
    pub name: String,
    pub description: Option<String>,
    pub inputs: Vec<ContractField>,
    pub outputs: Vec<ContractField>,
    pub errors: Vec<ContractError>,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractField {
    pub name: String,
    pub spec: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractError {
    pub code: String,
    pub description: String,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewRule {
    pub include: Option<ViewRuleExpr>,
    pub exclude: Option<ViewRuleExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewRuleExpr {
    pub wildcard: bool,
    pub recursive: bool,
    pub elements: Vec<String>,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeploymentNode {
    pub location: SourceLocation,
    pub id: String,
    pub label: Option<String>,
    pub technology: Option<String>,
    pub children: Vec<DeploymentNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintsBlock {
    pub location: SourceLocation,
    pub entries: Vec<ConstraintEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConventionsBlock {
    pub location: SourceLocation,
    pub entries: Vec<ConventionEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConventionEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendElement {
    pub location: SourceLocation,
    pub target: QualifiedIdent,
    pub assignments: Vec<ElementAssignment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleDecl {
    pub location: SourceLocation,
    pub selector: String,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementKindDef {
    pub location: SourceLocation,
    pub kind: ElementKind,
    pub title: Option<String>,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub style: Option<StyleBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaBlock {
    pub location: SourceLocation,
    pub name: String,
    pub node_kinds: Vec<String>,
    pub edge_kinds: Vec<String>,
    pub nesting: Vec<NestingRule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestingRule {
    pub parent: String,
    pub child: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagDef {
    pub location: SourceLocation,
    pub id: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataBlock {
    pub location: SourceLocation,
    pub entries: Vec<MetaEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaEntry {
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleBlock {
    pub location: SourceLocation,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaleBlock {
    pub location: SourceLocation,
    pub min: Option<usize>,
    pub max: Option<usize>,
    pub metric: Option<String>,
}
