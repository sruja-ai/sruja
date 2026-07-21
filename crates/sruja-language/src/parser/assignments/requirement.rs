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

use crate::ast::{AcceptanceCriteria, Adr, Requirement};

use super::super::blocks::{parse_kv_string_block, parse_metadata_block};
use super::super::primitives::{
    parse_identifier, parse_kv_string, parse_string, parse_string_array, parse_tag_array, ws, ws0,
    ws1,
};

pub(crate) fn parse_requirement_assignment(input: &str) -> IResult<&str, Requirement> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('=')).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("requirement").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, r#type) = parse_identifier(input)?;
    let (input, _) = ws1(input)?;
    let (input, title) = parse_string(input)?;
    let (input, _) = ws0(input)?;
    let (input, details) = opt(parse_requirement_details).parse(input)?;

    let mut description = None;
    let mut tags = Vec::new();
    let mut priority = None;
    let mut status = None;
    let mut acceptance_criteria = Vec::new();
    let mut user_journey = None;
    let mut scenarios = Vec::new();
    let mut adrs = Vec::new();
    let mut affects = Vec::new();
    let mut source = None;
    if let Some(d) = details {
        if d.description.is_some() {
            description = d.description;
        }
        tags = d.tags;
        priority = d.priority;
        status = d.status;
        acceptance_criteria = d.acceptance_criteria;
        user_journey = d.user_journey;
        scenarios = d.scenarios;
        adrs = d.adrs;
        affects = d.affects;
        source = d.source;
    }

    Ok((
        input,
        Requirement {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            title,
            r#type,
            description,
            tags,
            priority,
            status,
            acceptance_criteria,
            user_journey,
            scenarios,
            adrs,
            affects,
            source,
        },
    ))
}

pub(crate) fn parse_adr_assignment(input: &str) -> IResult<&str, Adr> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('=')).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("adr").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;

    let (input, fields) = opt(parse_kv_string_block).parse(input)?;
    let mut adr = Adr {
        location: SourceLocation::new(String::new(), 0, 0),
        id: id.clone(),
        title: title.unwrap_or(id),
        status: None,
        context: None,
        decision: None,
        consequences: None,
        affects: vec![],
    };

    if let Some(kvs) = fields {
        for (k, v) in kvs {
            match k.as_str() {
                "status" => adr.status = Some(v),
                "context" => adr.context = Some(v),
                "decision" => adr.decision = Some(v),
                "consequences" => adr.consequences = Some(v),
                "affects" => adr.affects.push(v),
                _ => {}
            }
        }
    }

    Ok((input, adr))
}

pub(crate) fn parse_requirement(input: &str) -> IResult<&str, Requirement> {
    let (input, _) = tag("requirement").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, r#type) = opt(parse_identifier).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, details) = opt(parse_requirement_details).parse(input)?;

    let mut out_type = r#type.unwrap_or_else(|| "functional".to_string());
    let mut description = None;
    let mut tags = Vec::new();
    let mut priority = None;
    let mut status = None;
    let mut acceptance_criteria = Vec::new();
    let mut user_journey = None;
    let mut scenarios = Vec::new();
    let mut adrs = Vec::new();
    let mut affects = Vec::new();
    let mut source = None;
    if let Some(d) = details {
        if d.r#type.is_some() {
            out_type = d.r#type.unwrap_or(out_type);
        }
        description = d.description;
        tags = d.tags;
        priority = d.priority;
        status = d.status;
        acceptance_criteria = d.acceptance_criteria;
        user_journey = d.user_journey;
        scenarios = d.scenarios;
        adrs = d.adrs;
        affects = d.affects;
        source = d.source;
    }

    Ok((
        input,
        Requirement {
            location: SourceLocation::new(String::new(), 0, 0),
            title: title.unwrap_or_else(|| id.clone()),
            r#type: out_type,
            description,
            tags,
            id,
            priority,
            status,
            acceptance_criteria,
            user_journey,
            scenarios,
            adrs,
            affects,
            source,
        },
    ))
}

#[derive(Debug, Clone, Default)]
struct RequirementDetails {
    r#type: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
    priority: Option<String>,
    status: Option<String>,
    acceptance_criteria: Vec<AcceptanceCriteria>,
    user_journey: Option<String>,
    scenarios: Vec<String>,
    adrs: Vec<String>,
    affects: Vec<String>,
    source: Option<String>,
}

#[derive(Debug, Clone)]
enum RequirementField {
    Type(String),
    Description(String),
    Tags(Vec<String>),
    Priority(String),
    Status(String),
    AcceptanceCriteria(AcceptanceCriteria),
    UserJourney(String),
    Scenario(String),
    Adr(String),
    Affects(String),
    Source(String),
    Ignored,
}

fn parse_acceptance_criteria_block(input: &str) -> IResult<&str, AcceptanceCriteria> {
    use nom::sequence::delimited;
    let (input, _) = tag("acceptance_criteria").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, fields) = delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_acceptance_criteria_field)),
        preceded(ws0, char('}')),
    )
    .parse(input)?;

    let mut ac = AcceptanceCriteria {
        given: None,
        when: None,
        then: None,
    };
    for (k, v) in fields {
        match k.as_str() {
            "given" => ac.given = Some(v),
            "when" => ac.when = Some(v),
            "then" => ac.then = Some(v),
            _ => {}
        }
    }
    Ok((input, ac))
}

fn parse_acceptance_criteria_field(input: &str) -> IResult<&str, (String, String)> {
    parse_kv_string(input)
}

fn parse_requirement_field(input: &str) -> IResult<&str, RequirementField> {
    alt((
        map(
            preceded(
                tag("type"),
                preceded(ws1, alt((parse_identifier, parse_string))),
            ),
            RequirementField::Type,
        ),
        map(
            preceded(tag("description"), preceded(ws1, parse_string)),
            RequirementField::Description,
        ),
        map(
            preceded(
                tag("tags"),
                preceded(ws1, alt((parse_string_array, parse_tag_array))),
            ),
            RequirementField::Tags,
        ),
        map(
            preceded(tag("priority"), preceded(ws1, parse_string)),
            RequirementField::Priority,
        ),
        map(
            preceded(tag("status"), preceded(ws1, parse_string)),
            RequirementField::Status,
        ),
        map(parse_acceptance_criteria_block, |ac| {
            RequirementField::AcceptanceCriteria(ac)
        }),
        map(
            preceded(tag("user_journey"), preceded(ws1, parse_string)),
            RequirementField::UserJourney,
        ),
        map(
            preceded(
                tag("scenario"),
                preceded(ws1, alt((parse_string, parse_identifier))),
            ),
            RequirementField::Scenario,
        ),
        map(
            preceded(
                tag("adr"),
                preceded(ws1, alt((parse_string, parse_identifier))),
            ),
            RequirementField::Adr,
        ),
        map(
            preceded(
                tag("affects"),
                preceded(ws1, alt((parse_string, parse_identifier))),
            ),
            RequirementField::Affects,
        ),
        map(
            preceded(tag("source"), preceded(ws1, parse_string)),
            RequirementField::Source,
        ),
        map(parse_metadata_block, |_| RequirementField::Ignored),
    ))
    .parse(input)
}

fn parse_requirement_details(input: &str) -> IResult<&str, RequirementDetails> {
    use nom::sequence::delimited;
    let (input, fields) = delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_requirement_field)),
        preceded(ws0, char('}')),
    )
    .parse(input)?;

    let mut details = RequirementDetails::default();
    for field in fields {
        match field {
            RequirementField::Type(t) => details.r#type = Some(t),
            RequirementField::Description(d) => details.description = Some(d),
            RequirementField::Tags(t) => details.tags = t,
            RequirementField::Priority(p) => details.priority = Some(p),
            RequirementField::Status(s) => details.status = Some(s),
            RequirementField::AcceptanceCriteria(ac) => details.acceptance_criteria.push(ac),
            RequirementField::UserJourney(uj) => details.user_journey = Some(uj),
            RequirementField::Scenario(s) => details.scenarios.push(s),
            RequirementField::Adr(a) => details.adrs.push(a),
            RequirementField::Affects(a) => details.affects.push(a),
            RequirementField::Source(s) => details.source = Some(s),
            RequirementField::Ignored => {}
        }
    }

    Ok((input, details))
}

pub(crate) fn parse_adr(input: &str) -> IResult<&str, Adr> {
    let (input, _) = tag("adr").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;

    let (input, fields) = opt(parse_kv_string_block).parse(input)?;
    let mut adr = Adr {
        location: SourceLocation::new(String::new(), 0, 0),
        id: id.clone(),
        title: title.unwrap_or(id),
        status: None,
        context: None,
        decision: None,
        consequences: None,
        affects: vec![],
    };

    if let Some(kvs) = fields {
        for (k, v) in kvs {
            match k.as_str() {
                "status" => adr.status = Some(v),
                "context" => adr.context = Some(v),
                "decision" => adr.decision = Some(v),
                "consequences" => adr.consequences = Some(v),
                "affects" => adr.affects.push(v),
                _ => {}
            }
        }
    }

    Ok((input, adr))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_requirement_minimal() {
        let input = r#"requirement R1"#;
        let result = parse_requirement(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, req) = result.unwrap();
        assert_eq!(req.id, "R1");
        assert_eq!(req.r#type, "functional");
        assert_eq!(req.title, "R1");
    }

    #[test]
    fn test_parse_requirement_with_type_and_title() {
        let input = r#"requirement R1 functional "User can log in""#;
        let result = parse_requirement(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, req) = result.unwrap();
        assert_eq!(req.id, "R1");
        assert_eq!(req.r#type, "functional");
        assert_eq!(req.title, "User can log in");
    }

    #[test]
    fn test_parse_requirement_with_details() {
        let input = r#"requirement R1 functional "User can log in" {
            description "Must support SSO"
            tags [#auth, @pii]
        }"#;
        let result = parse_requirement(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, req) = result.unwrap();
        assert_eq!(req.description, Some("Must support SSO".to_string()));
        assert_eq!(req.tags, vec!["#auth".to_string(), "@pii".to_string()]);
    }

    #[test]
    fn test_parse_adr_minimal() {
        let input = r#"adr ADR_1"#;
        let result = parse_adr(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, adr) = result.unwrap();
        assert_eq!(adr.id, "ADR_1");
        assert_eq!(adr.title, "ADR_1");
        assert!(adr.status.is_none());
    }

    #[test]
    fn test_parse_adr_with_title() {
        let input = r#"adr ADR_1 "Use PostgreSQL""#;
        let result = parse_adr(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, adr) = result.unwrap();
        assert_eq!(adr.id, "ADR_1");
        assert_eq!(adr.title, "Use PostgreSQL");
    }

    #[test]
    fn test_parse_adr_with_fields() {
        let input = r#"adr ADR_1 "Use PostgreSQL" {
            status "accepted"
            context "Need a reliable database"
            decision "Use PostgreSQL for all services"
            consequences "Need to manage migrations"
            affects "DatabaseService"
        }"#;
        let result = parse_adr(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, adr) = result.unwrap();
        assert_eq!(adr.status, Some("accepted".to_string()));
        assert_eq!(adr.context, Some("Need a reliable database".to_string()));
        assert_eq!(
            adr.decision,
            Some("Use PostgreSQL for all services".to_string())
        );
        assert_eq!(
            adr.consequences,
            Some("Need to manage migrations".to_string())
        );
        assert_eq!(adr.affects, vec!["DatabaseService".to_string()]);
    }

    #[test]
    fn test_parse_requirement_assignment_minimal() {
        let input = r#"R1 = requirement functional "User can log in""#;
        let result = parse_requirement_assignment(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, req) = result.unwrap();
        assert_eq!(req.id, "R1");
        assert_eq!(req.r#type, "functional");
        assert_eq!(req.title, "User can log in");
    }

    #[test]
    fn test_parse_adr_assignment_minimal() {
        let input = r#"ADR_1 = adr "Use PostgreSQL""#;
        let result = parse_adr_assignment(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, adr) = result.unwrap();
        assert_eq!(adr.id, "ADR_1");
        assert_eq!(adr.title, "Use PostgreSQL");
    }

    #[test]
    fn test_parse_requirement_with_type_in_block() {
        let input = r#"requirement R1 "Must be available" {
            type nonfunctional
            description "Must be available 99.9%"
            tags [#critical, @production]
        }"#;
        let result = parse_requirement(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, req) = result.unwrap();
        assert_eq!(req.r#type, "nonfunctional");
        assert_eq!(req.description, Some("Must be available 99.9%".to_string()));
        assert_eq!(
            req.tags,
            vec!["#critical".to_string(), "@production".to_string()]
        );
    }
}
