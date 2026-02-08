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
  level: intermediate
  rust_version: "1.56+"
  alternatives:
    - Use Vec::new() when size is unknown
    - Use iterators for streaming data
  related_rules:
    - mem-smallvec
    - mem-thinvec
    - perf-iter-lazy
---

# Preallocate with capacity

Use `Vec::with_capacity()`, `HashMap::with_capacity()`, etc. when you know the approximate size.

## Why

Preallocating avoids costly reallocations as you add elements. Reallocation involves:

- Allocating new memory
- Copying existing elements
- Deallocating old memory

This can be O(n) instead of O(1) amortized.

## Examples

### ❌ Don't

```rust
fn read_lines(path: &Path) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    for line in reader.lines() {
        lines.push(line?);
    }

    Ok(lines)
}
```

### ✅ Do

```rust
fn read_lines(path: &Path) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let line_count = count_lines_fast(path)?;
    let mut lines = Vec::with_capacity(line_count);

    for line in reader.lines() {
        lines.push(line?);
    }

    Ok(lines)
}
```

## When to Break This Rule

- **Unknown size**: When you have no reasonable estimate of the size
- **Streaming data**: When processing elements as they arrive without storing them

## Related

- [`mem-smallvec`](rules/mem-smallvec.md) - Use SmallVec for usually-small collections
- [`perf-iter-lazy`](rules/perf-iter-lazy.md) - Keep iterators lazy
