use crate::error::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct RuleMetadata {
    pub complexity: u8,
    pub frequency: String,
    pub confidence: String,
    pub category: String,
    pub level: String,
    #[serde(default)]
    pub applicable: Option<Applicable>,
    #[serde(default)]
    pub rust_version: Option<String>,
    #[serde(default)]
    pub alternatives: Option<Vec<String>>,
    #[serde(default)]
    pub related_rules: Option<Vec<String>>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Applicable {
    #[serde(default, rename = "async")]
    pub is_async: bool,
    #[serde(default)]
    pub embedded: bool,
    #[serde(default)]
    pub wasm: bool,
    #[serde(default)]
    pub web: bool,
    #[serde(default)]
    pub cli: bool,
    #[serde(default)]
    pub library: bool,
}

#[derive(Debug)]
pub struct Rule {
    pub id: String,
    pub metadata: RuleMetadata,
    pub content: String,
}

#[derive(Debug)]
pub struct Diagnostic {
    pub level: Level,
    pub message: String,
    pub line: Option<usize>,
}

#[derive(Debug, PartialEq)]
pub enum Level {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Error => write!(f, "ERROR"),
            Level::Warning => write!(f, "WARNING"),
            Level::Info => write!(f, "INFO"),
        }
    }
}

#[derive(Debug)]
pub struct SkillChecker {
    pub metadata_schema: serde_json::Value,
}

impl SkillChecker {
    pub fn new(schema_content: &str) -> Result<Self> {
        let metadata_schema: serde_json::Value = serde_json::from_str(schema_content)?;
        Ok(Self { metadata_schema })
    }

    pub fn check_file(&self, path: &Path) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();

        let content = std::fs::read_to_string(path)?;

        if let Some(frontmatter) = self.extract_frontmatter(&content) {
            diagnostics.extend(self.check_metadata(&frontmatter, path));
        } else {
            diagnostics.push(Diagnostic {
                level: Level::Error,
                message: "Missing metadata frontmatter".to_string(),
                line: Some(1),
            });
        }

        Ok(diagnostics)
    }

    fn extract_frontmatter(&self, content: &str) -> Option<String> {
        if !content.starts_with("---") {
            return None;
        }

        content
            .lines()
            .skip(1)
            .take_while(|line| *line != "---")
            .collect::<Vec<_>>()
            .join("\n")
            .into()
    }

    fn check_metadata(&self, frontmatter: &str, _path: &Path) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        if let Ok(metadata) = serde_yaml::from_str::<serde_yaml::Value>(frontmatter) {
            // Validate the entire frontmatter object (which includes 'metadata' key)
            // against the schema, not just the inner metadata
            if let Err(e) = self.validate_schema(&metadata) {
                diagnostics.push(Diagnostic {
                    level: Level::Error,
                    message: format!("Schema validation failed: {}", e),
                    line: Some(1),
                });
            }

            // Also check that metadata key exists
            if metadata.get("metadata").is_none() {
                diagnostics.push(Diagnostic {
                    level: Level::Error,
                    message: "Missing 'metadata' key in frontmatter".to_string(),
                    line: Some(1),
                });
            }
        } else {
            diagnostics.push(Diagnostic {
                level: Level::Error,
                message: "Invalid YAML frontmatter".to_string(),
                line: Some(1),
            });
        }

        diagnostics
    }

    fn validate_schema(&self, metadata: &serde_yaml::Value) -> Result<()> {
        let metadata_json = serde_json::to_value(metadata)?;
        let schema = jsonschema::Validator::new(&self.metadata_schema)
            .map_err(|e| crate::error::SkillLintError::SchemaValidation(e.to_string()))?;
        let result = schema.validate(&metadata_json);

        if let Err(errors) = result {
            let error_messages: Vec<String> = errors.map(|e| e.to_string()).collect();
            return Err(crate::error::SkillLintError::SchemaValidation(
                error_messages.join("; "),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_schema() -> &'static str {
        include_str!("../skill-schema.json")
    }

    #[test]
    fn checker_new_accepts_valid_schema() {
        let schema = minimal_schema();
        let checker = SkillChecker::new(schema).unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(schema).is_ok());
        let _ = checker;
    }

    #[test]
    fn checker_new_rejects_invalid_schema() {
        let err = SkillChecker::new("not json").unwrap_err();
        assert!(matches!(err, crate::error::SkillLintError::JsonParse(_)));
    }

    #[test]
    fn check_file_missing_frontmatter() {
        let checker = SkillChecker::new(minimal_schema()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("no_frontmatter.md");
        std::fs::write(&path, "No frontmatter here\n").unwrap();

        let diags = checker.check_file(&path).unwrap();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].level, Level::Error);
        assert!(diags[0].message.contains("Missing metadata"));
    }

    #[test]
    fn check_file_invalid_yaml_frontmatter() {
        let checker = SkillChecker::new(minimal_schema()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad_yaml.md");
        std::fs::write(&path, "---\nfoo: [unclosed\n---\n\n# Content\n").unwrap();

        let diags = checker.check_file(&path).unwrap();
        assert!(!diags.is_empty());
        assert!(diags
            .iter()
            .any(|d| d.message.contains("YAML") || d.message.contains("metadata")));
    }

    #[test]
    fn check_file_missing_metadata_key() {
        let checker = SkillChecker::new(minimal_schema()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("no_metadata_key.md");
        std::fs::write(&path, "---\nfoo: bar\n---\n\n# Content\n").unwrap();

        let diags = checker.check_file(&path).unwrap();
        assert!(diags
            .iter()
            .any(|d| d.message.contains("Missing 'metadata'")));
    }

    #[test]
    fn check_file_valid_metadata_passes() {
        let checker = SkillChecker::new(minimal_schema()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("valid.md");
        let content = r#"---
metadata:
  complexity: 2
  frequency: common
  confidence: high
  category: medium
  level: intermediate
---
# Rule
"#;
        std::fs::write(&path, content).unwrap();

        let diags = checker.check_file(&path).unwrap();
        assert!(
            diags.is_empty(),
            "expected no diagnostics, got: {:?}",
            diags
        );
    }

    #[test]
    fn check_file_schema_validation_fails() {
        let checker = SkillChecker::new(minimal_schema()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("invalid_schema.md");
        let content = r#"---
metadata:
  complexity: 99
  frequency: common
  confidence: high
  category: medium
  level: intermediate
---
# Rule
"#;
        std::fs::write(&path, content).unwrap();

        let diags = checker.check_file(&path).unwrap();
        assert!(!diags.is_empty());
        assert!(diags
            .iter()
            .any(|d| d.message.contains("Schema validation") || d.message.contains("validation")));
    }

    #[test]
    fn level_display() {
        assert_eq!(Level::Error.to_string(), "ERROR");
        assert_eq!(Level::Warning.to_string(), "WARNING");
        assert_eq!(Level::Info.to_string(), "INFO");
    }
}
