mod common;
use common::*;
use serde_json::Value;

#[test]
fn test_architecture_index_dsl_extensions() {
    let dir = create_test_repo();
    let sruja_file = r#"
MySystem = system "My System" {
  description "A top-level system"
  id "canonical-id-001"
  owner "Platform Team"
  domain "Payments"
  criticality high
  aliases ["LegacySystem", "OldPay"]
  source openapi "./specs/payments.yaml"
  source dockerfile "./docker/Dockerfile"

  MyApi = container "API" {
    technology "Rust"
    description "Public API"
    id "api-002"
    criticality critical
    source readme "./README.md"
  }
}
"#;
    write_file(dir.path(), "repo.sruja", sruja_file);

    // 1. Verify lint succeeds
    let (success, _, stderr) =
        run_sruja(&["lint", dir.path().join("repo.sruja").to_str().unwrap()]);
    assert!(success, "lint failed: {}", stderr);

    // 2. Verify export includes these fields
    let (success, stdout, stderr) = run_sruja(&[
        "export",
        "json",
        dir.path().join("repo.sruja").to_str().unwrap(),
    ]);
    assert!(success, "export failed: {}", stderr);

    let json: Value = serde_json::from_str(&stdout).expect("valid json export");
    let elements = json["elements"].as_object().expect("elements object");

    // Check MySystem
    let my_system = elements.get("MySystem").expect("MySystem element");
    assert_eq!(my_system["canonical_id"], "canonical-id-001");
    assert_eq!(my_system["owner"], "Platform Team");
    assert_eq!(my_system["domain"], "Payments");
    assert_eq!(my_system["criticality"], "high");
    assert_eq!(my_system["aliases"][0], "LegacySystem");
    assert_eq!(my_system["aliases"][1], "OldPay");

    let sources = my_system["sources"].as_array().expect("sources array");
    assert!(sources
        .iter()
        .any(|s| s["type"] == "openapi" && s["path"] == "./specs/payments.yaml"));
    assert!(sources
        .iter()
        .any(|s| s["type"] == "dockerfile" && s["path"] == "./docker/Dockerfile"));

    // Check MySystem.MyApi
    let my_api = elements.get("MySystem.MyApi").expect("MyApi element");
    assert_eq!(my_api["canonical_id"], "api-002");
    assert_eq!(my_api["criticality"], "critical");
    let api_sources = my_api["sources"].as_array().expect("api sources array");
    assert!(api_sources
        .iter()
        .any(|s| s["type"] == "readme" && s["path"] == "./README.md"));
}
