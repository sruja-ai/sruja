//! Response parsing helpers for LLM JSON output.
//!
//! These functions extract structured data (plans, critiques, learnings)
//! from LLM responses that may be wrapped in markdown code fences.

use super::*;

pub fn parse_plan_from_response(
    content: &str,
    goal: &crate::goal::GoalSpec,
    tdd: bool,
) -> Result<Plan, PlanParseError> {
    let goal_str = goal.statement.as_str();
    // Try to extract JSON from the response (may be wrapped in markdown).
    let json_str = extract_json(content);

    let value: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| PlanParseError::MalformedJson(format!("failed to parse JSON: {e}")))?;

    // Validate schema_version if present.
    if let Some(sv) = value.get("schema_version").and_then(|v| v.as_str()) {
        if sv != "1.0" {
            tracing::warn!(
                schema_version = sv,
                "plan has unexpected schema_version — proceeding anyway"
            );
        }
    }

    let subtasks_raw =
        value
            .get("subtasks")
            .and_then(|s| s.as_array())
            .ok_or(PlanParseError::MalformedJson(
                "missing or non-array `subtasks` field".to_string(),
            ))?;

    if subtasks_raw.is_empty() {
        return Err(PlanParseError::NoSubtasks);
    }

    let mut subtasks = Vec::new();
    for (idx, st) in subtasks_raw.iter().enumerate() {
        let id = st
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PlanParseError::MissingRequiredField {
                field: "id".to_string(),
                subtask_index: idx,
            })?
            .to_string();
        let description = st
            .get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PlanParseError::MissingRequiredField {
                field: "description".to_string(),
                subtask_index: idx,
            })?
            .to_string();
        let tier_str = st.get("tier").and_then(|v| v.as_str()).ok_or_else(|| {
            PlanParseError::MissingRequiredField {
                field: "tier".to_string(),
                subtask_index: idx,
            }
        })?;
        let kind_str = st.get("kind").and_then(|v| v.as_str()).ok_or_else(|| {
            PlanParseError::MissingRequiredField {
                field: "kind".to_string(),
                subtask_index: idx,
            }
        })?;

        subtasks.push(Subtask {
            id,
            description,
            tier: parse_tier(tier_str),
            kind: parse_kind(kind_str),
            files: st
                .get("files")
                .and_then(|f| f.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            acceptance_criteria: st
                .get("acceptance_criteria")
                .and_then(|a| a.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        });
    }

    let risks: Vec<String> = value
        .get("risks")
        .and_then(|r| r.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let schema_version = value
        .get("schema_version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(Plan {
        goal: goal_str.to_string(),
        goal_statement: goal.statement.clone(),
        criteria: goal.acceptance_criteria.clone(),
        subtasks,
        tdd,
        risks,
        schema_version,
        complexity: TaskComplexity::default(),
    })
}

pub(super) fn parse_critique_from_response(content: &str, usage: Usage) -> Critique {
    let json_str = extract_json(content);

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_str) {
        return Critique {
            approved: value
                .get("approved")
                .and_then(|a| a.as_bool())
                .unwrap_or(false),
            score: value.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0),
            issues: value
                .get("issues")
                .and_then(|i| i.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            suggestions: value
                .get("suggestions")
                .and_then(|s| s.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            usage,
            persona_breakdown: Vec::new(),
            injected_learning_ids: Vec::new(),
            criteria: value
                .get("criteria")
                .and_then(|c| c.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| {
                            let index = v.get("index")?.as_u64()? as usize;
                            let criterion = v.get("criterion")?.as_str()?.to_string();
                            let status = v.get("status")?.as_str()?;
                            let reason = v
                                .get("reason")
                                .and_then(|r| r.as_str())
                                .unwrap_or("")
                                .to_string();
                            let verdict = match status {
                                "addressed" => CriterionVerdict::Addressed,
                                "partial" => CriterionVerdict::Partial,
                                "missing" => CriterionVerdict::Missing,
                                _ => return None,
                            };
                            Some(CriterionStatus {
                                index,
                                criterion,
                                status: verdict,
                                reason,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default(),
            source: String::new(),
        };
    }

    // Fallback: check for approve/reject keywords.
    let lower = content.to_lowercase();
    let is_approved = lower.contains("\napproved") || lower.starts_with("approved");

    Critique {
        approved: is_approved,
        score: if is_approved { 0.8 } else { 0.3 },
        issues: vec!["could not parse structured critique".into()],
        suggestions: Vec::new(),
        usage,
        persona_breakdown: Vec::new(),
        injected_learning_ids: Vec::new(),
        criteria: Vec::new(),
        source: String::new(),
    }
}

pub(super) fn parse_learnings_from_response(content: &str) -> Vec<LearningEntry> {
    let json_str = extract_json(content);

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_str) {
        let arr = match &value {
            serde_json::Value::Array(a) => a.clone(),
            serde_json::Value::Object(_) => vec![value.clone()],
            _ => return Vec::new(),
        };

        return arr
            .iter()
            .filter_map(|v| {
                let context = v.get("context")?.as_str()?;
                let hypothesis = v.get("hypothesis")?.as_str()?;
                let advice = v.get("guardrail_advice")?.as_str()?;
                let kind = v.get("kind").and_then(|k| k.as_str()).unwrap_or("playbook");

                Some(match kind {
                    "guardrail" => LearningEntry::guardrail(context, hypothesis, advice),
                    _ => LearningEntry::playbook(context, hypothesis, advice),
                })
            })
            .collect();
    }

    Vec::new()
}

/// Extract JSON from a response that may contain markdown code fences.
pub(super) fn extract_json(content: &str) -> String {
    // Try to find JSON in code fences.
    if let Some(start) = content.find("```json") {
        let rest = &content[start + 7..];
        if let Some(end) = rest.find("```") {
            return rest[..end].trim().to_string();
        }
    }
    if let Some(start) = content.find("```") {
        let rest = &content[start + 3..];
        // Skip optional language tag.
        let rest = rest.lines().skip(1).collect::<Vec<_>>().join("\n");
        if let Some(end) = rest.find("```") {
            return rest[..end].trim().to_string();
        }
    }
    // Try to find a JSON object or array directly.
    if let Some(start) = content.find('{') {
        if let Some(end) = content.rfind('}') {
            return content[start..=end].to_string();
        }
    }
    if let Some(start) = content.find('[') {
        if let Some(end) = content.rfind(']') {
            return content[start..=end].to_string();
        }
    }
    content.to_string()
}

/// Known file extensions to recognize when scanning critique issues.
const KNOWN_EXTENSIONS: &[&str] = &[
    ".rs", ".ts", ".tsx", ".js", ".jsx", ".toml", ".md", ".json", ".yaml", ".yml",
];

/// Known file-path indicator phrases.
const FILE_INDICATORS: &[&str] = &["in file", "file:", "at "];

/// Extract file path and line number references from critique issues.
///
/// Uses simple string scanning to find patterns like:
/// - `src/foo.rs:42` (file with line number)
/// - `in file src/foo.rs` (file-level reference)
/// - `[correctness] src/foo.rs:12 – null input` (persona-prefixed)
///
/// No regex crate needed — all matching is done via `str::find` and `split`.
pub fn extract_file_references(issues: &[String]) -> Vec<(String, Vec<usize>)> {
    use std::collections::BTreeMap;

    let mut refs: BTreeMap<String, Vec<usize>> = BTreeMap::new();

    for issue in issues {
        // Find file:line patterns by scanning for known extensions followed by :digits
        let mut pos = 0;
        while pos < issue.len() {
            // Scan for a known extension
            let mut found_ext = None;
            for ext in KNOWN_EXTENSIONS {
                if let Some(ext_pos) = issue[pos..].find(ext) {
                    let abs_pos = pos + ext_pos;
                    let after_ext = abs_pos + ext.len();
                    // Check if followed by :digits
                    if after_ext < issue.len() && issue.as_bytes()[after_ext] == b':' {
                        let rest = &issue[after_ext + 1..];
                        let line_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                        if !line_str.is_empty() {
                            // Walk backward from extension to find path start
                            let path_start = abs_pos.saturating_sub(1);
                            let file_start = issue[..=path_start]
                                .rfind(|c: char| c == ' ' || c == '\t' || c == '`' || c == '–' || c == '-')
                                .map(|p| p + 1)
                                .unwrap_or(0);
                            let file = &issue[file_start..after_ext];
                            let file = file.trim().to_string();
                            if !file.is_empty() && !file.contains(' ') {
                                let line: usize = line_str.parse().unwrap_or(0);
                                refs.entry(file).or_default().push(line);
                            }
                            pos = after_ext + 1 + line_str.len();
                            found_ext = Some(());
                            break;
                        }
                    }
                    // No :digits — just a file reference (file-level)
                    let path_start = abs_pos.saturating_sub(1);
                    let file_start = issue[..=path_start]
                        .rfind(|c: char| c == ' ' || c == '\t' || c == '`' || c == '–' || c == '-')
                        .map(|p| p + 1)
                        .unwrap_or(0);
                    let file = &issue[file_start..after_ext];
                    let file = file.trim().to_string();
                    if !file.is_empty() && !file.contains(' ') {
                        refs.entry(file).or_default();
                    }
                    pos = after_ext;
                    found_ext = Some(());
                    break;
                }
            }
            if found_ext.is_none() {
                pos += 1;
            }
        }

        // Find "in file ..." or "file: ..." indicator patterns
        for indicator in FILE_INDICATORS {
            let mut search_pos = 0;
            while let Some(idx) = issue[search_pos..].find(indicator) {
                let abs_idx = search_pos + idx;
                let after = abs_idx + indicator.len();
                // Take the next whitespace-delimited token
                let rest = &issue[after..];
                let token: String = rest.chars().take_while(|c| !c.is_whitespace() && *c != ',' && *c != '.').collect();
                if !token.is_empty() && KNOWN_EXTENSIONS.iter().any(|e| token.ends_with(e)) {
                    refs.entry(token).or_default();
                }
                search_pos = after + 1;
            }
        }
    }

    // Deduplicate and sort line numbers per file
    for lines in refs.values_mut() {
        lines.sort();
        lines.dedup();
    }

    refs.into_iter().collect()
}

fn parse_tier(s: &str) -> TaskTier {
    match s.to_lowercase().as_str() {
        "cheap" | "low" | "simple" => TaskTier::Cheap,
        "premium" | "high" | "complex" | "hard" => TaskTier::Premium,
        _ => TaskTier::Mid,
    }
}

fn parse_kind(s: &str) -> SubtaskKind {
    match s.to_lowercase().as_str() {
        "test_author" | "test" | "write_test" | "testing" => SubtaskKind::TestAuthor,
        "implement" | "code" | "implementing" => SubtaskKind::Implement,
        "verify" | "verification" | "check" => SubtaskKind::Verify,
        "review" | "critique" => SubtaskKind::Review,
        _ => SubtaskKind::Comprehend,
    }
}
