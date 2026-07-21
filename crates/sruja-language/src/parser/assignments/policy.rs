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

use crate::ast::{
    Policy, PolicyEdgeExceptionAst, PolicyMetaSelectorAst, PolicyRuleAst, PolicySelectorAst,
};

use super::super::primitives::{
    parse_identifier, parse_kv_string, parse_string, parse_string_array, parse_tag_array, ws, ws0,
    ws1,
};

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
    Rule(Box<PolicyRuleAst>),
}

type PolicyBlockResult = (Vec<(String, String)>, Vec<PolicyRuleAst>);

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

    let rules: Vec<PolicyRuleAst> = entries
        .into_iter()
        .filter_map(|e| match e {
            PolicyBlockEntry::Rule(r) => Some(*r),
            _ => None,
        })
        .collect();

    Ok((input, (kvs, rules)))
}

fn parse_policy_rule_line(input: &str) -> IResult<&str, PolicyRuleAst> {
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
        let input =
            r#"rule deny edge from { id "A" } to { id "B" } message "no direct connection""#;
        let wrapped = format!("{{ {} }}", input);
        let result = parse_policy_block(&wrapped);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, (_, rules)) = result.unwrap();
        assert_eq!(rules.len(), 1);
        match &rules[0] {
            PolicyRuleAst::DenyEdge {
                from, to, message, ..
            } => {
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
                assert_eq!(tags, &vec!["#secure".to_string(), "@compliant".to_string()]);
                assert_eq!(message.as_deref(), Some("Tags required"));
            }
            _ => panic!("expected RequireTags"),
        }
    }

    #[test]
    fn test_parse_policy_rule_require_metadata() {
        let input = r#"rule require metadata on { kind "service" } key "owner" value "team-alpha""#;
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
    fn test_parse_policy_assignment_minimal() {
        let input = r#"P = policy "Security""#;
        let result = parse_policy_assignment(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, policy) = result.unwrap();
        assert_eq!(policy.id, "P");
        assert_eq!(policy.title, "Security");
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
}
