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

#[test]
fn validator_reports_custom_constraint_violation() {
    let source = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"
database = kind "Database"

User = person "User" { description "User" }

App = system "App" {
  description "App"
  Web = container "Web" {
    technology "React"
    description "Web"
  }
  DB = database "DB" {
    technology "PostgreSQL"
    description "DB"
  }
}

User -> App "uses"
App.Web -> App.DB "queries"

constraints {
  "web -> database forbidden"
}
"#;
    let program = parse_valid_program(source);
    let validator = Validator::with_default_rules();
    let diagnostics = validator.validate_sync(&program);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.message.contains("Custom constraint violated")),
        "expected custom constraint diagnostic: {diagnostics:?}"
    );
}

#[test]
fn validator_reports_state_machine_and_contract_issues() {
    let source = r#"
system = kind "System"
component = kind "Component"

App = system "App" {
  description "App"

  Svc = component "Service" {
    description "Svc"
    state_machine "SM" {
      initial "Created"
    }
    contract "Empty" {
      description "empty"
    }
  }
}
"#;
    let program = parse_valid_program(source);
    let validator = Validator::with_default_rules();
    let diagnostics = validator.validate_sync(&program);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.message.contains("Initial state")),
        "expected state machine diagnostic: {diagnostics:?}"
    );
    assert!(
        diagnostics
            .iter()
            .any(|d| d.message.contains("Contract 'Empty' is empty")),
        "expected contract diagnostic: {diagnostics:?}"
    );
}
