# Sruja Architecture Documentation

This directory contains implementation-aligned architecture-as-code files for Sruja itself. These files are meant to describe the product as it exists today, especially its architecture-as-code and context-engineering workflows for AI-assisted development.

## What These Files Cover

- **Platform architecture**: Core crates, CLI, WASM, extension, and supporting context-engineering modules
- **Context engineering flow**: How reviewed truth, scanned evidence, editor features, and AI-facing outputs fit together
- **Development workflow**: How contributors build, validate, release, and deploy the project

## Files

### `sruja-platform.sruja`

Current product architecture for the Sruja platform, including:

- Rust language, export, validation, scan, graph, diff, and intent crates
- CLI workflows such as `quickstart`, `sync`, `status`, `review`, `context`, `publish`, and `compose`
- WASM-powered editor and docs experiences
- VS Code extension capabilities and AI-oriented tooling

### `context-pipeline.sruja`

Context-engineering architecture for AI-assisted work, including:

- Reviewed repo truth in `repo.sruja`
- Fresh evidence in `.sruja/context.json`
- Task-scoped context export for AI tools
- Editor flows such as refresh context, docs thread, and copied context packs
- Federation artifacts such as `repo.bundle.json` and `system.index.json`

### `sruja-development-workflow.sruja`

Current contributor and release workflow, including:

- Local Rust, extension, docs, and smoke-test loops
- GitHub Actions CI for Rust, WASM, `.sruja`, skills, security, and version consistency
- Release Please, CLI release assets, extension publishing, and mdBook deployments

### `domain-schema.md`

Explanation of the Domain Schema DSL and the transition to generalized Context Graphs.

## Usage

### Lint the architecture docs

```bash
sruja lint docs/architecture/sruja-platform.sruja
sruja lint docs/architecture/context-pipeline.sruja
sruja lint docs/architecture/sruja-development-workflow.sruja
```

### Export a document or diagram

```bash
sruja export markdown docs/architecture/sruja-platform.sruja
sruja export mermaid docs/architecture/context-pipeline.sruja
```

### Inspect the structure

```bash
sruja tree docs/architecture/context-pipeline.sruja
```

## Maintenance Rules

- Keep these files aligned with implemented commands, workflows, and artifacts
- Prefer current repo paths and technologies over older historical descriptions
- If a feature is still aspirational, keep it out of these files until it exists
- Update the relevant architecture file when CLI commands, extension capabilities, or release flows change
- Lint changed `.sruja` files before merging

## Relationship to Other Docs

- **[../SCOPE.md](../SCOPE.md)** defines what the product is and is not
- **[../FEDERATION.md](../FEDERATION.md)** explains multi-repo retrieval artifacts and loading order
- **[../GETTING_STARTED_SKILL.md](../GETTING_STARTED_SKILL.md)** explains how users experience the context-engineering workflow
- **[../adr/](../adr/)** records why key decisions were made

## Working Principle

These architecture files are living documentation for the current implementation, not generic examples and not future-state sketches.
