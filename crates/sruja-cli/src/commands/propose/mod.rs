use super::CliError;
use std::path::Path;

pub mod types;
mod spec;
mod collect;
mod create;
mod list;
mod approve;

pub use types::ProposeCreateRequest;
pub use create::propose_create;
pub use list::propose_list;
pub use approve::propose_approve;

pub(super) fn derive_title(description: &str) -> String {
    let first_line = description.lines().next().unwrap_or("").trim();
    if first_line.is_empty() {
        return "Architecture Change".to_string();
    }
    let mut out = first_line.to_string();
    if out.len() > 64 {
        out.truncate(64);
    }
    out
}

fn new_short_id() -> String {
    let id = uuid::Uuid::new_v4().simple().to_string();
    id.chars().take(12).collect()
}

pub(super) fn new_short_id_unique(repo_path: &Path) -> Result<String, CliError> {
    let proposals_dir = repo_path.join(".sruja").join("proposals");
    for _ in 0..10 {
        let id = new_short_id();
        let candidate = proposals_dir.join(format!("{}.json", id));
        if !candidate.exists() {
            return Ok(id);
        }
    }
    Err(CliError::validation(
        "Failed to generate a unique proposal id after multiple attempts.".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::spec::{
        parse_add_element_spec, parse_add_relationship_spec,
        is_valid_identifier, is_valid_qualified_ident,
    };

    #[test]
    fn element_spec_parses_with_escaped_colon() {
        let (id, kind, label, tech) =
            parse_add_element_spec(r#"A:system:Payments\: Core:Go"#).unwrap();
        assert_eq!(id, "A");
        assert_eq!(kind, "system");
        assert_eq!(label, "Payments: Core");
        assert_eq!(tech.as_deref(), Some("Go"));
    }

    #[test]
    fn relationship_spec_parses_with_label() {
        let (s, t, l) = parse_add_relationship_spec("A -> B:uses api").unwrap();
        assert_eq!(s, "A");
        assert_eq!(t, "B");
        assert_eq!(l.as_deref(), Some("uses api"));
    }

    #[test]
    fn identifier_validation_matches_dsl_identifier_shape() {
        assert!(is_valid_identifier("A"));
        assert!(is_valid_identifier("A_b"));
        assert!(is_valid_identifier("a-b"));
        assert!(!is_valid_identifier("1A"));
        assert!(!is_valid_identifier("A.B"));
        assert!(!is_valid_identifier(""));
    }

    #[test]
    fn qualified_identifier_validation_accepts_dot_paths() {
        assert!(is_valid_qualified_ident("A"));
        assert!(is_valid_qualified_ident("System.Container"));
        assert!(is_valid_qualified_ident("A_b.C-d"));
        assert!(!is_valid_qualified_ident("A..B"));
        assert!(!is_valid_qualified_ident(".A"));
        assert!(!is_valid_qualified_ident("A."));
        assert!(!is_valid_qualified_ident(""));
    }
}
