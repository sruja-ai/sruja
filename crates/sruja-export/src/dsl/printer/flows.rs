//! Scenario and flow printing.

use sruja_language::{Flow, Scenario};

pub fn print_scenario(out: &mut String, scenario: &Scenario) {
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

pub fn print_flow(out: &mut String, flow: &Flow) {
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
