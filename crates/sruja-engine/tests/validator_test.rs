//! Basic tests for the validation engine: parse + validate_sync with default rules.

use sruja_diagnostics::Severity;
use sruja_engine::Validator;
use sruja_language::Parser;

fn parse_valid_program(source: &str) -> sruja_language::Program {
    let parser = Parser::new("test.sruja".to_string());
    match parser.parse(source) {
        Ok(program) => program,
        Err(diags) => panic!("Parse failed: {:?}", diags),
    }
}

#[test]
fn valid_minimal_dsl_produces_no_errors() {
    let source = r#"
        user = person "User" {
            description "End user"
        }
        web = system "Web" {
            description "Web app"
        }
        user -> web "uses"
    "#;
    let program = parse_valid_program(source);
    let validator = Validator::with_default_rules();
    let diagnostics = validator.validate_sync(&program);

    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Valid minimal DSL should produce no errors; got: {:?}",
        errors
    );
}

#[test]
fn validator_with_default_rules_runs_cycle_detection() {
    let source = r#"
        a = system "A" { description "A" }
        b = system "B" { description "B" }
        a -> b "calls"
        b -> a "calls"
    "#;
    let program = parse_valid_program(source);
    let validator = Validator::with_default_rules();
    let diagnostics = validator.validate_sync(&program);

    let has_cycle_diag = diagnostics
        .iter()
        .any(|d| d.message.contains("Circular") || d.message.to_lowercase().contains("cycle"));
    assert!(
        has_cycle_diag,
        "Cycle in DSL should produce a cycle-related diagnostic; got: {:?}",
        diagnostics
    );
}
