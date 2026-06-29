# VS Code Extension

> VS Code language support and diagram preview.

## Purpose

The VS Code extension provides syntax highlighting, real-time validation (via LSP or bundled WASM/CLI), diagram preview, and component knowledge navigation (open doc from hover/definition). It is the primary IDE experience for Sruja.

## Responsibilities

- Register Sruja DSL as a language (syntax, LSP client or WASM)
- Run validation (Sruja: Run validation) and show diagnostics
- Preview diagrams (Mermaid/export)
- Resolve and open component knowledge files (`doc ".sruja/knowledge/..."`)
- Refresh repo context (Sruja: Refresh repo context) when available

## Dependencies

- **Internal**: Sruja_LSP (LSP client), Sruja_CLI (optional CLI path), WASM (optional bundled)
- **External**: VS Code API, TypeScript

## Known Risks

- Extension must work with both LSP and fallback CLI/WASM
- Knowledge doc paths are workspace-relative; multi-root handling

## Suggested Improvements

- [ ] Document extension commands and settings in book/docs
- [ ] Keep component knowledge (doc) feature discoverable in UI

## Related Decisions

- ADR002: WASM for browser and extension (extension can use WASM or CLI)

## Code Locations

- `extension/` — VS Code extension (TypeScript): package.json, src/ (extension.ts, providers, cliRunner)
- `book/src/docs/vscode-extension.md`

---
*Last updated: 2025-03-16 by Sruja*
