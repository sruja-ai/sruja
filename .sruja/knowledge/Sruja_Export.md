# Export System

> Exports architecture models to various formats.

## Purpose

The export crate transforms the AST into JSON, Markdown, Mermaid, DSL (round-trip), and context formats. It is used by CLI (`sruja export`), LSP (preview), and WASM (browser export).

## Responsibilities

- Export to JSON (primary machine format)
- Export to Mermaid (diagrams)
- Export to Markdown (documentation)
- DSL printer (round-trip, formatting)
- Context export for discovery/skill workflows

## Dependencies

- **Internal**: Sruja_Language (AST), Sruja_Engine (validation before export)
- **External**: None beyond std/serde

## Known Risks

- Format stability: JSON schema and Mermaid syntax may evolve
- Large model export performance

## Suggested Improvements

- [ ] Document export format versions and compatibility
- [ ] Keep DSL printer in sync with language parser

## Related Decisions

- ADR003: JSON as Primary Export Format

## Code Locations

- `crates/sruja-export/` — JSON, Mermaid, Markdown, DSL printer, context
- `crates/sruja-export/EXPORT_COVERAGE.md`

---
*Last updated: 2025-03-16 by Sruja*
