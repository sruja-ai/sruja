use super::*;

// --- Parse plan tests ---

#[test]
fn parse_plan_requires_id_field() {
    let raw = r#"{"subtasks":[
            {"description":"write add()","tier":"mid","kind":"implement","files":["src/main.rs"]}
        ],"risks":[]}"#;
    let err = parse_plan_from_response(raw, &crate::goal::GoalSpec::new("add function"), false)
        .unwrap_err();
    assert!(
        matches!(err, PlanParseError::MissingRequiredField { ref field, subtask_index: 0 } if field == "id"),
        "expected MissingRequiredField for id on subtask 0, got: {err}"
    );
}

#[test]
fn parse_plan_error_on_missing_required_field() {
    let raw = r#"{"subtasks":[
            {"id":"s1","description":"ok","tier":"mid","kind":"implement"},
            {"id":"s2","description":"no tier here","kind":"verify"}
        ],"risks":[]}"#;
    let err =
        parse_plan_from_response(raw, &crate::goal::GoalSpec::new("test"), false).unwrap_err();
    assert!(
        matches!(err, PlanParseError::MissingRequiredField { ref field, subtask_index: 1 } if field == "tier"),
        "expected MissingRequiredField for tier on subtask 1, got: {err}"
    );
}

#[test]
fn parse_plan_preserves_explicit_ids() {
    let raw = r#"{"subtasks":[
            {"id":"custom-id","description":"task","tier":"premium","kind":"review"}
        ],"risks":[]}"#;
    let plan = parse_plan_from_response(raw, &crate::goal::GoalSpec::new("test"), false).unwrap();
    assert_eq!(plan.subtasks.len(), 1);
    assert_eq!(plan.subtasks[0].id, "custom-id");
}

#[test]
fn parse_plan_empty_array_returns_no_subtasks_error() {
    let raw = r#"{"subtasks":[],"risks":["nothing to do"]}"#;
    let err = parse_plan_from_response(raw, &crate::goal::GoalSpec::new("add the function"), false)
        .unwrap_err();
    assert!(
        matches!(err, PlanParseError::NoSubtasks),
        "expected NoSubtasks, got: {err}"
    );
}

#[test]
fn parse_plan_happy_path_with_all_fields() {
    let raw = r#"{"schema_version":"1.0","subtasks":[
            {"id":"s1","description":"write add()","tier":"mid","kind":"implement","files":["src/main.rs"],"acceptance_criteria":["it works"]}
        ],"risks":["none"]}"#;
    let plan =
        parse_plan_from_response(raw, &crate::goal::GoalSpec::new("add function"), true).unwrap();
    assert_eq!(plan.subtasks.len(), 1);
    assert_eq!(plan.subtasks[0].id, "s1");
    assert_eq!(plan.schema_version, "1.0");
    assert!(plan.tdd);
    assert_eq!(plan.risks, vec!["none"]);
}

#[test]
fn parse_plan_malformed_json_returns_error() {
    let err = parse_plan_from_response(
        "not json at all",
        &crate::goal::GoalSpec::new("test"),
        false,
    )
    .unwrap_err();
    assert!(
        matches!(err, PlanParseError::MalformedJson(_)),
        "expected MalformedJson, got: {err}"
    );
}

#[test]
fn parse_plan_missing_subtasks_array_returns_error() {
    let raw = r#"{"risks":["none"]}"#;
    let err =
        parse_plan_from_response(raw, &crate::goal::GoalSpec::new("test"), false).unwrap_err();
    assert!(
        matches!(err, PlanParseError::MalformedJson(_)),
        "expected MalformedJson for missing subtasks, got: {err}"
    );
}

#[test]
fn parse_plan_backward_compat_no_schema_version() {
    let raw = r#"{"subtasks":[
            {"id":"s1","description":"task","tier":"cheap","kind":"implement"}
        ],"risks":[]}"#;
    let plan = parse_plan_from_response(raw, &crate::goal::GoalSpec::new("test"), false).unwrap();
    assert_eq!(plan.schema_version, "");
    assert_eq!(plan.subtasks[0].id, "s1");
}

// --- Parse critique tests ---

#[test]
fn parse_critique_i_do_not_approve_does_not_flip_to_approved() {
    let raw = "I do not approve this plan; it's missing tests.";
    let critique = parse_critique_from_response(raw, Usage::default());
    assert!(!critique.approved, "'I do not approve' should be rejected");
    assert_eq!(critique.score, 0.3);
    assert!(critique
        .issues
        .contains(&"could not parse structured critique".to_string()));
}

#[test]
fn parse_critique_approved_keyword_at_line_start_passes() {
    let raw = "Approved - the plan looks solid.";
    let critique = parse_critique_from_response(raw, Usage::default());
    assert!(critique.approved, "'Approved' at start should pass");
    assert_eq!(critique.score, 0.8);
}

#[test]
fn parse_critique_approved_keyword_on_new_line_passes() {
    let raw = "I reviewed this.\nApproved - all good.";
    let critique = parse_critique_from_response(raw, Usage::default());
    assert!(critique.approved, "'\\nApproved' should pass");
    assert_eq!(critique.score, 0.8);
}

#[test]
fn parse_critique_do_not_approve_fails() {
    let raw = "do not approve - tests are missing.";
    let critique = parse_critique_from_response(raw, Usage::default());
    assert!(!critique.approved, "'do not approve' should fail");
    assert_eq!(critique.score, 0.3);
}
