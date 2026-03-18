//! Element definition, kind, body, SLO, and scale parsers.

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::{map, opt, value},
    multi::many0,
    sequence::preceded,
    IResult, Parser,
};
use sruja_diagnostics::SourceLocation;

use crate::ast::{
    Criticality, ElementAssignment, ElementDef, ElementDefBody, ElementDefBodyItem, ElementKind,
    ElementKindDef, ScaleBlock, SloAvailability, SloBlock, SloCurrent, SloErrorRate, SloLatency,
    SloThroughput, SourceBinding, SourceKind, StyleBlock,
};

use super::assignments::parse_scenario_step;
use super::blocks::{
    parse_constraints_block, parse_conventions_block, parse_metadata_block, parse_style_decl,
};
use super::primitives::{
    parse_identifier, parse_kv_string, parse_string, parse_string_array, parse_tag_array,
    parse_tag_ref, ws, ws0, ws1,
};
use super::relations::parse_relation;

pub(crate) fn parse_kind_def(input: &str) -> IResult<&str, ElementKindDef> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('=')).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("kind").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;

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

pub(crate) fn parse_element_def(input: &str) -> IResult<&str, ElementDef> {
    let (input, name) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('=')).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, kind) = parse_element_kind(input)?;
    let (input, _) = ws0(input)?;
    let (input, sub_kind) = opt(parse_identifier).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, tag_refs) = many0(parse_tag_ref).parse(input)?;
    let (input, _) = ws0(input)?;

    let (input, body) = if input.trim_start().starts_with('{') {
        match parse_element_def_body(input) {
            Ok((rest, parsed_body)) => (rest, Some(parsed_body)),
            Err(_) => {
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
            location: SourceLocation::new(String::new(), 0, 0),
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

pub(crate) fn parse_element_def_unassigned(input: &str) -> IResult<&str, ElementDef> {
    let (input, kind) = parse_unassigned_element_kind(input)?;
    let (input, _) = ws1(input)?;
    let (input, title) = parse_string(input)?;
    let (input, _) = ws0(input)?;
    let (input, tag_refs) = many0(parse_tag_ref).parse(input)?;
    let (input, _) = ws0(input)?;

    let (input, body) = if input.trim_start().starts_with('{') {
        match parse_element_def_body(input) {
            Ok((rest, parsed_body)) => (rest, Some(parsed_body)),
            Err(_) => {
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

    let name = derive_id_from_title(&title);

    Ok((
        input,
        ElementDef {
            location: SourceLocation::new(String::new(), 0, 0),
            assignment: ElementAssignment {
                location: SourceLocation::new(String::new(), 0, 0),
                name,
                kind,
                sub_kind: None,
                title: Some(title),
                tag_refs,
                body,
            },
        },
    ))
}

fn parse_unassigned_element_kind(input: &str) -> IResult<&str, ElementKind> {
    alt((
        value(ElementKind::Person, tag("person")),
        value(ElementKind::Role, tag("role")),
        value(ElementKind::System, tag("system")),
        value(ElementKind::Container, tag("container")),
        value(ElementKind::Component, tag("component")),
        value(ElementKind::Database, tag("database")),
        value(ElementKind::Queue, tag("queue")),
        value(ElementKind::Custom("service".to_string()), tag("service")),
        value(ElementKind::Custom("feedback".to_string()), tag("feedback")),
        value(
            ElementKind::Custom("causal_loop".to_string()),
            tag("causal_loop"),
        ),
        value(ElementKind::Custom("variable".to_string()), tag("variable")),
    ))
    .parse(input)
}

fn derive_id_from_title(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    let mut prev_underscore = false;

    for ch in title.chars() {
        if ch.is_alphanumeric() {
            out.push(ch);
            prev_underscore = false;
        } else if (ch == '_' || ch == '-' || ch.is_whitespace())
            && !prev_underscore
            && !out.is_empty()
        {
            out.push('_');
            prev_underscore = true;
        }
    }

    while out.ends_with('_') {
        out.pop();
    }

    if out.is_empty() {
        return "Element".to_string();
    }
    if !out
        .chars()
        .next()
        .is_some_and(|c| c.is_alphabetic() || c == '_')
    {
        out.insert(0, '_');
    }
    out
}

pub(crate) fn parse_element_kind(input: &str) -> IResult<&str, ElementKind> {
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
    ))
    .parse(input)
}

pub(crate) fn parse_element_def_body(input: &str) -> IResult<&str, ElementDefBody> {
    let (input, _) = preceded(ws0, char('{')).parse(input)?;

    let mut items = Vec::new();
    let mut current = input;

    loop {
        let (rest, _) = ws(current)?;
        if rest.is_empty() {
            break;
        }
        if rest.trim_start().starts_with('}') {
            current = rest;
            break;
        }
        match parse_element_body_item(rest) {
            Ok((next, item)) => {
                items.push(item);
                current = next;
            }
            Err(_) => {
                if let Some(newline_pos) = rest.find('\n') {
                    current = &rest[newline_pos + 1..];
                } else {
                    break;
                }
            }
        }
    }

    let (input, _) = ws0(current)?;
    let (input, _) = char('}').parse(input)?;

    let mut body = ElementDefBody::default();
    for item in items {
        match item {
            ElementDefBodyItem::Description(d) => body.description = Some(d),
            ElementDefBodyItem::Technology(t) => body.technology = Some(t),
            ElementDefBodyItem::Doc(d) => body.doc = Some(d),
            ElementDefBodyItem::Metadata(m) => body.metadata = m.entries,
            ElementDefBodyItem::Slo(s) => body.slo = Some(*s),
            ElementDefBodyItem::ElementDef(e) => body.items.push(ElementDefBodyItem::ElementDef(e)),
            ElementDefBodyItem::Relation(r) => body.items.push(ElementDefBodyItem::Relation(r)),
            ElementDefBodyItem::Step(s) => body.items.push(ElementDefBodyItem::Step(s)),
            ElementDefBodyItem::Constraints(c) => body.constraints = c.entries,
            ElementDefBodyItem::Conventions(c) => body.conventions = c.entries,
            ElementDefBodyItem::Style(s) => {
                body.style = Some(StyleBlock {
                    location: s.location,
                    properties: s.properties,
                })
            }
            ElementDefBodyItem::Scale(s) => body.scale = Some(s),
            ElementDefBodyItem::Tags(t) => body.items.push(ElementDefBodyItem::Tags(t)),
            ElementDefBodyItem::CanonicalId(id) => body.canonical_id = Some(id),
            ElementDefBodyItem::Aliases(a) => body.aliases = a,
            ElementDefBodyItem::Owner(o) => body.owner = Some(o),
            ElementDefBodyItem::Domain(d) => body.domain = Some(d),
            ElementDefBodyItem::Criticality(c) => body.criticality = Some(c),
            ElementDefBodyItem::Source(s) => body.sources.push(s),
            #[allow(unreachable_patterns)]
            _ => {}
        }
    }

    Ok((input, body))
}

fn parse_inline_step(input: &str) -> IResult<&str, crate::ast::ScenarioStep> {
    preceded(tag("step"), preceded(ws1, parse_scenario_step)).parse(input)
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
        map(
            preceded(
                alt((tag("doc"), tag("documentation"))),
                preceded(ws1, parse_string),
            ),
            ElementDefBodyItem::Doc,
        ),
        map(parse_metadata_block, ElementDefBodyItem::Metadata),
        map(parse_slo_block, |s| ElementDefBodyItem::Slo(Box::new(s))),
        map(parse_element_def, |e| {
            ElementDefBodyItem::ElementDef(Box::new(e))
        }),
        map(parse_element_def_unassigned, |e| {
            ElementDefBodyItem::ElementDef(Box::new(e))
        }),
        map(parse_inline_step, ElementDefBodyItem::Step),
        map(parse_relation, ElementDefBodyItem::Relation),
        map(parse_constraints_block, ElementDefBodyItem::Constraints),
        map(parse_conventions_block, ElementDefBodyItem::Conventions),
        map(parse_style_decl, ElementDefBodyItem::Style),
        map(parse_scale_block, ElementDefBodyItem::Scale),
        map(
            preceded(
                alt((tag("tags"), tag("tag"))),
                preceded(ws0, opt(alt((parse_string_array, parse_tag_array)))),
            ),
            |t| ElementDefBodyItem::Tags(t.unwrap_or_default()),
        ),
        // New fields for architecture index
        map(
            preceded(tag("id"), preceded(ws1, parse_string)),
            ElementDefBodyItem::CanonicalId,
        ),
        map(
            preceded(tag("aliases"), preceded(ws0, parse_string_array)),
            ElementDefBodyItem::Aliases,
        ),
        map(
            preceded(tag("owner"), preceded(ws1, parse_string)),
            ElementDefBodyItem::Owner,
        ),
        map(
            preceded(tag("domain"), preceded(ws1, parse_string)),
            ElementDefBodyItem::Domain,
        ),
        map(parse_criticality, ElementDefBodyItem::Criticality),
        map(parse_source_binding, ElementDefBodyItem::Source),
    ))
    .parse(input)
}

pub(crate) fn parse_slo_block(input: &str) -> IResult<&str, SloBlock> {
    use nom::sequence::delimited;
    let (input, _) = tag("slo").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, items) = delimited(
        char('{'),
        many0(preceded(ws, parse_slo_item)),
        preceded(ws0, char('}')),
    )
    .parse(input)?;

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
    ))
    .parse(input)
}

fn parse_slo_availability(input: &str) -> IResult<&str, SloAvailability> {
    use nom::sequence::delimited;
    let (input, _) = tag("availability").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, entries) = delimited(
        char('{'),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )
    .parse(input)?;

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
    use nom::sequence::delimited;
    let (input, _) = tag("latency").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, items) = delimited(
        char('{'),
        many0(preceded(ws, parse_latency_item)),
        preceded(ws0, char('}')),
    )
    .parse(input)?;

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
    ))
    .parse(input)
}

fn parse_slo_current(input: &str) -> IResult<&str, SloCurrent> {
    use nom::sequence::delimited;
    let (input, _) = tag("current").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, entries) = delimited(
        char('{'),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )
    .parse(input)?;

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
    use nom::sequence::delimited;
    let (input, _) = tag("errorRate").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, entries) = delimited(
        char('{'),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )
    .parse(input)?;

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
    use nom::sequence::delimited;
    let (input, _) = tag("throughput").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, entries) = delimited(
        char('{'),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )
    .parse(input)?;

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

pub(crate) fn parse_scale_block(input: &str) -> IResult<&str, ScaleBlock> {
    use nom::sequence::delimited;
    let (input, _) = tag("scale").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, items) = delimited(
        char('{'),
        many0(preceded(ws, parse_scale_item)),
        preceded(ws0, char('}')),
    )
    .parse(input)?;
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
        map(digit1, |s: &str| s.to_string()),
    ))
    .parse(input)?;
    Ok((input, (key, value)))
}

fn parse_criticality(input: &str) -> IResult<&str, Criticality> {
    let (input, _) = tag("criticality").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, level) = parse_identifier(input)?;
    let crit = match level.to_lowercase().as_str() {
        "low" => Criticality::Low,
        "medium" | "med" => Criticality::Medium,
        "high" => Criticality::High,
        "critical" => Criticality::Critical,
        _ => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )))
        }
    };
    Ok((input, crit))
}

fn parse_source_binding(input: &str) -> IResult<&str, SourceBinding> {
    let (input, _) = tag("source").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, kind_str) = parse_identifier(input)?;
    let (input, _) = ws1(input)?;
    let (input, path) = parse_string(input)?;
    let kind = SourceKind::parse(&kind_str);
    Ok((
        input,
        SourceBinding {
            kind,
            path,
            description: None,
        },
    ))
}
