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
        if let Some(doc) = &body.doc {
            out.push_str(&format!("{indent_str}  doc \"{doc}\"\n"));
        }
        if let Some(id) = &body.canonical_id {
            out.push_str(&format!("{indent_str}  id \"{id}\"\n"));
        }
        if !body.aliases.is_empty() {
            let formatted = body
                .aliases
                .iter()
                .map(|a| format!("\"{}\"", a))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("{indent_str}  aliases [{formatted}]\n"));
        }
        if let Some(owner) = &body.owner {
            out.push_str(&format!("{indent_str}  owner \"{owner}\"\n"));
        }
        if let Some(domain) = &body.domain {
            out.push_str(&format!("{indent_str}  domain \"{domain}\"\n"));
        }
        if let Some(criticality) = &body.criticality {
            out.push_str(&format!(
                "{indent_str}  criticality {}\n",
                criticality.as_str()
            ));
        }
        for source in &body.sources {
            out.push_str(&format!(
                "{indent_str}  source {} \"{}\"\n",
                source.kind, source.path
            ));
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
        if !body.metadata.is_empty() {
            out.push_str(&format!("{indent_str}  metadata {{\n"));
            for entry in &body.metadata {
                out.push_str(&format!("{indent_str}    {}", entry.key));
                if let Some(value) = &entry.value {
                    out.push_str(&format!(" \"{}\"", value));
                }
                out.push('\n');
            }
            out.push_str(&format!("{indent_str}  }}\n"));
        }
        if !body.constraints.is_empty() {
            out.push_str(&format!("{indent_str}  constraints {{\n"));
            for entry in &body.constraints {
                out.push_str(&format!(
                    "{indent_str}    {} \"{}\"\n",
                    entry.key, entry.value
                ));
            }
            out.push_str(&format!("{indent_str}  }}\n"));
        }
        if !body.conventions.is_empty() {
            out.push_str(&format!("{indent_str}  conventions {{\n"));
            for entry in &body.conventions {
                out.push_str(&format!(
                    "{indent_str}    {} \"{}\"\n",
                    entry.key, entry.value
                ));
            }
            out.push_str(&format!("{indent_str}  }}\n"));
        }
        if let Some(scale) = &body.scale {
            out.push_str(&format!("{indent_str}  scale {{\n"));
            if let Some(min) = scale.min {
                out.push_str(&format!("{indent_str}    min {min}\n"));
            }
            if let Some(max) = scale.max {
                out.push_str(&format!("{indent_str}    max {max}\n"));
            }
            if let Some(metric) = &scale.metric {
                out.push_str(&format!("{indent_str}    metric \"{metric}\"\n"));
            }
            out.push_str(&format!("{indent_str}  }}\n"));
        }
        if let Some(slo) = &body.slo {
            out.push_str(&format!("{indent_str}  slo {{\n"));
            if let Some(av) = &slo.availability {
                out.push_str(&format!("{indent_str}    availability {{\n"));
                if let Some(target) = &av.target {
                    out.push_str(&format!("{indent_str}      target \"{target}\"\n"));
                }
                if let Some(window) = &av.window {
                    out.push_str(&format!("{indent_str}      window \"{window}\"\n"));
                }
                if let Some(current) = &av.current {
                    out.push_str(&format!("{indent_str}      current \"{current}\"\n"));
                }
                out.push_str(&format!("{indent_str}    }}\n"));
            }
            if let Some(lat) = &slo.latency {
                out.push_str(&format!("{indent_str}    latency {{\n"));
                if let Some(p95) = &lat.p95 {
                    out.push_str(&format!("{indent_str}      p95 \"{p95}\"\n"));
                }
                if let Some(p99) = &lat.p99 {
                    out.push_str(&format!("{indent_str}      p99 \"{p99}\"\n"));
                }
                if let Some(window) = &lat.window {
                    out.push_str(&format!("{indent_str}      window \"{window}\"\n"));
                }
                if let Some(current) = &lat.current {
                    out.push_str(&format!("{indent_str}      current {{\n"));
                    if let Some(p95) = &current.p95 {
                        out.push_str(&format!("{indent_str}        p95 \"{p95}\"\n"));
                    }
                    if let Some(p99) = &current.p99 {
                        out.push_str(&format!("{indent_str}        p99 \"{p99}\"\n"));
                    }
                    out.push_str(&format!("{indent_str}      }}\n"));
                }
                out.push_str(&format!("{indent_str}    }}\n"));
            }
            if let Some(er) = &slo.error_rate {
                out.push_str(&format!("{indent_str}    errorRate {{\n"));
                if let Some(target) = &er.target {
                    out.push_str(&format!("{indent_str}      target \"{target}\"\n"));
                }
                if let Some(window) = &er.window {
                    out.push_str(&format!("{indent_str}      window \"{window}\"\n"));
                }
                if let Some(current) = &er.current {
                    out.push_str(&format!("{indent_str}      current \"{current}\"\n"));
                }
                out.push_str(&format!("{indent_str}    }}\n"));
            }
            if let Some(tp) = &slo.throughput {
                out.push_str(&format!("{indent_str}    throughput {{\n"));
                if let Some(target) = &tp.target {
                    out.push_str(&format!("{indent_str}      target \"{target}\"\n"));
                }
                if let Some(window) = &tp.window {
                    out.push_str(&format!("{indent_str}      window \"{window}\"\n"));
                }
                if let Some(current) = &tp.current {
                    out.push_str(&format!("{indent_str}      current \"{current}\"\n"));
                }
                out.push_str(&format!("{indent_str}    }}\n"));
            }
            out.push_str(&format!("{indent_str}  }}\n"));
        }
        if let Some(style) = &body.style {
            out.push_str(&format!("{indent_str}  style self {{\n"));
            let mut keys: Vec<&String> = style.properties.keys().collect();
            keys.sort();
            for key in keys {
                if let Some(value) = style.properties.get(key) {
                    out.push_str(&format!("{indent_str}    {key} \"{value}\"\n"));
                }
            }
            out.push_str(&format!("{indent_str}  }}\n"));
        }
        out.push_str(&indent_str);
        out.push_str("}\n");
    } else {
        out.push('\n');
    }
}
