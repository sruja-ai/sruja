//! Test for edge rendering with lhead/ltail attributes

use sruja_language::{ElementDef, ElementKind, Program, QualifiedIdent, Relation, TopLevelItem};
use sruja_diagnostics::SourceLocation;
use super::DotExporter;

fn dummy_location() -> SourceLocation {
    SourceLocation::new("test.sruja".to_string(), 1, 1)
}

#[test]
fn test_cross_cluster_edge_attributes() {
    // Create a simple program with elements in different clusters
    let program = Program {
        items: vec![
            // Parent element (cluster)
            TopLevelItem::ElementDef(ElementDef {
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
            }),
            // Child element in cluster
            TopLevelItem::ElementDef(ElementDef {
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
            }),
            // Another parent element (different cluster)
            TopLevelItem::ElementDef(ElementDef {
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
            }),
            // Child element in second cluster
            TopLevelItem::ElementDef(ElementDef {
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
            }),
            // Relation between services in different clusters
            TopLevelItem::Relation(Relation {
                location: dummy_location(),
                from: QualifiedIdent::qualified(vec!["system1".to_string(), "service1".to_string()]),
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
    assert!(dot_output.contains("ltail=\"cluster_system1\""), 
            "Edge should have ltail attribute for source cluster");
    assert!(dot_output.contains("lhead=\"cluster_system2\""), 
            "Edge should have lhead attribute for target cluster");
    
    // Verify the edge structure
    assert!(dot_output.contains("\"system1.service1\" -> \"system2.service2\""), 
            "Edge should connect the two services");
    assert!(dot_output.contains("label=\"calls\""), 
            "Edge should have the label");
}

#[test]
fn test_same_cluster_edge_no_attributes() {
    // Create a program with elements in the same cluster
    let program = Program {
        items: vec![
            // Parent element (cluster)
            TopLevelItem::ElementDef(ElementDef {
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
            }),
            // Child element in cluster
            TopLevelItem::ElementDef(ElementDef {
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
            }),
            // Another child element in same cluster
            TopLevelItem::ElementDef(ElementDef {
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
            }),
            // Relation between services in the same cluster
            TopLevelItem::Relation(Relation {
                location: dummy_location(),
                from: QualifiedIdent::qualified(vec!["system1".to_string(), "service1".to_string()]),
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
    assert!(!dot_output.contains("ltail=\"cluster_system1\""), 
            "Same-cluster edge should NOT have ltail attribute");
    assert!(!dot_output.contains("lhead=\"cluster_system1\""), 
            "Same-cluster edge should NOT have lhead attribute");
    
    // Verify the edge still exists
    assert!(dot_output.contains("\"system1.service1\" -> \"system1.service2\""), 
            "Edge should connect the two services");
    assert!(dot_output.contains("label=\"calls\""), 
            "Edge should have the label");
}
