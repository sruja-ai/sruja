use super::*;

#[test]
fn test_source_kind_parsing_and_display() {
    assert_eq!(SourceKind::parse("openapi"), SourceKind::OpenApi);
    assert_eq!(SourceKind::parse("asyncapi"), SourceKind::AsyncApi);
    assert_eq!(SourceKind::parse("kubernetes"), SourceKind::Kubernetes);
    assert_eq!(SourceKind::parse("k8s"), SourceKind::Kubernetes);
    assert_eq!(SourceKind::parse("dockerfile"), SourceKind::Dockerfile);
    assert_eq!(SourceKind::parse("docker"), SourceKind::Dockerfile);
    assert_eq!(SourceKind::parse("terraform"), SourceKind::Terraform);
    assert_eq!(SourceKind::parse("tf"), SourceKind::Terraform);
    assert_eq!(SourceKind::parse("docs"), SourceKind::Docs);
    assert_eq!(SourceKind::parse("doc"), SourceKind::Docs);
    assert_eq!(SourceKind::parse("readme"), SourceKind::Readme);
    assert_eq!(SourceKind::parse("proto"), SourceKind::Proto);
    assert_eq!(SourceKind::parse("protobuf"), SourceKind::Proto);
    assert_eq!(SourceKind::parse("config"), SourceKind::Config);
    assert_eq!(SourceKind::parse("graphql"), SourceKind::GraphQL);
    assert_eq!(SourceKind::parse("gql"), SourceKind::GraphQL);
    assert_eq!(SourceKind::parse("helm"), SourceKind::Helm);
    assert_eq!(
        SourceKind::parse("custom_kind"),
        SourceKind::Custom("custom_kind".to_string())
    );

    assert_eq!(SourceKind::OpenApi.as_str(), "openapi");
    assert_eq!(format!("{}", SourceKind::Kubernetes), "kubernetes");
    assert_eq!(
        format!("{}", SourceKind::Custom("custom".to_string())),
        "custom"
    );

    assert_eq!(
        SourceKind::from_str("openapi").unwrap(),
        SourceKind::OpenApi
    );
}

#[test]
fn test_source_kind_parse_case_insensitive() {
    assert_eq!(SourceKind::parse("OpenAPI"), SourceKind::OpenApi);
    assert_eq!(SourceKind::parse("OPENAPI"), SourceKind::OpenApi);
    assert_eq!(SourceKind::parse("Kubernetes"), SourceKind::Kubernetes);
    assert_eq!(SourceKind::parse("KUBERNETES"), SourceKind::Kubernetes);
}

#[test]
fn test_source_kind_display() {
    assert_eq!(format!("{}", SourceKind::OpenApi), "openapi");
    assert_eq!(format!("{}", SourceKind::AsyncApi), "asyncapi");
    assert_eq!(format!("{}", SourceKind::Kubernetes), "kubernetes");
    assert_eq!(format!("{}", SourceKind::Dockerfile), "dockerfile");
    assert_eq!(format!("{}", SourceKind::Terraform), "terraform");
    assert_eq!(format!("{}", SourceKind::Docs), "docs");
    assert_eq!(format!("{}", SourceKind::Readme), "readme");
    assert_eq!(format!("{}", SourceKind::Proto), "proto");
    assert_eq!(format!("{}", SourceKind::Config), "config");
    assert_eq!(format!("{}", SourceKind::GraphQL), "graphql");
    assert_eq!(format!("{}", SourceKind::Helm), "helm");
    assert_eq!(
        format!("{}", SourceKind::Custom("custom".to_string())),
        "custom"
    );
}

#[test]
fn test_source_kind_as_str() {
    assert_eq!(SourceKind::OpenApi.as_str(), "openapi");
    assert_eq!(SourceKind::AsyncApi.as_str(), "asyncapi");
    assert_eq!(SourceKind::Kubernetes.as_str(), "kubernetes");
    assert_eq!(SourceKind::Dockerfile.as_str(), "dockerfile");
    assert_eq!(SourceKind::Terraform.as_str(), "terraform");
    assert_eq!(SourceKind::Docs.as_str(), "docs");
    assert_eq!(SourceKind::Readme.as_str(), "readme");
    assert_eq!(SourceKind::Proto.as_str(), "proto");
    assert_eq!(SourceKind::Config.as_str(), "config");
    assert_eq!(SourceKind::GraphQL.as_str(), "graphql");
    assert_eq!(SourceKind::Helm.as_str(), "helm");
    assert_eq!(SourceKind::Custom("custom".to_string()).as_str(), "custom");
}

#[test]
fn test_source_kind_from_str() {
    assert_eq!(
        SourceKind::from_str("openapi").unwrap(),
        SourceKind::OpenApi
    );
    assert_eq!(
        SourceKind::from_str("asyncapi").unwrap(),
        SourceKind::AsyncApi
    );
    assert_eq!(
        SourceKind::from_str("kubernetes").unwrap(),
        SourceKind::Kubernetes
    );
    assert_eq!(SourceKind::from_str("k8s").unwrap(), SourceKind::Kubernetes);
    assert_eq!(
        SourceKind::from_str("dockerfile").unwrap(),
        SourceKind::Dockerfile
    );
    assert_eq!(
        SourceKind::from_str("docker").unwrap(),
        SourceKind::Dockerfile
    );
    assert_eq!(
        SourceKind::from_str("terraform").unwrap(),
        SourceKind::Terraform
    );
    assert_eq!(SourceKind::from_str("tf").unwrap(), SourceKind::Terraform);
    assert_eq!(SourceKind::from_str("docs").unwrap(), SourceKind::Docs);
    assert_eq!(SourceKind::from_str("doc").unwrap(), SourceKind::Docs);
    assert_eq!(SourceKind::from_str("readme").unwrap(), SourceKind::Readme);
    assert_eq!(SourceKind::from_str("proto").unwrap(), SourceKind::Proto);
    assert_eq!(
        SourceKind::from_str("protobuf").unwrap(),
        SourceKind::Proto
    );
    assert_eq!(SourceKind::from_str("config").unwrap(), SourceKind::Config);
    assert_eq!(
        SourceKind::from_str("graphql").unwrap(),
        SourceKind::GraphQL
    );
    assert_eq!(SourceKind::from_str("gql").unwrap(), SourceKind::GraphQL);
    assert_eq!(SourceKind::from_str("helm").unwrap(), SourceKind::Helm);
    assert_eq!(
        SourceKind::from_str("custom_kind").unwrap(),
        SourceKind::Custom("custom_kind".to_string())
    );
}

#[test]
fn test_source_binding_creation() {
    let binding = SourceBinding {
        kind: SourceKind::OpenApi,
        path: "/api/openapi.yaml".to_string(),
        description: Some("OpenAPI spec".to_string()),
    };

    assert_eq!(binding.kind, SourceKind::OpenApi);
    assert_eq!(binding.path, "/api/openapi.yaml");
    assert_eq!(binding.description, Some("OpenAPI spec".to_string()));
}

#[test]
fn test_source_binding_clone() {
    let binding = SourceBinding {
        kind: SourceKind::Kubernetes,
        path: "/k8s/deployment.yaml".to_string(),
        description: None,
    };

    let cloned = binding.clone();
    assert_eq!(binding, cloned);
}
