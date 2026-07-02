# add-crate

## Why It Matters

Adding crates correctly ensures they respect layer boundaries and don't introduce architectural violations.

## When to Apply

- Adding a new library or module
- Creating a new crate for a specific feature
- Extracting functionality into a separate crate

## Correct Approach

1. **Identify the target tier**
   - Core Engine: foundational libraries (diagnostics, language, engine)
   - Extraction: graph analysis, artifact extraction
   - Delivery: CLI, WASM bindings
   - Secondary: diff, intent, agent, memory

2. **Add to workspace** in root `Cargo.toml`:
   ```toml
   [workspace]
   members = ["crates/sruja-new-crate"]
   ```

3. **Set dependencies** in `crates/sruja-new-crate/Cargo.toml`:
   ```toml
   [dependencies]
   sruja-language = { path = "../sruja-language" }
   sruja-diagnostics = { path = "../sruja-diagnostics" }
   ```

4. **Create lib.rs** with public API:
   ```rust
   //! Sruja New Crate
   //!
   //! Description of what this crate does.

   pub fn my_function() -> Result<(), Box<dyn std::error::Error>> {
       // Implementation
       Ok(())
   }
   ```

5. **Validate**:
   ```bash
   cargo build --release
   cargo test -p sruja-new-crate
   cargo clippy -p sruja-new-crate -- -D warnings
   ```

## Incorrect Approach

- Adding crate without checking tier boundaries
- Creating dependencies that violate forbidden patterns
- Skipping validation after adding crate

## Summary

**Add crate: identify tier → add to workspace → set dependencies → validate.**
