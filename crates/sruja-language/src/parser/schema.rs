//! Schema block parsing.

use nom::{
    bytes::complete::tag,
    character::complete::char,
    combinator::opt,
    multi::many0,
    sequence::{delimited, preceded},
    IResult, Parser,
};
use sruja_diagnostics::SourceLocation;

use super::primitives::{parse_identifier, parse_string, parse_string_array, ws, ws0, ws1};
use crate::ast::{NestingRule, SchemaBlock};

pub(crate) fn parse_schema(input: &str) -> IResult<&str, SchemaBlock> {
    let (input, _) = tag("schema").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, name) = parse_string(input)?;
    let (input, _) = ws0(input)?;

    let (input, (node_kinds, edge_kinds, nesting)) = delimited(
        char('{'),
        (
            opt(preceded(ws, parse_node_kinds)),
            opt(preceded(ws, parse_edge_kinds)),
            opt(preceded(ws, parse_nesting)),
        ),
        preceded(ws0, char('}')),
    )
    .parse(input)?;

    Ok((
        input,
        SchemaBlock {
            location: SourceLocation::new(String::new(), 0, 0),
            name,
            node_kinds: node_kinds.unwrap_or_default(),
            edge_kinds: edge_kinds.unwrap_or_default(),
            nesting: nesting.unwrap_or_default(),
        },
    ))
}

fn parse_node_kinds(input: &str) -> IResult<&str, Vec<String>> {
    let (input, _) = tag("node_kinds").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = char(':').parse(input)?;
    let (input, _) = ws0(input)?;
    parse_string_array(input)
}

fn parse_edge_kinds(input: &str) -> IResult<&str, Vec<String>> {
    let (input, _) = tag("edge_kinds").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = char(':').parse(input)?;
    let (input, _) = ws0(input)?;
    parse_string_array(input)
}

fn parse_nesting(input: &str) -> IResult<&str, Vec<NestingRule>> {
    let (input, _) = tag("nesting").parse(input)?;
    let (input, _) = ws0(input)?;
    delimited(
        char('{'),
        many0(preceded(ws, parse_nesting_rule)),
        preceded(ws0, char('}')),
    )
    .parse(input)
}

fn parse_nesting_rule(input: &str) -> IResult<&str, NestingRule> {
    let (input, parent) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("->").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, child) = parse_identifier(input)?;
    Ok((input, NestingRule { parent, child }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_schema_basic() {
        let input = r#"schema "compliance" {
            node_kinds: ["regulation", "policy"]
            edge_kinds: ["mandates"]
            nesting {
                regulation -> policy
            }
        }"#;
        let result = parse_schema(input);
        assert!(result.is_ok());
        let (_, schema) = result.unwrap();
        assert_eq!(schema.name, "compliance");
        assert_eq!(schema.node_kinds, vec!["regulation", "policy"]);
        assert_eq!(schema.edge_kinds, vec!["mandates"]);
        assert_eq!(schema.nesting.len(), 1);
        assert_eq!(schema.nesting[0].parent, "regulation");
        assert_eq!(schema.nesting[0].child, "policy");
    }
}
