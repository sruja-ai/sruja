//! Parser unit tests.

#[cfg(test)]
mod tests {
    use crate::ast::{ElementKind, TopLevelItem};
    use crate::parser::{
        assignments::parse_scenario,
        elements::parse_element_def,
        import::parse_import,
        primitives::{line_to_byte_offset, parse_identifier, parse_string},
        relations::{parse_qualified_ident, parse_relation},
    };
    use crate::Parser;

    #[test]
    fn test_parse_identifier() {
        assert_eq!(
            parse_identifier("mySystem"),
            Ok(("", "mySystem".to_string()))
        );
        assert_eq!(
            parse_identifier("my-system_123"),
            Ok(("", "my-system_123".to_string()))
        );
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_string(r#""hello""#), Ok(("", "hello".to_string())));
        assert_eq!(parse_string(r#"'world'"#), Ok(("", "world".to_string())));
    }

    #[test]
    fn test_parse_qualified_ident() {
        let result = parse_qualified_ident("System.Container");
        assert!(result.is_ok());
        let (_, qid) = result.unwrap();
        assert_eq!(qid.parts, vec!["System", "Container"]);
    }

    #[test]
    fn test_parse_element_def() {
        let input = r#"MySystem = system "My System""#;
        let result = parse_element_def(input);
        assert!(result.is_ok());
        let (_, elem) = result.unwrap();
        assert_eq!(elem.assignment.name, "MySystem");
        assert_eq!(elem.assignment.kind, ElementKind::System);
        assert_eq!(elem.assignment.title, Some("My System".to_string()));
    }

    #[test]
    fn test_parse_relation() {
        let input = r#"SystemA -> SystemB "Uses" "SystemA uses SystemB""#;
        let result = parse_relation(input);
        assert!(result.is_ok());
        let (_, rel) = result.unwrap();
        assert_eq!(rel.from.parts, vec!["SystemA"]);
        assert_eq!(rel.to.parts, vec!["SystemB"]);
        assert_eq!(rel.label, Some("Uses".to_string()));
        assert_eq!(rel.description, Some("SystemA uses SystemB".to_string()));
    }

    #[test]
    fn test_parse_import() {
        let input = r#"import { ServiceA, ServiceB } from "projectA""#;
        let result = parse_import(input);
        assert!(result.is_ok());
        let (_, import_stmt) = result.unwrap();
        assert_eq!(import_stmt.elements.len(), 2);
        assert_eq!(import_stmt.from, "projectA");
    }

    #[test]
    fn test_parse_scenario() {
        let input = r#"scenario LoginFlow "User Login" {
            User -> WebApp "Credentials"
            WebApp -> DB "Verify"
        }"#;
        let result = parse_scenario(input);
        assert!(result.is_ok());
        let (_, scenario) = result.unwrap();
        assert_eq!(scenario.id, "LoginFlow");
        assert_eq!(scenario.title, "User Login".to_string());
        assert_eq!(scenario.steps.len(), 2);
    }

    #[test]
    fn test_parse_with_comments() {
        let input = r#"
        // This is a comment
        MySystem = system "My System"
        /* Multi-line
           comment */
        SystemA -> SystemB "Uses"
        "#;
        let parser = Parser::new("test.sruja");
        let result = parser.parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_incrementally_context_window() {
        let input = "A = system \"A\"\nB = system \"B\"\nA -> B \"uses\"\n";
        let parser = Parser::new("test.sruja");
        let existing = parser.parse(input).expect("initial parse");
        let edited = "A = system \"A\"\nB = system \"B Updated\"\nA -> B \"uses\"\n";
        let change_start = 22;
        let change_end = 35;
        let result = parser.parse_incrementally(edited, change_start, change_end, &existing, 2);
        assert!(result.is_ok(), "incremental parse should succeed");
        let inc = result.unwrap();
        assert!(!inc.updated_ast.items.is_empty());
        assert!(inc.changed_elements.contains(&"B".to_string()));
    }

    #[test]
    fn test_line_to_byte_offset() {
        let s = "a\nb\nc\n";
        assert_eq!(line_to_byte_offset(s, 0), 0);
        assert_eq!(line_to_byte_offset(s, 1), 2);
        assert_eq!(line_to_byte_offset(s, 2), 4);
        assert_eq!(line_to_byte_offset(s, 3), 6);
        assert_eq!(line_to_byte_offset(s, 4), 6);
    }

    #[test]
    fn test_parse_incrementally_many_cycles() {
        let parser = Parser::new("test.sruja");
        let base = "A = system \"A\"\nB = system \"B\"\nA -> B \"uses\"\n";
        let initial = parser.parse(base).expect("initial parse");
        let mut current_ast = initial;
        const CYCLES: usize = 50;
        for i in 0..CYCLES {
            let title = format!("B v{i}");
            let edited = format!("A = system \"A\"\nB = system \"{title}\"\nA -> B \"uses\"\n");
            let change_start = 22;
            let change_end = 22 + title.len();
            let result =
                parser.parse_incrementally(&edited, change_start, change_end, &current_ast, 2);
            assert!(result.is_ok(), "cycle {} should succeed", i);
            let inc = result.unwrap();
            current_ast = inc.updated_ast;
            assert!(
                !current_ast.items.is_empty(),
                "cycle {}: ast should be non-empty",
                i
            );
        }
    }

    #[test]
    fn test_parse_large_dsl() {
        let mut dsl = String::with_capacity(50_000);
        for i in 0..100 {
            dsl.push_str(&format!("S{i} = system \"System {i}\"\n"));
        }
        for i in 0..99 {
            dsl.push_str(&format!("S{i} -> S{} \"calls\"\n", i + 1));
        }
        let parser = Parser::new("large.sruja");
        let start = std::time::Instant::now();
        let result = parser.parse(&dsl);
        let elapsed_ms = start.elapsed().as_millis();
        assert!(result.is_ok(), "large DSL should parse: {:?}", result.err());
        let program = result.unwrap();
        let elem_count = program
            .items
            .iter()
            .filter(|i| matches!(i, TopLevelItem::ElementDef(_)))
            .count();
        assert!(
            elem_count >= 100,
            "expected at least 100 elements, got {}",
            elem_count
        );
        assert!(
            elapsed_ms < 5000,
            "large parse took {} ms (target <5s in debug)",
            elapsed_ms
        );
    }
}
