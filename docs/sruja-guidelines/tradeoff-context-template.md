---
# Trade-off Context Template

Use this template when documenting when to break a rule and the implications of doing so.

## Template Structure

```markdown
### When to Break This Rule

Use `<pattern>` when:

- **Scenario 1**: Specific situation with rationale
- **Scenario 2**: Another situation with rationale
- **Scenario 3**: Edge case with justification

### Cost Analysis

| Scenario | <Pattern Name> Cost | Alternative Cost | Recommendation |
|-----------|-------------------|-------------------|----------------|
| Small struct (< 128 bytes) | ~10ns | ~0ns | Use alternative |
| Medium struct (128-512 bytes) | ~50ns | ~0ns | Use alternative |
| Large struct (> 512 bytes) | ~200ns | ~0ns | Use alternative |
| In tight loop (1M+ iterations) | ~50ms | ~0ms | Use alternative |
| One-time operation | Negligible | Negligible | Use <pattern> for simplicity |
```

## Template Guidelines

### "When to Break This Rule" Section

Should include:
1. **Specific scenarios** with clear, actionable conditions
2. **Rationale** for each scenario (why it makes sense)
3. **Context considerations** (project type, constraints, goals)

Example scenarios:
- **Performance profiling**: When benchmarking proves alternative is faster
- **Prototyping**: Quick iteration takes priority over best practices
- **Data transformation**: Need to modify while preserving original
- **Closure complexity**: Avoiding complex lifetime annotations
- **API constraints**: External libraries force specific patterns
- **Legacy compatibility**: Maintaining compatibility with old code

### "Cost Analysis" Table

Columns:
- **Scenario**: Specific use case
- **Pattern Cost**: Performance/memory cost of following the rule
- **Alternative Cost**: Cost of breaking the rule (using alternative)
- **Recommendation**: Which approach to choose

Cost metrics to measure:
- **Performance**: ns, μs, ms (measure with benchmarks)
- **Memory**: bytes, allocations (measure with heaptrack)
- **Complexity**: LOC, cognitive load
- **Maintainability**: Readability, testability

### "Real-World Examples" Section

Should include:
1. **Acceptable violation** - When breaking rule is justified
2. **Unacceptable violation** - When rule should be followed

### "Related Rules" Section

Link to:
- Rules that complement this one
- Rules that provide alternatives
- Rules that conflict with this approach

## Example: Completed Template

```markdown
### When to Break This Rule

Use `.clone()` when:

- **Hot-path profiling**: During benchmarking, you've proven borrow overhead exceeds performance targets
- **Prototyping**: Quick iteration takes priority over optimization
- **Data transformation**: Need to modify while preserving original
- **Closure captures**: Avoid complex lifetime annotations in performance-critical code

### Cost Analysis

| Scenario | Clone Cost | Borrow Cost | Recommendation |
|-----------|-------------|--------------|----------------|
| Small struct (< 128 bytes) | ~10ns | ~0ns | Use borrow |
| Medium struct (128-512 bytes) | ~50ns | ~0ns | Use borrow |
| Large struct (> 512 bytes) | ~200ns | ~0ns | Use borrow |
| In tight loop (1M+ iterations) | ~50ms | ~0ms | Use borrow |
| One-time operation | Negligible | Negligible | Clone for simplicity |

### Real-World Examples

**Acceptable Clone:**

```rust
// Prototyping stage
fn quick_demo(data: Vec<i32>) {
    let copy = data.clone();
    // Complex logic that would require lifetime changes
    println!("{:?}", copy);
}
```

**Unacceptable Clone:**

```rust
// In production hot path
fn process_millions(data: &[i32]) -> Vec<i32> {
    let mut result = Vec::with_capacity(data.len());
    for &x in data {
        let copy = x.clone(); // Unnecessary clone for Copy type
        result.push(copy);
    }
    result
}
```

### Related Rules

- [`own-move-large`](rules/own-move-large.md) - Move large data instead of cloning
- [`perf-iter-lazy`](rules/perf-iter-lazy.md) - Keep iterators lazy
- [`anti-premature-optimize`](rules/anti-premature-optimize.md) - Profile before optimizing
```

## Best Practices

### 1. Use Benchmarks for Cost Analysis

When claiming performance costs, provide benchmark code:

```rust
#[bench]
fn bench_clone(b: &mut Bencher) {
    let data: Vec<u64> = (0..1000).collect();
    
    b.iter(|| {
        let _clone = data.clone();
    });
}

#[bench]
fn bench_borrow(b: &mut Bencher) {
    let data: Vec<u64> = (0..1000).collect();
    
    b.iter(|| {
        let _borrow: &[u64] = &data;
    });
}
```

### 2. Profile Before Optimizing

Always use profiling tools:
- **Flamegraph**: Visualize hot paths
- **Perf**: CPU and memory profiling
- **Heaptrack**: Track allocations
- **Criterion**: Statistical benchmarks

### 3. Consider Project Context

Document which projects this applies to:
- **Embedded systems**: Different constraints than general-purpose
- **WASM**: No threads, different runtime
- **CLI**: One-time startup vs long-running service
- **Library**: Public API ergonomics vs internal optimization

### 4. Provide Migration Path

When breaking rule, explain how to migrate back:
```markdown
### Migration from Alternative to Pattern

If you've been using the alternative and want to adopt the pattern:

1. Replace `<alternative>` with `<pattern>` calls
2. Add lifetime parameters if needed
3. Test with existing test suite
4. Profile to verify performance improvement

**Example:**

```rust
// Before (alternative)
fn process(data: &Vec<i32>) -> i32 {
    let copy = data.clone(); // Alternative
    // ...
}

// After (pattern)
fn process(data: &[i32]) -> i32 {
    let slice = data; // Pattern
    // ...
}
```
```

## Common Trade-off Categories

### Performance vs Ergonomics

| Pattern | Performance | Ergonomics | When to Choose Pattern |
|---------|-------------|------------|---------------------|
| `&T` over `.clone()` | Better | Worse | Performance-critical |
| `.clone()` over `&T` | Worse | Better | Prototyping |
| `impl Into<T>` | Neutral | Better | Public APIs |
| `&T` | Better | Worse | Generic interfaces |

### Safety vs Performance

| Pattern | Safety | Performance | When to Choose |
|---------|---------|-------------|--------------|
| `Mutex<T>` | High | Low | Threaded code |
| `RwLock<T>` | High | Medium | Read-heavy workloads |
| `Unsafe` | Low | High | After careful profiling |
| `Arc<T>` | High | Medium | Shared ownership |

### Memory vs CPU

| Pattern | Memory | CPU | When to Choose |
|---------|----------|------|--------------|
| `Vec<T>` (dynamic) | High | Low | Unknown size |
| `Box<[T]>` (fixed) | Low | High | Known size |
| `SmallVec` | Low | Low | Usually small |

## Trade-off Decision Framework

When deciding whether to break a rule, use this framework:

1. **Identify constraints**: What are your project's requirements?
2. **Measure impact**: Profile the specific code path
3. **Compare alternatives**: What are the trade-offs?
4. **Document decision**: Why did you choose this approach?
5. **Review periodically**: Re-evaluate as project evolves

### Questions to Ask

- Is this a hot path? (Called frequently)
- What's the data size involved?
- Are there memory constraints?
- What's the maintenance cost of this approach?
- Will this pattern scale?
- Are there external dependencies forcing this choice?

---

## Checklist for Rule Authors

When adding trade-off context to a rule:

- [ ] Specific scenarios with clear conditions
- [ ] Rationale for each scenario
- [ ] Cost analysis table with measured metrics
- [ ] Real-world code examples (good and bad)
- [ ] Related rules linked
- [ ] Context-specific notes (embedded, wasm, CLI, library)
- [ ] Migration path documented (if applicable)
- [ ] Benchmarks provided (for performance claims)

---

**Version:** 1.0.0  
**Last Updated:** 2025-02-08
