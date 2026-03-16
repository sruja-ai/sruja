//! Feedback loop and causal loop parsers.

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{map, opt, value},
    multi::many0,
    sequence::{delimited, preceded},
    IResult, Parser,
};
use sruja_diagnostics::SourceLocation;

use crate::ast::{
    CausalLoop, CausalLoopVariable, CausalPolarity, CausalRelationship, FeedbackLoop,
    FeedbackLoopType,
};

use super::primitives::{parse_identifier, parse_string, ws, ws0, ws1};
use super::relations::parse_relation;

use crate::ast::Relation;

pub(crate) fn parse_feedback_loop(input: &str) -> IResult<&str, FeedbackLoop> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('=')).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("feedback").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = parse_string(input)?;
    let (input, _) = ws0(input)?;

    let (input, body_fields) = opt(delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_feedback_loop_field)),
        preceded(ws0, char('}')),
    ))
    .parse(input)?;

    let mut loop_type = FeedbackLoopType::Reinforcing;
    let mut loop_id = None;
    let mut description = None;
    let mut relationships = Vec::new();

    if let Some(fields) = body_fields {
        for field in fields {
            match field {
                FeedbackLoopField::LoopType(t) => loop_type = t,
                FeedbackLoopField::LoopId(i) => loop_id = Some(i),
                FeedbackLoopField::Description(d) => description = Some(d),
                FeedbackLoopField::Relation(r) => relationships.push(r),
            }
        }
    }

    Ok((
        input,
        FeedbackLoop {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            loop_type,
            loop_id,
            title,
            description,
            relationships,
        },
    ))
}

enum FeedbackLoopField {
    LoopType(FeedbackLoopType),
    LoopId(String),
    Description(String),
    Relation(Relation),
}

fn parse_feedback_loop_field(input: &str) -> IResult<&str, FeedbackLoopField> {
    alt((
        map(
            preceded(tag("loop_type"), preceded(ws1, parse_feedback_loop_type)),
            FeedbackLoopField::LoopType,
        ),
        map(
            preceded(tag("loop_type"), preceded(ws1, parse_string)),
            |s| {
                FeedbackLoopField::LoopType(if s.eq_ignore_ascii_case("balancing") {
                    FeedbackLoopType::Balancing
                } else {
                    FeedbackLoopType::Reinforcing
                })
            },
        ),
        map(
            preceded(tag("loop_id"), preceded(ws1, parse_string)),
            FeedbackLoopField::LoopId,
        ),
        map(
            preceded(tag("description"), preceded(ws1, parse_string)),
            FeedbackLoopField::Description,
        ),
        map(parse_relation, FeedbackLoopField::Relation),
    ))
    .parse(input)
}

fn parse_feedback_loop_type(input: &str) -> IResult<&str, FeedbackLoopType> {
    alt((
        value(FeedbackLoopType::Reinforcing, tag("reinforcing")),
        value(FeedbackLoopType::Balancing, tag("balancing")),
    ))
    .parse(input)
}

pub(crate) fn parse_causal_loop(input: &str) -> IResult<&str, CausalLoop> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('=')).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("causal_loop").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = parse_string(input)?;
    let (input, _) = ws0(input)?;

    let (input, body_fields) = opt(delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_causal_loop_field)),
        preceded(ws0, char('}')),
    ))
    .parse(input)?;

    let mut loop_type = FeedbackLoopType::Reinforcing;
    let mut loop_id = None;
    let mut description = None;
    let mut variables = Vec::new();
    let mut relationships = Vec::new();

    if let Some(fields) = body_fields {
        for field in fields {
            match field {
                CausalLoopField::LoopType(t) => loop_type = t,
                CausalLoopField::LoopId(i) => loop_id = Some(i),
                CausalLoopField::Description(d) => description = Some(d),
                CausalLoopField::Variable(v) => variables.push(v),
                CausalLoopField::Relationship(r) => relationships.push(r),
            }
        }
    }

    Ok((
        input,
        CausalLoop {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            loop_type,
            loop_id,
            title,
            description,
            variables,
            relationships,
        },
    ))
}

enum CausalLoopField {
    LoopType(FeedbackLoopType),
    LoopId(String),
    Description(String),
    Variable(CausalLoopVariable),
    Relationship(CausalRelationship),
}

fn parse_causal_loop_field(input: &str) -> IResult<&str, CausalLoopField> {
    alt((
        map(
            preceded(tag("loop_type"), preceded(ws1, parse_feedback_loop_type)),
            CausalLoopField::LoopType,
        ),
        map(
            preceded(tag("loop_type"), preceded(ws1, parse_string)),
            |s| {
                CausalLoopField::LoopType(if s.eq_ignore_ascii_case("balancing") {
                    FeedbackLoopType::Balancing
                } else {
                    FeedbackLoopType::Reinforcing
                })
            },
        ),
        map(
            preceded(tag("loop_id"), preceded(ws1, parse_string)),
            CausalLoopField::LoopId,
        ),
        map(
            preceded(tag("description"), preceded(ws1, parse_string)),
            CausalLoopField::Description,
        ),
        map(parse_causal_loop_variable, CausalLoopField::Variable),
        map(parse_relation, |r| {
            CausalLoopField::Relationship(CausalRelationship {
                from: r.from.as_string(),
                to: r.to.as_string(),
                effect: r.label,
                polarity: CausalPolarity::Positive,
                delay: None,
            })
        }),
        map(
            parse_causal_loop_relationship,
            CausalLoopField::Relationship,
        ),
    ))
    .parse(input)
}

fn parse_causal_loop_variable(input: &str) -> IResult<&str, CausalLoopVariable> {
    let (input, _) = tag("variable").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, label) = opt(parse_string).parse(input)?;

    Ok((input, CausalLoopVariable { id, label }))
}

enum CausalRelField {
    Effect(String),
    Polarity(CausalPolarity),
    Delay(String),
}

fn parse_causal_loop_relationship(input: &str) -> IResult<&str, CausalRelationship> {
    let (input, from) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("->").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, to) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;

    let (input, body_fields) = opt(delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_causal_rel_field)),
        preceded(ws0, char('}')),
    ))
    .parse(input)?;

    let mut effect = None;
    let mut polarity = CausalPolarity::Positive;
    let mut delay = None;

    if let Some(fields) = body_fields {
        for field in fields {
            match field {
                CausalRelField::Effect(e) => effect = Some(e),
                CausalRelField::Polarity(p) => polarity = p,
                CausalRelField::Delay(d) => delay = Some(d),
            }
        }
    }

    Ok((
        input,
        CausalRelationship {
            from,
            to,
            effect,
            polarity,
            delay,
        },
    ))
}

fn parse_causal_rel_field(input: &str) -> IResult<&str, CausalRelField> {
    alt((
        map(
            preceded(tag("effect"), preceded(ws1, parse_string)),
            CausalRelField::Effect,
        ),
        map(
            preceded(tag("polarity"), preceded(ws1, parse_causal_polarity)),
            CausalRelField::Polarity,
        ),
        map(
            preceded(tag("delay"), preceded(ws1, parse_string)),
            CausalRelField::Delay,
        ),
    ))
    .parse(input)
}

fn parse_causal_polarity(input: &str) -> IResult<&str, CausalPolarity> {
    alt((
        value(CausalPolarity::Positive, tag("+")),
        value(CausalPolarity::Negative, tag("-")),
    ))
    .parse(input)
}
