//! ADRs, policies, views, scenarios, flows, and deployment.

use sruja_diagnostics::SourceLocation;

use super::relation::QualifiedIdent;
use super::spec::ScenarioStep;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
    pub location: SourceLocation,
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub steps: Vec<ScenarioStep>,
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
