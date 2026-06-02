use std::str::FromStr;
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

#[test]
fn test_node_kind_equality() {
    let kind1 = NodeKind::new("system");
    let kind2 = NodeKind::new("system");
    let kind3 = NodeKind::new("container");

    assert_eq!(kind1, kind2);
    assert_ne!(kind1, kind3);
}

#[test]
fn test_node_kind_from_string() {
    let kind: NodeKind = "test".to_string().into();
    assert_eq!(kind.as_str(), "test");
}

#[test]
fn test_node_kind_from_str_ref() {
    let kind: NodeKind = "test".into();
    assert_eq!(kind.as_str(), "test");
}

#[test]
fn test_node_kind_partial_eq_str() {
    let kind = NodeKind::new("system");
    assert_eq!(kind, "system");
    assert_eq!(kind, "system".to_string());
    assert_ne!(kind, "container");
}

#[test]
fn test_node_kind_is_custom() {
    assert!(!NodeKind::new("system").is_custom());
    assert!(!NodeKind::new("service").is_custom());
    assert!(!NodeKind::new("container").is_custom());
    assert!(!NodeKind::new("component").is_custom());
    assert!(!NodeKind::new("database").is_custom());
    assert!(!NodeKind::new("queue").is_custom());
    assert!(!NodeKind::new("external_api").is_custom());
    assert!(!NodeKind::new("frontend").is_custom());
    assert!(!NodeKind::new("module").is_custom());
    assert!(NodeKind::new("custom").is_custom());
    assert!(NodeKind::new("microservice").is_custom());
}

#[test]
fn test_edge_kind_equality() {
    let kind1 = EdgeKind::new("calls");
    let kind2 = EdgeKind::new("calls");
    let kind3 = EdgeKind::new("reads_from");

    assert_eq!(kind1, kind2);
    assert_ne!(kind1, kind3);
}

#[test]
fn test_edge_kind_from_string() {
    let kind: EdgeKind = "test".to_string().into();
    assert_eq!(kind.as_str(), "test");
}

#[test]
fn test_edge_kind_from_str_ref() {
    let kind: EdgeKind = "test".into();
    assert_eq!(kind.as_str(), "test");
}

#[test]
fn test_edge_kind_partial_eq_str() {
    let kind = EdgeKind::new("calls");
    assert_eq!(kind, "calls");
    assert_eq!(kind, "calls".to_string());
    assert_ne!(kind, "reads_from");
}

#[test]
fn test_edge_kind_is_custom() {
    assert!(!EdgeKind::new("depends_on").is_custom());
    assert!(!EdgeKind::new("calls").is_custom());
    assert!(!EdgeKind::new("reads_from").is_custom());
    assert!(!EdgeKind::new("writes_to").is_custom());
    assert!(!EdgeKind::new("publishes_to").is_custom());
    assert!(!EdgeKind::new("subscribes_to").is_custom());
    assert!(!EdgeKind::new("owns").is_custom());
    assert!(!EdgeKind::new("contains").is_custom());
    assert!(!EdgeKind::new("uses").is_custom());
    assert!(EdgeKind::new("custom").is_custom());
    assert!(EdgeKind::new("sends_to").is_custom());
}

#[test]
fn test_criticality_display() {
    assert_eq!(format!("{}", Criticality::Low), "low");
    assert_eq!(format!("{}", Criticality::Medium), "medium");
    assert_eq!(format!("{}", Criticality::High), "high");
    assert_eq!(format!("{}", Criticality::Critical), "critical");
}

#[test]
fn test_criticality_as_str() {
    assert_eq!(Criticality::Low.as_str(), "low");
    assert_eq!(Criticality::Medium.as_str(), "medium");
    assert_eq!(Criticality::High.as_str(), "high");
    assert_eq!(Criticality::Critical.as_str(), "critical");
}

#[test]
fn test_criticality_from_str_case_insensitive() {
    assert_eq!(Criticality::from_str("LOW").unwrap(), Criticality::Low);
    assert_eq!(Criticality::from_str("High").unwrap(), Criticality::High);
    assert_eq!(Criticality::from_str("CRITICAL").unwrap(), Criticality::Critical);
}

#[test]
fn test_criticality_from_str_invalid() {
    assert!(Criticality::from_str("invalid").is_err());
    assert!(Criticality::from_str("").is_err());
    assert!(Criticality::from_str("medium_high").is_err());
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
    assert_eq!(format!("{}", SourceKind::Custom("custom".to_string())), "custom");
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
    use std::str::FromStr;
    assert_eq!(SourceKind::from_str("openapi").unwrap(), SourceKind::OpenApi);
    assert_eq!(SourceKind::from_str("asyncapi").unwrap(), SourceKind::AsyncApi);
    assert_eq!(SourceKind::from_str("kubernetes").unwrap(), SourceKind::Kubernetes);
    assert_eq!(SourceKind::from_str("k8s").unwrap(), SourceKind::Kubernetes);
    assert_eq!(SourceKind::from_str("dockerfile").unwrap(), SourceKind::Dockerfile);
    assert_eq!(SourceKind::from_str("docker").unwrap(), SourceKind::Dockerfile);
    assert_eq!(SourceKind::from_str("terraform").unwrap(), SourceKind::Terraform);
    assert_eq!(SourceKind::from_str("tf").unwrap(), SourceKind::Terraform);
    assert_eq!(SourceKind::from_str("docs").unwrap(), SourceKind::Docs);
    assert_eq!(SourceKind::from_str("doc").unwrap(), SourceKind::Docs);
    assert_eq!(SourceKind::from_str("readme").unwrap(), SourceKind::Readme);
    assert_eq!(SourceKind::from_str("proto").unwrap(), SourceKind::Proto);
    assert_eq!(SourceKind::from_str("protobuf").unwrap(), SourceKind::Proto);
    assert_eq!(SourceKind::from_str("config").unwrap(), SourceKind::Config);
    assert_eq!(SourceKind::from_str("graphql").unwrap(), SourceKind::GraphQL);
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

#[test]
fn test_relation_creation() {
    let relation = Relation {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        from: QualifiedIdent::simple("A".to_string()),
        to: QualifiedIdent::simple("B".to_string()),
        label: Some("calls".to_string()),
        description: Some("A calls B".to_string()),
        technology: Some("HTTP".to_string()),
        tags: vec!["api".to_string()],
    };

    assert_eq!(relation.from.as_string(), "A");
    assert_eq!(relation.to.as_string(), "B");
    assert_eq!(relation.label, Some("calls".to_string()));
    assert_eq!(relation.description, Some("A calls B".to_string()));
    assert_eq!(relation.technology, Some("HTTP".to_string()));
    assert_eq!(relation.tags, vec!["api".to_string()]);
}

#[test]
fn test_relation_clone() {
    let relation = Relation {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        from: QualifiedIdent::simple("A".to_string()),
        to: QualifiedIdent::simple("B".to_string()),
        label: None,
        description: None,
        technology: None,
        tags: Vec::new(),
    };

    let cloned = relation.clone();
    assert_eq!(relation, cloned);
}

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
fn test_qualified_ident_clone() {
    let ident = QualifiedIdent::qualified(vec!["a".to_string(), "b".to_string()]);
    let cloned = ident.clone();
    assert_eq!(ident, cloned);
}

#[test]
fn test_qualified_ident_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(QualifiedIdent::simple("test".to_string()));
    assert!(set.contains(&QualifiedIdent::simple("test".to_string())));
}

#[test]
fn test_element_def_body_default() {
    let body = ElementDefBody::default();
    assert!(body.description.is_none());
    assert!(body.technology.is_none());
    assert!(body.doc.is_none());
    assert!(body.knowledge.is_none());
    assert!(body.metadata.is_empty());
    assert!(body.constraints.is_empty());
    assert!(body.conventions.is_empty());
    assert!(body.style.is_none());
    assert!(body.scale.is_none());
    assert!(body.slo.is_none());
    assert!(body.items.is_empty());
    assert!(body.gotchas.is_empty());
    assert!(body.operational_constraints.is_empty());
    assert!(body.runbooks.is_empty());
    assert!(body.canonical_id.is_none());
    assert!(body.aliases.is_empty());
    assert!(body.owner.is_none());
    assert!(body.domain.is_none());
    assert!(body.criticality.is_none());
    assert!(body.sources.is_empty());
    assert!(body.state_machines.is_empty());
    assert!(body.contracts.is_empty());
    assert!(body.fitness_functions.is_empty());
}

#[test]
fn test_element_def_body_clone() {
    let body = ElementDefBody {
        description: Some("test".to_string()),
        technology: Some("Rust".to_string()),
        ..Default::default()
    };

    let cloned = body.clone();
    assert_eq!(body, cloned);
}

#[test]
fn test_element_assignment_with_body() {
    let mut assignment = ElementAssignment::new("test", ElementKind::System);
    assignment.body = Some(ElementDefBody {
        description: Some("A test system".to_string()),
        technology: Some("Rust".to_string()),
        ..Default::default()
    });

    assert!(assignment.body.is_some());
    assert_eq!(assignment.body.unwrap().description, Some("A test system".to_string()));
}

#[test]
fn test_element_assignment_with_tags() {
    let mut assignment = ElementAssignment::new("test", ElementKind::System);
    assignment.tag_refs = vec!["tag1".to_string(), "tag2".to_string()];

    assert_eq!(assignment.tag_refs.len(), 2);
    assert!(assignment.tag_refs.contains(&"tag1".to_string()));
}

#[test]
fn test_element_assignment_with_sub_kind() {
    let mut assignment = ElementAssignment::new("test", ElementKind::Container);
    assignment.sub_kind = Some("microservice".to_string());

    assert_eq!(assignment.sub_kind, Some("microservice".to_string()));
}

#[test]
fn test_element_assignment_with_title() {
    let mut assignment = ElementAssignment::new("test", ElementKind::System);
    assignment.title = Some("My System".to_string());

    assert_eq!(assignment.title, Some("My System".to_string()));
}

#[test]
fn test_system_creation() {
    let system = System {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        id: "MySystem".to_string(),
        label: Some("My System".to_string()),
        description: Some("A test system".to_string()),
        items: Vec::new(),
    };

    assert_eq!(system.id, "MySystem");
    assert_eq!(system.label, Some("My System".to_string()));
    assert_eq!(system.description, Some("A test system".to_string()));
    assert!(system.items.is_empty());
}

#[test]
fn test_container_creation() {
    let container = Container {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        id: "WebServer".to_string(),
        label: Some("Web Server".to_string()),
        description: Some("Handles HTTP requests".to_string()),
        technology: Some("Node.js".to_string()),
        items: Vec::new(),
    };

    assert_eq!(container.id, "WebServer");
    assert_eq!(container.technology, Some("Node.js".to_string()));
}

#[test]
fn test_component_creation() {
    let component = Component {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        id: "AuthService".to_string(),
        label: Some("Auth Service".to_string()),
        description: Some("Handles authentication".to_string()),
        technology: Some("JWT".to_string()),
    };

    assert_eq!(component.id, "AuthService");
    assert_eq!(component.technology, Some("JWT".to_string()));
}

#[test]
fn test_extend_element_creation() {
    let extend = ExtendElement {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        target: QualifiedIdent::simple("MySystem".to_string()),
        assignments: vec![ElementAssignment::new("extra", ElementKind::Component)],
    };

    assert_eq!(extend.target.as_string(), "MySystem");
    assert_eq!(extend.assignments.len(), 1);
}

#[test]
fn test_element_kind_def_creation() {
    let kind_def = ElementKindDef {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        kind: ElementKind::Custom("microservice".to_string()),
        title: Some("Microservice".to_string()),
        description: Some("A microservice component".to_string()),
        technology: None,
        style: None,
    };

    assert_eq!(kind_def.kind, ElementKind::Custom("microservice".to_string()));
    assert_eq!(kind_def.title, Some("Microservice".to_string()));
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
fn test_feedback_loop_creation() {
    let feedback_loop = FeedbackLoop {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        id: "loop1".to_string(),
        loop_type: FeedbackLoopType::Reinforcing,
        loop_id: Some("growth".to_string()),
        title: "Growth Loop".to_string(),
        description: Some("Positive feedback loop".to_string()),
        relationships: Vec::new(),
    };

    assert_eq!(feedback_loop.id, "loop1");
    assert_eq!(feedback_loop.loop_type, FeedbackLoopType::Reinforcing);
    assert_eq!(feedback_loop.title, "Growth Loop");
}

#[test]
fn test_causal_loop_creation() {
    let causal_loop = CausalLoop {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        id: "cl1".to_string(),
        loop_type: FeedbackLoopType::Balancing,
        loop_id: Some("stability".to_string()),
        title: "Stability Loop".to_string(),
        description: Some("Balancing loop".to_string()),
        variables: vec![CausalLoopVariable {
            id: "var1".to_string(),
            label: Some("Variable 1".to_string()),
        }],
        relationships: vec![CausalRelationship {
            from: "var1".to_string(),
            to: "var2".to_string(),
            effect: Some("increases".to_string()),
            polarity: CausalPolarity::Positive,
            delay: Some("1h".to_string()),
        }],
    };

    assert_eq!(causal_loop.id, "cl1");
    assert_eq!(causal_loop.variables.len(), 1);
    assert_eq!(causal_loop.relationships.len(), 1);
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
fn test_incremental_parse_result_creation() {
    let result = IncrementalParseResult {
        updated_ast: Program::new(),
        changed_elements: vec!["element1".to_string()],
        changed_ranges: vec![(0, 10)],
        parsing_time_ms: 50,
    };

    assert!(result.updated_ast.items.is_empty());
    assert_eq!(result.changed_elements.len(), 1);
    assert_eq!(result.changed_ranges.len(), 1);
    assert_eq!(result.parsing_time_ms, 50);
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
