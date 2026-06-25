//! Known LLM provider presets for zero-config onboarding.

use sruja_agent::DEFAULT_MODEL;

/// A preset for a known LLM provider.
pub struct ProviderPreset {
    pub id: &'static str,
    pub name: &'static str,
    pub base_url: &'static str,
    pub default_model: &'static str,
    pub key_env: &'static str,
    pub key_hint: &'static str,
}

/// All known provider presets.
pub const PRESETS: &[ProviderPreset] = &[
    ProviderPreset {
        id: "openrouter",
        name: "OpenRouter",
        base_url: "https://openrouter.ai/api/v1",
        default_model: "anthropic/claude-sonnet-4",
        key_env: "OPENROUTER_API_KEY",
        key_hint: "sk-or-... (from openrouter.ai/keys)",
    },
    ProviderPreset {
        id: "openai",
        name: "OpenAI",
        base_url: "https://api.openai.com/v1",
        default_model: DEFAULT_MODEL,
        key_env: "OPENAI_API_KEY",
        key_hint: "sk-... (from platform.openai.com/api-keys)",
    },
    ProviderPreset {
        id: "zai",
        name: "Zhipu AI (z.ai)",
        base_url: "https://api.z.ai/api/coding/paas/v4",
        default_model: "GLM-4.7",
        key_env: "ZAI_API_KEY",
        key_hint: "From open.bigmodel.cn",
    },
    ProviderPreset {
        id: "ximimo",
        name: "Ximimo",
        base_url: "https://token-plan-sgp.xiaomimimo.com/v1",
        default_model: "mimo-v2.5-pro",
        key_env: "XIMIMO_API_KEY",
        key_hint: "From ximimo.com",
    },
    ProviderPreset {
        id: "groq",
        name: "Groq",
        base_url: "https://api.groq.com/openai/v1",
        default_model: "llama-3.3-70b-versatile",
        key_env: "GROQ_API_KEY",
        key_hint: "gsk_... (from console.groq.com/keys)",
    },
    ProviderPreset {
        id: "ollama",
        name: "Ollama (local)",
        base_url: "http://localhost:11434/v1",
        default_model: "llama3.2",
        key_env: "",
        key_hint: "No key needed — just run `ollama serve`",
    },
];

/// Look up a preset by id.
pub fn find_preset(id: &str) -> Option<&'static ProviderPreset> {
    PRESETS.iter().find(|p| p.id == id)
}
