//! Parser for Sruja DSL using nom parser combinators
//!
//! This module implements a parser for the Sruja DSL using `nom` parser combinators.
//! It parses source code directly into an AST without a separate lexing phase.

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::{char, multispace0, multispace1, line_ending},
    combinator::{map, opt, recognize, value, cut},
    multi::{many0, many1, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};
use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
use std::collections::HashMap;

use crate::ast::*;
use crate::token::lookup_ident;

/// Parser for Sruja DSL
pub struct Parser {
    filename: String,
}

impl Parser {
    /// Create a new parser
    pub fn new(filename: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
        }
    }

    /// Parse source code into a Program AST
    pub fn parse(&self, input: &str) -> Result<Program, Vec<Diagnostic>> {
        match parse_program(input) {
            Ok((remaining, program)) => {
                if !remaining.trim().is_empty() {
                    return Err(vec![Diagnostic::new(
                        sruja_diagnostics::codes::CODE_SYNTAX_ERROR,
                        Severity::Error,
                        format!("Unexpected input remaining: {}", &remaining[..remaining.len().min(50)]),
                        SourceLocation::new(self.filename.clone(), 0, 0),
                    )]);
                }
                Ok(program)
            }
            Err(e) => Err(vec![Diagnostic::new(
                sruja_diagnostics::codes::CODE_SYNTAX_ERROR,
                Severity::Error,
                format!("Parse error: {:?}", e),
                SourceLocation::new(self.filename.clone(), 0, 0),
            )]),
        }
    }
}

// Helper: Skip whitespace and comments
fn skip_whitespace_and_comments(input: &str) -> IResult<&str, ()> {
    let mut input = input;
    loop {
        // Skip whitespace
        let (new_input, _) = multispace0(input)?;
        input = new_input;
        
        // Try to skip comment
        let comment_result = alt((
            // Single-line comment: // ...
            preceded(tag("//"), take_until("\n")),
            // Multi-line comment: /* ... */
            delimited(tag("/*"), take_until("*/"), tag("*/")),
        ))(input);
        
        match comment_result {
            Ok((new_input, _)) => {
                input = new_input;
            }
            Err(_) => break,
        }
    }
    Ok((input, ()))
}

fn ws(input: &str) -> IResult<&str, ()> {
    skip_whitespace_and_comments(input)
}

fn ws0(input: &str) -> IResult<&str, ()> {
    multispace0(input).map(|(i, _)| (i, ()))
}

fn ws1(input: &str) -> IResult<&str, ()> {
    multispace1(input).map(|(i, _)| (i, ()))
}

// Nom parsers

/// Parse a complete program
fn parse_program(input: &str) -> IResult<&str, Program> {
    let (input, _) = ws(input)?;
    let (input, items) = many0(preceded(ws, parse_top_level_item))(input)?;
    Ok((input, Program::with_items(Program::new(), items)))
}

/// Parse a top-level item
fn parse_top_level_item(input: &str) -> IResult<&str, TopLevelItem> {
    alt((
        map(parse_element_def, TopLevelItem::ElementDef),
        map(parse_relation, TopLevelItem::Relation),
        map(parse_import, TopLevelItem::Import),
        map(parse_scenario, TopLevelItem::Scenario),
        map(parse_flow, TopLevelItem::Flow),
        map(parse_requirement, TopLevelItem::Requirement),
        map(parse_adr, TopLevelItem::Adr),
        map(parse_policy, TopLevelItem::Policy),
        map(parse_view, TopLevelItem::View),
        map(parse_metadata_block, |m| TopLevelItem::ElementDef(ElementDef {
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
        })), // Temporary conversion
    ))(input)
}

/// Parse an element definition: Name = Kind [SubKind] [Label] [#tags...] [Body]
fn parse_element_def(input: &str) -> IResult<&str, ElementDef> {
    let (input, name) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('='))(input)?;
    let (input, _) = ws0(input)?;
    let (input, kind) = parse_element_kind(input)?;
    let (input, _) = ws0(input)?;
    let (input, sub_kind) = opt(parse_identifier)(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, tag_refs) = many0(parse_tag_ref)(input)?;
    let (input, _) = ws0(input)?;
    let (input, body) = opt(parse_element_def_body)(input)?;

    Ok((
        input,
        ElementDef {
            location: SourceLocation::new(String::new(), 0, 0), // TODO: track position
            assignment: ElementAssignment {
                location: SourceLocation::new(String::new(), 0, 0),
                name,
                kind,
                sub_kind,
                title,
                tag_refs,
                body,
            },
        },
    ))
}

/// Parse element kind
fn parse_element_kind(input: &str) -> IResult<&str, ElementKind> {
    alt((
        value(ElementKind::Person, tag("person")),
        value(ElementKind::Role, tag("role")),
        value(ElementKind::System, tag("system")),
        value(ElementKind::Container, tag("container")),
        value(ElementKind::Component, tag("component")),
        value(ElementKind::Database, tag("database")),
        value(ElementKind::Queue, tag("queue")),
        value(ElementKind::Policy, tag("policy")),
        value(ElementKind::Requirement, tag("requirement")),
        value(ElementKind::Adr, tag("adr")),
        value(ElementKind::Flow, tag("flow")),
        value(ElementKind::Scenario, tag("scenario")),
        value(ElementKind::Story, tag("story")),
        map(parse_identifier, ElementKind::Custom),
    ))(input)
}

/// Parse element definition body
fn parse_element_def_body(input: &str) -> IResult<&str, ElementDefBody> {
    delimited(
        preceded(ws0, char('{')),
        map(
            many0(preceded(ws, parse_element_body_item)),
            |items| {
                let mut body = ElementDefBody::default();
                // Process items and populate body fields
                for item in items {
                    match item {
                        ElementDefBodyItem::Description(d) => body.description = Some(d),
                        ElementDefBodyItem::Technology(t) => body.technology = Some(t),
                        ElementDefBodyItem::Metadata(m) => body.metadata = m.entries,
                        ElementDefBodyItem::Slo(s) => body.slo = Some(s),
                        ElementDefBodyItem::ElementDef(e) => body.items.push(ElementDefBodyItem::ElementDef(e)),
                        ElementDefBodyItem::Relation(r) => body.items.push(ElementDefBodyItem::Relation(r)),
                        ElementDefBodyItem::Constraints(c) => body.constraints = c.entries,
                        ElementDefBodyItem::Conventions(c) => body.conventions = c.entries,
                        ElementDefBodyItem::Style(s) => body.style = Some(s),
                        ElementDefBodyItem::Scale(s) => body.scale = Some(s),
                        // Add more handlers
                        _ => {}
                    }
                }
                body
            },
        ),
        preceded(ws0, char('}')),
    )(input)
}

fn parse_element_body_item(input: &str) -> IResult<&str, ElementDefBodyItem> {
    alt((
        map(
            preceded(alt((tag("description"), tag("desc"))), preceded(ws1, parse_string)),
            ElementDefBodyItem::Description,
        ),
        map(
            preceded(alt((tag("technology"), tag("tech"))), preceded(ws1, parse_string)),
            ElementDefBodyItem::Technology,
        ),
        map(parse_metadata_block, |m| ElementDefBodyItem::Metadata(m)),
        map(parse_slo_block, ElementDefBodyItem::Slo),
        map(parse_element_def, ElementDefBodyItem::ElementDef),
        map(parse_relation, ElementDefBodyItem::Relation),
        map(parse_constraints_block, ElementDefBodyItem::Constraints),
        map(parse_conventions_block, ElementDefBodyItem::Conventions),
        map(parse_style_decl, ElementDefBodyItem::Style),
        map(parse_scale_block, ElementDefBodyItem::Scale),
    ))(input)
}

/// Parse an SLO block: slo { availability { ... } latency { ... } errorRate { ... } throughput { ... } }
fn parse_slo_block(input: &str) -> IResult<&str, SloBlock> {
    let (input, _) = tag("slo")(input)?;
    let (input, _) = ws0(input)?;
    let (input, items) = delimited(
        char('{'),
        many0(preceded(ws, parse_slo_item)),
        preceded(ws0, char('}')),
    )(input)?;

    let mut slo = SloBlock {
        location: SourceLocation::new(String::new(), 0, 0),
        availability: None,
        latency: None,
        error_rate: None,
        throughput: None,
    };

    for item in items {
        match item {
            SloItem::Availability(a) => slo.availability = Some(a),
            SloItem::Latency(l) => slo.latency = Some(l),
            SloItem::ErrorRate(e) => slo.error_rate = Some(e),
            SloItem::Throughput(t) => slo.throughput = Some(t),
        }
    }

    Ok((input, slo))
}

#[derive(Debug, Clone)]
enum SloItem {
    Availability(SloAvailability),
    Latency(SloLatency),
    ErrorRate(SloErrorRate),
    Throughput(SloThroughput),
}

fn parse_slo_item(input: &str) -> IResult<&str, SloItem> {
    alt((
        map(parse_slo_availability, SloItem::Availability),
        map(parse_slo_latency, SloItem::Latency),
        map(parse_slo_error_rate, SloItem::ErrorRate),
        map(parse_slo_throughput, SloItem::Throughput),
    ))(input)
}

fn parse_slo_availability(input: &str) -> IResult<&str, SloAvailability> {
    let (input, _) = tag("availability")(input)?;
    let (input, _) = ws0(input)?;
    let (input, entries) = delimited(
        char('{'),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )(input)?;

    let mut out = SloAvailability {
        target: None,
        window: None,
        current: None,
    };

    for (k, v) in entries {
        match k.as_str() {
            "target" => out.target = Some(v),
            "window" => out.window = Some(v),
            "current" => out.current = Some(v),
            _ => {}
        }
    }

    Ok((input, out))
}

fn parse_slo_latency(input: &str) -> IResult<&str, SloLatency> {
    let (input, _) = tag("latency")(input)?;
    let (input, _) = ws0(input)?;
    let (input, items) = delimited(
        char('{'),
        many0(preceded(ws, parse_latency_item)),
        preceded(ws0, char('}')),
    )(input)?;

    let mut out = SloLatency {
        p95: None,
        p99: None,
        window: None,
        current: None,
    };

    for item in items {
        match item {
            LatencyItem::P95(v) => out.p95 = Some(v),
            LatencyItem::P99(v) => out.p99 = Some(v),
            LatencyItem::Window(v) => out.window = Some(v),
            LatencyItem::Current(c) => out.current = Some(c),
        }
    }

    Ok((input, out))
}

#[derive(Debug, Clone)]
enum LatencyItem {
    P95(String),
    P99(String),
    Window(String),
    Current(SloCurrent),
}

fn parse_latency_item(input: &str) -> IResult<&str, LatencyItem> {
    alt((
        map(preceded(tag("p95"), preceded(ws1, parse_string)), LatencyItem::P95),
        map(preceded(tag("p99"), preceded(ws1, parse_string)), LatencyItem::P99),
        map(preceded(tag("window"), preceded(ws1, parse_string)), LatencyItem::Window),
        map(parse_slo_current, LatencyItem::Current),
    ))(input)
}

fn parse_slo_current(input: &str) -> IResult<&str, SloCurrent> {
    let (input, _) = tag("current")(input)?;
    let (input, _) = ws0(input)?;
    let (input, entries) = delimited(
        char('{'),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )(input)?;

    let mut out = SloCurrent { p95: None, p99: None };
    for (k, v) in entries {
        match k.as_str() {
            "p95" => out.p95 = Some(v),
            "p99" => out.p99 = Some(v),
            _ => {}
        }
    }
    Ok((input, out))
}

fn parse_slo_error_rate(input: &str) -> IResult<&str, SloErrorRate> {
    let (input, _) = tag("errorRate")(input)?;
    let (input, _) = ws0(input)?;
    let (input, entries) = delimited(
        char('{'),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )(input)?;

    let mut out = SloErrorRate {
        target: None,
        window: None,
        current: None,
    };
    for (k, v) in entries {
        match k.as_str() {
            "target" => out.target = Some(v),
            "window" => out.window = Some(v),
            "current" => out.current = Some(v),
            _ => {}
        }
    }
    Ok((input, out))
}

fn parse_slo_throughput(input: &str) -> IResult<&str, SloThroughput> {
    let (input, _) = tag("throughput")(input)?;
    let (input, _) = ws0(input)?;
    let (input, entries) = delimited(
        char('{'),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )(input)?;

    let mut out = SloThroughput {
        target: None,
        window: None,
        current: None,
    };
    for (k, v) in entries {
        match k.as_str() {
            "target" => out.target = Some(v),
            "window" => out.window = Some(v),
            "current" => out.current = Some(v),
            _ => {}
        }
    }
    Ok((input, out))
}

fn parse_kv_string(input: &str) -> IResult<&str, (String, String)> {
    let (input, key) = parse_identifier(input)?;
    let (input, _) = ws1(input)?;
    let (input, value) = parse_string(input)?;
    Ok((input, (key, value)))
}

/// Parse a scenario: (scenario | story) [ID] [Title] [Description] [{ Steps }]
fn parse_scenario(input: &str) -> IResult<&str, Scenario> {
    let (input, _) = alt((tag("scenario"), tag("story")))(input)?;
    let (input, _) = ws0(input)?;
    let (input, id) = opt(parse_identifier)(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, steps) = opt(parse_scenario_body)(input)?;

    Ok((
        input,
        Scenario {
            location: SourceLocation::new(String::new(), 0, 0),
            id: id.unwrap_or_default(),
            title: title.unwrap_or_default(),
            description,
            steps: steps.unwrap_or_default(),
        },
    ))
}

/// Parse scenario body
fn parse_scenario_body(input: &str) -> IResult<&str, Vec<ScenarioStep>> {
    delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_scenario_step)),
        preceded(ws0, char('}')),
    )(input)
}

/// Parse scenario step: [step] From -> To [Description] [Tags] [Order]
fn parse_scenario_step(input: &str) -> IResult<&str, ScenarioStep> {
    let (input, _) = opt(preceded(tag("step"), ws1))(input)?;
    let (input, from) = parse_qualified_ident(input)?;
    let (input, _) = preceded(ws0, tag("->"))(input)?;
    let (input, _) = ws0(input)?;
    let (input, to) = parse_qualified_ident(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, tags) = opt(parse_tag_array)(input)?;
    let (input, _) = ws0(input)?;
    let (input, order_raw) = opt(preceded(tag("order"), preceded(ws1, parse_string)))(input)?;

    let order = order_raw
        .as_deref()
        .and_then(|s| s.parse::<usize>().ok());

    Ok((
        input,
        ScenarioStep {
            from: Some(from),
            to: Some(to),
            description,
            tags: tags.unwrap_or_default(),
            order,
        },
    ))
}

/// Parse a flow: flow [ID] [Title] [Description] [{ Steps }]
fn parse_flow(input: &str) -> IResult<&str, Flow> {
    let (input, _) = tag("flow")(input)?;
    let (input, _) = ws0(input)?;
    let (input, id) = opt(parse_identifier)(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, steps) = opt(parse_flow_body)(input)?;

    Ok((
        input,
        Flow {
            location: SourceLocation::new(String::new(), 0, 0),
            id: id.unwrap_or_default(),
            title: title.unwrap_or_default(),
            description,
            steps: steps.unwrap_or_default(),
        },
    ))
}

/// Parse flow body
fn parse_flow_body(input: &str) -> IResult<&str, Vec<ScenarioStep>> {
    delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_flow_step)),
        preceded(ws0, char('}')),
    )(input)
}

/// Parse flow step: From -> To [Description]
fn parse_flow_step(input: &str) -> IResult<&str, ScenarioStep> {
    let (input, from) = parse_qualified_ident(input)?;
    let (input, _) = preceded(ws0, tag("->"))(input)?;
    let (input, _) = ws0(input)?;
    let (input, to) = parse_qualified_ident(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string)(input)?;

    Ok((
        input,
        ScenarioStep {
            from: Some(from),
            to: Some(to),
            description,
            tags: Vec::new(),
            order: None,
        },
    ))
}

/// Parse a requirement: requirement ID [Type] [Description] [{ Body }]
fn parse_requirement(input: &str) -> IResult<&str, Requirement> {
    let (input, _) = tag("requirement")(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, kind) = opt(parse_identifier)(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, _body) = opt(parse_requirement_body)(input)?;

    Ok((
        input,
        Requirement {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            kind,
            label: None,
            description,
        },
    ))
}

/// Parse requirement body
fn parse_requirement_body(input: &str) -> IResult<&str, ()> {
    delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_requirement_property)),
        preceded(ws0, char('}')),
    )(input)
    .map(|(i, _)| (i, ()))
}

fn parse_requirement_property(input: &str) -> IResult<&str, ()> {
    alt((
        preceded(alt((tag("type"), tag("description"), tag("tags"), tag("metadata"))), 
            preceded(ws0, alt((parse_string, parse_tag_array, parse_metadata_block)))),
    ))(input)
    .map(|(i, _)| (i, ()))
}

/// Parse an ADR: adr ID [Title] [{ Body }]
fn parse_adr(input: &str) -> IResult<&str, Adr> {
    let (input, _) = tag("adr")(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, _body) = opt(parse_adr_body)(input)?;

    Ok((
        input,
        Adr {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            label: title.clone(),
            description: title,
        },
    ))
}

/// Parse ADR body
fn parse_adr_body(input: &str) -> IResult<&str, ()> {
    delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_adr_property)),
        preceded(ws0, char('}')),
    )(input)
    .map(|(i, _)| (i, ()))
}

fn parse_adr_property(input: &str) -> IResult<&str, ()> {
    preceded(
        alt((tag("status"), tag("context"), tag("decision"), tag("consequences"), tag("tags"))),
        preceded(ws0, alt((parse_string, parse_tag_array))),
    )(input)
    .map(|(i, _)| (i, ()))
}

/// Parse a policy: policy ID [Title] [Description]
fn parse_policy(input: &str) -> IResult<&str, Policy> {
    let (input, _) = tag("policy")(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string)(input)?;

    Ok((
        input,
        Policy {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            label: title,
            description,
        },
    ))
}

/// Parse a view definition
fn parse_view(input: &str) -> IResult<&str, ViewDef> {
    let (input, _) = tag("view")(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(preceded(tag("title"), preceded(ws1, parse_string)))(input)?;
    let (input, _) = ws0(input)?;
    let (input, includes) = opt(preceded(tag("include"), preceded(ws0, parse_view_expression)))(input)?;
    let (input, _) = ws0(input)?;
    let (input, excludes) = opt(preceded(tag("exclude"), preceded(ws0, parse_view_expression)))(input)?;

    Ok((
        input,
        ViewDef {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            title,
            includes: includes.unwrap_or_default(),
            excludes: excludes.unwrap_or_default(),
        },
    ))
}

fn parse_view_expression(input: &str) -> IResult<&str, Vec<String>> {
    alt((
        map(char('*'), |_| vec!["*".to_string()]),
        separated_list1(preceded(ws0, char(',')), preceded(ws0, parse_identifier)),
    ))(input)
}

/// Parse constraints block: constraints { key "value" ... }
fn parse_constraints_block(input: &str) -> IResult<&str, ConstraintsBlock> {
    let (input, _) = tag("constraints")(input)?;
    let (input, _) = ws0(input)?;
    let (input, entries) = delimited(
        char('{'),
        many0(preceded(ws, parse_constraint_entry)),
        preceded(ws0, char('}')),
    )(input)?;
    Ok((
        input,
        ConstraintsBlock {
            location: SourceLocation::new(String::new(), 0, 0),
            entries,
        },
    ))
}

fn parse_constraint_entry(input: &str) -> IResult<&str, ConstraintEntry> {
    let (input, key) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, value) = parse_string(input)?;
    Ok((input, ConstraintEntry { key, value }))
}

/// Parse conventions block: conventions { key "value" ... }
fn parse_conventions_block(input: &str) -> IResult<&str, ConventionsBlock> {
    let (input, _) = tag("conventions")(input)?;
    let (input, _) = ws0(input)?;
    let (input, entries) = delimited(
        char('{'),
        many0(preceded(ws, parse_convention_entry)),
        preceded(ws0, char('}')),
    )(input)?;
    Ok((
        input,
        ConventionsBlock {
            location: SourceLocation::new(String::new(), 0, 0),
            entries,
        },
    ))
}

fn parse_convention_entry(input: &str) -> IResult<&str, ConventionEntry> {
    let (input, key) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, value) = parse_string(input)?;
    Ok((input, ConventionEntry { key, value }))
}

/// Parse style declaration: style selector { property "value" ... }
fn parse_style_decl(input: &str) -> IResult<&str, StyleDecl> {
    let (input, _) = tag("style")(input)?;
    let (input, _) = ws1(input)?;
    let (input, selector) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, properties) = delimited(
        char('{'),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )(input)?;
    let mut props_map = HashMap::new();
    for (k, v) in properties {
        props_map.insert(k, v);
    }
    Ok((
        input,
        StyleDecl {
            location: SourceLocation::new(String::new(), 0, 0),
            selector,
            properties: props_map,
        },
    ))
}

/// Parse scale block: scale { min 1 max 10 metric "instances" }
fn parse_scale_block(input: &str) -> IResult<&str, ScaleBlock> {
    let (input, _) = tag("scale")(input)?;
    let (input, _) = ws0(input)?;
    let (input, items) = delimited(
        char('{'),
        many0(preceded(ws, parse_scale_item)),
        preceded(ws0, char('}')),
    )(input)?;
    let mut scale = ScaleBlock {
        location: SourceLocation::new(String::new(), 0, 0),
        min: None,
        max: None,
        metric: None,
    };
    for (key, value) in items {
        match key.as_str() {
            "min" => scale.min = value.parse().ok(),
            "max" => scale.max = value.parse().ok(),
            "metric" => scale.metric = Some(value),
            _ => {}
        }
    }
    Ok((input, scale))
}

fn parse_scale_item(input: &str) -> IResult<&str, (String, String)> {
    let (input, key) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, value) = alt((
        parse_string,
        map(parse_identifier, |s| s),
    ))(input)?;
    Ok((input, (key, value)))
}

/// Parse a relation: From -> To [Label] [Tags]
fn parse_relation(input: &str) -> IResult<&str, Relation> {
    let (input, from) = parse_qualified_ident(input)?;
    let (input, _) = preceded(ws0, tag("->"))(input)?;
    let (input, _) = ws0(input)?;
    let (input, to) = parse_qualified_ident(input)?;
    let (input, _) = ws0(input)?;
    let (input, label) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, technology) = opt(preceded(alt((tag("technology"), tag("tech"))), preceded(ws1, parse_string)))(input)?;
    let (input, _) = ws0(input)?;
    let (input, tags) = opt(parse_tag_array)(input)?;

    Ok((
        input,
        Relation {
            location: SourceLocation::new(String::new(), 0, 0), // TODO: track position
            from,
            to,
            label,
            description,
            technology,
            tags: tags.unwrap_or_default(),
        },
    ))
}

/// Parse qualified identifier: Ident ('.' Ident)*
fn parse_qualified_ident(input: &str) -> IResult<&str, QualifiedIdent> {
    let (input, first) = parse_identifier(input)?;
    let (input, rest) = many0(preceded(char('.'), parse_identifier))(input)?;
    
    let mut parts = vec![first];
    parts.extend(rest);
    
    Ok((input, QualifiedIdent::qualified(parts)))
}

/// Parse import statement: import { elements... } from "path"
fn parse_import(input: &str) -> IResult<&str, ImportStatement> {
    let (input, _) = tag("import")(input)?;
    let (input, _) = ws1(input)?;
    let (input, elements) = delimited(
        char('{'),
        separated_list0(preceded(ws0, char(',')), preceded(ws0, parse_import_element)),
        preceded(ws0, char('}')),
    )(input)?;
    let (input, _) = ws1(input)?;
    let (input, _) = tag("from")(input)?;
    let (input, _) = ws1(input)?;
    let (input, from) = parse_string(input)?;

    Ok((
        input,
        ImportStatement {
            location: SourceLocation::new(String::new(), 0, 0), // TODO: track position
            elements,
            from,
        },
    ))
}

/// Parse import element (identifier or wildcard)
fn parse_import_element(input: &str) -> IResult<&str, ImportElement> {
    alt((
        value(ImportElement::Wildcard, char('*')),
        map(parse_identifier, ImportElement::Ident),
    ))(input)
}

/// Parse a metadata block: metadata { entries... }
fn parse_metadata_block(input: &str) -> IResult<&str, MetadataBlock> {
    let (input, _) = tag("metadata")(input)?;
    let (input, _) = ws1(input)?;
    let (input, entries) = delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_metadata_entry)),
        preceded(ws0, char('}')),
    )(input)?;

    Ok((
        input,
        MetadataBlock {
            location: SourceLocation::new(String::new(), 0, 0), // TODO: track position
            entries,
        },
    ))
}

/// Parse a metadata entry: key [value]
fn parse_metadata_entry(input: &str) -> IResult<&str, MetaEntry> {
    let (input, key) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, value) = opt(parse_string)(input)?;
    
    Ok((
        input,
        MetaEntry {
            key,
            value,
        },
    ))
}

/// Parse a tag reference: #Ident
fn parse_tag_ref(input: &str) -> IResult<&str, String> {
    let (input, _) = char('#')(input)?;
    let (input, ident) = parse_identifier(input)?;
    Ok((input, format!("#{}", ident)))
}

/// Parse a tag array: [Ident, Ident, ...]
fn parse_tag_array(input: &str) -> IResult<&str, Vec<String>> {
    delimited(
        preceded(ws0, char('[')),
        separated_list0(preceded(ws0, char(',')), preceded(ws0, parse_identifier)),
        preceded(ws0, char(']')),
    )(input)
}

/// Parse an identifier
fn parse_identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            take_while1(|c: char| c.is_alphabetic() || c == '_'),
            take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
        )),
        |s: &str| s.to_string(),
    )(input)
}

/// Parse a string literal (double or single quoted)
fn parse_string(input: &str) -> IResult<&str, String> {
    alt((
        delimited(char('"'), take_until("\""), char('"')),
        delimited(char('\''), take_until("'"), char('\'')),
    ))(input)
    .map(|(input, s)| (input, s.to_string()))
}

/// Parse a string array: [String, String, ...]
fn parse_string_array(input: &str) -> IResult<&str, Vec<String>> {
    delimited(
        preceded(ws0, char('[')),
        separated_list0(preceded(ws0, char(',')), preceded(ws0, parse_string)),
        preceded(ws0, char(']')),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_identifier() {
        assert_eq!(parse_identifier("mySystem"), Ok(("", "mySystem".to_string())));
        assert_eq!(parse_identifier("my-system_123"), Ok(("", "my-system_123".to_string())));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_string(r#""hello""#), Ok(("", "hello".to_string())));
        assert_eq!(parse_string(r#"'world'"#), Ok(("", "world".to_string())));
    }

    #[test]
    fn test_parse_qualified_ident() {
        let result = parse_qualified_ident("System.Container");
        assert!(result.is_ok());
        let (_, qid) = result.unwrap();
        assert_eq!(qid.parts, vec!["System", "Container"]);
    }

    #[test]
    fn test_parse_element_def() {
        let input = r#"MySystem = system "My System""#;
        let result = parse_element_def(input);
        assert!(result.is_ok());
        let (_, elem) = result.unwrap();
        assert_eq!(elem.assignment.name, "MySystem");
        assert_eq!(elem.assignment.kind, ElementKind::System);
        assert_eq!(elem.assignment.title, Some("My System".to_string()));
    }

    #[test]
    fn test_parse_relation() {
        let input = r#"SystemA -> SystemB "Uses" "SystemA uses SystemB""#;
        let result = parse_relation(input);
        assert!(result.is_ok());
        let (_, rel) = result.unwrap();
        assert_eq!(rel.from.parts, vec!["SystemA"]);
        assert_eq!(rel.to.parts, vec!["SystemB"]);
        assert_eq!(rel.label, Some("Uses".to_string()));
        assert_eq!(rel.description, Some("SystemA uses SystemB".to_string()));
    }

    #[test]
    fn test_parse_import() {
        let input = r#"import { ServiceA, ServiceB } from "projectA""#;
        let result = parse_import(input);
        assert!(result.is_ok());
        let (_, import_stmt) = result.unwrap();
        assert_eq!(import_stmt.elements.len(), 2);
        assert_eq!(import_stmt.from, "projectA");
    }

    #[test]
    fn test_parse_scenario() {
        let input = r#"scenario LoginFlow "User Login" {
            User -> WebApp "Credentials"
            WebApp -> DB "Verify"
        }"#;
        let result = parse_scenario(input);
        assert!(result.is_ok());
        let (_, scenario) = result.unwrap();
        assert_eq!(scenario.id, "LoginFlow");
        assert_eq!(scenario.label, Some("User Login".to_string()));
        assert_eq!(scenario.steps.len(), 2);
    }

    #[test]
    fn test_parse_with_comments() {
        let input = r#"
        // This is a comment
        MySystem = system "My System"
        /* Multi-line
           comment */
        SystemA -> SystemB "Uses"
        "#;
        let parser = Parser::new("test.sruja");
        let result = parser.parse(input);
        assert!(result.is_ok());
    }
}
