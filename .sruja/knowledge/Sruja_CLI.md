# Sruja CLI

> Command-line interface for Sruja operations.

## Purpose

The CLI is the primary entry point for headless and scripted use: lint, export, sync, discover, drift, compliance, knowledge, and other subcommands. It composes the language, engine, and export crates and can spawn or integrate with the LSP.

## Responsibilities

- Parse and validate `.sruja` files via `sruja lint`
- Export to JSON, Markdown, Mermaid, DSL via `sruja export`
- Repository context and discovery via `sruja sync`, `sruja discover`
- Drift and intent comparison via `sruja drift`, `sruja intent`
- Compliance and knowledge commands (`sruja compliance`, `sruja knowledge`)

## Dependencies

- **Internal**: Sruja_Language (parse), Sruja_Engine (validate), Sruja_Export (export), Sruja_LSP (optional start)
- **External**: clap (argument parsing), filesystem, optional network for discover/sync

## Known Risks

- CLI must stay thin; heavy logic lives in engine and language crates
- Exit codes and stderr contract for CI/scripts

## Suggested Improvements

- [ ] Document exit codes and machine-readable output for each subcommand
- [ ] Keep subcommand list in sync with `sruja --help`

## Related Decisions

- ADR001: Rust for Core and CLI

## Code Locations

- `crates/sruja-cli/` — CLI crate (main.rs, commands/, modules/)
- `Cargo.toml` workspace default member

---
*Last updated: 2025-03-16 by Sruja*
