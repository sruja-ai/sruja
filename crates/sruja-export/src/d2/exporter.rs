use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use sruja_language::{collect_elements, Program, Relation};

use super::constants::*;

#[derive(Debug, Clone)]
pub struct D2Config {
    pub direction: String,
    pub view_level: u8,
    pub target_id: Option<String>,
    pub link_template: Option<String>,
}

impl Default for D2Config {
    fn default() -> Self {
        Self {
            direction: "right".to_string(),
            view_level: 3,
            target_id: None,
            link_template: None,
        }
    }
}

pub struct D2Exporter {
    pub config: D2Config,
}

impl D2Exporter {
    pub fn new(config: D2Config) -> Self {
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
        let (view_elements, view_relations) = compute_view(
            elements,
            relations,
            self.config.view_level,
            self.config.target_id.clone(),
        );

        let mut out = String::new();
        out.push_str(&format!("direction: {}\n\n", self.config.direction));

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
            self.write_element(&mut out, &root, &view_elements, &children, "");
        }

        out.push('\n');
        for rel in &view_relations {
            self.write_relation(&mut out, rel);
        }

        out
    }

    fn write_element(
        &self,
        out: &mut String,
        fqn: &str,
        elements: &HashMap<String, sruja_language::ElementDef>,
        children: &BTreeMap<String, BTreeSet<String>>,
        indent: &str,
    ) {
        let Some(elem) = elements.get(fqn) else {
            return;
        };

        let id = sanitize_id(fqn);
        let title = elem
            .assignment
            .title
            .as_deref()
            .unwrap_or(&elem.assignment.name);
        let kind = normalize_kind(&elem.assignment.kind.to_string());

        out.push_str(&format!("{indent}{id}: \"{title}\" {{\n"));

        let (shape, fill) = match kind.as_str() {
            "person" => (SHAPE_PERSON, COLOR_PERSON_FILL),
            "system" | "container" => (SHAPE_PACKAGE, COLOR_SYSTEM_FILL),
            "database" | "datastore" => (SHAPE_CYLINDER, COLOR_DATABASE_FILL),
            "queue" => (SHAPE_QUEUE, COLOR_QUEUE_FILL),
            "component" => (SHAPE_RECTANGLE, COLOR_COMPONENT_FILL),
            "external" | "externalsystem" => (SHAPE_PACKAGE, COLOR_EXTERNAL_FILL),
            _ => (SHAPE_RECTANGLE, "#ffffff"),
        };

        out.push_str(&format!("{indent}{INDENT2}shape: {shape}\n"));
        out.push_str(&format!("{indent}{INDENT2}style.fill: \"{fill}\"\n"));
        out.push_str(&format!(
            "{indent}{INDENT2}style.stroke: \"{STROKE_COLOR}\"\n"
        ));

        if let Some(body) = &elem.assignment.body {
            if let Some(desc) = &body.description {
                let escaped = desc.replace('"', "\\\"");
                out.push_str(&format!("{indent}{INDENT2}tooltip: \"{escaped}\"\n"));
            }
        }

        let loc = &elem.location;
        if !loc.file.is_empty() && loc.line > 0 && loc.column > 0 {
            let template = self
                .config
                .link_template
                .as_deref()
                .unwrap_or("vscode://file/{file}:{line}:{column}");
            let link = template
                .replace("{file}", &loc.file)
                .replace("{line}", &loc.line.to_string())
                .replace("{column}", &loc.column.to_string());
            out.push_str(&format!("{indent}{INDENT2}link: \"{link}\"\n"));
        }

        if let Some(kids) = children.get(fqn) {
            out.push('\n');
            for child in kids {
                self.write_element(
                    out,
                    child,
                    elements,
                    children,
                    &format!("{indent}{INDENT2}"),
                );
            }
        }

        out.push_str(&format!("{indent}}}\n"));
    }

    fn write_relation(&self, out: &mut String, rel: &Relation) {
        let from = sanitize_id(&rel.from.as_string());
        let to = sanitize_id(&rel.to.as_string());
        let label = rel.label.as_ref().or(rel.description.as_ref());

        let mut base = if let Some(l) = label {
            let one_line = l.lines().next().unwrap_or("").replace('"', "\\\"");
            format!("{from} -> {to}: \"{one_line}\"")
        } else {
            format!("{from} -> {to}")
        };

        let loc = &rel.location;
        if !loc.file.is_empty() && loc.line > 0 && loc.column > 0 {
            let template = self
                .config
                .link_template
                .as_deref()
                .unwrap_or("vscode://file/{file}:{line}:{column}");
            let link = template
                .replace("{file}", &loc.file)
                .replace("{line}", &loc.line.to_string())
                .replace("{column}", &loc.column.to_string());

            base.push_str(&format!(" {{\n{INDENT2}link: \"{link}\"\n}}"));
        }

        out.push_str(&base);
        out.push('\n');
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

fn sanitize_id(id: &str) -> String {
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

fn parent_fqn(fqn: &str) -> Option<String> {
    let idx = fqn.rfind('.')?;
    Some(fqn[..idx].to_string())
}

fn compute_view(
    elements: &HashMap<String, sruja_language::ElementDef>,
    relations: &[Relation],
    mut level: u8,
    mut focus: Option<String>,
) -> (HashMap<String, sruja_language::ElementDef>, Vec<Relation>) {
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
                if id == focus.as_deref().unwrap_or("") {
                    return true;
                }
                if let Some(e) = elements.get(id) {
                    let k = normalize_kind(&e.assignment.kind.to_string());
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

    let mut out_elems = HashMap::new();
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
        let p = parent_fqn(&cur)?;
        cur = p;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_language::Parser;

    #[test]
    fn test_d2_export_empty_program() {
        let exporter = D2Exporter::new(D2Config::default());
        let program = Program { items: vec![] };
        let out = exporter.export(&program);
        assert!(out.is_empty());
    }

    #[test]
    fn test_d2_export_basic_system() {
        let input = r#"
        system = kind "System"
        container = kind "Container"
        
        App = system "App" {
            description "My App"
            Web = container "Web" {
                description "Web UI"
            }
            Api = container "API" {
                description "Backend"
            }
        }
        
        App.Web -> App.Api "calls"
        "#;

        let parser = Parser::new("test.sruja");
        let program = parser.parse(input).unwrap();

        let config = D2Config {
            direction: "down".to_string(),
            view_level: 2,
            target_id: None,
            link_template: None,
        };

        let exporter = D2Exporter::new(config);
        let out = exporter.export(&program);

        assert!(out.contains("direction: down"));
        assert!(out.contains("App: \"App\" {"));
        assert!(out.contains("Web: \"Web\" {"));
        assert!(out.contains("Api: \"API\" {"));
        assert!(out.contains("App_Web -> App_Api: \"calls\""));
        assert!(out.contains("shape: package"));
    }
}
