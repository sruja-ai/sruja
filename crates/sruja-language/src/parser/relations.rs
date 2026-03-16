//! Relation and qualified identifier parsers.

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::opt, multi::many0,
    sequence::preceded, IResult, Parser,
};
use sruja_diagnostics::SourceLocation;

use crate::ast::{QualifiedIdent, Relation};

use super::primitives::{parse_identifier, parse_string, parse_tag_array, ws0, ws1};

pub(crate) fn parse_qualified_ident(input: &str) -> IResult<&str, QualifiedIdent> {
    let (input, first) = parse_identifier(input)?;
    let (input, rest) = many0(preceded(char('.'), parse_identifier)).parse(input)?;
    let mut parts = vec![first];
    parts.extend(rest);
    Ok((input, QualifiedIdent::qualified(parts)))
}

pub(crate) fn parse_relation(input: &str) -> IResult<&str, Relation> {
    let (input, from) = parse_qualified_ident(input)?;
    let (input, _) = preceded(ws0, tag("->")).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, to) = parse_qualified_ident(input)?;
    let (input, _) = ws0(input)?;
    let (input, label) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, technology) = opt(preceded(
        alt((tag("technology"), tag("tech"))),
        preceded(ws1, parse_string),
    ))
    .parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, tags) = opt(parse_tag_array).parse(input)?;

    Ok((
        input,
        Relation {
            location: SourceLocation::new(String::new(), 0, 0),
            from,
            to,
            label,
            description,
            technology,
            tags: tags.unwrap_or_default(),
        },
    ))
}
