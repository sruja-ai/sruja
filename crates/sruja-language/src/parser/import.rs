//! Import statement parsers.

use nom::{
    character::complete::char,
    combinator::{map, value},
    multi::separated_list0,
    sequence::preceded,
    IResult, Parser,
};
use sruja_diagnostics::SourceLocation;

use crate::ast::{ImportElement, ImportStatement};

use super::primitives::{parse_identifier, parse_string, ws0, ws1};

pub(crate) fn parse_import(input: &str) -> IResult<&str, ImportStatement> {
    use nom::bytes::complete::tag;
    use nom::sequence::delimited;
    let (input, _) = tag("import").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, elements) = delimited(
        char('{'),
        separated_list0(
            preceded(ws0, char(',')),
            preceded(ws0, parse_import_element),
        ),
        preceded(ws0, char('}')),
    )
    .parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, _) = tag("from").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, from) = parse_string(input)?;

    Ok((
        input,
        ImportStatement {
            location: SourceLocation::new(String::new(), 0, 0),
            elements,
            from,
        },
    ))
}

pub(crate) fn parse_import_element(input: &str) -> IResult<&str, ImportElement> {
    use nom::branch::alt;
    alt((
        value(ImportElement::Wildcard, char('*')),
        map(parse_identifier, ImportElement::Ident),
    ))
    .parse(input)
}
