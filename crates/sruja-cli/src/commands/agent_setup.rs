//! `sruja agent setup` — interactive LLM provider configuration.

use std::io::{self, Write};
use std::path::Path;

use crate::commands::CliError;
use crate::integrations::providers::{self, PRESETS};

/// Run the interactive setup flow.
pub fn agent_setup(
    repo: &str,
    provider_override: Option<&str>,
    api_key_override: Option<&str>,
    model_override: Option<&str>,
) -> Result<(), CliError> {
    let repo_root = Path::new(repo);
    let sruja_dir = repo_root.join(".sruja");
    std::fs::create_dir_all(&sruja_dir)?;

    // Step 1: Select provider.
    let preset = if let Some(pid) = provider_override {
        providers::find_preset(pid).ok_or_else(|| {
            let known: Vec<&str> = PRESETS.iter().map(|p| p.id).collect();
            CliError::validation(format!(
                "Unknown provider '{pid}'. Known: {}",
                known.join(", ")
            ))
        })?
    } else {
        println!("Select an LLM provider:\n");
        for (i, p) in PRESETS.iter().enumerate() {
            println!("  {}. {} — {}", i + 1, p.name, p.default_model);
        }
        println!();
        print!("Enter number [1-{}]: ", PRESETS.len());
        io::stdout().flush().ok();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let idx: usize = input
            .trim()
            .parse::<usize>()
            .map_err(|_| CliError::validation("Invalid number"))?;
        if idx == 0 || idx > PRESETS.len() {
            return Err(CliError::validation(format!(
                "Pick a number between 1 and {}",
                PRESETS.len()
            )));
        }
        &PRESETS[idx - 1]
    };

    println!("\nProvider: {}", preset.name);

    // Step 2: API key (skip for providers that don't need one).
    let api_key = if preset.key_env.is_empty() {
        println!("No API key needed.");
        String::new()
    } else if let Some(key) = api_key_override {
        key.to_string()
    } else {
        // Check env var first.
        if let Ok(existing) = std::env::var(preset.key_env) {
            println!("Found {} in environment.", preset.key_env);
            existing
        } else {
            println!("API key hint: {}", preset.key_hint);
            print!("Enter API key (or set {} env var): ", preset.key_env);
            io::stdout().flush().ok();
            let mut key = String::new();
            io::stdin().read_line(&mut key)?;
            let key = key.trim().to_string();
            if key.is_empty() {
                return Err(CliError::validation("API key is required"));
            }
            key
        }
    };

    // Step 3: Model selection.
    let model = if let Some(m) = model_override {
        m.to_string()
    } else {
        println!("\nDefault model: {}", preset.default_model);
        print!("Use default? [Y/n]: ");
        io::stdout().flush().ok();
        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm)?;
        if confirm.trim().to_lowercase() == "n" {
            print!("Enter model name: ");
            io::stdout().flush().ok();
            let mut custom = String::new();
            io::stdin().read_line(&mut custom)?;
            custom.trim().to_string()
        } else {
            preset.default_model.to_string()
        }
    };

    // Step 4: Write config.toml.
    // Industry best practice: Store non-secret config only.
    // API keys stay in environment variables.
    // Merges with existing config to preserve [agent.models] and other sections.
    let config_path = sruja_dir.join("config.toml");

    // Load existing config or create new one.
    let mut config: toml::Value = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        toml::from_str(&content).unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()))
    } else {
        toml::Value::Table(toml::map::Map::new())
    };

    // Update the [integrations] section.
    let integrations = config
        .as_table_mut()
        .unwrap()
        .entry("integrations")
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()))
        .as_table_mut()
        .unwrap();

    integrations.insert(
        "default_provider".to_string(),
        toml::Value::String(preset.id.to_string()),
    );
    integrations.insert("model".to_string(), toml::Value::String(model.clone()));
    integrations.insert(
        "base_url".to_string(),
        toml::Value::String(preset.base_url.to_string()),
    );

    // Write config with header comment.
    let config_string = toml::to_string_pretty(&config)
        .map_err(|e| CliError::validation(format!("Failed to serialize config: {e}")))?;
    let config_content = format!(
        "# Sruja agent configuration\n\
         # API keys are stored in environment variables, not here.\n\
         # See: sruja agent setup --help\n\
         \n{}",
        config_string
    );

    std::fs::write(&config_path, &config_content)?;

    // Step 5: Print summary.
    println!("\n{}", "=".repeat(50));
    println!("Setup complete!");
    println!("{}", "=".repeat(50));
    println!("\nConfig written to: {}", config_path.display());
    println!("  Provider:  {}", preset.name);
    println!("  Base URL:  {}", preset.base_url);
    println!("  Model:     {}", model);

    if !api_key.is_empty() && !preset.key_env.is_empty() {
        println!("\n{}", "-".repeat(50));
        println!("IMPORTANT: API keys are stored in environment variables.");
        println!("Add this to your shell profile (~/.bashrc or ~/.zshrc):\n");
        println!("  export {}=\"your-api-key-here\"", preset.key_env);
        println!("\nThe CLI will automatically read from {}.", preset.key_env);
    }

    println!("\n{}", "-".repeat(50));
    println!("Test with:");
    println!("  sruja agent run --goal \"Summarize this repo\"");

    Ok(())
}
