//! State machine parser.

use nom::{
    bytes::complete::tag, character::complete::char, combinator::{map, opt}, multi::many0,
    sequence::{delimited, preceded}, IResult, Parser,
};
use sruja_diagnostics::SourceLocation;

use crate::ast::{StateMachine, StateTransition};
use super::primitives::{parse_string, parse_string_array, ws, ws0, ws1};

pub(crate) fn parse_state_machine(input: &str) -> IResult<&str, StateMachine> {
    let (input, _) = tag("state_machine").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, name) = parse_string(input)?;
    let (input, _) = ws0(input)?;
    
    let (input, body) = delimited(
        char('{'),
        many0(preceded(ws, parse_state_machine_item)),
        preceded(ws0, char('}')),
    ).parse(input)?;

    let mut initial_state = String::new();
    let mut terminal_states = Vec::new();
    let mut transitions = Vec::new();
    let mut description = None;

    for item in body {
        match item {
            StateMachineItem::Initial(s) => initial_state = s,
            StateMachineItem::Terminal(ss) => terminal_states.extend(ss),
            StateMachineItem::Transition(t) => transitions.push(t),
            StateMachineItem::Description(d) => description = Some(d),
        }
    }

    Ok((
        input,
        StateMachine {
            location: SourceLocation::new(String::new(), 0, 0),
            name,
            initial_state,
            terminal_states,
            transitions,
            description,
        },
    ))
}

enum StateMachineItem {
    Initial(String),
    Terminal(Vec<String>),
    Transition(StateTransition),
    Description(String),
}

fn parse_state_machine_item(input: &str) -> IResult<&str, StateMachineItem> {
    use nom::branch::alt;
    alt((
        map(preceded((tag("initial"), ws1), parse_string), StateMachineItem::Initial),
        map(preceded((tag("terminal"), ws1), parse_string_array), StateMachineItem::Terminal),
        map(preceded((tag("description"), ws1), parse_string), StateMachineItem::Description),
        map(parse_state_transition, StateMachineItem::Transition),
    )).parse(input)
}

fn parse_state_transition(input: &str) -> IResult<&str, StateTransition> {
    let (input, from) = parse_string(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("->").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, to) = parse_string(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("on").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, event) = parse_string(input)?;
    let (input, _) = ws0(input)?;
    
    let (input, body) = opt(delimited(
        char('{'),
        many0(preceded(ws, parse_transition_item)),
        preceded(ws0, char('}')),
    )).parse(input)?;

    let mut guard = None;
    let mut action = None;
    let mut description = None;

    if let Some(items) = body {
        for item in items {
            match item {
                TransitionItem::Guard(g) => guard = Some(g),
                TransitionItem::Action(a) => action = Some(a),
                TransitionItem::Description(d) => description = Some(d),
            }
        }
    }

    Ok((
        input,
        StateTransition {
            location: SourceLocation::new(String::new(), 0, 0),
            from,
            to,
            event,
            guard,
            action,
            description,
        },
    ))
}

enum TransitionItem {
    Guard(String),
    Action(String),
    Description(String),
}

fn parse_transition_item(input: &str) -> IResult<&str, TransitionItem> {
    use nom::branch::alt;
    alt((
        map(preceded((tag("guard"), ws1), parse_string), TransitionItem::Guard),
        map(preceded((tag("action"), ws1), parse_string), TransitionItem::Action),
        map(preceded((tag("description"), ws1), parse_string), TransitionItem::Description),
    )).parse(input)
}
