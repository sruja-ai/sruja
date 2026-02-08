---
metadata:
  complexity: 2
  frequency: common
  confidence: high
  applicable:
    is_async: false
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

`.unwrap()` will panic and terminate the program if the value is `None` or `Err`. This is unacceptable in production where you want to handle errors gracefully and provide meaningful error messages.

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

Use `.unwrap()` when:

- **Test code**: Using `.unwrap()` in tests is acceptable and often clearer
- **Examples**: Documentation examples where error handling would obscure the main point
- **Prototyping**: Quick development where you'll add proper error handling later
- **Debug assertions**: When intentionally causing panics to verify invariants
- **Command-line tools**: One-off scripts where crashing is acceptable

## Cost Analysis

| Scenario               | Unwrap Behavior            | Alternative Cost       | Recommendation            |
| ---------------------- | -------------------------- | ---------------------- | ------------------------- |
| Production code        | Crashes on error           | Graceful handling      | Never unwrap              |
| Test suite             | Immediate failure on error | Verbose diagnostics    | Acceptable for clarity    |
| Documentation examples | Simple & focused           | Verbose error handling | Acceptable for simplicity |
| Debug builds           | Crashes on invariant       | Detailed panic message | Use for assertions        |

### Impact Analysis

**Production use of `.unwrap()`:**

- **Availability**: Zero - any error causes crash
- **User experience**: Terrible - cryptic panic messages
- **Debuggability**: Hard - stack traces may not capture context
- **Monitoring**: No error metrics - panics are invisible to observability
- **Data corruption**: Possible if invariants are violated

**Graceful error handling:**

- **Availability**: High - errors logged, degraded service continues
- **User experience**: Good - meaningful error messages
- **Debuggability**: Excellent - full error context in logs
- **Monitoring**: Complete - error rates, types, patterns tracked
- **Data integrity**: Maintained - transactions can be rolled back

## Real-World Examples

### Acceptable Unwrap

````rust
// Test code - unwrap is fine here
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_input() {
        // ✅ In tests, unwrap is clear and acceptable
        let result = parse("123");
        assert_eq!(result, 123);
    }
}

// Documentation example
/// Parse a string into an integer.
///
/// # Examples
/// ```
/// let n = parse("42").unwrap();  // Returns 42
/// ```
fn parse(input: &str) -> Option<u32> {
    input.parse().ok()
}
````

### Unacceptable Unwrap

```rust
// Production HTTP handler
async fn handle_user(request: HttpRequest) -> HttpResponse {
    // ❌ Panics on auth failure - crashes entire service
    let user = authenticate_user(request.token).unwrap();

    HttpResponse::Ok(get_user_data(user))
}

// ✅ Better: handle errors gracefully
async fn handle_user_better(request: HttpRequest) -> HttpResponse {
    match authenticate_user(request.token).await {
        Ok(user) => HttpResponse::Ok(get_user_data(user)),
        Err(e) => {
            error_log!("Authentication failed: {}", e);
            HttpResponse::Unauthorized()
        }
    }
}

// Database query in production
fn get_user_by_email(email: &str) -> User {
    // ❌ Crashes on "user not found"
    database.query("SELECT * FROM users WHERE email = ?", &[email])
        .unwrap()
        .get(0)
}

// ✅ Better: return Option or Result
fn get_user_by_email_better(email: &str) -> Option<User> {
    database.query("SELECT * FROM users WHERE email = ?", &[email])
        .ok()
        .and_then(|row| row.get(0))
}

// Multiple unwaps in sequence - cascading failures
fn process_data(id: u32) -> Result {
    // ❌ Each unwrap can panic, hard to debug which one failed
    let config = load_config().unwrap();
    let data = fetch_data(id, config).unwrap();
    let result = process(data).unwrap();

    Ok(result)
}

// ✅ Better: use ? for clean propagation
fn process_data_better(id: u32) -> Result {
    let config = load_config()?;
    let data = fetch_data(id, &config)?;
    let result = process(&data)?;

    Ok(result)
}
```

### Assertion Patterns

```rust
// ✅ Use unwrap with expect for invariants
struct BoundedBuffer<T> {
    buffer: Vec<T>,
    capacity: usize,
}

impl<T> BoundedBuffer<T> {
    fn push(&mut self, item: T) {
        if self.buffer.len() < self.capacity {
            self.buffer.push(item);
        } else {
            // ✅ Panic with meaningful message for invariant violation
            panic!("Buffer overflow: capacity {} exceeded", self.capacity);
        }
    }

    fn get(&self, index: usize) -> &T {
        self.buffer[index].expect("Invariant: index should be valid")
    }
}
```

## Error Recovery Strategies

### 1. Graceful Degradation

```rust
// Instead of crashing, degrade functionality
fn get_cached_data(key: &str) -> Data {
    match cache.get(key) {
        Some(data) => data,
        None => {
            warn_log!("Cache miss for key: {}, using fallback", key);
            fallback_data(key)
        }
    }
}
```

### 2. Circuit Breaker Pattern

```rust
// Use Result to track and prevent repeated failures
use std::time::{Duration, Instant};

async fn fetch_with_retry(key: &str) -> Result<Data> {
    let timeout = Duration::from_secs(30);
    let deadline = Instant::now() + timeout;

    while Instant::now() < deadline {
        match fetch(key).await {
            Ok(data) => return Ok(data),
            Err(e) if is_transient(&e) => continue,
            Err(e) => return Err(e),
        }
    }

    Err(Error::Timeout)
}

fn is_transient(err: &Error) -> bool {
    matches!(err, Error::Network(_) | Error::Timeout)
}
```

### 3. Validation at Boundaries

```rust
// Validate inputs at API boundaries, fail fast
pub fn create_user(input: CreateUserRequest) -> Result<User, ValidationError> {
    // ✅ Validate before any database work
    let email = input.email.parse::<Email>()?;
    let name = validate_name(&input.name)?;

    let user = User::new(email, name);
    Ok(user)
}

fn validate_name(name: &str) -> Result<String, ValidationError> {
    if name.len() < 2 {
        return Err(ValidationError::TooShort);
    }
    if name.len() > 100 {
        return Err(ValidationError::TooLong);
    }

    Ok(name.to_string())
}
```

## Related Rules

- [`err-result-over-panic`](rules/err-result-over-panic.md) - Return Result instead of panicking
- [`err-expect-bugs-only`](rules/err-expect-bugs-only.md) - Use expect only for programming errors
- [`err-question-mark`](rules/err-question-mark.md) - Use ? operator for clean propagation
- [`err-anyhow-app`](rules/err-anyhow-app.md) - Use anyhow for application error handling

## References

- [Rust Book - Error Handling](https://doc.rust-lang.org/book/ch09-00-recoverable-errors-with-result.html)
- [Why Exceptions are Bad](https://www.joelonsoftware.com/2007/06/15/blog-why-exceptions-are-bad.html)
- [Clippy unwrap warnings](https://rust-lang.github.io/rust-clippy/master/index.html#unwrap_used)
