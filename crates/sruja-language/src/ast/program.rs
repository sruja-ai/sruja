//! Program root and top-level item enum.

use super::blocks::{ConstraintsBlock, ConventionsBlock, StyleDecl};
use super::element::{ElementDef, ElementKindDef, ExtendElement};
use super::extended::{CausalLoop, FeedbackLoop, FitnessDef, Incident};
use super::governance::{
    Adr, DeploymentNode, Flow, OverviewBlock, Policy, Requirement, Scenario, SchemaBlock, TagDef,
    ViewDef,
};
use super::relation::{ImportStatement, Relation};

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
