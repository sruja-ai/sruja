//! LLM-based evaluation of Sruja architecture files.
//!
//! Uses Rig with configurable providers (OpenAI, OpenRouter, Anthropic, Gemini, Ollama).
//! Set SRUJA_LLM_PROVIDER and the appropriate API key for your provider.

use std::path::Path;

use rig::client::{CompletionClient, Nothing};
use rig::completion::Prompt;
use rig::providers::{anthropic, gemini, ollama, openai, openrouter};

use super::CliError;

const EVAL_SYSTEM_PROMPT: &str = r#"You are a software architect evaluating a Sruja architecture DSL file.

Evaluate the architecture on a scale of 1-10 for each criterion:
1. **Completeness** (1-10): Are main components, modules, and subsystems captured?
2. **Accuracy** (1-10): Does it likely match the actual codebase architecture?
3. **Clarity** (1-10): Is it easy to understand the structure from this DSL?
4. **Usefulness** (1-10): Would this help a new developer understand the system?

Provide your response in this exact JSON format (no markdown, no explanation):
{
  "completeness": <1-10>,
  "accuracy": <1-10>,
  "clarity": <1-10>,
  "usefulness": <1-10>,
  "average": <average of 4 scores>,
  "strengths": ["<strength 1>", "<strength 2>"],
  "weaknesses": ["<weakness 1>", "<weakness 2>"],
  "verdict": "<Useful|Partially Useful|Not Useful>"
}"#;

/// Resolve provider and API key. Returns (provider, api_key) or error.
fn resolve_provider() -> Result<(String, Option<String>), CliError> {
    let provider = std::env::var("SRUJA_LLM_PROVIDER").unwrap_or_else(|_| "auto".to_string());
    let provider = provider.to_lowercase();

    let generic_key = std::env::var("SRUJA_LLM_API_KEY").ok();

    let (resolved, key) = if provider == "auto" {
        // Auto-detect: first provider with a key wins
        if std::env::var("OPENROUTER_API_KEY").is_ok() {
            (
                "openrouter".to_string(),
                generic_key.or_else(|| std::env::var("OPENROUTER_API_KEY").ok()),
            )
        } else if std::env::var("OPENAI_API_KEY").is_ok() {
            (
                "openai".to_string(),
                generic_key.or_else(|| std::env::var("OPENAI_API_KEY").ok()),
            )
        } else if std::env::var("ANTHROPIC_API_KEY").is_ok() {
            (
                "anthropic".to_string(),
                generic_key.or_else(|| std::env::var("ANTHROPIC_API_KEY").ok()),
            )
        } else if std::env::var("GEMINI_API_KEY").is_ok() || std::env::var("GOOGLE_API_KEY").is_ok()
        {
            let key = generic_key
                .or_else(|| std::env::var("GEMINI_API_KEY").ok())
                .or_else(|| std::env::var("GOOGLE_API_KEY").ok());
            ("gemini".to_string(), key)
        } else {
            return Err(CliError::Validation(
                "No LLM API key found. Set one of: OPENROUTER_API_KEY, OPENAI_API_KEY, \
                 ANTHROPIC_API_KEY, GEMINI_API_KEY. Or set SRUJA_LLM_PROVIDER=ollama for local models."
                    .to_string(),
            ));
        }
    } else if provider == "ollama" {
        ("ollama".to_string(), None)
    } else {
        let key = match provider.as_str() {
            "openrouter" => generic_key.or_else(|| std::env::var("OPENROUTER_API_KEY").ok()),
            "openai" => generic_key.or_else(|| std::env::var("OPENAI_API_KEY").ok()),
            "anthropic" => generic_key.or_else(|| std::env::var("ANTHROPIC_API_KEY").ok()),
            "gemini" => generic_key
                .or_else(|| std::env::var("GEMINI_API_KEY").ok())
                .or_else(|| std::env::var("GOOGLE_API_KEY").ok()),
            _ => {
                return Err(CliError::Validation(format!(
                    "Unknown SRUJA_LLM_PROVIDER '{}'. Use: openai, openrouter, anthropic, gemini, ollama",
                    provider
                )));
            }
        };
        (provider, key)
    };

    if resolved != "ollama" && key.as_ref().is_none_or(|k| k.is_empty()) {
        return Err(CliError::Validation(format!(
            "API key required for provider '{}'. Set the provider's API key env var or SRUJA_LLM_API_KEY.",
            resolved
        )));
    }

    Ok((resolved, key))
}

/// Default model per provider when SRUJA_EVAL_MODEL is not set.
fn default_model(provider: &str) -> &'static str {
    match provider {
        "openrouter" => "openai/gpt-4o-mini",
        "openai" => "gpt-4o-mini",
        "anthropic" => "claude-3-5-haiku-20241022",
        "gemini" => "gemini-1.5-flash",
        "ollama" => "llama3.2",
        _ => "gpt-4o-mini",
    }
}

/// Evaluate an architecture file using an LLM (any supported provider).
pub async fn eval(file: &str, _format: &str) -> Result<(), CliError> {
    let path = Path::new(file);
    let arch_path = if path.is_dir() {
        path.join("architecture.sruja")
    } else {
        path.to_path_buf()
    };

    if !arch_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("architecture.sruja not found at {}", arch_path.display()),
        )));
    }

    let content = std::fs::read_to_string(&arch_path).map_err(CliError::Io)?;

    let repo_name = arch_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let (provider, api_key) = resolve_provider()?;
    let model =
        std::env::var("SRUJA_EVAL_MODEL").unwrap_or_else(|_| default_model(&provider).to_string());

    let truncated = if content.len() > 8000 {
        format!("{}... (truncated)", &content[..8000])
    } else {
        content.clone()
    };

    let prompt = format!(
        "Evaluate this Sruja architecture DSL for the {} codebase. Return ONLY the JSON object.\n\nArchitecture DSL:\n{}\n",
        repo_name,
        truncated
    );

    let text: String = match provider.as_str() {
        "openrouter" => {
            let client: openrouter::Client =
                openrouter::Client::new(api_key.expect("key checked in resolve_provider"))
                    .map_err(|e| {
                        CliError::Validation(format!("OpenRouter client failed: {}", e))
                    })?;
            let agent = client.agent(&model).preamble(EVAL_SYSTEM_PROMPT).build();
            agent
                .prompt(prompt)
                .await
                .map_err(|e| CliError::Validation(format!("LLM request failed: {}", e)))?
        }
        "openai" => {
            let client: openai::Client =
                openai::Client::new(api_key.expect("key checked in resolve_provider"))
                    .map_err(|e| CliError::Validation(format!("OpenAI client failed: {}", e)))?;
            let agent = client.agent(&model).preamble(EVAL_SYSTEM_PROMPT).build();
            agent
                .prompt(prompt)
                .await
                .map_err(|e| CliError::Validation(format!("LLM request failed: {}", e)))?
        }
        "anthropic" => {
            let client: anthropic::Client =
                anthropic::Client::new(api_key.expect("key checked in resolve_provider"))
                    .map_err(|e| CliError::Validation(format!("Anthropic client failed: {}", e)))?;
            let agent = client.agent(&model).preamble(EVAL_SYSTEM_PROMPT).build();
            agent
                .prompt(prompt)
                .await
                .map_err(|e| CliError::Validation(format!("LLM request failed: {}", e)))?
        }
        "gemini" => {
            let client: gemini::Client =
                gemini::Client::new(api_key.expect("key checked in resolve_provider"))
                    .map_err(|e| CliError::Validation(format!("Gemini client failed: {}", e)))?;
            let agent = client.agent(&model).preamble(EVAL_SYSTEM_PROMPT).build();
            agent
                .prompt(prompt)
                .await
                .map_err(|e| CliError::Validation(format!("LLM request failed: {}", e)))?
        }
        "ollama" => {
            let client: ollama::Client = ollama::Client::new(Nothing).map_err(|e| {
                CliError::Validation(format!("Ollama client failed (is Ollama running?): {}", e))
            })?;
            let agent = client.agent(&model).preamble(EVAL_SYSTEM_PROMPT).build();
            agent
                .prompt(prompt)
                .await
                .map_err(|e| CliError::Validation(format!("LLM request failed: {}", e)))?
        }
        _ => {
            return Err(CliError::Validation(format!(
                "Unsupported provider: {}",
                provider
            )));
        }
    };

    let text = text
        .trim()
        .strip_prefix("```json")
        .unwrap_or(text.trim())
        .strip_suffix("```")
        .unwrap_or(text.trim())
        .trim();

    let json_start = text.find('{').ok_or_else(|| {
        CliError::Validation("Could not parse LLM response: no JSON found".to_string())
    })?;
    let json_end = text.rfind('}').ok_or_else(|| {
        CliError::Validation("Could not parse LLM response: no JSON found".to_string())
    })? + 1;
    let json_str = &text[json_start..json_end];

    let parsed: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| CliError::Validation(format!("Could not parse LLM response: {}", e)))?;

    // Pretty-print the evaluation
    eprintln!("{}", "═".repeat(60));
    eprintln!(
        "🤖 LLM Architecture Evaluation: {} (provider: {})",
        repo_name, provider
    );
    eprintln!("{}", "═".repeat(60));
    eprintln!();

    if let Some(obj) = parsed.as_object() {
        if let (Some(c), Some(a), Some(cl), Some(u), Some(avg)) = (
            obj.get("completeness"),
            obj.get("accuracy"),
            obj.get("clarity"),
            obj.get("usefulness"),
            obj.get("average"),
        ) {
            eprintln!("  Completeness:  {}/10", c);
            eprintln!("  Accuracy:     {}/10", a);
            eprintln!("  Clarity:     {}/10", cl);
            eprintln!("  Usefulness:  {}/10", u);
            eprintln!("  ─────────────────");
            eprintln!("  Average:     {}/10", avg);
            eprintln!();
        }
        if let Some(v) = obj.get("verdict") {
            eprintln!("  Verdict: {}", v);
            eprintln!();
        }
        if let Some(strengths) = obj.get("strengths").and_then(|s| s.as_array()) {
            if !strengths.is_empty() {
                eprintln!("  Strengths:");
                for s in strengths {
                    if let Some(st) = s.as_str() {
                        eprintln!("    • {}", st);
                    }
                }
                eprintln!();
            }
        }
        if let Some(weaknesses) = obj.get("weaknesses").and_then(|w| w.as_array()) {
            if !weaknesses.is_empty() {
                eprintln!("  Weaknesses:");
                for w in weaknesses {
                    if let Some(wk) = w.as_str() {
                        eprintln!("    • {}", wk);
                    }
                }
            }
        }
    } else {
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&parsed).unwrap_or_default()
        );
    }

    eprintln!();
    eprintln!("{}", "═".repeat(60));

    Ok(())
}
