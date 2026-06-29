# Sruja Platform

> Architecture-as-code platform. Component knowledge for the root system.

## Purpose

Sruja is the root system boundary for the architecture-as-code platform. It encompasses all tools for defining, visualizing, and analyzing software architecture: CLI, language engine, validation engine, export, WASM, VS Code extension, and the documentation book (mdbook). There are no separate Designer, Website/Playground, Layout, Storybook, or Social Publish apps in the current repo.

## Responsibilities

- Provide a single vocabulary and toolchain for architecture modeling (C4-style)
- Parse, validate, and export architecture DSL (`.sruja` files)
- Integrate with IDEs (VS Code extension) and run in browser (WASM)
- Support discovery, drift, compliance, and documentation workflows

## Dependencies

- **Rust crates**: sruja-language, sruja-engine, sruja-export, sruja-cli, sruja-wasm, sruja-scan, sruja-graph, sruja-diff, sruja-intent, sruja-diagnostics
- **External**: VS Code API (extension), Tree-sitter (scan)

## Known Risks

- WASM bundle size and feature parity with native CLI
- Tight coupling between engine and language if not kept modular

## Suggested Improvements

- [ ] Keep core (language, engine, export) dependency graph acyclic
- [ ] Document public API boundaries per crate
- [ ] Align book/docs with architecture views

## Related Decisions

- ADR001: Rust for Core and CLI
- ADR002: Client-Side Rendering for Playground (WASM)
- ADR003: JSON as Primary Export Format

## Code Locations

- `crates/` — Rust core and product crates (cli, language, engine, export, wasm, scan, graph, diff, intent, diagnostics)
- `extension/` — VS Code extension (TypeScript)
- `book/` — Documentation (mdbook, no separate playground app)
- `docs/architecture/` — Architecture DSL files (this system)

---
*Last updated: 2025-03-16 by Sruja*
