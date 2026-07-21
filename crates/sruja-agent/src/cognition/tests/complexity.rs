use super::*;

// --- Task complexity routing tests ---

#[test]
fn classify_trivial_comment_task() {
    let c = classify_task_complexity(
        "Add a short comment to the top of lib.rs",
        &["lib.rs".to_string()],
        &[],
    );
    assert_eq!(c, TaskComplexity::Trivial);
}

#[test]
fn classify_trivial_typo_task() {
    let c = classify_task_complexity("Fix typo in function name", &["main.rs".to_string()], &[]);
    assert_eq!(c, TaskComplexity::Trivial);
}

#[test]
fn classify_trivial_rename_single_file() {
    let c = classify_task_complexity(
        "Rename variable foo to bar",
        &["handler.rs".to_string()],
        &[],
    );
    assert_eq!(c, TaskComplexity::Trivial);
}

#[test]
fn classify_simple_small_change() {
    let c = classify_task_complexity(
        "Add input validation to the handler",
        &["handler.rs".to_string()],
        &[],
    );
    assert_eq!(c, TaskComplexity::Simple);
}

#[test]
fn classify_complex_migration() {
    let c = classify_task_complexity("Migrate the database schema", &[], &[]);
    assert_eq!(c, TaskComplexity::Complex);
}

#[test]
fn classify_complex_many_elements() {
    let c = classify_task_complexity(
        "Update all API endpoints",
        &[],
        &[
            "System.Api".to_string(),
            "System.Db".to_string(),
            "System.Auth".to_string(),
        ],
    );
    assert_eq!(c, TaskComplexity::Complex);
}

#[test]
fn classify_moderate_multi_file() {
    let c = classify_task_complexity(
        "Add JWT auth to the API",
        &[
            "auth.rs".to_string(),
            "middleware.rs".to_string(),
            "config.rs".to_string(),
        ],
        &["System.Api".to_string(), "System.Auth".to_string()],
    );
    assert_eq!(c, TaskComplexity::Moderate);
}

#[test]
fn complex_keywords_override_trivial() {
    let c = classify_task_complexity(
        "add a comment to migrate the database",
        &["migrations.rs".to_string()],
        &[],
    );
    assert_eq!(c, TaskComplexity::Complex);
}

#[test]
fn trivial_skips_tdd_and_artifacts() {
    assert!(!TaskComplexity::Trivial.enforce_tdd());
    assert!(!TaskComplexity::Trivial.generate_artifacts());
}

#[test]
fn simple_enforces_tdd_and_artifacts() {
    assert!(TaskComplexity::Simple.enforce_tdd());
    assert!(TaskComplexity::Simple.generate_artifacts());
}

#[test]
fn trivial_caps_tool_iterations() {
    assert_eq!(TaskComplexity::Trivial.max_tool_iterations(8), 7);
    assert_eq!(TaskComplexity::Simple.max_tool_iterations(8), 7);
    assert_eq!(TaskComplexity::Moderate.max_tool_iterations(8), 8);
}

// --- Research task classification tests ---

#[test]
fn classify_research_what_question() {
    let c = classify_task_complexity("what is the architecture of the parser", &[], &[]);
    assert_eq!(c, TaskComplexity::Research);
}

#[test]
fn classify_research_why_question() {
    let c = classify_task_complexity("why is the build failing", &["ci.yml".to_string()], &[]);
    assert_eq!(c, TaskComplexity::Research);
}

#[test]
fn classify_research_explain() {
    let c = classify_task_complexity("explain the migration system", &[], &[]);
    assert_eq!(c, TaskComplexity::Research);
}

#[test]
fn classify_research_analyze() {
    let c = classify_task_complexity("analyze the performance of the query engine", &[], &[]);
    assert_eq!(c, TaskComplexity::Research);
}

#[test]
fn classify_research_review() {
    let c = classify_task_complexity("review the security of the auth module", &[], &[]);
    assert_eq!(c, TaskComplexity::Research);
}

#[test]
fn classify_research_description() {
    let c = classify_task_complexity("describe the data flow between components", &[], &[]);
    assert_eq!(c, TaskComplexity::Research);
}

#[test]
fn classify_research_evaluate() {
    let c = classify_task_complexity("evaluate the parser performance", &[], &[]);
    assert_eq!(c, TaskComplexity::Research);
}

#[test]
fn classify_not_research_when_implementation_keyword() {
    let c = classify_task_complexity("fix the bug in the parser", &["parser.rs".to_string()], &[]);
    assert_eq!(c, TaskComplexity::Simple);
}

#[test]
fn classify_not_research_when_how_to() {
    let c = classify_task_complexity("how to add JWT auth to the API", &[], &[]);
    assert_eq!(c, TaskComplexity::Simple);
}

#[test]
fn classify_not_research_when_how_do() {
    let c = classify_task_complexity("how do I implement input validation", &[], &[]);
    assert_eq!(c, TaskComplexity::Simple);
}

#[test]
fn classify_not_research_with_implementation_keyword() {
    let c = classify_task_complexity("add JWT auth to the API", &[], &[]);
    assert_eq!(c, TaskComplexity::Simple);
}

#[test]
fn research_disables_tdd() {
    assert!(
        !TaskComplexity::Research.enforce_tdd(),
        "Research tasks should not enforce TDD (no code changes)"
    );
}

#[test]
fn research_generates_artifacts() {
    assert!(
        TaskComplexity::Research.generate_artifacts(),
        "Research should generate comprehension artifact"
    );
}

#[test]
fn research_caps_tool_iterations() {
    assert_eq!(
        TaskComplexity::Research.max_tool_iterations(8),
        8,
        "Research with 8 configured should use 8 (10 > 8)"
    );
    assert_eq!(
        TaskComplexity::Research.max_tool_iterations(12),
        10,
        "Research with 12 configured should cap at 10"
    );
}
