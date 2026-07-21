use super::*;

#[test]
fn test_node_kind_parsing_and_display() {
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
