//! AST structures for Sruja DSL
//!
//! This module defines the Abstract Syntax Tree structures that represent
//! parsed Sruja DSL code.

use serde::{Deserialize, Serialize};
use sruja_diagnostics::SourceLocation;

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
    ElementDef(ElementDef),
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

/// Element kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ElementKind {
    Person,
    Role,
    System,
    Container,
    Component,
    Database,
    Queue,
    Policy,
    Requirement,
    Adr,
    Flow,
    Scenario,
    Story,
    Custom(String), // For custom kinds
}

impl ElementKind {
    pub fn as_str(&self) -> &str {
        match self {
            ElementKind::Person => "person",
            ElementKind::Role => "role",
            ElementKind::System => "system",
            ElementKind::Container => "container",
            ElementKind::Component => "component",
            ElementKind::Database => "database",
            ElementKind::Queue => "queue",
            ElementKind::Policy => "policy",
            ElementKind::Requirement => "requirement",
            ElementKind::Adr => "adr",
            ElementKind::Flow => "flow",
            ElementKind::Scenario => "scenario",
            ElementKind::Story => "story",
            ElementKind::Custom(s) => s,
        }
    }
}

/// Element definition body
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
    ElementDef(ElementDef),
    Relation(Relation),
    Description(String),
    Technology(String),
    Metadata(MetadataBlock),
    Constraints(ConstraintsBlock),
    Conventions(ConventionsBlock),
    Style(StyleDecl),
    Scale(ScaleBlock),
    Slo(SloBlock),
}

/// System element (specialized ElementDef)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct System {
    pub location: SourceLocation,
    pub id: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub items: Vec<SystemItem>,
    // Post-processed fields
    pub containers: Vec<Container>,
    pub data_stores: Vec<DataStore>,
    pub queues: Vec<Queue>,
    pub persons: Vec<Person>,
    pub components: Vec<Component>,
    pub relations: Vec<Relation>,
    pub metadata: Vec<MetaEntry>,
}

/// Items in a system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemItem {
    Container(Container),
    DataStore(DataStore),
    Queue(Queue),
    Person(Person),
    Metadata(MetadataBlock),
    Constraints(ConstraintsBlock),
    Conventions(ConventionsBlock),
    Style(StyleDecl),
    Slo(SloBlock),
    Relation(Relation),
    Description(String),
}

/// Container element
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub location: SourceLocation,
    pub id: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub tags: Vec<String>,
    pub version: Option<String>,
    pub items: Vec<ContainerItem>,
}

/// Items in a container
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerItem {
    Component(Component),
    DataStore(DataStore),
    Queue(Queue),
    Technology(String),
    Tags(Vec<String>),
    Version(String),
    Metadata(MetadataBlock),
    Constraints(ConstraintsBlock),
    Conventions(ConventionsBlock),
    Style(StyleDecl),
    Scale(ScaleBlock),
    Slo(SloBlock),
    Relation(Relation),
    Description(String),
}

/// Component element
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub location: SourceLocation,
    pub id: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub metadata: Vec<MetaEntry>,
    pub relations: Vec<Relation>,
}

/// Person element
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Person {
    pub location: SourceLocation,
    pub id: String,
    pub label: Option<String>,
    pub description: Option<String>,
}

/// DataStore element
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataStore {
    pub location: SourceLocation,
    pub id: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub technology: Option<String>,
}

/// Queue element
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Queue {
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualifiedIdent {
    pub parts: Vec<String>,
}

impl QualifiedIdent {
    pub fn simple(ident: String) -> Self {
        Self {
            parts: vec![ident],
        }
    }

    pub fn qualified(parts: Vec<String>) -> Self {
        Self { parts }
    }

    pub fn as_string(&self) -> String {
        self.parts.join(".")
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
    pub label: Option<String>,
    pub description: Option<String>,
    pub steps: Vec<ScenarioStep>,
}

/// Scenario step
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioStep {
    pub actor: String,
    pub action: String,
}

/// Flow definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flow {
    pub location: SourceLocation,
    pub id: String,
    pub label: Option<String>,
    pub steps: Vec<FlowStep>,
}

/// Flow step
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowStep {
    pub actor: String,
    pub action: String,
}

/// Requirement definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    pub location: SourceLocation,
    pub id: String,
    pub kind: Option<String>, // functional, nonfunctional, etc.
    pub label: Option<String>,
    pub description: Option<String>,
}

/// ADR (Architecture Decision Record)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adr {
    pub location: SourceLocation,
    pub id: String,
    pub label: Option<String>,
    pub description: Option<String>,
}

/// Policy definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    pub location: SourceLocation,
    pub id: String,
    pub label: Option<String>,
    pub description: Option<String>,
}

/// View definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewDef {
    pub location: SourceLocation,
    pub id: String,
    pub title: Option<String>,
    pub includes: Vec<String>,
    pub excludes: Vec<String>,
}

/// Overview block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverviewBlock {
    pub location: SourceLocation,
    pub entries: Vec<MetaEntry>,
}

/// Deployment node
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeploymentNode {
    pub location: SourceLocation,
    pub id: String,
    pub label: Option<String>,
    pub items: Vec<DeploymentItem>,
}

/// Deployment item
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeploymentItem {
    DeploymentNode(DeploymentNode),
    Container(String), // Container ID reference
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
    pub element: String,
    pub constraint: String,
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
    pub element: String,
    pub convention: String,
}

/// Extend element
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendElement {
    pub location: SourceLocation,
    pub id: QualifiedIdent,
    pub body: ElementDefBody,
}

/// Style declaration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleDecl {
    pub location: SourceLocation,
    pub keyword: String, // "style" or "styles"
    pub body: StyleBlock,
}

/// Style block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleBlock {
    pub location: SourceLocation,
    pub entries: Vec<StyleEntry>,
}

/// Style entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleEntry {
    pub key: String,
    pub value: Option<String>,
    pub body: Option<StyleBlock>,
}

/// Metadata block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataBlock {
    pub location: SourceLocation,
    pub entries: Vec<MetaEntry>,
}

/// Metadata entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaEntry {
    pub location: SourceLocation,
    pub key: String,
    pub value: Option<String>,
    pub array: Vec<String>,
}

/// Scale block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaleBlock {
    pub location: SourceLocation,
    pub items: Vec<ScaleItem>,
}

/// Scale item
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScaleItem {
    Min(i32),
    Max(i32),
    Metric(String),
}

/// SLO block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SloBlock {
    pub location: SourceLocation,
    pub items: Vec<SloItem>,
}

/// SLO item
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SloItem {
    Availability(SloAvailability),
    Latency(SloLatency),
    ErrorRate(SloErrorRate),
    Throughput(SloThroughput),
    Cost(SloCost),
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
}

/// SLO Throughput
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SloThroughput {
    pub target: Option<String>,
    pub window: Option<String>,
}

/// SLO Cost
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SloCost {
    pub target: Option<String>,
    pub window: Option<String>,
}

/// Element kind definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementKindDef {
    pub location: SourceLocation,
    pub name: String,
    pub title: Option<String>,
    pub body: Option<ElementKindDefBody>,
}

/// Element kind definition body
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementKindDefBody {
    pub title: Option<String>,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub style: Option<StyleBlock>,
}

/// Tag definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagDef {
    pub location: SourceLocation,
    pub name: String,
    pub title: Option<String>,
}
