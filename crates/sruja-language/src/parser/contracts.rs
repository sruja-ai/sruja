//! API contract parser.

use nom::{
    bytes::complete::tag, character::complete::char, combinator::map, multi::many0,
    sequence::{delimited, preceded}, IResult, Parser,
};
use sruja_diagnostics::SourceLocation;

use crate::ast::{Contract, ContractError, ContractField};
use super::primitives::{parse_string, ws, ws0, ws1};

pub(crate) fn parse_contract(input: &str) -> IResult<&str, Contract> {
    let (input, _) = tag("contract").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, name) = parse_string(input)?;
    let (input, _) = ws0(input)?;
    
    let (input, body) = delimited(
        char('{'),
        many0(preceded(ws, parse_contract_item)),
        preceded(ws0, char('}')),
    ).parse(input)?;

    let mut description = None;
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    let mut errors = Vec::new();
    let mut constraints = Vec::new();

    for item in body {
        match item {
            ContractItem::Description(d) => description = Some(d),
            ContractItem::Input(fs) => inputs = fs,
            ContractItem::Output(fs) => outputs = fs,
            ContractItem::Error(es) => errors = es,
            ContractItem::Constraint(c) => constraints.push(c),
        }
    }

    Ok((
        input,
        Contract {
            location: SourceLocation::new(String::new(), 0, 0),
            name,
            description,
            inputs,
            outputs,
            errors,
            constraints,
        },
    ))
}

enum ContractItem {
    Description(String),
    Input(Vec<ContractField>),
    Output(Vec<ContractField>),
    Error(Vec<ContractError>),
    Constraint(String),
}

fn parse_contract_item(input: &str) -> IResult<&str, ContractItem> {
    use nom::branch::alt;
    alt((
        map(preceded((tag("description"), ws1), parse_string), ContractItem::Description),
        map(preceded((tag("input"), ws0), parse_field_block), ContractItem::Input),
        map(preceded((tag("output"), ws0), parse_field_block), ContractItem::Output),
        map(preceded((tag("error"), ws0), parse_error_block), ContractItem::Error),
        map(preceded((tag("constraint"), ws1), parse_string), ContractItem::Constraint),
    )).parse(input)
}

fn parse_field_block(input: &str) -> IResult<&str, Vec<ContractField>> {
    delimited(
        char('{'),
        many0(preceded(ws, parse_contract_field)),
        preceded(ws0, char('}')),
    ).parse(input)
}

fn parse_contract_field(input: &str) -> IResult<&str, ContractField> {
    use super::primitives::parse_identifier;
    let (input, name) = parse_identifier(input)?;
    let (input, _) = ws1(input)?;
    let (input, spec) = parse_string(input)?;
    Ok((input, ContractField { name, spec }))
}

fn parse_error_block(input: &str) -> IResult<&str, Vec<ContractError>> {
    delimited(
        char('{'),
        many0(preceded(ws, parse_contract_error)),
        preceded(ws0, char('}')),
    ).parse(input)
}

fn parse_contract_error(input: &str) -> IResult<&str, ContractError> {
    let (input, code) = parse_string(input)?;
    let (input, _) = ws1(input)?;
    let (input, description) = parse_string(input)?;
    Ok((input, ContractError { code, description }))
}
