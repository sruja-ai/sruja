//! Import statement printing.

use sruja_language::{ImportElement, ImportStatement};

pub fn print_import(out: &mut String, import: &ImportStatement) {
    out.push_str("import { ");
    let elems: Vec<String> = import
        .elements
        .iter()
        .map(|e| match e {
            ImportElement::Ident(s) => s.clone(),
            ImportElement::Wildcard => "*".to_string(),
            ImportElement::Boundary => "boundary".to_string(),
            ImportElement::Policy => "policy".to_string(),
        })
        .collect();
    out.push_str(&elems.join(", "));
    out.push_str(" } from \"");
    out.push_str(&import.from);
    out.push_str("\"\n");
}
