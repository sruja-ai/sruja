//! Tests for LSP features.

use crate::workspace::Document;
use sruja_language::ast::Program;
use sruja_language::Parser;
use tower_lsp::lsp_types::{SymbolKind, Url};

use super::*;

fn create_test_document(text: &str) -> Document {
    let uri = Url::parse("file:///test.sruja").unwrap();
    let mut doc = Document::new(uri, text.to_string(), 1);
    doc.parse();
    doc
}

fn create_test_program(text: &str) -> Program {
    let uri = Url::parse("file:///test.sruja").unwrap();
    let parser = Parser::new(uri.to_string());
    parser
        .parse(text)
        .unwrap_or_else(|_| Program { items: vec![] })
}

#[test]
fn test_word_bounds_middle() {
    let line = "hello world";
    let bounds = word_bounds(line, 6);
    assert_eq!(bounds, (6, 11));
}

#[test]
fn test_word_bounds_start() {
    let line = "hello world";
    let bounds = word_bounds(line, 0);
    assert_eq!(bounds, (0, 5));
}

#[test]
fn test_word_bounds_end() {
    let line = "hello world";
    let bounds = word_bounds(line, 11);
    assert_eq!(bounds, (6, 11));
}

#[test]
fn test_word_bounds_with_delimiter() {
    let line = "app->web";
    let bounds = word_bounds(line, 2);
    assert_eq!(bounds, (0, 4));
}

#[test]
fn test_word_bounds_empty_line() {
    let line = "";
    let bounds = word_bounds(line, 0);
    assert_eq!(bounds, (0, 0));
}

#[test]
fn test_is_ident_char() {
    assert!(is_ident_char('a'));
    assert!(is_ident_char('Z'));
    assert!(is_ident_char('0'));
    assert!(is_ident_char('_'));
    assert!(is_ident_char('-'));
    assert!(is_ident_char('.'));
    assert!(!is_ident_char(' '));
    assert!(!is_ident_char('{'));
    assert!(!is_ident_char('}'));
}

#[test]
fn test_last_token() {
    assert_eq!(last_token("app = system"), "system");
    assert_eq!(last_token("app ="), "app");
    assert_eq!(last_token("app"), "app");
    assert_eq!(last_token(""), "");
    assert_eq!(last_token("  "), "");
}

#[test]
fn test_find_element_hover() {
    let program = create_test_program(
        r#"
app = system "My App" {
  description "Test app"
}
"#,
    );
    let hover = find_element_hover(&program, "app");
    assert!(hover.is_some());
    let (kind, title) = hover.unwrap();
    assert_eq!(kind, "system");
    assert_eq!(title, "My App");
}

#[test]
fn test_find_element_hover_not_found() {
    let program = create_test_program(
        r#"
app = system "My App" {}
"#,
    );
    let hover = find_element_hover(&program, "nonexistent");
    assert!(hover.is_none());
}

#[test]
fn test_find_relation_hover() {
    let program = create_test_program(
        r#"
app = system "My App" {}
web = container "Web" {}
app -> web "HTTP"
"#,
    );
    let hover = find_relation_hover(&program, "app", "web");
    assert!(hover.is_some());
    let (_verb, label) = hover.unwrap();
    assert_eq!(label, "HTTP");
}

#[test]
fn test_get_hover_on_element() {
    let text = r#"
app = system "My App" {}
"#;
    let doc = create_test_document(text);
    let program = create_test_program(text);
    let hover = get_hover(&doc, &program, 1, 1);
    assert!(hover.is_some());
    assert!(hover.unwrap().range.is_some());
}

#[test]
fn test_get_hover_on_relation() {
    let text = r#"
app = system "My App" {}
web = container "Web" {}
app -> web "HTTP"
"#;
    let doc = create_test_document(text);
    let program = create_test_program(text);
    let hover = get_hover(&doc, &program, 3, 2);
    assert!(hover.is_some());
}

#[test]
fn test_get_hover_no_match() {
    let text = r#"
app = system "My App" {}
"#;
    let doc = create_test_document(text);
    let program = create_test_program(text);
    let hover = get_hover(&doc, &program, 10, 0);
    assert!(hover.is_none());
}

#[test]
fn test_get_completion_keywords() {
    let text = r#"
app = system "My App" {}
"#;
    let doc = create_test_document(text);
    let program = create_test_program(text);
    let items = get_completion(&doc, &program, 1, 0);
    assert!(!items.is_empty());
    assert!(items.iter().any(|i| i.label == "system"));
    assert!(items.iter().any(|i| i.label == "container"));
}

#[test]
fn test_get_completion_filtered() {
    let text = r#"
app = system "My App" {}
"#;
    let doc = create_test_document(text);
    let program = create_test_program(text);
    let items = get_completion(&doc, &program, 1, 7);
    assert!(!items.is_empty());
    assert!(items.iter().any(|i| i.label == "system"));
    assert!(!items.iter().any(|i| i.label == "container"));
}

#[test]
fn test_get_completion_after_arrow() {
    let text = r#"
app = system "My App" {}
web = container "Web" {}
app ->
"#;
    let doc = create_test_document(text);
    let program = create_test_program(text);
    let items = get_completion(&doc, &program, 3, 6);
    assert!(!items.is_empty());
    assert!(items.iter().any(|i| i.label == "reads"));
}

#[test]
fn test_find_definition() {
    let text = r#"
app = system "My App" {}
web = container "Web" {}
app -> web "HTTP"
"#;
    let doc = create_test_document(text);
    let program = create_test_program(text);
    let location = find_definition(&doc, &program, "app");
    let _ = location;
    let nonexistent = find_definition(&doc, &program, "nonexistent");
    assert!(nonexistent.is_none());
}

#[test]
fn test_find_definition_not_found() {
    let text = r#"
app = system "My App" {}
"#;
    let doc = create_test_document(text);
    let program = create_test_program(text);
    let location = find_definition(&doc, &program, "nonexistent");
    assert!(location.is_none());
}

#[test]
fn test_find_references() {
    let text = r#"
app = system "My App" {}
web = container "Web" {}
app -> web "HTTP"
"#;
    let doc = create_test_document(text);
    let program = create_test_program(text);
    let locations = find_references(&doc, &program, "app");
    assert!(!locations.is_empty());
    assert!(locations.len() >= 2);
}

#[test]
fn test_find_references_unique() {
    let text = r#"
app = system "My App" {}
app2 = system "App 2" {}
app -> app2 "HTTP"
"#;
    let doc = create_test_document(text);
    let program = create_test_program(text);
    let locations = find_references(&doc, &program, "app");
    assert_eq!(locations.len(), 2);
}

#[test]
fn test_get_document_symbols() {
    let text = r#"
app = system "My App" {
  description "Test"
}
web = container "Web" {}
"#;
    let doc = create_test_document(text);
    let program = create_test_program(text);
    let symbols = get_document_symbols(&doc, &program);
    assert!(!symbols.is_empty());
    assert_eq!(symbols.len(), 2);
    let app_symbol = symbols.iter().find(|s| s.name == "app");
    assert!(app_symbol.is_some());
    assert_eq!(app_symbol.unwrap().kind, SymbolKind::CLASS);
    let web_symbol = symbols.iter().find(|s| s.name == "web");
    assert!(web_symbol.is_some());
    assert_eq!(web_symbol.unwrap().kind, SymbolKind::MODULE);
}

#[test]
fn test_get_document_symbols_empty() {
    let text = "# comment only";
    let doc = create_test_document(text);
    let program = create_test_program(text);
    let symbols = get_document_symbols(&doc, &program);
    assert_eq!(symbols.len(), 0);
}

#[test]
fn test_format_document() {
    let text = r#"
app = system "My App" {}
"#;
    let doc = create_test_document(text);
    let program = create_test_program(text);
    let edits = format_document(&doc, &program);
    assert!(edits.is_some());
    let edit_vec = edits.unwrap();
    assert_eq!(edit_vec.len(), 1);
    assert!(!edit_vec[0].new_text.is_empty());
}

#[test]
fn test_format_document_empty() {
    let text = "";
    let doc = create_test_document(text);
    let program = create_test_program(text);
    let edits = format_document(&doc, &program);
    assert!(edits.is_some());
}

#[test]
fn test_collect_elements() {
    let program = create_test_program(
        r#"
app = system "My App" {}
web = container "Web" {}
app -> web "HTTP"
"#,
    );
    let (elements, relations) = collect_elements(&program);
    assert_eq!(elements.len(), 2);
    assert_eq!(relations.len(), 1);
    assert!(elements.contains_key("app"));
    assert!(elements.contains_key("web"));
}

#[test]
fn test_collect_elements_empty() {
    let program = create_test_program("# comment");
    let (elements, relations) = collect_elements(&program);
    assert_eq!(elements.len(), 0);
    assert_eq!(relations.len(), 0);
}
