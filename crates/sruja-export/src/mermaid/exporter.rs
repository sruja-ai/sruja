//! Mermaid exporter implementation (initial parity pass)
//!
//! This ports the Go mermaid exporter structure. The Go version relies on the
//! Go `views` engine for focus levels (L1/L2/L3). Rust doesn't have that yet,
//! so this implementation exports the full graph for now and keeps the same
//! visual conventions (styles/classes).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use sruja_language::{collect_elements, Program, Relation};

use super::constants::*;

#[derive(Debug, Clone)]
pub struct MermaidConfig {
    pub direction: String, // "LR" etc.
    /// 1=context, 2=container, 3=component
    pub view_level: u8,
    /// Optional node to focus on (system for L2, container for L3)
    pub target_id: Option<String>,
}

impl Default for MermaidConfig {
    fn default() -> Self {
        Self {
            direction: "LR".to_string(),
            view_level: 1,
            target_id: None,
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

    fn generate(
        &self,
        elements: &HashMap<String, sruja_language::ElementDef>,
        relations: &[Relation],
    ) -> String {
        // Filter/project elements and relations per Go views engine semantics (subset).
        let (view_elements, view_relations) = compute_view(
            elements,
            relations,
            self.config.view_level,
            self.config.target_id.clone(),
        );

        let mut out = String::new();
        out.push_str(&format!("graph {}\n", self.config.direction));
        out.push('\n');

        self.write_styles(&mut out);
        out.push('\n');

        // Build parent->children clusters using FQN.
        let mut children: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut roots: BTreeSet<String> = BTreeSet::new();

        for fqn in view_elements.keys() {
            if let Some(parent) = parent_fqn(fqn) {
                if view_elements.contains_key(&parent) {
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
                self.write_subgraph(&mut out, &root, &view_elements, &children, INDENT4);
            } else {
                self.write_node(&mut out, &root, &view_elements, INDENT4);
            }
        }

        out.push('\n');
        for rel in &view_relations {
            self.write_relation(&mut out, rel);
        }

        out
    }

    fn write_styles(&self, out: &mut String) {
        out.push_str(&format!("classDef {} {}\n", CLASS_PERSON, STYLE_PERSON));
        out.push_str(&format!("classDef {} {}\n", CLASS_SYSTEM, STYLE_SYSTEM));
        out.push_str(&format!(
            "classDef {} {}\n",
            CLASS_CONTAINER, STYLE_CONTAINER
        ));
        out.push_str(&format!("classDef {} {}\n", CLASS_DATABASE, STYLE_DATABASE));
        out.push_str(&format!("classDef {} {}\n", CLASS_QUEUE, STYLE_QUEUE));
        out.push_str(&format!("classDef {} {}\n", CLASS_EXTERNAL, STYLE_EXTERNAL));
        out.push_str(&format!(
            "classDef {} {}\n",
            CLASS_COMPONENT, STYLE_COMPONENT
        ));
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
                    self.write_subgraph(
                        out,
                        child,
                        elements,
                        children,
                        &format!("{indent}{INDENT4}"),
                    );
                } else {
                    self.write_node(out, child, elements, &format!("{indent}{INDENT4}"));
                }
            }
        }
        out.push_str(&format!("{indent}end\n"));
    }

    fn write_node(
        &self,
        out: &mut String,
        fqn: &str,
        elements: &HashMap<String, sruja_language::ElementDef>,
        indent: &str,
    ) {
        let Some(elem) = elements.get(fqn) else {
            return;
        };
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
        let label = rel.label.as_ref().or(rel.description.as_ref()).map(|s| {
            let one_line = first_line_truncated(s, MAX_EDGE_LABEL_CHARS);
            escape_quotes(newlines_to_br(&one_line))
        });
        match label {
            Some(l) if !l.is_empty() => {
                out.push_str(&format!("{INDENT4}{from} -->|\"{l}\"| {to}\n"));
            }
            _ => out.push_str(&format!("{INDENT4}{from} --> {to}\n")),
        }
    }
}

fn normalize_kind(kind: &str) -> String {
    let lower = kind.to_lowercase();
    match lower.as_str() {
        "database" | "db" | "storage" => "datastore".to_string(),
        "mq" => "queue".to_string(),
        "actor" => "person".to_string(),
        _ => lower,
    }
}

fn compute_view(
    elements: &HashMap<String, sruja_language::ElementDef>,
    relations: &[Relation],
    mut level: u8,
    mut focus: Option<String>,
) -> (HashMap<String, sruja_language::ElementDef>, Vec<Relation>) {
    // Auto-detect L2 (Go behavior): if L1, no focus, exactly one system and it has children.
    if level <= 1 && focus.is_none() {
        let systems: Vec<String> = elements
            .iter()
            .filter_map(|(id, e)| {
                let kind_str = e.assignment.kind.to_string();
                if normalize_kind(&kind_str) == "system" {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();

        if systems.len() == 1 {
            let sys = &systems[0];
            let prefix = format!("{}.", sys);
            let has_children = elements.keys().any(|id| id.starts_with(&prefix));
            if has_children {
                level = 2;
                focus = Some(sys.clone());
            }
        }
    }

    // Determine core set.
    let mut visible: HashSet<String> = HashSet::new();

    let is_core = |id: &str| -> bool {
        match level {
            1 => {
                if let Some(e) = elements.get(id) {
                    let kind_str = e.assignment.kind.to_string();
                    let k = normalize_kind(&kind_str);
                    k == "person" || k == "system"
                } else {
                    false
                }
            }
            2 => {
                if id == focus.as_deref().unwrap_or("") {
                    return true;
                }
                if let Some(e) = elements.get(id) {
                    let kind_str = e.assignment.kind.to_string();
                    let k = normalize_kind(&kind_str);
                    k == "container"
                        || k == "datastore"
                        || k == "queue"
                        || k == "system"
                        || k == "person"
                } else {
                    false
                }
            }
            _ => true,
        }
    };

    let add_element = |visible: &mut HashSet<String>, id: &str| {
        if elements.contains_key(id) {
            visible.insert(id.to_string());
        }
    };

    match level {
        1 => {
            for id in elements.keys() {
                if is_core(id) {
                    add_element(&mut visible, id);
                }
            }
        }
        2 => {
            if let Some(focus_id) = focus.as_deref() {
                if !elements.contains_key(focus_id) {
                    return (HashMap::new(), Vec::new());
                }
                add_element(&mut visible, focus_id);
                let prefix = format!("{}.", focus_id);
                for id in elements.keys() {
                    if id.starts_with(&prefix) && is_core(id) {
                        add_element(&mut visible, id);
                    }
                }
            } else {
                for id in elements.keys() {
                    if is_core(id) {
                        add_element(&mut visible, id);
                    }
                }
            }
        }
        3 => {
            // L3: if focus, include focus and all descendants; otherwise all.
            if let Some(focus_id) = focus.as_deref() {
                if !elements.contains_key(focus_id) {
                    return (HashMap::new(), Vec::new());
                }
                add_element(&mut visible, focus_id);
                let prefix = format!("{}.", focus_id);
                for id in elements.keys() {
                    if id.starts_with(&prefix) {
                        add_element(&mut visible, id);
                    }
                }
            } else {
                for id in elements.keys() {
                    add_element(&mut visible, id);
                }
            }
        }
        _ => {}
    }

    // Relation projection (simplified from Go views engine).
    let focus_ref = focus.as_deref();
    let mut projected: Vec<Relation> = Vec::new();
    for rel in relations {
        let from = rel.from.as_string();
        let to = rel.to.as_string();
        let source = project_id(&from, level, focus_ref, elements);
        let target = project_id(&to, level, focus_ref, elements);
        if source.is_empty() || target.is_empty() || source == target {
            continue;
        }
        if source.starts_with(&(target.clone() + "."))
            || target.starts_with(&(source.clone() + "."))
        {
            continue;
        }
        if !visible.contains(&source) && !visible.contains(&target) {
            continue;
        }
        visible.insert(source.clone());
        visible.insert(target.clone());

        projected.push(Relation {
            location: rel.location.clone(),
            from: sruja_language::QualifiedIdent::qualified(
                source.split('.').map(|s| s.to_string()).collect(),
            ),
            to: sruja_language::QualifiedIdent::qualified(
                target.split('.').map(|s| s.to_string()).collect(),
            ),
            label: rel.label.clone(),
            description: rel.description.clone(),
            technology: rel.technology.clone(),
            tags: rel.tags.clone(),
        });
    }

    // Build filtered element map
    let mut out_elems: HashMap<String, sruja_language::ElementDef> = HashMap::new();
    for id in visible {
        if let Some(e) = elements.get(&id) {
            out_elems.insert(id, e.clone());
        }
    }

    (out_elems, projected)
}

fn project_id(
    fqn: &str,
    level: u8,
    focus: Option<&str>,
    elements: &HashMap<String, sruja_language::ElementDef>,
) -> String {
    if elements.contains_key(fqn) {
        if level == 2 {
            if let Some(c) = get_container(fqn, elements) {
                return c;
            }
            return get_root(fqn);
        }
        if level == 3 {
            if let Some(c) = get_container(fqn, elements) {
                if Some(c.as_str()) != focus {
                    return c;
                }
            }
            return get_root(fqn);
        }
        return fqn.to_string();
    }
    // fallback: root
    get_root(fqn)
}

fn get_root(fqn: &str) -> String {
    fqn.split('.').next().unwrap_or(fqn).to_string()
}

fn get_container(
    fqn: &str,
    elements: &HashMap<String, sruja_language::ElementDef>,
) -> Option<String> {
    // Walk up ancestors: fqn, parent, grandparent...
    let mut cur = fqn.to_string();
    loop {
        if let Some(e) = elements.get(&cur) {
            if normalize_kind(&e.assignment.kind.to_string()) == "container" {
                return Some(cur);
            }
        }
        let p = parent_fqn(&cur)?;
        cur = p;
    }
}

fn parent_fqn(fqn: &str) -> Option<String> {
    let idx = fqn.rfind('.')?;
    Some(fqn[..idx].to_string())
}

fn sanitize_id(id: &str) -> String {
    // Mermaid ids can't contain '.'; keep stable mapping.
    id.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn escape_quotes(s: impl AsRef<str>) -> String {
    s.as_ref().replace('"', "\\\"")
}

/// Truncate a single line at word boundary; add ellipsis if truncated.
fn truncate_line(s: &str, max_chars: usize) -> String {
    let s = s.trim();
    if s.len() <= max_chars {
        return s.to_string();
    }
    let truncated = &s[..max_chars.saturating_sub(1)];
    if let Some(last_space) = truncated.rfind(|c: char| c.is_whitespace()) {
        format!("{}…", truncated[..last_space].trim_end())
    } else {
        format!("{}…", truncated)
    }
}

/// Take first line of text, optionally truncated; normalize newlines to space.
fn first_line_truncated(s: &str, max_chars: usize) -> String {
    let first = s
        .lines()
        .next()
        .map(|l| l.trim())
        .unwrap_or("")
        .replace('\r', " ");
    truncate_line(&first.replace('\n', " ").replace("  ", " "), max_chars)
}

/// Replace newlines and literal `\n` with `<br>` so Mermaid renders line breaks in node labels.
fn newlines_to_br(s: impl AsRef<str>) -> String {
    let s = s.as_ref();
    let s = s.replace("\\n", "<br>");
    let s = s.replace("\r\n", "<br>");
    let s = s.replace('\r', "<br>");
    s.replace('\n', "<br>")
}

fn display_title(fqn: &str, elements: &HashMap<String, sruja_language::ElementDef>) -> String {
    let raw = elements
        .get(fqn)
        .and_then(|e| e.assignment.title.clone())
        .unwrap_or_else(|| leaf_id(fqn).to_string());
    let normalized = newlines_to_br(&raw);
    first_line_truncated(&normalized.replace("<br>", " "), MAX_SUBGRAPH_TITLE_CHARS)
}

fn leaf_id(fqn: &str) -> &str {
    fqn.rsplit('.').next().unwrap_or(fqn)
}

fn format_label(elem: &sruja_language::ElementDef) -> String {
    // Best practice: keep node labels short so Mermaid can wrap/display without cutoff.
    // Title (primary), then up to one line of description, then technology; max MAX_NODE_LINES.
    let title = elem
        .assignment
        .title
        .clone()
        .unwrap_or_else(|| elem.assignment.name.clone());
    let id = elem.assignment.name.clone();
    let (desc, tech) = if let Some(body) = &elem.assignment.body {
        (
            body.description.clone().unwrap_or_default(),
            body.technology.clone().unwrap_or_default(),
        )
    } else {
        (String::new(), String::new())
    };

    let title_flat = title.replace(['\n', '\r'], " ");
    let line1 = truncate_line(title_flat.trim(), MAX_NODE_LINE_CHARS);
    let mut lines = vec![line1];

    if lines.len() < MAX_NODE_LINES && !desc.is_empty() {
        let desc_line = first_line_truncated(&desc, MAX_NODE_LINE_CHARS);
        if !desc_line.is_empty() {
            lines.push(desc_line);
        }
    }
    if lines.len() < MAX_NODE_LINES && !tech.is_empty() {
        let tech_line = truncate_line(tech.trim(), MAX_NODE_LINE_CHARS);
        if !tech_line.is_empty() {
            lines.push(tech_line);
        }
    }
    if lines.len() == 1 && title != id {
        let id_line = truncate_line(&id, MAX_NODE_LINE_CHARS);
        if !id_line.is_empty() && lines.len() < MAX_NODE_LINES {
            lines.push(id_line);
        }
    }
    lines.join("<br>")
}
