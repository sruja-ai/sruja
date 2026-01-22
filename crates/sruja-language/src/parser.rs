//! Parser for Sruja DSL using nom parser combinators
//!
//! This module implements a parser for the Sruja DSL using `nom` parser combinators.
//! It parses source code directly into an AST without a separate lexing phase.

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::{char, multispace0, multispace1},
    combinator::{map, opt, recognize, value},
    multi::{many0, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded},
    IResult,
};
use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
use std::collections::HashMap;

use crate::ast::*;

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
                let trimmed = remaining.trim();
                if !trimmed.is_empty() {
                    // Try to provide more context about what couldn't be parsed
                    let preview = if trimmed.len() > 100 {
                        format!("{}...", &trimmed[..100])
                    } else {
                        trimmed.to_string()
                    };

                    // Count lines to provide better error location
                    let lines_before_remaining = input.len() - remaining.len();
                    let line_number = input[..lines_before_remaining].matches('\n').count();
                    let line_number_u32 = line_number.min(u32::MAX as usize) as u32;

                    return Err(vec![Diagnostic::new(
                        sruja_diagnostics::codes::CODE_SYNTAX_ERROR,
                        Severity::Error,
                        format!(
                            "Unexpected input remaining at line {}: {}",
                            line_number + 1,
                            preview.replace('\n', "\\n").replace('\r', "\\r")
                        ),
                        SourceLocation::new(self.filename.clone(), line_number_u32, 0),
                    )]);
                }
                Ok(program)
            }
            Err(e) => {
                // Try to extract more information from the nom error
                let error_msg = match &e {
                    nom::Err::Error(err) => format!(
                        "Parse error at position {}: {:?}",
                        err.input.len(),
                        err.code
                    ),
                    nom::Err::Failure(err) => format!(
                        "Parse failure at position {}: {:?}",
                        err.input.len(),
                        err.code
                    ),
                    nom::Err::Incomplete(_) => "Incomplete input".to_string(),
                };

                Err(vec![Diagnostic::new(
                    sruja_diagnostics::codes::CODE_SYNTAX_ERROR,
                    Severity::Error,
                    error_msg,
                    SourceLocation::new(self.filename.clone(), 0, 0),
                )])
            }
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
        let comment_result: IResult<&str, &str> = alt((
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
/// Uses a more lenient approach: tries to parse items, and if one fails,
/// attempts to skip to the next line and continue parsing
fn parse_program(input: &str) -> IResult<&str, Program> {
    let (input, _) = ws(input)?;
    let mut items = Vec::new();
    let mut current = input;

    loop {
        // Skip whitespace
        let (rest, _) = ws(current)?;
        if rest.is_empty() {
            break;
        }

        // Try to parse a top-level item
        match parse_top_level_item(rest) {
            Ok((next, item)) => {
                items.push(item);
                current = next;
            }
            Err(_) => {
                // Parsing failed - try to skip to the next line and continue
                // This allows the parser to recover from syntax errors in one item
                // and continue parsing the rest
                if let Some(newline_pos) = rest.find('\n') {
                    // Skip to the next line
                    current = &rest[newline_pos + 1..];
                } else {
                    // No more newlines, we're done (or at the end)
                    // Return what we've parsed so far
                    break;
                }
            }
        }
    }

    Ok((current, Program::with_items(Program::new(), items)))
}

/// Parse a top-level item
fn parse_top_level_item(input: &str) -> IResult<&str, TopLevelItem> {
    alt((
        map(parse_kind_def, TopLevelItem::KindDef),
        map(parse_flow_assignment, TopLevelItem::Flow),
        map(parse_requirement_assignment, TopLevelItem::Requirement),
        map(parse_adr_assignment, TopLevelItem::Adr),
        map(parse_policy_assignment, TopLevelItem::Policy),
        map(parse_overview_block, TopLevelItem::Overview),
        map(parse_element_def, TopLevelItem::ElementDef),
        map(parse_relation, TopLevelItem::Relation),
        map(parse_import, TopLevelItem::Import),
        map(parse_scenario, TopLevelItem::Scenario),
        map(parse_flow, TopLevelItem::Flow),
        map(parse_requirement, TopLevelItem::Requirement),
        map(parse_adr, TopLevelItem::Adr),
        map(parse_policy, TopLevelItem::Policy),
        map(parse_view, TopLevelItem::View),
        map(parse_metadata_block, |m| {
            TopLevelItem::ElementDef(ElementDef {
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
            })
        }), // Temporary conversion
    ))(input)
}

/// Parse a kind definition: `identifier = kind "Title" [description] [technology] [style]`
/// Example: `person = kind "Person"`
fn parse_kind_def(input: &str) -> IResult<&str, ElementKindDef> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('='))(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("kind")(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;

    // For now, just parse the kind from the identifier
    let kind = match id.to_lowercase().as_str() {
        "person" => ElementKind::Person,
        "role" => ElementKind::Role,
        "system" => ElementKind::System,
        "container" => ElementKind::Container,
        "component" => ElementKind::Component,
        "database" => ElementKind::Database,
        "queue" => ElementKind::Queue,
        "externalsystem" | "external_system" => ElementKind::ExternalSystem,
        "datastore" => ElementKind::DataStore,
        _ => ElementKind::Custom(id.clone()),
    };

    Ok((
        input,
        ElementKindDef {
            location: SourceLocation::new(String::new(), 0, 0),
            kind,
            title,
            description: None,
            technology: None,
            style: None,
        },
    ))
}

/// Parse `REQ001 = requirement functional "..."` (preferred authoring form in examples).
fn parse_requirement_assignment(input: &str) -> IResult<&str, Requirement> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('='))(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("requirement")(input)?;
    let (input, _) = ws1(input)?;
    let (input, r#type) = parse_identifier(input)?;
    let (input, _) = ws1(input)?;
    let (input, title) = parse_string(input)?;

    Ok((
        input,
        Requirement {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            title,
            r#type,
            description: None,
            tags: Vec::new(),
        },
    ))
}

/// Parse `ADR001 = adr "Title" { status "..."; ... }` (preferred authoring form in examples).
fn parse_adr_assignment(input: &str) -> IResult<&str, Adr> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('='))(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("adr")(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;

    let (input, fields) = opt(parse_kv_string_block)(input)?;
    let mut adr = Adr {
        location: SourceLocation::new(String::new(), 0, 0),
        id: id.clone(),
        title: title.unwrap_or(id),
        status: None,
        context: None,
        decision: None,
        consequences: None,
    };

    if let Some(kvs) = fields {
        for (k, v) in kvs {
            match k.as_str() {
                "status" => adr.status = Some(v),
                "context" => adr.context = Some(v),
                "decision" => adr.decision = Some(v),
                "consequences" => adr.consequences = Some(v),
                _ => {}
            }
        }
    }

    Ok((input, adr))
}

/// Parse `FlowId = flow "Title" { step ... }` (preferred authoring form in examples).
fn parse_flow_assignment(input: &str) -> IResult<&str, Flow> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('='))(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("flow")(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, steps) = opt(parse_flow_body)(input)?;

    Ok((
        input,
        Flow {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            title: title.unwrap_or_default(),
            description: None,
            steps: steps.unwrap_or_default(),
        },
    ))
}

/// Parse `PolicyId = policy "Title" { category "..."; enforcement "..." }`.
fn parse_policy_assignment(input: &str) -> IResult<&str, Policy> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('='))(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("policy")(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;

    let (input, fields) = opt(parse_kv_string_block)(input)?;
    let mut policy = Policy {
        location: SourceLocation::new(String::new(), 0, 0),
        id: id.clone(),
        title: title.unwrap_or(id),
        category: "general".to_string(),
        enforcement: "warn".to_string(),
        description: None,
    };

    if let Some(kvs) = fields {
        for (k, v) in kvs {
            match k.as_str() {
                "category" => policy.category = v,
                "enforcement" => policy.enforcement = v,
                "description" => policy.description = Some(v),
                _ => {}
            }
        }
    }

    Ok((input, policy))
}

/// Parse a `{ key "value" ... }` block into raw (k,v) pairs.
fn parse_kv_string_block(input: &str) -> IResult<&str, Vec<(String, String)>> {
    delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )(input)
}

/// Parse an overview block.
///
/// The designer demo content uses `overview { ... }` as an extension. For now we
/// primarily need to **consume** the syntax so parsing can proceed; exporters can
/// incrementally start using these fields over time.
///
/// Uses a simple approach: consume everything between the opening and closing braces.
/// This handles arrays, nested structures, etc. by consuming the entire block content.
fn parse_overview_block(input: &str) -> IResult<&str, OverviewBlock> {
    let (input, _) = tag("overview")(input)?;
    let (input, _) = ws0(input)?;

    // Consume the entire block including nested braces/brackets
    // We'll find the matching closing brace by counting depth
    if !input.starts_with('{') {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        )));
    }

    let mut depth = 0;
    let mut in_string = false;
    let mut escape = false;

    for (i, ch) in input.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        match ch {
            '\\' => escape = true,
            '"' => in_string = !in_string,
            '{' if !in_string => {
                depth += 1;
            }
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    // Found matching closing brace
                    let remaining = &input[i + 1..];
                    return Ok((
                        remaining,
                        OverviewBlock {
                            location: SourceLocation::new(String::new(), 0, 0),
                            summary: None,
                            audience: None,
                            scope: None,
                            goals: Vec::new(),
                            non_goals: Vec::new(),
                            risks: Vec::new(),
                        },
                    ));
                }
            }
            _ => {}
        }
    }

    // No matching brace found
    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Tag,
    )))
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

    // If there's a '{' after whitespace, we must parse the body (or consume it)
    // This prevents "Unexpected input remaining" errors when body parsing fails
    let (input, body) = if input.trim_start().starts_with('{') {
        // Try to parse the body properly
        match parse_element_def_body(input) {
            Ok((rest, parsed_body)) => (rest, Some(parsed_body)),
            Err(_) => {
                // If body parsing fails, at least consume the block to allow parsing to continue
                // This is a fallback to prevent parser getting stuck
                let mut depth = 0;
                let mut in_string = false;
                let mut escape = false;
                let mut consumed = 0;

                for (i, ch) in input.char_indices() {
                    if escape {
                        escape = false;
                        continue;
                    }
                    match ch {
                        '\\' => escape = true,
                        '"' => in_string = !in_string,
                        '{' if !in_string => depth += 1,
                        '}' if !in_string => {
                            depth -= 1;
                            if depth == 0 {
                                consumed = i + 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                if consumed > 0 {
                    (&input[consumed..], None)
                } else {
                    // Couldn't find matching brace, return error
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Tag,
                    )));
                }
            }
        }
    } else {
        (input, None)
    };

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
    let (input, _) = preceded(ws0, char('{'))(input)?;

    // Parse body items, but be lenient - if an item fails to parse, skip it and continue
    let mut items = Vec::new();
    let mut current = input;

    loop {
        // Skip whitespace
        let (rest, _) = ws(current)?;
        if rest.is_empty() {
            break;
        }

        // Check if we've reached the closing brace
        if rest.trim_start().starts_with('}') {
            current = rest;
            break;
        }

        // Try to parse an item
        match parse_element_body_item(rest) {
            Ok((next, item)) => {
                items.push(item);
                current = next;
            }
            Err(_) => {
                // Item parsing failed - try to skip until next potential item or closing brace
                // This handles unknown syntax gracefully
                if let Some(close_pos) = rest.find('}') {
                    // Found closing brace - we're done
                    current = &rest[close_pos..];
                    break;
                } else if let Some(newline_pos) = rest.find('\n') {
                    // Skip to next line and try again
                    current = &rest[newline_pos + 1..];
                } else {
                    // No more content, break
                    break;
                }
            }
        }
    }

    // Skip whitespace before closing brace
    let (input, _) = ws0(current)?;
    let (input, _) = char('}')(input)?;

    // Process items and populate body fields
    let mut body = ElementDefBody::default();
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
            // Element bodies carry a `StyleBlock` (properties only). We currently parse
            // `StyleDecl` (selector + properties) and treat it as an element-local style.
            ElementDefBodyItem::Style(s) => {
                body.style = Some(StyleBlock {
                    location: s.location,
                    properties: s.properties,
                })
            }
            ElementDefBodyItem::Scale(s) => body.scale = Some(s),
            // All other body items are handled above
            // This catch-all is kept for future extensibility
            #[allow(unreachable_patterns)]
            _ => {}
        }
    }

    Ok((input, body))
}

fn parse_element_body_item(input: &str) -> IResult<&str, ElementDefBodyItem> {
    alt((
        map(
            preceded(
                alt((tag("description"), tag("desc"))),
                preceded(ws1, parse_string),
            ),
            ElementDefBodyItem::Description,
        ),
        map(
            preceded(
                alt((tag("technology"), tag("tech"))),
                preceded(ws1, parse_string),
            ),
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
        map(
            preceded(tag("p95"), preceded(ws1, parse_string)),
            LatencyItem::P95,
        ),
        map(
            preceded(tag("p99"), preceded(ws1, parse_string)),
            LatencyItem::P99,
        ),
        map(
            preceded(tag("window"), preceded(ws1, parse_string)),
            LatencyItem::Window,
        ),
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

    let mut out = SloCurrent {
        p95: None,
        p99: None,
    };
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

    let order = order_raw.as_deref().and_then(|s| s.parse::<usize>().ok());

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
    // Handle optional "step" keyword (matches parse_scenario_step behavior)
    let (input, _) = opt(preceded(tag("step"), ws1))(input)?;
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
    let id_for_title = id.clone();
    let (input, _) = ws0(input)?;
    let (input, r#type) = opt(parse_identifier)(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, _body) = opt(parse_requirement_body)(input)?;

    Ok((
        input,
        Requirement {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            title: description.clone().unwrap_or(id_for_title),
            r#type: r#type.unwrap_or_else(|| "functional".to_string()),
            description,
            tags: Vec::new(),
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
    // Currently we accept these properties for forward-compatibility, but the nom-based
    // parser doesn't materialize them into the `Requirement` struct yet.
    preceded(
        alt((
            tag("type"),
            tag("description"),
            tag("tags"),
            tag("metadata"),
        )),
        preceded(
            ws0,
            alt((
                map(parse_string, |_| ()),
                map(parse_tag_array, |_| ()),
                map(parse_metadata_block, |_| ()),
            )),
        ),
    )(input)
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
            id: id.clone(),
            title: title.unwrap_or(id),
            status: None,
            context: None,
            decision: None,
            consequences: None,
        },
    ))
}

/// Parse ADR body
fn parse_adr_body(input: &str) -> IResult<&str, ()> {
    // Best-effort: consume arbitrary key/value entries inside `{ ... }`.
    // NOTE: This does not support nested braces; extend if ADRs start embedding blocks.
    delimited(
        preceded(ws0, char('{')),
        opt(take_until("}")),
        preceded(ws0, char('}')),
    )(input)
    .map(|(i, _)| (i, ()))
}

#[allow(dead_code)]
fn parse_adr_property(input: &str) -> IResult<&str, ()> {
    // Forward-compatibility: accept ADR properties even if we don't materialize them yet.
    preceded(
        alt((
            tag("status"),
            tag("context"),
            tag("decision"),
            tag("consequences"),
            tag("tags"),
        )),
        preceded(
            ws0,
            alt((map(parse_string, |_| ()), map(parse_tag_array, |_| ()))),
        ),
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
            id: id.clone(),
            title: title.unwrap_or(id),
            category: "general".to_string(),
            enforcement: "warn".to_string(),
            description,
        },
    ))
}

/// Parse a view definition
/// Supports both: `view id { title "..."; include ... }` and `view id of target { ... }`
fn parse_view(input: &str) -> IResult<&str, ViewDef> {
    let (input, _) = tag("view")(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;

    // Handle optional "of target" syntax: `view id of target`
    let (input, view_of) = opt(preceded(
        preceded(ws0, tag("of")),
        preceded(ws1, parse_qualified_ident),
    ))(input)?;
    let (input, _) = ws0(input)?;

    // Parse body block if present
    let (input, body_fields) = opt(parse_view_body)(input)?;

    let mut title = None;
    let mut includes = None;
    let mut excludes = None;
    let mut description = None;

    if let Some(fields) = body_fields {
        for (k, v) in fields {
            match k.as_str() {
                "title" => title = Some(v),
                "include" => {
                    // Parse include expression from string
                    if v == "*" {
                        includes = Some(vec!["*".to_string()]);
                    } else {
                        includes = Some(v.split(',').map(|s| s.trim().to_string()).collect());
                    }
                }
                "exclude" => {
                    excludes = Some(v.split(',').map(|s| s.trim().to_string()).collect());
                }
                "description" => description = Some(v),
                _ => {} // Ignore other fields like "layout" for now
            }
        }
    }

    let to_expr = |elements: Vec<String>| ViewRuleExpr {
        wildcard: elements.len() == 1 && elements[0] == "*",
        recursive: false,
        elements: if elements.len() == 1 && elements[0] == "*" {
            Vec::new()
        } else {
            elements
        },
    };

    Ok((
        input,
        ViewDef {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            title,
            description,
            view_of: view_of.map(|q| q),
            tags: Vec::new(),
            rules: if includes.is_none() && excludes.is_none() {
                Vec::new()
            } else {
                vec![ViewRule {
                    include: includes.map(to_expr),
                    exclude: excludes.map(to_expr),
                }]
            },
        },
    ))
}

/// Parse view body: `{ title "..."; include ...; layout { ... } }`
/// Consumes the entire body, handling nested braces for layout blocks.
/// For now, just consumes the block; field extraction can be added later.
fn parse_view_body(input: &str) -> IResult<&str, Vec<(String, String)>> {
    if !input.starts_with('{') {
        return Ok((input, Vec::new()));
    }

    // Use brace-counting to find the matching closing brace (same as overview)
    let mut depth = 0;
    let mut in_string = false;
    let mut escape = false;

    for (i, ch) in input.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        match ch {
            '\\' => escape = true,
            '"' => in_string = !in_string,
            '{' if !in_string => {
                depth += 1;
            }
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    // Found matching closing brace
                    let remaining = &input[i + 1..];
                    // For now, return empty fields - we'll extract them later if needed
                    // This allows parsing to proceed without panicking
                    return Ok((remaining, Vec::new()));
                }
            }
            _ => {}
        }
    }

    // No matching brace found
    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Tag,
    )))
}

#[allow(dead_code)]
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
    let (input, value) = alt((parse_string, map(parse_identifier, |s| s)))(input)?;
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
    let (input, technology) = opt(preceded(
        alt((tag("technology"), tag("tech"))),
        preceded(ws1, parse_string),
    ))(input)?;
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
        separated_list0(
            preceded(ws0, char(',')),
            preceded(ws0, parse_import_element),
        ),
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

    Ok((input, MetaEntry { key, value }))
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
#[allow(dead_code)]
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
        assert_eq!(
            parse_identifier("mySystem"),
            Ok(("", "mySystem".to_string()))
        );
        assert_eq!(
            parse_identifier("my-system_123"),
            Ok(("", "my-system_123".to_string()))
        );
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
        assert_eq!(scenario.title, "User Login".to_string());
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
