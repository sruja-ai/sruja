//! DSL printer implementation (initial parity pass)
//!
//! Ports Go DSL printer to pretty-print AST back to Sruja DSL format.

use sruja_language::{Program, TopLevelItem};

pub struct DslPrinter;

impl DslPrinter {
    pub fn new() -> Self {
        Self
    }

    pub fn print(&self, program: &Program) -> String {
        if program.items.is_empty() {
            return String::new();
        }

        let mut out = String::new();

        for item in &program.items {
            match item {
                TopLevelItem::ElementDef(elem) => {
                    self.print_element(&mut out, elem, 0);
                }
                TopLevelItem::Relation(rel) => {
                    self.print_relation(&mut out, rel);
                }
                TopLevelItem::Import(import) => {
                    self.print_import(&mut out, import);
                }
                TopLevelItem::Scenario(scenario) => {
                    self.print_scenario(&mut out, scenario);
                }
                TopLevelItem::Flow(flow) => {
                    self.print_flow(&mut out, flow);
                }
                TopLevelItem::Requirement(req) => {
                    self.print_requirement(&mut out, req);
                }
                TopLevelItem::Adr(adr) => {
                    self.print_adr(&mut out, adr);
                }
                TopLevelItem::Policy(policy) => {
                    self.print_policy(&mut out, policy);
                }
                _ => {
                    // TODO: Print other top-level items
                }
            }
            out.push('\n');
        }

        out
    }

    fn print_element(&self, out: &mut String, elem: &sruja_language::ElementDef, indent: usize) {
        let indent_str = "  ".repeat(indent);
        let name = &elem.assignment.name;
        let kind = elem.assignment.kind.to_string().to_lowercase();

        out.push_str(&indent_str);
        out.push_str(name);
        out.push_str(" = ");
        out.push_str(&kind);

        if let Some(sub) = &elem.assignment.sub_kind {
            out.push(' ');
            out.push_str(sub);
        }

        if let Some(title) = &elem.assignment.title {
            out.push_str(" \"");
            out.push_str(title);
            out.push('"');
        }

        for tag in &elem.assignment.tag_refs {
            out.push(' ');
            out.push_str(tag);
        }

        if let Some(body) = &elem.assignment.body {
            out.push_str(" {\n");
            if let Some(desc) = &body.description {
                out.push_str(&format!("{indent_str}  description \"{desc}\"\n"));
            }
            if let Some(tech) = &body.technology {
                out.push_str(&format!("{indent_str}  technology \"{tech}\"\n"));
            }
            for item in &body.items {
                match item {
                    sruja_language::ElementDefBodyItem::ElementDef(nested) => {
                        self.print_element(out, nested, indent + 1);
                    }
                    sruja_language::ElementDefBodyItem::Relation(rel) => {
                        self.print_relation(out, rel);
                    }
                    _ => {}
                }
            }
            out.push_str(&indent_str);
            out.push_str("}\n");
        } else {
            out.push('\n');
        }
    }

    fn print_relation(&self, out: &mut String, rel: &sruja_language::Relation) {
        out.push_str(&rel.from.as_string());
        out.push_str(" -> ");
        out.push_str(&rel.to.as_string());
        if let Some(label) = &rel.label {
            out.push_str(" \"");
            out.push_str(label);
            out.push('"');
        }
        if let Some(tech) = &rel.technology {
            out.push_str(" [technology=\"");
            out.push_str(tech);
            out.push_str("\"]");
        }
        out.push('\n');
    }

    fn print_import(&self, out: &mut String, import: &sruja_language::ImportStatement) {
        out.push_str("import { ");
        let elems: Vec<String> = import
            .elements
            .iter()
            .map(|e| match e {
                sruja_language::ImportElement::Ident(s) => s.clone(),
                sruja_language::ImportElement::Wildcard => "*".to_string(),
            })
            .collect();
        out.push_str(&elems.join(", "));
        out.push_str(" } from \"");
        out.push_str(&import.from);
        out.push_str("\"\n");
    }

    fn print_scenario(&self, out: &mut String, scenario: &sruja_language::Scenario) {
        out.push_str("scenario ");
        if !scenario.id.is_empty() {
            out.push_str(&scenario.id);
            out.push(' ');
        }
        if !scenario.title.is_empty() {
            out.push_str("\"");
            out.push_str(&scenario.title);
            out.push_str("\" ");
        }
        if let Some(desc) = &scenario.description {
            out.push_str("\"");
            out.push_str(desc);
            out.push_str("\" ");
        }
        if !scenario.steps.is_empty() {
            out.push_str("{\n");
            for step in &scenario.steps {
                if let Some(from) = &step.from {
                    out.push_str(&format!("  {} -> ", from.as_string()));
                }
                if let Some(to) = &step.to {
                    out.push_str(&to.as_string());
                }
                if let Some(desc) = &step.description {
                    out.push_str(" \"");
                    out.push_str(desc);
                    out.push('"');
                }
                out.push('\n');
            }
            out.push_str("}\n");
        } else {
            out.push('\n');
        }
    }

    fn print_flow(&self, out: &mut String, flow: &sruja_language::Flow) {
        out.push_str("flow ");
        if !flow.id.is_empty() {
            out.push_str(&flow.id);
            out.push(' ');
        }
        if !flow.title.is_empty() {
            out.push_str("\"");
            out.push_str(&flow.title);
            out.push_str("\" ");
        }
        if let Some(desc) = &flow.description {
            out.push_str("\"");
            out.push_str(desc);
            out.push_str("\" ");
        }
        if !flow.steps.is_empty() {
            out.push_str("{\n");
            for step in &flow.steps {
                if let Some(from) = &step.from {
                    out.push_str(&format!("  {} -> ", from.as_string()));
                }
                if let Some(to) = &step.to {
                    out.push_str(&to.as_string());
                }
                if let Some(desc) = &step.description {
                    out.push_str(" \"");
                    out.push_str(desc);
                    out.push('"');
                }
                out.push('\n');
            }
            out.push_str("}\n");
        } else {
            out.push('\n');
        }
    }

    fn print_requirement(&self, out: &mut String, req: &sruja_language::Requirement) {
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
            out.push_str(" \"");
            out.push_str(desc);
            out.push('"');
        }
        out.push('\n');
    }

    fn print_adr(&self, out: &mut String, adr: &sruja_language::Adr) {
        out.push_str("adr ");
        out.push_str(&adr.id);
        if !adr.title.is_empty() {
            out.push_str(" \"");
            out.push_str(&adr.title);
            out.push('"');
        }
        if let Some(status) = &adr.status {
            out.push_str(" status=\"");
            out.push_str(status);
            out.push('"');
        }
        out.push('\n');
    }

    fn print_policy(&self, out: &mut String, policy: &sruja_language::Policy) {
        out.push_str("policy ");
        out.push_str(&policy.id);
        if !policy.title.is_empty() {
            out.push_str(" \"");
            out.push_str(&policy.title);
            out.push('"');
        }
        out.push_str(&format!(" category=\"{}\"", policy.category));
        out.push_str(&format!(" enforcement=\"{}\"", policy.enforcement));
        if let Some(desc) = &policy.description {
            out.push_str(" \"");
            out.push_str(desc);
            out.push('"');
        }
        out.push('\n');
    }
}

impl Default for DslPrinter {
    fn default() -> Self {
        Self::new()
    }
}
