# VS Code Extension Migration Strategy

## Current Dependencies

### 1. LSP Server
- **Uses**: `sruja lsp` CLI subprocess (line 89 in `extension.ts`)
- **Impact if moved**: ⚠️ **BREAKING** - Would need major rewrite

### 2. Markdown Preview
- **Uses**: `sruja export markdown <file>` CLI command (line 56 in `previewProvider.ts`)
- **Impact if moved**: ✅ **IMPROVEMENT** - Can use TypeScript directly

### 3. Mermaid Export
- **Uses**: Not used by extension
- **Impact if moved**: ✅ **NO IMPACT**

## Recommended Strategy

### ✅ Move Markdown to TypeScript (Improves Extension)

**Current**:
```typescript
// previewProvider.ts
execFile(srujaPath, ['export', 'markdown', absolutePath], ...)
```

**After Migration**:
```typescript
// previewProvider.ts
import { parseDslToJson } from '@sruja/shared/web/wasmAdapter';
import { exportToMarkdown } from '@sruja/shared/export/markdown';

// Parse DSL to JSON (using WASM or CLI fallback)
const json = await parseDslToJson(dsl);
// Export to markdown (TypeScript - no CLI needed)
const markdown = exportToMarkdown(JSON.parse(json));
```

**Benefits**:
- ✅ No CLI dependency for preview
- ✅ Faster (no subprocess)
- ✅ Works even if CLI not installed
- ✅ Better error handling

### ⚠️ Keep LSP in Go (Don't Move)

**Why**:
- Extension uses `sruja lsp` subprocess
- Moving to TS would require:
  - Node.js LSP server
  - Or in-process LSP
  - Major rewrite

**Recommendation**: Keep `sruja lsp` in Go

### ✅ Move LSP Helpers to TypeScript (Optional Enhancement)

**Keep in Go**:
- Core LSP server (`sruja lsp`)
- Parser
- Validator
- Diagnostics

**Move to TypeScript** (optional):
- Symbol extraction (can work on JSON)
- Hover info (can work on JSON)
- Completion (can work on JSON)

**Impact**: Minimal - Extension still uses Go LSP, helpers are optional

## Implementation Plan

### Phase 1: Markdown Preview (High Priority)

**Step 1**: Create TypeScript markdown exporter
```typescript
// packages/shared/src/export/markdown.ts
export function exportToMarkdown(arch: ArchitectureJSON): string {
  // Port Go markdown export logic
}
```

**Step 2**: Update preview provider
```typescript
// apps/vscode-extension/src/previewProvider.ts
import { exportToMarkdown } from '@sruja/shared/export/markdown';
import { initWasm, getWasmApi } from '@sruja/shared/web/wasmAdapter';

// In provideTextDocumentContent:
const dsl = fs.readFileSync(absolutePath, 'utf-8');
const wasmApi = await initWasm();
const json = await wasmApi.parseDslToJson(dsl);
const markdown = exportToMarkdown(JSON.parse(json));
resolve(markdown);
```

**Step 3**: Add WASM support to extension
- Bundle WASM files with extension
- Or use CDN/remote WASM
- Fallback to CLI if WASM not available

**Benefits**:
- ✅ Preview works without CLI
- ✅ Faster preview
- ✅ Better UX

### Phase 2: LSP Helpers (Optional)

**Keep**: Core LSP in Go (`sruja lsp`)

**Add**: TypeScript helpers for enhanced features
- Symbols from JSON
- Hover from JSON
- Completion suggestions

**Impact**: Extension continues to work, optional enhancements available

## Migration Impact Summary

| Component | Current | After Migration | Extension Impact |
|-----------|---------|-----------------|------------------|
| **Markdown Export** | CLI command | TypeScript function | ✅ **IMPROVES** - No CLI needed |
| **Mermaid Export** | Not used | TypeScript function | ✅ **NO IMPACT** |
| **LSP Core** | CLI: `sruja lsp` | CLI: `sruja lsp` (unchanged) | ✅ **NO IMPACT** |
| **LSP Helpers** | Go (via LSP) | TypeScript (optional) | ⚠️ **MINOR** - Optional enhancement |

## Critical Considerations

### 1. WASM in VS Code Extension

**Option A**: Bundle WASM with extension
- Include WASM files in extension package
- Load WASM in extension process
- **Pros**: Works offline, no network needed
- **Cons**: Larger extension size

**Option B**: Use CLI as fallback
- Try WASM first, fallback to CLI
- **Pros**: Works even if WASM fails
- **Cons**: Still needs CLI

**Option C**: CLI only (current)
- Keep using CLI for now
- Move to WASM later
- **Pros**: No changes needed
- **Cons**: Still needs CLI

### 2. LSP Server

**MUST KEEP IN GO** because:
- Extension uses stdio transport to subprocess
- Moving to TS would require Node.js subprocess
- Major rewrite needed
- Not worth the effort

**Recommendation**: Keep `sruja lsp` in Go, move only helpers to TS

## Final Recommendation

### ✅ Do Move: Markdown Export
- **Impact**: Positive - Extension improves
- **Effort**: Medium - Create TS exporter, update preview provider
- **Risk**: Low - Can keep CLI as fallback

### ⚠️ Consider: LSP Helpers
- **Impact**: Neutral - Optional enhancements
- **Effort**: Medium - Create TS helpers
- **Risk**: Low - Core LSP stays in Go

### ❌ Don't Move: LSP Core
- **Impact**: Negative - Major rewrite
- **Effort**: High - Complete LSP rewrite
- **Risk**: High - Breaking changes

## Migration Checklist

### Markdown Preview
- [ ] Create `packages/shared/src/export/markdown.ts`
- [ ] Port markdown export logic from Go
- [ ] Add WASM support to extension (or use CLI fallback)
- [ ] Update `previewProvider.ts` to use TS exporter
- [ ] Test preview functionality
- [ ] Remove CLI dependency (optional)

### LSP (Keep in Go)
- [x] Keep `sruja lsp` command in Go
- [ ] (Optional) Create TS helpers for symbols/hover
- [ ] (Optional) Update extension to use TS helpers

## Answer to Your Question

**Will moving LSP, markdown, and mermaid affect VS Code extension?**

### Markdown: ✅ **IMPROVES** Extension
- Can remove CLI dependency
- Faster preview
- Better UX

### Mermaid: ✅ **NO IMPACT**
- Not used by extension

### LSP: ⚠️ **BREAKS** if moved entirely
- **Solution**: Keep LSP core in Go
- Move only helpers to TS (optional)

**Best Approach**: Move markdown to TS, keep LSP in Go, move helpers optionally.
