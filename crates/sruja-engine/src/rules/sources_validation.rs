use crate::DomainSchema;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
use sruja_language::{collect_elements, Program, SourceBinding, SourceKind};

use crate::validator::Rule;

pub struct SourcesValidationRule;

impl Rule for SourcesValidationRule {
    fn name(&self) -> &str {
        "Sources Validation"
    }

    fn validate(&self, program: &Program, _schema: &DomainSchema) -> Vec<Diagnostic> {
        if program.items.is_empty() {
            return vec![];
        }

        let (elements, _relations) = collect_elements(program);
        let mut diags: Vec<Diagnostic> = Vec::with_capacity(16);

        for (fqn, elem) in &elements {
            let Some(body) = &elem.assignment.body else {
                continue;
            };
            if body.sources.is_empty() {
                continue;
            }

            validate_element_sources(fqn, &elem.location, &body.sources, &mut diags);
        }

        diags
    }
}

fn validate_element_sources(
    element_fqn: &str,
    element_loc: &SourceLocation,
    sources: &[SourceBinding],
    diags: &mut Vec<Diagnostic>,
) {
    let mut seen_exact: HashSet<(SourceKind, String)> = HashSet::new();
    let mut path_to_kinds: HashMap<String, HashSet<SourceKind>> = HashMap::new();

    for s in sources {
        let normalized_path = s.path.trim().to_string();

        if normalized_path.is_empty() {
            diags.push(
                Diagnostic::new(
                    sruja_diagnostics::codes::CODE_INVALID_PROPERTY,
                    Severity::Error,
                    format!(
                        "Element '{}' has a '{}' source with an empty path",
                        element_fqn,
                        s.kind.as_str()
                    ),
                    element_loc.clone(),
                )
                .with_suggestions(vec![
                    "Provide a non-empty path (typically repo-relative)".to_string(),
                    format!("Example: source {} \"./path/to/file\"", s.kind.as_str()),
                ]),
            );
            continue;
        }

        let exact_key = (s.kind.clone(), normalized_path.clone());
        if !seen_exact.insert(exact_key) {
            diags.push(
                Diagnostic::new(
                    sruja_diagnostics::codes::CODE_INVALID_PROPERTY,
                    Severity::Error,
                    format!(
                        "Element '{}' has a duplicate '{}' source binding for path '{}'",
                        element_fqn,
                        s.kind.as_str(),
                        normalized_path
                    ),
                    element_loc.clone(),
                )
                .with_suggestions(vec![
                    "Remove the duplicate 'source' line".to_string(),
                    "If multiple references are intended, use a single entry and describe the intent in documentation".to_string(),
                ]),
            );
        }

        path_to_kinds
            .entry(normalized_path.clone())
            .or_default()
            .insert(s.kind.clone());

        if Path::new(&normalized_path).is_absolute() {
            diags.push(
                Diagnostic::new(
                    sruja_diagnostics::codes::CODE_BEST_PRACTICE,
                    Severity::Warning,
                    format!(
                        "Element '{}' uses an absolute '{}' source path '{}'",
                        element_fqn,
                        s.kind.as_str(),
                        normalized_path
                    ),
                    element_loc.clone(),
                )
                .with_suggestions(vec![
                    "Prefer repo-relative paths so the architecture file is portable".to_string(),
                    "If this must be absolute, ensure it exists in all environments where lint runs".to_string(),
                ]),
            );
        }
    }

    for (path, kinds) in path_to_kinds {
        if kinds.len() <= 1 {
            continue;
        }
        let kinds_str = kinds
            .into_iter()
            .map(|k| k.as_str().to_string())
            .collect::<Vec<_>>()
            .join(", ");

        diags.push(
            Diagnostic::new(
                sruja_diagnostics::codes::CODE_INVALID_PROPERTY,
                Severity::Error,
                format!(
                    "Element '{}' binds the same path '{}' to multiple source kinds ({})",
                    element_fqn, path, kinds_str
                ),
                element_loc.clone(),
            )
            .with_suggestions(vec![
                "Use a single source kind per file path to keep the intent unambiguous".to_string(),
                "If the file is intentionally multi-purpose, keep one binding and capture the nuance in documentation".to_string(),
            ]),
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::DomainSchema;
    use super::*;
    use sruja_language::Parser;

    fn validate_program(file: &str, input: &str) -> Vec<Diagnostic> {
        let parser = Parser::new(file.to_string());
        let program = parser.parse(input).expect("parse failed");
        let rule = SourcesValidationRule;
        rule.validate(&program, &DomainSchema::architecture())
    }

    #[test]
    fn detects_duplicate_source_binding() {
        let input = r#"
API = container "API" {
  source openapi "./specs/api.yaml"
  source openapi "./specs/api.yaml"
}
"#;
        let diags = validate_program("/tmp/test.sruja", input);
        assert!(diags.iter().any(|d| d.code == "E301"));
        assert!(diags.iter().any(|d| d.message.contains("duplicate")));
    }

    #[test]
    fn detects_empty_source_path() {
        let input = r#"
API = container "API" {
  source openapi ""
}
"#;
        let diags = validate_program("/tmp/test.sruja", input);
        assert!(diags.iter().any(|d| d.code == "E301"));
        assert!(diags.iter().any(|d| d.message.contains("empty path")));
    }
}
