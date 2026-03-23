//! Parser unit tests.

#[cfg(test)]
mod tests {
    use crate::ast::{ElementKind, TopLevelItem};
    use crate::parser::{
        assignments::{
            parse_adr_assignment, parse_flow_assignment, parse_policy_assignment,
            parse_requirement_assignment, parse_scenario, parse_scenario_assignment,
        },
        elements::parse_element_def,
        import::parse_import,
        primitives::{
            line_to_byte_offset, parse_identifier, parse_string, parse_tag_array, parse_tag_ref,
        },
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
    fn test_parse_tag_ref() {
        assert_eq!(parse_tag_ref("#api"), Ok(("", "#api".to_string())));
        assert_eq!(parse_tag_ref("@pii"), Ok(("", "@pii".to_string())));
    }

    #[test]
    fn test_parse_tag_array() {
        let result = parse_tag_array(r#"[#api, @pii, internal]"#);
        assert!(result.is_ok());
        let (_, tags) = result.unwrap();
        assert_eq!(
            tags,
            vec![
                "#api".to_string(),
                "@pii".to_string(),
                "internal".to_string()
            ]
        );
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
    fn test_parse_element_def_with_tags() {
        let input = r#"MySystem = system "My System" #core @external"#;
        let result = parse_element_def(input);
        assert!(result.is_ok());
        let (_, elem) = result.unwrap();
        assert_eq!(
            elem.assignment.tag_refs,
            vec!["#core".to_string(), "@external".to_string()]
        );
    }

    #[test]
    fn test_parse_element_def_with_doc() {
        let input = r#"PaymentService = container "Payment Service" {
  technology "Node.js"
  description "Handles payment processing"
  doc ".sruja/knowledge/payment-service.md"
}"#;
        let result = parse_element_def(input);
        assert!(result.is_ok());
        let (_, elem) = result.unwrap();
        assert_eq!(elem.assignment.name, "PaymentService");
        let body = elem.assignment.body.as_ref().expect("body present");
        assert_eq!(
            body.description.as_deref(),
            Some("Handles payment processing")
        );
        assert_eq!(body.technology.as_deref(), Some("Node.js"));
        assert_eq!(
            body.doc.as_deref(),
            Some(".sruja/knowledge/payment-service.md")
        );
    }

    #[test]
    fn test_parse_element_def_unassigned_system() {
        let input = r#"system "My System" { }"#;
        let parser = Parser::new("test.sruja");
        let result = parser.parse(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let program = result.unwrap();
        let elem = program
            .items
            .iter()
            .find_map(|i| match i {
                TopLevelItem::ElementDef(e) => Some((**e).clone()),
                _ => None,
            })
            .expect("element should be present");
        assert_eq!(elem.assignment.name, "My_System");
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
    fn test_parse_scenario_step_tags_and_order() {
        let input = r#"scenario LoginFlow "User Login" {
            step User -> WebApp "Credentials" [#auth, @pii] order 1
        }"#;
        let result = parse_scenario(input);
        assert!(result.is_ok());
        let (_, scenario) = result.unwrap();
        assert_eq!(scenario.steps.len(), 1);
        let step = &scenario.steps[0];
        assert_eq!(step.tags, vec!["#auth".to_string(), "@pii".to_string()]);
        assert_eq!(step.order, Some(1));
    }

    #[test]
    fn test_parse_flow_assignment_title_description_and_steps() {
        let input = r#"Login = flow "Login Flow" "Successful login" {
            User -> Web "open"
            Web -> DB "read" [#sql] order 2
        }"#;
        let result = parse_flow_assignment(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, flow) = result.unwrap();
        assert_eq!(flow.id, "Login");
        assert_eq!(flow.title, "Login Flow");
        assert_eq!(flow.description, Some("Successful login".to_string()));
        assert_eq!(flow.steps.len(), 2);
        assert_eq!(flow.steps[1].tags, vec!["#sql".to_string()]);
        assert_eq!(flow.steps[1].order, Some(2));
    }

    #[test]
    fn test_parse_scenario_assignment_with_block_body_steps_array() {
        let input = r#"HappyPath = scenario "Checkout" {
            description "User checks out"
            steps [
                User -> Web "browse"
                Web -> DB "load" order "10"
            ]
        }"#;
        let result = parse_scenario_assignment(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, scenario) = result.unwrap();
        assert_eq!(scenario.id, "HappyPath");
        assert_eq!(scenario.title, "Checkout");
        assert_eq!(scenario.description, Some("User checks out".to_string()));
        assert_eq!(scenario.steps.len(), 2);
        assert_eq!(scenario.steps[1].order, Some(10));
    }

    #[test]
    fn test_parse_requirement_assignment_with_details_and_metadata_ignored() {
        let input = r#"R1 = requirement functional "Users can log in" {
            description "Must support SSO"
            tags [#auth, @pii]
            metadata {
                owner "team-auth"
            }
        }"#;
        let result = parse_requirement_assignment(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, req) = result.unwrap();
        assert_eq!(req.id, "R1");
        assert_eq!(req.r#type, "functional");
        assert_eq!(req.title, "Users can log in");
        assert_eq!(req.description, Some("Must support SSO".to_string()));
        assert_eq!(req.tags, vec!["#auth".to_string(), "@pii".to_string()]);
    }

    #[test]
    fn test_parse_adr_assignment_title_defaults_to_id() {
        let input = r#"ADR_1 = adr { status "accepted" }"#;
        let result = parse_adr_assignment(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, adr) = result.unwrap();
        assert_eq!(adr.id, "ADR_1");
        assert_eq!(adr.title, "ADR_1");
        assert_eq!(adr.status, Some("accepted".to_string()));
    }

    #[test]
    fn test_parse_policy_assignment_block_kvs_and_rules() {
        let input = r#"P = policy "Security" {
            category "security"
            enforcement "deny"
            description "Policy description"
            rule require tags on { kind "container" } tags [#tier1] message "msg" suggest "s1"
            rule deny edge from { id "A" } to { id "B" } message "no"
        }"#;
        let result = parse_policy_assignment(input);
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, policy) = result.unwrap();
        assert_eq!(policy.id, "P");
        assert_eq!(policy.title, "Security");
        assert_eq!(policy.category, "security");
        assert_eq!(policy.enforcement, "deny");
        assert_eq!(policy.description, Some("Policy description".to_string()));
        assert_eq!(policy.rules.len(), 2);
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
    fn test_parse_error_includes_location_context_and_suggestions() {
        let input = "=\n";
        let parser = Parser::new("test.sruja");
        let result = parser.parse(input);
        assert!(result.is_err());
        let diags = result.err().unwrap();
        assert!(!diags.is_empty());
        let d = &diags[0];
        assert!(d.location.line > 0, "line should be 1-indexed");
        assert!(d.location.column > 0, "column should be 1-indexed");
        assert!(
            !d.context.is_empty(),
            "context should be present for parser errors"
        );
        assert!(
            !d.suggestions.is_empty(),
            "suggestions should be present for parser errors"
        );
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

    #[test]
    fn test_parse_element_with_canonical_id() {
        let input = r#"
Payments = container "Payment Service" {
  id "svc.payments"
  technology "Go"
}
"#;
        let result = parse_element_def(input.trim());
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, elem) = result.unwrap();
        assert_eq!(elem.assignment.name, "Payments");
        let body = elem.assignment.body.as_ref().unwrap();
        assert_eq!(body.canonical_id, Some("svc.payments".to_string()));
    }

    #[test]
    fn test_parse_element_with_aliases() {
        let input = r#"
API = container "API Service" {
  aliases ["payments-api", "PAYMENTS_SVC"]
}
"#;
        let result = parse_element_def(input.trim());
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, elem) = result.unwrap();
        let body = elem.assignment.body.as_ref().unwrap();
        assert_eq!(body.aliases, vec!["payments-api", "PAYMENTS_SVC"]);
    }

    #[test]
    fn test_parse_element_with_owner_domain() {
        let input = r#"
Checkout = container "Checkout Service" {
  owner "team-checkout"
  domain "commerce"
}
"#;
        let result = parse_element_def(input.trim());
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, elem) = result.unwrap();
        let body = elem.assignment.body.as_ref().unwrap();
        assert_eq!(body.owner, Some("team-checkout".to_string()));
        assert_eq!(body.domain, Some("commerce".to_string()));
    }

    #[test]
    fn test_parse_element_with_criticality() {
        let input = r#"
Database = database "Primary DB" {
  criticality critical
}
"#;
        let result = parse_element_def(input.trim());
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, elem) = result.unwrap();
        let body = elem.assignment.body.as_ref().unwrap();
        assert_eq!(body.criticality, Some(crate::ast::Criticality::Critical));
    }

    #[test]
    fn test_parse_element_with_source_bindings() {
        let input = r#"
API = container "API Service" {
  source openapi "./specs/api.yaml"
  source kubernetes "./k8s/api/"
  source docs "./docs/api.md"
}
"#;
        let result = parse_element_def(input.trim());
        assert!(result.is_ok(), "should parse: {:?}", result.err());
        let (_, elem) = result.unwrap();
        let body = elem.assignment.body.as_ref().unwrap();
        assert_eq!(body.sources.len(), 3);

        let openapi = &body.sources[0];
        assert_eq!(openapi.kind.as_str(), "openapi");
        assert_eq!(openapi.path, "./specs/api.yaml");

        let k8s = &body.sources[1];
        assert_eq!(k8s.kind.as_str(), "kubernetes");
        assert_eq!(k8s.path, "./k8s/api/");

        let docs = &body.sources[2];
        assert_eq!(docs.kind.as_str(), "docs");
        assert_eq!(docs.path, "./docs/api.md");
    }

    #[test]
    fn test_parse_element_full_architecture_index() {
        let input = r#"
Payments = container "Payment Service" {
  technology "Go"
  description "Handles payment processing"
  
  id "svc.payments"
  aliases ["payments-api", "payments-service"]
  
  owner "team-payments"
  domain "commerce"
  criticality high
  
  source openapi "./specs/payments.yaml"
  source kubernetes "./k8s/payments/"
  source docs "./docs/payments.md"
}
"#;
        let result = parse_element_def(input.trim());
        assert!(
            result.is_ok(),
            "should parse full element: {:?}",
            result.err()
        );
        let (_, elem) = result.unwrap();
        assert_eq!(elem.assignment.name, "Payments");

        let body = elem.assignment.body.as_ref().unwrap();
        assert_eq!(body.technology, Some("Go".to_string()));
        assert_eq!(
            body.description,
            Some("Handles payment processing".to_string())
        );
        assert_eq!(body.canonical_id, Some("svc.payments".to_string()));
        assert_eq!(body.aliases, vec!["payments-api", "payments-service"]);
        assert_eq!(body.owner, Some("team-payments".to_string()));
        assert_eq!(body.domain, Some("commerce".to_string()));
        assert_eq!(body.criticality, Some(crate::ast::Criticality::High));
        assert_eq!(body.sources.len(), 3);
    }
}
