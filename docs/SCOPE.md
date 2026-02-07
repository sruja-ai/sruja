# Product scope

Sruja is an **architecture-as-code tool for the AI SDLC process**—not a diagramming product. We ship only:

1. **Rust parser** (`sruja-language`) – core DSL parsing
2. **CLI** (`sruja-cli`) – validate, export, run from terminal
3. **WASM** (`sruja-wasm`) – browser/Node export and parsing
4. **mdBook** (`book/`) – **this is the website** (no separate Astro/React site)
5. **VS Code extension** (`apps/vscode-extension`) – edit, preview, LSP

Nothing else is in scope (no designer app, no storybook, no social-publish, no separate website app).

---

## What stays

| Item | Role |
|------|------|
| **crates/sruja-language** | Parser + AST |
| **crates/sruja-diagnostics** | Errors/locations (used by language, LSP) |
| **crates/sruja-export** | DOT/Mermaid/Markdown/JSON export (CLI + WASM) |
| **crates/sruja-cli** | CLI (parse, validate, export) |
| **crates/sruja-wasm** | WASM bindings for browser/Node |
| **crates/sruja-lsp** | LSP server (used by VS Code extension) |
| **crates/sruja-engine** | Validation rules (if CLI/extension use them; otherwise can be removed or slimmed) |
| **book/** | mdBook source; build output = deployed website |
| **apps/vscode-extension** | VS Code extension (preview, LSP, snippets) |
| **packages/shared** | Minimal: only what the VS Code extension needs (Node WASM adapter, types, utils). No browser/React. |
| **packages/eslint-config** | Lint config for the extension (and any remaining TS). |
| **packages/tsconfig** | Base TS configs. |
| **examples/** | Example `.sruja` files (used by book and CLI). |

## What to remove or slim

| Item | Action |
|------|--------|
| **packages/ui** | Remove – no web app; mdBook is static, extension uses shared only. |
| **apps/website** | If it still exists – remove; website = mdBook build. |
| **apps/designer** | Remove if present. |
| **apps/storybook** | Remove if present. |
| **apps/social-publish** | Remove if present. |
| **packages/shared** | Slim: drop browser-only paths (e.g. `web/wasmAdapter.ts` if only extension uses Node adapter), PostHog/analytics if not used, heavy unused exports. Keep Node WASM adapter, LSP shim, types/utils used by extension. |
| **Root package.json workspaces** | Change to `["apps/vscode-extension", "packages/shared", "packages/eslint-config"]` (drop ui and any removed apps). |
| **Turbo / build scripts** | Adjust so `build` only builds shared + extension; no website/ui/storybook targets. |
| **sruja-engine** | Keep if CLI or extension run validation; else remove and have CLI only parse/export. |

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
