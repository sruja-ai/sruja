---
metadata:
  complexity: 3
  frequency: common
  confidence: high
  applicable:
    async: true
    embedded: false
    wasm: false
  category: critical
  level: intermediate
  rust_version: "1.0+"
  alternatives:
    - Use clone() in hot-path profiling code
    - Use clone() when benchmarking proves borrow overhead > 50ns
  related_rules:
    - own-move-large
    - anti-clone-excessive
---

# Prefer borrowing over cloning

Use `&T` references instead of `.clone()` when you only need read access.

## Why

Cloning data copies the entire value, which can be expensive for large types. Borrowing with `&T` is zero-cost and enables Rust's ownership system to enforce memory safety at compile time.

## Examples

### ❌ Don't

```rust
fn process(data: Vec<i32>) {
    let copy = data.clone();
    analyze(&copy);
}
```

### ✅ Do

```rust
fn process(data: Vec<i32>) {
    analyze(&data);
}
```

## When to Break This Rule

- **Hot-path profiling**: During benchmarking, you've proven borrow overhead exceeds performance targets
- **Prototyping**: Quick iteration takes priority over optimization
- **Data transformation**: Need to modify while preserving original
- **Closure captures**: Avoid complex lifetime annotations in performance-critical code

## Related

- [`own-move-large`](rules/own-move-large.md) - Move large data instead of cloning
- [`anti-clone-excessive`](rules/anti-clone-excessive.md) - Don't clone when borrowing works
