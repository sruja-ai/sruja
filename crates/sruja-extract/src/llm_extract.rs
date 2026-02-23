//! LLM-based extraction of architecture intents from conversation messages.
//!
//! Uses Rig with OpenRouter for LLM calls.

use chrono::Utc;
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Prompt;
use rig::providers::openrouter;
use serde::Deserialize;
use sruja_graph::{generate_id, NodeKind, RequirementPriority};

use crate::{
    ConstraintType, ConversationMessage, ExtractedContent, Extraction, ExtractionEngine,
    ExtractionStatus, Intent, RiskSeverity,
};

#[derive(Debug, Deserialize)]
struct LlmExtraction {
    intent: String,
    confidence: f32,
    #[serde(rename = "content")]
    content_json: serde_json::Value,
}

/// Extract architecture intents from a message using an LLM.
pub async fn extract_from_message_async(
    _engine: &ExtractionEngine,
    message: &ConversationMessage,
) -> Result<Vec<Extraction>, String> {
    let model_name = std::env::var("SRUJA_EXTRACTION_MODEL")
        .unwrap_or_else(|_| "openai/gpt-4o-mini".to_string());

    let client = openrouter::Client::from_env();
    let rig_agent = client
        .agent(&model_name)
        .preamble(EXTRACTION_SYSTEM_PROMPT)
        .build();

    let prompt = build_extraction_prompt(&message.content);

    let text: String = rig_agent
        .prompt(prompt)
        .await
        .map_err(|e| format!("LLM request failed: {}", e))?;
    parse_llm_response(text.trim(), message)
}

const EXTRACTION_SYSTEM_PROMPT: &str = r#"You extract architecture-related intents from discussion messages. Return ONLY valid JSON - no markdown, no explanation.

Output format: a JSON object with an "extractions" array. Each extraction has:
- "intent": one of "decision" | "requirement" | "constraint" | "policy" | "risk" | "component"
- "confidence": 0.0 to 1.0
- "content": object matching the intent:

For "decision": { "title", "context", "decision", "alternatives": [], "consequences": [] }
For "requirement": { "title", "description", "priority": "Must"|"Should"|"Could"|"Wont" }
For "constraint": { "source", "target", "constraint_type": "CannotCall"|"MustUse"|"MustNotUse"|"Requires", "description" }
For "policy": { "name", "description", "rules": [] }
For "risk": { "description", "severity": "Low"|"Medium"|"High"|"Critical", "mitigation": optional }
For "component": { "name", "kind": "Service"|"Database"|"Queue"|"Module"|"ExternalApi", "technology": optional, "description": optional }

If nothing architecture-related, return: {"extractions": []}"#;

fn build_extraction_prompt(content: &str) -> String {
    format!(
        "Extract architecture intents from this message. Return ONLY the JSON object.\n\nMessage: \"{}\"",
        content.replace('"', "\\\"")
    )
}

fn parse_llm_response(
    text: &str,
    message: &ConversationMessage,
) -> Result<Vec<Extraction>, String> {
    let text = text
        .strip_prefix("```json")
        .unwrap_or(text)
        .strip_suffix("```")
        .unwrap_or(text)
        .trim();

    #[derive(Deserialize)]
    struct Response {
        extractions: Vec<LlmExtraction>,
    }

    let response: Response =
        serde_json::from_str(text).map_err(|e| format!("Parse error: {}", e))?;

    let mut result = Vec::new();
    for llm in response.extractions {
        if let Some(ext) = llm_to_extraction(llm, message) {
            result.push(ext);
        }
    }
    Ok(result)
}

fn llm_to_extraction(llm: LlmExtraction, message: &ConversationMessage) -> Option<Extraction> {
    let intent = match llm.intent.to_lowercase().as_str() {
        "decision" => Intent::Decision,
        "requirement" => Intent::Requirement,
        "constraint" => Intent::Constraint,
        "policy" => Intent::Policy,
        "risk" => Intent::Risk,
        "component" => Intent::ComponentMention,
        _ => return None,
    };

    let content = parse_content(intent, llm.content_json)?;

    Some(Extraction {
        id: generate_id(),
        intent,
        confidence: llm.confidence.clamp(0.0, 1.0),
        content,
        source_message_ids: vec![message.id.clone()],
        thread_root_message_id: None,
        status: ExtractionStatus::Draft,
        created_at: Utc::now(),
    })
}

fn parse_content(intent: Intent, v: serde_json::Value) -> Option<ExtractedContent> {
    let obj = v.as_object()?;
    Some(match intent {
        Intent::Decision => ExtractedContent::Decision {
            title: obj.get("title")?.as_str()?.to_string(),
            context: obj
                .get("context")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string(),
            decision: obj.get("decision")?.as_str()?.to_string(),
            alternatives: obj
                .get("alternatives")
                .and_then(|a| a.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            consequences: obj
                .get("consequences")
                .and_then(|c| c.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        },
        Intent::Requirement => ExtractedContent::Requirement {
            title: obj.get("title")?.as_str()?.to_string(),
            description: obj
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string(),
            priority: obj
                .get("priority")
                .and_then(|p| p.as_str())
                .map(|s| match s {
                    "Must" => RequirementPriority::Must,
                    "Should" => RequirementPriority::Should,
                    "Could" => RequirementPriority::Could,
                    "Wont" => RequirementPriority::Wont,
                    _ => RequirementPriority::Should,
                })
                .unwrap_or(RequirementPriority::Should),
        },
        Intent::Constraint => ExtractedContent::Constraint {
            source: obj.get("source")?.as_str()?.to_string(),
            target: obj.get("target")?.as_str()?.to_string(),
            constraint_type: obj
                .get("constraint_type")
                .and_then(|t| t.as_str())
                .map(|s| match s {
                    "MustUse" => ConstraintType::MustUse,
                    "MustNotUse" => ConstraintType::MustNotUse,
                    "Requires" => ConstraintType::Requires,
                    _ => ConstraintType::CannotCall,
                })
                .unwrap_or(ConstraintType::CannotCall),
            description: obj
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string(),
        },
        Intent::Policy => ExtractedContent::Policy {
            name: obj.get("name")?.as_str()?.to_string(),
            description: obj
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string(),
            rules: obj
                .get("rules")
                .and_then(|r| r.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        },
        Intent::Risk => ExtractedContent::Risk {
            description: obj.get("description")?.as_str()?.to_string(),
            severity: obj
                .get("severity")
                .and_then(|s| s.as_str())
                .map(|s| match s {
                    "Medium" => RiskSeverity::Medium,
                    "High" => RiskSeverity::High,
                    "Critical" => RiskSeverity::Critical,
                    _ => RiskSeverity::Low,
                })
                .unwrap_or(RiskSeverity::Low),
            mitigation: obj
                .get("mitigation")
                .and_then(|m| m.as_str())
                .map(String::from),
        },
        Intent::ComponentMention => ExtractedContent::Component {
            name: obj.get("name")?.as_str()?.to_string(),
            kind: obj
                .get("kind")
                .and_then(|k| k.as_str())
                .map(|s| match s {
                    "Database" => NodeKind::Database,
                    "Queue" => NodeKind::Queue,
                    "Module" => NodeKind::Module,
                    "ExternalApi" => NodeKind::ExternalApi,
                    _ => NodeKind::Service,
                })
                .unwrap_or(NodeKind::Service),
            technology: obj
                .get("technology")
                .and_then(|t| t.as_str())
                .map(String::from),
            description: obj
                .get("description")
                .and_then(|d| d.as_str())
                .map(String::from),
        },
        _ => return None,
    })
}
