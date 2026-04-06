# Sruja Project Context

Sruja is an architecture-as-code platform. It discovers architecture from codebases, validates it against declarations, and detects drift.

## Build and Check Commands
- `make check` — Build, test, and lint everything (fmt + clippy)
- `make build` — Build Rust libraries
- `make test` — Run all Rust tests
- `make lint` — Run clippy

## Sruja Specifics
- `.sruja` files define architecture.
- `sruja lint <file>` — Validate architecture DSL.
- `sruja daily -r .` — Run drift analysis and update AI context.

## Agent Guidelines
Refer to `AGENTS.md` for detailed coding standards, directory structure, and workflow requirements. You MUST strictly follow the instructions in that file.

## Developer Note
Sruja uses a mix of Rust (CLI/Core) and TypeScript (VS Code Extension). The extension relies on WASM built from Rust crates.
