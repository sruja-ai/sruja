use super::*;

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
