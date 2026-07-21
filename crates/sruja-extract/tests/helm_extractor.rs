mod common;
use common::*;
use sruja_extract::helm::HelmExtractor;

#[test]
fn helm_extractor_name() {
    assert_eq!(HelmExtractor::new().name(), "helm");
}

#[test]
fn helm_extractor_detects_chart_yaml() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("Chart.yaml");
    fs::write(
        &file_path,
        "apiVersion: v2\nname: payment-chart\ndescription: Payment service Helm chart\nversion: 1.0.0\n",
    )
    .unwrap();

    let results = check(&HelmExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].binding.kind, SourceKind::Helm);
    assert_eq!(
        results[0].suggested_element.as_deref(),
        Some("payment-chart")
    );
}

#[test]
fn helm_extractor_values_with_chart() {
    let tmp = temp_dir();
    let chart = tmp.path().join("Chart.yaml");
    fs::write(&chart, "apiVersion: v2\nname: my-chart\n").unwrap();

    let values = tmp.path().join("values.yaml");
    fs::write(&values, "replicaCount: 3\nimage:\n  repository: nginx\n").unwrap();

    let results = check(&HelmExtractor::new(), &values, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].binding.kind, SourceKind::Helm);
}

#[test]
fn helm_extractor_values_without_chart_ignored() {
    let tmp = temp_dir();
    let values = tmp.path().join("values.yaml");
    fs::write(&values, "replicaCount: 3\n").unwrap();

    let results = check(&HelmExtractor::new(), &values, tmp.path());
    assert!(
        results.is_empty(),
        "values.yaml without Chart.yaml should be ignored"
    );
}

#[test]
fn helm_extractor_ignores_non_chart() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("Chart.yaml");
    fs::write(&file_path, "something: else\nno_api_version: true\n").unwrap();

    let results = check(&HelmExtractor::new(), &file_path, tmp.path());
    assert!(results.is_empty());
}

#[test]
fn helm_extractor_chart_without_description() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("Chart.yaml");
    fs::write(
        &file_path,
        "apiVersion: v2\nname: simple-chart\nversion: 0.1.0\n",
    )
    .unwrap();

    let results = check(&HelmExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].suggested_element.as_deref(),
        Some("simple-chart")
    );
    assert!(results[0]
        .binding
        .description
        .as_deref()
        .unwrap()
        .contains("simple-chart"));
}
