# Language Engine

> DSL parsing, AST, and language processing.

## Purpose

The language crate parses Sruja DSL source into an Abstract Syntax Tree (AST), provides lexer and printer, and supplies the AST to the validation engine and export. It defines the grammar, element kinds, and structure of `.sruja` files.

## Responsibilities

- Lex and parse Sruja DSL (Nom parser combinators)
- Build and represent AST (elements, relations, requirements, ADRs, policies, views, flows)
- Resolve imports (stdlib, relative)
- Support LSP/WASM by exposing parse and traversal APIs

## Dependencies

- **Internal**: Sruja_Stdlib (element kinds), Sruja_Engine (validation of AST), Sruja_Export (consumes AST)
- **External**: Nom, sruja-diagnostics for error reporting

## Known Risks

- Grammar changes must stay backward compatible or versioned
- Parser performance for very large files

## Suggested Improvements

- [ ] Keep grammar and AST in sync with language spec (book/src/reference/language-spec.md, docs/LANGUAGE_SPECIFICATION.md)
- [ ] Document AST node types for consumers (engine, export, LSP)

## Related Decisions

- ADR001: Rust for Core and CLI (Nom for parsing)

## Code Locations

- `crates/sruja-language/` — Parser, AST, lexer, printer, traversal
- `book/src/reference/language-spec.md`, `docs/LANGUAGE_SPECIFICATION.md`

---
*Last updated: 2025-03-16 by Sruja*
