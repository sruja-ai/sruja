//! View definition printing.

use sruja_language::ViewDef;

pub fn print_view(out: &mut String, view: &ViewDef) {
    out.push_str("view ");
    out.push_str(&view.id);

    if let Some(scope) = &view.view_of {
        out.push_str(" of ");
        out.push_str(&scope.as_string());
    }

    out.push_str(" {\n");

    if view.title != view.id {
        out.push_str("  title \"");
        out.push_str(&view.title);
        out.push_str("\"\n");
    }

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
}
