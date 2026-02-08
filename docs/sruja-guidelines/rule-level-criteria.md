# Rust Skills Rule Level Criteria

Defines criteria for categorizing rules by experience level: beginner, intermediate, advanced.

## Overview

Rules are categorized by level to reduce cognitive load and provide appropriate guidance for developers at different stages of their Rust journey.

## Level Definitions

### Beginner (20-25 rules)

**Target Audience:** New Rust developers (0-6 months experience)

**Criteria:**

- Complexity: 1-2
- Critical safety rules that prevent panics and undefined behavior
- Foundational concepts required for basic Rust competence
- High frequency patterns encountered daily
- Clear, unambiguous guidance (confidence: high)

**Focus Areas:**

- Ownership basics (borrowing vs cloning)
- Error handling fundamentals (Result, Option)
- Common anti-patterns (unwrap abuse, string types)
- Basic memory management (Vec::new vs with_capacity)

**Rule Examples:**

- `err-no-unwrap-prod` - Don't use .unwrap() in production
- `err-result-over-panic` - Return Result instead of panicking
- `own-slice-over-vec` - Use &[T] not &Vec<T>
- `type-option-nullable` - Use Option for nullable values
- `type-result-fallible` - Use Result for fallible operations

**When to Apply:**

- New Rust developers learning the language
- Onboarding team members to Rust codebase
- Code reviews for junior developers
- Foundation of Rust knowledge before advanced patterns

### Intermediate (60-80 rules)

**Target Audience:** Daily Rust users (6-24 months experience)

**Criteria:**

- Complexity: 2-4
- Performance optimization patterns
- Async/await patterns and best practices
- API design guidelines for libraries and applications
- Common idiomatic patterns across Rust ecosystem
- Practical trade-offs and when to break rules

**Focus Areas:**

- Async patterns (tokio, concurrency)
- Memory optimization (SmallVec, ThinVec)
- Error handling with context (anyhow, thiserror)
- Iterator patterns and performance
- API design principles (Builder, newtypes)

**Rule Examples:**

- `async-tokio-runtime` - Use Tokio for production async
- `async-no-lock-await` - Never hold Mutex across .await
- `mem-with-capacity` - Preallocate when size known
- `mem-smallvec` - Use SmallVec for usually-small collections
- `api-builder-pattern` - Use Builder for complex construction
- `api-newtype-safety` - Use newtypes for type safety

**When to Apply:**

- Daily Rust development work
- Building production applications
- Designing libraries for public consumption
- Performance-critical code paths
- Writing idiomatic Rust code

### Advanced (80-100 rules)

**Target Audience:** Library authors and expert Rust users (24+ months experience)

**Criteria:**

- Complexity: 3-5
- Compiler optimization techniques
- Low-level memory optimization
- Edge cases and niche patterns
- Advanced trait system usage
- Anti-pattern detection in complex scenarios
- Nuanced trade-offs with multiple valid approaches

**Focus Areas:**

- Compiler hints (inline, cold, likely)
- SIMD and data-parallel operations
- Zero-copy patterns and arena allocators
- Advanced trait bounds and associated types
- FFI and unsafe code patterns
- Performance profiling and benchmarking
- Complex ownership scenarios (Cow, Pin, Arc)

**Rule Examples:**

- `opt-inline-always-rare` - Use #[inline(always)] sparingly
- `opt-simd-portable` - Use portable SIMD for data-parallel ops
- `opt-cache-friendly` - Design cache-friendly layouts (SoA)
- `perf-black-box-bench` - Use black_box() in benchmarks
- `mem-arena-allocator` - Use arena allocators for batch allocations
- `mem-zero-copy` - Use zero-copy with Bytes
- `own-cow-conditional` - Use Cow for conditional ownership

**When to Apply:**

- Writing libraries used by thousands of developers
- Performance-critical systems (latency-sensitive)
- Working with unsafe code and FFI
- Complex trait systems and generic bounds
- Deep optimization and profiling work

## Level Assignment Guidelines

### Decision Matrix

| Factor            | Beginner           | Intermediate                    | Advanced              |
| ----------------- | ------------------ | ------------------------------- | --------------------- |
| **Complexity**    | 1-2                | 2-4                             | 3-5                   |
| **Safety Impact** | Prevents panics/UB | Prevents bugs, improves quality | Optimizes performance |
| **Frequency**     | Common             | Common/Rare                     | Rare/Very-rare        |
| **Prerequisites** | None               | Basic Rust knowledge            | Advanced concepts     |
| **Confidence**    | High               | High/Medium                     | Medium/Low            |

### Complexity Scale

| Level | Description                          | Examples                |
| ----- | ------------------------------------ | ----------------------- |
| **1** | Trivial, one-line change             | `own-slice-over-vec`    |
| **2** | Simple conceptual change             | `mem-with-capacity`     |
| **3** | Moderate complexity, trade-offs      | `own-borrow-over-clone` |
| **4** | Complex, requires deep understanding | `api-builder-pattern`   |
| **5** | Very complex, edge cases             | `opt-simd-portable`     |

### Frequency Impact

- **Common**: Appears in >50% of codebases → Lower level
- **Rare**: Appears in 10-50% of codebases → Appropriate level
- **Very-rare**: Appears in <10% of codebases → Higher level

## Level Progression Path

```
Beginner Rules (Foundation)
    ↓ Learn basic ownership, error handling
Intermediate Rules (Daily Use)
    ↓ Learn async, performance, API design
Advanced Rules (Optimization)
    ↓ Master compiler, memory, low-level
Expert
```

## Example: Rule Classification

### Example 1: `err-no-unwrap-prod`

- **Complexity:** 1 (trivial - don't use unwrap)
- **Frequency:** Common
- **Safety Impact:** Prevents panics
- **Prerequisites:** None
- **Confidence:** High
- **Assignment:** **Beginner**

### Example 2: `async-no-lock-await`

- **Complexity:** 3 (understand async, locks, deadlocks)
- **Frequency:** Common
- **Safety Impact:** Prevents deadlocks
- **Prerequisites:** Basic async, Mutex knowledge
- **Confidence:** High
- **Assignment:** **Intermediate**

### Example 3: `opt-simd-portable`

- **Complexity:** 5 (SIMD intrinsics, data alignment)
- **Frequency:** Very-rare
- **Safety Impact:** Performance optimization
- **Prerequisites:** Advanced Rust, low-level understanding
- **Confidence:** Medium (context-dependent)
- **Assignment:** **Advanced**

## Level-Specific AGENTS.md

### Beginner AGENTS.md

- Quick reference (10-20 rules)
- Focus on critical safety and common errors
- One-page, easy to scan
- Examples are simple and clear

### Intermediate AGENTS.md

- Standard patterns (50-80 rules)
- Focus on performance, patterns, idiomatic code
- Multi-page, categorized by topic
- Examples show real-world scenarios

### Advanced AGENTS.md

- Optimization deep-dive (remaining rules)
- Focus on edge cases, compiler tuning
- Comprehensive reference
- Examples include benchmarks and trade-offs

## Maintenance

- Review level assignments quarterly
- Update based on community feedback
- Track violation rates by level
- Adjust complexity scoring based on developer surveys

---

**Version:** 1.0.0
**Last Updated:** 2025-02-08
