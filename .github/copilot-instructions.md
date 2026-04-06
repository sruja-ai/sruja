# GitHub Copilot Instructions for Sruja

Refer to the primary instructions in `AGENTS.md`.

## Core Commands
- `make build` — Build the platform
- `make test` — Run workspace tests
- `make lint` — Run clippy/format checks
- `make daily` — Sync context and check drift

## Architecture-as-Code
- `.sruja` files are the source of truth for architecture.
- Use `sruja-architecture` skill via the CLI for discovery andAuthoring.
- Always lint `.sruja` files before committing.
