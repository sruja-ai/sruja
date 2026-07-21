mod common;
use common::*;
use sruja_extract::proto::ProtoExtractor;

#[test]
fn proto_extractor_name() {
    assert_eq!(ProtoExtractor::new().name(), "proto");
}

#[test]
fn proto_extractor_detects_services() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("payment.proto");
    fs::write(
        &file_path,
        "syntax = \"proto3\";\npackage payment.v1;\n\nservice PaymentService {\n  rpc Charge(ChargeRequest) returns (ChargeResponse);\n}\n",
    )
    .unwrap();

    let results = check(&ProtoExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].suggested_element.as_deref(),
        Some("PaymentService")
    );
    assert!(results[0]
        .binding
        .description
        .as_deref()
        .unwrap()
        .contains("payment.v1"));
}

#[test]
fn proto_extractor_detects_schema_without_service() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("models.proto");
    fs::write(
        &file_path,
        "syntax = \"proto3\";\npackage models;\n\nmessage User {\n  string id = 1;\n}\n",
    )
    .unwrap();

    let results = check(&ProtoExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert!(
        results[0].confidence < 0.8,
        "schema-only proto should have lower confidence"
    );
}

#[test]
fn proto_extractor_ignores_non_proto() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("service.rs");
    fs::write(&file_path, "service PaymentService {}").unwrap();

    let results = check(&ProtoExtractor::new(), &file_path, tmp.path());
    assert!(results.is_empty());
}

#[test]
fn proto_extractor_service_without_package() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("bare.proto");
    fs::write(
        &file_path,
        "syntax = \"proto3\";\n\nservice HealthCheck {\n  rpc Check(Empty) returns (Status);\n}\n",
    )
    .unwrap();

    let results = check(&ProtoExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].suggested_element.as_deref(), Some("HealthCheck"));
    assert!(results[0]
        .binding
        .description
        .as_deref()
        .unwrap()
        .starts_with("gRPC service: HealthCheck"));
}
