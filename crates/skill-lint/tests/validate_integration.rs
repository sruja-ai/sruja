//! Integration tests for the validate command: schema + skill files on disk.

use std::path::PathBuf;

fn schema_content() -> String {
    std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("skill-schema.json"),
    )
    .unwrap()
}

#[tokio::test]
async fn validate_run_all_valid_files() {
    let dir = tempfile::tempdir().unwrap();
    let schema_path = dir.path().join("schema.json");
    let skills_dir = dir.path().join("skills");
    std::fs::create_dir_all(&skills_dir).unwrap();

    std::fs::write(&schema_path, schema_content()).unwrap();
    let valid_md = r#"---
metadata:
  complexity: 2
  frequency: common
  confidence: high
  category: medium
  level: intermediate
---
# Rule
"#;
    std::fs::write(skills_dir.join("rule1.md"), valid_md).unwrap();
    std::fs::write(skills_dir.join("rule2.md"), valid_md).unwrap();

    let result = skill_lint::commands::validate::run(schema_path, skills_dir).await;
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
}

#[tokio::test]
async fn validate_run_with_invalid_file_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let schema_path = dir.path().join("schema.json");
    let skills_dir = dir.path().join("skills");
    std::fs::create_dir_all(&skills_dir).unwrap();

    std::fs::write(&schema_path, schema_content()).unwrap();
    std::fs::write(skills_dir.join("bad.md"), "---\nno_metadata_key: true\n---\n").unwrap();

    let result = skill_lint::commands::validate::run(schema_path, skills_dir).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn validate_run_skips_non_md_files() {
    let dir = tempfile::tempdir().unwrap();
    let schema_path = dir.path().join("schema.json");
    let skills_dir = dir.path().join("skills");
    std::fs::create_dir_all(&skills_dir).unwrap();

    std::fs::write(&schema_path, schema_content()).unwrap();
    let valid_md = r#"---
metadata:
  complexity: 1
  frequency: rare
  confidence: low
  category: low
  level: beginner
---
# Only valid file
"#;
    std::fs::write(skills_dir.join("rule.md"), valid_md).unwrap();
    std::fs::write(skills_dir.join("readme.txt"), "not markdown").unwrap();

    let result = skill_lint::commands::validate::run(schema_path, skills_dir).await;
    assert!(result.is_ok());
}
