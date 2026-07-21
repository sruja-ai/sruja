use super::*;
use crate::cognition::subagent::{Role, SubAgentBudget, SubAgentSpec};

// --- Sub-agent isolation tests ---

#[test]
fn writer_subagent_has_no_exploration_tools() {
    let agent = isolated_agent();
    let names = agent.scoped_tool_names(Role::Writer);
    assert!(
        names.contains(&"file_write".to_string()),
        "writer needs file_write: {names:?}"
    );
    assert!(
        names.contains(&"file_edit".to_string()),
        "writer needs file_edit: {names:?}"
    );
    assert!(
        !names.contains(&"grep".to_string()),
        "writer must not have grep: {names:?}"
    );
    assert!(
        !names.contains(&"file_read".to_string()),
        "writer must not have file_read: {names:?}"
    );
    assert!(
        !names.iter().any(|n| n.starts_with("sruja_lookup")),
        "writer must not have lookup tools: {names:?}"
    );
}

#[test]
fn reader_subagent_has_no_write_tools() {
    let agent = isolated_agent();
    let names = agent.scoped_tool_names(Role::Reader);
    assert!(
        names.contains(&"grep".to_string()),
        "reader needs grep: {names:?}"
    );
    assert!(
        names.contains(&"sruja_focus".to_string()),
        "reader needs sruja_focus: {names:?}"
    );
    assert!(
        !names
            .iter()
            .any(|n| n.starts_with("file_write") || n.starts_with("file_edit")),
        "reader must not have write tools: {names:?}"
    );
}

#[tokio::test]
async fn delegate_reader_returns_compressed_report_with_citations() {
    let agent = isolated_agent();
    let spec = SubAgentSpec {
        role: Role::Reader,
        goal: crate::goal::GoalSpec::new("Review MySystem.ApiContainer for unused imports"),
        inject: vec!["Focus on src/api.rs".to_string()],
        budget: SubAgentBudget::default(),
        system_prompt: None,
        user_prompt: None,
    };
    let report = agent.delegate(spec).await.expect("delegate succeeds");
    assert!(report.converged, "single-shot LLM should converge");
    assert!(report.ok, "reader report should be ok");
    assert!(
        report.citations.iter().any(|c| c.starts_with("MySystem")),
        "citations should include architecture element IDs: {:?}",
        report.citations
    );
    assert!(
        report.summary.len() <= SubAgentBudget::default().max_summary_chars + 30,
        "summary must be bounded: len={}",
        report.summary.len()
    );
}
