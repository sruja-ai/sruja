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
    let config_path = sruja_dir.join("config.toml");
    let mut config = if config_path.exists() {
        std::fs::read_to_string(&config_path).unwrap_or_default()
    } else {
        String::new()
    };

    // Remove existing [integrations] section if present to avoid duplicates.
    if let Some(start) = config.find("\n[integrations]") {
        let rest = &config[start + 1..];
        let end = rest
            .find("\n[")
            .map(|e| start + 1 + e)
            .unwrap_or(config.len());
        config = format!("{}{}", &config[..start], &config[end..]);
    }

    // Build the new [integrations] section.
    let mut section = format!(
        "[integrations]\ndefault_provider = \"openai\"\nmodel = \"{}\"\nbase_url = \"{}\"\n",
        model, preset.base_url
    );
    if !api_key.is_empty() {
        // We don't write the key to config.toml — it stays in env vars.
        // But we tell the user to set it.
    }

    // Prepend the section.
    if !config.is_empty() && !config.starts_with('\n') {
        section.push('\n');
    }
    config = format!("{section}{config}");

    std::fs::write(&config_path, config.trim())?;

    // Step 5: Print summary.
    println!("\n{}", "=".repeat(50));
    println!("Setup complete!");
    println!("{}", "=".repeat(50));
    println!("\nConfig written to: {}", config_path.display());
    println!("  Provider:  {}", preset.name);
    println!("  Base URL:  {}", preset.base_url);
    println!("  Model:     {}", model);

    if !api_key.is_empty() {
        println!("\nAdd to your shell profile (~/.bashrc or ~/.zshrc):");
        println!(
            "  export OPENAI_API_KEY=\"{}\"",
            &api_key[..8.min(api_key.len())]
        );
        println!("  export OPENAI_BASE_URL=\"{}\"", preset.base_url);
        println!("  export OPENAI_MODEL=\"{}\"", model);
        println!("\nThe CLI also reads from .sruja/config.toml automatically.");
    }

    println!("\nTest with:");
    println!("  sruja agent run --goal \"Summarize this repo\"");

    Ok(())
}
