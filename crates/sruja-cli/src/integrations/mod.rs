use std::io::{Read as _, Write as _};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct SrujaConfigFile {
    #[serde(default)]
    pub integrations: IntegrationsConfig,
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default)]
    pub baseline: BaselineConfig,
    #[serde(default)]
    pub sandbox: SandboxConfig,
    #[serde(default)]
    pub context_engineering: ContextEngineeringConfig,
    #[serde(default)]
    pub verify: VerifyConfig,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct IntegrationsConfig {
    /// Default enrichment provider (e.g. "cmd", "openai", "plugin:foo")
    pub default_provider: Option<String>,
    /// Default command used when provider is "cmd"
    pub cmd: Option<String>,
    /// Default model used when provider is "openai"
    pub model: Option<String>,
    /// Default base URL used when provider is "openai"
    pub base_url: Option<String>,
    /// Timeout for enrichment in milliseconds
    pub timeout_ms: Option<u64>,
    /// Max bytes to read from enrichment output
    pub max_bytes: Option<usize>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct AgentConfig {
    /// Allowlisted Sruja subcommands that the agent may execute in `--mode apply`.
    /// Example: ["sync","drift","review","lint","intent-check"]
    pub allowed_sruja_subcommands: Option<Vec<String>>,
    /// Allowlisted top-level executables for verification commands (e.g. ["cargo","npm"]).
    pub allowed_verify_executables: Option<Vec<String>>,
    /// Default maximum steps for `agent run` in apply mode.
    pub max_steps: Option<usize>,
    /// Default maximum runtime per step (ms).
    pub max_runtime_ms_per_step: Option<u64>,
    /// Default number of MaTTS trajectories for `agent run --trajectories`.
    pub default_trajectories: Option<usize>,
    /// If true, `agent run --mode apply` may persist learnings to agentic memory.
    ///
    /// Default is false (no automatic memory writes) to keep the agent loop conservative.
    pub auto_record_learnings: Option<bool>,
    /// If true, automatically prune stale memories during briefings.
    pub auto_prune: Option<bool>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct BaselineConfig {
    /// Baseline interpretation mode for drift diff (scan vs DSL).
    ///
    /// - overview: baseline is intentionally high-level; treat "missing" as non-actionable.
    /// - exhaustive: baseline is intended as inventory; treat "missing" as actionable drift.
    /// - auto: infer based on relative size (fallback).
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct SandboxConfig {
    /// Policy when sandbox/worktree execution is unavailable for requested trajectories.
    ///
    /// - warn_and_degrade: proceed with a single primary trajectory and record a warning.
    /// - fail_fast: return an error.
    pub policy: Option<String>,
    /// Keep sandboxes on failure for inspection (default: false).
    pub keep_on_failure: Option<bool>,
    /// Remove sandboxes on success (default: true).
    pub cleanup_on_success: Option<bool>,
}

/// Configuration for context engineering features (BM25, compression, hybrid retrieval).
///
/// Loaded from `[context_engineering]` in `.sruja/config.toml`.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ContextEngineeringConfig {
    /// Token budget threshold for observation compression (default: 8000).
    pub compression_token_threshold: Option<usize>,
    /// Max length for compressed observation output (default: 120).
    /// Reserved for future use when `compress_single` accepts configurable limits.
    #[allow(dead_code)]
    pub compression_max_output_len: Option<usize>,
    /// Number of recent observations to keep uncompressed (default: 3).
    pub compression_keep_recent: Option<usize>,
    /// After `context_compressed` events, hint hosts to skip re-compress for N turns (default: 4).
    pub compression_suppress_recompress_turns: Option<u32>,
    /// Max BM25 results for focus external context (default: 10).
    pub bm25_max_results_focus: Option<usize>,
    /// Max BM25 results for MCP search (default: 5).
    pub bm25_max_results_mcp: Option<usize>,
}

/// Configuration for verification profiles.
///
/// Loaded from `[verify]` in `.sruja/config.toml`.
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[allow(dead_code)]
pub struct VerifyConfig {
    /// Default verification profile (coding, bugfix, review, arch).
    pub default_profile: Option<String>,
    /// Custom step definitions for the coding profile.
    pub coding: Option<VerifyProfileConfig>,
    /// Custom step definitions for the bugfix profile.
    pub bugfix: Option<VerifyProfileConfig>,
    /// Custom step definitions for the review profile.
    pub review: Option<VerifyProfileConfig>,
    /// Custom step definitions for the arch profile.
    pub arch: Option<VerifyProfileConfig>,
}

/// Per-profile verification configuration.
#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct VerifyProfileConfig {
    /// Verification steps to run (e.g. ["lint", "check", "drift-if-arch"]).
    pub steps: Option<Vec<String>>,
    /// Timeout per step in milliseconds.
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
pub struct EnrichmentLimits {
    pub timeout_ms: u64,
    pub max_bytes: usize,
}

impl EnrichmentLimits {
    pub fn with_defaults(timeout_ms: u64, max_bytes: usize) -> Self {
        Self {
            timeout_ms,
            max_bytes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedEnrichmentPlan {
    pub provider: String,
    pub cmd: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub limits: EnrichmentLimits,
}

pub fn load_repo_config(repo_root: &Path) -> Option<SrujaConfigFile> {
    let p = repo_root.join(".sruja").join("config.toml");
    let content = std::fs::read_to_string(&p).ok()?;
    toml::from_str::<SrujaConfigFile>(&content).ok()
}

pub fn resolve_enrichment_plan(
    repo_root: &Path,
    enrich_cmd_override: Option<&str>,
    model_override: Option<&str>,
    base_url_override: Option<&str>,
    timeout_ms_override: Option<u64>,
    max_bytes_override: Option<usize>,
) -> ResolvedEnrichmentPlan {
    // Defaults are conservative and enterprise-friendly.
    let defaults = EnrichmentLimits::with_defaults(15_000, 20_000);

    let cfg = load_repo_config(repo_root);
    let cfg_i = cfg.as_ref().map(|c| &c.integrations);

    let provider = std::env::var("SRUJA_ENRICH_PROVIDER")
        .ok()
        .or_else(|| cfg_i.and_then(|c| c.default_provider.clone()))
        .unwrap_or_else(|| "cmd".to_string());

    let cmd = enrich_cmd_override
        .map(|s| s.to_string())
        .or_else(|| std::env::var("SRUJA_ENRICH_CMD").ok())
        .or_else(|| cfg_i.and_then(|c| c.cmd.clone()));

    let model = model_override
        .map(|s| s.to_string())
        .or_else(|| std::env::var("SRUJA_ENRICH_MODEL").ok())
        // Back-compat
        .or_else(|| std::env::var("SRUJA_LLM_MODEL").ok())
        .or_else(|| cfg_i.and_then(|c| c.model.clone()));

    let base_url = base_url_override
        .map(|s| s.to_string())
        .or_else(|| std::env::var("SRUJA_ENRICH_BASE_URL").ok())
        // Back-compat
        .or_else(|| std::env::var("SRUJA_LLM_BASE_URL").ok())
        .or_else(|| cfg_i.and_then(|c| c.base_url.clone()));

    let timeout_ms = timeout_ms_override
        .or_else(|| {
            std::env::var("SRUJA_ENRICH_TIMEOUT_MS")
                .ok()
                .and_then(|v| v.parse().ok())
        })
        .or_else(|| cfg_i.and_then(|c| c.timeout_ms))
        .unwrap_or(defaults.timeout_ms);

    let max_bytes = max_bytes_override
        .or_else(|| {
            std::env::var("SRUJA_ENRICH_MAX_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
        })
        .or_else(|| cfg_i.and_then(|c| c.max_bytes))
        .unwrap_or(defaults.max_bytes);

    ResolvedEnrichmentPlan {
        provider,
        cmd,
        model,
        base_url,
        limits: EnrichmentLimits {
            timeout_ms,
            max_bytes,
        },
    }
}

pub fn run_cmd_enrichment(
    cmd: &str,
    stdin_payload: &[u8],
    limits: EnrichmentLimits,
) -> Result<String, String> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn command: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(stdin_payload)
            .map_err(|e| format!("Failed to write stdin: {e}"))?;
    }

    let timeout = Duration::from_millis(limits.timeout_ms.max(1));
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    return Err(format!("Command timed out after {}ms", timeout.as_millis()));
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => return Err(format!("Failed waiting for command: {e}")),
        }
    }

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    if let Some(mut outp) = child.stdout.take() {
        let _ = outp
            .by_ref()
            .take(limits.max_bytes as u64 + 1)
            .read_to_end(&mut stdout);
    }
    if let Some(mut errp) = child.stderr.take() {
        let _ = errp.by_ref().take(4096).read_to_end(&mut stderr);
    }

    let exit = child
        .wait()
        .map_err(|e| format!("Failed to collect exit status: {e}"))?;
    if !exit.success() {
        let msg = String::from_utf8_lossy(&stderr).trim().to_string();
        return Err(if msg.is_empty() {
            format!("Command exited non-zero: {exit}")
        } else {
            msg
        });
    }

    if stdout.len() > limits.max_bytes {
        return Err(format!(
            "Command output exceeded max bytes ({}).",
            limits.max_bytes
        ));
    }

    let out = String::from_utf8_lossy(&stdout).trim().to_string();
    if out.is_empty() {
        return Err("Command produced empty output".to_string());
    }
    Ok(out)
}

pub fn resolve_openai_auth() -> Option<String> {
    std::env::var("OPENAI_API_KEY")
        .ok()
        .or_else(|| std::env::var("SRUJA_ENRICH_API_KEY").ok())
        // Back-compat
        .or_else(|| std::env::var("SRUJA_LLM_API_KEY").ok())
}

pub fn run_openai_markdown(
    system_prompt: &str,
    user_prompt: &str,
    model: &str,
    base_url: &str,
    api_key: &str,
) -> Result<String, String> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let req = serde_json::json!({
        "model": model,
        "temperature": 0.2,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt }
        ]
    });

    let response = ureq::post(&url)
        .header("Authorization", &format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .send_json(req)
        .map_err(|e| format!("LLM request failed: {e}"))?;

    let body: serde_json::Value = response
        .into_body()
        .read_json::<serde_json::Value>()
        .map_err(|e| format!("Failed to parse LLM response as JSON: {e}"))?;

    let content = body
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c0| c0.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if content.is_empty() {
        return Err("LLM response missing choices[0].message.content".to_string());
    }
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_agent_config_defaults_and_fields() {
        let toml = r#"
[integrations]
default_provider = "cmd"
cmd = "echo ok"

[agent]
allowed_sruja_subcommands = ["sync", "drift"]
allowed_verify_executables = ["cargo"]
max_steps = 3
max_runtime_ms_per_step = 1000
auto_record_learnings = true
auto_prune = true
"#;
        let cfg: SrujaConfigFile = toml::from_str(toml).expect("parse toml");
        assert_eq!(cfg.integrations.default_provider.as_deref(), Some("cmd"));
        assert_eq!(
            cfg.agent.allowed_sruja_subcommands.as_deref(),
            Some(["sync".to_string(), "drift".to_string()].as_slice())
        );
        assert_eq!(
            cfg.agent.allowed_verify_executables.as_deref(),
            Some(["cargo".to_string()].as_slice())
        );
        assert_eq!(cfg.agent.max_steps, Some(3));
        assert_eq!(cfg.agent.max_runtime_ms_per_step, Some(1000));
        assert_eq!(cfg.agent.auto_record_learnings, Some(true));
        assert_eq!(cfg.agent.auto_prune, Some(true));
    }
}
