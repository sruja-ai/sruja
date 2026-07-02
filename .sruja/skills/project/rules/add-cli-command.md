# add-cli-command

## Why It Matters

CLI commands are the user-facing interface. Adding them correctly ensures consistency and discoverability.

## When to Apply

- Adding a new user-facing feature
- Exposing existing functionality via CLI
- Creating new analysis or export commands

## Correct Approach

1. **Define the command** in `src/cli/commands.rs`:
   ```rust
   #[command(name = "my-command")]
   MyCommand {
       /// Path to repository root
       #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
       repo: String,
       /// Output format (text or json)
       #[arg(long, short = 'f', default_value = "text")]
       format: String,
   },
   ```

2. **Add handler** in `src/cli/run.rs`:
   ```rust
   Commands::MyCommand { repo, format } => {
       commands::my_module::my_command(&repo, &format).await
   }
   ```

3. **Implement the command** in `src/commands/my_module.rs`:
   ```rust
   use super::CliError;

   pub async fn my_command(repo: &str, format: &str) -> Result<(), CliError> {
       // Implementation
       Ok(())
   }
   ```

4. **Export** in `src/commands/mod.rs`:
   ```rust
   pub use my_module::my_command;
   ```

5. **Test**:
   ```bash
   cargo test -p sruja-cli
   ./target/release/sruja my-command --help
   ./target/release/sruja my-command -r . -f json
   ```

## Incorrect Approach

- Adding command without proper argument validation
- Not supporting both text and json output formats
- Skipping help text documentation

## Summary

**Add CLI command: define args → add handler → implement → export → test.**
