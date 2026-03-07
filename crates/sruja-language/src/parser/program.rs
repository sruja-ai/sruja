//! Program and top-level item parsing.

use nom::{branch::alt, combinator::map, IResult};

use crate::ast::{ElementAssignment, ElementDef, ElementKind, Program, TopLevelItem};

use super::assignments::{
    parse_adr, parse_adr_assignment, parse_flow, parse_flow_assignment, parse_policy,
    parse_policy_assignment, parse_requirement, parse_requirement_assignment, parse_scenario,
};
use super::blocks::parse_metadata_block;
use super::elements::{parse_element_def, parse_kind_def};
use super::import::parse_import;
use super::loops::{parse_causal_loop, parse_feedback_loop};
use super::overview_views::{parse_overview_block, parse_view};
use super::primitives::ws;
use super::relations::parse_relation;

/// Parse a complete program. Lenient: on item failure, skip to next line and continue.
pub(super) fn parse_program(input: &str) -> IResult<&str, Program> {
    let (input, _) = ws(input)?;
    let mut items = Vec::new();
    let mut remaining = input;
    loop {
        let (rest, _) = ws(remaining)?;
        if rest.is_empty() {
            remaining = rest;
            break;
        }
        match parse_top_level_item(rest) {
            Ok((next, item)) => {
                items.push(item);
                remaining = next;
            }
            Err(_) => {
                if let Some(newline_pos) = rest.find('\n') {
                    remaining = &rest[newline_pos + 1..];
                } else {
                    break;
                }
            }
        }
    }
    Ok((remaining, Program::with_items(Program::new(), items)))
}

/// Parse a single top-level item.
pub(super) fn parse_top_level_item(input: &str) -> IResult<&str, TopLevelItem> {
    alt((
        map(parse_kind_def, TopLevelItem::KindDef),
        map(parse_flow_assignment, TopLevelItem::Flow),
        map(parse_requirement_assignment, TopLevelItem::Requirement),
        map(parse_adr_assignment, TopLevelItem::Adr),
        map(parse_policy_assignment, TopLevelItem::Policy),
        map(parse_overview_block, TopLevelItem::Overview),
        map(parse_feedback_loop, TopLevelItem::FeedbackLoop),
        map(parse_causal_loop, TopLevelItem::CausalLoop),
        map(parse_element_def, |e| TopLevelItem::ElementDef(Box::new(e))),
        map(parse_relation, TopLevelItem::Relation),
        map(parse_import, TopLevelItem::Import),
        map(parse_scenario, TopLevelItem::Scenario),
        map(parse_flow, TopLevelItem::Flow),
        map(parse_requirement, TopLevelItem::Requirement),
        map(parse_adr, TopLevelItem::Adr),
        map(parse_policy, TopLevelItem::Policy),
        map(parse_view, TopLevelItem::View),
        map(parse_metadata_block, |m| {
            TopLevelItem::ElementDef(Box::new(ElementDef {
                location: m.location.clone(),
                assignment: ElementAssignment {
                    location: m.location.clone(),
                    name: "metadata".to_string(),
                    kind: ElementKind::Custom("metadata".to_string()),
                    sub_kind: None,
                    title: None,
                    tag_refs: Vec::new(),
                    body: None,
                },
            }))
        }),
    ))(input)
}
