//! Requirement, ADR, and policy printing.

use sruja_language::{Adr, Policy, PolicyRuleAst, PolicySelectorAst, Requirement};

fn quote_if_dotted(s: &str) -> String {
    if s.contains('.') {
        format!("\"{}\"", s)
    } else {
        s.to_string()
    }
}

pub fn print_requirement(out: &mut String, req: &Requirement) {
    let has_body = req.priority.is_some()
        || req.status.is_some()
        || !req.acceptance_criteria.is_empty()
        || req.user_journey.is_some()
        || !req.scenarios.is_empty()
        || !req.adrs.is_empty()
        || !req.affects.is_empty()
        || req.source.is_some()
        || !req.tags.is_empty()
        || (req.description.is_some() && req.description.as_ref() != Some(&req.title));

    if !has_body {
        out.push_str("requirement ");
        out.push_str(&req.id);
        out.push(' ');
        out.push_str(&req.r#type);
        if !req.title.is_empty() {
            out.push_str(" \"");
            out.push_str(&req.title);
            out.push('"');
        }
        if let Some(desc) = &req.description {
            if desc != &req.title {
                out.push_str(" \"");
                out.push_str(desc);
                out.push('"');
            }
        }
        out.push('\n');
        return;
    }

    out.push_str(&req.id);
    out.push_str(" = requirement ");
    out.push_str(&req.r#type);
    if !req.title.is_empty() {
        out.push_str(" \"");
        out.push_str(&req.title);
        out.push('"');
    }
    out.push_str(" {\n");
    if let Some(desc) = &req.description {
        out.push_str(&format!("    description \"{}\"\n", desc));
    }
    if let Some(priority) = &req.priority {
        out.push_str(&format!("    priority \"{}\"\n", priority));
    }
    if let Some(status) = &req.status {
        out.push_str(&format!("    status \"{}\"\n", status));
    }
    if let Some(user_journey) = &req.user_journey {
        out.push_str(&format!("    user_journey \"{}\"\n", user_journey));
    }
    for ac in &req.acceptance_criteria {
        out.push_str("    acceptance_criteria {\n");
        if let Some(given) = &ac.given {
            out.push_str(&format!("        given \"{}\"\n", given));
        }
        if let Some(when) = &ac.when {
            out.push_str(&format!("        when \"{}\"\n", when));
        }
        if let Some(then) = &ac.then {
            out.push_str(&format!("        then \"{}\"\n", then));
        }
        out.push_str("    }\n");
    }
    for scenario in &req.scenarios {
        out.push_str(&format!("    scenario {}\n", quote_if_dotted(scenario)));
    }
    for adr in &req.adrs {
        out.push_str(&format!("    adr {}\n", quote_if_dotted(adr)));
    }
    for affect in &req.affects {
        out.push_str(&format!("    affects {}\n", quote_if_dotted(affect)));
    }
    if let Some(source) = &req.source {
        out.push_str(&format!("    source \"{}\"\n", source));
    }
    if !req.tags.is_empty() {
        let formatted = req
            .tags
            .iter()
            .map(|t| format!("\"{}\"", t))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("    tags [{}]\n", formatted));
    }
    out.push_str("}\n");
}

pub fn print_adr(out: &mut String, adr: &Adr) {
    out.push_str("adr ");
    out.push_str(&adr.id);
    if !adr.title.is_empty() {
        out.push_str(" \"");
        out.push_str(&adr.title);
        out.push('"');
    }
    let has_body = adr.status.is_some()
        || adr.context.is_some()
        || adr.decision.is_some()
        || adr.consequences.is_some()
        || !adr.affects.is_empty();
    if has_body {
        out.push_str(" {\n");
        if let Some(status) = &adr.status {
            out.push_str(&format!("    status \"{}\"\n", status));
        }
        if let Some(context) = &adr.context {
            out.push_str(&format!("    context \"{}\"\n", context));
        }
        if let Some(decision) = &adr.decision {
            out.push_str(&format!("    decision \"{}\"\n", decision));
        }
        if let Some(consequences) = &adr.consequences {
            out.push_str(&format!("    consequences \"{}\"\n", consequences));
        }
        for affect in &adr.affects {
            out.push_str(&format!("    affects \"{}\"\n", affect));
        }
        out.push_str("}\n");
    } else {
        out.push('\n');
    }
}

pub fn print_policy(out: &mut String, policy: &Policy) {
    out.push_str("policy ");
    out.push_str(&policy.id);
    if !policy.title.is_empty() {
        out.push_str(" \"");
        out.push_str(&policy.title);
        out.push('"');
    }
    let has_body = policy.category != "general"
        || policy.enforcement != "warn"
        || policy.description.is_some()
        || !policy.rules.is_empty();
    if has_body {
        out.push_str(" {\n");
        if policy.category != "general" {
            out.push_str(&format!("    category \"{}\"\n", policy.category));
        }
        if policy.enforcement != "warn" {
            out.push_str(&format!("    enforcement \"{}\"\n", policy.enforcement));
        }
        if let Some(desc) = &policy.description {
            out.push_str(&format!("    description \"{}\"\n", desc));
        }
        for rule in &policy.rules {
            match rule {
                PolicyRuleAst::DenyEdge {
                    from,
                    to,
                    except,
                    message,
                    suggestions,
                } => {
                    out.push_str(&format!(
                        "    rule deny edge from {} to {}",
                        format_selector(from),
                        format_selector(to)
                    ));
                    for e in except {
                        out.push_str(&format!(
                            " except from {} to {}",
                            format_selector(&e.from),
                            format_selector(&e.to)
                        ));
                    }
                    if let Some(m) = message {
                        out.push_str(&format!(" message \"{}\"", m));
                    }
                    for s in suggestions {
                        out.push_str(&format!(" suggest \"{}\"", s));
                    }
                    out.push('\n');
                }
                PolicyRuleAst::RequireTags {
                    selector,
                    tags,
                    except,
                    message,
                    suggestions,
                } => {
                    let formatted = tags
                        .iter()
                        .map(|t| format!("\"{}\"", t))
                        .collect::<Vec<_>>()
                        .join(", ");
                    out.push_str(&format!(
                        "    rule require tags on {} tags [{}]",
                        format_selector(selector),
                        formatted
                    ));
                    for e in except {
                        out.push_str(&format!(" except {}", format_selector(e)));
                    }
                    if let Some(m) = message {
                        out.push_str(&format!(" message \"{}\"", m));
                    }
                    for s in suggestions {
                        out.push_str(&format!(" suggest \"{}\"", s));
                    }
                    out.push('\n');
                }
                PolicyRuleAst::RequireMetadata {
                    selector,
                    key,
                    value,
                    except,
                    message,
                    suggestions,
                } => {
                    out.push_str(&format!(
                        "    rule require metadata on {} key \"{}\"",
                        format_selector(selector),
                        key
                    ));
                    if let Some(v) = value {
                        out.push_str(&format!(" value \"{}\"", v));
                    }
                    for e in except {
                        out.push_str(&format!(" except {}", format_selector(e)));
                    }
                    if let Some(m) = message {
                        out.push_str(&format!(" message \"{}\"", m));
                    }
                    for s in suggestions {
                        out.push_str(&format!(" suggest \"{}\"", s));
                    }
                    out.push('\n');
                }
                PolicyRuleAst::RequireSlo {
                    selector,
                    except,
                    message,
                    suggestions,
                } => {
                    out.push_str(&format!(
                        "    rule require slo on {}",
                        format_selector(selector)
                    ));
                    for e in except {
                        out.push_str(&format!(" except {}", format_selector(e)));
                    }
                    if let Some(m) = message {
                        out.push_str(&format!(" message \"{}\"", m));
                    }
                    for s in suggestions {
                        out.push_str(&format!(" suggest \"{}\"", s));
                    }
                    out.push('\n');
                }
            }
        }
        out.push_str("}\n");
    } else {
        out.push('\n');
    }
}

pub fn print_incident(out: &mut String, inc: &sruja_language::Incident) {
    out.push_str("incident ");
    out.push_str(&inc.id);
    if !inc.title.is_empty() {
        out.push_str(" \"");
        out.push_str(&inc.title);
        out.push('"');
    }
    let has_body = inc.date.is_some()
        || inc.severity.is_some()
        || inc.cause.is_some()
        || inc.resolution.is_some()
        || inc.lesson.is_some()
        || !inc.affected.is_empty();

    if has_body {
        out.push_str(" {\n");
        if let Some(date) = &inc.date {
            out.push_str(&format!("    date \"{}\"\n", date));
        }
        if let Some(severity) = &inc.severity {
            out.push_str(&format!("    severity \"{}\"\n", severity));
        }
        for affected in &inc.affected {
            out.push_str(&format!("    affects {}\n", affected.as_string()));
        }
        if let Some(cause) = &inc.cause {
            out.push_str(&format!("    cause \"{}\"\n", cause));
        }
        if let Some(resolution) = &inc.resolution {
            out.push_str(&format!("    resolution \"{}\"\n", resolution));
        }
        if let Some(lesson) = &inc.lesson {
            out.push_str(&format!("    lesson \"{}\"\n", lesson));
        }
        out.push_str("}\n");
    } else {
        out.push('\n');
    }
}

fn format_selector(selector: &PolicySelectorAst) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(kind) = &selector.kind {
        parts.push(format!("kind \"{}\"", kind));
    }
    if let Some(id) = &selector.id {
        parts.push(format!("id \"{}\"", id));
    }
    for tag in &selector.tags {
        parts.push(format!("tag \"{}\"", tag));
    }
    if let Some(technology) = &selector.technology {
        parts.push(format!("technology \"{}\"", technology));
    }
    for meta in &selector.meta {
        if let Some(value) = &meta.value {
            parts.push(format!("meta \"{}\"=\"{}\"", meta.key, value));
        } else {
            parts.push(format!("meta \"{}\"", meta.key));
        }
    }

    if parts.is_empty() {
        "{}".to_string()
    } else {
        format!("{{ {} }}", parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_diagnostics::SourceLocation;
    use sruja_language::{
        Adr, Policy, PolicyMetaSelectorAst, PolicyRuleAst, PolicySelectorAst, Requirement,
    };

    #[test]
    fn test_print_requirement() {
        let mut out = String::new();
        let req = Requirement {
            location: SourceLocation::new("test".to_string(), 1, 1),
            id: "REQ_1".to_string(),
            r#type: "functional".to_string(),
            title: "Must login".to_string(),
            description: Some("User must be able to login".to_string()),
            tags: vec![],
            priority: None,
            status: None,
            acceptance_criteria: vec![],
            user_journey: None,
            scenarios: vec![],
            adrs: vec![],
            affects: vec![],
            source: None,
        };
        print_requirement(&mut out, &req);
        assert!(out.contains("REQ_1 = requirement functional \"Must login\""));
        assert!(out.contains("description \"User must be able to login\""));
    }

    #[test]
    fn test_print_requirement_with_enriched_fields() {
        use sruja_language::AcceptanceCriteria;
        let mut out = String::new();
        let req = Requirement {
            location: SourceLocation::new("test".to_string(), 1, 1),
            id: "R1".to_string(),
            r#type: "functional".to_string(),
            title: "User can log in".to_string(),
            description: None,
            tags: vec!["auth".to_string()],
            priority: Some("must".to_string()),
            status: Some("accepted".to_string()),
            acceptance_criteria: vec![AcceptanceCriteria {
                given: Some("User has valid credentials".to_string()),
                when: Some("User submits login form".to_string()),
                then: Some("System authenticates and redirects".to_string()),
            }],
            user_journey: Some("User opens app and enters credentials".to_string()),
            scenarios: vec!["LoginFlow".to_string()],
            adrs: vec!["ADR001".to_string()],
            affects: vec!["MySystem.Auth".to_string()],
            source: Some("prd://checkout-prd".to_string()),
        };
        print_requirement(&mut out, &req);
        assert!(out.contains("R1 = requirement functional \"User can log in\""));
        assert!(out.contains("priority \"must\""));
        assert!(out.contains("status \"accepted\""));
        assert!(out.contains("user_journey \"User opens app and enters credentials\""));
        assert!(out.contains("acceptance_criteria {"));
        assert!(out.contains("given \"User has valid credentials\""));
        assert!(out.contains("when \"User submits login form\""));
        assert!(out.contains("then \"System authenticates and redirects\""));
        assert!(out.contains("scenario LoginFlow"));
        assert!(out.contains("adr ADR001"));
        assert!(out.contains("affects \"MySystem.Auth\""));
        assert!(out.contains("source \"prd://checkout-prd\""));
        assert!(out.contains("tags [\"auth\"]"));
    }

    #[test]
    fn test_print_adr() {
        let mut out = String::new();
        let adr = Adr {
            location: SourceLocation::new("test".to_string(), 1, 1),
            id: "ADR_1".to_string(),
            title: "Use Rust".to_string(),
            status: Some("accepted".to_string()),
            context: Some("Need speed".to_string()),
            decision: Some("Rust".to_string()),
            consequences: Some("Fast".to_string()),
            affects: vec![],
        };
        print_adr(&mut out, &adr);
        assert!(out.contains("adr ADR_1 \"Use Rust\" {"));
        assert!(out.contains("status \"accepted\""));
        assert!(out.contains("context \"Need speed\""));
        assert!(out.contains("decision \"Rust\""));
        assert!(out.contains("consequences \"Fast\""));
    }

    #[test]
    fn test_print_policy() {
        let mut out = String::new();
        let policy = Policy {
            location: SourceLocation::new("test".to_string(), 1, 1),
            id: "POL_1".to_string(),
            title: "No circular deps".to_string(),
            category: "architecture".to_string(),
            enforcement: "error".to_string(),
            description: Some("Prevent cycles".to_string()),
            rules: vec![PolicyRuleAst::DenyEdge {
                from: PolicySelectorAst {
                    kind: Some("container".to_string()),
                    id: None,
                    tags: vec![],
                    technology: None,
                    meta: vec![],
                },
                to: PolicySelectorAst {
                    id: Some("DB".to_string()),
                    kind: None,
                    tags: vec![],
                    technology: None,
                    meta: vec![],
                },
                except: vec![],
                message: Some("No DB access".to_string()),
                suggestions: vec!["Use API".to_string()],
            }],
        };
        print_policy(&mut out, &policy);
        assert!(out.contains("policy POL_1 \"No circular deps\" {"));
        assert!(out.contains("category \"architecture\""));
        assert!(out.contains("enforcement \"error\""));
        assert!(out.contains("description \"Prevent cycles\""));
        assert!(out.contains("rule deny edge from { kind \"container\" } to { id \"DB\" } message \"No DB access\" suggest \"Use API\""));
    }

    #[test]
    fn test_format_selector() {
        let selector = PolicySelectorAst {
            kind: Some("component".to_string()),
            id: Some("Auth".to_string()),
            tags: vec!["critical".to_string()],
            technology: Some("Rust".to_string()),
            meta: vec![PolicyMetaSelectorAst {
                key: "tier".to_string(),
                value: Some("1".to_string()),
            }],
        };
        let formatted = format_selector(&selector);
        assert!(formatted.contains("kind \"component\""));
        assert!(formatted.contains("id \"Auth\""));
        assert!(formatted.contains("tag \"critical\""));
        assert!(formatted.contains("technology \"Rust\""));
        assert!(formatted.contains("meta \"tier\"=\"1\""));
    }
}
