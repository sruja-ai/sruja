//! Test for edge rendering with lhead/ltail attributes

use super::DotExporter;
use sruja_diagnostics::SourceLocation;
use sruja_language::{ElementDef, ElementKind, Program, QualifiedIdent, Relation, TopLevelItem};

fn dummy_location() -> SourceLocation {
    SourceLocation::new("test.sruja".to_string(), 1, 1)
}

#[test]
fn test_cross_cluster_edge_attributes() {
    // Create a simple program with elements in different clusters
    let program = Program {
        items: vec![
            // Parent element (cluster)
            TopLevelItem::ElementDef(Box::new(ElementDef {
                location: dummy_location(),
                assignment: sruja_language::ElementAssignment {
                    location: dummy_location(),
                    name: "system1".to_string(),
                    kind: ElementKind::System,
                    sub_kind: None,
                    title: Some("System 1".to_string()),
                    tag_refs: vec![],
                    body: None,
                },
            })),
            // Child element in cluster
            TopLevelItem::ElementDef(Box::new(ElementDef {
                location: dummy_location(),
                assignment: sruja_language::ElementAssignment {
                    location: dummy_location(),
                    name: "system1.service1".to_string(),
                    kind: ElementKind::Container,
                    sub_kind: None,
                    title: Some("Service 1".to_string()),
                    tag_refs: vec![],
                    body: None,
                },
            })),
            // Another parent element (different cluster)
            TopLevelItem::ElementDef(Box::new(ElementDef {
                location: dummy_location(),
                assignment: sruja_language::ElementAssignment {
                    location: dummy_location(),
                    name: "system2".to_string(),
                    kind: ElementKind::System,
                    sub_kind: None,
                    title: Some("System 2".to_string()),
                    tag_refs: vec![],
                    body: None,
                },
            })),
            // Child element in second cluster
            TopLevelItem::ElementDef(Box::new(ElementDef {
                location: dummy_location(),
                assignment: sruja_language::ElementAssignment {
                    location: dummy_location(),
                    name: "system2.service2".to_string(),
                    kind: ElementKind::Container,
                    sub_kind: None,
                    title: Some("Service 2".to_string()),
                    tag_refs: vec![],
                    body: None,
                },
            })),
            // Relation between services in different clusters
            TopLevelItem::Relation(Relation {
                location: dummy_location(),
                from: QualifiedIdent::qualified(vec![
                    "system1".to_string(),
                    "service1".to_string(),
                ]),
                to: QualifiedIdent::qualified(vec!["system2".to_string(), "service2".to_string()]),
                label: Some("calls".to_string()),
                description: None,
                technology: None,
                tags: vec![],
            }),
        ],
    };

    let mut config = super::DotConfig::default();
    config.view_level = 3; // Show all levels
    let exporter = DotExporter::new(config);
    let dot_output = exporter.export(&program);

    eprintln!("DOT output:\n{}", dot_output);

    // Verify that lhead and ltail attributes are present for cross-cluster edge
    assert!(
        dot_output.contains("ltail=\"cluster_system1\""),
        "Edge should have ltail attribute for source cluster"
    );
    assert!(
        dot_output.contains("lhead=\"cluster_system2\""),
        "Edge should have lhead attribute for target cluster"
    );

    // Verify the edge structure
    assert!(
        dot_output.contains("\"system1.service1\" -> \"system2.service2\""),
        "Edge should connect the two services"
    );
    assert!(
        dot_output.contains("label=\"calls\""),
        "Edge should have the label"
    );
}

#[test]
fn test_same_cluster_edge_no_attributes() {
    // Create a program with elements in the same cluster
    let program = Program {
        items: vec![
            // Parent element (cluster)
            TopLevelItem::ElementDef(Box::new(ElementDef {
                location: dummy_location(),
                assignment: sruja_language::ElementAssignment {
                    location: dummy_location(),
                    name: "system1".to_string(),
                    kind: ElementKind::System,
                    sub_kind: None,
                    title: Some("System 1".to_string()),
                    tag_refs: vec![],
                    body: None,
                },
            })),
            // Child element in cluster
            TopLevelItem::ElementDef(Box::new(ElementDef {
                location: dummy_location(),
                assignment: sruja_language::ElementAssignment {
                    location: dummy_location(),
                    name: "system1.service1".to_string(),
                    kind: ElementKind::Container,
                    sub_kind: None,
                    title: Some("Service 1".to_string()),
                    tag_refs: vec![],
                    body: None,
                },
            })),
            // Another child element in same cluster
            TopLevelItem::ElementDef(Box::new(ElementDef {
                location: dummy_location(),
                assignment: sruja_language::ElementAssignment {
                    location: dummy_location(),
                    name: "system1.service2".to_string(),
                    kind: ElementKind::Container,
                    sub_kind: None,
                    title: Some("Service 2".to_string()),
                    tag_refs: vec![],
                    body: None,
                },
            })),
            // Relation between services in the same cluster
            TopLevelItem::Relation(Relation {
                location: dummy_location(),
                from: QualifiedIdent::qualified(vec![
                    "system1".to_string(),
                    "service1".to_string(),
                ]),
                to: QualifiedIdent::qualified(vec!["system1".to_string(), "service2".to_string()]),
                label: Some("calls".to_string()),
                description: None,
                technology: None,
                tags: vec![],
            }),
        ],
    };

    let mut config = super::DotConfig::default();
    config.view_level = 3; // Show all levels
    let exporter = DotExporter::new(config);
    let dot_output = exporter.export(&program);

    // Verify that lhead and ltail attributes are NOT present for same-cluster edge
    assert!(
        !dot_output.contains("ltail=\"cluster_system1\""),
        "Same-cluster edge should NOT have ltail attribute"
    );
    assert!(
        !dot_output.contains("lhead=\"cluster_system1\""),
        "Same-cluster edge should NOT have lhead attribute"
    );

    // Verify the edge still exists
    assert!(
        dot_output.contains("\"system1.service1\" -> \"system1.service2\""),
        "Edge should connect the two services"
    );
    assert!(
        dot_output.contains("label=\"calls\""),
        "Edge should have the label"
    );
}

#[test]
fn test_fqdn_edge_resolution_nested_elements() {
    // Create a program with nested elements and simple name relations
    let program = Program {
        items: vec![
            // Parent system with nested containers
            TopLevelItem::ElementDef(Box::new(ElementDef {
                location: dummy_location(),
                assignment: sruja_language::ElementAssignment {
                    location: dummy_location(),
                    name: "Backend".to_string(),
                    kind: ElementKind::System,
                    sub_kind: None,
                    title: Some("Backend System".to_string()),
                    tag_refs: vec![],
                    body: Some(sruja_language::ElementDefBody {
                        description: None,
                        technology: None,
                        metadata: vec![],
                        constraints: vec![],
                        conventions: vec![],
                        style: None,
                        scale: None,
                        slo: None,
                        items: vec![
                            // Nested container API
                            sruja_language::ElementDefBodyItem::ElementDef(Box::new(ElementDef {
                                location: dummy_location(),
                                assignment: sruja_language::ElementAssignment {
                                    location: dummy_location(),
                                    name: "API".to_string(),
                                    kind: ElementKind::Container,
                                    sub_kind: None,
                                    title: Some("API Service".to_string()),
                                    tag_refs: vec![],
                                    body: None,
                                },
                            })),
                            // Nested container DB
                            sruja_language::ElementDefBodyItem::ElementDef(Box::new(ElementDef {
                                location: dummy_location(),
                                assignment: sruja_language::ElementAssignment {
                                    location: dummy_location(),
                                    name: "DB".to_string(),
                                    kind: ElementKind::Database,
                                    sub_kind: None,
                                    title: Some("Database".to_string()),
                                    tag_refs: vec![],
                                    body: None,
                                },
                            })),
                            // Relation with simple names inside the body
                            sruja_language::ElementDefBodyItem::Relation(Relation {
                                location: dummy_location(),
                                from: QualifiedIdent::simple("API".to_string()),
                                to: QualifiedIdent::simple("DB".to_string()),
                                label: Some("reads/writes".to_string()),
                                description: None,
                                technology: None,
                                tags: vec![],
                            }),
                        ],
                    }),
                },
            })),
            // Another system with nested container
            TopLevelItem::ElementDef(Box::new(ElementDef {
                location: dummy_location(),
                assignment: sruja_language::ElementAssignment {
                    location: dummy_location(),
                    name: "Frontend".to_string(),
                    kind: ElementKind::System,
                    sub_kind: None,
                    title: Some("Frontend System".to_string()),
                    tag_refs: vec![],
                    body: Some(sruja_language::ElementDefBody {
                        description: None,
                        technology: None,
                        metadata: vec![],
                        constraints: vec![],
                        conventions: vec![],
                        style: None,
                        scale: None,
                        slo: None,
                        items: vec![sruja_language::ElementDefBodyItem::ElementDef(Box::new(
                            ElementDef {
                                location: dummy_location(),
                                assignment: sruja_language::ElementAssignment {
                                    location: dummy_location(),
                                    name: "WebApp".to_string(),
                                    kind: ElementKind::Container,
                                    sub_kind: None,
                                    title: Some("Web Application".to_string()),
                                    tag_refs: vec![],
                                    body: None,
                                },
                            },
                        ))],
                    }),
                },
            })),
            // Cross-system relation with FQDN notation
            TopLevelItem::Relation(Relation {
                location: dummy_location(),
                from: QualifiedIdent::qualified(vec!["Frontend".to_string(), "WebApp".to_string()]),
                to: QualifiedIdent::qualified(vec!["Backend".to_string(), "API".to_string()]),
                label: Some("calls API".to_string()),
                description: None,
                technology: None,
                tags: vec![],
            }),
        ],
    };

    let mut config = super::DotConfig::default();
    config.view_level = 3; // Show all levels
    let exporter = DotExporter::new(config);

    let dot_output = exporter.export(&program);

    // Verify that the nested relation (API -> DB) is resolved to FQDNs
    assert!(
        dot_output.contains("\"Backend.API\" -> \"Backend.DB\""),
        "Nested relation should use resolved FQDNs Backend.API -> Backend.DB"
    );
    assert!(
        dot_output.contains("label=\"reads/writes\""),
        "Nested relation should have the label"
    );

    // Verify that cross-system relation uses FQDNs correctly
    assert!(
        dot_output.contains("\"Frontend.WebApp\" -> \"Backend.API\""),
        "Cross-system relation should use FQDNs Frontend.WebApp -> Backend.API"
    );
    assert!(
        dot_output.contains("label=\"calls API\""),
        "Cross-system relation should have the label"
    );

    // Verify cluster structure
    assert!(
        dot_output.contains("subgraph \"cluster_Backend\""),
        "Backend should be a cluster"
    );
    assert!(
        dot_output.contains("subgraph \"cluster_Frontend\""),
        "Frontend should be a cluster"
    );

    // Verify that cross-cluster edges have lhead/ltail attributes
    assert!(
        dot_output.contains("ltail=\"cluster_Frontend\""),
        "Cross-system edge should have ltail for Frontend cluster"
    );
    assert!(
        dot_output.contains("lhead=\"cluster_Backend\""),
        "Cross-system edge should have lhead for Backend cluster"
    );
}
