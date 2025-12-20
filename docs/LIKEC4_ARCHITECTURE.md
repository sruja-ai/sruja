# LikeC4 Integration Architecture

## Overview

Sruja uses LikeC4-compatible syntax and JSON format, but **keeps its own parser**. We generate LikeC4-compatible JSON and use `@likec4/diagram` React components for rendering.

## Architecture Flow

```
┌─────────────────────────────────────┐
│  .sruja file (LikeC4 syntax)       │
│  specification { ... }              │
│  model { ... }                      │
│  views { ... }                      │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  Sruja Go Parser                    │
│  (pkg/language/parser.go)           │
│  - Parses LikeC4 syntax             │
│  - Supports Sruja extensions        │
│  - We DON'T use LikeC4's parser     │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  Architecture AST                   │
│  (internal Go structures)           │
└──────────────┬──────────────────────┘
               │
               ├─────────────────────────┐
               │                         │
               ▼                         ▼
┌─────────────────────────┐  ┌──────────────────────────┐
│ LikeC4 JSON Exporter    │  │ Other Exporters          │
│ (pkg/export/json/)      │  │ - Mermaid                │
│                         │  │ - Markdown               │
│ Generates:              │  │ - etc.                   │
│ SrujaModelDump          │  └──────────────────────────┘
│ (LikeC4-compatible)     │
└──────────────┬──────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  LikeC4-Compatible JSON             │
│  {                                  │
│    specification: {...},            │
│    elements: {...},                 │
│    relations: [...],                │
│    views: {...},                    │
│    sruja: {...}  // extensions      │
│  }                                  │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  @likec4/diagram React Components   │
│  - LikeC4View                       │
│  - LikeC4ModelProvider              │
│  - Rendering only                   │
└─────────────────────────────────────┘
```

## Key Points

### 1. We Keep Our Parser ✅
- Sruja's Go parser (`pkg/language/parser.go`) handles LikeC4 syntax
- We **don't use** LikeC4's TypeScript/JavaScript parser
- Our parser supports Sruja extensions (requirements, policies, etc.)

### 2. We Generate LikeC4 JSON ✅
- Export to LikeC4-compatible format (`pkg/export/json/likec4_exporter.go`)
- JSON structure matches LikeC4's expected format
- Includes Sruja extensions in `sruja` field

### 3. We Use LikeC4 for Rendering Only ✅
- `@likec4/diagram` provides React components
- `@likec4/core` is a dependency (types/utilities)
- We **don't use** LikeC4's parsing/CLI tools

## Benefits

1. **Single Parser**: One parser to maintain (Go-based, fast)
2. **LikeC4 Compatibility**: Can use LikeC4's rendering ecosystem
3. **Sruja Extensions**: Governance features work seamlessly
4. **No Parser Duplication**: Don't need to maintain two parsers

## Dependencies

```json
{
  "@likec4/diagram": "^1.44.0",  // React components for rendering
  "@likec4/core": "^1.44.0"      // Types/utilities (dependency of diagram)
}
```

We **don't need**:
- `@likec4/core` for parsing (we use our own)
- LikeC4 CLI (we use our own export commands)

## Example

```go
// Our parser parses LikeC4 syntax
program, _, err := parser.Parse("file.sruja", dslContent)

// We convert to Architecture AST
arch := ConvertLikeC4ToArchitecture(program, "file.sruja")

// We export to LikeC4-compatible JSON
exporter := NewLikeC4Exporter()
json := exporter.Export(arch)

// Frontend uses @likec4/diagram to render JSON
<LikeC4ModelProvider model={json}>
  <LikeC4View viewId="index" />
</LikeC4ModelProvider>
```
