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

    #[test]
    fn test_unknown_template_uses_default_header() {
        let input = r#"
system = kind "System"
A = system "System A"
"#;
        let program = parse(input);
        let exporter = ContextExporter::new("unknown_template");
        let out = exporter.export(&program);

        assert!(out.contains("# System Architecture Context"));
    }

    #[test]
    fn test_context_export_with_multiple_systems() {
        let input = r#"
system = kind "System"

Shop = system "Shop" {
  description "Online store"
}

User = system "User" {
  description "User management"
}
"#;
        let program = parse(input);
        let exporter = ContextExporter::new("general");
        let out = exporter.export(&program);

        assert!(out.contains("Shop"));
        assert!(out.contains("User"));
        assert!(out.contains("Online store"));
        assert!(out.contains("User management"));
        assert!(out.contains("## Systems"));
    }

    #[test]
    fn test_context_export_limits_relations_to_10() {
        let input = r#"
system = kind "System"
A = system "A"
B = system "B"
C = system "C"
D = system "D"
E = system "E"
F = system "F"
G = system "G"
H = system "H"
I = system "I"
J = system "J"
K = system "K"
L = system "L"
A -> B "calls"
A -> C "calls"
A -> D "calls"
A -> E "calls"
A -> F "calls"
A -> G "calls"
A -> H "calls"
A -> I "calls"
A -> J "calls"
A -> K "calls"
A -> L "calls"
"#;
        let program = parse(input);
        let exporter = ContextExporter::new("general");
        let out = exporter.export(&program);

        assert!(out.contains("Key Relationships"));
        let count = out.matches("→").count();
        assert_eq!(count, 10, "should limit to 10 relations");
    }
}
