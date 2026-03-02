//! DSL printer implementation (modular).
//!
//! Pretty-prints AST back to Sruja DSL format. Each AST category is handled by a dedicated submodule.

mod blocks;
mod definitions;
mod deployment;
mod flows;
mod governance;
mod import;
mod loops;
mod overview;
mod relation;
mod style;
mod view;

use sruja_language::{Program, TopLevelItem};

pub struct DslPrinter;

impl DslPrinter {
    pub fn new() -> Self {
        Self
    }

    pub fn print(&self, program: &Program) -> String {
        if program.items.is_empty() {
            return String::new();
        }

        let mut out = String::new();

        for item in &program.items {
            match item {
                TopLevelItem::ElementDef(elem) => relation::print_element(self, &mut out, elem, 0),
                TopLevelItem::Relation(rel) => relation::print_relation(&mut out, rel, 0),
                TopLevelItem::Import(import) => import::print_import(&mut out, import),
                TopLevelItem::Scenario(scenario) => flows::print_scenario(&mut out, scenario),
                TopLevelItem::Flow(flow) => flows::print_flow(&mut out, flow),
                TopLevelItem::Requirement(req) => governance::print_requirement(&mut out, req),
                TopLevelItem::Adr(adr) => governance::print_adr(&mut out, adr),
                TopLevelItem::Policy(policy) => governance::print_policy(&mut out, policy),
                TopLevelItem::View(view) => view::print_view(&mut out, view),
                TopLevelItem::KindDef(kind_def) => definitions::print_kind_def(&mut out, kind_def),
                TopLevelItem::TagDef(tag_def) => definitions::print_tag_def(&mut out, tag_def),
                TopLevelItem::Overview(overview) => overview::print_overview(&mut out, overview),
                TopLevelItem::Style(style) => style::print_style(&mut out, style),
                TopLevelItem::Deployment(deployment) => {
                    deployment::print_deployment(&mut out, deployment, 0)
                }
                TopLevelItem::Constraints(constraints) => {
                    blocks::print_constraints(&mut out, constraints)
                }
                TopLevelItem::Conventions(conventions) => {
                    blocks::print_conventions(&mut out, conventions)
                }
                TopLevelItem::Extend(extend) => blocks::print_extend(&mut out, extend),
                TopLevelItem::FeedbackLoop(loop_data) => {
                    loops::print_feedback_loop(&mut out, loop_data)
                }
                TopLevelItem::CausalLoop(loop_data) => {
                    loops::print_causal_loop(&mut out, loop_data)
                }
            }
        }

        while out.ends_with('\n') || out.ends_with('\r') {
            out.pop();
        }
        out.push('\n');

        out
    }
}

impl Default for DslPrinter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
