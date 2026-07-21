//! Layered configuration loading with deep merge support.
//!
//! Supports three configuration layers (lowest to highest priority):
//!
//! 1. **System**: `/etc/sruja/config.toml` — enterprise policies
//! 2. **User**: `$XDG_CONFIG_HOME/sruja/config.toml` or `~/.config/sruja/config.toml` — personal preferences
//! 3. **Project**: `{repo}/.sruja/config.toml` — per-project settings
//!
//! Layers are deep-merged: tables combine recursively, non-table values from
//! higher-priority layers override lower ones. This lets a project config
//! override a single field (e.g., `model`) while inheriting everything else.

use std::path::{Path, PathBuf};

/// A single configuration layer with metadata.
#[derive(Debug, Clone)]
pub struct ConfigLayer {
    /// Human-readable layer name (e.g. "system", "user", "project", "env")
    pub name: &'static str,
    /// Resolved filesystem path, or `None` if the file was not found.
    pub path: Option<PathBuf>,
    /// Parsed TOML value (empty table if not found).
    pub value: toml::Value,
}

impl ConfigLayer {
    fn new(name: &'static str, path: Option<PathBuf>, value: toml::Value) -> Self {
        Self { name, path, value }
    }

    fn empty(name: &'static str) -> Self {
        Self {
            name,
            path: None,
            value: toml::Value::Table(toml::map::Map::new()),
        }
    }
}

/// Multi-source layered configuration assembled from system, user, and project
/// config files.
///
/// ## Priority
///
/// When the same key exists in multiple layers, the highest-priority value wins:
///
/// | Layer     | Priority | Example Path                         |
/// |-----------|----------|--------------------------------------|
/// | system    | lowest   | `/etc/sruja/config.toml`             |
/// | user      |          | `~/.config/sruja/config.toml`        |
/// | project   | highest  | `/repo/.sruja/config.toml`           |
///
/// Tables are merged recursively so that a user config can set `[agent.models]`
/// without re-specifying `[integrations]`.
#[derive(Debug)]
pub struct LayeredConfig {
    /// All loaded layers in merge order (index 0 = lowest priority).
    pub layers: Vec<ConfigLayer>,
    /// Deep-merged result across all layers.
    pub merged: toml::Value,
}

impl LayeredConfig {
    /// Load configuration from all available layers.
    ///
    /// Missing files are silently skipped. Parse warnings are emitted to stderr
    /// but do not fail the overall load — the offending layer is treated as empty.
    pub fn load(repo_path: &Path) -> Self {
        let mut merged = toml::Value::Table(toml::map::Map::new());
        let mut layers: Vec<ConfigLayer> = Vec::new();

        // 1. System layer (lowest priority)
        let system_path = system_config_path();
        let system_layer = load_layer("system", &system_path);
        deep_merge(&mut merged, system_layer.value.clone());
        layers.push(system_layer);

        // 2. User layer
        let user_path = user_config_path();
        let user_layer = load_layer("user", &user_path);
        deep_merge(&mut merged, user_layer.value.clone());
        layers.push(user_layer);

        // 3. Project layer (highest priority among files)
        let project_path = repo_path.join(".sruja/config.toml");
        let project_layer = load_layer("project", &project_path);
        deep_merge(&mut merged, project_layer.value.clone());
        layers.push(project_layer);

        // 4. Environment variable override (highest priority)
        // SRUJA_CONFIG sections can be specified as inline TOML via env var.
        // Format: SRUJA_CONFIG='[integrations]\nmodel = "gpt-4"'
        let env_overrides = load_env_overrides();
        if env_overrides
            .value
            .as_table()
            .is_some_and(|t| !t.is_empty())
        {
            deep_merge(&mut merged, env_overrides.value.clone());
            layers.push(env_overrides);
        }

        Self { layers, merged }
    }

    /// Deserialize the merged configuration into a concrete type.
    ///
    /// # Errors
    /// Returns a description of what went wrong if deserialization fails.
    pub fn deserialize<T>(&self) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        // Round-trip through TOML string for reliable deserialization.
        // toml::Value deserialization via serde has edge cases with table
        // ordering and spanned values; string round-trip is always correct.
        let encoded = toml::to_string_pretty(&self.merged)
            .map_err(|e| format!("cannot serialize merged config: {e}"))?;
        toml::from_str(&encoded).map_err(|e| format!("{e}"))
    }

    /// Return the project-layer config path for a repository.
    pub fn project_config_path(repo_path: &Path) -> PathBuf {
        repo_path.join(".sruja/config.toml")
    }

    /// Collect all resolved config file paths (existing files only).
    pub fn existing_config_paths(repo_path: &Path) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for p in &[
            system_config_path(),
            user_config_path(),
            repo_path.join(".sruja/config.toml"),
        ] {
            if p.exists() {
                paths.push(p.clone());
            }
        }
        paths
    }
}

// ---------------------------------------------------------------------------
// Path resolution
// ---------------------------------------------------------------------------

/// System-level config path.
///
/// - Linux: `/etc/sruja/config.toml`
/// - macOS: `/Library/Preferences/sruja/config.toml`
/// - Other: `/etc/sruja/config.toml`
fn system_config_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/Library/Preferences/sruja/config.toml")
    }
    #[cfg(not(target_os = "macos"))]
    {
        PathBuf::from("/etc/sruja/config.toml")
    }
}

/// User-level config path following XDG Base Directory Specification.
///
/// Uses `$XDG_CONFIG_HOME/sruja/config.toml`, falling back to
/// `$HOME/.config/sruja/config.toml`.
fn user_config_path() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("sruja/config.toml");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return PathBuf::from(home).join(".config/sruja/config.toml");
        }
    }
    // Fallback: relative path (unlikely but graceful)
    PathBuf::from(".config/sruja/config.toml")
}

// ---------------------------------------------------------------------------
// Layer loading
// ---------------------------------------------------------------------------

/// Load a single TOML file as a layer. Returns an empty layer if the file
/// does not exist or cannot be parsed.
fn load_layer(name: &'static str, path: &Path) -> ConfigLayer {
    if !path.exists() {
        return ConfigLayer::empty(name);
    }
    match std::fs::read_to_string(path) {
        Ok(content) => match toml::from_str::<toml::Value>(&content) {
            Ok(value) => ConfigLayer::new(name, Some(path.to_path_buf()), value),
            Err(e) => {
                eprintln!(
                    "⚠  Failed to parse {name} config at {}: {e}. Layer skipped.",
                    path.display()
                );
                ConfigLayer::empty(name)
            }
        },
        Err(e) => {
            eprintln!(
                "⚠  Failed to read {name} config at {}: {e}. Layer skipped.",
                path.display()
            );
            ConfigLayer::empty(name)
        }
    }
}

/// Load configuration from the `SRUJA_CONFIG` environment variable.
///
/// The value should be a TOML snippet, e.g.:
/// ```text
/// SRUJA_CONFIG='[integrations]\nmodel = "gpt-4"'
/// ```
///
/// Newlines in env vars can be literal (in most shells) or escaped (`\n`).
fn load_env_overrides() -> ConfigLayer {
    let raw = match std::env::var("SRUJA_CONFIG") {
        Ok(v) => v,
        Err(_) => return ConfigLayer::empty("env"),
    };
    if raw.is_empty() {
        return ConfigLayer::empty("env");
    }

    // Handle literal \n as newlines
    let content = raw.replace("\\n", "\n");

    match toml::from_str::<toml::Value>(&content) {
        Ok(value) => ConfigLayer::new("env", None, value),
        Err(e) => {
            eprintln!("⚠  Failed to parse SRUJA_CONFIG env var: {e}. Env layer skipped.");
            ConfigLayer::empty("env")
        }
    }
}

// ---------------------------------------------------------------------------
// Deep merge
// ---------------------------------------------------------------------------

/// Recursively merge `overlay` into `base`.
///
/// Rules:
/// - Both are tables: merge key-by-key, recursing when both sides are tables.
/// - Overlay is not a table: overlay replaces base entirely.
/// - Base is not a table but overlay is: overlay wins.
/// - Arrays: overlay replaces base (no element-level merge).
fn deep_merge(base: &mut toml::Value, overlay: toml::Value) {
    match (base, overlay) {
        (toml::Value::Table(ref mut base_map), toml::Value::Table(overlay_map)) => {
            for (key, overlay_val) in overlay_map {
                match base_map.get_mut(&key) {
                    Some(base_val) if base_val.is_table() && overlay_val.is_table() => {
                        deep_merge(base_val, overlay_val);
                    }
                    _ => {
                        base_map.insert(key, overlay_val);
                    }
                }
            }
        }
        (base, overlay) => *base = overlay,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    /// Serialize tests that mutate environment variables to prevent races.
    /// Recover from mutex poison gracefully.
    /// When a test panics while holding the lock, subsequent tests should
    /// still be able to acquire it.
    fn acquire_env_lock() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Helper to parse a TOML document string into a Value.
    fn toml_val(s: &str) -> toml::Value {
        toml::from_str(s).unwrap()
    }

    #[test]
    fn test_deep_merge_empty_base() {
        let mut base = toml::Value::Table(toml::map::Map::new());
        let overlay = toml_val("[integrations]\nmodel = \"gpt-4\"\n");
        deep_merge(&mut base, overlay);
        assert_eq!(
            base.get("integrations")
                .and_then(|i| i.get("model"))
                .and_then(|m| m.as_str()),
            Some("gpt-4")
        );
    }

    #[test]
    fn test_deep_merge_override_value() {
        let mut base = toml_val("[integrations]\nmodel = \"gpt-3.5\"\n");
        let overlay = toml_val("[integrations]\nmodel = \"gpt-4\"\n");
        deep_merge(&mut base, overlay);
        assert_eq!(
            base.get("integrations")
                .and_then(|i| i.get("model"))
                .and_then(|m| m.as_str()),
            Some("gpt-4")
        );
    }

    #[test]
    fn test_deep_merge_tables_combine() {
        let mut base = toml_val(
            r#"
[integrations]
default_provider = "zai"

[agent.models]
cheap = { provider = "zai", model = "GLM-4-Flash" }
"#,
        );
        let overlay = toml_val(
            r#"
[agent.models]
premium = { provider = "openrouter", model = "claude-sonnet-4" }
"#,
        );
        deep_merge(&mut base, overlay);

        // Original keys preserved
        assert_eq!(
            base.get("integrations")
                .and_then(|i| i.get("default_provider"))
                .and_then(|d| d.as_str()),
            Some("zai")
        );
        // New keys added
        assert_eq!(
            base.get("agent")
                .and_then(|a| a.get("models"))
                .and_then(|m| m.get("premium"))
                .and_then(|p| p.get("model"))
                .and_then(|m| m.as_str()),
            Some("claude-sonnet-4")
        );
        // Old keys in same table preserved
        assert_eq!(
            base.get("agent")
                .and_then(|a| a.get("models"))
                .and_then(|m| m.get("cheap"))
                .and_then(|c| c.get("model"))
                .and_then(|m| m.as_str()),
            Some("GLM-4-Flash")
        );
    }

    #[test]
    fn test_deep_merge_array_replaced() {
        let mut base = toml_val("tools = [\"a\", \"b\"]\n");
        let overlay = toml_val("tools = [\"c\"]\n");
        deep_merge(&mut base, overlay);
        let arr = base.get("tools").and_then(|t| t.as_array());
        assert!(arr.is_some());
        assert_eq!(arr.unwrap().len(), 1);
        assert_eq!(arr.unwrap()[0].as_str(), Some("c"));
    }

    #[test]
    fn test_layered_config_project_only() {
        let _lock = acquire_env_lock();
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        let sruja_dir = repo.join(".sruja");
        fs::create_dir_all(&sruja_dir).unwrap();

        fs::write(
            sruja_dir.join("config.toml"),
            r#"
[integrations]
default_provider = "zai"
model = "glm-4-flash"
"#,
        )
        .unwrap();

        let layered = LayeredConfig::load(repo);
        let cfg: toml::Value = layered.deserialize().unwrap();

        assert_eq!(
            cfg.get("integrations")
                .and_then(|i| i.get("default_provider"))
                .and_then(|d| d.as_str()),
            Some("zai")
        );
        assert_eq!(
            cfg.get("integrations")
                .and_then(|i| i.get("model"))
                .and_then(|m| m.as_str()),
            Some("glm-4-flash")
        );

        // Three file-based layers present (env layer only added if SRUJA_CONFIG set)
        assert_eq!(layered.layers.len(), 3); // system, user, project
        assert_eq!(layered.layers[2].name, "project");
        assert!(layered.layers[2].path.is_some());
        assert!(layered.layers[0].path.is_none()); // system not found
        assert!(layered.layers[1].path.is_none()); // user not found
    }

    #[test]
    fn test_layered_config_project_overrides_system() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        let sruja_dir = repo.join(".sruja");
        fs::create_dir_all(&sruja_dir).unwrap();

        // We can't write to /etc/sruja in tests, so we test the merge logic
        // directly via LayeredConfig by verifying the deep_merge order.
        // Simulate by constructing layers manually.
        let mut merged = toml::Value::Table(toml::map::Map::new());

        let system = toml_val(
            r#"
[integrations]
default_provider = "openai"
model = "gpt-4"

[agent.models]
cheap = { provider = "openai", model = "gpt-4o-mini" }
"#,
        );
        deep_merge(&mut merged, system);

        let project = toml_val(
            r#"
[integrations]
default_provider = "zai"

[agent.models]
premium = { provider = "zai", model = "GLM-4.7" }
"#,
        );
        deep_merge(&mut merged, project);

        // Project overrides default_provider
        assert_eq!(
            merged
                .get("integrations")
                .and_then(|i| i.get("default_provider"))
                .and_then(|d| d.as_str()),
            Some("zai")
        );
        // System model preserved (not overridden by project)
        assert_eq!(
            merged
                .get("integrations")
                .and_then(|i| i.get("model"))
                .and_then(|m| m.as_str()),
            Some("gpt-4")
        );
        // Project adds premium model, system cheap model preserved
        assert_eq!(
            merged
                .get("agent")
                .and_then(|a| a.get("models"))
                .and_then(|m| m.get("premium"))
                .and_then(|p| p.get("model"))
                .and_then(|m| m.as_str()),
            Some("GLM-4.7")
        );
        assert_eq!(
            merged
                .get("agent")
                .and_then(|a| a.get("models"))
                .and_then(|m| m.get("cheap"))
                .and_then(|c| c.get("model"))
                .and_then(|m| m.as_str()),
            Some("gpt-4o-mini")
        );
    }

    #[test]
    fn test_env_overrides_highest_priority() {
        let mut merged = toml::Value::Table(toml::map::Map::new());

        let project = toml_val(
            r#"
[integrations]
model = "glm-4-flash"
"#,
        );
        deep_merge(&mut merged, project);

        let env = toml_val(
            r#"
[integrations]
model = "gpt-4"
"#,
        );
        deep_merge(&mut merged, env);

        assert_eq!(
            merged
                .get("integrations")
                .and_then(|i| i.get("model"))
                .and_then(|m| m.as_str()),
            Some("gpt-4")
        );
    }

    #[test]
    fn test_load_env_overrides_parses_literal_newlines() {
        let _lock = acquire_env_lock();
        // Simulate SRUJA_CONFIG='[integrations]\nmodel = "gpt-4"'
        std::env::set_var("SRUJA_CONFIG", "[integrations]\\nmodel = \"gpt-4\"");
        let layer = load_env_overrides();
        std::env::remove_var("SRUJA_CONFIG");

        assert_eq!(layer.name, "env");
        assert!(layer.path.is_none());
        assert_eq!(
            layer
                .value
                .get("integrations")
                .and_then(|i| i.get("model"))
                .and_then(|m| m.as_str()),
            Some("gpt-4")
        );
    }

    #[test]
    fn test_empty_env_var_skipped() {
        let _lock = acquire_env_lock();
        std::env::set_var("SRUJA_CONFIG", "");
        let layer = load_env_overrides();
        std::env::remove_var("SRUJA_CONFIG");
        assert!(layer.value.as_table().map_or(false, |t| t.is_empty()));
    }

    #[test]
    fn test_deserialize_to_sruja_config() {
        let _lock = acquire_env_lock();
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        let sruja_dir = repo.join(".sruja");
        fs::create_dir_all(&sruja_dir).unwrap();

        fs::write(
            sruja_dir.join("config.toml"),
            r#"
[integrations]
default_provider = "zai"
model = "glm-4-flash"
base_url = "https://open.bigmodel.cn/api/paas/v4"

[integrations.providers.zai]
base_url = "https://api.z.ai/api/coding/paas/v4"
key_env = "ZAI_API_KEY"
"#,
        )
        .unwrap();

        let layered = LayeredConfig::load(repo);
        #[derive(serde::Deserialize)]
        struct TestConfig {
            integrations: TestIntegrations,
        }
        #[derive(serde::Deserialize)]
        struct TestIntegrations {
            default_provider: Option<String>,
            model: Option<String>,
            base_url: Option<String>,
        }

        let cfg: TestConfig = layered.deserialize().unwrap();
        assert_eq!(cfg.integrations.default_provider.as_deref(), Some("zai"));
        assert_eq!(cfg.integrations.model.as_deref(), Some("glm-4-flash"));
    }

    #[test]
    fn test_user_config_path_respects_xdg() {
        let _lock = acquire_env_lock();
        // Without XDG_CONFIG_HOME, should fall back to $HOME/.config
        let prev_xdg = std::env::var("XDG_CONFIG_HOME").ok();
        std::env::remove_var("XDG_CONFIG_HOME");
        let prev_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", "/home/testuser");

        let path = user_config_path();
        assert_eq!(
            path,
            PathBuf::from("/home/testuser/.config/sruja/config.toml")
        );

        // Restore
        if let Some(xdg) = prev_xdg {
            std::env::set_var("XDG_CONFIG_HOME", xdg);
        }
        if let Some(home) = prev_home {
            std::env::set_var("HOME", home);
        }
    }

    #[test]
    fn test_existing_config_paths_only_existing_files() {
        let _lock = acquire_env_lock();
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        let sruja_dir = repo.join(".sruja");
        fs::create_dir_all(&sruja_dir).unwrap();
        fs::write(sruja_dir.join("config.toml"), "").unwrap();

        let paths = LayeredConfig::existing_config_paths(repo);
        // Only the project file should exist
        assert_eq!(paths.len(), 1);
        assert!(paths[0].ends_with(".sruja/config.toml"));
    }

    #[test]
    fn test_deep_merge_three_layers_integration() {
        // Full integration test: system + user + project
        let mut merged = toml::Value::Table(toml::map::Map::new());

        // System layer: sets base provider and model
        let system = toml_val(
            r#"
[integrations]
default_provider = "openai"
model = "gpt-4"

[agent]
allowed_sruja_subcommands = ["sync", "drift"]

[agent.models]
cheap = { provider = "openai", model = "gpt-4o-mini" }
premium = { provider = "openai", model = "gpt-4" }
"#,
        );
        deep_merge(&mut merged, system);

        // User layer: overrides premium model, adds review tier
        let user = toml_val(
            r#"
[agent.models]
premium = { provider = "openrouter", model = "claude-sonnet-4" }
review = { provider = "openrouter", model = "claude-opus-4" }
"#,
        );
        deep_merge(&mut merged, user);

        // Project layer: overrides default_provider, adds cheap model
        let project = toml_val(
            r#"
[integrations]
default_provider = "zai"

[integrations.providers.zai]
base_url = "https://api.z.ai/api/coding/paas/v4"
key_env = "ZAI_API_KEY"

[agent.models]
cheap = { provider = "zai", model = "GLM-4-Flash" }
"#,
        );
        deep_merge(&mut merged, project);

        // Verify: project override of default_provider
        assert_eq!(
            merged
                .get("integrations")
                .and_then(|i| i.get("default_provider"))
                .and_then(|d| d.as_str()),
            Some("zai")
        );
        // Verify: system model still present (not overridden)
        assert_eq!(
            merged
                .get("integrations")
                .and_then(|i| i.get("model"))
                .and_then(|m| m.as_str()),
            Some("gpt-4")
        );
        // Verify: user override of premium model
        assert_eq!(
            merged
                .get("agent")
                .and_then(|a| a.get("models"))
                .and_then(|m| m.get("premium"))
                .and_then(|p| p.get("model"))
                .and_then(|m| m.as_str()),
            Some("claude-sonnet-4")
        );
        // Verify: project override of cheap model
        assert_eq!(
            merged
                .get("agent")
                .and_then(|a| a.get("models"))
                .and_then(|m| m.get("cheap"))
                .and_then(|c| c.get("model"))
                .and_then(|m| m.as_str()),
            Some("GLM-4-Flash")
        );
        // Verify: user-added review tier
        assert_eq!(
            merged
                .get("agent")
                .and_then(|a| a.get("models"))
                .and_then(|m| m.get("review"))
                .and_then(|r| r.get("model"))
                .and_then(|m| m.as_str()),
            Some("claude-opus-4")
        );
        // Verify: system allowed_sruja_subcommands
        assert_eq!(
            merged
                .get("agent")
                .and_then(|a| a.get("allowed_sruja_subcommands"))
                .and_then(|s| s.as_array())
                .map(|a| a.len()),
            Some(2)
        );
    }
}
