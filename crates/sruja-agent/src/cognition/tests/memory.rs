use super::*;

// --- Memory-in-critique tests (U4) ---

#[tokio::test]
async fn critique_injects_guardrail_blind_spots_and_bumps_retrieval_count() {
    let guardrail = LearningEntry::guardrail(
        "boundary crossing added",
        "change crosses forbidden dependency",
        "This change crosses a forbidden dependency boundary. Consider alternative approach.",
    );
    let playbook = LearningEntry::new(
        "pattern works",
        "regex pattern extraction succeeded",
        "Pattern extraction approach is validated.",
    );

    let config = AgentConfig {
        critique_personas: CritiquePersona::default_personas(),
        critique_mode: CritiqueMode::Full,
        ..Default::default()
    };
    let tempdir = tempfile::tempdir().expect("tempdir");

    let llm = PersonaScriptedLlm::new(vec![]);
    let concrete_mem = std::sync::Mutex::new(AgenticMemory::default());
    let mem_arc: std::sync::Arc<dyn crate::memory::Memory + Send + Sync> =
        std::sync::Arc::new(concrete_mem);
    let agent = Agent::builder()
        .llm(llm)
        .tools(ToolRegistry::new())
        .config(config)
        .memory_backend(tempdir.path(), mem_arc.clone())
        .build()
        .expect("agent builds");

    mem_arc.record(guardrail.clone()).expect("record guardrail");
    mem_arc.record(playbook.clone()).expect("record playbook");
    mem_arc.save_to_path(tempdir.path()).expect("save memory");

    let result = agent
        .critique(
            &Plan {
                goal: "boundary crossing".into(),
                goal_statement: "boundary crossing".into(),
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

    assert!(result.injected_learning_ids.contains(&guardrail.id));
    assert!(!result.injected_learning_ids.contains(&playbook.id));

    assert!(
        mem_arc.count() >= 2,
        "memory should have at least 2 entries"
    );
}

#[tokio::test]
async fn critique_no_memory_shows_no_blind_spots_section() {
    let config = AgentConfig {
        critique_personas: CritiquePersona::default_personas(),
        critique_mode: CritiqueMode::Full,
        ..Default::default()
    };

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
            score: 0.9,
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
            score: 0.9,
            issues: vec![],
        },
        PersonaResponse {
            system_prompt_contains: "adversarial",
            approved: true,
            score: 0.9,
            issues: vec![],
        },
    ]);
    let agent = Agent::builder()
        .llm(llm.clone())
        .tools(ToolRegistry::new())
        .config(config)
        .build()
        .expect("agent builds");

    let _ = agent
        .critique(
            &Plan {
                goal: "no memory".into(),
                goal_statement: "no memory".into(),
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

    let all_prompts = format!(
        "{}\n{}",
        llm.received_system_prompt(),
        llm.received_user_prompt()
    );
    assert!(!all_prompts.contains("Known blind spots"));
    assert!(all_prompts.contains("What the actor claims it did"));
}

#[tokio::test]
async fn critique_playbooks_excluded_from_blind_spots() {
    let guardrail = LearningEntry::guardrail(
        "memory leak on disconnect",
        "connection not closed",
        "Always close connections in a finally block.",
    );
    let playbook = LearningEntry::new(
        "successful pattern",
        "caching worked well",
        "Use Redis for caching.",
    );

    let config = AgentConfig {
        critique_personas: CritiquePersona::default_personas(),
        critique_mode: CritiqueMode::Full,
        ..Default::default()
    };
    let tempdir = tempfile::tempdir().expect("tempdir");

    let llm = PersonaScriptedLlm::new(vec![PersonaResponse {
        system_prompt_contains: "correctness",
        approved: true,
        score: 0.9,
        issues: vec![],
    }]);
    let mem_arc: std::sync::Arc<dyn crate::memory::Memory + Send + Sync> =
        std::sync::Arc::new(std::sync::Mutex::new(AgenticMemory::default()));
    let mem_for_test = mem_arc.clone();
    let agent = Agent::builder()
        .llm(llm.clone())
        .tools(ToolRegistry::new())
        .config(config)
        .memory_backend(tempdir.path(), mem_arc)
        .build()
        .expect("agent builds");

    mem_for_test
        .record(guardrail.clone())
        .expect("record guardrail");
    mem_for_test
        .record(playbook.clone())
        .expect("record playbook");
    mem_for_test
        .save_to_path(tempdir.path())
        .expect("save memory");

    let _ = agent
        .critique(
            &Plan {
                goal: "connection not closed".into(),
                goal_statement: "connection not closed".into(),
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

    let user_prompt = llm.received_user_prompt();
    assert!(
        user_prompt.contains("Always close connections"),
        "guardrail advice should appear in prompt"
    );
    assert!(
        !user_prompt.contains("Redis for caching"),
        "playbook should not appear in blind-spots prompt"
    );
}

#[tokio::test]
async fn critique_roundtrips_with_ensemble() {
    let guardrail = LearningEntry::guardrail(
        "unchecked unwrap",
        "potential panic",
        "Always handle Result types properly.",
    );

    let config = AgentConfig {
        critique_personas: CritiquePersona::default_personas(),
        critique_mode: CritiqueMode::Full,
        ..Default::default()
    };
    let tempdir = tempfile::tempdir().expect("tempdir");

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
    let mem_arc: std::sync::Arc<dyn crate::memory::Memory + Send + Sync> =
        std::sync::Arc::new(std::sync::Mutex::new(AgenticMemory::default()));
    let mem_for_test = mem_arc.clone();
    let agent = Agent::builder()
        .llm(llm.clone())
        .tools(ToolRegistry::new())
        .config(config)
        .memory_backend(tempdir.path(), mem_arc)
        .build()
        .expect("agent builds");

    mem_for_test
        .record(guardrail.clone())
        .expect("record guardrail");
    mem_for_test
        .save_to_path(tempdir.path())
        .expect("save memory");

    let result = agent
        .critique(
            &Plan {
                goal: "potential panic".into(),
                goal_statement: "potential panic".into(),
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
    assert!(result.injected_learning_ids.contains(&guardrail.id));
    assert_eq!(result.persona_breakdown.len(), 5);
}
