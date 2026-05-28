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

#[test]
fn test_parse_overview_and_view_in_program() {
    let input = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

overview {
  summary "Test system"
  goals ["Deliver value"]
}

User = person "User" {
  description "Actor"
}

App = system "App" {
  description "Application"
  Web = container "Web" {
    technology "React"
    description "UI"
  }
}

view Diagram of App {
  title "Containers"
  include *
}
"#;
    let parser = Parser::new("example.sruja".to_string());
    let program = parser
        .parse(input)
        .expect("parse program with overview and view");
    use sruja_language::TopLevelItem;
    assert!(program
        .items
        .iter()
        .any(|i| matches!(i, TopLevelItem::Overview(_))));
    assert!(program
        .items
        .iter()
        .any(|i| matches!(i, TopLevelItem::View(_))));
}
