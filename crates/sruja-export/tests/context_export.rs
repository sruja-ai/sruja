//! Tests for Context exporter functionality

#[cfg(test)]
mod tests {
    use sruja_export::ContextExporter;
    use sruja_language::Parser;

    fn parse(input: &str) -> sruja_language::Program {
        Parser::new("test.sruja".to_string())
            .parse(input)
            .expect("Failed to parse")
    }

    #[test]
    fn test_empty_program_returns_empty() {
        let program = sruja_language::Program::new();
        let exporter = ContextExporter::new("general");
        let out = exporter.export(&program);
        assert!(out.is_empty());
    }

    #[test]
    fn test_general_template_exports_systems_and_persons() {
        let input = r#"
person = kind "Person"
system = kind "System"

User = person "End User" {
  description "Application user"
}

Shop = system "Shop" {
  description "Online store"
  technology "Node.js"
}

User -> Shop "uses"
"#;
        let program = parse(input);
        let exporter = ContextExporter::new("general");
        let out = exporter.export(&program);

        assert!(out.contains("# System Architecture Context"));
        assert!(out.contains("## Systems"));
        assert!(out.contains("## Actors"));
        assert!(out.contains("Shop"));
        assert!(out.contains("Online store"));
        assert!(out.contains("Node.js"));
        assert!(out.contains("End User"));
        assert!(out.contains("Application user"));
        assert!(out.contains("Key Relationships"));
        assert!(out.contains("User → Shop"));
    }

    #[test]
    fn test_proposal_template() {
        let input = r#"
system = kind "System"
A = system "System A"
"#;
        let program = parse(input);
        let exporter = ContextExporter::new("proposal");
        let out = exporter.export(&program);

        assert!(out.contains("# Architecture Proposal"));
        assert!(out.contains("proposed system architecture"));
    }

    #[test]
    fn test_security_template() {
        let input = r#"
system = kind "System"
A = system "System A"
"#;
        let program = parse(input);
        let exporter = ContextExporter::new("security");
        let out = exporter.export(&program);

        assert!(out.contains("# Security Architecture Review"));
        assert!(out.contains("security-focused view"));
    }
}
