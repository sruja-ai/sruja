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
  alternatives:
    - Accept &Vec for FFI bindings
    - Accept &String when working with specific APIs
  related_rules:
    - own-cow-conditional
    - anti-string-for-str
---

# Use slices over owned vectors

Accept `&[T]` instead of `&Vec<T>`, and `&str` instead of `&String`.

## Why

Slices (`&[T]`, `&str`) are more flexible because they can reference any contiguous memory, including:

- Subslices of existing data
- Static string literals
- Array slices

Owned references (`&Vec<T>`, `&String`) force the data to be heap-allocated, which is more restrictive.

## Examples

### ❌ Don't

```rust
fn sum(vec: &Vec<i32>) -> i32 {
    vec.iter().sum()
}

fn greet(name: &String) -> String {
    format!("Hello, {}!", name)
}
```

### ✅ Do

```rust
fn sum(vec: &[i32]) -> i32 {
    vec.iter().sum()
}

fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

## When to Break This Rule

- **FFI bindings**: When interfacing with C code that requires owned pointers
- **Specific API requirements**: Some crates require `&Vec<T>` or `&String` for internal reasons

## Related

- [`own-cow-conditional`](rules/own-cow-conditional.md) - Use Cow for conditional ownership
- [`anti-string-for-str`](rules/anti-string-for-str.md) - Don't use &String when &str works
