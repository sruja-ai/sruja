use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::char,
    combinator::{map, opt, recognize, value},
    multi::many0,
    sequence::{delimited, pair, preceded},
    IResult, Parser,
};
use sruja_diagnostics::SourceLocation;

use crate::ast::DeploymentNode;

use super::primitives::{parse_string, ws, ws0, ws1};

fn parse_deployment_ref(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            take_while1(|c: char| c.is_alphabetic() || c == '_'),
            take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-' || c == '.'),
        )),
        |s: &str| s.to_string(),
    )
    .parse(input)
}

pub(crate) fn parse_deployment(input: &str) -> IResult<&str, DeploymentNode> {
    parse_deployment_node(input)
}

fn parse_deployment_node(input: &str) -> IResult<&str, DeploymentNode> {
    let (input, kind) = alt((
        value("deployment", tag("deployment")),
        value("node", tag("node")),
        value("infrastructure", tag("infrastructure")),
        value("containerInstance", tag("containerInstance")),
    ))
    .parse(input)?;

    let (input, _) = ws1(input)?;
    let (input, id) = parse_deployment_ref(input)?;
    let (input, _) = ws0(input)?;

    let (input, first_string) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, second_string) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;

    let (input, children) = opt(delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_deployment_node)),
        preceded(ws0, char('}')),
    ))
    .parse(input)?;

    let (label, technology) = match kind {
        "deployment" | "node" => match (first_string, second_string) {
            (Some(a), Some(b)) => (Some(a), Some(b)),
            (Some(a), None) if a == "infrastructure" || a == "containerInstance" => (None, Some(a)),
            (a, b) => (a, b),
        },
        "infrastructure" => (
            first_string,
            second_string.or_else(|| Some(kind.to_string())),
        ),
        "containerInstance" => (None, second_string.or_else(|| Some(kind.to_string()))),
        other => (
            first_string,
            second_string.or_else(|| Some(other.to_string())),
        ),
    };

    Ok((
        input,
        DeploymentNode {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            label,
            technology,
            children: children.unwrap_or_default(),
        },
    ))
}
