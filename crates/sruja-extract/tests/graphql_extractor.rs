mod common;
use common::*;
use sruja_extract::graphql::GraphqlExtractor;

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
    assert_eq!(results[0].binding.kind, SourceKind::GraphQL);
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
