//! Element definitions and legacy C4 hierarchy types.

use sruja_diagnostics::SourceLocation;

use super::blocks::{
    ConstraintEntry, ConstraintsBlock, ConventionEntry, ConventionsBlock, MetaEntry, MetadataBlock,
    ScaleBlock, StyleBlock, StyleDecl,
};
use super::extended::{FitnessDef, SloBlock};
use super::kinds::{Criticality, ElementKind, SourceBinding};
use super::relation::{QualifiedIdent, Relation};
use super::spec::{Contract, ScenarioStep, StateMachine};

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

/// Legacy nested system representation (pre-unified element syntax).
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
pub struct ExtendElement {
    pub location: SourceLocation,
    pub target: QualifiedIdent,
    pub assignments: Vec<ElementAssignment>,
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
