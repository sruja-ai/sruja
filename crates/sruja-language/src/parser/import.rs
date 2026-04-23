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
        nom::sequence::tuple((
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
        )),
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


pub(crate) fn parse_import_element(input: &str) -> IResult<&str, ImportElement> {
    use nom::branch::alt;
    alt((
        value(ImportElement::Wildcard, char('*')),
        map(parse_identifier, ImportElement::Ident),
    ))
    .parse(input)
}
