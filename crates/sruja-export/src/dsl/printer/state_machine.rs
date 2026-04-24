//! State machine printer.

use crate::dsl::printer::definitions::indent;
use sruja_language::{StateMachine, StateTransition};

pub fn print_state_machine(out: &mut String, sm: &StateMachine, depth: usize) {
    indent(out, depth);
    out.push_str("state_machine \"");
    out.push_str(&sm.name);
    out.push_str("\" {\n");

    if let Some(desc) = &sm.description {
        indent(out, depth + 1);
        out.push_str("description \"");
        out.push_str(desc);
        out.push_str("\"\n");
    }

    indent(out, depth + 1);
    out.push_str("initial \"");
    out.push_str(&sm.initial_state);
    out.push_str("\"\n");

    if !sm.terminal_states.is_empty() {
        indent(out, depth + 1);
        out.push_str("terminal [");
        for (i, state) in sm.terminal_states.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push('"');
            out.push_str(state);
            out.push('"');
        }
        out.push_str("]\n");
    }

    for transition in &sm.transitions {
        print_transition(out, transition, depth + 1);
    }

    indent(out, depth);
    out.push_str("}\n");
}

fn print_transition(out: &mut String, t: &StateTransition, depth: usize) {
    indent(out, depth);
    out.push('"');
    out.push_str(&t.from);
    out.push_str("\" -> \"");
    out.push_str(&t.to);
    out.push_str("\" on \"");
    out.push_str(&t.event);
    out.push('"');

    if t.guard.is_some() || t.action.is_some() || t.description.is_some() {
        out.push_str(" {\n");
        if let Some(g) = &t.guard {
            indent(out, depth + 1);
            out.push_str("guard \"");
            out.push_str(g);
            out.push_str("\"\n");
        }
        if let Some(a) = &t.action {
            indent(out, depth + 1);
            out.push_str("action \"");
            out.push_str(a);
            out.push_str("\"\n");
        }
        if let Some(d) = &t.description {
            indent(out, depth + 1);
            out.push_str("description \"");
            out.push_str(d);
            out.push_str("\"\n");
        }
        indent(out, depth);
        out.push_str("}\n");
    } else {
        out.push('\n');
    }
}
