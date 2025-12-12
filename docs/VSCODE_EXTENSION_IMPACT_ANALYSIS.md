# VS Code Extension Impact Analysis

## Question: Will moving LSP, Markdown, and Mermaid to TypeScript affect the VS Code extension?

## Current VS Code Extension Architecture

### What the Extension Uses

1. **LSP Server**: Uses `sruja lsp` CLI subprocess
   - Location: `extension.ts` line 88
   - Command: `sruja lsp` (stdio transport)
   - **Impact**: Moving LSP to TS would require major changes

2. **Markdown Preview**: Uses `sruja export markdown` CLI command
   - Location: `previewProvider.ts` line 56
   - Command: `sruja export markdown <file>`
   - **Impact**: Moving markdown to TS would require changes

3. **No WASM Usage**: Extension doesn't use WASM at all
   - Uses CLI subprocesses only
   - **Impact**: Moving to TS doesn't directly affect extension

## Impact Analysis

### Scenario 1: Move Markdown/Mermaid Export to TypeScript

#### Current Flow
```
VS Code Extension → CLI: `sruja export markdown <file>` → Go markdown exporter → Markdown output
```

#### After Migration
```
VS Code Extension → CLI: `sruja export markdown <file>` → Go markdown exporter → Markdown output
```

**Wait, this won't work!** If we remove markdown export from Go CLI, the command won't exist.

#### Options

**Option A: Keep CLI Command, Use TypeScript Internally**
- Keep `sruja export markdown` CLI command
- CLI calls TypeScript via Node.js
- **Problem**: Requires Node.js runtime, defeats purpose

**Option B: Extension Uses TypeScript Directly**
- Extension calls TypeScript markdown exporter
- No CLI needed for preview
- **Better**: Native TypeScript, faster

**Option C: Hybrid - Keep CLI for Compatibility**
- Keep `sruja export markdown` in Go (minimal wrapper)
- Or create new TypeScript-based command
- **Trade-off**: Some duplication

### Scenario 2: Move LSP to TypeScript

#### Current Flow
```
VS Code Extension → CLI: `sruja lsp` → Go LSP server → Language features
```

#### After Migration
```
VS Code Extension → TypeScript LSP server → Language features
```

**Major Change Required!**

#### Impact

**Current**: Extension uses `LanguageClient` with stdio transport to Go subprocess

**After**: Would need to use TypeScript LSP server instead

**Options**:

**Option A: TypeScript LSP Server (Node.js)**
- Extension spawns Node.js process running TS LSP server
- Similar to current Go subprocess
- **Pros**: Native TypeScript, faster development
- **Cons**: Requires Node.js runtime

**Option B: In-Process TypeScript LSP**
- LSP server runs in extension process
- No subprocess needed
- **Pros**: Faster, no process overhead
- **Cons**: Extension process must handle LSP protocol

**Option C: Keep Go LSP, Move Helpers to TS**
- Keep core LSP in Go (parser, validator)
- Move helpers (symbols, hover, completion) to TS
- **Best**: Minimal changes, best of both worlds

## Recommended Approach

### For Markdown/Mermaid Preview

**Move to TypeScript in Extension** (Option B)

```typescript
// Instead of CLI call:
// execFile(srujaPath, ['export', 'markdown', file])

// Use TypeScript directly:
import { exportToMarkdown } from '@sruja/shared/export/markdown';
const json = await parseDslToJson(dsl); // Still use Go WASM for parsing
const markdown = exportToMarkdown(JSON.parse(json));
```

**Benefits**:
- ✅ No CLI dependency for preview
- ✅ Faster (no subprocess overhead)
- ✅ Better error handling
- ✅ Works offline (no CLI needed)

**Changes Required**:
1. Add TypeScript markdown exporter to `packages/shared`
2. Update `previewProvider.ts` to use TS exporter
3. Keep WASM for parsing (DSL → JSON)
4. Remove CLI dependency for preview

### For LSP

**Keep Core in Go, Move Helpers to TS** (Option C)

**Keep in Go**:
- Parser (DSL → AST)
- Validator (AST → Diagnostics)
- Core LSP server (`sruja lsp`)

**Move to TypeScript**:
- Symbol extraction (from JSON)
- Hover info (from JSON)
- Completion suggestions
- Go-to-definition (from JSON)

**Benefits**:
- ✅ Minimal changes to extension
- ✅ LSP server still works
- ✅ Helpers can work on JSON (no WASM needed)
- ✅ Faster development for helpers

**Changes Required**:
1. Keep `sruja lsp` command in Go
2. Move LSP helpers to TypeScript
3. Extension uses Go LSP for core features
4. Extension uses TS helpers for UI features

## Detailed Impact Matrix

| Feature | Current | After Migration | Extension Impact |
|---------|---------|-----------------|------------------|
| **Markdown Preview** | CLI: `sruja export markdown` | TS: `exportToMarkdown(JSON)` | ✅ **Improvement** - No CLI needed |
| **Mermaid Export** | CLI: `sruja export mermaid` | TS: `exportToMermaid(JSON)` | ✅ **No impact** - Not used by extension |
| **LSP Core** | CLI: `sruja lsp` | CLI: `sruja lsp` (unchanged) | ✅ **No impact** - Stays in Go |
| **LSP Helpers** | Go: symbols, hover, completion | TS: symbols, hover, completion | ⚠️ **Minor** - Update to use TS |
| **Parsing** | Go: Parser | Go: Parser (unchanged) | ✅ **No impact** - Stays in Go |

## Implementation Plan

### Phase 1: Markdown Preview (High Priority)

**Current Code** (`previewProvider.ts`):
```typescript
execFile(srujaPath, ['export', 'markdown', absolutePath], ...)
```

**New Code**:
```typescript
// Parse DSL to JSON using WASM (if available) or CLI
const json = await parseDslToJson(dsl);
// Export to markdown using TypeScript
const markdown = exportToMarkdown(JSON.parse(json));
```

**Benefits**:
- No CLI dependency
- Faster preview
- Better error handling
- Works even if CLI not installed

### Phase 2: LSP Helpers (Low Priority)

**Current**: LSP server provides all features via Go

**New**: 
- Core features (diagnostics, parsing) via Go LSP
- Helper features (symbols, hover) via TypeScript

**Impact**: Minimal - extension still uses Go LSP server

## Migration Checklist

### Markdown Preview
- [ ] Create TypeScript markdown exporter
- [ ] Update `previewProvider.ts` to use TS exporter
- [ ] Add WASM parsing support (or keep CLI for parsing)
- [ ] Test preview functionality
- [ ] Remove CLI dependency for preview

### LSP Helpers (Optional)
- [ ] Create TypeScript LSP helpers
- [ ] Update extension to use TS helpers where beneficial
- [ ] Keep Go LSP for core features
- [ ] Test LSP functionality

## Risk Assessment

### Low Risk ✅
- **Markdown Preview**: Moving to TS improves extension (no CLI needed)
- **Mermaid**: Not used by extension, no impact

### Medium Risk ⚠️
- **LSP Helpers**: Moving helpers to TS requires code changes but keeps core in Go

### High Risk ❌
- **LSP Core**: Moving entire LSP to TS would require major rewrite (NOT recommended)

## Recommendation

### ✅ Do Move: Markdown/Mermaid Export
- **Impact**: Positive - Extension improves (no CLI dependency)
- **Effort**: Medium - Need to create TS exporters
- **Risk**: Low - Can fallback to CLI if needed

### ⚠️ Consider: LSP Helpers
- **Impact**: Neutral - Extension still works, helpers faster
- **Effort**: Medium - Need to create TS helpers
- **Risk**: Low - Core LSP stays in Go

### ❌ Don't Move: LSP Core
- **Impact**: Negative - Major rewrite required
- **Effort**: High - Complete LSP rewrite
- **Risk**: High - Breaking changes

## Final Answer

**Moving markdown/mermaid to TypeScript will IMPROVE the VS Code extension** by:
1. Removing CLI dependency for preview
2. Making preview faster (no subprocess)
3. Better error handling
4. Works even if CLI not installed

**Moving LSP helpers to TypeScript has MINIMAL impact** because:
1. Core LSP stays in Go (no changes needed)
2. Helpers can be optional enhancements
3. Extension continues to work as-is

**Moving LSP core to TypeScript would BREAK the extension** and is NOT recommended.
