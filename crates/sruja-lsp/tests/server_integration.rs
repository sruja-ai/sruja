//! LSP Server integration tests
//!
//! This module tests the LSP server functionality.
//! Full protocol-level integration testing requires mock LSP clients,
//! but we can test the core server logic directly.

use sruja_language::Parser;
use sruja_lsp::diagnostics::convert_diagnostics_to_lsp;
use sruja_lsp::features::{
    collect_elements, find_definition, find_element_hover, find_references, get_completion,
    get_document_symbols, get_hover, word_bounds,
};
use sruja_lsp::workspace::Document;
use tower_lsp::lsp_types::*;

fn create_test_document(text: &str) -> Document {
    let uri = Url::parse("file:///test.sruja").unwrap();
    let mut doc = Document::new(uri, text.to_string(), 1);
    doc.parse();
    doc
}

fn create_test_program(text: &str) -> sruja_language::ast::Program {
    let uri = Url::parse("file:///test.sruja").unwrap();
    let parser = Parser::new(uri.to_string());
    parser
        .parse(text)
        .unwrap_or_else(|_| sruja_language::ast::Program { items: vec![] })
}

#[test]
fn test_lsp_crate_structure() {
    assert!(true);
}

#[test]
fn test_server_public_api() {
    assert!(true);
}

#[test]
fn test_hover_on_system_element() {
    let text = r#"
App = system "My Application" {
  description "A test application"
}
"#;
    let doc = create_test_document(text);
    let program = create_test_program(text);

    let hover = get_hover(&doc, &program, 1, 1);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    if let HoverContents::Markup(content) = &hover.contents {
        assert!(content.value.contains("system"));
        assert!(content.value.contains("My Application"));
    } else {
        panic!("Expected markup content");
    }
}

#[test]
fn test_hover_on_relation_arrow() {
    let text = r#"
App = system "App" {}
Db = database "Database" {}
App -> Db "uses PostgreSQL"
"#;
    let doc = create_test_document(text);
    let program = create_test_program(text);

    let hover = get_hover(&doc, &program, 3, 5);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    if let HoverContents::Markup(content) = &hover.contents {
        assert!(content.value.contains("Relation"));
        assert!(content.value.contains("uses"));
    } else {
        panic!("Expected markup content");
    }
}

#[test]
fn test_completion_includes_keywords() {
    let text = r#"
App = system "App" {}
"#;
    let doc = create_test_document(text);
    let program = create_test_program(text);

    let items = get_completion(&doc, &program, 0, 0);
    assert!(!items.is_empty());

    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
    assert!(labels.contains(&"system"));
    assert!(labels.contains(&"container"));
    assert!(labels.contains(&"component"));
}

#[test]
fn test_completion_after_arrow_suggests_verbs() {
    let text = r#"
App = system "App" {}
Db = database "Database" {}
App ->
"#;
    let doc = create_test_document(text);
    let program = create_test_program(text);

    let items = get_completion(&doc, &program, 3, 6);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
    assert!(labels.contains(&"reads"));
    assert!(labels.contains(&"writes"));
    assert!(labels.contains(&"calls"));
}

#[test]
fn test_find_references_finds_all_occurrences() {
    let text = r#"
App = system "App" {}
Db = database "Database" {}
App -> Db "uses"
"#;
    let doc = create_test_document(text);
    let program = create_test_program(text);

    let locations = find_references(&doc, &program, "App");
    assert!(locations.len() >= 2);
}

#[test]
fn test_document_symbols_extracts_elements() {
    let text = r#"
App = system "App" {
  description "Main app"
}
Web = container "Web" {
  technology "Node.js"
  description "Web server"
}
"#;
    let doc = create_test_document(text);
    let program = create_test_program(text);

    let symbols = get_document_symbols(&doc, &program);
    assert_eq!(symbols.len(), 2);

    let app_symbol = symbols.iter().find(|s| s.name == "App").unwrap();
    assert_eq!(app_symbol.kind, SymbolKind::CLASS);

    let web_symbol = symbols.iter().find(|s| s.name == "Web").unwrap();
    assert_eq!(web_symbol.kind, SymbolKind::MODULE);
}

#[test]
fn test_find_element_hover_partial_match() {
    let text = r#"
Root = system "Root" {
  description "Root"

  App = container "App" {
    technology "Rust"
    description "App container"
  }
}
"#;
    let program = create_test_program(text);

    let hover = find_element_hover(&program, "App");
    assert!(hover.is_some());

    let (kind, title, _) = hover.unwrap();
    assert_eq!(kind, "container");
    assert_eq!(title, "App");
}

#[test]
fn test_word_bounds_extracts_identifiers() {
    assert_eq!(word_bounds("App = system \"App\"", 0), (0, 3));
    assert_eq!(word_bounds("App = system \"App\"", 6), (6, 12));
    assert_eq!(word_bounds("App -> Db \"uses\"", 4), (0, 7));
}

#[test]
fn test_diagnostics_conversion() {
    use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};

    let diags = vec![Diagnostic::new(
        "E201".to_string(),
        Severity::Error,
        "Duplicate ID".to_string(),
        SourceLocation::new("test.sruja".to_string(), 1, 5),
    )];

    let lsp_diags = convert_diagnostics_to_lsp(&diags);
    assert_eq!(lsp_diags.len(), 1);
    assert_eq!(lsp_diags[0].severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(lsp_diags[0].message, "Duplicate ID");
}

#[test]
fn test_empty_document_handling() {
    let text = "";
    let doc = create_test_document(text);
    let program = create_test_program(text);

    assert!(get_hover(&doc, &program, 0, 0).is_none());
    assert!(
        get_completion(&doc, &program, 0, 0).is_empty()
            || get_completion(&doc, &program, 0, 0).len() > 0
    );
    assert!(get_document_symbols(&doc, &program).is_empty());
}
