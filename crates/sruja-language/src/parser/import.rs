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
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::sequence::delimited;

    let (input, _) = tag("import").parse(input)?;
    let (input, _) = ws1(input)?;

    // Try structured import: import { A, B } from "path"
    let structured = map(
        (
            delimited(
                char('{'),
                separated_list0(
                    preceded(ws0, char(',')),
                    preceded(ws0, parse_import_element),
                ),
                preceded(ws0, char('}')),
            ),
            ws1,
            tag("from"),
            ws1,
            parse_string,
        ),
        |(elements, _, _, _, from)| ImportStatement {
            location: SourceLocation::new(String::new(), 0, 0),
            elements,
            from,
        },
    );

    // Try simple import: import "path"
    let simple = map(parse_string, |path| ImportStatement {
        location: SourceLocation::new(String::new(), 0, 0),
        elements: vec![ImportElement::Wildcard],
        from: path,
    });

    alt((structured, simple)).parse(input)
}

/// Parse a keyword only if not followed by an identifier character.
fn parse_keyword<'a>(keyword: &'static str) -> impl Fn(&'a str) -> IResult<&'a str, &'a str> {
    move |input: &'a str| {
        let (rest, matched) = nom::bytes::complete::tag(keyword)(input)?;
        // Check that the next char is not an identifier char (alphanumeric, _, -)
        if rest.is_empty() {
            Ok((rest, matched))
        } else {
            let next = rest.chars().next().unwrap();
            if next.is_alphanumeric() || next == '_' || next == '-' {
                Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Tag,
                )))
            } else {
                Ok((rest, matched))
            }
        }
    }
}

pub(crate) fn parse_import_element(input: &str) -> IResult<&str, ImportElement> {
    use nom::branch::alt;
    alt((
        value(ImportElement::Wildcard, char('*')),
        value(ImportElement::Boundary, parse_keyword("boundary")),
        value(ImportElement::Policy, parse_keyword("policy")),
        map(parse_identifier, ImportElement::Ident),
    ))
    .parse(input)
}
