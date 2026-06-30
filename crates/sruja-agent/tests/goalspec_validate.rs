//! Tests for GoalSpec::validate() element-ID validation.

use sruja_agent::GoalSpec;

/// Helper to create a GoalSpec with target_elements set.
fn goal_with_targets(targets: &[&str]) -> GoalSpec {
    let mut g = GoalSpec::new("test goal");
    g.target_elements = targets.iter().map(|s| s.to_string()).collect();
    g
}

#[test]
fn validate_valid_element_ids_returns_ok() {
    let g = goal_with_targets(&["Sruja.CLI", "Sruja.Agent"]);
    let available = vec![
        "Sruja.CLI".to_string(),
        "Sruja.Agent".to_string(),
        "Sruja.Core".to_string(),
    ];
    assert!(g.validate(Some(&available)).is_ok());
}

#[test]
fn validate_unknown_element_id_returns_err_listing_bad_ids() {
    let g = goal_with_targets(&["Sruja.CLI", "Fake.Module"]);
    let available = vec!["Sruja.CLI".to_string(), "Sruja.Core".to_string()];
    let err = g.validate(Some(&available)).unwrap_err();
    assert!(
        err.iter().any(|s| s.contains("Fake.Module")),
        "error should mention bad IDs: {err:?}"
    );
}

#[test]
fn validate_empty_target_elements_returns_ok() {
    let g = GoalSpec::new("do nothing special");
    let available = vec!["Sruja.CLI".to_string()];
    assert!(g.validate(Some(&available)).is_ok());
}

#[test]
fn validate_no_available_elements_returns_ok() {
    let g = goal_with_targets(&["Sruja.CLI"]);
    assert!(g.validate(None).is_ok());
}
