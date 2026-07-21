mod common;
use common::*;
use sruja_extract::alias::AliasExtractor;

#[test]
fn alias_extractor_name() {
    assert_eq!(AliasExtractor::new().name(), "alias");
}

#[test]
fn alias_extractor_detects_docker_compose() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("docker-compose.yaml");
    fs::write(
        &file_path,
        "services:\n  payment-service:\n    image: payment:v1\n  user-service:\n    image: user:v1\n",
    )
    .unwrap();

    let results = check(&AliasExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 2);
    assert!(results
        .iter()
        .any(|r| r.suggested_element.as_deref() == Some("payment-service")));
    assert!(results
        .iter()
        .any(|r| r.suggested_element.as_deref() == Some("user-service")));
}

#[test]
fn alias_extractor_handles_compose_yaml() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("compose.yaml");
    fs::write(&file_path, "services:\n  api:\n    image: api:v1\n").unwrap();

    let results = check(&AliasExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].suggested_element.as_deref(), Some("api"));
}

#[test]
fn alias_extractor_handles_override_compose() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("docker-compose.prod.yaml");
    fs::write(&file_path, "services:\n  api:\n    image: api:v1\n").unwrap();

    let results = check(&AliasExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert!(
        results[0].confidence < 0.9,
        "override files should have lower confidence"
    );
}

#[test]
fn alias_extractor_ignores_non_compose_files() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("config.yaml");
    fs::write(&file_path, "services:\n  api:\n    image: api:v1\n").unwrap();

    let results = check(&AliasExtractor::new(), &file_path, tmp.path());
    assert!(results.is_empty());
}

#[test]
fn alias_extractor_stops_at_next_top_level_key() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("docker-compose.yaml");
    fs::write(
        &file_path,
        "services:\n  web:\n    image: web:v1\nvolumes:\n  data:\n    driver: local\n",
    )
    .unwrap();

    let results = check(&AliasExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].suggested_element.as_deref(), Some("web"));
}
