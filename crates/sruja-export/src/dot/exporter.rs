//! DOT exporter implementation (initial parity pass)
//!
//! Ports Go dot exporter structure. Uses the same view-level filtering
//! approach as Mermaid for consistency.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use sruja_language::{collect_elements, Program, Relation};

use super::constants::*;

#[derive(Debug, Clone)]
pub struct DotConfig {
    pub rank_dir: String, // "TB" or "LR"
    pub node_sep: f64,
    pub rank_sep: f64,
    pub view_level: u8,
    pub target_id: Option<String>,
    pub node_sizes: std::collections::HashMap<String, (f64, f64)>, // width, height for specific nodes
    pub view_id: Option<String>,                                   // Optional view definition ID
    pub filename: Option<String>, // Optional filename for error reporting
}

impl Default for DotConfig {
    fn default() -> Self {
        Self {
            rank_dir: "TB".to_string(),
            node_sep: DEFAULT_NODE_SEP,
            rank_sep: DEFAULT_RANK_SEP,
            view_level: 1,
            target_id: None,
            node_sizes: std::collections::HashMap::new(),
            view_id: None,
            filename: None,
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
            program,
            &elements,
            &relations,
            self.config.view_level,
            self.config.target_id.clone(),
            self.config.view_id.clone(),
        );

        self.generate(&view_elements, &view_relations)
    }

    /// Export DOT format along with elements and relations for debugging/testing
    pub fn export_with_relations(
        &self,
        program: &Program,
    ) -> (
        String,
        HashMap<String, sruja_language::ElementDef>,
        Vec<Relation>,
    ) {
        if program.items.is_empty() {
            return (String::new(), HashMap::new(), Vec::new());
        }

        let (elements, relations) = collect_elements(program);
        if elements.is_empty() {
            return (String::new(), HashMap::new(), Vec::new());
        }

        // Apply view-level filtering (reuse Mermaid logic)
        let (view_elements, view_relations) = compute_view(
            program,
            &elements,
            &relations,
            self.config.view_level,
            self.config.target_id.clone(),
            self.config.view_id.clone(),
        );

        let dot_output = self.generate(&view_elements, &view_relations);

        (dot_output, view_elements, view_relations)
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
        out.push_str(&format!(
            "    nodesep={:.2},\n",
            self.config.node_sep / 72.0
        ));
        out.push_str(&format!(
            "    ranksep={:.2},\n",
            self.config.rank_sep / 72.0
        ));
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
        out.push_str("    style=\"rounded,filled\"\n");
        out.push_str("  ];\n\n");

        out.push_str("  edge [\n");
        out.push_str(&format!("    fontname=\"{}\",\n", FONT_NAME));
        out.push_str(&format!("    fontsize={},\n", FONT_SIZE_EDGE));
        out.push_str(&format!("    fontcolor=\"{}\",\n", COLOR_SLATE_700));
        out.push_str(&format!("    color=\"{}\",\n", COLOR_SLATE_500));
        out.push_str(&format!("    penwidth={},\n", PEN_WIDTH_EDGE));
        out.push_str(&format!("    arrowsize={:.2}\n", ARROW_SIZE));
        out.push_str("  ];\n\n");

        // Build clusters and parent map
        let mut children: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut roots: BTreeSet<String> = BTreeSet::new();
        let mut parent_map: HashMap<String, String> = HashMap::new();

        for fqn in elements.keys() {
            if let Some(parent) = parent_fqn(fqn) {
                if elements.contains_key(&parent) {
                    children
                        .entry(parent.clone())
                        .or_default()
                        .insert(fqn.clone());
                    parent_map.insert(fqn.clone(), parent);
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
            self.write_edge(&mut out, rel, &parent_map);
        }

        out.push_str("}\n");
        out
    }

    fn write_node(
        &self,
        out: &mut String,
        fqn: &str,
        elements: &HashMap<String, sruja_language::ElementDef>,
    ) {
        let Some(elem) = elements.get(fqn) else {
            return;
        };
        let id = escape_id(fqn);
        let label = build_html_label(elem);
        // Use custom node size if provided, otherwise calculate default
        let (width, height) = if let Some((w, h)) = self.config.node_sizes.get(fqn) {
            (*w, *h)
        } else {
            get_node_size(elem)
        };

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
        out.push_str(&format!("  subgraph \"cluster_{}\" {{\n", id));
        out.push_str(&format!("    label=\"{}\";\n", escape_quotes(&label)));
        out.push_str("    style=filled;\n");
        out.push_str(&format!("    fillcolor=\"{}\";\n", COLOR_GRAY_BG));
        out.push_str(&format!("    fontcolor=\"{}\";\n", COLOR_SLATE_800));
        out.push_str(&format!("    fontsize={};\n", FONT_SIZE_GLOBAL + 2));

        for child in children {
            if let Some(e) = elements.get(child) {
                let child_id = escape_id(child);
                let child_label = build_html_label(e);
                // Use custom node size if provided, otherwise calculate default
                let (width, height) = if let Some((w, h)) = self.config.node_sizes.get(child) {
                    (*w, *h)
                } else {
                    get_node_size(e)
                };
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

    fn write_edge(&self, out: &mut String, rel: &Relation, parent_map: &HashMap<String, String>) {
        let from = rel.from.as_string();
        let to = rel.to.as_string();
        let from_id = escape_id(&from);
        let to_id = escape_id(&to);
        let label = rel.label.as_ref().or(rel.description.as_ref());

        // Collect edge attributes
        let mut attrs: Vec<String> = Vec::new();

        // Add label if present
        if let Some(l) = label {
            if !l.is_empty() {
                attrs.push(format!("label=\"{}\"", escape_quotes(l)));
            }
        }

        // Add lhead/ltail for cross-cluster edges
        // These attributes clip edges to cluster boundaries when compound=true
        let from_parent = parent_map.get(&from);
        let to_parent = parent_map.get(&to);

        // Only add lhead/ltail when edge crosses cluster boundaries
        // (i.e., when parents are different and both are non-empty)
        if let (Some(fp), Some(tp)) = (from_parent, to_parent) {
            if fp != tp {
                attrs.push(format!("ltail=\"cluster_{}\"", escape_id(fp)));
                attrs.push(format!("lhead=\"cluster_{}\"", escape_id(tp)));
            }
        }

        // Build edge statement
        let attrs_str = if attrs.is_empty() {
            String::new()
        } else {
            format!(" [{}]", attrs.join(", "))
        };

        out.push_str(&format!(
            "  \"{}\" -> \"{}\"{};\n",
            from_id, to_id, attrs_str
        ));
    }
}

// Reuse view computation from Mermaid (same logic)
fn compute_view(
    program: &sruja_language::Program,
    elements: &HashMap<String, sruja_language::ElementDef>,
    relations: &[Relation],
    level: u8,
    focus: Option<String>,
    view_id: Option<String>,
) -> (HashMap<String, sruja_language::ElementDef>, Vec<Relation>) {
    // Do not auto-promote L1 to L2 when there is a single system with children.
    // Callers that want container view (e.g. designer) must request level 2 with focus explicitly.
    // At L1, project_id collapses all FQNs to root so nested edges (e.g. web.api -> web.db) become self-loops and are filtered.

    let mut visible: HashSet<String> = HashSet::new();

    // If a custom view is specified, use its rules instead of level-based logic
    if let Some(ref view_id_str) = view_id {
        // Find view definition in program
        let view_def = program.items.iter().find_map(|item| {
            if let sruja_language::TopLevelItem::View(view_def) = item {
                if view_def.id == *view_id_str {
                    Some(view_def)
                } else {
                    None
                }
            } else {
                None
            }
        });

        if let Some(vd) = view_def {
            // Apply view rules
            for rule in &vd.rules {
                // Handle include rules
                if let Some(ref include_expr) = rule.include {
                    if include_expr.wildcard {
                        // include *: include all elements
                        for id in elements.keys() {
                            visible.insert(id.clone());
                        }
                    } else {
                        // include specific elements
                        for elem_id in &include_expr.elements {
                            if elements.contains_key(elem_id) {
                                visible.insert(elem_id.clone());
                            }
                        }
                    }
                }

                // Handle exclude rules
                if let Some(ref exclude_expr) = rule.exclude {
                    if exclude_expr.wildcard {
                        // exclude *: clear everything (should be rare)
                        visible.clear();
                    } else {
                        for elem_id in &exclude_expr.elements {
                            visible.remove(elem_id);
                        }
                    }
                }
            }
        }
    }

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

    match level {
        1 => {
            for id in elements.keys() {
                if is_core(id) {
                    visible.insert(id.clone());
                }
            }
        }
        2 => {
            if let Some(focus_id) = focus.as_deref() {
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
            if let Some(focus_id) = focus.as_deref() {
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

    // Expand visible to include relation endpoints that connect to visible nodes
    // (e.g. at L2 with focus=system_a, include "user" when we have user -> system_a)
    let focus_ref = focus.as_deref();
    let mut expanded = true;
    while expanded {
        expanded = false;
        for rel in relations.iter() {
            let from = rel.from.as_string();
            let to = rel.to.as_string();
            let source = project_id(&from, level, focus_ref, elements);
            let target = project_id(&to, level, focus_ref, elements);
            if source.is_empty() || target.is_empty() || source == target {
                continue;
            }
            if visible.contains(&source)
                && elements.contains_key(&target)
                && visible.insert(target.clone())
            {
                expanded = true;
            }
            if visible.contains(&target)
                && elements.contains_key(&source)
                && visible.insert(source.clone())
            {
                expanded = true;
            }
        }
    }

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
        // Only include edge if both endpoints are in the visible set
        if !visible.contains(&source) || !visible.contains(&target) {
            continue;
        }

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

    let mut out_elems: HashMap<String, sruja_language::ElementDef> = HashMap::new();
    for id in visible {
        if let Some(e) = elements.get(&id) {
            out_elems.insert(id, e.clone());
        }
    }

    (out_elems, projected)
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

fn project_id(
    fqn: &str,
    level: u8,
    focus: Option<&str>,
    elements: &HashMap<String, sruja_language::ElementDef>,
) -> String {
    // At L1 always collapse to root so we never emit full FQN (avoids web.api -> web.db at L1)
    if level == 1 {
        return get_root(fqn);
    }
    if elements.contains_key(fqn) {
        if level == 2 {
            if let Some(c) = get_container(fqn, elements) {
                return c;
            }
            return get_root(fqn);
        }
        if level == 3 {
            // Show all levels: keep full FQN so edges between siblings (e.g. API -> DB) are preserved
            if focus.is_none() {
                return fqn.to_string();
            }
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
            let kind_str = e.assignment.kind.to_string();
            if normalize_kind(&kind_str) == "container" {
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
    let kind_str = elem.assignment.kind.to_string();
    let kind = normalize_kind(&kind_str);
    match kind.as_str() {
        "person" => (MIN_WIDTH_PERSON, MIN_HEIGHT_PERSON),
        "system" => (MIN_WIDTH_SYSTEM, MIN_HEIGHT_SYSTEM),
        "container" => (MIN_WIDTH_CONTAINER, MIN_HEIGHT_CONTAINER),
        "component" => (MIN_WIDTH_COMPONENT, MIN_HEIGHT_COMPONENT),
        "datastore" | "queue" => (MIN_WIDTH_INFRASTRUCTURE, MIN_HEIGHT_INFRASTRUCTURE),
        _ => (MIN_WIDTH_SYSTEM, MIN_HEIGHT_SYSTEM),
    }
}
