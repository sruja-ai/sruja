---
metadata:
  complexity: 2
  frequency: common
  confidence: high
  applicable:
    async: false
    embedded: false
    wasm: false
  category: critical
  level: beginner
  rust_version: "1.0+"
  alternatives: []
  related_rules:
    - err-result-over-panic
    - err-expect-bugs-only
---

# No unwrap in production

Never use `.unwrap()` in production code.

## Why

`.unwrap()` will panic and terminate the program if the value is `None` or `Err`. This is unacceptable in production where you want to handle errors gracefully.

## Examples

### ❌ Don't

```rust
fn get_user(id: u32) -> User {
    database.find_user(id).unwrap()
}

fn process_request(req: Request) -> Response {
    let user = authenticate(req).unwrap();
    handle_request(user)
}
```

### ✅ Do

```rust
fn get_user(id: u32) -> Option<User> {
    database.find_user(id)
}

fn process_request(req: Request) -> Result<Response, AuthError> {
    let user = authenticate(req)?;
    Ok(handle_request(user))
}
```

## When to Break This Rule

- **Test code**: Using `.unwrap()` in tests is acceptable and often clearer
- **Examples**: Documentation examples where error handling would obscure the main point
- **Prototyping**: Quick development where you'll add proper error handling later

## Related

- [`err-result-over-panic`](rules/err-result-over-panic.md) - Return Result, don't panic
- [`err-expect-bugs-only`](rules/err-expect-bugs-only.md) - Use expect only for bugs
