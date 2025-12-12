# Go to TypeScript Migration Plan

## Goal
Reduce Go codebase size by moving non-essential functionality to TypeScript, keeping only core functionality in Go.

## Core Principle
**Keep in Go**: Parser, AST, Core Validation  
**Move to TypeScript**: Exporters, Formatting, UI-related code

## Current State Analysis

### What's in Go (cmd/wasm/main.go)
1. ✅ **Parser** (`parseDsl`) - MUST STAY
2. ✅ **JSON Converter** (`jsonToDsl`) - MUST STAY (core)
3. ❌ **Markdown Export** (`dslToMarkdown`) - CAN MOVE
4. ❌ **Mermaid Export** (`dslToMermaid`) - CAN MOVE
5. ❌ **LSP Functions** (diagnostics, symbols, hover, completion) - PARTIAL MOVE
6. ❌ **Formatting** (`format`) - CAN MOVE
7. ❌ **Scoring** (`score`) - CAN MOVE

### What's in TypeScript (packages/)
- Already has: WASM adapter, viewer components
- Missing: Exporters, formatters

## Migration Strategy

### Phase 1: Move Exporters to TypeScript (High Impact)

#### 1.1 Markdown Export → TypeScript
**Current**: `pkg/export/markdown` (~3,176 lines)
**Move to**: `packages/shared/src/export/markdown.ts`

**What to move**:
- Template rendering
- Mermaid diagram embedding
- Section generation (systems, containers, etc.)
- TOC generation
- All helper functions

**Keep in Go**:
- Nothing (entire package can move)

**Benefits**:
- ~500KB-1MB reduction in WASM
- Native TypeScript (no WASM needed for exports)
- Faster development (no compilation)

**Implementation**:
```typescript
// packages/shared/src/export/markdown.ts
export function exportToMarkdown(arch: ArchitectureJSON): string {
  // Generate markdown from JSON
  // Embed mermaid diagrams
  // Generate TOC
}
```

**WASM API Change**:
```go
// Remove: dslToMarkdown
// Keep: parseDsl (returns JSON)
// TS calls: parseDsl → exportToMarkdown(JSON)
```

#### 1.2 Mermaid Export → TypeScript
**Current**: `pkg/export/mermaid` (~1,417 lines)
**Move to**: `packages/shared/src/export/mermaid.ts`

**What to move**:
- Diagram generation
- Node/edge rendering
- Style generation
- All generators

**Keep in Go**:
- Nothing (entire package can move)

**Benefits**:
- ~300-500KB reduction in WASM
- Native TypeScript
- Better integration with frontend

**Implementation**:
```typescript
// packages/shared/src/export/mermaid.ts
export function exportToMermaid(arch: ArchitectureJSON, options?: MermaidOptions): string {
  // Generate mermaid diagram from JSON
}
```

### Phase 2: Move Formatting to TypeScript

#### 2.1 Code Formatter → TypeScript
**Current**: `pkg/language/printer.go` (AST → DSL)
**Move to**: `packages/shared/src/format/formatter.ts`

**What to move**:
- AST to DSL formatting
- Indentation logic
- Spacing rules

**Keep in Go**:
- Parser (DSL → AST) - MUST STAY

**Benefits**:
- ~100-200KB reduction
- Formatting can work on JSON (no WASM needed)
- Faster (native TS)

**Implementation**:
```typescript
// packages/shared/src/format/formatter.ts
export function format(arch: ArchitectureJSON, options?: FormatOptions): string {
  // Convert JSON back to formatted DSL
  // Handle indentation, spacing
}
```

**WASM API Change**:
```go
// Remove: format function
// TS calls: parseDsl → format(JSON)
```

### Phase 3: Move Scoring to TypeScript

#### 3.1 Architecture Scoring → TypeScript
**Current**: `pkg/engine/scorer.go`
**Move to**: `packages/shared/src/score/scorer.ts`

**What to move**:
- Score calculation
- Grade assignment
- All scoring logic

**Keep in Go**:
- Validation rules (needed for scoring)

**Benefits**:
- ~200-300KB reduction
- Scoring can work on JSON

**Implementation**:
```typescript
// packages/shared/src/score/scorer.ts
export function calculateScore(arch: ArchitectureJSON): ScoreCard {
  // Calculate architecture score from JSON
}
```

### Phase 4: Simplify LSP Functions

#### 4.1 Move LSP Helpers to TypeScript
**Current**: `cmd/wasm/main.go` (getSymbols, hover, completion)
**Move to**: `packages/shared/src/lsp/helpers.ts`

**What to move**:
- Symbol extraction (from JSON)
- Hover info generation
- Completion suggestions
- Definition finding

**Keep in Go**:
- Diagnostics (needs parser/validator)
- Core parsing

**Benefits**:
- ~200-300KB reduction
- LSP features work on JSON

**Implementation**:
```typescript
// packages/shared/src/lsp/helpers.ts
export function extractSymbols(arch: ArchitectureJSON): Symbol[] {
  // Extract symbols from JSON
}

export function findHoverInfo(arch: ArchitectureJSON, line: number, col: number): HoverInfo {
  // Find hover info from JSON
}
```

## Minimal Go WASM API

After migration, Go WASM only needs:

```go
// Core parsing (MUST STAY)
sruja_parse_dsl(dsl: string) → { ok: bool, json: string, error: string }

// JSON to DSL (MUST STAY - needed for round-trip)
sruja_json_to_dsl(json: string) → { ok: bool, dsl: string, error: string }

// Diagnostics (MUST STAY - needs validator)
sruja_get_diagnostics(dsl: string) → { ok: bool, data: Diagnostic[], error: string }
```

**Removed from Go**:
- `dslToMarkdown` → Move to TS
- `dslToMermaid` → Move to TS
- `format` → Move to TS
- `score` → Move to TS
- `getSymbols` → Move to TS (work on JSON)
- `hover` → Move to TS (work on JSON)
- `completion` → Move to TS
- `goToDefinition` → Move to TS (work on JSON)

## Size Reduction Estimate

| Component | Current Size | After Move | Reduction |
|-----------|--------------|------------|-----------|
| Markdown Export | ~500KB-1MB | 0 | -500KB-1MB |
| Mermaid Export | ~300-500KB | 0 | -300-500KB |
| Formatting | ~100-200KB | 0 | -100-200KB |
| Scoring | ~200-300KB | 0 | -200-300KB |
| LSP Helpers | ~200-300KB | 0 | -200-300KB |
| **Total** | **~1.3-2.3MB** | **0** | **-1.3-2.3MB** |

**New WASM Size**: 8MB → **5.7-6.7MB** (15-29% reduction)

With wasm-opt: **5.2-6.2MB**  
With compression: **~1.3-1.6MB** (down from ~2MB)

## Implementation Plan

### Step 1: Create TypeScript Exporters (Week 1)
1. Create `packages/shared/src/export/markdown.ts`
2. Port markdown export logic from Go
3. Test with existing examples
4. Update WASM adapter to use TS exporter

### Step 2: Create TypeScript Mermaid Exporter (Week 1)
1. Create `packages/shared/src/export/mermaid.ts`
2. Port mermaid generation logic
3. Test diagram generation
4. Update WASM adapter

### Step 3: Create TypeScript Formatter (Week 2)
1. Create `packages/shared/src/format/formatter.ts`
2. Port formatting logic from `printer.go`
3. Test formatting with examples
4. Update VS Code extension to use TS formatter

### Step 4: Move Scoring (Week 2)
1. Create `packages/shared/src/score/scorer.ts`
2. Port scoring logic
3. Test scoring accuracy
4. Update WASM adapter

### Step 5: Move LSP Helpers (Week 3)
1. Create `packages/shared/src/lsp/helpers.ts`
2. Port symbol extraction, hover, completion
3. Update VS Code extension
4. Test LSP features

### Step 6: Remove from Go (Week 3)
1. Remove exported functions from `cmd/wasm/main.go`
2. Remove packages: `pkg/export/markdown`, `pkg/export/mermaid`
3. Remove or simplify: `pkg/language/printer.go`
4. Remove: `pkg/engine/scorer.go` (or keep minimal version)
5. Update tests

## Benefits

### 1. Size Reduction
- **15-29% smaller WASM** (1.3-2.3MB reduction)
- Faster downloads
- Better performance

### 2. Development Speed
- **Faster iteration**: No Go compilation for exports
- **Better tooling**: TypeScript has better IDE support
- **Easier debugging**: Native browser debugging

### 3. Maintainability
- **Single language**: TypeScript for frontend, Go for core
- **Clear separation**: Core (Go) vs Presentation (TS)
- **Easier testing**: TypeScript tests run faster

### 4. Performance
- **Native TypeScript**: No WASM overhead for exports
- **Faster exports**: Direct JavaScript execution
- **Better caching**: TypeScript bundles can be cached separately

## Risks & Mitigations

### Risk 1: JSON Round-trip Accuracy
**Issue**: JSON might lose some information  
**Mitigation**: 
- Ensure JSON schema captures all AST data
- Test round-trip: DSL → JSON → DSL → JSON
- Add validation tests

### Risk 2: Formatting Differences
**Issue**: TS formatter might produce different output  
**Mitigation**:
- Port exact formatting logic
- Compare outputs with Go version
- Use same test cases

### Risk 3: Export Quality
**Issue**: TS exports might differ from Go  
**Mitigation**:
- Port exact logic
- Use same templates
- Compare outputs

## Testing Strategy

1. **Unit Tests**: Test each TS module independently
2. **Integration Tests**: Test DSL → JSON → Export flow
3. **Comparison Tests**: Compare TS output with Go output
4. **Regression Tests**: Ensure no functionality lost

## Migration Checklist

- [ ] Create TypeScript markdown exporter
- [ ] Create TypeScript mermaid exporter
- [ ] Create TypeScript formatter
- [ ] Create TypeScript scorer
- [ ] Create TypeScript LSP helpers
- [ ] Update WASM adapter
- [ ] Update VS Code extension
- [ ] Remove Go code
- [ ] Update tests
- [ ] Update documentation
- [ ] Measure size reduction
- [ ] Verify functionality

## What Stays in Go (Essential)

1. **Parser** (`pkg/language/parser.go`) - Core functionality
2. **AST** (`pkg/language/ast.go`) - Core data structures
3. **Validator** (`pkg/engine/validator.go`) - Core validation
4. **Validation Rules** (`pkg/engine/*_rule.go`) - Core rules
5. **JSON Converter** (`internal/converter`) - Round-trip support
6. **Diagnostics** (`pkg/diagnostics`) - Error reporting

## What Moves to TypeScript (Non-Essential)

1. **Markdown Export** - Presentation layer
2. **Mermaid Export** - Presentation layer
3. **Formatting** - Can work on JSON
4. **Scoring** - Can work on JSON
5. **LSP Helpers** - Can work on JSON
6. **UI Components** - Already in TS

## Final Architecture

```
Go (Core):
  - Parser (DSL → AST)
  - Validator (AST → Diagnostics)
  - JSON Converter (AST ↔ JSON)
  - WASM API: parseDsl, jsonToDsl, getDiagnostics

TypeScript (Presentation):
  - Markdown Export (JSON → Markdown)
  - Mermaid Export (JSON → Mermaid)
  - Formatter (JSON → DSL)
  - Scorer (JSON → Score)
  - LSP Helpers (JSON → Symbols/Hover/etc.)
  - UI Components
```

## Success Metrics

1. **Size**: WASM < 6MB (from 8MB)
2. **Functionality**: All features work
3. **Performance**: Exports faster (native TS)
4. **Maintainability**: Clear separation of concerns
