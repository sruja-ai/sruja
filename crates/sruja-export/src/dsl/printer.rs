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
                    self.print_relation(&mut out, rel, 0);
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
                TopLevelItem::View(view) => {
                    self.print_view(&mut out, view);
                }
                TopLevelItem::KindDef(kind_def) => {
                    self.print_kind_def(&mut out, kind_def);
                }
                TopLevelItem::TagDef(tag_def) => {
                    self.print_tag_def(&mut out, tag_def);
                }
                TopLevelItem::Overview(overview) => {
                    self.print_overview(&mut out, overview);
                }
                TopLevelItem::Style(style) => {
                    self.print_style(&mut out, style);
                }
                TopLevelItem::Deployment(deployment) => {
                    self.print_deployment(&mut out, deployment, 0);
                }
                TopLevelItem::Constraints(constraints) => {
                    self.print_constraints(&mut out, constraints);
                }
                TopLevelItem::Conventions(conventions) => {
                    self.print_conventions(&mut out, conventions);
                }
                TopLevelItem::Extend(extend) => {
                    self.print_extend(&mut out, extend);
                }
                TopLevelItem::FeedbackLoop(loop_data) => {
                    self.print_feedback_loop(&mut out, loop_data);
                }
                TopLevelItem::CausalLoop(loop_data) => {
                    self.print_causal_loop(&mut out, loop_data);
                }
            }
        }

        // Trim trailing whitespace/newlines for idempotent output
        while out.ends_with('\n') || out.ends_with('\r') {
            out.pop();
        }
        out.push('\n');

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
                        self.print_relation(out, rel, indent + 1);
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

    fn print_relation(&self, out: &mut String, rel: &sruja_language::Relation, indent: usize) {
        let indent_str = "  ".repeat(indent);
        out.push_str(&indent_str);
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
            out.push('"');
            out.push_str(&scenario.title);
            out.push('"');
            out.push(' ');
        }
        if let Some(desc) = &scenario.description {
            out.push('"');
            out.push_str(desc);
            out.push('"');
            out.push(' ');
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
            out.push('"');
            out.push_str(&flow.title);
            out.push('"');
            out.push(' ');
        }
        if let Some(desc) = &flow.description {
            out.push('"');
            out.push_str(desc);
            out.push('"');
            out.push(' ');
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
            // Only print description if it's different from title
            if desc != &req.title {
                out.push_str(" \"");
                out.push_str(desc);
                out.push('"');
            }
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
        // Check if any body fields exist
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

    fn print_policy(&self, out: &mut String, policy: &sruja_language::Policy) {
        out.push_str("policy ");
        out.push_str(&policy.id);
        if !policy.title.is_empty() {
            out.push_str(" \"");
            out.push_str(&policy.title);
            out.push('"');
        }
        // Check if any body fields exist (non-default values)
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

    fn print_view(&self, out: &mut String, view: &sruja_language::ViewDef) {
        out.push_str("view ");
        out.push_str(&view.id);
        if let Some(title) = &view.title {
            out.push_str(" \"");
            out.push_str(title);
            out.push('"');
        }
        if let Some(desc) = &view.description {
            out.push_str(" \"");
            out.push_str(desc);
            out.push('"');
        }
        if !view.rules.is_empty() {
            out.push_str(" {\n");
            for rule in &view.rules {
                if let Some(include) = &rule.include {
                    out.push_str("    include ");
                    if include.wildcard {
                        out.push('*');
                    } else {
                        out.push_str(&include.elements.join(", "));
                    }
                    if include.recursive {
                        out.push_str(" recursive");
                    }
                    out.push('\n');
                }
                if let Some(exclude) = &rule.exclude {
                    out.push_str("    exclude ");
                    if exclude.wildcard {
                        out.push('*');
                    } else {
                        out.push_str(&exclude.elements.join(", "));
                    }
                    out.push('\n');
                }
            }
            out.push_str("}\n");
        } else {
            out.push('\n');
        }
    }

    fn print_kind_def(&self, out: &mut String, kind_def: &sruja_language::ElementKindDef) {
        out.push_str(&kind_def.kind.to_string().to_lowercase());
        out.push_str(" = kind");
        if let Some(title) = &kind_def.title {
            out.push_str(" \"");
            out.push_str(title);
            out.push('"');
        }
        out.push('\n');
    }

    fn print_tag_def(&self, out: &mut String, tag_def: &sruja_language::TagDef) {
        out.push_str("tag ");
        out.push_str(&tag_def.id);
        if let Some(color) = &tag_def.color {
            out.push_str(&format!(" color=\"{}\"", color));
        }
        out.push('\n');
    }

    fn print_overview(&self, out: &mut String, overview: &sruja_language::OverviewBlock) {
        out.push_str("overview {\n");
        if let Some(summary) = &overview.summary {
            out.push_str(&format!("    summary \"{}\"\n", summary));
        }
        if let Some(audience) = &overview.audience {
            out.push_str(&format!("    audience \"{}\"\n", audience));
        }
        if let Some(scope) = &overview.scope {
            out.push_str(&format!("    scope \"{}\"\n", scope));
        }
        if !overview.goals.is_empty() {
            let goals_str = overview
                .goals
                .iter()
                .map(|g| format!("\"{}\"", g))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("    goals [{}]\n", goals_str));
        }
        if !overview.non_goals.is_empty() {
            let non_goals_str = overview
                .non_goals
                .iter()
                .map(|g| format!("\"{}\"", g))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("    non_goals [{}]\n", non_goals_str));
        }
        if !overview.risks.is_empty() {
            let risks_str = overview
                .risks
                .iter()
                .map(|r| format!("\"{}\"", r))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("    risks [{}]\n", risks_str));
        }
        out.push_str("}\n");
    }

    fn print_style(&self, out: &mut String, style: &sruja_language::StyleDecl) {
        out.push_str("style ");
        out.push_str(&style.selector);
        out.push_str(" {\n");
        for (key, value) in &style.properties {
            out.push_str(&format!("    {} \"{}\"\n", key, value));
        }
        out.push_str("}\n");
    }

    fn print_deployment(
        &self,
        out: &mut String,
        deployment: &sruja_language::DeploymentNode,
        indent: usize,
    ) {
        let indent_str = "  ".repeat(indent);
        out.push_str(&indent_str);
        out.push_str("deployment ");
        out.push_str(&deployment.id);
        if let Some(label) = &deployment.label {
            out.push_str(" \"");
            out.push_str(label);
            out.push('"');
        }
        if let Some(tech) = &deployment.technology {
            out.push_str(&format!(" \"{}\"", tech));
        }
        if !deployment.children.is_empty() {
            out.push_str(" {\n");
            for child in &deployment.children {
                self.print_deployment(out, child, indent + 1);
            }
            out.push_str(&indent_str);
            out.push_str("}\n");
        } else {
            out.push('\n');
        }
    }

    fn print_constraints(&self, out: &mut String, constraints: &sruja_language::ConstraintsBlock) {
        out.push_str("constraints {\n");
        for entry in &constraints.entries {
            out.push_str(&format!("    {} \"{}\"\n", entry.key, entry.value));
        }
        out.push_str("}\n");
    }

    fn print_conventions(&self, out: &mut String, conventions: &sruja_language::ConventionsBlock) {
        out.push_str("conventions {\n");
        for entry in &conventions.entries {
            out.push_str(&format!("    {} \"{}\"\n", entry.key, entry.value));
        }
        out.push_str("}\n");
    }

    fn print_extend(&self, out: &mut String, extend: &sruja_language::ExtendElement) {
        out.push_str("extend ");
        out.push_str(&extend.target.as_string());
        out.push_str(" {\n");
        for assignment in &extend.assignments {
            out.push_str("    ");
            out.push_str(&assignment.name);
            out.push_str(" = ");
            out.push_str(&assignment.kind.to_string().to_lowercase());
            if let Some(title) = &assignment.title {
                out.push_str(" \"");
                out.push_str(title);
                out.push('"');
            }
            out.push('\n');
        }
        out.push_str("}\n");
    }

    fn print_feedback_loop(&self, out: &mut String, loop_data: &sruja_language::FeedbackLoop) {
        out.push_str(loop_data.id.as_str());
        out.push_str(" = feedback \"");
        out.push_str(&loop_data.title);
        out.push_str("\" {\n");
        out.push_str(&format!("  loop_type \"{}\"\n", loop_data.loop_type));
        if let Some(loop_id) = &loop_data.loop_id {
            out.push_str(&format!("  loop_id \"{}\"\n", loop_id));
        }
        if let Some(desc) = &loop_data.description {
            out.push_str(&format!("  description \"{}\"\n", desc));
        }
        for rel in &loop_data.relationships {
            self.print_relation(out, rel, 1);
        }
        out.push_str("}\n");
    }

    fn print_causal_loop(&self, out: &mut String, loop_data: &sruja_language::CausalLoop) {
        out.push_str(loop_data.id.as_str());
        out.push_str(" = causal_loop \"");
        out.push_str(&loop_data.title);
        out.push_str("\" {\n");
        out.push_str(&format!("  loop_type \"{}\"\n", loop_data.loop_type));
        if let Some(loop_id) = &loop_data.loop_id {
            out.push_str(&format!("  loop_id \"{}\"\n", loop_id));
        }
        if let Some(desc) = &loop_data.description {
            out.push_str(&format!("  description \"{}\"\n", desc));
        }
        for rel in &loop_data.relationships {
            out.push_str("  ");
            out.push_str(&rel.from);
            out.push_str(" -> ");
            out.push_str(&rel.to);
            out.push_str(" {\n");
            if let Some(effect) = &rel.effect {
                out.push_str(&format!("    effect \"{}\"\n", effect));
            }
            out.push_str(&format!("    polarity \"{}\"\n", rel.polarity));
            if let Some(delay) = &rel.delay {
                out.push_str(&format!("    delay \"{}\"\n", delay));
            }
            out.push_str("  }\n");
        }
        out.push_str("}\n");
    }
}

impl Default for DslPrinter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_language::Parser;

    fn parse(input: &str) -> sruja_language::Program {
        Parser::new("test.sruja".to_string())
            .parse(input)
            .expect("Failed to parse")
    }

    #[test]
    fn test_empty_program_prints_empty() {
        let program = sruja_language::Program::new();
        let printer = DslPrinter::new();
        let out = printer.print(&program);
        assert_eq!(out, "");
    }

    #[test]
    fn test_print_element_and_relation() {
        let input = r#"
person = kind "Person"
system = kind "System"

User = person "User"
A = system "System A" {
  description "A test system"
  technology "Rust"
}
A -> User "serves"
"#;
        let program = parse(input);
        let printer = DslPrinter::new();
        let out = printer.print(&program);

        assert!(out.contains("User = person \"User\""));
        assert!(out.contains("A = system \"System A\""));
        assert!(out.contains("description \"A test system\""));
        assert!(out.contains("technology \"Rust\""));
        assert!(out.contains("A -> User \"serves\""));
    }

    #[test]
    fn test_print_requirement_and_adr() {
        let input = r#"
R1 = requirement functional "Users must login"
ADR001 = adr "Use HTTPS" {
  status "Accepted"
  context "Security"
  decision "Use TLS"
  consequences "Encrypted"
}
"#;
        let program = parse(input);
        let printer = DslPrinter::new();
        let out = printer.print(&program);

        assert!(out.contains("requirement"));
        assert!(out.contains("functional"));
        assert!(out.contains("Users must login"));
        assert!(out.contains("adr"));
        assert!(out.contains("Use HTTPS"));
        assert!(out.contains("status \"Accepted\""));
    }

    #[test]
    fn test_print_import() {
        let input = r#"import { A, B } from "other.sruja"
"#;
        let program = parse(input);
        let printer = DslPrinter::new();
        let out = printer.print(&program);

        assert!(out.contains("import {"));
        assert!(out.contains("A, B"));
        assert!(out.contains("from \"other.sruja\""));
    }
}
