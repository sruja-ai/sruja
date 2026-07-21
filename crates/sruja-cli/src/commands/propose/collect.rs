use super::CliError;
use std::collections::{HashMap, HashSet};

pub(super) fn collect_program_top_level_elements(program: &sruja_language::ast::Program) -> HashSet<String> {
    use sruja_language::ast::TopLevelItem;
    let mut out: HashSet<String> = HashSet::new();
    for item in &program.items {
        if let TopLevelItem::ElementDef(def) = item {
            out.insert(def.assignment.name.clone());
        }
    }
    out
}

pub(super) fn collect_program_element_fqns(program: &sruja_language::ast::Program) -> HashSet<String> {
    let (elements, _relations) = sruja_language::collect_elements(program);
    elements.keys().cloned().collect()
}

pub(super) fn collect_program_relationships(
    program: &sruja_language::ast::Program,
) -> HashSet<(String, String)> {
    let mut out: HashSet<(String, String)> = HashSet::new();
    for rel in sruja_language::collect_all_relations(program) {
        out.insert((rel.from.as_string(), rel.to.as_string()));
    }
    out
}

pub(super) fn collect_program_defined_kinds(
    program: &sruja_language::ast::Program,
) -> HashMap<String, String> {
    use sruja_language::ast::TopLevelItem;
    let mut out: HashMap<String, String> = HashMap::new();
    for item in &program.items {
        if let TopLevelItem::KindDef(def) = item {
            let canonical = format!("{}", def.kind);
            out.insert(canonical.to_lowercase(), canonical);
        }
    }
    out
}

pub(super) fn normalize_element_kind(
    kind: &str,
    has_baseline: bool,
    baseline_defined_kinds: &HashMap<String, String>,
) -> Result<String, CliError> {
    let kind = kind.trim();
    if kind.is_empty() {
        return Err(CliError::validation(
            "Element kind must be non-empty".to_string(),
        ));
    }

    let lc = kind.to_lowercase();
    let canonical = match lc.as_str() {
        "person" => Some("person"),
        "role" => Some("role"),
        "system" => Some("system"),
        "container" => Some("container"),
        "component" => Some("component"),
        "database" => Some("database"),
        "queue" => Some("queue"),
        "policy" => Some("policy"),
        "requirement" => Some("requirement"),
        "adr" => Some("adr"),
        "flow" => Some("flow"),
        "scenario" => Some("scenario"),
        "story" => Some("story"),
        "datastore" | "data_store" => Some("datastore"),
        "externalsystem" | "external_system" => Some("externalSystem"),
        _ => None,
    };

    if let Some(canonical) = canonical {
        return Ok(canonical.to_string());
    }

    if has_baseline {
        if let Some(canonical) = baseline_defined_kinds.get(&lc) {
            return Ok(canonical.clone());
        }
        return Err(CliError::validation(format!(
            "Unknown element kind '{}'. Define it first in repo.sruja with '<KindId> = kind \"...\" {{ ... }}'.",
            kind
        )));
    }

    Err(CliError::validation(format!(
        "Unknown element kind '{}'. Allowed built-in kinds: person, role, system, container, component, database, queue, policy, requirement, adr, flow, scenario, story, datastore, externalSystem.",
        kind
    )))
}
