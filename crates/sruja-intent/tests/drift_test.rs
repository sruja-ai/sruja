//! Basic tests for intent/drift: DriftDetector and IntentModel.

use sruja_intent::{DriftDetector, IntentModel};
use std::fs;

#[test]
fn drift_detector_default_constructs() {
    let _detector = DriftDetector::default();
}

#[test]
fn intent_model_from_empty_file_does_not_panic() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("empty.sruja");
    fs::write(&path, "").expect("write");
    let result = IntentModel::from_sruja_file(&path);
    // Parser may return Ok(empty model) or Err; we only require no panic
    let _ = result;
}

#[test]
fn intent_model_from_minimal_sruja_succeeds() {
    let minimal = r#"
        person = kind "Person"
        system = kind "System"

        user = person "User" { description "User" }
        web = system "Web" { description "Web" }
        user -> web "uses"
    "#;
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("minimal.sruja");
    fs::write(&path, minimal).expect("write");

    let result = IntentModel::from_sruja_file(&path);
    assert!(result.is_ok(), "Minimal .sruja should parse: {:?}", result);
    let model = result.unwrap();
    assert!(!model.components.is_empty(), "model should have components");
}
