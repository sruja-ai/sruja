//! Tests for the validator module

use std::sync::Arc;

use sruja_diagnostics::Diagnostic;
use sruja_language::{Parser, Program};

use super::core::Validator;
use super::rule::Rule;
use crate::rules::UniqueIdRule;

fn create_test_program(input: &str) -> Program {
    let parser = Parser::new("test.sruja".to_string());
    parser.parse(input).expect("Parse failed")
}

#[test]
fn test_validator_new_empty() {
    let validator = Validator::new();
    assert_eq!(validator.rule_count(), 0);
}

#[test]
fn test_validator_with_default_rules() {
    let validator = Validator::with_default_rules();
    assert!(validator.rule_count() > 0);
    assert!(validator.has_rule("Unique IDs"));
    assert!(validator.has_rule("Valid References"));
    assert!(validator.has_rule("Orphan Detection"));
}

#[test]
fn test_validator_register_rule() {
    let mut validator = Validator::new();
    assert_eq!(validator.rule_count(), 0);

    validator.register_rule(Arc::new(UniqueIdRule));
    assert_eq!(validator.rule_count(), 1);
    assert!(validator.has_rule("Unique IDs"));
}

#[test]
fn test_validator_sync_validation() {
    let input = r#"
A = system "System A"
B = system "System B"
A -> B "calls"
"#;
    let program = create_test_program(input);
    let validator = Validator::with_default_rules();

    let diagnostics = validator.validate_sync(&program);
    assert!(diagnostics.is_empty());
}

#[test]
fn test_validator_sync_with_errors() {
    let input = r#"
A = system "System A"
A = system "System A Duplicate"
A -> B "calls"
"#;
    let program = create_test_program(input);
    let validator = Validator::with_default_rules();

    let diagnostics = validator.validate_sync(&program);
    assert!(!diagnostics.is_empty());
    assert!(diagnostics
        .iter()
        .any(|d| d.code == sruja_diagnostics::codes::CODE_DUPLICATE_ID));
}

#[test]
fn test_validator_builder_basic() {
    let validator = Validator::builder().build();
    assert_eq!(validator.rule_count(), 0);
}

#[test]
fn test_validator_builder_with_defaults() {
    let validator = Validator::builder().with_default_rules().build();

    assert!(validator.rule_count() > 0);
    assert!(validator.has_rule("Unique IDs"));
}

#[test]
fn test_validator_builder_exclude_rule() {
    let validator = Validator::builder()
        .with_default_rules()
        .exclude_rule("Orphan Detection")
        .build();

    assert!(!validator.has_rule("Orphan Detection"));
    assert!(validator.has_rule("Unique IDs"));
}

#[test]
fn test_validator_builder_fail_fast() {
    let input = r#"
A = system "System A"
A = system "Duplicate"
B = system "System B"
B = system "Duplicate 2"
"#;
    let program = create_test_program(input);

    let validator = Validator::builder()
        .with_default_rules()
        .with_fail_fast(true)
        .build();

    let diagnostics = validator.validate_sync(&program);
    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics
        .iter()
        .all(|d| d.code == sruja_diagnostics::codes::CODE_DUPLICATE_ID));
}

#[test]
#[cfg(feature = "async")]
fn test_validator_parallel_execution() {
    let input = r#"
A = system "System A"
B = system "System B"
A -> B "calls"
"#;
    let program = Arc::new(create_test_program(input));

    let validator = Validator::builder()
        .with_default_rules()
        .with_parallel(true)
        .build();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let diagnostics = rt.block_on(validator.validate(program));
    assert!(diagnostics.is_empty());
}

#[test]
fn test_validator_clone() {
    let validator = Validator::with_default_rules();
    let cloned = validator.clone();

    assert_eq!(validator.rule_count(), cloned.rule_count());
    assert!(cloned.has_rule("Unique IDs"));
}

#[test]
fn test_default_impl() {
    let validator = Validator::default();
    assert!(validator.rule_count() > 0);
    assert!(validator.has_rule("Unique IDs"));
}

#[test]
fn test_custom_rule() {
    struct TestRule;

    impl Rule for TestRule {
        fn name(&self) -> &str {
            "Test Rule"
        }

        fn validate(&self, _program: &Program) -> Vec<Diagnostic> {
            vec![]
        }
    }

    let validator = Validator::builder().with_rule(Arc::new(TestRule)).build();

    assert!(validator.has_rule("Test Rule"));
    assert_eq!(validator.rule_count(), 1);
}

#[test]
fn test_excluded_rule_not_registered() {
    struct TestRule;

    impl Rule for TestRule {
        fn name(&self) -> &str {
            "Test Rule"
        }

        fn validate(&self, _program: &Program) -> Vec<Diagnostic> {
            vec![]
        }
    }

    let mut validator = Validator::new();
    validator.excluded_rules.insert("Test Rule".to_string());
    validator.register_rule(Arc::new(TestRule));

    assert_eq!(validator.rule_count(), 0);
    assert!(!validator.has_rule("Test Rule"));
}
