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
