//! Style declaration printing.

use sruja_language::StyleDecl;

pub fn print_style(out: &mut String, style: &StyleDecl) {
    out.push_str("style ");
    out.push_str(&style.selector);
    out.push_str(" {\n");
    for (key, value) in &style.properties {
        out.push_str(&format!("    {} \"{}\"\n", key, value));
    }
    out.push_str("}\n");
}
