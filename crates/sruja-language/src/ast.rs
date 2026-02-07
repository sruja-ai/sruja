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

/// Element definition body containing nested items
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ElementDefBody {
    pub description: Option<String>,
    pub technology: Option<String>,
    pub metadata: Vec<MetaEntry>,
    pub constraints: Vec<ConstraintEntry>,
    pub conventions: Vec<ConventionEntry>,
    pub style: Option<StyleBlock>,
    pub scale: Option<ScaleBlock>,
    pub slo: Option<SloBlock>,
    pub items: Vec<ElementDefBodyItem>,
}

/// Items that can appear in an element body
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementDefBodyItem {
    ElementDef(Box<ElementDef>),
    Relation(Relation),
    Description(String),
    Technology(String),
    Metadata(MetadataBlock),
    Constraints(ConstraintsBlock),
    Conventions(ConventionsBlock),
    Style(StyleDecl),
    Scale(ScaleBlock),
    Slo(Box<SloBlock>),
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
}

/// View definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewDef {
    pub location: SourceLocation,
    pub id: String,
    pub title: Option<String>,
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
