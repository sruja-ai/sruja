//! Tests for JSON exporter functionality

#[cfg(test)]
mod tests {
    use sruja_export::json::Exporter;
    use sruja_language::Parser;

    fn parse(input: &str) -> sruja_language::Program {
        Parser::new("test.sruja".to_string())
            .parse(input)
            .expect("Failed to parse")
    }

    #[test]
    fn test_empty_program_returns_empty_json() {
        let program = sruja_language::Program::new();
        let exporter = Exporter::new();
        let json = exporter.export(&program).expect("export failed");
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_basic_elements_export() {
        let input = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

User = person "End User"

Backend = system "Backend System" {
  description "Core backend system"
  API = container "API Service"
  DB = container "Database"
  API -> DB "reads/writes"
}
"#;
        let program = parse(input);
        let exporter = Exporter::new();
        let json = exporter.export(&program).expect("export failed");

        let parsed: serde_json::Value = serde_json::from_str(&json).expect("invalid JSON");
        let elements = parsed
            .get("elements")
            .and_then(|v| v.as_object())
            .expect("elements object missing");

        assert!(elements.contains_key("User"));
        assert!(elements.contains_key("Backend"));
        assert!(elements.contains_key("Backend.API"));
        assert!(elements.contains_key("Backend.DB"));

        let backend = elements.get("Backend").and_then(|v| v.as_object()).unwrap();
        assert_eq!(
            backend.get("title").and_then(|v| v.as_str()),
            Some("Backend System")
        );
        assert_eq!(
            backend.get("description").and_then(|v| v.as_str()),
            Some("Core backend system")
        );
    }

    #[test]
    fn test_relations_export() {
        let input = r#"
person = kind "Person"
system = kind "System"

User = person "User"
A = system "System A"
B = system "System B"

User -> A "uses"
A -> B "calls"
"#;
        let program = parse(input);
        let exporter = Exporter::new();
        let json = exporter.export(&program).expect("export failed");

        let parsed: serde_json::Value = serde_json::from_str(&json).expect("invalid JSON");
        let relations = parsed
            .get("relations")
            .and_then(|v| v.as_array())
            .expect("relations array missing");

        assert_eq!(relations.len(), 2);
        let rel0 = relations[0].as_object().unwrap();
        assert_eq!(
            rel0.get("source")
                .and_then(|v| v.get("model"))
                .and_then(|v| v.as_str()),
            Some("User")
        );
        assert_eq!(
            rel0.get("target")
                .and_then(|v| v.get("model"))
                .and_then(|v| v.as_str()),
            Some("A")
        );
        assert_eq!(rel0.get("title").and_then(|v| v.as_str()), Some("uses"));
    }

    #[test]
    fn test_export_compact() {
        let input = r#"
system = kind "System"
A = system "System A"
"#;
        let program = parse(input);
        let exporter = Exporter::new();
        let compact = exporter
            .export_compact(&program)
            .expect("export_compact failed");

        let as_str = String::from_utf8(compact).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&as_str).expect("invalid JSON");
        assert!(parsed.get("elements").is_some());
        // Compact should have no pretty-print indentation
        assert!(!as_str.contains("\n  "));
    }

    #[test]
    fn test_scenarios_and_requirements_in_sruja_extensions() {
        let input = r#"
person = kind "Person"
system = kind "System"

User = person "User"
Shop = system "Shop"

 scenario login "User login" "User logs in via form" {
   User -> Shop "submits credentials"
 }

R1 = requirement functional "Users must be able to login"
ADR001 = adr "Use HTTPS" {
  status "Accepted"
  context "Security"
  decision "Use TLS 1.3"
  consequences "Encrypted traffic"
}
"#;
        let program = parse(input);
        let exporter = Exporter::new();
        let json = exporter.export(&program).expect("export failed");

        let parsed: serde_json::Value = serde_json::from_str(&json).expect("invalid JSON");
        let sruja = parsed
            .get("sruja")
            .and_then(|v| v.as_object())
            .expect("sruja extensions missing");

        let scenarios = sruja.get("scenarios").and_then(|v| v.as_array()).unwrap();
        assert!(!scenarios.is_empty());

        let requirements = sruja
            .get("requirements")
            .and_then(|v| v.as_array())
            .unwrap();
        assert!(!requirements.is_empty());

        let adrs = sruja.get("adrs").and_then(|v| v.as_array()).unwrap();
        assert!(!adrs.is_empty());
    }

    #[test]
    fn test_to_model_dump_structure() {
        let input = r#"
system = kind "System"
A = system "System A" {
  description "Test system"
  technology "Rust"
}
"#;
        let program = parse(input);
        let exporter = Exporter::new();
        let dump = exporter.to_model_dump(&program);

        assert_eq!(dump.elements.len(), 1);
        assert!(dump.elements.contains_key("A"));
        let elem = &dump.elements["A"];
        assert_eq!(elem.title, "System A");
        assert_eq!(elem.description, Some("Test system".to_string()));
        assert_eq!(elem.technology, Some("Rust".to_string()));
    }

    #[test]
    fn test_export_element_with_doc() {
        let input = r#"
container = kind "Container"
PaymentService = container "Payment Service" {
  technology "Node.js"
  description "Handles payment processing"
  doc ".sruja/knowledge/PaymentService.md"
}
"#;
        let program = parse(input);
        let exporter = Exporter::new();
        let json = exporter.export(&program).expect("export failed");

        let parsed: serde_json::Value = serde_json::from_str(&json).expect("invalid JSON");
        let elements = parsed
            .get("elements")
            .and_then(|v| v.as_object())
            .expect("elements object missing");
        let elem = elements
            .get("PaymentService")
            .and_then(|v| v.as_object())
            .expect("PaymentService element missing");
        assert_eq!(
            elem.get("doc").and_then(|v| v.as_str()),
            Some(".sruja/knowledge/PaymentService.md")
        );
    }
}
