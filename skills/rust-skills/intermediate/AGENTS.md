# Rust Skills - Intermediate Level

Standard patterns for daily Rust developers (6-24 months experience). Focus on performance optimization, async patterns, API design, and idiomatic code.

## When to Use

This intermediate-level skill set is ideal when:

- Building production applications
- Writing libraries for public consumption
- Working with async/await and concurrency
- Optimizing performance-critical code
- Designing clean, maintainable APIs

---

## Ownership & Borrowing (Intermediate)

- [`own-borrow-over-clone`](rules/own-borrow-over-clone.md) - Prefer `&T` borrowing over `.clone()` for read access
- [`own-cow-conditional`](rules/own-cow-conditional.md) - Use `Cow<'a, T>` for conditional ownership
- [`own-move-large`](rules/own-move-large.md) - Move large data instead of cloning when transferring ownership

---

## Error Handling (Intermediate)

- [`err-anyhow-app`](rules/err-anyhow-app.md) - Use `anyhow` for application error handling
- [`err-context-chain`](rules/err-context-chain.md) - Add context with `.context()` or `.with_context()`
- [`err-question-mark`](rules/err-question-mark.md) - Use `?` operator for clean error propagation
- [`err-thiserror-lib`](rules/err-thiserror-lib.md) - Use `thiserror` for library error types

---

## Memory Optimization (Critical)

- [`mem-with-capacity`](rules/mem-with-capacity.md) - Use `with_capacity()` when size is known
- [`mem-smallvec`](rules/mem-smallvec.md) - Use `SmallVec` for usually-small collections
- [`mem-clone-from`](rules/mem-clone-from.md) - Use `clone_from()` to reuse allocations
- [`mem-reuse-collections`](rules/mem-reuse-collections.md) - Reuse collections with `clear()` in loops

---

## Async/Await Patterns (Critical)

- [`async-tokio-runtime`](rules/async-tokio-runtime.md) - Use Tokio for production async runtime
- [`async-no-lock-await`](rules/async-no-lock-await.md) - Never hold `Mutex`/`RwLock` across `.await`
- [`async-spawn-blocking`](rules/async-spawn-blocking.md) - Use `spawn_blocking` for CPU-intensive work
- [`async-tokio-fs`](rules/async-tokio-fs.md) - Use `tokio::fs` not `std::fs` in async code
- [`async-cancellation-token`](rules/async-cancellation-token.md) - Use `CancellationToken` for graceful shutdown

---

## Compiler Optimization (High)

- [`opt-inline-small`](rules/opt-inline-small.md) - Use `#[inline]` for small hot functions
- [`opt-cold-unlikely`](rules/opt-cold-unlikely.md) - Use `#[cold]` for error/unlikely paths
- [`opt-likely-hint`](rules/opt-likely-hint.md) - Use `likely()`/`unlikely()` for branch hints

---

## API Design (High)

- [`api-builder-pattern`](rules/api-builder-pattern.md) - Use Builder pattern for complex construction
- [`api-newtype-safety`](rules/api-newtype-safety.md) - Use newtypes for type-safe distinctions
- [`api-parse-dont-validate`](rules/api-parse-dont-validate.md) - Parse into validated types at boundaries
- [`api-impl-into`](rules/api-impl-into.md) - Accept `impl Into<T>` for flexible string inputs
- [`api-impl-asref`](rules/api-impl-asref.md) - Accept `impl AsRef<T>` for borrowed inputs

---

## Naming Conventions (Medium)

- [`name-iter-convention`](rules/name-iter-convention.md) - Use `iter`/`iter_mut`/`into_iter` for iterators
- [`name-is-has-bool`](rules/name-is-has-bool.md) - Use `is_`, `has_`, `can_` for boolean methods
- [`name-no-get-prefix`](rules/name-no-get-prefix.md) - No `get_` prefix for simple getters

---

## Type Safety (Medium)

- [`type-newtype-ids`](rules/type-newtype-ids.md) - Wrap IDs in newtypes: `UserId(u64)`
- [`type-newtype-validated`](rules/type-newtype-validated.md) - Newtypes for validated data: `Email`, `Url`
- [`type-enum-states`](rules/type-enum-states.md) - Use enums for mutually exclusive states

---

## Performance Patterns (Medium)

- [`perf-iter-over-index`](rules/perf-iter-over-index.md) - Prefer iterators over manual indexing
- [`perf-iter-lazy`](rules/perf-iter-lazy.md) - Keep iterators lazy, collect() only when needed
- [`perf-collect-once`](rules/perf-collect-once.md) - Don't `collect()` intermediate iterators
- [`perf-entry-api`](rules/perf-entry-api.md) - Use `entry()` API for map insert-or-update
- [`perf-drain-reuse`](rules/perf-drain-reuse.md) - Use `drain()` to reuse allocations
- [`perf-extend-batch`](rules/perf-extend-batch.md) - Use `extend()` for batch insertions

---

## Quick Reference by Category

### Async & Concurrency (5 rules)

| Rule                       | Focus                    | Level    |
| -------------------------- | ------------------------ | -------- |
| `async-tokio-runtime`      | Production async runtime | Critical |
| `async-no-lock-await`      | Deadlock prevention      | Critical |
| `async-spawn-blocking`     | CPU work in async        | Critical |
| `async-tokio-fs`           | Async file I/O           | Critical |
| `async-cancellation-token` | Graceful shutdown        | Critical |

### Memory Optimization (4 rules)

| Rule                    | Benefit             | When to Use   |
| ----------------------- | ------------------- | ------------- |
| `mem-with-capacity`     | Avoid reallocations | Size known    |
| `mem-smallvec`          | Stack allocation    | Usually small |
| `mem-clone-from`        | Reuse allocations   | Clone loops   |
| `mem-reuse-collections` | Reduce allocs       | Hot loops     |

### Error Handling (4 rules)

| Rule                | Library | App | Purpose           |
| ------------------- | ------- | --- | ----------------- |
| `err-anyhow-app`    |         | ✓   | Context errors    |
| `err-thiserror-lib` | ✓       |     | Custom errors     |
| `err-context-chain` | ✓       | ✓   | Error context     |
| `err-question-mark` | ✓       | ✓   | Clean propagation |

### API Design (5 rules)

| Rule                      | Benefit              | Example             |
| ------------------------- | -------------------- | ------------------- |
| `api-builder-pattern`     | Complex construction | `ConfigBuilder`     |
| `api-newtype-safety`      | Type safety          | `UserId(u64)`       |
| `api-impl-into`           | Flexible inputs      | `impl Into<String>` |
| `api-impl-asref`          | Borrowed inputs      | `impl AsRef<str>`   |
| `api-parse-dont-validate` | Boundary validation  | `Email::parse()`    |

---

## Common Intermediate Scenarios

### 1. Building Async API Service

**Apply these rules:**

- `async-tokio-runtime` - Use Tokio runtime
- `async-no-lock-await` - Don't hold locks across await
- `async-tokio-fs` - Use async file I/O
- `err-anyhow-app` - Use anyhow for errors
- `api-builder-pattern` - Builder for config

### 2. Optimizing Hot Loop

**Apply these rules:**

- `mem-with-capacity` - Preallocate
- `perf-iter-lazy` - Keep iterators lazy
- `perf-collect-once` - Don't collect intermediate
- `perf-entry-api` - Use entry API for maps
- `opt-inline-small` - Inline small functions

### 3. Designing Public Library API

**Apply these rules:**

- `api-newtype-safety` - Type-safe IDs
- `api-impl-into` - Accept `impl Into<T>`
- `api-impl-asref` - Accept `impl AsRef<T>`
- `api-parse-dont-validate` - Parse at boundaries
- `err-thiserror-lib` - Custom error types

---

## Trade-offs

### When to Use SmallVec vs Vec

| Scenario                        | Recommendation     |
| ------------------------------- | ------------------ |
| 0-4 elements, stack-allocated   | `SmallVec<[_; 4]>` |
| Often small, occasionally large | `SmallVec<[_; 8]>` |
| Always large                    | `Vec<T>`           |

### When to Clone vs Borrow

| Scenario                       | Clone Cost | Borrow Cost | Recommendation       |
| ------------------------------ | ---------- | ----------- | -------------------- |
| Small struct (< 128 bytes)     | ~10ns      | ~0ns        | Use borrow           |
| In tight loop (1M+ iterations) | ~50ms      | ~0ms        | Use borrow           |
| One-time operation             | Negligible | Negligible  | Clone for simplicity |

### anyhow vs thiserror

| Context          | Use                            |
| ---------------- | ------------------------------ |
| Application code | `anyhow`                       |
| Library crate    | `thiserror`                    |
| Both             | `thiserror` + `anyhow` context |

---

## Learning Path

1. **Master async/await** - Tokio, concurrency, cancellation
2. **Learn memory optimization** - Preallocation, SmallVec, clone patterns
3. **Practice API design** - Builders, newtypes, trait bounds
4. **Understand error handling** - Custom types, context, propagation

---

## Next Steps

After mastering intermediate rules:

→ **Advanced Level**: Compiler optimization, SIMD, zero-copy, FFI
→ Optimize performance-critical systems
→ Write high-performance libraries

---

**Total Rules:** 23+ intermediate rules
**Complexity Range:** 2-4
**Focus:** Performance, async, API design

---

**See Also:**

- [Beginner Rules](../beginner/AGENTS.md) - Foundation concepts
- [Advanced Rules](../advanced/AGENTS.md) - Deep optimization
- [Full Reference](../AGENTS.md) - All 179 rules

---

**Version:** 1.0.0
**Last Updated:** 2025-02-08
