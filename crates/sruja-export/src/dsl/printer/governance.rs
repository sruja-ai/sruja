//! Requirement, ADR, and policy printing.

use sruja_language::{Adr, Policy, PolicyRuleAst, PolicySelectorAst, Requirement};

pub fn print_requirement(out: &mut String, req: &Requirement) {
    if req.description.is_some() {
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
    out.push('\n');
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
        || adr.consequences.is_some();
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
