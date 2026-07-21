mod common;
use common::*;
use std::collections::HashMap;
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
use sruja_extract::{
    DiagnosticLevel, ExtractError, ExtractionConfig, ExtractionDiagnostic, ExtractionEngine,
    ExtractionReport, ExtractionStats,
};

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

    assert!(
        report
            .sources
            .iter()
            .all(|s| s.binding.kind != SourceKind::Dockerfile),
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
        ) -> Result<Vec<DiscoveredSource>, ExtractError> {
            if ctx.file_name() == "test.marker" {
                Ok(vec![DiscoveredSource {
                    binding: sruja_language::ast::SourceBinding {
                        kind: SourceKind::Custom("test".to_string()),
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
        SourceKind::Custom("test".to_string())
    );
}

// =========================================================================
// Engine error paths
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
        ) -> Result<Vec<DiscoveredSource>, ExtractError> {
            Err(ExtractError::Parse {
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
        DiagnosticLevel::Error
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
        ) -> Result<Vec<DiscoveredSource>, ExtractError> {
            Ok(Vec::new())
        }
        fn finalize(&self) -> Result<Vec<DiscoveredSource>, ExtractError> {
            Ok(vec![DiscoveredSource {
                binding: sruja_language::ast::SourceBinding {
                    kind: SourceKind::Custom("finalized".to_string()),
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
        ) -> Result<Vec<DiscoveredSource>, ExtractError> {
            Ok(Vec::new())
        }
        fn finalize(&self) -> Result<Vec<DiscoveredSource>, ExtractError> {
            Err(ExtractError::Discovery(
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
    let report = ExtractionReport {
        sources: Vec::new(),
        stats: ExtractionStats {
            files_scanned: 10,
            files_matched: 2,
            total_sources: 3,
            by_extractor: HashMap::new(),
            by_kind: HashMap::new(),
            duration_ms: 42,
        },
        diagnostics: vec![ExtractionDiagnostic {
            level: DiagnosticLevel::Warning,
            extractor: "test".to_string(),
            path: "file.txt".to_string(),
            message: "something went wrong".to_string(),
        }],
    };

    let display = format!("{report}");
    assert!(display.contains("Diagnostics: 1 issues"));
}

// =========================================================================
// Default trait impls
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
