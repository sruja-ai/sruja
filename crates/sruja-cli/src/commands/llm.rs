//! Shared LLM provider resolution and completion.
//!
//! Used by timeline (suggest-refs, explain) and AI commands (explain, ask).
//! Env: SRUJA_LLM_PROVIDER, SRUJA_LLM_API_KEY, SRUJA_TIMELINE_MODEL, provider-specific keys.

use rig::client::{CompletionClient, Nothing};
use rig::completion::Prompt;
use rig::providers::{anthropic, gemini, ollama, openai, openrouter};

use super::CliError;

/// Resolve provider name and optional API key from env.
pub fn resolve_provider() -> Result<(String, Option<String>), CliError> {
    let provider = std::env::var("SRUJA_LLM_PROVIDER").unwrap_or_else(|_| "auto".to_string());
    let provider = provider.to_lowercase();

    let generic_key = std::env::var("SRUJA_LLM_API_KEY").ok();

    let (resolved, key) = if provider == "auto" {
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
                 ANTHROPIC_API_KEY, GEMINI_API_KEY, or SRUJA_LLM_PROVIDER=ollama."
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
            "API key required for provider '{}'.",
            resolved
        )));
    }

    Ok((resolved, key))
}

/// Default model per provider (used when SRUJA_TIMELINE_MODEL / SRUJA_LLM_MODEL not set).
pub fn default_model(provider: &str) -> &'static str {
    match provider {
        "openrouter" => "openai/gpt-4o-mini",
        "openai" => "gpt-4o-mini",
        "anthropic" => "claude-3-5-haiku-20241022",
        "gemini" => "gemini-1.5-flash",
        "ollama" => "llama3.2",
        _ => "gpt-4o-mini",
    }
}

/// Model name for AI/timeline (SRUJA_LLM_MODEL or SRUJA_TIMELINE_MODEL or default).
pub fn resolve_model(provider: &str) -> String {
    std::env::var("SRUJA_LLM_MODEL")
        .ok()
        .or_else(|| std::env::var("SRUJA_TIMELINE_MODEL").ok())
        .unwrap_or_else(|| default_model(provider).to_string())
}

/// Call LLM with system and user prompt; returns raw response text.
pub async fn call_llm(system: &str, user_prompt: &str) -> Result<String, CliError> {
    let (provider, api_key) = resolve_provider()?;
    let model = resolve_model(&provider);

    let text: String = match provider.as_str() {
        "openrouter" => {
            let client: openrouter::Client =
                openrouter::Client::new(api_key.expect("key checked")).map_err(|e| {
                    CliError::Validation(format!("OpenRouter client failed: {}", e))
                })?;
            let agent = client.agent(&model).preamble(system).build();
            agent
                .prompt(user_prompt)
                .await
                .map_err(|e| CliError::Validation(format!("LLM request failed: {}", e)))?
        }
        "openai" => {
            let client: openai::Client =
                openai::Client::new(api_key.expect("key checked")).map_err(|e| {
                    CliError::Validation(format!("OpenAI client failed: {}", e))
                })?;
            let agent = client.agent(&model).preamble(system).build();
            agent
                .prompt(user_prompt)
                .await
                .map_err(|e| CliError::Validation(format!("LLM request failed: {}", e)))?
        }
        "anthropic" => {
            let client: anthropic::Client =
                anthropic::Client::new(api_key.expect("key checked")).map_err(|e| {
                    CliError::Validation(format!("Anthropic client failed: {}", e))
                })?;
            let agent = client.agent(&model).preamble(system).build();
            agent
                .prompt(user_prompt)
                .await
                .map_err(|e| CliError::Validation(format!("LLM request failed: {}", e)))?
        }
        "gemini" => {
            let client: gemini::Client =
                gemini::Client::new(api_key.expect("key checked")).map_err(|e| {
                    CliError::Validation(format!("Gemini client failed: {}", e))
                })?;
            let agent = client.agent(&model).preamble(system).build();
            agent
                .prompt(user_prompt)
                .await
                .map_err(|e| CliError::Validation(format!("LLM request failed: {}", e)))?
        }
        "ollama" => {
            let client: ollama::Client =
                ollama::Client::new(Nothing).map_err(|e| {
                    CliError::Validation(format!("Ollama client failed: {}", e))
                })?;
            let agent = client.agent(&model).preamble(system).build();
            agent
                .prompt(user_prompt)
                .await
                .map_err(|e| CliError::Validation(format!("LLM request failed: {}", e)))?
        }
        _ => return Err(CliError::Validation(format!("Unsupported provider: {}", provider))),
    };

    Ok(text)
}
