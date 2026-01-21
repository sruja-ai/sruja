//! Mermaid exporter implementation (initial parity pass)
//!
//! This ports the Go mermaid exporter structure. The Go version relies on the
//! Go `views` engine for focus levels (L1/L2/L3). Rust doesn't have that yet,
//! so this implementation exports the full graph for now and keeps the same
//! visual conventions (styles/classes).

use std::collections::{BTreeMap, BTreeSet, HashMap};

use sruja_language::{collect_elements, Program, Relation};

use super::constants::*;

#[derive(Debug, Clone)]
pub struct MermaidConfig {
    pub direction: String, // "LR" etc.
}

impl Default for MermaidConfig {
    fn default() -> Self {
        Self {
            direction: "LR".to_string(),
        }
    }
}

pub struct MermaidExporter {
    pub config: MermaidConfig,
}

impl MermaidExporter {
    pub fn new(config: MermaidConfig) -> Self {
        Self { config }
    }

    pub fn export(&self, program: &Program) -> String {
        if program.items.is_empty() {
            return String::new();
        }

        let (elements, relations) = collect_elements(program);
        if elements.is_empty() {
            return String::new();
        }

        self.generate(&elements, &relations)
    }

    fn generate(&self, elements: &HashMap<String, sruja_language::ElementDef>, relations: &[Relation]) -> String {
        let mut out = String::new();
        out.push_str(&format!("graph {}\n", self.config.direction));
        out.push('\n');

        self.write_styles(&mut out);
        out.push('\n');

        // Build parent->children clusters using FQN.
        let mut children: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut roots: BTreeSet<String> = BTreeSet::new();

        for fqn in elements.keys() {
            if let Some(parent) = parent_fqn(fqn) {
                if elements.contains_key(&parent) {
                    children.entry(parent).or_default().insert(fqn.clone());
                } else {
                    roots.insert(fqn.clone());
                }
            } else {
                roots.insert(fqn.clone());
            }
        }

        for root in roots {
            if children.contains_key(&root) {
                self.write_subgraph(&mut out, &root, elements, &children, INDENT4);
            } else {
                self.write_node(&mut out, &root, elements, INDENT4);
            }
        }

        out.push('\n');
        for rel in relations {
            self.write_relation(&mut out, rel);
        }

        out
    }

    fn write_styles(&self, out: &mut String) {
        out.push_str(&format!("classDef {} {}\n", CLASS_PERSON, STYLE_PERSON));
        out.push_str(&format!("classDef {} {}\n", CLASS_SYSTEM, STYLE_SYSTEM));
        out.push_str(&format!("classDef {} {}\n", CLASS_CONTAINER, STYLE_CONTAINER));
        out.push_str(&format!("classDef {} {}\n", CLASS_DATABASE, STYLE_DATABASE));
        out.push_str(&format!("classDef {} {}\n", CLASS_QUEUE, STYLE_QUEUE));
        out.push_str(&format!("classDef {} {}\n", CLASS_EXTERNAL, STYLE_EXTERNAL));
        out.push_str(&format!("classDef {} {}\n", CLASS_COMPONENT, STYLE_COMPONENT));
    }

    fn write_subgraph(
        &self,
        out: &mut String,
        parent: &str,
        elements: &HashMap<String, sruja_language::ElementDef>,
        children: &BTreeMap<String, BTreeSet<String>>,
        indent: &str,
    ) {
        let id = sanitize_id(parent);
        let label = escape_quotes(display_title(parent, elements));
        out.push_str(&format!("{indent}subgraph {id}[\"{label}\"]\n"));
        out.push_str(&format!("{indent}{INDENT4}direction TB\n"));

        if let Some(kids) = children.get(parent) {
            for child in kids {
                if children.contains_key(child) {
                    self.write_subgraph(out, child, elements, children, &format!("{indent}{INDENT4}"));
                } else {
                    self.write_node(out, child, elements, &format!("{indent}{INDENT4}"));
                }
            }
        }
        out.push_str(&format!("{indent}end\n"));
    }

    fn write_node(&self, out: &mut String, fqn: &str, elements: &HashMap<String, sruja_language::ElementDef>, indent: &str) {
        let Some(elem) = elements.get(fqn) else { return };
        let kind = elem.assignment.kind.to_string().to_lowercase();

        let id = sanitize_id(fqn);
        let label = escape_quotes(format_label(elem));
        out.push_str(&format!("{indent}{id}[\"{label}\"]\n"));

        let class = match kind.as_str() {
            "person" => Some(CLASS_PERSON),
            "system" => Some(CLASS_SYSTEM),
            "container" => Some(CLASS_CONTAINER),
            "database" => Some(CLASS_DATABASE),
            "queue" => Some(CLASS_QUEUE),
            "component" => Some(CLASS_COMPONENT),
            "externalsystem" | "external" => Some(CLASS_EXTERNAL),
            "datastore" => Some(CLASS_DATABASE),
            _ => None,
        };
        if let Some(class) = class {
            out.push_str(&format!("{indent}class {id} {class}\n"));
        }
    }

    fn write_relation(&self, out: &mut String, rel: &Relation) {
        let from = sanitize_id(&rel.from.as_string());
        let to = sanitize_id(&rel.to.as_string());
        let label = rel.label.as_ref().or(rel.description.as_ref()).map(|s| escape_quotes(s));
        match label {
            Some(l) if !l.is_empty() => {
                out.push_str(&format!("{INDENT4}{from} -->|\"{l}\"| {to}\n"));
            }
            _ => out.push_str(&format!("{INDENT4}{from} --> {to}\n")),
        }
    }
}

fn parent_fqn(fqn: &str) -> Option<String> {
    let idx = fqn.rfind('.')?;
    Some(fqn[..idx].to_string())
}

fn sanitize_id(id: &str) -> String {
    // Mermaid ids can't contain '.'; keep stable mapping.
    id.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}

fn escape_quotes(s: impl AsRef<str>) -> String {
    s.as_ref().replace('"', "\\\"")
}

fn display_title(fqn: &str, elements: &HashMap<String, sruja_language::ElementDef>) -> String {
    elements
        .get(fqn)
        .and_then(|e| e.assignment.title.clone())
        .unwrap_or_else(|| leaf_id(fqn).to_string())
}

fn leaf_id(fqn: &str) -> &str {
    fqn.rsplit('.').next().unwrap_or(fqn)
}

fn format_label(elem: &sruja_language::ElementDef) -> String {
    // Similar to Go: Title + optional description/technology in the label.
    let title = elem
        .assignment
        .title
        .clone()
        .unwrap_or_else(|| elem.assignment.name.clone());
    let id = elem.assignment.name.clone();
    let (desc, tech) = if let Some(body) = &elem.assignment.body {
        (body.description.clone().unwrap_or_default(), body.technology.clone().unwrap_or_default())
    } else {
        (String::new(), String::new())
    };

    let mut lines = vec![title];
    if !desc.is_empty() {
        lines.push(desc);
    }
    if !tech.is_empty() {
        lines.push(tech);
    }
    if lines.len() == 1 && title != id {
        lines.push(id);
    }
    lines.join("\\n")
}

