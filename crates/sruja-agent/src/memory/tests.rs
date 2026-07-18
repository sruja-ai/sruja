//! Tests for agentic memory.

use super::*;
use chrono::Utc;
use tempfile::tempdir;

fn make_entry(context: &str, hypothesis: &str, elements: Vec<&str>) -> LearningEntry {
    LearningEntry {
        id: types::generate_entry_id(),
        kind: None,
        timestamp: Utc::now(),
        run_id: None,
        repo: None,
        selector: None,
        context: context.to_string(),
        hypothesis: hypothesis.to_string(),
        outcome: ExperimentOutcome::Failed,
        reason: None,
        guardrail_advice: String::new(),
        affected_elements: elements.into_iter().map(String::from).collect(),
        evidence_refs: Vec::new(),
        confidence: None,
        tags: Vec::new(),
        hitl_kind: None,
        related_ids: Vec::new(),
        retrieval_count: 0,
        task_success_after: 0,
        task_total_after: 0,
        category: None,
        signals_match: Vec::new(),
        constraints: None,
        validation: Vec::new(),
        blast_radius: None,
    }
}

#[test]
fn test_utility_tracking() {
    let mut memory = AgenticMemory::default();
    let entry = make_entry("ctx", "hyp", vec!["API"]);
    let id = entry.id.clone();
    memory.add_learning(entry);

    assert_eq!(memory.learnings.len(), 1);
    assert_eq!(memory.learnings[0].id, id);
    AgenticMemory::record_retrievals(&mut memory, &[id.as_str()]);
    AgenticMemory::record_task_outcomes(&mut memory, &[id.as_str()], true);
    AgenticMemory::record_task_outcomes(&mut memory, &[id.as_str()], false);

    let e = &memory.learnings[0];
    assert_eq!(e.retrieval_count, 1);
    assert_eq!(e.task_total_after, 2);
    assert_eq!(e.task_success_after, 1);
    assert_eq!(e.utility_ratio(), Some(0.5));
}

#[test]
fn test_update_learning() {
    let mut memory = AgenticMemory::default();
    let entry = make_entry("old context", "hyp", vec![]);
    let id = entry.id.clone();
    memory.add_learning(entry);

    memory
        .update_learning(
            &id,
            LearningPatch {
                context: Some("new context".to_string()),
                guardrail_advice: Some("updated advice".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

    assert_eq!(memory.learnings[0].context, "new context");
    assert_eq!(memory.learnings[0].guardrail_advice, "updated advice");
    assert!(!memory.learnings[0].tags.is_empty());
}

#[test]
fn test_delete_learning() {
    let mut memory = AgenticMemory::default();
    let e1 = make_entry("a", "h1", vec![]);
    let id1 = e1.id.clone();
    let mut e2 = make_entry("b", "h2", vec![]);
    e2.related_ids = vec![id1.clone()];
    memory.add_learning(e1);
    memory.add_learning(e2);

    memory.delete_learning(&id1).unwrap();
    assert_eq!(memory.learnings.len(), 1);
    assert!(memory.learnings[0].related_ids.is_empty());
}

#[test]
fn test_merge_learnings() {
    let mut memory = AgenticMemory::default();
    let mut e1 = make_entry("shared topic A", "h1", vec!["API.Routes"]);
    e1.retrieval_count = 3;
    e1.task_success_after = 1;
    e1.task_total_after = 2;
    let id1 = e1.id.clone();
    let mut e2 = make_entry("shared topic B", "h2", vec!["API.Service"]);
    e2.retrieval_count = 2;
    let id2 = e2.id.clone();
    memory.add_learning(e1);
    memory.add_learning(e2);

    let merged_id = memory
        .merge_learnings(
            &[id1.clone(), id2.clone()],
            LearningEntry {
                context: "Merged boundary guidance".to_string(),
                hypothesis: "Combined".to_string(),
                outcome: ExperimentOutcome::Success,
                guardrail_advice: "Use service layer".to_string(),
                ..make_entry("", "", vec![])
            },
        )
        .unwrap();

    assert_eq!(memory.learnings.len(), 1);
    assert_eq!(memory.learnings[0].id, merged_id);
    assert_eq!(memory.learnings[0].retrieval_count, 5);
    assert_eq!(memory.learnings[0].task_total_after, 0);
    assert_eq!(memory.learnings[0].task_success_after, 0);
    assert!(memory.learnings[0]
        .affected_elements
        .iter()
        .any(|e| e == "API.Routes"));
}

#[test]
fn test_curation_report() {
    let mut memory = AgenticMemory::default();
    let mut e = make_entry("low utility", "h", vec![]);
    e.retrieval_count = 5;
    e.task_success_after = 0;
    e.task_total_after = 4;
    memory.add_learning(e);

    let report = memory.curation_report();
    assert_eq!(report.total_entries, 1);
    assert_eq!(report.low_utility.len(), 1);
}

#[test]
fn test_add_and_find_relevant() {
    let mut memory = AgenticMemory::default();
    let entry = LearningEntry {
        id: "test-1".to_string(),
        kind: None,
        timestamp: Utc::now(),
        run_id: None,
        repo: None,
        selector: None,
        context: "Refactoring API".to_string(),
        hypothesis: "Test hypothesis".to_string(),
        outcome: ExperimentOutcome::Success,
        reason: None,
        guardrail_advice: "Keep doing this".to_string(),
        affected_elements: vec!["Sruja.API".to_string(), "Sruja.CLI".to_string()],
        evidence_refs: Vec::new(),
        confidence: None,
        tags: Vec::new(),
        hitl_kind: None,
        related_ids: Vec::new(),
        retrieval_count: 0,
        task_success_after: 0,
        task_total_after: 0,
        category: None,
        signals_match: Vec::new(),
        constraints: None,
        validation: Vec::new(),
        blast_radius: None,
    };

    memory.add_learning(entry.clone());

    assert_eq!(memory.find_relevant("Sruja.API").len(), 1);
    assert_eq!(memory.find_relevant("Sruja.API.V1").len(), 1);
    assert_eq!(memory.find_relevant("api").len(), 1);
    assert_eq!(memory.find_relevant("Other").len(), 0);
}

#[test]
fn test_auto_tag_extraction() {
    let mut memory = AgenticMemory::default();
    let entry = LearningEntry {
        id: "tag-test".to_string(),
        kind: None,
        timestamp: Utc::now(),
        run_id: None,
        repo: None,
        selector: None,
        context: "Boundary violation in service layer".to_string(),
        hypothesis: "Direct database access from routes".to_string(),
        outcome: ExperimentOutcome::Failed,
        reason: None,
        guardrail_advice: "Always use service layer for database queries".to_string(),
        affected_elements: vec!["API.Routes".to_string()],
        evidence_refs: Vec::new(),
        confidence: None,
        tags: Vec::new(),
        hitl_kind: None,
        related_ids: Vec::new(),
        retrieval_count: 0,
        task_success_after: 0,
        task_total_after: 0,
        category: None,
        signals_match: Vec::new(),
        constraints: None,
        validation: Vec::new(),
        blast_radius: None,
    };
    memory.add_learning(entry);

    let tags = &memory.learnings[0].tags;
    assert!(!tags.is_empty(), "Tags should be auto-generated");
    assert!(
        tags.iter()
            .any(|t| t.contains("boundary") || t.contains("violation")),
        "Should extract domain-relevant tags: {:?}",
        tags
    );
}

#[test]
fn test_bidirectional_linking() {
    let mut memory = AgenticMemory::default();

    let e1 = make_entry(
        "Boundary violation refactoring",
        "Move DB calls to service layer",
        vec!["API.Routes", "API.Service"],
    );
    let e2 = make_entry(
        "Another boundary violation fix",
        "Extract repository pattern",
        vec!["API.Routes", "API.Repository"],
    );

    memory.add_learning(e1);
    memory.add_learning(e2);

    let first = &memory.learnings[0];
    let second = &memory.learnings[1];

    assert!(
        !first.related_ids.is_empty() || !second.related_ids.is_empty(),
        "Entries sharing affected elements should be linked"
    );
    if !second.related_ids.is_empty() {
        assert!(second.related_ids.contains(&first.id));
    }
    if !first.related_ids.is_empty() {
        assert!(first.related_ids.contains(&second.id));
    }
}

#[test]
fn test_find_cluster() {
    let mut memory = AgenticMemory::default();

    let e1 = make_entry("Boundary violation A", "hypothesis A", vec!["API.Routes"]);
    let e2 = make_entry("Boundary violation B", "hypothesis B", vec!["API.Routes"]);
    let e3 = make_entry("Unrelated topic", "hypothesis C", vec!["Database.Schema"]);

    memory.add_learning(e1);
    let id1 = memory.learnings[0].id.clone();
    memory.add_learning(e2);
    memory.add_learning(e3);

    let cluster = memory.find_cluster(&id1);
    assert!(
        cluster.len() >= 2,
        "Cluster should include linked entries, got {}",
        cluster.len()
    );

    let cluster_ids: Vec<&str> = cluster.iter().map(|e| e.id.as_str()).collect();
    assert!(cluster_ids.contains(&id1.as_str()));
}

#[test]
fn test_find_by_tag() {
    let mut memory = AgenticMemory::default();
    let mut entry = make_entry("boundary violation test", "hypothesis", vec!["API"]);
    entry.tags = vec!["boundary".to_string(), "violation".to_string()];
    memory.add_learning_raw(entry);

    let results = memory.find_by_tag("boundary");
    assert_eq!(results.len(), 1);

    let results = memory.find_by_tag("nonexistent");
    assert!(results.is_empty());
}

#[test]
fn test_all_tags() {
    let mut memory = AgenticMemory::default();
    let mut e1 = make_entry("ctx", "hyp", vec![]);
    e1.tags = vec!["alpha".to_string(), "beta".to_string()];
    let mut e2 = make_entry("ctx", "hyp", vec![]);
    e2.tags = vec!["beta".to_string(), "gamma".to_string()];
    memory.add_learning_raw(e1);
    memory.add_learning_raw(e2);

    let tags = memory.all_tags();
    assert_eq!(tags, vec!["alpha", "beta", "gamma"]);
}

#[test]
fn test_save_and_load() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();
    let mut memory = AgenticMemory::default();
    let entry = LearningEntry {
        id: "save-test".to_string(),
        kind: None,
        timestamp: Utc::now(),
        run_id: None,
        repo: None,
        selector: None,
        context: "Test".to_string(),
        hypothesis: "Hypo".to_string(),
        outcome: ExperimentOutcome::Failed,
        reason: Some("Error".to_string()),
        guardrail_advice: "Don't".to_string(),
        affected_elements: vec!["ID1".to_string()],
        evidence_refs: Vec::new(),
        confidence: None,
        tags: Vec::new(),
        hitl_kind: None,
        related_ids: Vec::new(),
        retrieval_count: 0,
        task_success_after: 0,
        task_total_after: 0,
        category: None,
        signals_match: Vec::new(),
        constraints: None,
        validation: Vec::new(),
        blast_radius: None,
    };

    memory.add_learning(entry);
    memory.save(repo_root).unwrap();

    let loaded = AgenticMemory::load(repo_root).unwrap();
    assert_eq!(loaded.learnings.len(), 1);
    assert_eq!(loaded.learnings[0].context, "Test");
    assert!(!loaded.learnings[0].id.is_empty(), "ID should persist");
}

#[test]
fn test_load_nonexistent() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();

    let loaded = AgenticMemory::load(repo_root).unwrap();
    assert_eq!(loaded.learnings.len(), 0);
}

#[test]
fn test_clear_and_exists() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();

    assert!(!AgenticMemory::exists(repo_root));

    let memory = AgenticMemory::default();
    memory.save(repo_root).unwrap();

    assert!(AgenticMemory::exists(repo_root));

    AgenticMemory::clear(repo_root).unwrap();

    assert!(!AgenticMemory::exists(repo_root));
}

#[test]
fn test_load_invalid_json() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();
    let path = AgenticMemory::get_path(repo_root);

    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "invalid json").unwrap();

    let result = AgenticMemory::load(repo_root);
    assert!(matches!(result, Err(MemoryError::Serialization(_))));
}

#[test]
fn test_find_relevant_edge_cases() {
    let mut memory = AgenticMemory::default();
    let entry = LearningEntry {
        id: "edge-case".to_string(),
        kind: None,
        timestamp: Utc::now(),
        run_id: None,
        repo: None,
        selector: None,
        context: "Some Context".to_string(),
        hypothesis: "".to_string(),
        outcome: ExperimentOutcome::Success,
        reason: None,
        guardrail_advice: "".to_string(),
        affected_elements: vec!["Sruja.API.V1".to_string()],
        evidence_refs: Vec::new(),
        confidence: None,
        tags: Vec::new(),
        hitl_kind: None,
        related_ids: Vec::new(),
        retrieval_count: 0,
        task_success_after: 0,
        task_total_after: 0,
        category: None,
        signals_match: Vec::new(),
        constraints: None,
        validation: Vec::new(),
        blast_radius: None,
    };
    memory.add_learning(entry);

    assert_eq!(memory.find_relevant("Sruja.API").len(), 1);
    assert_eq!(memory.find_relevant("Sruja.API.V1.Endpoint").len(), 1);
    assert_eq!(memory.find_relevant("Sruja.API.V1").len(), 1);
    assert_eq!(memory.find_relevant("Sruja.API.V2").len(), 0);
}

#[test]
fn test_load_save_custom_path() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("custom_memory.json");
    let mut memory = AgenticMemory::default();
    memory.add_learning(LearningEntry {
        id: "custom-path".to_string(),
        kind: None,
        timestamp: Utc::now(),
        run_id: None,
        repo: None,
        selector: None,
        context: "Custom".to_string(),
        hypothesis: "H".to_string(),
        outcome: ExperimentOutcome::Success,
        reason: None,
        guardrail_advice: "G".to_string(),
        affected_elements: vec![],
        evidence_refs: Vec::new(),
        confidence: None,
        tags: Vec::new(),
        hitl_kind: None,
        related_ids: Vec::new(),
        retrieval_count: 0,
        task_success_after: 0,
        task_total_after: 0,
        category: None,
        signals_match: Vec::new(),
        constraints: None,
        validation: Vec::new(),
        blast_radius: None,
    });

    memory.save_to_path(&path).unwrap();
    assert!(path.exists());

    let loaded = AgenticMemory::load_from_path(&path).unwrap();
    assert_eq!(loaded.learnings.len(), 1);
    assert_eq!(loaded.learnings[0].context, "Custom");
}

#[test]
fn test_learning_entry_relevance() {
    let entry = LearningEntry {
        id: "rel-test".to_string(),
        kind: None,
        timestamp: Utc::now(),
        run_id: None,
        repo: None,
        selector: None,
        context: "API Refactoring".to_string(),
        hypothesis: "".to_string(),
        outcome: ExperimentOutcome::Success,
        reason: None,
        guardrail_advice: "".to_string(),
        affected_elements: vec!["System.Core".to_string()],
        evidence_refs: Vec::new(),
        confidence: None,
        tags: Vec::new(),
        hitl_kind: None,
        related_ids: Vec::new(),
        retrieval_count: 0,
        task_success_after: 0,
        task_total_after: 0,
        category: None,
        signals_match: Vec::new(),
        constraints: None,
        validation: Vec::new(),
        blast_radius: None,
    };

    assert!(entry.is_relevant_to("System.Core"));
    assert!(entry.is_relevant_to("System.Core.UI"));
    assert!(entry.is_relevant_to("System"));
    assert!(entry.is_relevant_to("api"));
    assert!(!entry.is_relevant_to("Database"));
}

#[test]
fn test_save_to_path_replaces_longer_existing_content() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("memory.json");
    let mut memory = AgenticMemory::default();
    memory.add_learning(LearningEntry {
        id: "replace-test".to_string(),
        kind: None,
        timestamp: Utc::now(),
        run_id: None,
        repo: None,
        selector: None,
        context: "Short".to_string(),
        hypothesis: "H".to_string(),
        outcome: ExperimentOutcome::Success,
        reason: None,
        guardrail_advice: "G".to_string(),
        affected_elements: vec![],
        evidence_refs: Vec::new(),
        confidence: None,
        tags: Vec::new(),
        hitl_kind: None,
        related_ids: Vec::new(),
        retrieval_count: 0,
        task_success_after: 0,
        task_total_after: 0,
        category: None,
        signals_match: Vec::new(),
        constraints: None,
        validation: Vec::new(),
        blast_radius: None,
    });

    std::fs::write(&path, "{".repeat(4096)).unwrap();

    memory.save_to_path(&path).unwrap();

    let loaded = AgenticMemory::load_from_path(&path).unwrap();
    assert_eq!(loaded.learnings.len(), 1);
    assert_eq!(loaded.learnings[0].context, "Short");
}

#[test]
fn test_backward_compatible_deserialization() {
    let legacy_json = r#"{
        "learnings": [{
            "timestamp": "2026-01-01T00:00:00Z",
            "context": "Legacy entry",
            "hypothesis": "Old format",
            "outcome": "failed",
            "reason": null,
            "guardrail_advice": "Upgrade",
            "affected_elements": ["X"]
        }]
    }"#;
    let memory: AgenticMemory = serde_json::from_str(legacy_json).unwrap();
    assert_eq!(memory.learnings.len(), 1);
    assert!(memory.learnings[0].tags.is_empty());
    assert!(memory.learnings[0].related_ids.is_empty());
    assert!(memory.learnings[0].kind.is_none());
    assert!(memory.learnings[0].run_id.is_none());
}

#[test]
fn test_decay_score_recent_entry() {
    let entry = make_entry("ctx", "hyp", vec![]);
    let score = entry.decay_score();
    assert!(
        score > 0.9,
        "Recent entry should have high decay score, got {}",
        score
    );
}

#[test]
fn test_decay_score_old_entry() {
    let mut entry = make_entry("ctx", "hyp", vec![]);
    entry.timestamp = Utc::now() - chrono::Duration::days(250);
    let score = entry.decay_score();
    assert!(
        score < 0.15,
        "Old entry should have low decay score, got {}",
        score
    );
}

#[test]
fn test_decay_score_with_retrievals() {
    let mut entry = make_entry("ctx", "hyp", vec![]);
    entry.timestamp = Utc::now() - chrono::Duration::days(120);
    let score_no_retrievals = entry.decay_score();
    entry.retrieval_count = 10;
    let score_with_retrievals = entry.decay_score();
    assert!(
        score_with_retrievals > score_no_retrievals,
        "Retrievals should boost decay score: {} vs {}",
        score_with_retrievals,
        score_no_retrievals
    );
}

#[test]
fn test_curation_report_includes_stale() {
    let mut memory = AgenticMemory::default();
    let mut old = make_entry("old context", "hyp", vec![]);
    old.timestamp = Utc::now() - chrono::Duration::days(250);
    old.retrieval_count = 0;
    memory.add_learning(old);

    let report = memory.curation_report();
    assert_eq!(report.stale_entries.len(), 1, "Old entry should be stale");
    assert!(report.stale_entries[0].decay_score < 0.15);
}

#[test]
fn test_auto_archive_stale() {
    let mut memory = AgenticMemory::default();
    let mut old = make_entry("old", "hyp", vec![]);
    old.timestamp = Utc::now() - chrono::Duration::days(250);
    memory.add_learning(old);

    let recent = make_entry("recent", "hyp", vec![]);
    memory.add_learning(recent);

    let archived = memory.auto_archive_stale(0.15, 30);
    assert_eq!(archived.len(), 1, "Should archive old entry");
    assert_eq!(memory.learnings.len(), 1, "Recent entry should remain");
}

#[test]
fn test_auto_archive_preserves_invariants() {
    let mut memory = AgenticMemory::default();
    let mut old_invariant = make_entry("invariant", "hyp", vec![]);
    old_invariant.timestamp = Utc::now() - chrono::Duration::days(250);
    old_invariant.kind = Some(LearningKind::Invariant);
    memory.add_learning(old_invariant);

    let archived = memory.auto_archive_stale(0.15, 30);
    assert_eq!(archived.len(), 0, "Should not archive invariants");
    assert_eq!(memory.learnings.len(), 1);
}

#[test]
fn test_category_filter_on_search() {
    use super::types::LearningCategory;

    let mem = std::sync::Mutex::new(AgenticMemory::default());

    let mut e1 = make_entry("repair the boundary violation", "fix", vec!["API"]);
    e1.category = Some(LearningCategory::Repair);
    mem.lock().unwrap().add_learning(e1);

    let mut e2 = make_entry("optimize the query performance", "speed", vec!["API"]);
    e2.category = Some(LearningCategory::Optimize);
    mem.lock().unwrap().add_learning(e2);

    let mut e3 = make_entry("repair the database connection", "fix", vec!["DB"]);
    e3.category = Some(LearningCategory::Repair);
    mem.lock().unwrap().add_learning(e3);

    // Query by affected element — all 3 match because they share "API" or "DB" but
    // we use a broad text query "the" that appears in all contexts.
    let all = mem.search("repair", 10, None);
    // All entries whose context contains "repair" should be returned.
    assert!(
        all.len() >= 2,
        "at least repair entries should match, got {}",
        all.len()
    );

    // Filter to Repair only.
    let repair = mem.search("repair", 10, Some(LearningCategory::Repair));
    assert!(
        !repair.is_empty(),
        "Repair filter should return at least 1 entry"
    );
    assert!(repair
        .iter()
        .all(|e| e.category == Some(LearningCategory::Repair)));

    // Filter to Innovate — none should match.
    let innovate = mem.search("repair", 10, Some(LearningCategory::Innovate));
    assert_eq!(innovate.len(), 0, "Innovate filter should return 0 entries");
}
