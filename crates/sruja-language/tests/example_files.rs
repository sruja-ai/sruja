// Integration tests for example/valid .sruja constructs (overview, views, etc.).
// See error_cases.rs for error-handling tests.

use sruja_language::Parser;

#[test]
fn test_parse_simple_system() {
    let input = r#"
A = system "System A"
B = system "System B"
A -> B
"#;
    let parser = Parser::new("example.sruja".to_string());
    let result = parser.parse(input);
    assert!(result.is_ok());
    let program = result.unwrap();
    assert!(!program.items.is_empty());
}

#[test]
fn test_parse_with_description() {
    let input = r#"
A = system "System A" {
    description "An example system."
}
"#;
    let parser = Parser::new("example.sruja".to_string());
    let result = parser.parse(input);
    assert!(result.is_ok());
}
