#[cfg(test)]
mod tests {
    use sruja_export::d2::{D2Config, D2Exporter};
    use sruja_language::Parser;

    #[test]
    fn test_d2_export_basic() {
        let input = r#"
system = kind "System"
App = system "My App"
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = D2Config::default();
        let exporter = D2Exporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("direction: right"));
        assert!(output.contains("App: \"My App\""));
        assert!(output.contains("shape: package"));
    }

    #[test]
    fn test_d2_export_with_default_links() {
        let input = r#"
system = kind "System"
App = system "My App"
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = D2Config::default();
        let exporter = D2Exporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("link: \"vscode://file/test.sruja:3:1\""));
    }

    #[test]
    fn test_d2_export_with_custom_link_template() {
        let input = r#"
system = kind "System"
App = system "My App"
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = D2Config {
            link_template: Some("https://github.com/org/repo/blob/main/{file}#L{line}".to_string()),
            ..Default::default()
        };
        let exporter = D2Exporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("link: \"https://github.com/org/repo/blob/main/test.sruja#L3\""));
    }

    #[test]
    fn test_d2_export_relation_links() {
        let input = r#"
system = kind "System"
A = system "System A"
B = system "System B"

A -> B "links to"
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = D2Config::default();
        let exporter = D2Exporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("A -> B: \"links to\" {"));
        assert!(output.contains("link: \"vscode://file/test.sruja:6:1\""));
    }

    #[test]
    fn test_d2_export_nested_elements() {
        let input = r#"
system = kind "System"
container = kind "Container"

Shop = system "Shop" {
    API = container "API"
}
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let config = D2Config::default();
        let exporter = D2Exporter::new(config);
        let output = exporter.export(&program);

        assert!(output.contains("Shop: \"Shop\" {"));
        assert!(output.contains("link: \"vscode://file/test.sruja:5:1\""));
        assert!(output.contains("link: \"vscode://file/test.sruja:6:5\""));
    }
}
