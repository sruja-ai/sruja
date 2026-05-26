use serde_json::Value;

pub(crate) fn append_mcp_tool_call_event(
    repo: &str,
    tool: &str,
    args: &Value,
    run_id: Option<&str>,
    ok: bool,
    error: Option<&str>,
    elapsed_ms: u64,
) -> Result<(), String> {
    let repo_path = std::path::Path::new(repo);

    let elements = args
        .get("elements")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .or_else(|| {
            args.get("element_id")
                .and_then(|v| v.as_str())
                .map(|s| vec![s.to_string()])
        });

    let args_keys = args
        .as_object()
        .map(|m| {
            let mut keys = m.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            keys
        })
        .unwrap_or_default();

    let record = crate::commands::context_events::ContextEventRecord {
        schema_version: crate::commands::context_events::CONTEXT_EVENTS_SCHEMA_V2.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        kind: "context_retrieved".to_string(),
        outcome: if ok {
            "ok".to_string()
        } else {
            "fail".to_string()
        },
        policy_fingerprint: crate::commands::context_events::policy_fingerprint(repo_path),
        strict: None,
        details: serde_json::json!({
            "repo": repo,
            "tool": tool,
            "elapsed_ms": elapsed_ms,
            "ok": ok,
            "error": error,
            "args_keys": args_keys,
        }),
        trace_id: run_id.map(|s| s.to_string()),
        decision_id: None,
        run_id: run_id.map(|s| s.to_string()),
        workflow_id: None,
        actor: Some("agent".to_string()),
        source: Some("mcp".to_string()),
        tool: Some(tool.to_string()),
        elements,
        subject_ids: None,
        evidence_refs: None,
        summary: Some(format!("mcp tools/call: {tool}")),
        ..Default::default()
    };

    crate::commands::context_events::validate_context_event_record(&record)?;
    crate::commands::context_events::append_context_event(repo_path, record);
    Ok(())
}
