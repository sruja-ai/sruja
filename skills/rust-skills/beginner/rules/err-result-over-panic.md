---
metadata:
  complexity: 3
  frequency: rare
  confidence: high
  applicable:
    async: false
    embedded: false
    wasm: false
  category: critical
  level: intermediate
  rust_version: "1.0+"
  alternatives:
    - Return Result<T, E> when errors are expected
    - Use Option for missing values
  related_rules:
    - own-move-large
    - err-result-over-panic
---

# Result over panic

Return `Result<T, E>` instead of panicking on recoverable errors.

## Why

Panics unwind the stack and abort the program (in release builds), which is unacceptable for production software. Using `Result<T, E>` allows callers to handle errors gracefully.

## Examples

### ❌ Don't

```rust
fn parse_age(input: &str) -> u8 {
    input.parse::<u8>().unwrap()
}

fn read_config() -> Config {
    let content = std::fs::read_to_string("config.toml").unwrap();
    toml::from_str(&content).unwrap()
}
```

### ✅ Do

```rust
fn parse_age(input: &str) -> Result<u8, ParseIntError> {
    input.parse::<u8>()
}

fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("config.toml")?;
    Ok(toml::from_str(&content)?)
}
```

## When to Break This Rule

- **Programming errors**: Use `.expect()` for truly unrecoverable situations (e.g., logic bugs)
- **Prototyping**: Quick proof-of-concept code where error handling isn't the focus
- **Test fixtures**: When the test setup guarantees no errors

## Related

- [`err-expect-bugs-only`](rules/err-expect-bugs-only.md) - Use expect only for programming errors
- [`err-question-mark`](rules/err-question-mark.md) - Use ? operator for clean propagation
