//! Prompt templates and JSON envelope parsing for grounded AI answers.
//!
//! LLM must return a strict JSON envelope; parse fails => do not write new facts.

use serde::{Deserialize, Serialize};

use crate::commands::CliError;

/// Machine-parsable envelope returned by the model (plan §8.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub answer_markdown: String,
    pub confidence: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub facts: Vec<EnvelopeFact>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assumptions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gaps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeFact {
    pub statement: String,
    pub fact_type: String, // flow|boundary|dependency|...
    pub confidence: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_paths: Vec<String>,
}

/// Extract JSON envelope from model response. Handles markdown code fence.
/// Returns Err if no valid JSON object found.
pub fn parse_envelope(raw: &str) -> Result<Envelope, CliError> {
    let trimmed = raw.trim();
    let json_str = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```").map(|t| t.trim()))
        .unwrap_or(trimmed);

    let start = json_str.find('{').ok_or_else(|| {
        CliError::Validation("LLM response did not contain a JSON object".to_string())
    })?;
    let end = json_str.rfind('}').ok_or_else(|| {
        CliError::Validation("LLM response did not contain a JSON object".to_string())
    })? + 1;
    let obj_str = &json_str[start..end];

    serde_json::from_str(obj_str).map_err(|e| {
        CliError::Validation(format!(
            "Could not parse LLM response as envelope JSON: {}",
            e
        ))
    })
}

/// System prompt for architecture explain/ask: cite only evidence, return envelope.
pub const EXPLAIN_SYSTEM: &str = r#"You are an architecture explainer. You answer only from the provided evidence (scan graph summary and file paths). Do not invent facts.

Rules:
1. Cite only file paths or nodes that appear in the evidence section.
2. Mark any assumptions explicitly in the "assumptions" array.
3. Put confidence in [0.0, 1.0] based on how well the evidence supports the answer.
4. Reply with a single JSON object (no markdown around it) with: answer_markdown, confidence, facts (array of { statement, fact_type, confidence, evidence_paths }), assumptions, gaps.

fact_type must be one of: flow, boundary, dependency, decision, risk, ownership.
evidence_paths must be a subset of the file paths provided in the evidence. Do not cite paths not in the evidence."#;

/// Build user prompt: topic or question + evidence block.
pub fn explain_user_prompt(topic: &str, evidence_block: &str) -> String {
    format!(
        r#"Topic: {}

Evidence (from repository scan and memory):
{}
{}
Reply with only the JSON object (no explanation before or after)."#,
        topic,
        evidence_block,
        "Return JSON with keys: answer_markdown, confidence, facts, assumptions, gaps."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_envelope_pure_json() {
        let raw =
            r#"{"answer_markdown":"X","confidence":0.8,"facts":[],"assumptions":[],"gaps":[]}"#;
        let e = parse_envelope(raw).unwrap();
        assert_eq!(e.answer_markdown, "X");
        assert!((e.confidence - 0.8).abs() < 1e-9);
    }

    #[test]
    fn parse_envelope_with_code_fence() {
        let raw = r#"```json
{"answer_markdown":"Y","confidence":0.5,"facts":[{"statement":"S","fact_type":"flow","confidence":0.6,"evidence_paths":[]}],"assumptions":[],"gaps":[]}
```"#;
        let e = parse_envelope(raw).unwrap();
        assert_eq!(e.answer_markdown, "Y");
        assert_eq!(e.facts.len(), 1);
        assert_eq!(e.facts[0].statement, "S");
    }

    #[test]
    fn parse_envelope_missing_object_fails() {
        assert!(parse_envelope("no json here").is_err());
        assert!(parse_envelope("[]").is_err());
    }

    /// Golden sample: full envelope as in plan §8.2; parsing must remain stable.
    const GOLDEN_ENVELOPE: &str = r#"{"answer_markdown":"Requests enter via the API gateway.","confidence":0.73,"facts":[{"statement":"HTTP requests enter through API gateway.","fact_type":"flow","confidence":0.68,"evidence_paths":["crates/api/src/gateway.rs"]},{"statement":"Service routing is centralized.","fact_type":"boundary","confidence":0.71,"evidence_paths":["src/router.rs"]}],"assumptions":["Auth is assumed already applied."],"gaps":["No evidence for rate limiting."]}"#;

    #[test]
    fn parse_envelope_golden_full_structure() {
        let e = parse_envelope(GOLDEN_ENVELOPE).unwrap();
        assert_eq!(e.answer_markdown, "Requests enter via the API gateway.");
        assert!((e.confidence - 0.73).abs() < 1e-9);
        assert_eq!(e.facts.len(), 2);
        assert_eq!(
            e.facts[0].statement,
            "HTTP requests enter through API gateway."
        );
        assert_eq!(e.facts[0].fact_type, "flow");
        assert_eq!(e.facts[0].evidence_paths, &["crates/api/src/gateway.rs"]);
        assert_eq!(e.facts[1].fact_type, "boundary");
        assert_eq!(e.assumptions, &["Auth is assumed already applied."]);
        assert_eq!(e.gaps, &["No evidence for rate limiting."]);
    }
}
