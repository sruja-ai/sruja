use std::io::{Read as _, Write as _};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SrujaConfigFile {
    #[serde(default)]
    pub integrations: IntegrationsConfig,
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
