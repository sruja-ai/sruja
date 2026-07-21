mod common;
use common::*;
use sruja_extract::dependency::DependencyExtractor;

#[test]
fn dependency_extractor_name() {
    assert_eq!(DependencyExtractor::new().name(), "dependency");
}

#[test]
fn dependency_extractor_finds_service_urls() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("config.yaml");
    fs::write(&file_path, "PAYMENT_SERVICE_URL=https://api.example.com\n").unwrap();

    let results = check(&DependencyExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
    assert!(results.iter().any(|r| r
        .binding
        .description
        .as_deref()
        .unwrap()
        .contains("payment")));
}

#[test]
fn dependency_extractor_finds_host_configs() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("config.ts");
    fs::write(&file_path, "const PAYMENT_HOST = 'localhost:8080';\n").unwrap();

    let results = check(&DependencyExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
}

#[test]
fn dependency_extractor_deduplicates() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("config.ts");
    fs::write(
        &file_path,
        "const PAYMENT_URL = 'http://pay';\nconst PAYMENT_HOST = 'pay:8080';\n",
    )
    .unwrap();

    let results = check(&DependencyExtractor::new(), &file_path, tmp.path());
    let payment_count = results
        .iter()
        .filter(|r| r.suggested_element.as_deref().unwrap().contains("payment"))
        .count();
    assert_eq!(
        payment_count, 1,
        "duplicate signals for same service should be merged"
    );
}

#[test]
fn dependency_extractor_filters_generic_noise() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("config.yaml");
    fs::write(
        &file_path,
        "BASE_URL=https://example.com\nAPP_HOST=localhost\n",
    )
    .unwrap();

    let results = check(&DependencyExtractor::new(), &file_path, tmp.path());
    assert!(
        results.is_empty(),
        "Generic noise like BASE_URL should be filtered"
    );
}

#[test]
fn dependency_extractor_ignores_non_source_files() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("image.png");
    fs::write(&file_path, "fake png content").unwrap();

    let results = check(&DependencyExtractor::new(), &file_path, tmp.path());
    assert!(results.is_empty());
}

#[test]
fn dependency_extractor_finds_endpoint_suffix() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("config.env");
    fs::write(&file_path, "ORDER_ENDPOINT=https://orders.internal\n").unwrap();

    let results = check(&DependencyExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
}
