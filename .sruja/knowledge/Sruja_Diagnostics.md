# Sruja Diagnostics

> Structured diagnostic reporting with source locations.

## Purpose

Provides structured error and warning reporting with source locations, context, and suggestions for the Sruja DSL. Compatible with the Go implementation while leveraging Rust's type safety. Used as a foundation crate by the parser, diff, and agent crates.

## Responsibilities

- Define `Diagnostic` type with severity, location, message, and suggestions
- Provide `ErrorReporter` trait for collecting diagnostics
- Format diagnostics for human-readable output
- Format diagnostics for GitHub Actions annotations
- Define standard diagnostic codes

## Dependencies

- **Internal**: None (leaf crate)
- **External**: serde, serde_json

## Key Types

- `Diagnostic` — Structured diagnostic with severity, location, message
- `Severity` — Error, Warning, Info, Hint
- `SourceLocation` — File path, line, column
- `ErrorReporter` — Trait for collecting diagnostics
- `BasicErrorReporter` — Simple in-memory reporter
- `format_diagnostic()` — Human-readable formatting
- `format_github_actions_annotation()` — CI annotation formatting

## Code Locations

- `crates/sruja-diagnostics/` — Diagnostics crate
- `src/types.rs` — Core types
- `src/reporter.rs` — Reporter trait
- `src/format.rs` — Formatting functions
- `src/codes.rs` — Standard diagnostic codes

## Notes

- Both `rlib` and `cdylib` (for FFI/WASM interop)
- No internal dependencies — this is a leaf crate

---
*Last updated: 2026-06-06*
