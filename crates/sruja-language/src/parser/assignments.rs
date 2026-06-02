//! Assignment and flow/scenario/requirement/ADR/policy parsers.

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

use crate::ast::{Adr, Flow, Policy, Requirement, Scenario, ScenarioStep};

use super::blocks::{parse_kv_string_block, parse_metadata_block};
use super::primitives::{
    parse_identifier, parse_kv_string, parse_string, parse_string_array, parse_tag_array, ws, ws0,
    ws1,
};
use super::relations::parse_qualified_ident;

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
    if let Some(d) = details {
        if d.description.is_some() {
            description = d.description;
        }
        tags = d.tags;
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

pub(crate) fn parse_policy_assignment(input: &str) -> IResult<&str, Policy> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('=')).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("policy").parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;

    let (input, block) = opt(parse_policy_block).parse(input)?;
    let mut policy = Policy {
        location: SourceLocation::new(String::new(), 0, 0),
        id: id.clone(),
        title: title.unwrap_or(id),
        category: "general".to_string(),
        enforcement: "warn".to_string(),
        description: None,
        rules: Vec::new(),
    };

    if let Some((kvs, rules)) = block {
        for (k, v) in kvs {
            match k.as_str() {
                "category" => policy.category = v,
                "enforcement" => policy.enforcement = v,
                "description" => policy.description = Some(v),
                _ => {}
            }
        }
        policy.rules = rules;
    }

    Ok((input, policy))
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
    if let Some(d) = details {
        if d.r#type.is_some() {
            out_type = d.r#type.unwrap_or(out_type);
        }
        description = d.description;
        tags = d.tags;
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
        },
    ))
}

#[derive(Debug, Clone, Default)]
struct RequirementDetails {
    r#type: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
}

#[derive(Debug, Clone)]
enum RequirementField {
    Type(String),
    Description(String),
    Tags(Vec<String>),
    Ignored,
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

pub(crate) fn parse_policy(input: &str) -> IResult<&str, Policy> {
    let (input, _) = tag("policy").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;

    let (input, block) = opt(parse_policy_block).parse(input)?;
    let mut policy = Policy {
        location: SourceLocation::new(String::new(), 0, 0),
        id: id.clone(),
        title: title.unwrap_or(id),
        category: "general".to_string(),
        enforcement: "warn".to_string(),
        description: None,
        rules: Vec::new(),
    };

    if let Some((kvs, rules)) = block {
        for (k, v) in kvs {
            match k.as_str() {
                "category" => policy.category = v,
                "enforcement" => policy.enforcement = v,
                "description" => policy.description = Some(v),
                _ => {}
            }
        }
        policy.rules = rules;
    }

    Ok((input, policy))
}

#[derive(Debug, Clone)]
enum PolicyBlockEntry {
    Kv(String, String),
    Rule(Box<crate::ast::PolicyRuleAst>),
}

type PolicyBlockResult = (Vec<(String, String)>, Vec<crate::ast::PolicyRuleAst>);

fn parse_policy_block(input: &str) -> IResult<&str, PolicyBlockResult> {
    use nom::sequence::delimited;
    let (input, entries) = delimited(
        preceded(ws0, char('{')),
        many0(preceded(
            ws,
            alt((
                map(parse_policy_rule_line, |r| {
                    PolicyBlockEntry::Rule(Box::new(r))
                }),
                map(parse_kv_string, |(k, v)| PolicyBlockEntry::Kv(k, v)),
            )),
        )),
        preceded(ws0, char('}')),
    )
    .parse(input)?;

    let kvs: Vec<(String, String)> = entries
        .iter()
        .filter_map(|e| match e {
            PolicyBlockEntry::Kv(k, v) => Some((k.clone(), v.clone())),
            _ => None,
        })
        .collect();

    let rules: Vec<crate::ast::PolicyRuleAst> = entries
        .into_iter()
        .filter_map(|e| match e {
            PolicyBlockEntry::Rule(r) => Some(*r),
            _ => None,
        })
        .collect();

    Ok((input, (kvs, rules)))
}

fn parse_policy_rule_line(input: &str) -> IResult<&str, crate::ast::PolicyRuleAst> {
    use crate::ast::{
        PolicyEdgeExceptionAst, PolicyMetaSelectorAst, PolicyRuleAst, PolicySelectorAst,
    };

    let (input, _) = tag("rule").parse(input)?;
    let (input, _) = ws1(input)?;

    fn parse_tags_array(input: &str) -> IResult<&str, Vec<String>> {
        alt((parse_string_array, parse_tag_array)).parse(input)
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum SelectorItem {
        Kind(String),
        Id(String),
        Tag(String),
        Technology(String),
        Meta { key: String, value: Option<String> },
    }

    fn parse_selector_item(input: &str) -> IResult<&str, SelectorItem> {
        fn parse_meta(input: &str) -> IResult<&str, SelectorItem> {
            use nom::combinator::opt;
            use nom::sequence::preceded;

            let (input, _) = tag("meta").parse(input)?;
            let (input, _) = ws1(input)?;
            let (input, key) = parse_string(input)?;
            let (input, value) = opt(preceded(
                ws0,
                preceded(char('='), preceded(ws0, parse_string)),
            ))
            .parse(input)?;
            Ok((input, SelectorItem::Meta { key, value }))
        }

        alt((
            map(
                preceded(tag("kind"), preceded(ws1, parse_string)),
                SelectorItem::Kind,
            ),
            map(
                preceded(tag("id"), preceded(ws1, parse_string)),
                SelectorItem::Id,
            ),
            map(
                preceded(tag("tag"), preceded(ws1, parse_string)),
                SelectorItem::Tag,
            ),
            map(
                preceded(tag("technology"), preceded(ws1, parse_string)),
                SelectorItem::Technology,
            ),
            parse_meta,
        ))
        .parse(input)
    }

    fn parse_policy_selector(input: &str) -> IResult<&str, PolicySelectorAst> {
        use nom::sequence::delimited;

        let (input, items) = delimited(
            preceded(ws0, char('{')),
            many0(preceded(ws, parse_selector_item)),
            preceded(ws0, char('}')),
        )
        .parse(input)?;

        let mut selector = PolicySelectorAst::default();
        for item in items {
            match item {
                SelectorItem::Kind(kind) => selector.kind = Some(kind),
                SelectorItem::Id(id) => selector.id = Some(id),
                SelectorItem::Tag(tag) => selector.tags.push(tag),
                SelectorItem::Technology(technology) => selector.technology = Some(technology),
                SelectorItem::Meta { key, value } => {
                    selector.meta.push(PolicyMetaSelectorAst { key, value })
                }
            }
        }

        Ok((input, selector))
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TailItem {
        Message(String),
        Suggest(String),
        ExceptSelector(PolicySelectorAst),
        ExceptEdge(PolicyEdgeExceptionAst),
    }

    fn parse_tail_message(input: &str) -> IResult<&str, TailItem> {
        map(
            preceded(tag("message"), preceded(ws1, parse_string)),
            TailItem::Message,
        )
        .parse(input)
    }

    fn parse_tail_suggest(input: &str) -> IResult<&str, TailItem> {
        map(
            preceded(tag("suggest"), preceded(ws1, parse_string)),
            TailItem::Suggest,
        )
        .parse(input)
    }

    fn parse_tail_except_edge(input: &str) -> IResult<&str, TailItem> {
        let (input, _) = tag("except").parse(input)?;
        let (input, _) = ws1(input)?;
        let (input, _) = tag("from").parse(input)?;
        let (input, _) = ws1(input)?;
        let (input, from) = parse_policy_selector(input)?;
        let (input, _) = ws1(input)?;
        let (input, _) = tag("to").parse(input)?;
        let (input, _) = ws1(input)?;
        let (input, to) = parse_policy_selector(input)?;
        Ok((
            input,
            TailItem::ExceptEdge(PolicyEdgeExceptionAst { from, to }),
        ))
    }

    fn parse_tail_except_selector(input: &str) -> IResult<&str, TailItem> {
        let (input, _) = tag("except").parse(input)?;
        let (input, _) = ws1(input)?;
        let (input, selector) = parse_policy_selector(input)?;
        Ok((input, TailItem::ExceptSelector(selector)))
    }

    fn parse_tail_items(input: &str) -> IResult<&str, Vec<TailItem>> {
        many0(preceded(
            ws1,
            alt((
                parse_tail_message,
                parse_tail_suggest,
                parse_tail_except_edge,
                parse_tail_except_selector,
            )),
        ))
        .parse(input)
    }

    fn build_deny_edge(
        from: PolicySelectorAst,
        to: PolicySelectorAst,
        tail: Vec<TailItem>,
    ) -> PolicyRuleAst {
        let mut message: Option<String> = None;
        let mut suggestions: Vec<String> = Vec::new();
        let mut except: Vec<PolicyEdgeExceptionAst> = Vec::new();

        for item in tail {
            match item {
                TailItem::Message(m) => message = Some(m),
                TailItem::Suggest(s) => suggestions.push(s),
                TailItem::ExceptEdge(e) => except.push(e),
                TailItem::ExceptSelector(_) => {}
            }
        }

        PolicyRuleAst::DenyEdge {
            from,
            to,
            except,
            message,
            suggestions,
        }
    }

    fn build_require_tags(
        selector: PolicySelectorAst,
        tags: Vec<String>,
        tail: Vec<TailItem>,
    ) -> PolicyRuleAst {
        let mut message: Option<String> = None;
        let mut suggestions: Vec<String> = Vec::new();
        let mut except: Vec<PolicySelectorAst> = Vec::new();

        for item in tail {
            match item {
                TailItem::Message(m) => message = Some(m),
                TailItem::Suggest(s) => suggestions.push(s),
                TailItem::ExceptSelector(s) => except.push(s),
                TailItem::ExceptEdge(_) => {}
            }
        }

        PolicyRuleAst::RequireTags {
            selector,
            tags,
            except,
            message,
            suggestions,
        }
    }

    fn build_require_metadata(
        selector: PolicySelectorAst,
        key: String,
        value: Option<String>,
        tail: Vec<TailItem>,
    ) -> PolicyRuleAst {
        let mut message: Option<String> = None;
        let mut suggestions: Vec<String> = Vec::new();
        let mut except: Vec<PolicySelectorAst> = Vec::new();

        for item in tail {
            match item {
                TailItem::Message(m) => message = Some(m),
                TailItem::Suggest(s) => suggestions.push(s),
                TailItem::ExceptSelector(s) => except.push(s),
                TailItem::ExceptEdge(_) => {}
            }
        }

        PolicyRuleAst::RequireMetadata {
            selector,
            key,
            value,
            except,
            message,
            suggestions,
        }
    }

    fn build_require_slo(selector: PolicySelectorAst, tail: Vec<TailItem>) -> PolicyRuleAst {
        let mut message: Option<String> = None;
        let mut suggestions: Vec<String> = Vec::new();
        let mut except: Vec<PolicySelectorAst> = Vec::new();

        for item in tail {
            match item {
                TailItem::Message(m) => message = Some(m),
                TailItem::Suggest(s) => suggestions.push(s),
                TailItem::ExceptSelector(s) => except.push(s),
                TailItem::ExceptEdge(_) => {}
            }
        }

        PolicyRuleAst::RequireSlo {
            selector,
            except,
            message,
            suggestions,
        }
    }

    fn parse_policy_rule_deny_edge(input: &str) -> IResult<&str, PolicyRuleAst> {
        let (input, _) = tag("deny").parse(input)?;
        let (input, _) = ws1(input)?;
        let (input, _) = tag("edge").parse(input)?;
        let (input, _) = ws1(input)?;
        let (input, _) = tag("from").parse(input)?;
        let (input, _) = ws1(input)?;
        let (input, from) = parse_policy_selector(input)?;
        let (input, _) = ws1(input)?;
        let (input, _) = tag("to").parse(input)?;
        let (input, _) = ws1(input)?;
        let (input, to) = parse_policy_selector(input)?;
        let (input, tail) = parse_tail_items(input)?;
        Ok((input, build_deny_edge(from, to, tail)))
    }

    fn parse_policy_rule_require_tags(input: &str) -> IResult<&str, PolicyRuleAst> {
        let (input, _) = tag("require").parse(input)?;
        let (input, _) = ws1(input)?;
        let (input, _) = tag("tags").parse(input)?;
        let (input, _) = ws1(input)?;
        let (input, _) = tag("on").parse(input)?;
        let (input, _) = ws1(input)?;
        let (input, selector) = parse_policy_selector(input)?;
        let (input, _) = ws1(input)?;
        let (input, _) = tag("tags").parse(input)?;
        let (input, _) = ws0(input)?;
        let (input, tags) = parse_tags_array(input)?;
        let (input, tail) = parse_tail_items(input)?;
        Ok((input, build_require_tags(selector, tags, tail)))
    }

    fn parse_policy_rule_require_metadata(input: &str) -> IResult<&str, PolicyRuleAst> {
        use nom::combinator::opt;
        use nom::sequence::preceded;

        let (input, _) = tag("require").parse(input)?;
        let (input, _) = ws1(input)?;
        let (input, _) = tag("metadata").parse(input)?;
        let (input, _) = ws1(input)?;
        let (input, _) = tag("on").parse(input)?;
        let (input, _) = ws1(input)?;
        let (input, selector) = parse_policy_selector(input)?;
        let (input, _) = ws1(input)?;
        let (input, _) = tag("key").parse(input)?;
        let (input, _) = ws1(input)?;
        let (input, key) = parse_string(input)?;
        let (input, value) = opt(preceded(
            ws1,
            preceded(tag("value"), preceded(ws1, parse_string)),
        ))
        .parse(input)?;
        let (input, tail) = parse_tail_items(input)?;
        Ok((input, build_require_metadata(selector, key, value, tail)))
    }

    fn parse_policy_rule_require_slo(input: &str) -> IResult<&str, PolicyRuleAst> {
        let (input, _) = tag("require").parse(input)?;
        let (input, _) = ws1(input)?;
        let (input, _) = tag("slo").parse(input)?;
        let (input, _) = ws1(input)?;
        let (input, _) = tag("on").parse(input)?;
        let (input, _) = ws1(input)?;
        let (input, selector) = parse_policy_selector(input)?;
        let (input, tail) = parse_tail_items(input)?;
        Ok((input, build_require_slo(selector, tail)))
    }

    alt((
        parse_policy_rule_deny_edge,
        parse_policy_rule_require_tags,
        parse_policy_rule_require_metadata,
        parse_policy_rule_require_slo,
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::PolicyRuleAst;

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
    fn test_parse_policy_minimal() {
        let input = r#"policy SecurityPolicy"#;
        let result = parse_policy(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, policy) = result.unwrap();
        assert_eq!(policy.id, "SecurityPolicy");
        assert_eq!(policy.title, "SecurityPolicy");
        assert_eq!(policy.category, "general");
        assert_eq!(policy.enforcement, "warn");
    }

    #[test]
    fn test_parse_policy_with_title() {
        let input = r#"policy SecurityPolicy "Security Rules""#;
        let result = parse_policy(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, policy) = result.unwrap();
        assert_eq!(policy.id, "SecurityPolicy");
        assert_eq!(policy.title, "Security Rules");
    }

    #[test]
    fn test_parse_policy_with_block() {
        let input = r#"policy SecurityPolicy "Security Rules" {
            category "security"
            enforcement "deny"
            description "Enforce security policies"
            rule require tags on { kind "container" } tags [#secure] message "Must have secure tag"
        }"#;
        let result = parse_policy(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, policy) = result.unwrap();
        assert_eq!(policy.category, "security");
        assert_eq!(policy.enforcement, "deny");
        assert_eq!(
            policy.description,
            Some("Enforce security policies".to_string())
        );
        assert_eq!(policy.rules.len(), 1);
    }

    #[test]
    fn test_parse_policy_rule_deny_edge() {
        let input = r#"rule deny edge from { id "A" } to { id "B" } message "no direct connection""#;
        let wrapped = format!("{{ {} }}", input);
        let result = parse_policy_block(&wrapped);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, (_, rules)) = result.unwrap();
        assert_eq!(rules.len(), 1);
        match &rules[0] {
            PolicyRuleAst::DenyEdge { from, to, message, .. } => {
                assert_eq!(from.id, Some("A".to_string()));
                assert_eq!(to.id, Some("B".to_string()));
                assert_eq!(message.as_deref(), Some("no direct connection"));
            }
            _ => panic!("expected DenyEdge"),
        }
    }

    #[test]
    fn test_parse_policy_rule_require_tags() {
        let input = r#"rule require tags on { kind "container" } tags [#secure, @compliant] message "Tags required""#;
        let wrapped = format!("{{ {} }}", input);
        let result = parse_policy_block(&wrapped);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, (_, rules)) = result.unwrap();
        assert_eq!(rules.len(), 1);
        match &rules[0] {
            PolicyRuleAst::RequireTags {
                selector,
                tags,
                message,
                ..
            } => {
                assert_eq!(selector.kind, Some("container".to_string()));
                assert_eq!(
                    tags,
                    &vec!["#secure".to_string(), "@compliant".to_string()]
                );
                assert_eq!(message.as_deref(), Some("Tags required"));
            }
            _ => panic!("expected RequireTags"),
        }
    }

    #[test]
    fn test_parse_policy_rule_require_metadata() {
        let input =
            r#"rule require metadata on { kind "service" } key "owner" value "team-alpha""#;
        let wrapped = format!("{{ {} }}", input);
        let result = parse_policy_block(&wrapped);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, (_, rules)) = result.unwrap();
        assert_eq!(rules.len(), 1);
        match &rules[0] {
            PolicyRuleAst::RequireMetadata {
                selector,
                key,
                value,
                ..
            } => {
                assert_eq!(selector.kind, Some("service".to_string()));
                assert_eq!(key, "owner");
                assert_eq!(value.as_deref(), Some("team-alpha"));
            }
            _ => panic!("expected RequireMetadata"),
        }
    }

    #[test]
    fn test_parse_policy_rule_require_slo() {
        let input = r#"rule require slo on { kind "container" } message "SLO required""#;
        let wrapped = format!("{{ {} }}", input);
        let result = parse_policy_block(&wrapped);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, (_, rules)) = result.unwrap();
        assert_eq!(rules.len(), 1);
        match &rules[0] {
            PolicyRuleAst::RequireSlo {
                selector, message, ..
            } => {
                assert_eq!(selector.kind, Some("container".to_string()));
                assert_eq!(message.as_deref(), Some("SLO required"));
            }
            _ => panic!("expected RequireSlo"),
        }
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
    fn test_parse_policy_assignment_minimal() {
        let input = r#"P = policy "Security""#;
        let result = parse_policy_assignment(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, policy) = result.unwrap();
        assert_eq!(policy.id, "P");
        assert_eq!(policy.title, "Security");
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
        assert_eq!(
            req.description,
            Some("Must be available 99.9%".to_string())
        );
        assert_eq!(
            req.tags,
            vec!["#critical".to_string(), "@production".to_string()]
        );
    }

    #[test]
    fn test_parse_policy_with_rules() {
        let input = r#"policy SecurityPolicy "Security Rules" {
            category "security"
            enforcement "deny"
            rule require tags on { kind "container" } tags [#secure]
        }"#;
        let result = parse_policy(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, policy) = result.unwrap();
        assert_eq!(policy.category, "security");
        assert_eq!(policy.enforcement, "deny");
        assert_eq!(policy.rules.len(), 1);
    }

    #[test]
    fn test_parse_policy_with_deny_edge_rule() {
        let input = r#"policy SecurityPolicy "Security Rules" {
            rule deny edge from { id "A" } to { id "B" } message "no direct connection"
        }"#;
        let result = parse_policy(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, policy) = result.unwrap();
        assert_eq!(policy.rules.len(), 1);
    }

    #[test]
    fn test_parse_policy_with_require_metadata_rule() {
        let input = r#"policy SecurityPolicy "Security Rules" {
            rule require metadata on { kind "service" } key "owner" value "team-alpha"
        }"#;
        let result = parse_policy(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, policy) = result.unwrap();
        assert_eq!(policy.rules.len(), 1);
    }

    #[test]
    fn test_parse_policy_with_require_slo_rule() {
        let input = r#"policy SecurityPolicy "Security Rules" {
            rule require slo on { kind "container" } message "SLO required"
        }"#;
        let result = parse_policy(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, policy) = result.unwrap();
        assert_eq!(policy.rules.len(), 1);
    }

    #[test]
    fn test_parse_policy_with_multiple_rules() {
        let input = r#"policy SecurityPolicy "Security Rules" {
            category "security"
            rule require tags on { kind "container" } tags [#secure]
            rule deny edge from { id "A" } to { id "B" }
            rule require metadata on { kind "service" } key "owner"
        }"#;
        let result = parse_policy(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, policy) = result.unwrap();
        assert_eq!(policy.rules.len(), 3);
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
