use super::*;
use serde_json::Value;

#[test]
fn mcp_resources_list_includes_invariant_uri() {
    let resources = crate::commands::mcp_resources::list_resources(".").expect("list");
    assert!(resources
        .iter()
        .any(|r| r.uri == "sruja://context/invariant.md"));
}

#[test]
fn mcp_prompts_list_includes_mcp_guide() {
    let prompts = crate::commands::mcp_prompts::list_prompts();
    assert!(prompts.iter().any(|p| p.name == "sruja_mcp_guide"));
}

#[test]
fn invariant_brief_fits_token_budget() {
    let ctx = crate::commands::context::types::ArchitectureContext {
        repo: "test".to_string(),
        summary: crate::commands::context::types::ContextSummary {
            total_crates: None,
            total_modules: 10,
            total_services: 2,
            total_databases: 1,
            total_external_apis: 0,
        },
        layers: vec![],
        boundaries: vec![crate::commands::context::types::BoundaryRule {
            from: "ui".to_string(),
            to: "data".to_string(),
            allowed: false,
            reason: "use services".to_string(),
        }],
        forbidden_patterns: vec![],
        active_decisions: vec![],
        focus: None,
        system_context: None,
        max_tokens: 700,
    };
    let brief = crate::commands::context::format_invariant_brief(&ctx);
    assert!(brief.len() < 4_000, "brief should stay compact");
    assert!(brief.contains("sruja_list_architecture_index"));
}
