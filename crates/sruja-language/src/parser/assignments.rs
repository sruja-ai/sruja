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

pub(crate) fn parse_requirement_assignment(input: &str) -> IResult<&str, Requirement> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('=')).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("requirement").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, r#type) = parse_identifier(input)?;
    let (input, _) = ws1(input)?;
    let (input, title) = parse_string(input)?;

    Ok((
        input,
        Requirement {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            title,
            r#type,
            description: None,
            tags: Vec::new(),
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
    };

    if let Some(kvs) = fields {
        for (k, v) in kvs {
            match k.as_str() {
                "status" => adr.status = Some(v),
                "context" => adr.context = Some(v),
                "decision" => adr.decision = Some(v),
                "consequences" => adr.consequences = Some(v),
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
    let (input, steps) = opt(parse_flow_body).parse(input)?;

    Ok((
        input,
        Flow {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            title: title.unwrap_or_default(),
            description: None,
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
    let (input, steps) = opt(parse_scenario_body).parse(input)?;

    Ok((
        input,
        Scenario {
            location: SourceLocation::new(String::new(), 0, 0),
            id: id.unwrap_or_default(),
            title: title.unwrap_or_default(),
            description,
            steps: steps.unwrap_or_default(),
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

pub(crate) fn parse_scenario_step(input: &str) -> IResult<&str, ScenarioStep> {
    let (input, _) = opt(preceded(tag("step"), ws1)).parse(input)?;
    let (input, from) = parse_qualified_ident(input)?;
    let (input, _) = preceded(ws0, tag("->")).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, to) = parse_qualified_ident(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, tags) = opt(parse_tag_array).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, order_raw) =
        opt(preceded(tag("order"), preceded(ws1, parse_string))).parse(input)?;

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

    Ok((
        input,
        Flow {
            location: SourceLocation::new(String::new(), 0, 0),
            id: id.unwrap_or_default(),
            title: title.unwrap_or_default(),
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
    let (input, _) = opt(preceded(tag("step"), ws1)).parse(input)?;
    let (input, from) = parse_qualified_ident(input)?;
    let (input, _) = preceded(ws0, tag("->")).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, to) = parse_qualified_ident(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string).parse(input)?;

    Ok((
        input,
        ScenarioStep {
            from: Some(from),
            to: Some(to),
            description,
            tags: Vec::new(),
            order: None,
        },
    ))
}

pub(crate) fn parse_requirement(input: &str) -> IResult<&str, Requirement> {
    let (input, _) = tag("requirement").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let id_for_title = id.clone();
    let (input, _) = ws0(input)?;
    let (input, r#type) = opt(parse_identifier).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string).parse(input)?;
    let (input, _) = ws0(input)?;
    let (input, _body) = opt(parse_requirement_body).parse(input)?;

    Ok((
        input,
        Requirement {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            title: description.clone().unwrap_or(id_for_title),
            r#type: r#type.unwrap_or_else(|| "functional".to_string()),
            description,
            tags: Vec::new(),
        },
    ))
}

fn parse_requirement_body(input: &str) -> IResult<&str, ()> {
    use nom::sequence::delimited;
    delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_requirement_property)),
        preceded(ws0, char('}')),
    )
    .parse(input)
    .map(|(i, _)| (i, ()))
}

fn parse_requirement_property(input: &str) -> IResult<&str, ()> {
    preceded(
        alt((
            tag("type"),
            tag("description"),
            tag("tags"),
            tag("metadata"),
        )),
        preceded(
            ws0,
            alt((
                map(parse_string, |_| ()),
                map(parse_tag_array, |_| ()),
                map(parse_metadata_block, |_| ()),
            )),
        ),
    )
    .parse(input)
    .map(|(i, _)| (i, ()))
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
    };

    if let Some(kvs) = fields {
        for (k, v) in kvs {
            match k.as_str() {
                "status" => adr.status = Some(v),
                "context" => adr.context = Some(v),
                "decision" => adr.decision = Some(v),
                "consequences" => adr.consequences = Some(v),
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
                map(parse_kv_string, |(k, v)| PolicyBlockEntry::Kv(k, v)),
                map(parse_policy_rule_line, |r| {
                    PolicyBlockEntry::Rule(Box::new(r))
                }),
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
