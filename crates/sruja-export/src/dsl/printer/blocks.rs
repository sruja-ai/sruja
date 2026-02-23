//! Constraints, conventions, and extend block printing.

use sruja_language::{ConventionsBlock, ConstraintsBlock, ExtendElement};

pub fn print_constraints(out: &mut String, constraints: &ConstraintsBlock) {
    out.push_str("constraints {\n");
    for entry in &constraints.entries {
        out.push_str(&format!("    {} \"{}\"\n", entry.key, entry.value));
    }
    out.push_str("}\n");
}

pub fn print_conventions(out: &mut String, conventions: &ConventionsBlock) {
    out.push_str("conventions {\n");
    for entry in &conventions.entries {
        out.push_str(&format!("    {} \"{}\"\n", entry.key, entry.value));
    }
    out.push_str("}\n");
}

pub fn print_extend(out: &mut String, extend: &ExtendElement) {
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
