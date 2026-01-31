//! Sruja Language Processing Library
//!
//! This crate provides parsing, AST representation, and traversal utilities
//! for the Sruja DSL.

pub mod ast;
pub mod parser;
pub mod token;
pub mod traversal;

pub use ast::*;
pub use parser::Parser;
pub use traversal::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_system() {
        let input = r#"
system MySystem "My System" {
    description "A test system"
}
"#;
        let parser = Parser::new("test.sruja".to_string());
        let result = parser.parse(input);
        assert!(result.is_ok(), "Should parse simple system");
    }

    #[test]
    fn test_parse_relation() {
        let input = r#"
system A "System A"
system B "System B"
A -> B "calls"
"#;
        let parser = Parser::new("test.sruja".to_string());
        let result = parser.parse(input);
        assert!(result.is_ok(), "Should parse relation");
    }

    #[test]
    fn test_parse_nested_elements() {
        let input = r#"
system MySystem "My System" {
    container API "API Container" {
        component Handler "Request Handler"
    }
}
"#;
        let parser = Parser::new("test.sruja".to_string());
        let result = parser.parse(input);
        assert!(result.is_ok(), "Should parse nested elements");
    }

    #[test]
    fn test_fqdn_edge_resolution() {
        let input = r#"
Backend = system "Backend System" {
    API = container "API Service"
    DB = container "Database"

    API -> DB "reads/writes"
}

Frontend = system "Frontend System" {
    WebApp = container "Web Application"
}

Frontend.WebApp -> Backend.API "calls API"
"#;
        let parser = Parser::new("test.sruja".to_string());
        let result = parser.parse(input);
        assert!(result.is_ok(), "Should parse system with nested relations");

        let program = result.unwrap();
        let (elements, relations) = collect_elements(&program);

        // Check that elements are stored with FQDNs
        assert!(
            elements.contains_key("Backend"),
            "Backend system should exist"
        );
        assert!(
            elements.contains_key("Backend.API"),
            "Backend.API container should exist"
        );
        assert!(
            elements.contains_key("Backend.DB"),
            "Backend.DB container should exist"
        );
        assert!(
            elements.contains_key("Frontend"),
            "Frontend system should exist"
        );
        assert!(
            elements.contains_key("Frontend.WebApp"),
            "Frontend.WebApp container should exist"
        );

        // Check that relations have resolved FQDNs
        assert_eq!(relations.len(), 2, "Should have 2 relations");

        // Find the relation: API -> DB (defined inside Backend body)
        let rel1 = relations
            .iter()
            .find(|r| r.label.as_deref() == Some("reads/writes"))
            .expect("Should find 'reads/writes' relation");
        assert_eq!(
            rel1.from.as_string(),
            "Backend.API",
            "Relation 'reads/writes' from should be resolved to Backend.API"
        );
        assert_eq!(
            rel1.to.as_string(),
            "Backend.DB",
            "Relation 'reads/writes' to should be resolved to Backend.DB"
        );

        // Find the relation: Frontend.WebApp -> Backend.API (defined with FQDN)
        let rel2 = relations
            .iter()
            .find(|r| r.label.as_deref() == Some("calls API"))
            .expect("Should find 'calls API' relation");
        assert_eq!(
            rel2.from.as_string(),
            "Frontend.WebApp",
            "Relation 'calls API' from should be Frontend.WebApp"
        );
        assert_eq!(
            rel2.to.as_string(),
            "Backend.API",
            "Relation 'calls API' to should be Backend.API"
        );
    }
}
