//! Requirement, ADR, and policy printing.

use sruja_language::{Adr, Policy, Requirement};

pub fn print_requirement(out: &mut String, req: &Requirement) {
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
        || policy.description.is_some();
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
        out.push_str("}\n");
    } else {
        out.push('\n');
    }
}
