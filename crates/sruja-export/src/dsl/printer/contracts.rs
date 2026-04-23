//! API contract printer.

use crate::dsl::printer::definitions::indent;
use sruja_language::Contract;

pub fn print_contract(out: &mut String, c: &Contract, depth: usize) {
    indent(out, depth);
    out.push_str("contract \"");
    out.push_str(&c.name);
    out.push_str("\" {\n");

    if let Some(desc) = &c.description {
        indent(out, depth + 1);
        out.push_str("description \"");
        out.push_str(desc);
        out.push_str("\"\n");
    }

    if !c.inputs.is_empty() {
        indent(out, depth + 1);
        out.push_str("input {\n");
        for field in &c.inputs {
            indent(out, depth + 2);
            out.push_str(&field.name);
            out.push_str(" \"");
            out.push_str(&field.spec);
            out.push_str("\"\n");
        }
        indent(out, depth + 1);
        out.push_str("}\n");
    }

    if !c.outputs.is_empty() {
        indent(out, depth + 1);
        out.push_str("output {\n");
        for field in &c.outputs {
            indent(out, depth + 2);
            out.push_str(&field.name);
            out.push_str(" \"");
            out.push_str(&field.spec);
            out.push_str("\"\n");
        }
        indent(out, depth + 1);
        out.push_str("}\n");
    }

    if !c.errors.is_empty() {
        indent(out, depth + 1);
        out.push_str("error {\n");
        for error in &c.errors {
            indent(out, depth + 2);
            out.push_str("\"");
            out.push_str(&error.code);
            out.push_str("\" \"");
            out.push_str(&error.description);
            out.push_str("\"\n");
        }
        indent(out, depth + 1);
        out.push_str("}\n");
    }

    for constraint in &c.constraints {
        indent(out, depth + 1);
        out.push_str("constraint \"");
        out.push_str(constraint);
        out.push_str("\"\n");
    }

    indent(out, depth);
    out.push_str("}\n");
}
