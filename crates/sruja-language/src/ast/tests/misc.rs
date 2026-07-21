use super::*;

#[test]
fn test_import_statement_creation() {
    let import = ImportStatement {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        elements: vec![
            ImportElement::Ident("MySystem".to_string()),
            ImportElement::Wildcard,
        ],
        from: "./other.sruja".to_string(),
    };

    assert_eq!(import.elements.len(), 2);
    assert_eq!(import.from, "./other.sruja".to_string());
}

#[test]
fn test_import_element_variants() {
    let ident = ImportElement::Ident("test".to_string());
    let wildcard = ImportElement::Wildcard;

    assert!(matches!(ident, ImportElement::Ident(_)));
    assert!(matches!(wildcard, ImportElement::Wildcard));
}

#[test]
fn test_slo_block_creation() {
    let slo = SloBlock {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        availability: Some(SloAvailability {
            target: Some("99.9%".to_string()),
            window: Some("30d".to_string()),
            current: Some("99.95%".to_string()),
        }),
        latency: Some(SloLatency {
            p95: Some("100ms".to_string()),
            p99: Some("200ms".to_string()),
            window: Some("5m".to_string()),
            current: Some(SloCurrent {
                p95: Some("95ms".to_string()),
                p99: Some("180ms".to_string()),
            }),
        }),
        error_rate: Some(SloErrorRate {
            target: Some("0.1%".to_string()),
            window: Some("1h".to_string()),
            current: Some("0.05%".to_string()),
        }),
        throughput: Some(SloThroughput {
            target: Some("1000 rps".to_string()),
            window: Some("1m".to_string()),
            current: Some("950 rps".to_string()),
        }),
    };

    assert!(slo.availability.is_some());
    assert!(slo.latency.is_some());
    assert!(slo.error_rate.is_some());
    assert!(slo.throughput.is_some());
}

#[test]
fn test_fitness_def_creation() {
    let fitness = FitnessDef {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        id: "fitness1".to_string(),
        target: "99.9%".to_string(),
        measure: "availability".to_string(),
    };

    assert_eq!(fitness.id, "fitness1");
    assert_eq!(fitness.target, "99.9%");
    assert_eq!(fitness.measure, "availability");
}

#[test]
fn test_fitness_def_clone() {
    let fitness = FitnessDef {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        id: "fitness1".to_string(),
        target: "99.9%".to_string(),
        measure: "availability".to_string(),
    };

    let cloned = fitness.clone();
    assert_eq!(fitness, cloned);
}

#[test]
fn test_incident_creation() {
    let incident = Incident {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        id: "incident1".to_string(),
        title: "Service Outage".to_string(),
        date: Some("2024-01-15".to_string()),
        severity: Some("critical".to_string()),
        affected: vec![QualifiedIdent::simple("API".to_string())],
        cause: Some("Database connection failure".to_string()),
        resolution: Some("Restarted database".to_string()),
        lesson: Some("Add connection pooling".to_string()),
    };

    assert_eq!(incident.id, "incident1");
    assert_eq!(incident.severity, Some("critical".to_string()));
    assert_eq!(incident.affected.len(), 1);
}
