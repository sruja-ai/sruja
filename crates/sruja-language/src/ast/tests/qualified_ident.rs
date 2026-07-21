use super::*;

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
