use sruja_diagnostics::SourceLocation;

use super::*;

#[test]
fn test_program_new_creates_empty_program() {
    let program = Program::new();
    assert!(program.items.is_empty());
}

#[test]
fn test_program_default_creates_empty_program() {
    let program = Program::default();
    assert!(program.items.is_empty());
}

#[test]
fn test_program_with_items() {
    let items = vec![TopLevelItem::ElementDef(Box::new(ElementDef {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        assignment: ElementAssignment::new("test", ElementKind::System),
    }))];
    let program = Program::new().with_items(items.clone());
    assert_eq!(program.items.len(), 1);
    assert_eq!(program.items, items);
}

#[test]
fn test_program_push_item() {
    let mut program = Program::new();
    assert_eq!(program.items.len(), 0);

    let item = TopLevelItem::ElementDef(Box::new(ElementDef {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        assignment: ElementAssignment::new("test", ElementKind::System),
    }));
    program.push_item(item.clone());

    assert_eq!(program.items.len(), 1);
    assert_eq!(program.items[0], item);
}

#[test]
fn test_element_assignment_new_creates_with_defaults() {
    let assignment = ElementAssignment::new("my_system", ElementKind::System);
    assert_eq!(assignment.name, "my_system");
    assert_eq!(assignment.kind, ElementKind::System);
    assert!(assignment.sub_kind.is_none());
    assert!(assignment.title.is_none());
    assert!(assignment.tag_refs.is_empty());
    assert!(assignment.body.is_none());
    assert_eq!(assignment.location.file, "");
    assert_eq!(assignment.location.line, 0);
    assert_eq!(assignment.location.column, 0);
}

#[test]
fn test_element_assignment_new_with_different_kinds() {
    let kinds = vec![
        (ElementKind::Person, "user"),
        (ElementKind::System, "system"),
        (ElementKind::Container, "web"),
        (ElementKind::Component, "auth"),
        (ElementKind::Database, "db"),
        (ElementKind::Queue, "queue"),
        (ElementKind::ExternalSystem, "external"),
        (ElementKind::DataStore, "datastore"),
        (ElementKind::Policy, "policy"),
        (ElementKind::Requirement, "req"),
        (ElementKind::Adr, "adr"),
        (ElementKind::Flow, "flow"),
        (ElementKind::Scenario, "scenario"),
        (ElementKind::Story, "story"),
        (ElementKind::Custom("custom_kind".to_string()), "custom"),
    ];

    for (kind, name) in kinds {
        let assignment = ElementAssignment::new(name, kind.clone());
        assert_eq!(assignment.name, name);
        assert_eq!(assignment.kind, kind);
    }
}

#[test]
fn test_element_kind_display_standard_kinds() {
    assert_eq!(format!("{}", ElementKind::Person), "person");
    assert_eq!(format!("{}", ElementKind::Role), "role");
    assert_eq!(format!("{}", ElementKind::System), "system");
    assert_eq!(format!("{}", ElementKind::Container), "container");
    assert_eq!(format!("{}", ElementKind::Component), "component");
    assert_eq!(format!("{}", ElementKind::Database), "database");
    assert_eq!(format!("{}", ElementKind::Queue), "queue");
    assert_eq!(format!("{}", ElementKind::ExternalSystem), "externalSystem");
    assert_eq!(format!("{}", ElementKind::DataStore), "datastore");
}

#[test]
fn test_element_kind_display_governance_kinds() {
    assert_eq!(format!("{}", ElementKind::Policy), "policy");
    assert_eq!(format!("{}", ElementKind::Requirement), "requirement");
    assert_eq!(format!("{}", ElementKind::Adr), "adr");
    assert_eq!(format!("{}", ElementKind::Flow), "flow");
    assert_eq!(format!("{}", ElementKind::Scenario), "scenario");
    assert_eq!(format!("{}", ElementKind::Story), "story");
}

#[test]
fn test_element_kind_display_custom_kind() {
    let custom_kind = ElementKind::Custom("MyCustomKind".to_string());
    assert_eq!(format!("{}", custom_kind), "MyCustomKind");
}

#[test]
fn test_qualified_ident_simple() {
    let ident = QualifiedIdent::simple("test".to_string());
    assert_eq!(ident.parts, vec!["test"]);
    assert_eq!(ident.as_string(), "test");
}

#[test]
fn test_qualified_ident_qualified() {
    let ident = QualifiedIdent::qualified(vec![
        "module".to_string(),
        "submodule".to_string(),
        "item".to_string(),
    ]);
    assert_eq!(ident.parts, vec!["module", "submodule", "item"]);
    assert_eq!(ident.as_string(), "module.submodule.item");
}

#[test]
fn test_qualified_ident_as_string_single_part() {
    let ident = QualifiedIdent::simple("single".to_string());
    assert_eq!(ident.as_string(), "single");
}

#[test]
fn test_qualified_ident_as_string_multiple_parts() {
    let ident = QualifiedIdent::qualified(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    assert_eq!(ident.as_string(), "a.b.c");
}

#[test]
fn test_qualified_ident_display() {
    let ident = QualifiedIdent::qualified(vec!["foo".to_string(), "bar".to_string()]);
    assert_eq!(format!("{}", ident), "foo.bar");
}

#[test]
fn test_feedback_loop_type_to_symbol_reinforcing() {
    let loop_type = FeedbackLoopType::Reinforcing;
    assert_eq!(loop_type.to_symbol(), "+");
}

#[test]
fn test_feedback_loop_type_to_symbol_balancing() {
    let loop_type = FeedbackLoopType::Balancing;
    assert_eq!(loop_type.to_symbol(), "-");
}

#[test]
fn test_feedback_loop_type_display_reinforcing() {
    let loop_type = FeedbackLoopType::Reinforcing;
    assert_eq!(format!("{}", loop_type), "reinforcing");
}

#[test]
fn test_feedback_loop_type_display_balancing() {
    let loop_type = FeedbackLoopType::Balancing;
    assert_eq!(format!("{}", loop_type), "balancing");
}

#[test]
fn test_causal_polarity_to_symbol_positive() {
    let polarity = CausalPolarity::Positive;
    assert_eq!(polarity.to_symbol(), "+");
}

#[test]
fn test_causal_polarity_to_symbol_negative() {
    let polarity = CausalPolarity::Negative;
    assert_eq!(polarity.to_symbol(), "-");
}

#[test]
fn test_causal_polarity_display_positive() {
    let polarity = CausalPolarity::Positive;
    assert_eq!(format!("{}", polarity), "+");
}

#[test]
fn test_causal_polarity_display_negative() {
    let polarity = CausalPolarity::Negative;
    assert_eq!(format!("{}", polarity), "-");
}

#[test]
fn test_comprehensive_ast_construction() {
    let mut program = Program::new();

    let system_elem = ElementDef {
        location: SourceLocation::new("example.sruja".to_string(), 1, 1),
        assignment: ElementAssignment::new("MyApp", ElementKind::System),
    };
    program.push_item(TopLevelItem::ElementDef(Box::new(system_elem)));

    let container_elem = ElementDef {
        location: SourceLocation::new("example.sruja".to_string(), 5, 1),
        assignment: ElementAssignment::new("WebServer", ElementKind::Container),
    };
    program.push_item(TopLevelItem::ElementDef(Box::new(container_elem)));

    let relation = TopLevelItem::Relation(Relation {
        location: SourceLocation::new("example.sruja".to_string(), 10, 1),
        from: QualifiedIdent::simple("MyApp".to_string()),
        to: QualifiedIdent::simple("WebServer".to_string()),
        label: Some("HTTPS".to_string()),
        description: None,
        technology: None,
        tags: Vec::new(),
    });
    program.push_item(relation);

    assert_eq!(program.items.len(), 3);
}

#[test]
fn test_element_assignment_cloning() {
    let assignment = ElementAssignment::new("original", ElementKind::System);
    let cloned = assignment.clone();

    assert_eq!(assignment.name, cloned.name);
    assert_eq!(assignment.kind, cloned.kind);
}

#[test]
fn test_program_builder_pattern() {
    let program = Program::new().with_items(vec![
        TopLevelItem::ElementDef(Box::new(ElementDef {
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            assignment: ElementAssignment::new("A", ElementKind::System),
        })),
        TopLevelItem::ElementDef(Box::new(ElementDef {
            location: SourceLocation::new("test.sruja".to_string(), 2, 1),
            assignment: ElementAssignment::new("B", ElementKind::System),
        })),
    ]);

    assert_eq!(program.items.len(), 2);
}

#[test]
fn test_node_kind_parsing_and_display() {
    use std::str::FromStr;

    assert_eq!(
        NodeKind::from_str("system").unwrap().as_str(),
        NodeKind::SYSTEM
    );
    assert_eq!(
        NodeKind::from_str("service").unwrap().as_str(),
        NodeKind::SERVICE
    );
    assert_eq!(
        NodeKind::from_str("container").unwrap().as_str(),
        NodeKind::CONTAINER
    );
    assert_eq!(
        NodeKind::from_str("component").unwrap().as_str(),
        NodeKind::COMPONENT
    );
    assert_eq!(
        NodeKind::from_str("database").unwrap().as_str(),
        NodeKind::DATABASE
    );
    assert_eq!(
        NodeKind::from_str("queue").unwrap().as_str(),
        NodeKind::QUEUE
    );
    assert_eq!(
        NodeKind::from_str("external_api").unwrap().as_str(),
        NodeKind::EXTERNAL_API
    );
    assert_eq!(
        NodeKind::from_str("frontend").unwrap().as_str(),
        NodeKind::FRONTEND
    );
    assert_eq!(
        NodeKind::from_str("module").unwrap().as_str(),
        NodeKind::MODULE
    );

    let arbitrary = NodeKind::from_str("arbitrary_kind").unwrap();
    assert_eq!(arbitrary.as_str(), "arbitrary_kind");
    assert!(arbitrary.is_custom());

    assert_eq!(NodeKind::new(NodeKind::SYSTEM).as_str(), "system");
    assert_eq!(format!("{}", NodeKind::new(NodeKind::DATABASE)), "database");

    let custom = NodeKind::new("custom_node");
    assert_eq!(custom.kind_str(), "custom_node");
    assert!(custom.is_custom());
    assert_eq!(custom.to_string_kind(), "custom_node");
    assert_eq!(NodeKind::new(NodeKind::SYSTEM).to_string_kind(), "system");
}

#[test]
fn test_edge_kind_parsing_and_display() {
    use std::str::FromStr;

    assert_eq!(
        EdgeKind::from_str("depends_on").unwrap().as_str(),
        EdgeKind::DEPENDS_ON
    );
    assert_eq!(
        EdgeKind::from_str("calls").unwrap().as_str(),
        EdgeKind::CALLS
    );
    assert_eq!(
        EdgeKind::from_str("reads_from").unwrap().as_str(),
        EdgeKind::READS_FROM
    );
    assert_eq!(
        EdgeKind::from_str("writes_to").unwrap().as_str(),
        EdgeKind::WRITES_TO
    );
    assert_eq!(
        EdgeKind::from_str("publishes_to").unwrap().as_str(),
        EdgeKind::PUBLISHES_TO
    );
    assert_eq!(
        EdgeKind::from_str("subscribes_to").unwrap().as_str(),
        EdgeKind::SUBSCRIBES_TO
    );
    assert_eq!(EdgeKind::from_str("owns").unwrap().as_str(), EdgeKind::OWNS);
    assert_eq!(
        EdgeKind::from_str("contains").unwrap().as_str(),
        EdgeKind::CONTAINS
    );
    assert_eq!(EdgeKind::from_str("uses").unwrap().as_str(), EdgeKind::USES);

    let arbitrary = EdgeKind::from_str("arbitrary_edge").unwrap();
    assert_eq!(arbitrary.as_str(), "arbitrary_edge");
    assert!(arbitrary.is_custom());

    assert_eq!(EdgeKind::new(EdgeKind::CALLS).as_str(), "calls");
    assert_eq!(
        format!("{}", EdgeKind::new(EdgeKind::READS_FROM)),
        "reads_from"
    );

    let custom = EdgeKind::new("custom_edge");
    assert_eq!(custom.kind_str(), "custom_edge");
    assert!(custom.is_custom());
    assert_eq!(custom.to_string_kind(), "custom_edge");
    assert_eq!(EdgeKind::new(EdgeKind::CALLS).to_string_kind(), "calls");
}

#[test]
fn test_criticality_parsing_and_display() {
    use std::str::FromStr;

    assert_eq!(Criticality::from_str("low").unwrap(), Criticality::Low);
    assert_eq!(
        Criticality::from_str("medium").unwrap(),
        Criticality::Medium
    );
    assert_eq!(Criticality::from_str("med").unwrap(), Criticality::Medium);
    assert_eq!(Criticality::from_str("high").unwrap(), Criticality::High);
    assert_eq!(
        Criticality::from_str("critical").unwrap(),
        Criticality::Critical
    );
    assert!(Criticality::from_str("invalid_criticality").is_err());

    assert_eq!(Criticality::Low.as_str(), "low");
    assert_eq!(format!("{}", Criticality::High), "high");
}

#[test]
fn test_source_kind_parsing_and_display() {
    use std::str::FromStr;

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
