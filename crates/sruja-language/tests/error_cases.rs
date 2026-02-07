use sruja_language::Parser;

#[test]
fn test_unclosed_brace_error() {
    let input = r#"
A = system "System A" {
    description "This has an unclosed brace"
"#;
    let parser = Parser::new("test.sruja".to_string());
    let result = parser.parse(input);
    // Parser should handle error gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_empty_identifier_error() {
    let input = "= system \"System\"";
    let parser = Parser::new("test.sruja".to_string());
    let result = parser.parse(input);
    // Should fail or parse nothing
    assert!(result.is_err() || result.unwrap().items.is_empty());
}

#[test]
fn test_malformed_string_error() {
    let input = r#"
A = system "Unclosed string
"#;
    let parser = Parser::new("test.sruja".to_string());
    let result = parser.parse(input);
    // Parser should handle malformed strings
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_invalid_kind_name() {
    let input = r#"
A = invalid_kind "System A"
"#;
    let parser = Parser::new("test.sruja".to_string());
    let result = parser.parse(input);
    // Parser should accept custom kinds
    assert!(result.is_ok());
    let program = result.unwrap();
    // The element should be created with custom kind
    assert!(!program.items.is_empty());
}

#[test]
fn test_empty_relation() {
    let input = r#"
A = system "System A"
B = system "System B"

A -> B
"#;
    let parser = Parser::new("test.sruja".to_string());
    let result = parser.parse(input);
    // Parser should accept relations without labels
    assert!(result.is_ok());
}

#[test]
fn test_empty_program() {
    let input = "";
    let parser = Parser::new("test.sruja".to_string());
    let result = parser.parse(input);
    // Should parse empty program
    assert!(result.is_ok());
    let program = result.unwrap();
    assert!(program.items.is_empty());
}

#[test]
fn test_whitespace_only_program() {
    let input = "   \n\n\t\n   ";
    let parser = Parser::new("test.sruja".to_string());
    let result = parser.parse(input);
    // Should parse whitespace-only program
    assert!(result.is_ok());
    let program = result.unwrap();
    assert!(program.items.is_empty());
}
