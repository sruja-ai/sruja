use super::CliError;

pub(super) fn split_escaped(input: &str, sep: char) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut escape = false;
    for ch in input.chars() {
        if escape {
            cur.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        if ch == sep {
            parts.push(cur);
            cur = String::new();
            continue;
        }
        cur.push(ch);
    }
    if escape {
        cur.push('\\');
    }
    parts.push(cur);
    parts
}

pub(super) fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

pub(super) fn is_valid_qualified_ident(s: &str) -> bool {
    if s.trim().is_empty() {
        return false;
    }
    s.split('.').all(is_valid_identifier)
}

pub(super) fn root_ident(s: &str) -> &str {
    s.split('.').next().unwrap_or(s)
}

pub(super) fn is_under_root(fqn: &str, root: &str) -> bool {
    fqn == root || fqn.starts_with(&format!("{}.", root))
}

pub(super) fn kind_requires_description_str(kind_lc: &str) -> bool {
    matches!(
        kind_lc,
        "container" | "component" | "database" | "datastore" | "data_store"
    )
}

pub(super) fn kind_requires_technology_str(kind_lc: &str) -> bool {
    matches!(
        kind_lc,
        "container" | "database" | "datastore" | "data_store"
    )
}

pub(super) fn parse_add_element_spec(
    spec: &str,
) -> Result<(String, String, String, Option<String>), CliError> {
    let parts = split_escaped(spec, ':');
    if parts.len() < 3 || parts.len() > 4 {
        return Err(CliError::validation(format!(
            "Invalid element spec '{}'. Expected id:kind:label[:tech]. Use \\: to escape ':' in fields.",
            spec
        )));
    }
    let id = parts[0].trim().to_string();
    let kind = parts[1].trim().to_string();
    let label = parts[2].trim().to_string();
    if id.is_empty() || kind.is_empty() || label.is_empty() {
        return Err(CliError::validation(format!(
            "Invalid element spec '{}'. id, kind, and label must be non-empty.",
            spec
        )));
    }
    if !is_valid_identifier(&id) {
        return Err(CliError::validation(format!(
            "Invalid element spec '{}'. id '{}' must be an identifier (letters, digits, _, -).",
            spec, id
        )));
    }
    if !is_valid_identifier(&kind) {
        return Err(CliError::validation(format!(
            "Invalid element spec '{}'. kind '{}' must be an identifier (letters, digits, _, -).",
            spec, kind
        )));
    }
    let tech = parts
        .get(3)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Ok((id, kind, label, tech))
}

pub(super) fn parse_add_relationship_spec(spec: &str) -> Result<(String, String, Option<String>), CliError> {
    let (left, right) = spec.split_once("->").ok_or_else(|| {
        CliError::validation(format!(
            "Invalid relationship spec '{}'. Expected source->target[:label].",
            spec
        ))
    })?;
    let source = left.trim().to_string();
    let rest = right.trim();
    if source.is_empty() || rest.is_empty() {
        return Err(CliError::validation(format!(
            "Invalid relationship spec '{}'. Expected source->target[:label].",
            spec
        )));
    }
    if !is_valid_qualified_ident(&source) {
        return Err(CliError::validation(format!(
            "Invalid relationship spec '{}'. source '{}' must be a qualified identifier (e.g., A or System.Container).",
            spec, source
        )));
    }
    let parts = split_escaped(rest, ':');
    if parts.len() > 2 {
        return Err(CliError::validation(format!(
            "Invalid relationship spec '{}'. Expected source->target[:label]. Use \\: to escape ':' in label.",
            spec
        )));
    }
    let target = parts[0].trim().to_string();
    if target.is_empty() {
        return Err(CliError::validation(format!(
            "Invalid relationship spec '{}'. Target must be non-empty.",
            spec
        )));
    }
    if !is_valid_qualified_ident(&target) {
        return Err(CliError::validation(format!(
            "Invalid relationship spec '{}'. target '{}' must be a qualified identifier (e.g., B or System.Container).",
            spec, target
        )));
    }
    let label = parts
        .get(1)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Ok((source, target, label))
}

pub(super) fn parse_remove_relationship_spec(spec: &str) -> Result<(String, String), CliError> {
    let (left, right) = spec.split_once("->").ok_or_else(|| {
        CliError::validation(format!(
            "Invalid relationship spec '{}'. Expected source->target.",
            spec
        ))
    })?;
    let source = left.trim().to_string();
    let target = right.trim().to_string();
    if source.is_empty() || target.is_empty() {
        return Err(CliError::validation(format!(
            "Invalid relationship spec '{}'. Expected source->target.",
            spec
        )));
    }
    if !is_valid_qualified_ident(&source) {
        return Err(CliError::validation(format!(
            "Invalid relationship spec '{}'. source '{}' must be a qualified identifier (e.g., A or System.Container).",
            spec, source
        )));
    }
    if !is_valid_qualified_ident(&target) {
        return Err(CliError::validation(format!(
            "Invalid relationship spec '{}'. target '{}' must be a qualified identifier (e.g., B or System.Container).",
            spec, target
        )));
    }
    Ok((source, target))
}
