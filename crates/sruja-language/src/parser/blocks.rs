//! Block parsers: kv blocks, metadata, constraints, conventions, style.

use std::collections::HashMap;

use nom::{
    bytes::complete::tag, character::complete::char, combinator::map, multi::many0,
    sequence::preceded, IResult, Parser,
};
use sruja_diagnostics::SourceLocation;

use crate::ast::{
    ConstraintEntry, ConstraintsBlock, ConventionEntry, ConventionsBlock, Incident, MetaEntry, MetadataBlock,
    QualifiedIdent, StyleDecl,
};

use super::primitives::{
    parse_identifier, parse_kv_string, parse_string, parse_string_array, parse_tag_array, ws, ws0,
    ws1,
};
use super::relations::parse_qualified_ident;

enum MaybeKeyedString {
    Keyed(String, String),
    Bare(String),
}

pub(crate) fn parse_kv_string_block(input: &str) -> IResult<&str, Vec<(String, String)>> {
    use nom::sequence::delimited;
    delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )
    .parse(input)
}

pub(crate) fn parse_metadata_block(input: &str) -> IResult<&str, MetadataBlock> {
    use nom::sequence::delimited;
    let (input, _) = tag("metadata").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, entries) = delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_metadata_entry)),
        preceded(ws0, char('}')),
    )
    .parse(input)?;
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
    ))
    .parse(input)?;
    Ok((input, MetaEntry { key, value }))
}

pub(crate) fn parse_constraints_block(input: &str) -> IResult<&str, ConstraintsBlock> {
    use nom::branch::alt;
    use nom::sequence::delimited;
    let (input, _) = tag("constraints").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, raw_entries) = delimited(
        char('{'),
        many0(preceded(
            ws,
            alt((
                map(parse_constraint_entry, |e| {
                    MaybeKeyedString::Keyed(e.key, e.value)
                }),
                map(parse_string, MaybeKeyedString::Bare),
            )),
        )),
        preceded(ws0, char('}')),
    )
    .parse(input)?;

    let mut entries = Vec::with_capacity(raw_entries.len());
    for (idx, raw) in raw_entries.into_iter().enumerate() {
        match raw {
            MaybeKeyedString::Keyed(key, value) => entries.push(ConstraintEntry { key, value }),
            MaybeKeyedString::Bare(value) => entries.push(ConstraintEntry {
                key: format!("C{}", idx + 1),
                value,
            }),
        }
    }
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
    use nom::branch::alt;
    use nom::sequence::delimited;
    let (input, _) = tag("conventions").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, raw_entries) = delimited(
        char('{'),
        many0(preceded(
            ws,
            alt((
                map(parse_convention_entry, |e| {
                    MaybeKeyedString::Keyed(e.key, e.value)
                }),
                map(parse_string, MaybeKeyedString::Bare),
            )),
        )),
        preceded(ws0, char('}')),
    )
    .parse(input)?;

    let mut entries = Vec::with_capacity(raw_entries.len());
    for (idx, raw) in raw_entries.into_iter().enumerate() {
        match raw {
            MaybeKeyedString::Keyed(key, value) => entries.push(ConventionEntry { key, value }),
            MaybeKeyedString::Bare(value) => entries.push(ConventionEntry {
                key: format!("V{}", idx + 1),
                value,
            }),
        }
    }
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

pub(crate) fn parse_incident(input: &str) -> IResult<&str, Incident> {
    use nom::branch::alt;
    let (input, _) = tag("incident").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = parse_string(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = char('{').parse(input)?;

    let mut date = None;
    let mut severity = None;
    let mut affected = Vec::new();
    let mut cause = None;
    let mut resolution = None;
    let mut lesson = None;

    let mut current = input;
    loop {
        let (rest, _) = ws(current)?;
        if rest.is_empty() {
            break;
        }
        if rest.trim_start().starts_with('}') {
            current = rest;
            break;
        }

        let result: IResult<&str, (&str, String, Vec<QualifiedIdent>)> = alt((
            map(
                preceded(tag("date"), preceded(ws1, parse_string)),
                |s| ("date", s, vec![]),
            ),
            map(
                preceded(tag("severity"), preceded(ws1, parse_string)),
                |s| ("severity", s, vec![]),
            ),
            map(
                preceded(tag("affected"), preceded(ws0, parse_qualified_ident_array)),
                |arr| ("affected", String::new(), arr),
            ),
            map(
                preceded(tag("cause"), preceded(ws1, parse_string)),
                |s| ("cause", s, vec![]),
            ),
            map(
                preceded(tag("resolution"), preceded(ws1, parse_string)),
                |s| ("resolution", s, vec![]),
            ),
            map(
                preceded(tag("lesson"), preceded(ws1, parse_string)),
                |s| ("lesson", s, vec![]),
            ),
        ))
        .parse(rest);

        match result {
            Ok((next, (key, s, arr))) => {
                match key {
                    "date" => date = Some(s),
                    "severity" => severity = Some(s),
                    "affected" => affected = arr,
                    "cause" => cause = Some(s),
                    "resolution" => resolution = Some(s),
                    "lesson" => lesson = Some(s),
                    _ => {}
                }
                current = next;
            }
            Err(_) => {
                if let Some(newline_pos) = rest.find('\n') {
                    current = &rest[newline_pos + 1..];
                } else {
                    break;
                }
            }
        }
    }

    let (input, _) = ws0(current)?;
    let (input, _) = char('}').parse(input)?;

    Ok((
        input,
        Incident {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            title,
            date,
            severity,
            affected,
            cause,
            resolution,
            lesson,
        },
    ))
}

fn parse_qualified_ident_array(input: &str) -> IResult<&str, Vec<QualifiedIdent>> {
    use nom::multi::separated_list0;
    use nom::sequence::delimited;
    delimited(
        preceded(ws0, char('[')),
        separated_list0(
            preceded(ws0, char(',')),
            preceded(ws0, parse_qualified_ident),
        ),
        preceded(ws0, char(']')),
    )
    .parse(input)
}

pub(crate) fn parse_style_decl(input: &str) -> IResult<&str, StyleDecl> {
    use nom::sequence::delimited;
    let (input, _) = tag("style").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, selector) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, properties) = delimited(
        char('{'),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )
    .parse(input)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_kv_string_block_empty() {
        let result = parse_kv_string_block("{}");
        assert!(result.is_ok());
        let (_, entries) = result.unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_kv_string_block_single() {
        let result = parse_kv_string_block("{ key \"value\" }");
        assert!(result.is_ok());
        let (_, entries) = result.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "key");
        assert_eq!(entries[0].1, "value");
    }

    #[test]
    fn test_parse_kv_string_block_multiple() {
        let result = parse_kv_string_block("{ key1 \"val1\" key2 \"val2\" }");
        assert!(result.is_ok());
        let (_, entries) = result.unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_parse_metadata_block_empty() {
        let result = parse_metadata_block("metadata {}");
        assert!(result.is_ok());
        let (_, block) = result.unwrap();
        assert!(block.entries.is_empty());
    }

    #[test]
    fn test_parse_metadata_block_with_string() {
        let result = parse_metadata_block("metadata { author \"John\" }");
        assert!(result.is_ok());
        let (_, block) = result.unwrap();
        assert_eq!(block.entries.len(), 1);
        assert_eq!(block.entries[0].key, "author");
        assert_eq!(block.entries[0].value, Some("John".to_string()));
    }

    #[test]
    fn test_parse_constraints_block_empty() {
        let result = parse_constraints_block("constraints {}");
        assert!(result.is_ok());
        let (_, block) = result.unwrap();
        assert!(block.entries.is_empty());
    }

    #[test]
    fn test_parse_constraints_block_with_entries() {
        let result = parse_constraints_block("constraints { max_connections \"100\" }");
        assert!(result.is_ok());
        let (_, block) = result.unwrap();
        assert_eq!(block.entries.len(), 1);
        assert_eq!(block.entries[0].key, "max_connections");
        assert_eq!(block.entries[0].value, "100");
    }

    #[test]
    fn test_parse_conventions_block_empty() {
        let result = parse_conventions_block("conventions {}");
        assert!(result.is_ok());
        let (_, block) = result.unwrap();
        assert!(block.entries.is_empty());
    }

    #[test]
    fn test_parse_conventions_block_with_entries() {
        let result = parse_conventions_block("conventions { naming \"camelCase\" }");
        assert!(result.is_ok());
        let (_, block) = result.unwrap();
        assert_eq!(block.entries.len(), 1);
        assert_eq!(block.entries[0].key, "naming");
        assert_eq!(block.entries[0].value, "camelCase");
    }

    #[test]
    fn test_parse_style_decl_basic() {
        let result = parse_style_decl("style button { color \"blue\" }");
        assert!(result.is_ok());
        let (_, style) = result.unwrap();
        assert_eq!(style.selector, "button");
        assert_eq!(style.properties.get("color"), Some(&"blue".to_string()));
    }

    #[test]
    fn test_parse_style_decl_multiple_props() {
        let result = parse_style_decl("style container { color \"red\" size \"large\" }");
        assert!(result.is_ok());
        let (_, style) = result.unwrap();
        assert_eq!(style.properties.len(), 2);
    }

    #[test]
    fn test_parse_constraint_entry() {
        let result = parse_constraint_entry("timeout \"30s\"");
        assert!(result.is_ok());
        let (_, entry) = result.unwrap();
        assert_eq!(entry.key, "timeout");
        assert_eq!(entry.value, "30s");
    }

    #[test]
    fn test_parse_convention_entry() {
        let result = parse_convention_entry("format \"json\"");
        assert!(result.is_ok());
        let (_, entry) = result.unwrap();
        assert_eq!(entry.key, "format");
        assert_eq!(entry.value, "json");
    }
}
