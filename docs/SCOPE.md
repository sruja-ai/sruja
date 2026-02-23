# Product scope

**Strategy:** For Architecture Intelligence direction, module decisions, and execution plan, see [architecture/AI_FIRST_MODULE_ANALYSIS_FINAL.md](../architecture/AI_FIRST_MODULE_ANALYSIS_FINAL.md).

Sruja is an **architecture-as-code tool for the AI SDLC process**—not a diagramming product. We ship:

1. **Rust parser** (`sruja-language`) – core DSL parsing
2. **CLI** (`sruja-cli`) – validate, export, scan, why, drift
3. **WASM** (`sruja-wasm`) – browser/Node export and parsing
4. **mdBook** (`book/`) – **this is the website** (no separate Astro/React site)
5. **VS Code extension** (`extension/`) – edit, preview, LSP
6. **Architecture intelligence** – sruja-app (desktop), sruja-chat, sruja-graph, sruja-extract, sruja-scan

Nothing else is in scope (no designer app, no storybook, no social-publish, no separate website app).

---

## What stays

| Item | Role |
|------|------|
| **crates/sruja-language** | Parser + AST |
| **crates/sruja-diagnostics** | Errors/locations (used by language, LSP) |
| **crates/sruja-export** | DOT/Mermaid/Markdown/JSON export (CLI + WASM) |
| **crates/sruja-cli** | CLI (parse, validate, export, scan, why, drift) |
| **crates/sruja-wasm** | WASM bindings for browser/Node |
| **crates/sruja-lsp** | LSP server (used by VS Code extension) |
| **crates/sruja-engine** | Validation rules |
| **crates/sruja-app** | Desktop app (Dioxus) — architecture collaboration |
| **crates/sruja-chat** | Chat, agents, extraction |
| **crates/sruja-graph** | Knowledge graph for decisions |
| **crates/sruja-extract** | LLM extraction |
| **crates/sruja-scan** | Repo scanning (npm, cargo) |
| **crates/sruja-mcp** | MCP server |
| **book/** | mdBook source; build output = deployed website |
| **extension/** | VS Code extension (preview, LSP, snippets) |
| **examples/** | Example `.sruja` files (used by book and CLI). |

## What is out of scope

| Item | Notes |
|------|-------|
| **packages/ui** | No web app; mdBook is static. |
| **apps/website** | Website = mdBook build. |
| **apps/designer** | No standalone designer app. |
| **apps/storybook** | Not in scope. |
| **sruja-engine** | Kept for CLI/extension validation. |

## mdBook as the website

- **Source:** `book/` (mdBook).
- **Build:** `mdbook build book` (or existing script); output e.g. `book/book/`.
- **Deploy:** Serve `book/book/` as the public site (GitHub Pages, Netlify, etc.).
- **WASM in book:** Keep `book/copy-wasm.sh` (or equivalent) so the book can use the same WASM for live snippets if desired.

## Cleanup done

References to ui, website, designer, storybook, social-publish removed from configs and docs. Turbo, release-please, size-limit, CI, Chromatic, social-publish workflow, PR template, book, and README updated. nextjs.json removed. Lockfile pruned via npm install. Optional next: slim packages/shared if extension-only desired.

## Next steps (optional)

1. Remove out-of-scope apps (if any remain) (website, designer, storybook, social-publish) if they exist in the repo.
2. Remove `packages/ui` and update root `package.json` workspaces.
3. Slim `packages/shared` to extension-only needs; remove or stub browser-only and analytics if unused.
4. Make mdBook the single “website” in CI/deploy (build book, deploy `book/book/`).
5. Optionally remove or slim `sruja-engine` if validation is not required for the minimal scope.
