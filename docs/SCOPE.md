# Product scope

Sruja is an **architecture-as-code tool for the AI SDLC process**—not a diagramming product. We ship:

1. **Rust parser** (`sruja-language`) – core DSL parsing
2. **CLI** (`sruja-cli`) – validate, export, scan, quickstart, why, drift, analyze (structural/semantic/intent/runtime)
3. **WASM** (`sruja-wasm`) – browser/Node export and parsing
4. **mdBook** (`book/`) – **this is the website** (no separate Astro/React site)
5. **VS Code extension** (`extension/`) – edit, preview, LSP
6. **Context engineering** – CLI: quickstart, drift, why, analyze, context; sruja-graph, sruja-scan, sruja-diff, sruja-intent, sruja-report.

Nothing else is in scope (no designer app, no storybook, no social-publish, no separate website app).

---

## What stays

| Item | Role |
|------|------|
| **crates/sruja-language** | Parser + AST |
| **crates/sruja-diagnostics** | Errors/locations (used by language, LSP) |
| **crates/sruja-export** | DOT/Mermaid/Markdown/JSON export (CLI + WASM) |
| **crates/sruja-cli** | CLI (lint, validate, export, scan, quickstart, why, drift, complexity, semantic, analyze, intent, runtime) |
| **crates/sruja-wasm** | WASM bindings for browser/Node |
| **crates/sruja-lsp** | LSP server (used by VS Code extension) |
| **crates/sruja-engine** | Validation rules |
| **crates/sruja-graph** | Knowledge graph, centrality, coupling |
| **crates/sruja-scan** | Repo scanning (multi-language tree-sitter) |
| **crates/sruja-diff** | Drift detection (code vs. intent) |
| **crates/sruja-intent** | Intent vs. reality comparison |
| **crates/sruja-report** | Compliance and reporting |
| **crates/sruja-types** | Shared types |
| **book/** | mdBook source; build output = deployed website |
| **extension/** | VS Code extension (preview, LSP, snippets) |
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
