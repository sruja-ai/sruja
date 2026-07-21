mod common;
use common::*;
use sruja_extract::openapi::OpenApiExtractor;

#[test]
fn openapi_extractor_name() {
    assert_eq!(OpenApiExtractor::new().name(), "openapi");
}

#[test]
fn openapi_extractor_detects_yaml() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("openapi.yaml");
    fs::write(&file_path, "openapi: 3.0.0\ninfo:\n  title: Test API").unwrap();

    let results = check(&OpenApiExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].binding.kind, SourceKind::OpenApi);
    assert_eq!(results[0].confidence, 0.8);
}

#[test]
fn openapi_extractor_detects_json() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("api.json");
    fs::write(&file_path, r#"{"openapi": "3.0.0"}"#).unwrap();

    let results = check(&OpenApiExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
}

#[test]
fn openapi_extractor_detects_swagger() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("swagger.yaml");
    fs::write(&file_path, "swagger: 2.0\ninfo:\n  title: Test").unwrap();

    let results = check(&OpenApiExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
}

#[test]
fn openapi_extractor_extracts_title() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("openapi.yaml");
    fs::write(
        &file_path,
        "openapi: 3.0.0\ninfo:\n  title: Payment Service API\n  version: 1.0",
    )
    .unwrap();

    let results = check(&OpenApiExtractor::new(), &file_path, tmp.path());
    assert!(results[0]
        .binding
        .description
        .as_deref()
        .unwrap()
        .contains("Payment Service API"));
}

#[test]
fn openapi_extractor_ignores_non_api_files() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("config.yaml");
    fs::write(&file_path, "setting: value").unwrap();

    let results = check(&OpenApiExtractor::new(), &file_path, tmp.path());
    assert!(results.is_empty());
}

#[test]
fn openapi_extractor_suggests_element_from_parent_dir() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("payment-service").join("openapi.yaml");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, "openapi: 3.0.0").unwrap();

    let results = check(&OpenApiExtractor::new(), &file_path, tmp.path());
    assert_eq!(
        results[0].suggested_element.as_deref(),
        Some("payment-service")
    );
}

#[test]
fn openapi_extractor_handles_swagger_in_filename() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("my-swagger-spec.yaml");
    fs::write(&file_path, "openapi: 3.0.0").unwrap();

    let results = check(&OpenApiExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
}
