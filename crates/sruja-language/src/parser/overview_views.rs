//! Overview block and view definition parsers.

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::map,
    multi::{many0, separated_list1},
    sequence::{delimited, preceded},
    IResult, Parser,
};
use sruja_diagnostics::SourceLocation;

use crate::ast::{ViewDef, ViewRule, ViewRuleExpr};

use super::primitives::{parse_identifier, parse_string, parse_string_array, ws, ws0, ws1};
use super::relations::parse_qualified_ident;

use crate::ast::OverviewBlock;

#[derive(Debug, Clone)]
pub(crate) enum OverviewItem {
    Summary(String),
    Audience(String),
    Scope(String),
    Goals(Vec<String>),
    NonGoals(Vec<String>),
    Risks(Vec<String>),
}

pub(crate) fn parse_overview_block(input: &str) -> IResult<&str, OverviewBlock> {
    let (input, _) = tag("overview").parse(input)?;
    let (input, _) = ws0(input)?;

    let (input, items) = delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_overview_item)),
        preceded(ws0, char('}')),
    )
    .parse(input)?;

    let mut overview = OverviewBlock {
        location: SourceLocation::new(String::new(), 0, 0),
        summary: None,
        audience: None,
        scope: None,
        goals: Vec::new(),
        non_goals: Vec::new(),
        risks: Vec::new(),
    };

    for item in items {
        match item {
            OverviewItem::Summary(s) => overview.summary = Some(s),
            OverviewItem::Audience(a) => overview.audience = Some(a),
            OverviewItem::Scope(s) => overview.scope = Some(s),
            OverviewItem::Goals(g) => overview.goals = g,
            OverviewItem::NonGoals(ng) => overview.non_goals = ng,
            OverviewItem::Risks(r) => overview.risks = r,
        }
    }

    Ok((input, overview))
}

fn parse_overview_item(input: &str) -> IResult<&str, OverviewItem> {
    alt((
        map(
            preceded(tag("summary"), preceded(ws1, parse_string)),
            OverviewItem::Summary,
        ),
        map(
            preceded(tag("audience"), preceded(ws1, parse_string)),
            OverviewItem::Audience,
        ),
        map(
            preceded(tag("scope"), preceded(ws1, parse_string)),
            OverviewItem::Scope,
        ),
        map(
            preceded(tag("goals"), preceded(ws1, parse_string_array)),
            OverviewItem::Goals,
        ),
        map(
            preceded(
                alt((tag("nonGoals"), tag("non_goals"))),
                preceded(ws1, parse_string_array),
            ),
            OverviewItem::NonGoals,
        ),
        map(
            preceded(tag("risks"), preceded(ws1, parse_string_array)),
            OverviewItem::Risks,
        ),
    ))
    .parse(input)
}

pub(crate) fn parse_view(input: &str) -> IResult<&str, ViewDef> {
    use nom::combinator::opt;

    if input.starts_with("view ") {
        return parse_view_block_syntax(input);
    }

    let (input, id) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("=").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("view").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, title) = parse_string(input)?;
    let (input, _) = ws0(input)?;

    let (input, view_of) = opt(preceded(
        preceded(ws0, tag("of")),
        preceded(ws1, parse_qualified_ident),
    ))
    .parse(input)?;
    let (input, _) = ws0(input)?;

    let (input, body_fields) = opt(parse_view_body).parse(input)?;

    let mut includes = None;
    let mut excludes = None;
    let mut description = None;

    if let Some(fields) = body_fields {
        for (k, v) in fields {
            match k.as_str() {
                "include" => {
                    if v.trim() == "*" {
                        includes = Some(vec!["*".to_string()]);
                    } else {
                        let elements = v
                            .split(&[',', ' '][..])
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                            .collect();
                        includes = Some(elements);
                    }
                }
                "exclude" => {
                    let elements = v
                        .split(&[',', ' '][..])
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect();
                    excludes = Some(elements);
                }
                "description" => description = Some(v),
                _ => {}
            }
        }
    }

    let to_expr = |elements: Vec<String>| ViewRuleExpr {
        wildcard: elements.len() == 1 && elements[0] == "*",
        recursive: false,
        elements: if elements.len() == 1 && elements[0] == "*" {
            Vec::new()
        } else {
            elements
        },
    };

    Ok((
        input,
        ViewDef {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            title,
            description,
            view_of,
            tags: Vec::new(),
            rules: if includes.is_none() && excludes.is_none() {
                Vec::new()
            } else {
                vec![ViewRule {
                    include: includes.map(to_expr),
                    exclude: excludes.map(to_expr),
                }]
            },
        },
    ))
}

fn parse_view_block_syntax(input: &str) -> IResult<&str, ViewDef> {
    use nom::combinator::opt;
    let (input, _) = tag("view").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;

    let (input, view_of) = opt(preceded(
        preceded(ws0, tag("of")),
        preceded(ws1, parse_qualified_ident),
    ))
    .parse(input)?;
    let (input, _) = ws0(input)?;

    let (input, body_items) = delimited(
        char('{'),
        many0(preceded(ws, parse_view_block_item)),
        preceded(ws0, char('}')),
    )
    .parse(input)?;

    let mut title = id.clone();
    let mut description = None;
    let mut includes = None;
    let mut excludes = None;

    for (key, value) in body_items {
        match key.as_str() {
            "title" => title = value,
            "description" => description = Some(value),
            "include" => {
                if value.trim() == "*" {
                    includes = Some(vec!["*".to_string()]);
                } else {
                    let elements = value
                        .split(&[',', ' '][..])
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect();
                    includes = Some(elements);
                }
            }
            "exclude" => {
                let elements = value
                    .split(&[',', ' '][..])
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();
                excludes = Some(elements);
            }
            _ => {}
        }
    }

    let to_expr = |elements: Vec<String>| ViewRuleExpr {
        wildcard: elements.len() == 1 && elements[0] == "*",
        recursive: false,
        elements: if elements.len() == 1 && elements[0] == "*" {
            Vec::new()
        } else {
            elements
        },
    };

    Ok((
        input,
        ViewDef {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            title,
            description,
            view_of,
            tags: Vec::new(),
            rules: if includes.is_none() && excludes.is_none() {
                Vec::new()
            } else {
                vec![ViewRule {
                    include: includes.map(to_expr),
                    exclude: excludes.map(to_expr),
                }]
            },
        },
    ))
}

fn parse_view_block_item(input: &str) -> IResult<&str, (String, String)> {
    let (input, key) = parse_identifier(input)?;
    let (input, _) = ws1(input)?;

    let (input, value) = if input.starts_with('"') {
        map(parse_string, |s| s).parse(input)?
    } else {
        map(parse_view_identifier_or_wildcard, |s| s).parse(input)?
    };

    Ok((input, (key, value)))
}

fn parse_view_body(input: &str) -> IResult<&str, Vec<(String, String)>> {
    if !input.starts_with('{') {
        return Ok((input, Vec::new()));
    }

    let (input, items) = delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_view_body_item)),
        preceded(ws0, char('}')),
    )
    .parse(input)?;

    Ok((input, items))
}

fn parse_view_body_item(input: &str) -> IResult<&str, (String, String)> {
    let (input, key) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;

    let (input, value) = if input.starts_with('"') {
        map(parse_string, |s| s).parse(input)?
    } else {
        map(parse_view_identifier_or_wildcard, |s| s).parse(input)?
    };

    Ok((input, (key, value)))
}

fn parse_view_identifier_or_wildcard(input: &str) -> IResult<&str, String> {
    use nom::combinator::value;
    alt((
        value("*".to_string(), char('*')),
        map(parse_qualified_ident, |q| q.as_string()),
        map(separated_list1(ws1, parse_qualified_ident), |idents| {
            idents
                .iter()
                .map(|q| q.as_string())
                .collect::<Vec<_>>()
                .join(" ")
        }),
    ))
    .parse(input)
}
