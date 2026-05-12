//! Integration tests for all sruja-extract extractors and the engine.

use sruja_extract::alias::AliasExtractor;
use sruja_extract::asyncapi::AsyncApiExtractor;
use sruja_extract::config::ConfigExtractor;
use sruja_extract::dependency::DependencyExtractor;
use sruja_extract::dockerfile::DockerfileExtractor;
use sruja_extract::docs::DocExtractor;
use sruja_extract::graphql::GraphqlExtractor;
use sruja_extract::helm::HelmExtractor;
use sruja_extract::kubernetes::KubernetesExtractor;
use sruja_extract::openapi::OpenApiExtractor;
use sruja_extract::proto::ProtoExtractor;
use sruja_extract::terraform::TerraformExtractor;
use sruja_extract::{DiscoveredSource, ExtractionConfig, ExtractionEngine, Extractor, FileContext};
use std::fs;

fn temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn check(
    extractor: &dyn Extractor,
    path: &std::path::Path,
    root: &std::path::Path,
) -> Vec<DiscoveredSource> {
    let ctx = FileContext::new(path, root);
    extractor
        .check_file(&ctx)
        .expect("check_file should not error")
}

// =========================================================================
// DocExtractor
// =========================================================================

#[test]
fn doc_extractor_name() {
    let extractor = DocExtractor::new();
    assert_eq!(extractor.name(), "docs");
}

#[test]
fn doc_extractor_detects_markdown() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("README.md");
    fs::write(&file_path, "# Test Project\nSome content").unwrap();

    let results = check(&DocExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].binding.kind,
        sruja_language::ast::SourceKind::Readme
    );
}

#[test]
fn doc_extractor_detects_docs() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("CHANGELOG.md");
    fs::write(&file_path, "# Changes").unwrap();

    let results = check(&DocExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].binding.kind,
        sruja_language::ast::SourceKind::Docs
    );
}

#[test]
fn doc_extractor_detects_rst() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("doc.rst");
    fs::write(&file_path, "Document").unwrap();

    let results = check(&DocExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
}

#[test]
fn doc_extractor_detects_asciidoc() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("guide.adoc");
    fs::write(&file_path, "= Guide Title\nContent").unwrap();

    let results = check(&DocExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
}

#[test]
fn doc_extractor_extracts_title_from_markdown() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("guide.md");
    fs::write(&file_path, "# My Architecture Guide\nContent").unwrap();

    let results = check(&DocExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].binding.description.as_deref(),
        Some("My Architecture Guide")
    );
}

#[test]
fn doc_extractor_extracts_title_from_asciidoc() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("guide.adoc");
    fs::write(&file_path, "= AsciiDoc Title\nContent").unwrap();

    let results = check(&DocExtractor::new(), &file_path, tmp.path());
    assert_eq!(
        results[0].binding.description.as_deref(),
        Some("AsciiDoc Title")
    );
}

#[test]
fn doc_extractor_ignores_non_doc_files() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("script.rs");
    fs::write(&file_path, "fn main() {}").unwrap();

    let results = check(&DocExtractor::new(), &file_path, tmp.path());
    assert!(results.is_empty());
}

#[test]
fn doc_extractor_includes_relative_path() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("docs").join("guide.md");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, "# Guide").unwrap();

    let results = check(&DocExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
    assert!(results[0].binding.path.contains("docs/guide.md"));
}

#[test]
fn doc_readme_higher_confidence_than_other_docs() {
    let tmp = temp_dir();

    let readme = tmp.path().join("README.md");
    fs::write(&readme, "# Readme").unwrap();
    let readme_results = check(&DocExtractor::new(), &readme, tmp.path());

    let changelog = tmp.path().join("notes.md");
    fs::write(&changelog, "# Notes").unwrap();
    let changelog_results = check(&DocExtractor::new(), &changelog, tmp.path());

    assert!(readme_results[0].confidence > changelog_results[0].confidence);
}

// =========================================================================
// KubernetesExtractor
// =========================================================================

#[test]
fn kubernetes_extractor_name() {
    assert_eq!(KubernetesExtractor::new().name(), "kubernetes");
}

#[test]
fn kubernetes_extractor_detects_deployment() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("deployment.yaml");
    fs::write(
        &file_path,
        "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: payment-service\nspec:\n  replicas: 3\n",
    )
    .unwrap();

    let results = check(&KubernetesExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
    assert_eq!(
        results[0].suggested_element.as_deref(),
        Some("payment-service")
    );
}

#[test]
fn kubernetes_extractor_detects_service() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("service.yaml");
    fs::write(
        &file_path,
        "apiVersion: v1\nkind: Service\nmetadata:\n  name: user-api\n",
    )
    .unwrap();

    let results = check(&KubernetesExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
    assert_eq!(results[0].suggested_element.as_deref(), Some("user-api"));
}

#[test]
fn kubernetes_extractor_detects_statefulset() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("statefulset.yaml");
    fs::write(
        &file_path,
        "apiVersion: apps/v1\nkind: StatefulSet\nmetadata:\n  name: database\n",
    )
    .unwrap();

    let results = check(&KubernetesExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
    assert_eq!(results[0].suggested_element.as_deref(), Some("database"));
}

#[test]
fn kubernetes_extractor_detects_daemonset() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("daemonset.yaml");
    fs::write(
        &file_path,
        "apiVersion: apps/v1\nkind: DaemonSet\nmetadata:\n  name: log-collector\n",
    )
    .unwrap();

    let results = check(&KubernetesExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
    assert_eq!(
        results[0].suggested_element.as_deref(),
        Some("log-collector")
    );
}

#[test]
fn kubernetes_extractor_detects_cronjob() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("cronjob.yaml");
    fs::write(
        &file_path,
        "apiVersion: batch/v1\nkind: CronJob\nmetadata:\n  name: nightly-cleanup\n",
    )
    .unwrap();

    let results = check(&KubernetesExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
}

#[test]
fn kubernetes_extractor_detects_ingress() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("ingress.yaml");
    fs::write(
        &file_path,
        "apiVersion: networking.k8s.io/v1\nkind: Ingress\nmetadata:\n  name: api-gateway\n",
    )
    .unwrap();

    let results = check(&KubernetesExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
    assert_eq!(results[0].suggested_element.as_deref(), Some("api-gateway"));
}

#[test]
fn kubernetes_extractor_multi_document_yaml() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("all.yaml");
    fs::write(
        &file_path,
        "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: svc-a\n---\napiVersion: v1\nkind: Service\nmetadata:\n  name: svc-b\n",
    )
    .unwrap();

    let results = check(&KubernetesExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 2);
}

#[test]
fn kubernetes_extractor_ignores_non_k8s_files() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("config.yaml");
    fs::write(&file_path, "setting: value\napiVersion: v1").unwrap();

    let results = check(&KubernetesExtractor::new(), &file_path, tmp.path());
    assert!(results.is_empty());
}

#[test]
fn kubernetes_extractor_ignores_non_yaml() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("config.json");
    fs::write(&file_path, "{\"apiVersion\": \"v1\"}").unwrap();

    let results = check(&KubernetesExtractor::new(), &file_path, tmp.path());
    assert!(results.is_empty());
}

#[test]
fn kubernetes_workloads_higher_confidence() {
    let tmp = temp_dir();
    let deploy = tmp.path().join("deploy.yaml");
    fs::write(
        &deploy,
        "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: app\n",
    )
    .unwrap();
    let deploy_results = check(&KubernetesExtractor::new(), &deploy, tmp.path());

    let configmap = tmp.path().join("cm.yaml");
    fs::write(
        &configmap,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: settings\n",
    )
    .unwrap();
    let cm_results = check(&KubernetesExtractor::new(), &configmap, tmp.path());

    assert!(deploy_results[0].confidence > cm_results[0].confidence);
}

// =========================================================================
// OpenApiExtractor
// =========================================================================

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
    assert_eq!(
        results[0].binding.kind,
        sruja_language::ast::SourceKind::OpenApi
    );
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

// =========================================================================
// AliasExtractor
// =========================================================================

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

// =========================================================================
// DependencyExtractor
// =========================================================================

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

// =========================================================================
// DockerfileExtractor
// =========================================================================

#[test]
fn dockerfile_extractor_name() {
    assert_eq!(DockerfileExtractor::new().name(), "dockerfile");
}

#[test]
fn dockerfile_extractor_detects_dockerfile() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("Dockerfile");
    fs::write(
        &file_path,
        "FROM node:20-alpine\nWORKDIR /app\nCOPY . .\nEXPOSE 3000\nCMD [\"node\", \"server.js\"]\n",
    )
    .unwrap();

    let results = check(&DockerfileExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].binding.kind,
        sruja_language::ast::SourceKind::Dockerfile
    );
    let desc = results[0].binding.description.as_deref().unwrap();
    assert!(desc.contains("node:20-alpine"), "should contain base image");
    assert!(desc.contains("3000"), "should contain port");
}

#[test]
fn dockerfile_extractor_detects_variants() {
    let tmp = temp_dir();
    for name in &[
        "Dockerfile.prod",
        "Dockerfile.dev",
        "app.dockerfile",
        "Containerfile",
    ] {
        let file_path = tmp.path().join(name);
        fs::write(&file_path, "FROM ubuntu:22.04\nRUN apt-get update\n").unwrap();

        let results = check(&DockerfileExtractor::new(), &file_path, tmp.path());
        assert!(!results.is_empty(), "should detect {name}");
    }
}

#[test]
fn dockerfile_extractor_ignores_non_dockerfile() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("Makefile");
    fs::write(&file_path, "FROM = something\nall: build\n").unwrap();

    let results = check(&DockerfileExtractor::new(), &file_path, tmp.path());
    assert!(results.is_empty());
}

#[test]
fn dockerfile_extractor_suggests_parent_dir() {
    let tmp = temp_dir();
    let dir = tmp.path().join("payment-service");
    fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("Dockerfile");
    fs::write(&file_path, "FROM golang:1.22\n").unwrap();

    let results = check(&DockerfileExtractor::new(), &file_path, tmp.path());
    assert_eq!(
        results[0].suggested_element.as_deref(),
        Some("payment-service")
    );
}

// =========================================================================
// TerraformExtractor
// =========================================================================

#[test]
fn terraform_extractor_name() {
    assert_eq!(TerraformExtractor::new().name(), "terraform");
}

#[test]
fn terraform_extractor_detects_resources() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("main.tf");
    fs::write(
        &file_path,
        r#"
resource "aws_ecs_service" "payment" {
  name = "payment-svc"
}

resource "aws_rds_instance" "orders_db" {
  engine = "postgres"
}

module "vpc" {
  source = "./modules/vpc"
}
"#,
    )
    .unwrap();

    let results = check(&TerraformExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 3);
    assert_eq!(
        results[0].binding.kind,
        sruja_language::ast::SourceKind::Terraform
    );
}

#[test]
fn terraform_extractor_ignores_non_tf() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("main.rs");
    fs::write(&file_path, "fn main() {}").unwrap();

    let results = check(&TerraformExtractor::new(), &file_path, tmp.path());
    assert!(results.is_empty());
}

#[test]
fn terraform_extractor_resources_higher_confidence() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("main.tf");
    fs::write(
        &file_path,
        "resource \"aws_lambda_function\" \"handler\" {\n}\n\nmodule \"utils\" {\n  source = \"./m\"\n}\n",
    )
    .unwrap();

    let results = check(&TerraformExtractor::new(), &file_path, tmp.path());
    let resource = results
        .iter()
        .find(|r| {
            r.binding
                .description
                .as_deref()
                .unwrap()
                .contains("resource.")
        })
        .unwrap();
    let module = results
        .iter()
        .find(|r| r.binding.description.as_deref().unwrap().contains("module"))
        .unwrap();
    assert!(resource.confidence > module.confidence);
}

// =========================================================================
// ProtoExtractor
// =========================================================================

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

// =========================================================================
// GraphqlExtractor
// =========================================================================

#[test]
fn graphql_extractor_name() {
    assert_eq!(GraphqlExtractor::new().name(), "graphql");
}

#[test]
fn graphql_extractor_detects_schema() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("schema.graphql");
    fs::write(
        &file_path,
        "type Query {\n  user(id: ID!): User\n}\n\ntype User {\n  id: ID!\n  name: String!\n}\n",
    )
    .unwrap();

    let results = check(&GraphqlExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].binding.kind,
        sruja_language::ast::SourceKind::GraphQL
    );
    assert!(results[0]
        .binding
        .description
        .as_deref()
        .unwrap()
        .contains("Query"));
}

#[test]
fn graphql_extractor_detects_gql_extension() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("types.gql");
    fs::write(
        &file_path,
        "type Mutation {\n  createUser(input: CreateUserInput!): User\n}\n",
    )
    .unwrap();

    let results = check(&GraphqlExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
}

#[test]
fn graphql_extractor_ignores_non_schema() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("query.graphql");
    fs::write(&file_path, "query GetUser {\n  user(id: 1) { name }\n}\n").unwrap();

    let results = check(&GraphqlExtractor::new(), &file_path, tmp.path());
    assert!(
        results.is_empty(),
        "operation-only .graphql files are not schema definitions"
    );
}

#[test]
fn graphql_extractor_ignores_non_graphql() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("schema.json");
    fs::write(&file_path, "{\"type\": \"Query\"}").unwrap();

    let results = check(&GraphqlExtractor::new(), &file_path, tmp.path());
    assert!(results.is_empty());
}

// =========================================================================
// HelmExtractor
// =========================================================================

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
    assert_eq!(
        results[0].binding.kind,
        sruja_language::ast::SourceKind::Helm
    );
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
    assert_eq!(
        results[0].binding.kind,
        sruja_language::ast::SourceKind::Helm
    );
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

// =========================================================================
// AsyncApiExtractor
// =========================================================================

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
    assert_eq!(
        results[0].binding.kind,
        sruja_language::ast::SourceKind::AsyncApi
    );
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

// =========================================================================
// ConfigExtractor
// =========================================================================

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
    assert_eq!(
        results[0].binding.kind,
        sruja_language::ast::SourceKind::Config
    );
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

// =========================================================================
// DiscoveredSource serialization
// =========================================================================

#[test]
fn discovered_source_serialization() {
    let source = DiscoveredSource {
        binding: sruja_language::ast::SourceBinding {
            kind: sruja_language::ast::SourceKind::OpenApi,
            path: "api.yaml".to_string(),
            description: Some("Test API".to_string()),
        },
        suggested_element: Some("MyService".to_string()),
        confidence: 0.9,
    };

    let json = serde_json::to_string(&source).expect("serialize");
    let deserialized: DiscoveredSource = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(deserialized.binding.path, "api.yaml");
    assert_eq!(deserialized.suggested_element.as_deref(), Some("MyService"));
    assert_eq!(deserialized.confidence, 0.9);
}

#[test]
fn discovered_source_display() {
    let source = DiscoveredSource {
        binding: sruja_language::ast::SourceBinding {
            kind: sruja_language::ast::SourceKind::Kubernetes,
            path: "deploy.yaml".to_string(),
            description: Some("Deployment".to_string()),
        },
        suggested_element: Some("api-gateway".to_string()),
        confidence: 0.8,
    };

    let display = format!("{source}");
    assert!(display.contains("kubernetes"));
    assert!(display.contains("deploy.yaml"));
    assert!(display.contains("api-gateway"));
    assert!(display.contains("80%"));
}

#[test]
fn discovered_source_partial_eq() {
    let a = DiscoveredSource {
        binding: sruja_language::ast::SourceBinding {
            kind: sruja_language::ast::SourceKind::OpenApi,
            path: "api.yaml".to_string(),
            description: Some("A".to_string()),
        },
        suggested_element: Some("svc".to_string()),
        confidence: 0.5,
    };
    let b = DiscoveredSource {
        binding: sruja_language::ast::SourceBinding {
            kind: sruja_language::ast::SourceKind::OpenApi,
            path: "api.yaml".to_string(),
            description: Some("B".to_string()),
        },
        suggested_element: Some("svc".to_string()),
        confidence: 0.9,
    };

    assert_eq!(
        a, b,
        "PartialEq should compare path+kind+element, not description/confidence"
    );
}

// =========================================================================
// FileContext
// =========================================================================

#[test]
fn file_context_relative_path() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("src").join("main.rs");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, "fn main() {}").unwrap();

    let ctx = FileContext::new(&file_path, tmp.path());
    assert_eq!(ctx.relative_path(), "src/main.rs");
}

#[test]
fn file_context_lazy_content() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("test.txt");
    fs::write(&file_path, "hello world").unwrap();

    let ctx = FileContext::new(&file_path, tmp.path());
    assert_eq!(ctx.content(), Some("hello world"));
    // Reading again should return cached value
    assert_eq!(ctx.content(), Some("hello world"));
}

#[test]
fn file_context_missing_file() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("nonexistent.txt");

    let ctx = FileContext::new(&file_path, tmp.path());
    assert_eq!(ctx.content(), None);
}

#[test]
fn file_context_file_name() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("Dockerfile.prod");
    fs::write(&file_path, "FROM alpine").unwrap();

    let ctx = FileContext::new(&file_path, tmp.path());
    assert_eq!(ctx.file_name(), "Dockerfile.prod");
    assert_eq!(ctx.file_name_lower(), "dockerfile.prod");
}

#[test]
fn file_context_extension() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("schema.graphql");
    fs::write(&file_path, "").unwrap();

    let ctx = FileContext::new(&file_path, tmp.path());
    assert_eq!(ctx.extension(), "graphql");
}

#[test]
fn file_context_parent_dir_name() {
    let tmp = temp_dir();
    let dir = tmp.path().join("payment-service");
    fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("main.rs");
    fs::write(&file_path, "").unwrap();

    let ctx = FileContext::new(&file_path, tmp.path());
    assert_eq!(ctx.parent_dir_name(), Some("payment-service"));
}

// =========================================================================
// ExtractionConfig
// =========================================================================

#[test]
fn extraction_config_struct_update() {
    let config = ExtractionConfig {
        min_confidence: 0.5,
        max_file_size: 1024,
        extra_ignore_patterns: vec!["*.log".to_string()],
        respect_gitignore: false,
        follow_symlinks: true,
        ..Default::default()
    };

    assert_eq!(config.min_confidence, 0.5);
    assert_eq!(config.max_file_size, 1024);
    assert!(!config.respect_gitignore);
    assert!(config.follow_symlinks);
}

#[test]
fn extraction_config_with_min_confidence() {
    let config = ExtractionConfig::with_min_confidence(0.7);
    assert_eq!(config.min_confidence, 0.7);
    assert_eq!(config.max_file_size, 10 * 1024 * 1024);
}

#[test]
fn extraction_config_enabled_extractors() {
    let config = ExtractionConfig {
        enabled_extractors: Some(vec!["docs".to_string(), "kubernetes".to_string()]),
        ..Default::default()
    };

    let engine = ExtractionEngine::with_config(config);

    let tmp = temp_dir();
    let dockerfile = tmp.path().join("Dockerfile");
    fs::write(&dockerfile, "FROM alpine\n").unwrap();

    let readme = tmp.path().join("README.md");
    fs::write(&readme, "# Test\n").unwrap();

    let report = engine.discover(tmp.path());

    // Should find README (docs) but not Dockerfile (dockerfile extractor not enabled)
    assert!(
        report
            .sources
            .iter()
            .all(|s| s.binding.kind != sruja_language::ast::SourceKind::Dockerfile),
        "dockerfile extractor should be disabled"
    );
}

// =========================================================================
// ExtractionEngine
// =========================================================================

#[test]
fn engine_discover_produces_report() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("openapi.yaml");
    fs::write(&file_path, "openapi: 3.0.0\ninfo:\n  title: Test\n").unwrap();

    let dockerfile = tmp.path().join("Dockerfile");
    fs::write(&dockerfile, "FROM node:20\nEXPOSE 3000\n").unwrap();

    let engine = ExtractionEngine::default();
    let report = engine.discover(tmp.path());

    assert!(report.stats.files_scanned > 0);
    assert!(report.stats.files_matched > 0);
    assert!(!report.sources.is_empty());
    assert!(!report.stats.by_kind.is_empty());
}

#[test]
fn engine_min_confidence_filter() {
    let tmp = temp_dir();

    // Dependency signals have confidence 0.3
    let config_file = tmp.path().join("config.yaml");
    fs::write(&config_file, "PAYMENT_SERVICE_URL=http://pay\n").unwrap();

    // OpenAPI has confidence 0.8
    let openapi = tmp.path().join("openapi.yaml");
    fs::write(&openapi, "openapi: 3.0.0\n").unwrap();

    let config = ExtractionConfig::with_min_confidence(0.5);
    let engine = ExtractionEngine::with_config(config);
    let report = engine.discover(tmp.path());

    assert!(
        report.sources.iter().all(|s| s.confidence >= 0.5),
        "all results should have confidence >= 0.5"
    );
}

#[test]
fn engine_discover_all_backward_compat() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("README.md");
    fs::write(&file_path, "# Test\n").unwrap();

    let engine = ExtractionEngine::default();
    let sources = engine.discover_all(tmp.path());
    assert!(!sources.is_empty());
}

#[test]
fn engine_max_file_size_filter() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("huge.yaml");
    let content = "openapi: 3.0.0\n".to_string() + &"x".repeat(200);
    fs::write(&file_path, &content).unwrap();

    let config = ExtractionConfig {
        max_file_size: 50,
        ..Default::default()
    };
    let engine = ExtractionEngine::with_config(config);
    let report = engine.discover(tmp.path());

    assert!(report.sources.is_empty(), "large files should be skipped");
}

#[test]
fn engine_report_display() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("openapi.yaml");
    fs::write(&file_path, "openapi: 3.0.0\n").unwrap();

    let engine = ExtractionEngine::default();
    let report = engine.discover(tmp.path());

    let display = format!("{report}");
    assert!(display.contains("Extraction Report"));
    assert!(display.contains("Files scanned"));
}

#[test]
fn engine_add_custom_extractor() {
    struct TestExtractor;
    impl Extractor for TestExtractor {
        fn name(&self) -> &'static str {
            "test"
        }
        fn check_file(
            &self,
            ctx: &FileContext,
        ) -> Result<Vec<DiscoveredSource>, sruja_extract::ExtractError> {
            if ctx.file_name() == "test.marker" {
                Ok(vec![DiscoveredSource {
                    binding: sruja_language::ast::SourceBinding {
                        kind: sruja_language::ast::SourceKind::Custom("test".to_string()),
                        path: ctx.relative_path().to_string(),
                        description: Some("test marker".to_string()),
                    },
                    suggested_element: None,
                    confidence: 1.0,
                }])
            } else {
                Ok(Vec::new())
            }
        }
    }

    let tmp = temp_dir();
    let file_path = tmp.path().join("test.marker");
    fs::write(&file_path, "").unwrap();

    let mut engine = ExtractionEngine::with_extractors(Vec::new(), ExtractionConfig::default());
    engine.add_extractor(Box::new(TestExtractor));

    let report = engine.discover(tmp.path());
    assert_eq!(report.sources.len(), 1);
    assert_eq!(
        report.sources[0].binding.kind,
        sruja_language::ast::SourceKind::Custom("test".to_string())
    );
}

// =========================================================================
// Coverage gap: Engine error paths
// =========================================================================

#[test]
fn engine_extractor_error_captured_as_diagnostic() {
    struct FailingExtractor;
    impl Extractor for FailingExtractor {
        fn name(&self) -> &'static str {
            "failing"
        }
        fn check_file(
            &self,
            ctx: &FileContext,
        ) -> Result<Vec<DiscoveredSource>, sruja_extract::ExtractError> {
            Err(sruja_extract::ExtractError::Parse {
                path: ctx.relative_path().to_string(),
                message: "intentional test failure".to_string(),
            })
        }
    }

    let tmp = temp_dir();
    fs::write(tmp.path().join("test.txt"), "content").unwrap();

    let mut engine = ExtractionEngine::with_extractors(Vec::new(), ExtractionConfig::default());
    engine.add_extractor(Box::new(FailingExtractor));

    let report = engine.discover(tmp.path());
    assert!(report.sources.is_empty());
    assert!(!report.diagnostics.is_empty());
    assert_eq!(
        report.diagnostics[0].level,
        sruja_extract::DiagnosticLevel::Error
    );
    assert_eq!(report.diagnostics[0].extractor, "failing");
    assert!(report.diagnostics[0].message.contains("intentional"));
}

#[test]
fn engine_finalize_results_included() {
    struct FinalizeExtractor;
    impl Extractor for FinalizeExtractor {
        fn name(&self) -> &'static str {
            "finalizer"
        }
        fn check_file(
            &self,
            _ctx: &FileContext,
        ) -> Result<Vec<DiscoveredSource>, sruja_extract::ExtractError> {
            Ok(Vec::new())
        }
        fn finalize(&self) -> Result<Vec<DiscoveredSource>, sruja_extract::ExtractError> {
            Ok(vec![DiscoveredSource {
                binding: sruja_language::ast::SourceBinding {
                    kind: sruja_language::ast::SourceKind::Custom("finalized".to_string()),
                    path: "virtual".to_string(),
                    description: Some("from finalize".to_string()),
                },
                suggested_element: None,
                confidence: 1.0,
            }])
        }
    }

    let tmp = temp_dir();
    let mut engine = ExtractionEngine::with_extractors(Vec::new(), ExtractionConfig::default());
    engine.add_extractor(Box::new(FinalizeExtractor));

    let report = engine.discover(tmp.path());
    assert_eq!(report.sources.len(), 1);
    assert_eq!(report.sources[0].binding.path, "virtual");
    assert!(report.stats.by_extractor.contains_key("finalizer"));
}

#[test]
fn engine_finalize_error_captured() {
    struct FinalizeFailing;
    impl Extractor for FinalizeFailing {
        fn name(&self) -> &'static str {
            "finalize-fail"
        }
        fn check_file(
            &self,
            _ctx: &FileContext,
        ) -> Result<Vec<DiscoveredSource>, sruja_extract::ExtractError> {
            Ok(Vec::new())
        }
        fn finalize(&self) -> Result<Vec<DiscoveredSource>, sruja_extract::ExtractError> {
            Err(sruja_extract::ExtractError::Discovery(
                "finalize boom".to_string(),
            ))
        }
    }

    let tmp = temp_dir();
    let mut engine = ExtractionEngine::with_extractors(Vec::new(), ExtractionConfig::default());
    engine.add_extractor(Box::new(FinalizeFailing));

    let report = engine.discover(tmp.path());
    assert!(report.sources.is_empty());
    assert_eq!(report.diagnostics.len(), 1);
    assert!(report.diagnostics[0].message.contains("finalize boom"));
}

#[test]
fn engine_report_display_with_diagnostics() {
    let report = sruja_extract::ExtractionReport {
        sources: Vec::new(),
        stats: sruja_extract::ExtractionStats {
            files_scanned: 10,
            files_matched: 2,
            total_sources: 3,
            by_extractor: std::collections::HashMap::new(),
            by_kind: std::collections::HashMap::new(),
            duration_ms: 42,
        },
        diagnostics: vec![sruja_extract::ExtractionDiagnostic {
            level: sruja_extract::DiagnosticLevel::Warning,
            extractor: "test".to_string(),
            path: "file.txt".to_string(),
            message: "something went wrong".to_string(),
        }],
    };

    let display = format!("{report}");
    assert!(display.contains("Diagnostics: 1 issues"));
}

// =========================================================================
// Coverage gap: Alias — services section followed by other keys
// =========================================================================

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

// =========================================================================
// Coverage gap: Config — Cargo.toml edge cases
// =========================================================================

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

// =========================================================================
// Coverage gap: Helm — chart without description, name edge case
// =========================================================================

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

// =========================================================================
// Coverage gap: AsyncAPI — title not found
// =========================================================================

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

// =========================================================================
// Coverage gap: GraphQL — schema with >5 types (truncation)
// =========================================================================

#[test]
fn graphql_extractor_truncates_many_types() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("schema.graphql");
    fs::write(
        &file_path,
        "type Query { id: ID }\ntype User { id: ID }\ntype Post { id: ID }\ntype Comment { id: ID }\ntype Tag { id: ID }\ntype Category { id: ID }\ntype Author { id: ID }\n",
    )
    .unwrap();

    let results = check(&GraphqlExtractor::new(), &file_path, tmp.path());
    let desc = results[0].binding.description.as_deref().unwrap();
    assert!(
        desc.contains("+"),
        "should indicate truncated types: {desc}"
    );
}

// =========================================================================
// Coverage gap: GraphQL — schema with only input/enum (no types extracted)
// =========================================================================

#[test]
fn graphql_extractor_schema_with_enum_only() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("enums.graphql");
    fs::write(&file_path, "enum Status {\n  ACTIVE\n  INACTIVE\n}\n").unwrap();

    let results = check(&GraphqlExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].binding.description.as_deref(),
        Some("GraphQL schema")
    );
}

// =========================================================================
// Coverage gap: Proto — service without package
// =========================================================================

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

// =========================================================================
// Coverage gap: Terraform — data blocks
// =========================================================================

#[test]
fn terraform_extractor_detects_data_blocks() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("data.tf");
    fs::write(
        &file_path,
        "data \"aws_vpc\" \"main\" {\n  default = true\n}\n",
    )
    .unwrap();

    let results = check(&TerraformExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    let desc = results[0].binding.description.as_deref().unwrap();
    assert!(desc.contains("data.aws_vpc"));
    assert!(
        results[0].suggested_element.is_none(),
        "data blocks should not suggest elements"
    );
}

// =========================================================================
// Coverage gap: Dockerfile — no EXPOSE, multi-stage
// =========================================================================

#[test]
fn dockerfile_extractor_no_ports() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("Dockerfile");
    fs::write(&file_path, "FROM python:3.12\nRUN pip install flask\n").unwrap();

    let results = check(&DockerfileExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    let desc = results[0].binding.description.as_deref().unwrap();
    assert!(desc.contains("python:3.12"));
    assert!(!desc.contains("ports"));
}

// =========================================================================
// Coverage gap: Default trait impls
// =========================================================================

#[test]
fn extractors_default_impls() {
    let _: DocExtractor = Default::default();
    let _: OpenApiExtractor = Default::default();
    let _: KubernetesExtractor = Default::default();
    let _: AliasExtractor = Default::default();
    let _: DependencyExtractor = Default::default();
    let _: DockerfileExtractor = Default::default();
    let _: TerraformExtractor = Default::default();
    let _: ProtoExtractor = Default::default();
    let _: GraphqlExtractor = Default::default();
    let _: HelmExtractor = Default::default();
    let _: AsyncApiExtractor = Default::default();
    let _: ConfigExtractor = Default::default();
}
