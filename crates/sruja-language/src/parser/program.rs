//! Program and top-level item parsing.

use nom::{branch::alt, combinator::map, IResult, Parser};

use crate::ast::{ElementAssignment, ElementDef, ElementKind, Program, TopLevelItem};

use super::assignments::{
    parse_adr, parse_adr_assignment, parse_flow, parse_flow_assignment, parse_policy,
    parse_policy_assignment, parse_requirement, parse_requirement_assignment, parse_scenario,
    parse_scenario_assignment,
};
use super::blocks::parse_metadata_block;
use super::blocks::{
    parse_constraints_block, parse_conventions_block, parse_fitness, parse_incident,
    parse_style_decl,
};
use super::deployment::parse_deployment;
use super::elements::{parse_element_def, parse_element_def_unassigned, parse_kind_def};
use super::import::parse_import;
use super::loops::{parse_causal_loop, parse_feedback_loop};
use super::overview_views::{parse_overview_block, parse_view};
use super::primitives::ws;
use super::relations::parse_relation;
use super::schema::parse_schema;

pub(super) fn parse_program(input: &str) -> IResult<&str, Program> {
    let (input, _) = ws(input)?;
    let mut items = Vec::new();
    let mut remaining = input;
    loop {
        let (rest, _) = ws(remaining)?;
        if rest.is_empty() {
            remaining = rest;
            break;
        }
        let (next, item) = parse_top_level_item(rest)?;
        items.push(item);
        remaining = next;
    }
    Ok((remaining, Program::with_items(Program::new(), items)))
}

pub(super) fn parse_top_level_item(input: &str) -> IResult<&str, TopLevelItem> {
    alt((
        alt((
            map(parse_kind_def, TopLevelItem::KindDef),
            map(parse_scenario_assignment, TopLevelItem::Scenario),
            map(parse_flow_assignment, TopLevelItem::Flow),
            map(parse_requirement_assignment, TopLevelItem::Requirement),
            map(parse_adr_assignment, TopLevelItem::Adr),
            map(parse_policy_assignment, TopLevelItem::Policy),
            map(parse_overview_block, TopLevelItem::Overview),
            map(parse_feedback_loop, TopLevelItem::FeedbackLoop),
            map(parse_causal_loop, TopLevelItem::CausalLoop),
            map(parse_deployment, TopLevelItem::Deployment),
            map(parse_constraints_block, TopLevelItem::Constraints),
            map(parse_conventions_block, TopLevelItem::Conventions),
            map(parse_style_decl, TopLevelItem::Style),
            map(parse_element_def_unassigned, |e| {
                TopLevelItem::ElementDef(Box::new(e))
            }),
            map(parse_element_def, |e| TopLevelItem::ElementDef(Box::new(e))),
        )),
        alt((
            map(parse_relation, TopLevelItem::Relation),
            map(parse_import, TopLevelItem::Import),
            map(parse_scenario, TopLevelItem::Scenario),
            map(parse_flow, TopLevelItem::Flow),
            map(parse_requirement, TopLevelItem::Requirement),
            map(parse_adr, TopLevelItem::Adr),
            map(parse_policy, TopLevelItem::Policy),
            map(parse_view, TopLevelItem::View),
            map(parse_schema, TopLevelItem::Schema),
            map(parse_incident, TopLevelItem::Incident),
            map(parse_fitness, TopLevelItem::Fitness),
            map(parse_metadata_block, |m| {
                TopLevelItem::ElementDef(Box::new(ElementDef {
                    location: m.location.clone(),
                    assignment: ElementAssignment {
                        location: m.location.clone(),
                        name: "metadata".to_string(),
                        kind: ElementKind::Custom("metadata".to_string()),
                        sub_kind: None,
                        title: None,
                        tag_refs: Vec::new(),
                        body: None,
                    },
                }))
            }),
        )),
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::TopLevelItem;

    #[test]
    fn test_parse_program_empty() {
        let input = "";
        let result = parse_program(input);
        assert!(result.is_ok());
        let (_, program) = result.unwrap();
        assert!(program.items.is_empty());
    }

    #[test]
    fn test_parse_program_whitespace_only() {
        let input = "   \n  \n  ";
        let result = parse_program(input);
        assert!(result.is_ok());
        let (_, program) = result.unwrap();
        assert!(program.items.is_empty());
    }

    #[test]
    fn test_parse_program_single_element() {
        let input = r#"MySystem = system "My System""#;
        let result = parse_program(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, program) = result.unwrap();
        assert_eq!(program.items.len(), 1);
        assert!(matches!(program.items[0], TopLevelItem::ElementDef(_)));
    }

    #[test]
    fn test_parse_program_multiple_elements() {
        let input = r#"
MySystem = system "My System"
WebApp = container "Web Application"
DB = database "Database"
"#;
        let result = parse_program(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, program) = result.unwrap();
        assert_eq!(program.items.len(), 3);
    }

    #[test]
    fn test_parse_program_with_relations() {
        let input = r#"
MySystem = system "My System"
WebApp = container "Web Application"
MySystem -> WebApp "contains"
"#;
        let result = parse_program(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, program) = result.unwrap();
        assert_eq!(program.items.len(), 3);
    }

    #[test]
    fn test_parse_program_with_import() {
        let input = r#"
import { ServiceA, ServiceB } from "projectA"
MySystem = system "My System"
"#;
        let result = parse_program(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, program) = result.unwrap();
        assert_eq!(program.items.len(), 2);
    }

    #[test]
    fn test_parse_program_with_scenario() {
        let input = r#"
scenario LoginFlow "User Login" {
    User -> WebApp "Credentials"
    WebApp -> DB "Verify"
}
"#;
        let result = parse_program(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, program) = result.unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_program_with_flow() {
        let input = r#"
flow LoginFlow "User Login" {
    User -> WebApp "open"
    WebApp -> DB "query"
}
"#;
        let result = parse_program(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, program) = result.unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_program_with_requirement() {
        let input = r#"
requirement R1 functional "User can log in" {
    description "Must support SSO"
    tags [#auth, @pii]
}
"#;
        let result = parse_program(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, program) = result.unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_program_with_adr() {
        let input = r#"
adr ADR_1 "Use PostgreSQL" {
    status "accepted"
    context "Need a reliable database"
}
"#;
        let result = parse_program(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, program) = result.unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_program_with_policy() {
        let input = r#"
policy SecurityPolicy "Security Rules" {
    category "security"
    enforcement "deny"
}
"#;
        let result = parse_program(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, program) = result.unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_program_with_comments() {
        let input = r#"
// This is a comment
MySystem = system "My System"
/* Multi-line
   comment */
SystemA -> SystemB "Uses"
"#;
        let result = parse_program(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, program) = result.unwrap();
        // The relation may or may not parse depending on context
        assert!(program.items.len() >= 2);
    }

    #[test]
    fn test_parse_top_level_item_element() {
        let input = r#"MySystem = system "My System""#;
        let result = parse_top_level_item(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, item) = result.unwrap();
        assert!(matches!(item, TopLevelItem::ElementDef(_)));
    }

    #[test]
    fn test_parse_top_level_item_relation() {
        let input = r#"SystemA -> SystemB "Uses""#;
        let result = parse_top_level_item(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, item) = result.unwrap();
        assert!(matches!(item, TopLevelItem::Relation(_)));
    }

    #[test]
    fn test_parse_top_level_item_import() {
        let input = r#"import { ServiceA } from "projectA""#;
        let result = parse_top_level_item(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, item) = result.unwrap();
        assert!(matches!(item, TopLevelItem::Import(_)));
    }

    #[test]
    fn test_parse_top_level_item_scenario() {
        let input = r#"scenario LoginFlow "User Login""#;
        let result = parse_top_level_item(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, item) = result.unwrap();
        assert!(matches!(item, TopLevelItem::Scenario(_)));
    }

    #[test]
    fn test_parse_top_level_item_flow() {
        let input = r#"flow LoginFlow "User Login""#;
        let result = parse_top_level_item(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, item) = result.unwrap();
        assert!(matches!(item, TopLevelItem::Flow(_)));
    }

    #[test]
    fn test_parse_top_level_item_requirement() {
        let input = r#"requirement R1 functional "User can log in""#;
        let result = parse_top_level_item(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, item) = result.unwrap();
        assert!(matches!(item, TopLevelItem::Requirement(_)));
    }

    #[test]
    fn test_parse_top_level_item_adr() {
        let input = r#"adr ADR_1 "Use PostgreSQL""#;
        let result = parse_top_level_item(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, item) = result.unwrap();
        assert!(matches!(item, TopLevelItem::Adr(_)));
    }

    #[test]
    fn test_parse_top_level_item_policy() {
        let input = r#"policy SecurityPolicy "Security Rules""#;
        let result = parse_top_level_item(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, item) = result.unwrap();
        assert!(matches!(item, TopLevelItem::Policy(_)));
    }

    #[test]
    fn test_parse_program_complex() {
        let input = r#"
// Systems
MyApp = system "My Application" {
    description "A web application"
}

// Containers
WebApp = container "Web Application" {
    technology "React"
    description "Frontend"
}

API = container "API Service" {
    technology "Node.js"
    description "Backend API"
}

DB = database "Database" {
    technology "PostgreSQL"
    description "Main database"
}

// Import
import { SharedLib } from "shared"
"#;
        let result = parse_program(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, program) = result.unwrap();
        assert!(program.items.len() >= 4);
    }
}
