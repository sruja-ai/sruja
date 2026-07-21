mod common;
use common::*;
use sruja_extract::asyncapi::AsyncApiExtractor;

#[test]
fn asyncapi_extractor_name() {
    assert_eq!(AsyncApiExtractor::new().name(), "asyncapi");
}

#[test]
fn asyncapi_extractor_detects_by_filename() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("asyncapi.yaml");
    fs::write(
        &file_path,
        "asyncapi: 2.6.0\ninfo:\n  title: Order Events\n  version: 1.0.0\nchannels:\n  orders:\n    subscribe: {}\n",
    )
    .unwrap();

    let results = check(&AsyncApiExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].binding.kind, SourceKind::AsyncApi);
    assert!(results[0]
        .binding
        .description
        .as_deref()
        .unwrap()
        .contains("Order Events"));
}

#[test]
fn asyncapi_extractor_detects_by_content() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("events.yaml");
    fs::write(&file_path, "asyncapi: 3.0.0\ninfo:\n  title: Events\n").unwrap();

    let results = check(&AsyncApiExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
}

#[test]
fn asyncapi_extractor_ignores_non_asyncapi() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("events.yaml");
    fs::write(&file_path, "events:\n  - name: order_placed\n").unwrap();

    let results = check(&AsyncApiExtractor::new(), &file_path, tmp.path());
    assert!(results.is_empty());
}

#[test]
fn asyncapi_extractor_no_title_in_info() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("asyncapi.yaml");
    fs::write(
        &file_path,
        "asyncapi: 2.6.0\ninfo:\n  version: 1.0.0\nchannels: {}\n",
    )
    .unwrap();

    let results = check(&AsyncApiExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].binding.description.as_deref(),
        Some("Discovered AsyncAPI specification")
    );
}
