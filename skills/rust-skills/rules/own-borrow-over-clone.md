---
metadata:
  complexity: 3
  frequency: common
  confidence: high
  applicable:
    is_async: true
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

Use `.clone()` when:

- **Hot-path profiling**: During benchmarking, you've proven borrow overhead exceeds performance targets (typically > 50ns for hot loops)
- **Prototyping**: Quick iteration takes priority over optimization during early development
- **Data transformation**: Need to modify data while preserving the original
- **Closure captures**: Avoid complex lifetime annotations in performance-critical code
- **API compatibility**: Working with external APIs that require owned values
- **Simple iteration**: One-time code paths where clarity matters more than micro-optimizations

## Cost Analysis

| Scenario                       | Clone Cost | Borrow Cost | Recommendation       |
| ------------------------------ | ---------- | ----------- | -------------------- |
| Small struct (< 128 bytes)     | ~10ns      | ~0ns        | Use borrow           |
| Medium struct (128-512 bytes)  | ~50ns      | ~0ns        | Use borrow           |
| Large struct (> 512 bytes)     | ~200ns     | ~0ns        | Use borrow           |
| In tight loop (1M+ iterations) | ~50ms      | ~0ms        | Use borrow           |
| One-time operation             | Negligible | Negligible  | Clone for simplicity |

\*Costs measured on x86-64 with typical Rust compiler optimizations. Actual costs vary by target and compiler version.

### Performance Notes

- Borrowing adds no runtime cost - it's just a reference
- Cloning involves memory allocation and memcpy
- For Copy types, cloning is as cheap as moving
- Profile with `cargo bench` or `criterion` for your specific use case

## Real-World Examples

### Acceptable Clone

```rust
// Prototyping - quick iteration
fn quick_demo(data: &Vec<i32>) {
    // Clone for simplicity while exploring logic
    let copy = data.clone();
    let result = complex_transform(&copy);
    println!("{:?}", result);
}

// Data transformation - need to modify both versions
fn transform_and_preserve(original: &mut Vec<i32>) {
    let working_copy = original.clone();
    transform(&mut working_copy);
    // ... working_copy modified, original preserved separately
}
```

### Unacceptable Clone

```rust
// Hot path in production - avoid unnecessary clones
fn process_stream(stream: impl Stream<Item = Data>) -> Vec<Result> {
    stream
        .map(|data| {
            // ❌ Don't clone for each item in hot path
            let owned = data.content.clone();
            process(owned)
        })
        .collect()
}

// ✅ Better: borrow where possible
fn process_stream_better(stream: impl Stream<Item = Data>) -> Vec<Result> {
    stream
        .map(|data| {
            process(&data.content)
        })
        .collect()
}
```

## Related Rules

- [`own-move-large`](rules/own-move-large.md) - Move large data instead of cloning
- [`perf-iter-lazy`](rules/perf-iter-lazy.md) - Keep iterators lazy
- [`anti-premature-optimize`](rules/anti-premature-optimize.md) - Profile before optimizing

## References

- [Rust Book - Ownership](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)
- [Rust Performance Book - Cloning](https://nnethercote.github.io/perf-book/standard-library.html#clone)
- [clippy::clone_on_ref_ptr](https://rust-lang.github.io/rust-clippy/master/index.html#clone_on_ref_ptr)
