mod common;
use common::*;
use sruja_extract::config::ConfigExtractor;

#[test]
fn config_extractor_name() {
    assert_eq!(ConfigExtractor::new().name(), "config");
}

#[test]
fn config_extractor_detects_package_json() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("package.json");
    fs::write(&file_path, r#"{"name": "my-frontend", "version": "1.0.0"}"#).unwrap();

    let results = check(&ConfigExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].binding.kind, SourceKind::Config);
    assert_eq!(results[0].suggested_element.as_deref(), Some("my-frontend"));
    assert!(results[0]
        .binding
        .description
        .as_deref()
        .unwrap()
        .contains("Node.js"));
}

#[test]
fn config_extractor_detects_cargo_toml() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("Cargo.toml");
    fs::write(
        &file_path,
        "[package]\nname = \"my-service\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    let results = check(&ConfigExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].suggested_element.as_deref(), Some("my-service"));
    assert!(results[0]
        .binding
        .description
        .as_deref()
        .unwrap()
        .contains("Rust"));
}

#[test]
fn config_extractor_detects_go_mod() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("go.mod");
    fs::write(&file_path, "module github.com/org/payment-svc\n\ngo 1.22\n").unwrap();

    let results = check(&ConfigExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].suggested_element.as_deref(), Some("payment-svc"));
}

#[test]
fn config_extractor_detects_pyproject() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("pyproject.toml");
    fs::write(&file_path, "[project]\nname = \"my-ml-pipeline\"\n").unwrap();

    let results = check(&ConfigExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert!(results[0]
        .binding
        .description
        .as_deref()
        .unwrap()
        .contains("Python"));
}

#[test]
fn config_extractor_ignores_random_files() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("config.yaml");
    fs::write(&file_path, "key: value\n").unwrap();

    let results = check(&ConfigExtractor::new(), &file_path, tmp.path());
    assert!(results.is_empty());
}

#[test]
fn config_extractor_cargo_toml_multiple_sections() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("Cargo.toml");
    fs::write(
        &file_path,
        "[package]\nname = \"my-crate\"\n\n[dependencies]\nserde = \"1\"\n",
    )
    .unwrap();

    let results = check(&ConfigExtractor::new(), &file_path, tmp.path());
    assert_eq!(results[0].suggested_element.as_deref(), Some("my-crate"));
}

#[test]
fn config_extractor_cargo_toml_workspace_no_package_name() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("Cargo.toml");
    fs::write(
        &file_path,
        "[workspace]\nmembers = [\"crates/*\"]\n\n[workspace.package]\nversion = \"1.0\"\n",
    )
    .unwrap();

    let results = check(&ConfigExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    // Falls through to parent dir name since no [package] name found
    assert!(results[0].suggested_element.is_some());
}

#[test]
fn config_extractor_package_json_empty_name() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("package.json");
    fs::write(&file_path, r#"{"name": "", "version": "1.0.0"}"#).unwrap();

    let results = check(&ConfigExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    // Empty name falls through to parent dir
    assert_ne!(results[0].suggested_element.as_deref(), Some(""));
}
