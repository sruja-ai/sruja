//! View definition printing.

use sruja_language::ViewDef;

pub fn print_view(out: &mut String, view: &ViewDef) {
    out.push_str(&view.id);
    out.push_str(" = view \"");
    out.push_str(&view.title);
    out.push('"');

    if let Some(scope) = &view.view_of {
        out.push_str(" of ");
        out.push_str(&scope.as_string());
    }

    let has_description = view.description.is_some();
    let has_rules = !view.rules.is_empty();

    if has_description || has_rules {
        out.push_str(" {\n");

        if let Some(desc) = &view.description {
            out.push_str("  description \"");
            out.push_str(desc);
            out.push_str("\"\n");
        }

        for rule in &view.rules {
            if let Some(include) = &rule.include {
                out.push_str("  include ");
                if include.wildcard {
                    out.push('*');
                } else {
                    out.push_str(&include.elements.join(" "));
                }
                if include.recursive {
                    out.push_str(" recursive");
                }
                out.push('\n');
            }
            if let Some(exclude) = &rule.exclude {
                out.push_str("  exclude ");
                if exclude.wildcard {
                    out.push('*');
                } else {
                    out.push_str(&exclude.elements.join(" "));
                }
                out.push('\n');
            }
        }
        out.push_str("}\n");
    } else {
        out.push('\n');
    }
}
