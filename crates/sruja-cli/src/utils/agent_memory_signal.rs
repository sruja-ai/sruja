use crate::commands::CliError;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AgentMemorySignal {
    pub learnings_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_learning_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age_days: Option<u64>,
    pub is_stale: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<String>,
}

pub fn read_agent_memory_signal(repo_path: &Path) -> Result<Option<AgentMemorySignal>, CliError> {
    let p = repo_path.join(".sruja").join("agent_memory.json");
    if !p.exists() {
        return Ok(None);
    }

    let txt = std::fs::read_to_string(&p)?;
    let v: serde_json::Value =
        serde_json::from_str(&txt).map_err(|e| CliError::validation(e.to_string()))?;
    let learnings = v
        .get("learnings")
        .and_then(|l| l.as_array())
        .cloned()
        .unwrap_or_default();

    let learnings_count = learnings.len();
    let latest = learnings
        .iter()
        .filter_map(|e| {
            e.get("timestamp")
                .and_then(|t| t.as_str())
                .map(|s| s.to_string())
        })
        .filter_map(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| (dt, s))
        })
        .max_by_key(|(dt, _)| *dt)
        .map(|(_, s)| s);

    let (age_days, is_stale_by_time) = if let Some(ref ts) = latest {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
            let days = if duration.num_seconds() > 0 {
                (duration.num_seconds() as u64) / (60 * 60 * 24)
            } else {
                0
            };
            (Some(days), days >= 30)
        } else {
            (None, false)
        }
    } else {
        (None, true)
    };

    let is_stale_by_count = learnings_count < 5;
    let is_stale = is_stale_by_time || is_stale_by_count;

    let recommendation = if is_stale {
        Some(
            "Agent memory adoption looks low. When Sruja catches a miss, record a correction learning (guardrail) so future agents avoid repeating it."
                .to_string(),
        )
    } else {
        None
    };

    Ok(Some(AgentMemorySignal {
        learnings_count,
        latest_learning_at: latest,
        age_days,
        is_stale,
        recommendation,
    }))
}
