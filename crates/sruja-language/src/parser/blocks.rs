//! Block parsers: kv blocks, metadata, constraints, conventions, style.

use std::collections::HashMap;

use nom::{
    bytes::complete::tag, character::complete::char, combinator::map, multi::many0,
    sequence::preceded, IResult,
};
use sruja_diagnostics::SourceLocation;

use crate::ast::{
    ConstraintEntry, ConstraintsBlock, ConventionEntry, ConventionsBlock, MetaEntry, MetadataBlock,
    StyleDecl,
};

use super::primitives::{
    parse_identifier, parse_kv_string, parse_string, parse_string_array, parse_tag_array, ws, ws0,
    ws1,
};

pub(crate) fn parse_kv_string_block(input: &str) -> IResult<&str, Vec<(String, String)>> {
    use nom::multi::many0 as many0_nom;
    use nom::sequence::delimited;
    delimited(
        preceded(ws0, char('{')),
        many0_nom(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )(input)
}

pub(crate) fn parse_metadata_block(input: &str) -> IResult<&str, MetadataBlock> {
    use nom::multi::many0 as many0_nom;
    use nom::sequence::delimited;
    let (input, _) = tag("metadata")(input)?;
    let (input, _) = ws1(input)?;
    let (input, entries) = delimited(
        preceded(ws0, char('{')),
        many0_nom(preceded(ws, parse_metadata_entry)),
        preceded(ws0, char('}')),
    )(input)?;
    Ok((
        input,
        MetadataBlock {
            location: SourceLocation::new(String::new(), 0, 0),
            entries,
        },
    ))
}

pub(crate) fn parse_metadata_entry(input: &str) -> IResult<&str, MetaEntry> {
    use nom::branch::alt;
    let (input, key) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, value) = alt((
        map(parse_string_array, |arr| Some(arr.join(", "))),
        map(parse_tag_array, |arr| Some(arr.join(", "))),
        map(parse_string, Some),
    ))(input)?;
    Ok((input, MetaEntry { key, value }))
}

pub(crate) fn parse_constraints_block(input: &str) -> IResult<&str, ConstraintsBlock> {
    use nom::multi::many0 as many0_nom;
    use nom::sequence::delimited;
    let (input, _) = tag("constraints")(input)?;
    let (input, _) = ws0(input)?;
    let (input, entries) = delimited(
        char('{'),
        many0_nom(preceded(ws, parse_constraint_entry)),
        preceded(ws0, char('}')),
    )(input)?;
    Ok((
        input,
        ConstraintsBlock {
            location: SourceLocation::new(String::new(), 0, 0),
            entries,
        },
    ))
}

pub(crate) fn parse_constraint_entry(input: &str) -> IResult<&str, ConstraintEntry> {
    let (input, key) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, value) = parse_string(input)?;
    Ok((input, ConstraintEntry { key, value }))
}

pub(crate) fn parse_conventions_block(input: &str) -> IResult<&str, ConventionsBlock> {
    use nom::multi::many0 as many0_nom;
    use nom::sequence::delimited;
    let (input, _) = tag("conventions")(input)?;
    let (input, _) = ws0(input)?;
    let (input, entries) = delimited(
        char('{'),
        many0_nom(preceded(ws, parse_convention_entry)),
        preceded(ws0, char('}')),
    )(input)?;
    Ok((
        input,
        ConventionsBlock {
            location: SourceLocation::new(String::new(), 0, 0),
            entries,
        },
    ))
}

pub(crate) fn parse_convention_entry(input: &str) -> IResult<&str, ConventionEntry> {
    let (input, key) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, value) = parse_string(input)?;
    Ok((input, ConventionEntry { key, value }))
}

pub(crate) fn parse_style_decl(input: &str) -> IResult<&str, StyleDecl> {
    use nom::sequence::delimited;
    let (input, _) = tag("style")(input)?;
    let (input, _) = ws1(input)?;
    let (input, selector) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, properties) = delimited(
        char('{'),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )(input)?;
    let mut props_map = HashMap::new();
    for (k, v) in properties {
        props_map.insert(k, v);
    }
    Ok((
        input,
        StyleDecl {
            location: SourceLocation::new(String::new(), 0, 0),
            selector,
            properties: props_map,
        },
    ))
}
