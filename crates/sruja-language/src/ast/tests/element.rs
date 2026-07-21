use super::*;

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
fn test_element_assignment_cloning() {
    let assignment = ElementAssignment::new("original", ElementKind::System);
    let cloned = assignment.clone();

    assert_eq!(assignment.name, cloned.name);
    assert_eq!(assignment.kind, cloned.kind);
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
    assert_eq!(
        assignment.body.unwrap().description,
        Some("A test system".to_string())
    );
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

    assert_eq!(
        kind_def.kind,
        ElementKind::Custom("microservice".to_string())
    );
    assert_eq!(kind_def.title, Some("Microservice".to_string()));
}
