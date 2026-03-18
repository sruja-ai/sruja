//! Relation and element printing (core C4-style nodes).

use sruja_language::{ElementDef, ElementDefBodyItem, Relation};

/// Print a single relation line.
pub fn print_relation(out: &mut String, rel: &Relation, indent: usize) {
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

/// Print an element definition (recursive; uses printer for nested elements and relations).
#[allow(clippy::only_used_in_recursion)]
pub fn print_element(
    printer: &super::DslPrinter,
    out: &mut String,
    elem: &ElementDef,
    indent: usize,
) {
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
                ElementDefBodyItem::ElementDef(nested) => {
                    print_element(printer, out, nested, indent + 1);
                }
                ElementDefBodyItem::Relation(rel) => {
                    print_relation(out, rel, indent + 1);
                }
                ElementDefBodyItem::Tags(tags) => {
                    let formatted = tags
                        .iter()
                        .map(|t| format!("\"{}\"", t))
                        .collect::<Vec<_>>()
                        .join(", ");
                    out.push_str(&format!("{indent_str}  tags [{formatted}]\n"));
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
