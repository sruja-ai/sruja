use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{map, opt},
    multi::many0,
    sequence::preceded,
    IResult, Parser,
};
use sruja_diagnostics::SourceLocation;

use crate::ast::{Flow, Scenario, ScenarioStep};

use super::super::primitives::{
    parse_identifier, parse_string, parse_string_array, parse_tag_array, ws, ws0, ws1,
};
use super::super::relations::parse_qualified_ident;

#[derive(Debug, Clone)]
enum ScenarioBodyData {
    Steps(Vec<ScenarioStep>),
    Block {
        title: Option<String>,
        description: Option<String>,
        steps: Vec<ScenarioStep>,
    },
}

fn parse_steps_array(input: &str) -> IResult<&str, Vec<ScenarioStep>> {
    use nom::sequence::delimited;
    delimited(
        preceded(ws0, char('[')),
        many0(preceded(ws, parse_scenario_step)),
        preceded(ws0, char(']')),
    )
    .parse(input)
}

#[derive(Debug, Clone)]
enum ScenarioBlockItem {
    Title(String),
    Description(String),
    Steps(Vec<ScenarioStep>),
}

fn parse_scenario_block_item(input: &str) -> IResult<&str, ScenarioBlockItem> {
    alt((
        map(
            preceded(tag("title"), preceded(ws1, parse_string)),
            ScenarioBlockItem::Title,
        ),
        map(
            preceded(tag("description"), preceded(ws1, parse_string)),
            ScenarioBlockItem::Description,
        ),
        map(
            preceded(tag("steps"), preceded(ws0, parse_steps_array)),
            ScenarioBlockItem::Steps,
        ),
    ))
    .parse(input)
}

fn parse_scenario_block_body(input: &str) -> IResult<&str, ScenarioBodyData> {
    use nom::sequence::delimited;
    let (input, items) = delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_scenario_block_item)),
        preceded(ws0, char('}')),
    )
    .parse(input)?;

    let mut title = None;
    let mut description = None;
    let mut steps = Vec::new();
    for item in items {
        match item {
            ScenarioBlockItem::Title(t) => title = Some(t),
            ScenarioBlockItem::Description(d) => description = Some(d),
            ScenarioBlockItem::Steps(s) => steps = s,
        }
    }

    Ok((
        input,
        ScenarioBodyData::Block {
            title,
            description,
            steps,
        },
    ))
}

fn parse_scenario_body_any(input: &str) -> IResult<&str, ScenarioBodyData> {
    alt((
        parse_scenario_block_body,
        map(parse_scenario_body, ScenarioBodyData::Steps),
    ))
    .parse(input)
}

pub(crate) fn parse_flow_assignment(input: &str) -> IResult<&str, Flow> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('=')).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("flow").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, steps) = opt(parse_flow_body).parse(input)?;

    Ok((
        input,
        Flow {
            location: SourceLocation::new(String::new(), 0, 0),
            id: id.clone(),
            title: title.unwrap_or_else(|| id.clone()),
            description,
            steps: steps.unwrap_or_default(),
        },
    ))
}

pub(crate) fn parse_scenario(input: &str) -> IResult<&str, Scenario> {
    let (input, _) = alt((tag("scenario"), tag("story"))).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, id) = opt(parse_identifier).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, body) = opt(parse_scenario_body_any).parse(input)?;

    let mut out_title = title.unwrap_or_default();
    let mut out_description = description;
    let mut out_steps = Vec::new();
    if let Some(b) = body {
        match b {
            ScenarioBodyData::Steps(steps) => out_steps = steps,
            ScenarioBodyData::Block {
                title,
                description,
                steps,
            } => {
                if title.is_some() {
                    out_title = title.unwrap_or_default();
                }
                if description.is_some() {
                    out_description = description;
                }
                out_steps = steps;
            }
        }
    }
    let out_id = id.unwrap_or_default();
    if out_title.is_empty() {
        out_title = out_id.clone();
    }

    Ok((
        input,
        Scenario {
            location: SourceLocation::new(String::new(), 0, 0),
            id: out_id,
            title: out_title,
            description: out_description,
            steps: out_steps,
        },
    ))
}

pub(crate) fn parse_scenario_assignment(input: &str) -> IResult<&str, Scenario> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('=')).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = alt((tag("scenario"), tag("story"))).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, body) = opt(parse_scenario_body_any).parse(input)?;

    let mut out_title = title.unwrap_or_else(|| id.clone());
    let mut out_description = description;
    let mut out_steps = Vec::new();
    if let Some(b) = body {
        match b {
            ScenarioBodyData::Steps(steps) => out_steps = steps,
            ScenarioBodyData::Block {
                title,
                description,
                steps,
            } => {
                if title.is_some() {
                    out_title = title.unwrap_or_else(|| id.clone());
                }
                if description.is_some() {
                    out_description = description;
                }
                out_steps = steps;
            }
        }
    }

    Ok((
        input,
        Scenario {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            title: out_title,
            description: out_description,
            steps: out_steps,
        },
    ))
}

pub(crate) fn parse_scenario_body(input: &str) -> IResult<&str, Vec<ScenarioStep>> {
    use nom::sequence::delimited;
    delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_scenario_step)),
        preceded(ws0, char('}')),
    )
    .parse(input)
}

fn parse_step_line(input: &str) -> IResult<&str, ScenarioStep> {
    use nom::branch::alt;
    use nom::character::complete::digit1;
    use nom::combinator::map;

    let (input, _) = opt(preceded(tag("step"), ws1)).parse(input)?;
    let (input, from) = parse_qualified_ident(input)?;
    let (input, _) = preceded(ws0, tag("->")).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, to) = parse_qualified_ident(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, tags) = opt(alt((parse_string_array, parse_tag_array))).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, order_raw) = opt(preceded(
        tag("order"),
        preceded(
            ws1,
            alt((parse_string, map(digit1, |s: &str| s.to_string()))),
        ),
    ))
    .parse(input)?;

    let order = order_raw.as_deref().and_then(|s| s.parse::<usize>().ok());

    Ok((
        input,
        ScenarioStep {
            from: Some(from),
            to: Some(to),
            description,
            tags: tags.unwrap_or_default(),
            order,
        },
    ))
}

pub(crate) fn parse_scenario_step(input: &str) -> IResult<&str, ScenarioStep> {
    parse_step_line(input)
}

pub(crate) fn parse_flow(input: &str) -> IResult<&str, Flow> {
    let (input, _) = tag("flow").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, id) = opt(parse_identifier).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, steps) = opt(parse_flow_body).parse(input)?;

    let out_id = id.unwrap_or_default();
    let out_title = title.unwrap_or_else(|| out_id.clone());

    Ok((
        input,
        Flow {
            location: SourceLocation::new(String::new(), 0, 0),
            id: out_id,
            title: out_title,
            description,
            steps: steps.unwrap_or_default(),
        },
    ))
}

pub(crate) fn parse_flow_body(input: &str) -> IResult<&str, Vec<ScenarioStep>> {
    use nom::sequence::delimited;
    delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_flow_step)),
        preceded(ws0, char('}')),
    )
    .parse(input)
}

fn parse_flow_step(input: &str) -> IResult<&str, ScenarioStep> {
    parse_step_line(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_flow_minimal() {
        let input = r#"flow LoginFlow"#;
        let result = parse_flow(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, flow) = result.unwrap();
        assert_eq!(flow.id, "LoginFlow");
        assert_eq!(flow.title, "LoginFlow");
        assert!(flow.description.is_none());
        assert!(flow.steps.is_empty());
    }

    #[test]
    fn test_parse_flow_with_title() {
        let input = r#"flow LoginFlow "User Login""#;
        let result = parse_flow(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, flow) = result.unwrap();
        assert_eq!(flow.id, "LoginFlow");
        assert_eq!(flow.title, "User Login");
    }

    #[test]
    fn test_parse_flow_with_title_and_description() {
        let input = r#"flow LoginFlow "User Login" "Successful login flow""#;
        let result = parse_flow(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, flow) = result.unwrap();
        assert_eq!(flow.id, "LoginFlow");
        assert_eq!(flow.title, "User Login");
        assert_eq!(flow.description, Some("Successful login flow".to_string()));
    }

    #[test]
    fn test_parse_flow_with_steps() {
        let input = r#"flow LoginFlow "User Login" {
            User -> WebApp "open"
            WebApp -> DB "query"
        }"#;
        let result = parse_flow(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, flow) = result.unwrap();
        assert_eq!(flow.steps.len(), 2);
    }

    #[test]
    fn test_parse_scenario_minimal() {
        let input = r#"scenario LoginFlow"#;
        let result = parse_scenario(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, scenario) = result.unwrap();
        assert_eq!(scenario.id, "LoginFlow");
        assert_eq!(scenario.title, "LoginFlow");
        assert!(scenario.description.is_none());
        assert!(scenario.steps.is_empty());
    }

    #[test]
    fn test_parse_scenario_with_title() {
        let input = r#"scenario LoginFlow "User Login""#;
        let result = parse_scenario(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, scenario) = result.unwrap();
        assert_eq!(scenario.id, "LoginFlow");
        assert_eq!(scenario.title, "User Login");
    }

    #[test]
    fn test_parse_scenario_with_steps() {
        let input = r#"scenario LoginFlow "User Login" {
            User -> WebApp "Credentials"
            WebApp -> DB "Verify"
        }"#;
        let result = parse_scenario(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, scenario) = result.unwrap();
        assert_eq!(scenario.steps.len(), 2);
    }

    #[test]
    fn test_parse_scenario_assignment_minimal() {
        let input = r#"LoginFlow = scenario "User Login""#;
        let result = parse_scenario_assignment(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, scenario) = result.unwrap();
        assert_eq!(scenario.id, "LoginFlow");
        assert_eq!(scenario.title, "User Login");
    }

    #[test]
    fn test_parse_flow_assignment_minimal() {
        let input = r#"Login = flow "Login Flow""#;
        let result = parse_flow_assignment(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, flow) = result.unwrap();
        assert_eq!(flow.id, "Login");
        assert_eq!(flow.title, "Login Flow");
    }

    #[test]
    fn test_parse_step_line_basic() {
        let input = r#"User -> WebApp "open""#;
        let result = parse_scenario_step(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, step) = result.unwrap();
        assert_eq!(step.from.unwrap().parts, vec!["User"]);
        assert_eq!(step.to.unwrap().parts, vec!["WebApp"]);
        assert_eq!(step.description, Some("open".to_string()));
    }

    #[test]
    fn test_parse_step_line_with_tags() {
        let input = r#"User -> WebApp "open" [#auth]"#;
        let result = parse_scenario_step(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, step) = result.unwrap();
        assert_eq!(step.tags, vec!["#auth".to_string()]);
    }

    #[test]
    fn test_parse_step_line_with_order() {
        let input = r#"User -> WebApp "open" order 5"#;
        let result = parse_scenario_step(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, step) = result.unwrap();
        assert_eq!(step.order, Some(5));
    }

    #[test]
    fn test_parse_step_line_with_step_keyword() {
        let input = r#"step User -> WebApp "open""#;
        let result = parse_scenario_step(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, step) = result.unwrap();
        assert_eq!(step.from.unwrap().parts, vec!["User"]);
    }

    #[test]
    fn test_parse_scenario_with_block_body() {
        let input = r#"scenario Checkout "Checkout Flow" {
            title "Checkout Flow"
            description "User checks out"
            steps [
                User -> WebApp "browse"
                WebApp -> DB "load"
            ]
        }"#;
        let result = parse_scenario(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, scenario) = result.unwrap();
        assert_eq!(scenario.title, "Checkout Flow");
        assert_eq!(scenario.description, Some("User checks out".to_string()));
        assert_eq!(scenario.steps.len(), 2);
    }

    #[test]
    fn test_parse_flow_body() {
        let input = r#"{
            User -> WebApp "open"
            WebApp -> DB "query"
        }"#;
        let result = parse_flow_body(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, steps) = result.unwrap();
        assert_eq!(steps.len(), 2);
    }

    #[test]
    fn test_parse_scenario_body() {
        let input = r#"{
            User -> WebApp "Credentials"
            WebApp -> DB "Verify"
        }"#;
        let result = parse_scenario_body(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, steps) = result.unwrap();
        assert_eq!(steps.len(), 2);
    }

    #[test]
    fn test_story_keyword_alias() {
        let input = r#"story LoginFlow "User Login""#;
        let result = parse_scenario(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, scenario) = result.unwrap();
        assert_eq!(scenario.id, "LoginFlow");
    }

    #[test]
    fn test_story_assignment_keyword_alias() {
        let input = r#"LoginFlow = story "User Login""#;
        let result = parse_scenario_assignment(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, scenario) = result.unwrap();
        assert_eq!(scenario.id, "LoginFlow");
    }
}
