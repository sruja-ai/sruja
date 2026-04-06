# Windsurf Project Rules: Sruja Architecture Platform

## Context
Sruja is a platform for architecture-as-code. It bridges the gap between high-level architectural declarations and actual code using static analysis and drift detection.

## Tech Stack
- **Rust (Core)**: Fast performance, WASM compatibility.
- **TypeScript (Extension)**: VS Code integration via WASM.
- **mdBook (Documentation)**: Centralized documentation.

## Mandatory Coding Standards
- **General**: Follow `AGENTS.md` which is the central authority on coding style and practices.
- **Error Handling**: Use `Result` with `thiserror` (Rust) and `try/catch` with `vscode.window.showErrorMessage` (TypeScript).
- **Architecture Integrity**: Run `sruja lint repo.sruja` after every architecture modification.

## Primary Commands
- `make build` / `make test` — Compilation and verification.
- `make check` — Build + Test + Lint.
- `make daily` — Daily check for architectural drift.

Refer to `AGENTS.md` for all detailed requirements.
