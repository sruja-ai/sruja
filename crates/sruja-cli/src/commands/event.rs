//! `sruja event` — append and list `.sruja/context_events.jsonl` rows (including v2 decision traces).

use crate::commands::context_events::{
    append_context_event_from_json_line, read_context_events_query, ContextEventQuery,
};
use crate::commands::CliError;
use std::io::Read;
use std::path::Path;

pub async fn event_append(repo: &str, json: Option<&str>) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let line = if let Some(j) = json {
        j.to_string()
    } else {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        // one line from stdin
        buf.lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .to_string()
    };
    append_context_event_from_json_line(repo_path, &line).map_err(CliError::validation)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn event_list(
    repo: &str,
    format: &str,
    limit: usize,
    kind: Option<&str>,
    details_substring: Option<&str>,
    decision_id: Option<&str>,
    trace_id: Option<&str>,
    element_id: Option<&str>,
    decision_lineage_only: bool,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let events = read_context_events_query(
        repo_path,
        ContextEventQuery {
            limit,
            kind_filter: kind,
            details_substring,
            decision_id,
            trace_id,
            run_id: None,
            element_id,
            decision_lineage_only,
        },
    )
    .map_err(CliError::Io)?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&events)?);
        return Ok(());
    }

    for ev in &events {
        println!(
            "{}  {}  {}  {}",
            ev.timestamp,
            ev.kind,
            ev.outcome,
            ev.summary.as_deref().unwrap_or("")
        );
    }
    Ok(())
}
