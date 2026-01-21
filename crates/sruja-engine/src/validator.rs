// ... existing code ...

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_language::{Parser, Program};

    #[test]
    fn test_unique_id_rule() {
        let input = r#"
system A "System A"
system A "System A Duplicate"
"#;
        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).unwrap();
        
        let mut validator = Validator::new();
        validator.register_default_rules();
        let diagnostics = validator.validate_sync(&program);
        
        // Should have duplicate ID error
        assert!(diagnostics.iter().any(|d| d.code == "CODE_DUPLICATE_IDENTIFIER"));
    }

    #[test]
    fn test_valid_ref_rule() {
        let input = r#"
system A "System A"
A -> B "calls"
"#;
        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).unwrap();
        
        let mut validator = Validator::new();
        validator.register_default_rules();
        let diagnostics = validator.validate_sync(&program);
        
        // Should have invalid reference error
        assert!(diagnostics.iter().any(|d| d.code == "CODE_INVALID_REFERENCE"));
    }
}
