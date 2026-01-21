//! DOT exporter implementation (initial parity pass)
//!
//! Ports Go dot exporter structure. Uses the same view-level filtering
//! approach as Mermaid for consistency.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use sruja_language::{collect_elements, ElementKind, Program, Relation};

use super::constants::*;

#[derive(Debug, Clone)]
pub struct DotConfig {
    pub rank_dir: String, // "TB" or "LR"
    pub node_sep: f64,
    pub rank_sep: f64,
    pub view_level: u8,
    pub target_id: Option<String>,
}

impl Default for DotConfig {
    fn default() -> Self {
        Self {
            rank_dir: "TB".to_string(),
            node_sep: DEFAULT_NODE_SEP,
            rank_sep: DEFAULT_RANK_SEP,
            view_level: 1,
            target_id: None,
        }
    }
}

pub struct DotExporter {
    pub config: DotConfig,
}

impl DotExporter {
    pub fn new(config: DotConfig) -> Self {
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

        // Apply view-level filtering (reuse Mermaid logic)
        let (view_elements, view_relations) = compute_view(
            &elements,
            &relations,
            self.config.view_level,
            self.config.target_id.as_deref(),
        );

        self.generate(&view_elements, &view_relations)
    }

    fn generate(
        &self,
        elements: &HashMap<String, sruja_language::ElementDef>,
        relations: &[Relation],
    ) -> String {
        let mut out = String::new();
        out.push_str("digraph G {\n");
        out.push_str("  graph [\n");
        out.push_str(&format!("    rankdir=\"{}\",\n", self.config.rank_dir));
        out.push_str(&format!("    nodesep={:.2},\n", self.config.node_sep / 72.0));
        out.push_str(&format!("    ranksep={:.2},\n", self.config.rank_sep / 72.0));
        out.push_str("    layout=\"dot\",\n");
        out.push_str("    compound=true,\n");
        out.push_str("    splines=ortho,\n");
        out.push_str(&format!("    fontname=\"{}\",\n", FONT_NAME));
        out.push_str(&format!("    fontsize={},\n", FONT_SIZE_GLOBAL));
        out.push_str(&format!("    pad={:.1},\n", GRAPH_PAD));
        out.push_str("    dpi=72\n");
        out.push_str("  ];\n\n");

        // Global node/edge attributes
        out.push_str("  node [\n");
        out.push_str(&format!("    fontname=\"{}\",\n", FONT_NAME));
        out.push_str(&format!("    fontsize={},\n", FONT_SIZE_GLOBAL));
        out.push_str(&format!("    fontcolor=\"{}\",\n", COLOR_SLATE_700));
        out.push_str("    shape=box,\n");
        out.push_str("    style=rounded,filled\n");
        out.push_str("  ];\n\n");

        out.push_str("  edge [\n");
        out.push_str(&format!("    fontname=\"{}\",\n", FONT_NAME));
        out.push_str(&format!("    fontsize={},\n", FONT_SIZE_EDGE));
        out.push_str(&format!("    fontcolor=\"{}\",\n", COLOR_SLATE_700));
        out.push_str(&format!("    color=\"{}\",\n", COLOR_SLATE_500));
        out.push_str(&format!("    penwidth={},\n", PEN_WIDTH_EDGE));
        out.push_str(&format!("    arrowsize={:.2}\n", ARROW_SIZE));
        out.push_str("  ];\n\n");

        // Build clusters
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

        // Write root nodes
        for root in &roots {
            if !children.contains_key(root) {
                self.write_node(&mut out, root, elements);
            }
        }

        // Write clusters
        for (parent, kids) in &children {
            self.write_cluster(&mut out, parent, kids, elements);
        }

        // Write edges
        for rel in relations {
            self.write_edge(&mut out, rel);
        }

        out.push_str("}\n");
        out
    }

    fn write_node(&self, out: &mut String, fqn: &str, elements: &HashMap<String, sruja_language::ElementDef>) {
        let Some(elem) = elements.get(fqn) else { return };
        let id = escape_id(fqn);
        let label = build_html_label(elem);
        let (width, height) = get_node_size(elem);

        out.push_str(&format!("  \"{}\" [\n", id));
        out.push_str(&format!("    label=<{}>,\n", label));
        if width > 0.0 {
            out.push_str(&format!("    width={:.2},\n", width / 72.0));
        }
        if height > 0.0 {
            out.push_str(&format!("    height={:.2},\n", height / 72.0));
        }
        out.push_str("    margin=0\n");
        out.push_str("  ];\n");
    }

    fn write_cluster(
        &self,
        out: &mut String,
        parent: &str,
        children: &BTreeSet<String>,
        elements: &HashMap<String, sruja_language::ElementDef>,
    ) {
        let id = escape_id(parent);
        let label = display_title(parent, elements);
        out.push_str(&format!("  subgraph cluster_{} {{\n", id));
        out.push_str(&format!("    label=\"{}\";\n", escape_quotes(&label)));
        out.push_str(&format!("    style=filled;\n"));
        out.push_str(&format!("    fillcolor=\"{}\";\n", COLOR_GRAY_BG));
        out.push_str(&format!("    fontcolor=\"{}\";\n", COLOR_SLATE_800));
        out.push_str(&format!("    fontsize={};\n", FONT_SIZE_GLOBAL + 2));

        for child in children {
            if let Some(e) = elements.get(child) {
                let child_id = escape_id(child);
                let child_label = build_html_label(e);
                let (width, height) = get_node_size(e);
                out.push_str(&format!("    \"{}\" [\n", child_id));
                out.push_str(&format!("      label=<{}>,\n", child_label));
                if width > 0.0 {
                    out.push_str(&format!("      width={:.2},\n", width / 72.0));
                }
                if height > 0.0 {
                    out.push_str(&format!("      height={:.2},\n", height / 72.0));
                }
                out.push_str("      margin=0\n");
                out.push_str("    ];\n");
            }
        }

        out.push_str("  }\n");
    }

    fn write_edge(&self, out: &mut String, rel: &Relation) {
        let from = escape_id(&rel.from.as_string());
        let to = escape_id(&rel.to.as_string());
        let label = rel.label.as_ref().or(rel.description.as_ref());
        match label {
            Some(l) if !l.is_empty() => {
                out.push_str(&format!(
                    "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
                    from,
                    to,
                    escape_quotes(l)
                ));
            }
            _ => out.push_str(&format!("  \"{}\" -> \"{}\";\n", from, to)),
        }
    }
}

// Reuse view computation from Mermaid (same logic)
fn compute_view(
    elements: &HashMap<String, sruja_language::ElementDef>,
    relations: &[Relation],
    mut level: u8,
    mut focus: Option<&str>,
) -> (HashMap<String, sruja_language::ElementDef>, Vec<Relation>) {
    if level <= 1 && focus.is_none() {
        let systems: Vec<String> = elements
            .iter()
            .filter_map(|(id, e)| {
                if normalize_kind(&e.assignment.kind.to_string()) == "system" {
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
                focus = Some(sys.as_str());
            }
        }
    }

    let mut visible: HashSet<String> = HashSet::new();

    let is_core = |id: &str| -> bool {
        match level {
            1 => {
                if let Some(e) = elements.get(id) {
                    let k = normalize_kind(&e.assignment.kind.to_string());
                    k == "person" || k == "system"
                } else {
                    false
                }
            }
            2 => {
                if id == focus.unwrap_or("") {
                    return true;
                }
                if let Some(e) = elements.get(id) {
                    let k = normalize_kind(&e.assignment.kind.to_string());
                    k == "container" || k == "datastore" || k == "queue" || k == "system" || k == "person"
                } else {
                    false
                }
            }
            _ => true,
        }
    };

    match level {
        1 => {
            for id in elements.keys() {
                if is_core(id) {
                    visible.insert(id.clone());
                }
            }
        }
        2 => {
            if let Some(focus_id) = focus {
                if !elements.contains_key(focus_id) {
                    return (HashMap::new(), Vec::new());
                }
                visible.insert(focus_id.to_string());
                let prefix = format!("{}.", focus_id);
                for id in elements.keys() {
                    if id.starts_with(&prefix) && is_core(id) {
                        visible.insert(id.clone());
                    }
                }
            } else {
                for id in elements.keys() {
                    if is_core(id) {
                        visible.insert(id.clone());
                    }
                }
            }
        }
        3 => {
            if let Some(focus_id) = focus {
                if !elements.contains_key(focus_id) {
                    return (HashMap::new(), Vec::new());
                }
                visible.insert(focus_id.to_string());
                let prefix = format!("{}.", focus_id);
                for id in elements.keys() {
                    if id.starts_with(&prefix) {
                        visible.insert(id.clone());
                    }
                }
            } else {
                for id in elements.keys() {
                    visible.insert(id.clone());
                }
            }
        }
        _ => {}
    }

    let mut projected: Vec<Relation> = Vec::new();
    for rel in relations {
        let from = rel.from.as_string();
        let to = rel.to.as_string();
        let source = project_id(&from, level, focus, elements);
        let target = project_id(&to, level, focus, elements);
        if source.is_empty() || target.is_empty() || source == target {
            continue;
        }
        if source.starts_with(&(target.clone() + ".")) || target.starts_with(&(source.clone() + ".")) {
            continue;
        }
        if !visible.contains(&source) && !visible.contains(&target) {
            continue;
        }
        visible.insert(source.clone());
        visible.insert(target.clone());

        projected.push(Relation {
            location: rel.location.clone(),
            from: sruja_language::QualifiedIdent::qualified(source.split('.').map(|s| s.to_string()).collect()),
            to: sruja_language::QualifiedIdent::qualified(target.split('.').map(|s| s.to_string()).collect()),
            label: rel.label.clone(),
            description: rel.description.clone(),
            technology: rel.technology.clone(),
            tags: rel.tags.clone(),
        });
    }

    let mut out_elems: HashMap<String, sruja_language::ElementDef> = HashMap::new();
    for id in visible {
        if let Some(e) = elements.get(&id) {
            out_elems.insert(id, e.clone());
        }
    }

    (out_elems, projected)
}

fn normalize_kind(kind: &str) -> &str {
    match kind.to_lowercase().as_str() {
        "database" | "db" | "storage" => "datastore",
        "mq" => "queue",
        "actor" => "person",
        other => other,
    }
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
    get_root(fqn)
}

fn get_root(fqn: &str) -> String {
    fqn.split('.').next().unwrap_or(fqn).to_string()
}

fn get_container(
    fqn: &str,
    elements: &HashMap<String, sruja_language::ElementDef>,
) -> Option<String> {
    let mut cur = fqn.to_string();
    loop {
        if let Some(e) = elements.get(&cur) {
            if normalize_kind(&e.assignment.kind.to_string()) == "container" {
                return Some(cur);
            }
        }
        let Some(p) = parent_fqn(&cur) else { return None };
        cur = p;
    }
}

fn parent_fqn(fqn: &str) -> Option<String> {
    let idx = fqn.rfind('.')?;
    Some(fqn[..idx].to_string())
}

fn escape_id(id: &str) -> String {
    id.replace('"', "\\\"")
}

fn escape_quotes(s: &str) -> String {
    s.replace('"', "\\\"")
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

fn build_html_label(elem: &sruja_language::ElementDef) -> String {
    let title = elem
        .assignment
        .title
        .clone()
        .unwrap_or_else(|| elem.assignment.name.clone());
    let (desc, tech) = if let Some(body) = &elem.assignment.body {
        (
            body.description.clone().unwrap_or_default(),
            body.technology.clone().unwrap_or_default(),
        )
    } else {
        (String::new(), String::new())
    };

    let mut parts = vec![format!("<b>{}</b>", escape_html(&title))];
    if !desc.is_empty() {
        parts.push(format!("<br/>{}", escape_html(&desc)));
    }
    if !tech.is_empty() {
        parts.push(format!("<br/><i>{}</i>", escape_html(&tech)));
    }
    parts.join("")
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn get_node_size(elem: &sruja_language::ElementDef) -> (f64, f64) {
    let kind = normalize_kind(&elem.assignment.kind.to_string());
    match kind {
        "person" => (MIN_WIDTH_PERSON, MIN_HEIGHT_PERSON),
        "system" => (MIN_WIDTH_SYSTEM, MIN_HEIGHT_SYSTEM),
        "container" => (MIN_WIDTH_CONTAINER, MIN_HEIGHT_CONTAINER),
        "component" => (MIN_WIDTH_COMPONENT, MIN_HEIGHT_COMPONENT),
        "datastore" | "queue" => (MIN_WIDTH_INFRASTRUCTURE, MIN_HEIGHT_INFRASTRUCTURE),
        _ => (MIN_WIDTH_SYSTEM, MIN_HEIGHT_SYSTEM),
    }
}
