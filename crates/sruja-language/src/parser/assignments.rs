//! Assignment and flow/scenario/requirement/ADR/policy parsers.

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{map, opt},
    multi::many0,
    sequence::preceded,
    IResult, Parser,
};
use sruja_diagnostics::SourceLocation;

use crate::ast::{Adr, Flow, Policy, Requirement, Scenario, ScenarioStep};

use super::blocks::{parse_kv_string_block, parse_metadata_block};
use super::primitives::{parse_identifier, parse_string, parse_tag_array, ws, ws0, ws1};
use super::relations::parse_qualified_ident;

pub(crate) fn parse_requirement_assignment(input: &str) -> IResult<&str, Requirement> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('=')).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("requirement").parse(input)?;
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

pub(crate) fn parse_adr_assignment(input: &str) -> IResult<&str, Adr> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('=')).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("adr").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;

    let (input, fields) = opt(parse_kv_string_block).parse(input)?;
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

pub(crate) fn parse_flow_assignment(input: &str) -> IResult<&str, Flow> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('=')).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("flow").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, steps) = opt(parse_flow_body).parse(input)?;

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

pub(crate) fn parse_policy_assignment(input: &str) -> IResult<&str, Policy> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('=')).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("policy").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;

    let (input, fields) = opt(parse_kv_string_block).parse(input)?;
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

pub(crate) fn parse_scenario(input: &str) -> IResult<&str, Scenario> {
    let (input, _) = alt((tag("scenario"), tag("story"))).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, id) = opt(parse_identifier).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, steps) = opt(parse_scenario_body).parse(input)?;

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

pub(crate) fn parse_scenario_body(input: &str) -> IResult<&str, Vec<ScenarioStep>> {
    use nom::sequence::delimited;
    delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_scenario_step)),
        preceded(ws0, char('}')),
    )
    .parse(input)
}

pub(crate) fn parse_scenario_step(input: &str) -> IResult<&str, ScenarioStep> {
    let (input, _) = opt(preceded(tag("step"), ws1)).parse(input)?;
    let (input, from) = parse_qualified_ident(input)?;
    let (input, _) = preceded(ws0, tag("->")).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, to) = parse_qualified_ident(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, tags) = opt(parse_tag_array).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, order_raw) =
        opt(preceded(tag("order"), preceded(ws1, parse_string))).parse(input)?;

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

pub(crate) fn parse_flow(input: &str) -> IResult<&str, Flow> {
    let (input, _) = tag("flow").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, id) = opt(parse_identifier).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, steps) = opt(parse_flow_body).parse(input)?;

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

pub(crate) fn parse_flow_body(input: &str) -> IResult<&str, Vec<ScenarioStep>> {
    use nom::sequence::delimited;
    delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_flow_step)),
        preceded(ws0, char('}')),
    )
    .parse(input)
}

fn parse_flow_step(input: &str) -> IResult<&str, ScenarioStep> {
    let (input, _) = opt(preceded(tag("step"), ws1)).parse(input)?;
    let (input, from) = parse_qualified_ident(input)?;
    let (input, _) = preceded(ws0, tag("->")).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, to) = parse_qualified_ident(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string).parse(input)?;

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

pub(crate) fn parse_requirement(input: &str) -> IResult<&str, Requirement> {
    let (input, _) = tag("requirement").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let id_for_title = id.clone();
    let (input, _) = ws0(input)?;
    let (input, r#type) = opt(parse_identifier).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, _body) = opt(parse_requirement_body).parse(input)?;

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

fn parse_requirement_body(input: &str) -> IResult<&str, ()> {
    use nom::sequence::delimited;
    delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_requirement_property)),
        preceded(ws0, char('}')),
    )
    .parse(input)
    .map(|(i, _)| (i, ()))
}

fn parse_requirement_property(input: &str) -> IResult<&str, ()> {
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
    )
    .parse(input)
    .map(|(i, _)| (i, ()))
}

pub(crate) fn parse_adr(input: &str) -> IResult<&str, Adr> {
    let (input, _) = tag("adr").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;

    let (input, fields) = opt(parse_kv_string_block).parse(input)?;
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

pub(crate) fn parse_policy(input: &str) -> IResult<&str, Policy> {
    let (input, _) = tag("policy").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;

    let (input, fields) = opt(parse_kv_string_block).parse(input)?;
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
