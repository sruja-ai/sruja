# common-patterns

## Why It Matters

Following established patterns ensures consistency and reduces the risk of introducing architectural violations.

## When to Apply

- Designing new components
- Refactoring existing code
- Reviewing architecture changes

## Rust Patterns

### Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

// Use ? operator for propagation
fn my_function() -> Result<(), MyError> {
    let value = get_value()?;
    Ok(())
}
```

### Module Organization

```rust
// lib.rs - Public API surface
pub mod types;
pub mod error;
pub use types::{MyType, MyOtherType};
pub use error::MyError;

// Internal implementation in submodules
mod internal;
```

### Serde Serialization

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyType {
    pub field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MyEnum {
    Variant1,
    Variant2,
}
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        let result = my_function();
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_case() {
        let result = my_function_with_error();
        assert!(result.is_err());
    }
}
```

## TypeScript Patterns (Extension)

### Error Handling

```typescript
try {
    const result = await operation();
    return result;
} catch (err) {
    vscode.window.showErrorMessage(`Error: ${err}`);
}
```

### VS Code API

```typescript
import * as vscode from "vscode";

export function activate(context: vscode.ExtensionContext) {
    const disposable = vscode.commands.registerCommand(
        'extension.myCommand',
        () => {
            // Implementation
        }
    );
    context.subscriptions.push(disposable);
}
```

## Summary

**Patterns: follow established conventions, check existing code, maintain consistency.**
