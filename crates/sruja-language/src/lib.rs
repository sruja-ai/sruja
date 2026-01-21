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
}
