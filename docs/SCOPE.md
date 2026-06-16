# Product scope

Sruja is an **AI coding agent and context engineering platform for AI-assisted SDLC**—not a diagramming product. We ship:

1. **Rust parser** (`sruja-language`) – core DSL parsing with pluggable schemas
2. **CLI** (`sruja-cli`) – validate, export, scan, quickstart, why, drift, critique, propose, context, focus, agent, and more
3. **WASM** (`sruja-wasm`) – browser/Node export and parsing
4. **mdBook** (`book/`) – **this is the website** (no separate Astro/React site)
5. **VS Code extension** (`extension/`) – edit, preview, language features (WASM-powered)
6. **Context engineering** – CLI: quickstart, drift, why, context, focus, context-score, context-graph, ingest; sruja-graph, sruja-scan, sruja-diff, sruja-intent
7. **Agentic memory** (`sruja-agent`) – persistent learning and guardrails for AI agents

Nothing else is in scope (no designer app, no storybook, no social-publish, no separate website app).

---

## What stays

| Item | Role |
|------|------|
| **crates/sruja-language** | Parser + AST + pluggable domain schemas |
| **crates/sruja-diagnostics** | Errors/locations (used by language, engine, CLI, WASM) |
| **crates/sruja-export** | DOT/Mermaid/Markdown/JSON/HTML/D3 export (CLI + WASM) |
| **crates/sruja-cli** | CLI (lint, validate, export, scan, quickstart, why, drift, intent, critique, propose, focus, context-score, context-graph, agent, compliance, ingest, federation) |
| **crates/sruja-wasm** | WASM bindings for browser/Node |
| **crates/sruja-engine** | Validation rules |
| **crates/sruja-graph** | Knowledge graph, centrality, coupling |
| **crates/sruja-graph-core** | Core graph types and primitives |
| **crates/sruja-scan** | Repo scanning (multi-language tree-sitter) |
| **crates/sruja-diff** | Drift detection (code vs. intent) + proposal system |
| **crates/sruja-intent** | Intent vs. reality comparison + adversarial critique engine |
| **crates/sruja-extract** | Source code extraction utilities |
| **crates/sruja-agent** | Agentic memory – persistent learning and guardrails |
| **book/** | mdBook source; build output = deployed website |
| **extension/** | VS Code extension (preview, language features, snippets) |
| **book/valid-examples/** | Canonical example `.sruja` files (rendered in the book). |

## What is out of scope

| Item | Notes |
|------|-------|
| **apps/website** | Website = mdBook build. |
| **apps/designer** | No standalone designer app. |
| **apps/storybook** | Not in scope. |
| **packages/** | No JS/TS monorepo packages. |

## mdBook as the website

- **Source:** `book/` (mdBook).
- **Build:** `mdbook build book` (or existing script); output e.g. `book/book/`.
- **Deploy:** Serve `book/book/` as the public site (GitHub Pages, Netlify, etc.).
- **WASM in book:** Keep `book/copy-wasm.sh` (or equivalent) so the book can use the same WASM for live snippets if desired.

## Cleanup done

References to website, designer, storybook, and social-publish have been removed from configs and docs to match the current Rust + mdBook scope.

## Next steps (optional)

1. Remove out-of-scope apps (if any remain) (website, designer, storybook, social-publish) if they exist in the repo.
2. Make mdBook the single “website” in CI/deploy (build book, deploy `book/book/`).
3. Optionally remove or slim `sruja-engine` if validation is not required for the minimal scope.
