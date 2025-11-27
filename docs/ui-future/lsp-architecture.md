# LSP Architecture (Future)

Language Server Protocol (LSP) architecture for future editor integration.

**Status**: Planned for future UI phase  
**Current Focus**: Go CLI tool (see [Main Documentation](../README.md))

## Overview

**Future**: A **Language Server Protocol (LSP)** server for the DSL, providing professional IDE features in web-based editors.

## Architecture Flow

```
Frontend (Next.js + Monaco + React Flow)
        |
        | WebSocket (JSON-RPC)
        v
Architecture LSP Server (Go)
        |
        | DSL parsing, semantics, validation
        v
DSL Parser (Go, participle)
        |
        v
Zod Model (Architecture v2)
```

## Go LSP Server

### Transport and Protocol

- JSON-RPC 2.0 over stdio or TCP.
- Implements core `textDocument/*` and `workspace/*` methods.

### Handlers

- `initialize`: advertise capabilities for diagnostics, completion, hover, formatting.
- `textDocument/didOpen` and `didChange`: parse with Go parser, run validation, publish diagnostics.
- Diagnostics: map participle errors (line/col) to LSP `Diagnostic` ranges with severity.
- `textDocument/completion`: keyword and symbol suggestions from AST and model index.
- `textDocument/hover`: node and edge details sourced from model/metadata.
- `textDocument/formatting`: serializer-driven formatting for round-trip stability.

### Data Flow (Go)

```
LSP Request
  -> Parser (Go, participle)
  -> Model Build (Go)
  -> Validation Engine (Go)
  -> Diagnostics/Completions/Hover
  -> LSP Response
```

### Integration

- Uses the same Go AST and Validation Engine as the CLI.
- Shares provider-based file access for multi-file projects.

### Project Awareness (index.json)

- Generate `.architecture/index.json` on open/save:
  - Node IDs → file locations (line/col)
  - ADRs and journey links
  - Type index
- Use index.json to power completion, go-to-symbol (future), and fast diagnostics.

## LSP Features in MVP

| Feature                                | MVP Status | Priority | Notes                           |
| -------------------------------------- | ---------- | -------- | ------------------------------- |
| Diagnostics (syntax + semantic errors) | ✅ YES      | Critical | Required for bidirectional sync |
| Autocomplete (keywords + node names)   | ✅ YES      | High     | Makes editor usable             |
| Hover (basic info about nodes)         | ✅ YES      | Medium   | Easy to add, high value         |
| Formatting                             | 🌙 Optional | Low      | Easy if serializer exists       |
| Go to Definition                       | ❌ Later    | -        | Not needed in MVP               |
| Rename Symbol                          | ❌ Later    | -        | Harder, can wait                |
| Code Actions                           | ❌ Later    | -        | Future value                    |
| Cross-module indexing                  | ❌ Later    | -        | Only after modules exist        |

## Why LSP in MVP?

- ✅ **Much better dev experience** - Professional IDE features
- ✅ **10× fewer bugs** - Catch errors before they propagate
- ✅ **Better DSL evolution** - Easier to extend grammar
- ✅ **Future-proof foundation** - Ready for multi-file, AI/MCP
- ✅ **Impressive MVP** - Stands out in demos
- ✅ **Text is source of truth** - LSP validates correctness

## Implementation Complexity

| Task                  | Difficulty | Time Estimate |
| --------------------- | ---------- | ------------- |
| LSP server bootstrap  | Easy       | 0.5–1 day     |
| WebSocket bridge      | Easy       | 0.5 day       |
| Parse + diagnostics   | Medium     | 1–2 days      |
| Completion            | Medium     | 2–3 days      |
| Hover                 | Easy       | 0.5–1 day     |
| Integrate with Monaco | Medium     | 1 day         |

**Total: ~1.5 weeks** solid implementation.

## Timeline Impact

- **Without LSP MVP**: 6–8 weeks
- **With LSP MVP**: 8–10 weeks (+2 weeks)
- **Benefits**: Professional IDE features, better DX, fewer bugs, future-proof

## Conclusion

**Including LSP in MVP is the right strategic call** because:
- Text is the source of truth for the DSL
- LSP ensures text correctness
- Professional IDE experience from day one
- Foundation for future features
- Manageable implementation (~1.5 weeks)
- High ROI (10× better DX)

**The 2-week extension is absolutely worth it.**

---

[← Back to UI & Future Features](./README.md) | [Main Documentation](../README.md)
