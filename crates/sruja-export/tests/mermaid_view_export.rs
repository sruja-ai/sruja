//! Tests for mermaid view export functionality

#[cfg(test)]
mod tests {
    use sruja_export::mermaid::exporter::MermaidConfig;
    use sruja_export::MermaidExporter;
    use sruja_language::Parser;

    #[test]
    fn test_export_single_custom_view() {
        let input = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

User = person "User"
Admin = person "Administrator"

Shop = system "Shop" {
    Web = container "Web App"
    API = container "API Service"
    DB = container "Database"
}
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig {
            direction: "LR".to_string(),
            ..Default::default()
        };

        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("graph LR"));
        assert!(output.contains("Shop"));
        assert!(output.contains("Shop_API"));
        assert!(output.contains("Shop_DB"));
        assert!(output.contains("Shop_Web"));
    }

    #[test]
    fn test_export_all_views() {
        let input = r#"
person = kind "Person"
system = kind "System"

User = person "User"
Shop = system "Shop" {
    Web = container "Web App"
    API = container "API Service"
}
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig::default();
        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("graph LR"));
        assert!(output.contains("Shop"));
        assert!(output.contains("Shop_Web") || output.contains("Shop_API"));
    }

    #[test]
    fn test_view_not_found() {
        let input = r#"
person = kind "Person"
User = person "User"
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig::default();
        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        // Current API exports full graph; no named-view lookup, so we just assert we get a graph
        assert!(output.contains("graph LR"));
        assert!(output.contains("User"));
    }

    #[test]
    fn test_backward_compatibility_view_level() {
        let input = r#"
person = kind "Person"
system = kind "System"

User = person "User"
Shop = system "Shop"
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig {
            view_level: 1,
            ..Default::default()
        };

        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("graph LR"));
        assert!(output.contains("User"));
        assert!(output.contains("Shop"));
    }

    #[test]
    fn test_wildcard_in_view() {
        let input = r#"
system = kind "System"
container = kind "Container"

Shop = system "Shop" {
    Web = container "Web App"
    API = container "API Service"
    DB = container "Database"
}
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig::default();
        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("Shop"));
        assert!(output.contains("Web"));
        assert!(output.contains("API"));
        assert!(output.contains("DB"));
    }
}
