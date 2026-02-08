---
metadata:
  complexity: 3
  frequency: rare
  confidence: high
  applicable:
    is_async: false
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
    - err-no-unwrap-prod
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

Use panics when:

- **Programming errors**: Use `.expect()` for truly unrecoverable situations (logic bugs, invariants)
- **Prototyping**: Quick proof-of-concept code where error handling isn't the focus
- **Test fixtures**: When test setup guarantees no errors
- **Assertions**: Intentionally panicking to verify invariants

## Cost Analysis

| Scenario               | Panic Behavior             | Result Cost             | Recommendation            |
| ---------------------- | -------------------------- | ----------------------- | ------------------------- |
| Production code        | Crashes on error           | Graceful handling       | Use Result                |
| Test code              | Immediate failure on error | Verbose diagnostics     | Acceptable for clarity    |
| Documentation examples | Simple & focused           | Verbose error handling  | Acceptable for simplicity |
| Debug builds           | Crashes on invariant       | Detailed panic message  | Use for assertions        |
| Invariant checks       | Crashes on bug             | Early failure detection | Acceptable for checks     |

### Impact Analysis

**Production panics:**

- **Availability**: Zero - any error causes crash
- **User experience**: Terrible - cryptic panic messages
- **Debuggability**: Hard - stack traces may not capture error context
- **Monitoring**: No error metrics - panics are invisible to observability
- **Data corruption**: Possible if invariants are violated during cleanup

**Result-based error handling:**

- **Availability**: High - errors logged, degraded service continues
- **User experience**: Good - meaningful error messages
- **Debuggability**: Excellent - full error context in logs
- **Monitoring**: Complete - error rates, types, patterns tracked
- **Data integrity**: Maintained - transactions can be rolled back

## Real-World Examples

### Acceptable Panic

```rust
// Test code - panic with expect is fine here
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Value out of range")]
    fn test_out_of_bounds() {
        // ✅ Intentional panic to test invariant
        let data = [1, 2, 3];
        let _index = unsafe { *data.get_unchecked(10) };
    }
}

// Invariant check with descriptive message
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
            panic!("Buffer overflow: capacity {} exceeded, {}", self.capacity, self.buffer.len());
        }
    }
}
```

### Unacceptable Panic

```rust
// Production HTTP handler
async fn handle_order(order: Order) -> HttpResponse {
    // ❌ Panics on parse error - crashes entire service
    let items: Vec<OrderItem> = order.items
        .into_iter()
        .map(|item| parse_item(item).unwrap())
        .collect();

    HttpResponse::Ok(create_order(items))
}

// ✅ Better: handle errors gracefully
async fn handle_order_better(order: Order) -> HttpResponse {
    let mut items = Vec::with_capacity(order.items.len());

    for item in order.items {
        match parse_item(&item) {
            Ok(parsed) => items.push(parsed),
            Err(e) => {
                error_log!("Failed to parse item {}: {}", item.id, e);
                return HttpResponse::BadRequest(format!("Invalid item: {}", e));
            }
        }
    }

    HttpResponse::Ok(create_order(items))
}

// Database operation with panic
fn delete_user(id: u64) {
    // ❌ Crashes on database error
    let conn = get_connection().unwrap();
    conn.execute("DELETE FROM users WHERE id = ?", &[id]).unwrap();
}

// ✅ Better: propagate database errors
fn delete_user_better(id: u64) -> Result<(), DbError> {
    let conn = get_connection()?;
    conn.execute("DELETE FROM users WHERE id = ?", &[id])?;
    Ok(())
}
```

### Error Recovery Strategies

### 1. Result Chain with Context

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum ProcessingError {
    #[error("Failed to parse input: {0}")]
    ParseError(#[from] ParseIntError),

    #[error("I/O error reading config: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

fn process_config(path: &Path) -> Result<Config, ProcessingError> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;

    Ok(config)
}
```

### 2. Fallback and Defaults

```rust
// Provide sensible defaults instead of panicking
fn get_timeout(config: &Config) -> Duration {
    config.timeout.unwrap_or(Duration::from_secs(30))
}

fn get_max_retries(config: &Config) -> u32 {
    config.max_retries.unwrap_or(3)
}
```

### 3. Validation at Boundaries

```rust
// Validate inputs at API boundaries, fail fast
pub fn create_user(input: CreateUserRequest) -> Result<User, ValidationError> {
    let email = input.email.parse::<Email>()?;
    let name = validate_name(&input.name)?;

    Ok(User::new(email, name))
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

## Assertion Best Practices

```rust
// ✅ Use expect with descriptive messages
assert_eq!(result, expected, "Expected {} but got {}", expected, result);
assert!(condition, "Invariant violated: {}", explanation);

// ✅ Use debug_assert for expensive checks
debug_assert!(is_sorted(&data), "Data should be sorted");

// ✅ Don't use unwrap for external input
fn process_external(input: &str) -> Result {
    // ❌ Never: unwrap() on user input
    let value = input.parse().unwrap();

    // ✅ Better: parse and handle error
    let value = input.parse()?;
    Ok(value)
}
```

## Related Rules

- [`err-no-unwrap-prod`](rules/err-no-unwrap-prod.md) - Never use .unwrap() in production
- [`err-expect-bugs-only`](rules/err-expect-bugs-only.md) - Use expect only for programming errors
- [`err-question-mark`](rules/err-question-mark.md) - Use ? operator for clean propagation
- [`err-anyhow-app`](rules/err-anyhow-app.md) - Use anyhow for application error handling
- [`err-thiserror-lib`](rules/err-thiserror-lib.md) - Use thiserror for library error types

## References

- [Rust Book - Error Handling](https://doc.rust-lang.org/book/ch09-00-recoverable-errors-with-result.html)
- [Why Exceptions are Bad](https://www.joelonsoftware.com/2007/06/15/blog-why-exceptions-are-bad.html)
- [Error Handling Best Practices](https://rust-lang.github.io/rust-clippy/master/index.html#result_err_used)
