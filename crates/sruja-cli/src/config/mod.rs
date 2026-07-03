//! Unified configuration resolution for agent commands.
//!
//! Follows industry best practices:
//! - **12-factor app**: Secrets in environment variables, never in config files
//! - **Layered resolution**: CLI flags > env vars > config.toml > defaults
//! - **Provider-aware**: Uses provider's `key_env` to find the correct API key
//! - **Multi-provider**: Different providers for different tasks (cheap/mid/premium/review)
//! - **Fail-fast**: Clear error messages when configuration is missing

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::commands::CliError;
use crate::integrations::providers::{self, ProviderPreset};

/// Configuration loaded from `.sruja/config.toml`.
///
/// This stores **non-secret** configuration only.
/// API keys are resolved from environment variables at runtime.
#[derive(Debug, Default, Deserialize)]
pub struct SrujaConfig {
    /// The `[integrations]` section containing LLM provider settings.
    #[serde(default)]
    pub integrations: IntegrationsConfig,
    /// The `[agent]` section containing agent-specific settings.
    #[serde(default)]
    pub agent: AgentConfig,
}

/// The `[integrations]` section of config.toml.
#[derive(Debug, Default, Deserialize)]
pub struct IntegrationsConfig {
    /// Default provider id (e.g., "openrouter", "zai", "ximimo").
    pub default_provider: Option<String>,
    /// Provider preset id (legacy single-provider format).
    pub provider_id: Option<String>,
    /// Model name (legacy single-provider format).
    pub model: Option<String>,
    /// Base URL for the LLM API (legacy single-provider format).
    pub base_url: Option<String>,
    /// Named provider configurations.
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

/// Configuration for a named provider.
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderConfig {
    /// Base URL for the LLM API.
    pub base_url: Option<String>,
    /// Environment variable name for the API key.
    pub key_env: Option<String>,
}

/// The `[agent]` section of config.toml.
#[derive(Debug, Default, Deserialize)]
pub struct AgentConfig {
    /// Model tier configurations.
    #[serde(default)]
    pub models: ModelTiers,
}

/// Model tier configurations for different tasks.
#[derive(Debug, Default, Deserialize)]
pub struct ModelTiers {
    /// Cheap/fast model for simple tasks.
    pub cheap: Option<ModelTier>,
    /// Mid-tier model for general tasks.
    pub mid: Option<ModelTier>,
    /// Premium model for complex tasks.
    pub premium: Option<ModelTier>,
    /// Review model for code review.
    pub review: Option<ModelTier>,
}

/// Configuration for a single model tier.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelTier {
    /// Provider id (e.g., "zai", "openrouter").
    pub provider: String,
    /// Model name (e.g., "GLM-4.7", "anthropic/claude-sonnet-4").
    pub model: String,
}

/// Resolved LLM configuration ready for use.
///
/// All fields are guaranteed to be non-empty after successful resolution.
#[derive(Debug)]
pub struct ResolvedLlmConfig {
    /// The API key resolved from environment variables.
    pub api_key: String,
    /// The base URL for the LLM API.
    pub base_url: String,
    /// The model name to use.
    pub model: String,
    /// Human-readable provider name (for logging).
    pub provider_name: String,
    /// Provider ID (e.g. "zai", "ximimo") — used for prefix routing.
    pub provider_id: String,
}

/// Resolved multi-provider configuration for all model tiers.
#[derive(Debug)]
pub struct ResolvedMultiProviderConfig {
    /// Configuration for the cheap tier.
    pub cheap: ResolvedLlmConfig,
    /// Configuration for the mid tier.
    pub mid: ResolvedLlmConfig,
    /// Configuration for the premium tier.
    pub premium: ResolvedLlmConfig,
    /// Configuration for the review tier.
    pub review: ResolvedLlmConfig,
}

/// Resolve LLM configuration using the industry-standard chain:
///
/// 1. **CLI flags** (highest priority)
/// 2. **Environment variables** (provider-specific + fallbacks)
/// 3. **Config file** (`.sruja/config.toml`)
/// 4. **Built-in defaults** (lowest priority)
///
/// # Arguments
/// * `repo_path` - Path to the repository root
/// * `cli_model` - Model override from CLI flags
/// * `cli_base_url` - Base URL override from CLI flags
/// * `cli_api_key` - API key override from CLI flags (for testing only)
///
/// # Environment Variables
/// The resolver checks environment variables in this order:
/// 1. Provider-specific key (e.g., `OPENROUTER_API_KEY`, `ZAI_API_KEY`)
/// 2. Generic fallbacks: `OPENAI_API_KEY`, `SRUJA_ENRICH_API_KEY`
///
/// # Errors
/// Returns `CliError::validation` if no API key is found after checking all sources.
pub fn resolve_llm_config(
    repo_path: &Path,
    cli_model: Option<&str>,
    cli_base_url: Option<&str>,
    cli_api_key: Option<&str>,
) -> Result<ResolvedLlmConfig, CliError> {
    // Load config.toml (non-fatal if missing)
    let config = load_sruja_config(repo_path);

    // Resolve provider preset
    let preset = resolve_provider_preset(&config)?;

    // Resolve in order: CLI > env vars > config.toml > defaults
    let model = cli_model
        .or(config.integrations.model.as_deref())
        .unwrap_or(preset.default_model)
        .to_string();

    let base_url = cli_base_url
        .or(config.integrations.base_url.as_deref())
        .unwrap_or(preset.base_url)
        .to_string();

    // API key resolution (secrets always from env vars)
    let api_key = resolve_api_key(cli_api_key, preset)?;

    Ok(ResolvedLlmConfig {
        api_key,
        base_url,
        model,
        provider_name: preset.name.to_string(),
        provider_id: preset.id.to_string(),
    })
}

/// Resolve multi-provider configuration for all model tiers.
///
/// This allows using different providers for different tasks:
/// - **cheap**: Fast, inexpensive model for simple tasks
/// - **mid**: General-purpose model for most tasks
/// - **premium**: High-quality model for complex tasks
/// - **review**: Specialized model for code review
///
/// # Arguments
/// * `repo_path` - Path to the repository root
///
/// # Configuration
/// Configure in `.sruja/config.toml`:
/// ```toml
/// [integrations]
/// default_provider = "zai"
///
/// [integrations.providers.zai]
/// base_url = "https://api.z.ai/api/coding/paas/v4"
/// key_env = "ZAI_API_KEY"
///
/// [integrations.providers.openrouter]
/// base_url = "https://openrouter.ai/api/v1"
/// key_env = "OPENROUTER_API_KEY"
///
/// [agent.models]
/// cheap = { provider = "zai", model = "GLM-4-Flash" }
/// mid = { provider = "zai", model = "GLM-4.7" }
/// premium = { provider = "openrouter", model = "anthropic/claude-sonnet-4" }
/// review = { provider = "openrouter", model = "google/gemini-2.5-flash" }
/// ```
///
/// # Errors
/// Returns `CliError::validation` if any provider is misconfigured or API key is missing.
pub fn resolve_multi_provider_config(
    repo_path: &Path,
) -> Result<ResolvedMultiProviderConfig, CliError> {
    let config = load_sruja_config(repo_path);

    // Resolve each tier
    let cheap = resolve_tier_config(repo_path, &config, &config.agent.models.cheap, "cheap")?;
    let mid = resolve_tier_config(repo_path, &config, &config.agent.models.mid, "mid")?;
    let premium = resolve_tier_config(repo_path, &config, &config.agent.models.premium, "premium")?;
    let review = resolve_tier_config(repo_path, &config, &config.agent.models.review, "review")?;

    Ok(ResolvedMultiProviderConfig {
        cheap,
        mid,
        premium,
        review,
    })
}

/// Resolve configuration for a single model tier.
fn resolve_tier_config(
    _repo_path: &Path,
    config: &SrujaConfig,
    tier: &Option<ModelTier>,
    tier_name: &str,
) -> Result<ResolvedLlmConfig, CliError> {
    match tier {
        Some(tier) => {
            // Get provider config from named providers
            let provider_config = config.integrations.providers.get(&tier.provider);

            // Get base URL from provider config or preset
            let base_url = provider_config
                .and_then(|p| p.base_url.clone())
                .or_else(|| providers::find_preset(&tier.provider).map(|p| p.base_url.to_string()))
                .ok_or_else(|| {
                    CliError::validation(format!(
                        "No base_url configured for provider '{}'. \
                         Add [integrations.providers.{}] with base_url, \
                         or use a known provider (zai, openrouter, ximimo, groq, ollama).",
                        tier.provider, tier.provider
                    ))
                })?;

            // Get API key from env var
            let key_env = provider_config
                .and_then(|p| p.key_env.clone())
                .unwrap_or_else(|| format!("{}_API_KEY", tier.provider.to_uppercase()));

            let api_key = std::env::var(&key_env).map_err(|_| {
                CliError::validation(format!(
                    "No API key found for provider '{}' (tier '{}').\n\
                     \n\
                     Run the setup wizard to configure your provider:\n\
                       sruja agent setup\n\
                     \n\
                     Or set the environment variable directly:\n\
                       export {}=\"your-key-here\"",
                    tier.provider, tier_name, key_env
                ))
            })?;

            // Get provider name for logging
            let provider_name = providers::find_preset(&tier.provider)
                .map(|p| p.name.to_string())
                .unwrap_or_else(|| tier.provider.clone());

            Ok(ResolvedLlmConfig {
                api_key,
                base_url,
                model: tier.model.clone(),
                provider_name,
                provider_id: tier.provider.clone(),
            })
        }
        None => {
            // Fall back to default provider
            let preset = resolve_provider_preset(config)?;
            let api_key = resolve_api_key(None, preset)?;

            Ok(ResolvedLlmConfig {
                api_key,
                base_url: preset.base_url.to_string(),
                model: preset.default_model.to_string(),
                provider_name: preset.name.to_string(),
                provider_id: preset.id.to_string(),
            })
        }
    }
}

/// Load `.sruja/config.toml` if it exists.
fn load_sruja_config(repo_path: &Path) -> SrujaConfig {
    let config_path = repo_path.join(".sruja/config.toml");
    match std::fs::read_to_string(&config_path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!(
                    "⚠  Failed to parse {}: {e}. Using default config.",
                    config_path.display()
                );
                SrujaConfig::default()
            }
        },
        Err(_) => SrujaConfig::default(),
    }
}

/// Resolve the provider preset from config or default.
fn resolve_provider_preset(config: &SrujaConfig) -> Result<&'static ProviderPreset, CliError> {
    let provider_id = config
        .integrations
        .default_provider
        .as_deref()
        .or(config.integrations.provider_id.as_deref())
        .unwrap_or("openai");

    providers::find_preset(provider_id).ok_or_else(|| {
        let known: Vec<&str> = providers::PRESETS.iter().map(|p| p.id).collect();
        CliError::validation(format!(
            "Unknown provider '{provider_id}' in .sruja/config.toml. \
             Known providers: {}",
            known.join(", ")
        ))
    })
}

/// Resolve API key using the provider-aware chain.
///
/// Priority:
/// 1. CLI override (for testing)
/// 2. Provider-specific env var (e.g., `OPENROUTER_API_KEY`)
/// 3. Generic fallbacks: `OPENAI_API_KEY`, `SRUJA_ENRICH_API_KEY`
fn resolve_api_key(
    cli_override: Option<&str>,
    preset: &ProviderPreset,
) -> Result<String, CliError> {
    // 1. CLI override (highest priority)
    if let Some(key) = cli_override {
        if !key.is_empty() {
            return Ok(key.to_string());
        }
    }

    // 2. Provider-specific env var
    if !preset.key_env.is_empty() {
        if let Ok(key) = std::env::var(preset.key_env) {
            if !key.is_empty() {
                return Ok(key);
            }
        }
    }

    // 3. Generic fallbacks
    for fallback in &["OPENAI_API_KEY", "SRUJA_ENRICH_API_KEY"] {
        if let Ok(key) = std::env::var(fallback) {
            if !key.is_empty() {
                return Ok(key);
            }
        }
    }

    // 4. No key found
    Err(CliError::validation(format!(
        "No API key found for provider '{}'.\n\
         \n\
         Run the setup wizard to configure your provider:\n\
           sruja agent setup\n\
         \n\
         Or set the environment variable directly:\n\
           export {}=\"your-key-here\"",
        preset.name, preset.key_env
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    /// Serialize tests that mutate environment variables to prevent races.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_resolve_from_env_vars() {
        let _lock = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();

        // Set env var
        std::env::set_var("OPENAI_API_KEY", "test-key-123");

        let result = resolve_llm_config(repo, None, None, None).unwrap();
        assert_eq!(result.api_key, "test-key-123");
        assert_eq!(result.base_url, "https://api.openai.com/v1");
        assert_eq!(result.model, "gpt-4o-mini");

        std::env::remove_var("OPENAI_API_KEY");
    }

    #[test]
    fn test_resolve_from_config_toml() {
        let _lock = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        let sruja_dir = repo.join(".sruja");
        fs::create_dir_all(&sruja_dir).unwrap();

        fs::write(
            sruja_dir.join("config.toml"),
            r#"
[integrations]
provider_id = "zai"
model = "glm-4-flash"
base_url = "https://open.bigmodel.cn/api/paas/v4"
"#,
        )
        .unwrap();

        std::env::set_var("ZAI_API_KEY", "zai-key-456");

        let result = resolve_llm_config(repo, None, None, None).unwrap();
        assert_eq!(result.api_key, "zai-key-456");
        assert_eq!(result.base_url, "https://open.bigmodel.cn/api/paas/v4");
        assert_eq!(result.model, "glm-4-flash");
        assert_eq!(result.provider_name, "Zhipu AI (z.ai)");
        assert_eq!(result.provider_id, "zai");

        std::env::remove_var("ZAI_API_KEY");
    }

    #[test]
    fn test_multi_provider_config() {
        let _lock = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        let sruja_dir = repo.join(".sruja");
        fs::create_dir_all(&sruja_dir).unwrap();

        fs::write(
            sruja_dir.join("config.toml"),
            r#"
[integrations]
default_provider = "zai"

[integrations.providers.zai]
base_url = "https://api.z.ai/api/coding/paas/v4"
key_env = "ZAI_API_KEY"

[integrations.providers.openrouter]
base_url = "https://openrouter.ai/api/v1"
key_env = "OPENROUTER_API_KEY"

[agent.models]
cheap = { provider = "zai", model = "GLM-4-Flash" }
mid = { provider = "zai", model = "GLM-4.7" }
premium = { provider = "openrouter", model = "anthropic/claude-sonnet-4" }
review = { provider = "openrouter", model = "google/gemini-2.5-flash" }
"#,
        )
        .unwrap();

        std::env::set_var("ZAI_API_KEY", "zai-key");
        std::env::set_var("OPENROUTER_API_KEY", "or-key");

        let result = resolve_multi_provider_config(repo).unwrap();

        assert_eq!(result.cheap.model, "GLM-4-Flash");
        assert_eq!(result.cheap.provider_name, "Zhipu AI (z.ai)");
        assert_eq!(result.cheap.provider_id, "zai");
        assert_eq!(result.mid.model, "GLM-4.7");
        assert_eq!(result.premium.model, "anthropic/claude-sonnet-4");
        assert_eq!(result.premium.provider_id, "openrouter");
        assert_eq!(result.review.model, "google/gemini-2.5-flash");
        assert_eq!(result.review.provider_id, "openrouter");

        std::env::remove_var("ZAI_API_KEY");
        std::env::remove_var("OPENROUTER_API_KEY");
    }

    #[test]
    fn test_cli_flags_override_config() {
        let _lock = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        let sruja_dir = repo.join(".sruja");
        fs::create_dir_all(&sruja_dir).unwrap();

        fs::write(
            sruja_dir.join("config.toml"),
            r#"
[integrations]
provider_id = "zai"
model = "glm-4-flash"
"#,
        )
        .unwrap();

        std::env::set_var("ZAI_API_KEY", "zai-key");

        let result = resolve_llm_config(
            repo,
            Some("custom-model"),
            Some("https://custom.api/v1"),
            None,
        )
        .unwrap();

        assert_eq!(result.model, "custom-model");
        assert_eq!(result.base_url, "https://custom.api/v1");
        assert_eq!(result.api_key, "zai-key");

        std::env::remove_var("ZAI_API_KEY");
    }

    #[test]
    fn test_provider_specific_env_var() {
        let _lock = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        let sruja_dir = repo.join(".sruja");
        fs::create_dir_all(&sruja_dir).unwrap();

        fs::write(
            sruja_dir.join("config.toml"),
            r#"
[integrations]
provider_id = "openrouter"
"#,
        )
        .unwrap();

        std::env::set_var("OPENROUTER_API_KEY", "or-key-789");

        let result = resolve_llm_config(repo, None, None, None).unwrap();
        assert_eq!(result.api_key, "or-key-789");
        assert_eq!(result.provider_name, "OpenRouter");
        assert_eq!(result.provider_id, "openrouter");

        std::env::remove_var("OPENROUTER_API_KEY");
    }

    #[test]
    fn test_missing_api_key_fails_fast() {
        let _lock = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();

        // Ensure no env vars are set
        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("SRUJA_ENRICH_API_KEY");

        let result = resolve_llm_config(repo, None, None, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No API key found"));
    }
}
