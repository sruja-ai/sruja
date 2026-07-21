mod common;
use common::*;

#[test]
fn discovered_source_serialization() {
    let source = DiscoveredSource {
        binding: sruja_language::ast::SourceBinding {
            kind: SourceKind::OpenApi,
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
            kind: SourceKind::Kubernetes,
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
            kind: SourceKind::OpenApi,
            path: "api.yaml".to_string(),
            description: Some("A".to_string()),
        },
        suggested_element: Some("svc".to_string()),
        confidence: 0.5,
    };
    let b = DiscoveredSource {
        binding: sruja_language::ast::SourceBinding {
            kind: SourceKind::OpenApi,
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
