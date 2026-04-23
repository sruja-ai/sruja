//! Kind and tag definition printing.

use sruja_language::{ElementKindDef, TagDef};

pub fn print_kind_def(out: &mut String, kind_def: &ElementKindDef) {
    out.push_str(&kind_def.kind.to_string().to_lowercase());
    out.push_str(" = kind");
    if let Some(title) = &kind_def.title {
        out.push_str(" \"");
        out.push_str(title);
        out.push('"');
    }
    out.push('\n');
}

pub fn print_tag_def(out: &mut String, tag_def: &TagDef) {
    out.push_str("tag ");
    out.push_str(&tag_def.id);
    if let Some(color) = &tag_def.color {
        out.push_str(&format!(" color=\"{}\"", color));
    }
    out.push('\n');
}

pub fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}
