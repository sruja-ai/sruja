# IDE Support (VS Code Extension)

> VS Code extension language features powered by the WASM build.

## Purpose

The VS Code extension provides language intelligence for Sruja DSL inside VS Code: diagnostics (lint), hover, and preview. It reuses the same Rust core logic via the WASM build (parser + validation + export).

## Responsibilities

- Parse documents on open/change and emit diagnostics
- Hover: show kind, description, and doc path (component knowledge) where available
- Provide preview for diagrams and exported views

## Dependencies

- **Internal**: sruja-language (parse), sruja-engine (validate), sruja-export (preview), sruja-wasm (bridge)
- **External**: VS Code API

## Known Risks

- WASM bundle size and feature parity with native CLI
- Latency targets for editor interactions (hover/diagnostics/preview)

## Suggested Improvements

- [ ] Document supported VS Code features and configuration options
- [ ] Keep feature parity between CLI and WASM where both are used

## Related Decisions

- ADR001: Rust for Core and CLI

## Code Locations

- `extension/` — VS Code extension (TypeScript)
- `crates/sruja-wasm/` — WASM bindings used by the extension

---
*Last updated: 2025-03-16 by Sruja*
