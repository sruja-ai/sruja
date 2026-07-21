use super::*;

// --- Ensemble critic tests (U1) ---

#[tokio::test]
async fn ensemble_one_persona_blocks_union_issues_and_min_score() {
    let llm = PersonaScriptedLlm::new(vec![
        PersonaResponse {
            system_prompt_contains: "correctness",
            approved: false,
            score: 0.2,
            issues: vec!["buffer overflow on empty input".into()],
        },
        PersonaResponse {
            system_prompt_contains: "acceptance criteria",
            approved: true,
            score: 0.8,
            issues: vec![],
        },
        PersonaResponse {
            system_prompt_contains: "boundary",
            approved: true,
            score: 0.9,
            issues: vec![],
        },
        PersonaResponse {
            system_prompt_contains: "regression",
            approved: true,
            score: 0.85,
            issues: vec![],
        },
        PersonaResponse {
            system_prompt_contains: "adversarial",
            approved: true,
            score: 0.95,
            issues: vec![],
        },
    ]);

    let config = AgentConfig {
        critique_personas: CritiquePersona::default_personas(),
        critique_mode: CritiqueMode::Full,
        ..Default::default()
    };
    let agent = Agent::builder()
        .llm(llm)
        .tools(ToolRegistry::new())
        .config(config)
        .build()
        .expect("agent builds");
    let result = agent
        .critique(
            &Plan {
                goal: "fix bug".into(),
                goal_statement: "fix bug".into(),
                criteria: Vec::new(),
                subtasks: vec![],
                tdd: false,
                risks: vec![],
                schema_version: String::new(),
                complexity: TaskComplexity::default(),
            },
            &[],
        )
        .await
        .expect("critique runs");

    assert!(!result.approved);
    assert!(result
        .issues
        .contains(&"buffer overflow on empty input".into()));
    assert!((result.score - 0.2).abs() < f64::EPSILON);
    assert_eq!(result.persona_breakdown.len(), 5);
    let correctness_result = result
        .persona_breakdown
        .iter()
        .find(|p| p.id == "correctness")
        .expect("correctness persona recorded");
    assert!(!correctness_result.approved);
    assert_eq!(correctness_result.score, 0.2);
    assert!(correctness_result.issues == vec!["buffer overflow on empty input"]);
}

#[test]
fn ensemble_empty_personas_fallback_to_single_critic() {
    let mut config = AgentConfig::default();
    config.critique_personas.clear();

    let agent = Agent::builder()
        .llm(ScriptedLlm::approve_after(0))
        .tools(ToolRegistry::new())
        .config(config)
        .build()
        .expect("agent builds");

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let result = agent
            .critique(
                &Plan {
                    goal: "test".into(),
                    goal_statement: "test".into(),
                    criteria: Vec::new(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                    schema_version: String::new(),
                    complexity: TaskComplexity::default(),
                },
                &[],
            )
            .await
            .expect("critique runs");

        assert!(result.approved);
        assert_eq!(result.persona_breakdown, vec![]);
        assert_eq!(result.injected_learning_ids, Vec::<String>::new());
    });
}

#[tokio::test]
async fn ensemble_union_dedup_issues() {
    let llm = PersonaScriptedLlm::new(vec![
        PersonaResponse {
            system_prompt_contains: "correctness",
            approved: true,
            score: 0.9,
            issues: vec!["tests missing".into()],
        },
        PersonaResponse {
            system_prompt_contains: "acceptance criteria",
            approved: true,
            score: 0.8,
            issues: vec!["tests missing".into()],
        },
        PersonaResponse {
            system_prompt_contains: "boundary",
            approved: true,
            score: 0.9,
            issues: vec!["tests missing".into()],
        },
        PersonaResponse {
            system_prompt_contains: "regression",
            approved: true,
            score: 0.85,
            issues: vec!["tests missing".into()],
        },
        PersonaResponse {
            system_prompt_contains: "adversarial",
            approved: true,
            score: 0.95,
            issues: vec!["tests missing".into()],
        },
    ]);

    let config = AgentConfig {
        critique_personas: CritiquePersona::default_personas(),
        critique_mode: CritiqueMode::Full,
        ..Default::default()
    };
    let agent = Agent::builder()
        .llm(llm)
        .tools(ToolRegistry::new())
        .config(config)
        .build()
        .expect("agent builds");
    let result = agent
        .critique(
            &Plan {
                goal: "test".into(),
                goal_statement: "test".into(),
                criteria: Vec::new(),
                subtasks: vec![],
                tdd: false,
                risks: vec![],
                schema_version: String::new(),
                complexity: TaskComplexity::default(),
            },
            &[],
        )
        .await
        .expect("critique runs");

    assert!(result.approved);
    assert_eq!(result.issues.len(), 1);
    assert_eq!(result.issues[0], "tests missing");
}

#[tokio::test]
async fn ensemble_parallel_dispatch_is_concurrent() {
    let llm = PersonaScriptedLlm::new(vec![
        PersonaResponse {
            system_prompt_contains: "correctness",
            approved: true,
            score: 0.9,
            issues: vec![],
        },
        PersonaResponse {
            system_prompt_contains: "acceptance criteria",
            approved: true,
            score: 0.8,
            issues: vec![],
        },
        PersonaResponse {
            system_prompt_contains: "boundary",
            approved: true,
            score: 0.9,
            issues: vec![],
        },
        PersonaResponse {
            system_prompt_contains: "regression",
            approved: true,
            score: 0.85,
            issues: vec![],
        },
        PersonaResponse {
            system_prompt_contains: "adversarial",
            approved: true,
            score: 0.95,
            issues: vec![],
        },
    ]);

    let config = AgentConfig {
        critique_personas: CritiquePersona::default_personas(),
        critique_mode: CritiqueMode::Full,
        ..Default::default()
    };
    let agent = Agent::builder()
        .llm(llm.clone())
        .tools(ToolRegistry::new())
        .config(config)
        .build()
        .expect("agent builds");
    let result = agent
        .critique(
            &Plan {
                goal: "concurrency check".into(),
                goal_statement: "concurrency check".into(),
                criteria: Vec::new(),
                subtasks: vec![],
                tdd: false,
                risks: vec![],
                schema_version: String::new(),
                complexity: TaskComplexity::default(),
            },
            &[],
        )
        .await
        .expect("critique runs");

    assert!(result.approved);
    assert!(llm.max_concurrent.load(Ordering::SeqCst) >= 2);
}

#[tokio::test]
async fn ensemble_all_personas_approve() {
    let llm = PersonaScriptedLlm::new(vec![
        PersonaResponse {
            system_prompt_contains: "correctness",
            approved: true,
            score: 1.0,
            issues: vec![],
        },
        PersonaResponse {
            system_prompt_contains: "acceptance criteria",
            approved: true,
            score: 0.95,
            issues: vec![],
        },
        PersonaResponse {
            system_prompt_contains: "boundary",
            approved: true,
            score: 0.9,
            issues: vec![],
        },
        PersonaResponse {
            system_prompt_contains: "regression",
            approved: true,
            score: 0.85,
            issues: vec![],
        },
        PersonaResponse {
            system_prompt_contains: "adversarial",
            approved: true,
            score: 0.95,
            issues: vec![],
        },
    ]);

    let config = AgentConfig {
        critique_personas: CritiquePersona::default_personas(),
        critique_mode: CritiqueMode::Full,
        ..Default::default()
    };
    let agent = Agent::builder()
        .llm(llm)
        .tools(ToolRegistry::new())
        .config(config)
        .build()
        .expect("agent builds");
    let result = agent
        .critique(
            &Plan {
                goal: "all good".into(),
                goal_statement: "all good".into(),
                criteria: Vec::new(),
                subtasks: vec![],
                tdd: false,
                risks: vec![],
                schema_version: String::new(),
                complexity: TaskComplexity::default(),
            },
            &[],
        )
        .await
        .expect("critique runs");

    assert!(result.approved);
    assert!(result.issues.is_empty());
    assert!((result.score - 0.85).abs() < f64::EPSILON);
    assert_eq!(result.persona_breakdown.len(), 5);
}

#[tokio::test]
async fn ensemble_score_is_min_not_mean() {
    let llm = PersonaScriptedLlm::new(vec![
        PersonaResponse {
            system_prompt_contains: "correctness",
            approved: true,
            score: 1.0,
            issues: vec![],
        },
        PersonaResponse {
            system_prompt_contains: "acceptance criteria",
            approved: true,
            score: 1.0,
            issues: vec![],
        },
        PersonaResponse {
            system_prompt_contains: "boundary",
            approved: true,
            score: 1.0,
            issues: vec![],
        },
        PersonaResponse {
            system_prompt_contains: "regression",
            approved: true,
            score: 0.2,
            issues: vec![],
        },
        PersonaResponse {
            system_prompt_contains: "adversarial",
            approved: true,
            score: 1.0,
            issues: vec![],
        },
    ]);

    let config = AgentConfig {
        critique_personas: CritiquePersona::default_personas(),
        critique_mode: CritiqueMode::Full,
        ..Default::default()
    };
    let agent = Agent::builder()
        .llm(llm)
        .tools(ToolRegistry::new())
        .config(config)
        .build()
        .expect("agent builds");
    let result = agent
        .critique(
            &Plan {
                goal: "score check".into(),
                goal_statement: "score check".into(),
                criteria: Vec::new(),
                subtasks: vec![],
                tdd: false,
                risks: vec![],
                schema_version: String::new(),
                complexity: TaskComplexity::default(),
            },
            &[],
        )
        .await
        .expect("critique runs");

    assert!(result.approved);
    assert!((result.score - 0.2).abs() < f64::EPSILON);
}
