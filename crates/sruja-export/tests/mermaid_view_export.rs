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

    #[test]
    fn test_empty_program_returns_empty() {
        let program = sruja_language::Program::new();
        let config = MermaidConfig::default();
        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);
        assert!(output.is_empty());
    }

    #[test]
    fn test_direction_tb() {
        let input = r#"
system = kind "System"
App = system "App"
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig {
            direction: "TB".to_string(),
            ..Default::default()
        };

        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("graph TB"));
    }

    #[test]
    fn test_relations_with_labels() {
        let input = r#"
person = kind "Person"
system = kind "System"

User = person "User"
Shop = system "Shop"

User -> Shop "uses website"
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig::default();
        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("-->|"));
        assert!(output.contains("uses website"));
    }

    #[test]
    fn test_database_kind_style() {
        let input = r#"
system = kind "System"
container = kind "Container"

App = system "App" {
    DB = container "PostgreSQL"
}
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig::default();
        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("App_DB"));
        assert!(output.contains("classDef container"));
    }

    #[test]
    fn test_queue_kind_style() {
        let input = r#"
system = kind "System"
container = kind "Container"

App = system "App" {
    MQ = container "Message Queue"
}
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig::default();
        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("App_MQ"));
        assert!(output.contains("classDef container"));
    }

    #[test]
    fn test_view_level_2_with_target() {
        let input = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

User = person "User"
Shop = system "Shop" {
    API = container "API"
    DB = container "Database"
}
Payment = system "Payment"
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig {
            view_level: 2,
            target_id: Some("Shop".to_string()),
            ..Default::default()
        };

        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("Shop"));
        assert!(output.contains("Shop_API"));
    }

    #[test]
    fn test_view_level_3_with_target() {
        let input = r#"
system = kind "System"
container = kind "Container"
component = kind "Component"

Shop = system "Shop" {
    API = container "API" {
        Auth = component "Auth"
        Users = component "Users"
    }
}
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig {
            view_level: 3,
            target_id: Some("Shop.API".to_string()),
            ..Default::default()
        };

        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("Shop_API"));
    }

    #[test]
    fn test_subgraph_for_nested_elements() {
        let input = r#"
system = kind "System"
container = kind "Container"

Shop = system "Shop" {
    Web = container "Web"
    API = container "API"
}
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig::default();
        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("subgraph"));
        assert!(output.contains("end"));
    }

    #[test]
    fn test_class_definitions() {
        let input = r#"
person = kind "Person"
system = kind "System"

User = person "User"
App = system "App"
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig::default();
        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("classDef person"));
        assert!(output.contains("classDef system"));
    }

    #[test]
    fn test_special_chars_in_labels_escaped() {
        let input = r#"
system = kind "System"
App = system "App with quotes"
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig::default();
        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("App with quotes"));
    }

    #[test]
    fn test_external_system_style() {
        let input = r#"
person = kind "Person"
system = kind "System"
externalSystem = kind "ExternalSystem"

User = person "User"
App = system "App"
Ext = externalSystem "External API"

User -> App "uses"
App -> Ext "calls"
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig {
            view_level: 3,
            ..Default::default()
        };
        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("Ext"));
        assert!(output.contains("classDef external"));
    }

    #[test]
    fn test_datastore_kind() {
        let input = r#"
system = kind "System"
datastore = kind "DataStore"

App = system "App"
Store = datastore "Cache"
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig {
            view_level: 2,
            ..Default::default()
        };
        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("Store"));
        assert!(output.contains("classDef database"));
    }

    #[test]
    fn test_long_label_truncated() {
        let input = r#"
system = kind "System"
App = system "This is a very long system name that should be truncated at some point"
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig::default();
        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("This is a very long"));
    }

    #[test]
    fn test_multiple_systems_no_auto_l2() {
        let input = r#"
person = kind "Person"
system = kind "System"

User = person "User"
Shop = system "Shop"
Payment = system "Payment"
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig {
            view_level: 1,
            ..Default::default()
        };

        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("User"));
        assert!(output.contains("Shop"));
        assert!(output.contains("Payment"));
    }

    #[test]
    fn test_component_kind_style() {
        let input = r#"
system = kind "System"
container = kind "Container"
component = kind "Component"

Shop = system "Shop" {
    API = container "API" {
        Auth = component "Auth Service"
    }
}
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig::default();
        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("classDef component"));
    }

    #[test]
    fn test_relations_without_label() {
        let input = r#"
system = kind "System"

A = system "System A"
B = system "System B"

A -> B
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = MermaidConfig::default();
        let exporter = MermaidExporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("A --> B"));
    }
}
