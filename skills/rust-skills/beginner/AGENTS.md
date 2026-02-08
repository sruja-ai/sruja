# Rust Skills - Beginner Level

Quick reference for new Rust developers (0-6 months experience). Focus on critical safety rules and common mistakes.

## When to Use

This beginner-level skill set is ideal when:

- Learning Rust for the first time
- Onboarding new team members
- Code reviewing junior developers
- Building a foundation of Rust knowledge

---

## Critical Safety Rules (Foundational)

### Ownership & Borrowing

- [`own-slice-over-clone`](rules/own-slice-over-vec.md) - Use `&[T]` instead of `&Vec<T>`, `&str` instead of `&String`

### Error Handling

- [`err-no-unwrap-prod`](rules/err-no-unwrap-prod.md) - Never use `.unwrap()` in production code
- [`err-result-over-panic`](rules/err-result-over-panic.md) - Return `Result<T, E>` instead of panicking on recoverable errors

### Type Safety

- [`type-option-nullable`](rules/type-option-nullable.md) - Use `Option<T>` for nullable values instead of null/uninitialized
- [`type-result-fallible`](rules/type-result-fallible.md) - Use `Result<T, E>` for fallible operations

---

## Quick Reference

| Rule ID                 | Title                     | Why It Matters                        |
| ----------------------- | ------------------------- | ------------------------------------- |
| `err-no-unwrap-prod`    | No unwrap in production   | Prevents panics and crashes           |
| `err-result-over-panic` | Result over panic         | Graceful error handling               |
| `own-slice-over-vec`    | Slices over owned vectors | More flexible, zero-cost abstractions |
| `type-option-nullable`  | Option for nullable       | Null safety at compile time           |

---

## Common Beginner Mistakes

### 1. Using `.unwrap()` Everywhere

**Problem:** Code panics in production.

**Solution:** Use pattern matching or `?` operator.

```rust
// ❌ Don't
let value = some_option.unwrap();

// ✅ Do
let value = some_option?;
// or
match some_option {
    Some(v) => { /* use v */ },
    None => { /* handle missing */ }
}
```

### 2. Passing `&Vec<T>` and `&String`

**Problem:** Forces heap allocation, less flexible.

**Solution:** Use `&[T]` and `&str`.

```rust
// ❌ Don't
fn process(vec: &Vec<i32>, s: &String) { }

// ✅ Do
fn process(vec: &[i32], s: &str) { }
```

### 3. Panicking Instead of Returning Errors

**Problem:** Program terminates unexpectedly.

**Solution:** Return `Result<T, E>`.

```rust
// ❌ Don't
fn parse(input: &str) -> i32 {
    input.parse().unwrap()
}

// ✅ Do
fn parse(input: &str) -> Result<i32, ParseIntError> {
    input.parse()
}
```

---

## Learning Path

1. **Start with error handling** - Understand `Option` and `Result`
2. **Learn ownership basics** - Borrowing, lifetimes, moves
3. **Practice pattern matching** - `match`, `if let`, `while let`
4. **Review common anti-patterns** - What not to do

---

## Next Steps

After mastering beginner rules:

→ **Intermediate Level**: Async patterns, performance, API design
→ Build production applications with confidence
→ Read standard library documentation

---

**Total Rules:** 5 beginner rules
**Complexity Range:** 1-2
**Focus:** Safety and foundational concepts

---

**See Also:**

- [Intermediate Rules](../intermediate/AGENTS.md) - Next level up
- [Advanced Rules](../advanced/AGENTS.md) - Deep optimization
- [Full Reference](../AGENTS.md) - All 179 rules

---

**Version:** 1.0.0
**Last Updated:** 2025-02-08
