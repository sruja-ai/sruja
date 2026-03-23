//! Primitive parsers: whitespace, comments, identifiers, strings, tags.

use nom::{
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::{char, multispace0, multispace1},
    combinator::map,
    multi::separated_list0,
    sequence::{delimited, pair, preceded},
    IResult, Parser,
};

pub(crate) fn line_to_byte_offset(input: &str, line_index: usize) -> usize {
    if line_index == 0 {
        return 0;
    }
    let mut count = 0;
    for (i, c) in input.char_indices() {
        if c == '\n' {
            count += 1;
            if count == line_index {
                return i + 1;
            }
        }
    }
    input.len()
}

pub(crate) fn skip_whitespace_and_comments(input: &str) -> IResult<&str, ()> {
    use nom::branch::alt;
    let mut input = input;
    loop {
        let (new_input, _) = multispace0.parse(input)?;
        input = new_input;
        let comment_result: IResult<&str, &str> = alt((
            preceded(tag("//"), take_until("\n")),
            delimited(tag("/*"), take_until("*/"), tag("*/")),
        ))
        .parse(input);
        match comment_result {
            Ok((new_input, _)) => input = new_input,
            Err(_) => break,
        }
    }
    Ok((input, ()))
}

pub(crate) fn ws(input: &str) -> IResult<&str, ()> {
    skip_whitespace_and_comments(input)
}

pub(crate) fn ws0(input: &str) -> IResult<&str, ()> {
    multispace0.parse(input).map(|(i, _)| (i, ()))
}

pub(crate) fn ws1(input: &str) -> IResult<&str, ()> {
    multispace1.parse(input).map(|(i, _)| (i, ()))
}

pub(crate) fn parse_identifier(input: &str) -> IResult<&str, String> {
    use nom::combinator::recognize;
    map(
        recognize(pair(
            take_while1(|c: char| c.is_alphabetic() || c == '_'),
            take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
        )),
        |s: &str| s.to_string(),
    )
    .parse(input)
}

pub(crate) fn parse_string(input: &str) -> IResult<&str, String> {
    use nom::branch::alt;
    alt((
        delimited(char('"'), take_until("\""), char('"')),
        delimited(char('\''), take_until("'"), char('\'')),
    ))
    .map(|s: &str| s.to_string())
    .parse(input)
}

pub(crate) fn parse_string_array(input: &str) -> IResult<&str, Vec<String>> {
    delimited(
        preceded(ws0, char('[')),
        separated_list0(preceded(ws0, char(',')), preceded(ws0, parse_string)),
        preceded(ws0, char(']')),
    )
    .parse(input)
}

pub(crate) fn parse_kv_string(input: &str) -> IResult<&str, (String, String)> {
    use nom::branch::alt;
    let (input, key) = parse_identifier(input)?;
    let (input, _) = ws1(input)?;
    let (input, value) = alt((parse_string, parse_identifier)).parse(input)?;
    Ok((input, (key, value)))
}

pub(crate) fn parse_tag_ref(input: &str) -> IResult<&str, String> {
    use nom::branch::alt;
    let (input, prefix) = alt((char('#'), char('@'))).parse(input)?;
    let (input, ident) = parse_identifier(input)?;
    Ok((input, format!("{}{}", prefix, ident)))
}

pub(crate) fn parse_tag_array(input: &str) -> IResult<&str, Vec<String>> {
    use nom::branch::alt;
    delimited(
        preceded(ws0, char('[')),
        separated_list0(
            preceded(ws0, char(',')),
            preceded(ws0, alt((parse_tag_ref, parse_identifier))),
        ),
        preceded(ws0, char(']')),
    )
    .parse(input)
}
