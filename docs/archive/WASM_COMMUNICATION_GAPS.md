# WASM Communication Gaps Review

## Overview
This document reviews the coding gaps in the communication between the designer app (`apps/designer/`) and the Go backend via WASM (`cmd/wasm/`).

## Executive Summary

The designer app and Go backend communicate exclusively through WASM, but there are several gaps:
1. **Missing validation check** for `sruja_dsl_to_likec4` function in `wasmAdapter.ts`
2. **Unused LSP functions** - Go backend exports 9 LSP functions that the designer app doesn't use
3. **Missing score function** - Go exports `sruja_score` but it's not exposed in TypeScript WASM API
4. **Result format inconsistency** - Different return formats between functions
5. **No LSP integration** - Designer uses Monaco editor but doesn't leverage WASM LSP functions

---

## 1. Missing Validation Check for `sruja_dsl_to_likec4`

### Location
- File: `packages/shared/src/web/wasmAdapter.ts`
- Line: 177

### Issue
The `wasmAdapter.ts` file validates that core functions exist (parseFn, jsonToDslFn, mermaidFn, markdownFn) but doesn't check for `likec4Fn` even though:
- It's retrieved at line 166: `const likec4Fn = (window as any).sruja_dsl_to_likec4`
- It's used in the API at line 246
- The designer app heavily uses `convertDslToLikeC4` function

### Impact
If `sruja_dsl_to_likec4` fails to load, the error will only surface at runtime when the function is called, not during initialization.

### Recommendation
Add `likec4Fn` to the validation check:

```typescript
// Require core functions
if (!parseFn || !jsonToDslFn || !mermaidFn || !markdownFn || !likec4Fn) {
  const missing = []
  if (!parseFn) missing.push('sruja_parse_dsl')
  if (!jsonToDslFn) missing.push('sruja_json_to_dsl')
  if (!mermaidFn) missing.push('sruja_dsl_to_mermaid')
  if (!markdownFn) missing.push('sruja_dsl_to_markdown')
  if (!likec4Fn) missing.push('sruja_dsl_to_likec4') // ADD THIS
  // ... rest of error handling
}
```

---

## 2. Unused LSP Functions

### Location
- Go exports: `cmd/wasm/main.go` lines 28-36
- TypeScript LSP shim: `packages/shared/src/lsp/wasmLspShim.ts`
- Designer usage: None

### Exported but Unused Functions

The Go backend exports these LSP functions:
1. `sruja_get_diagnostics` - Get diagnostics (errors/warnings) ✅ Implemented
2. `sruja_get_symbols` - Get all symbols (identifiers) ✅ Implemented
3. `sruja_hover` - Hover information ✅ Implemented
4. `sruja_completion` - Completion suggestions ✅ Implemented
5. `sruja_go_to_definition` - Go to definition ✅ Implemented
6. `sruja_find_references` - Find all references ✅ Implemented
7. `sruja_rename` - Rename symbol and references ✅ Implemented
8. `sruja_format` - Format DSL text ✅ Implemented
9. `sruja_score` - Calculate architecture score ⚠️ Uses different return format

### Issue
- The designer app uses `SrujaMonacoEditor` component (`apps/designer/src/components/Panels/DSLPanel.tsx`)
- There's a WASM LSP shim available (`packages/shared/src/lsp/wasmLspShim.ts`)
- But the designer app doesn't initialize or use any LSP features
- This means users don't get:
  - Real-time diagnostics/errors in the editor
  - Hover information
  - Auto-completion
  - Go to definition
  - Find references
  - Format on save/type

### Impact
- Poor developer experience in DSL editor
- No inline error highlighting
- No code intelligence features
- Manual formatting required

### Recommendation
1. Initialize WASM LSP in `DSLPanel.tsx`:
```typescript
import { initializeMonacoWasmLsp, createWasmLspApi } from '@sruja/shared'
import * as monaco from 'monaco-editor'

// After editor creation
useEffect(() => {
  const editor = monacoEditorRef.current
  if (editor) {
    const wasmLspApi = createWasmLspApi()
    initializeMonacoWasmLsp(monaco, editor, wasmLspApi)
  }
}, [monacoEditorRef])
```

2. Alternatively, integrate LSP into `SrujaMonacoEditor` component in the shared UI package.

---

## 3. Missing Score Function in TypeScript API

### Location
- Go export: `cmd/wasm/main.go` line 36
- Go implementation: `cmd/wasm/lsp_handlers.go` lines 383-419
- TypeScript API: `packages/shared/src/web/wasmAdapter.ts` - **NOT EXPOSED**

### Issue
- Go backend exports `sruja_score` function
- The function calculates architecture quality score (0-100)
- The designer app has its own validation scoring (`apps/designer/src/utils/architectureValidator.ts`)
- But the Go backend's `sruja_score` uses the actual engine scorer (`engine.NewScorer()`)
- The TypeScript `WasmApi` interface doesn't include a `score` method

### Impact
- Two different scoring systems:
  - TypeScript: `architectureValidator.ts` - basic validation
  - Go: `engine.NewScorer()` - full architecture scoring
- Inconsistency between validation shown in UI vs backend scoring
- Missing access to comprehensive scoring from WASM

### Current Usage
The designer app uses `useValidation` hook which calls `validateArchitecture` from `architectureValidator.ts`, not the WASM score function.

### Recommendation
1. Add `score` to `WasmApi` interface:
```typescript
export type WasmApi = {
  parseDslToJson: (dsl: string, filename?: string) => Promise<string>
  printJsonToDsl: (json: string) => Promise<string>
  dslToMermaid: (dsl: string) => Promise<string>
  dslToMarkdown: (dsl: string) => Promise<string>
  dslToLikeC4: (dsl: string, filename?: string) => Promise<string>
  score: (dsl: string) => Promise<object> // ADD THIS
}
```

2. Implement in `wasmAdapter.ts`:
```typescript
score: async (dsl: string) => {
  try {
    const scoreFn = (window as any).sruja_score
    if (!scoreFn) throw new Error('sruja_score not available')
    const r = scoreFn(dsl)
    if (!r || !r.ok) {
      throw new Error(r?.error || 'score calculation failed')
    }
    return JSON.parse(r.json || r.data)
  } catch (error) {
    logger.error('WASM score exception', { error: error instanceof Error ? error.message : String(error) })
    throw error
  }
}
```

3. Update validation hook to use WASM score or compare both scores.

---

## 4. Result Format Inconsistency

### Location
- Go: `cmd/wasm/results.go`
- TypeScript: `packages/shared/src/web/wasmAdapter.ts`

### Issue
Two different result formats:

1. **`result()` function** (used by parse, export functions):
   - Returns: `{ ok: bool, json?: string, dsl?: string, data?: string, error?: string }`
   - All success fields (`json`, `dsl`, `data`) are set to the same value

2. **`lspResult()` function** (used by LSP functions):
   - Returns: `{ ok: bool, data?: any, error?: string }`
   - Only has `data` field

### Impact
TypeScript code needs to check different fields:
- Parse functions: `r.json`
- Export functions: `r.data`
- LSP functions: `r.data`

This is confusing and error-prone.

### Current Handling
- `parseDslToJson`: Uses `r.json` ✓
- `printJsonToDsl`: Uses `r.dsl` ✓
- `dslToMermaid/Markdown/LikeC4`: Uses `r.data` ✓
- LSP functions: Use `r.data` ✓

### Recommendation
Standardize on a single format. Since `result()` already sets all fields (`json`, `dsl`, `data`), the TypeScript side can always use `r.data` for consistency. Alternatively, update Go to use consistent field names.

---

## 5. Synchronous vs Asynchronous Call Mismatch

### Location
- Go: All WASM functions are synchronous
- TypeScript: All WASM API methods are async

### Issue
- Go WASM functions return immediately (synchronous JS functions)
- TypeScript wraps them in async functions
- The async wrapper adds no value since WASM calls block the main thread anyway

### Impact
- Unnecessary Promise overhead
- Confusing API (looks async but blocks)
- Can't leverage true async operations

### Current Pattern
```typescript
parseDslToJson: async (dsl: string, filename?: string) => {
  const r = parseFn(dsl, file) // Synchronous call
  // ...
}
```

### Recommendation
Either:
1. Keep as-is (convenient for future async support)
2. Make synchronous and document as blocking calls
3. Move WASM to Web Worker for true async support

---

## 6. Error Handling Gaps

### Location
- `packages/shared/src/web/wasmAdapter.ts`
- Various error scenarios not handled

### Issues

1. **No timeout for WASM loading**:
   - Current: Waits up to 150 retries (7.5 seconds)
   - If WASM fails to load, user waits unnecessarily long

2. **Silent failures in LSP shim**:
   - `packages/shared/src/lsp/wasmLspShim.ts` catches errors but only logs warnings
   - Functions return empty arrays/null instead of propagating errors

3. **No retry mechanism**:
   - If WASM function call fails, no retry
   - Network failures loading WASM file are not retried

### Recommendation
1. Add timeout with user-friendly error:
```typescript
const timeout = new Promise((_, reject) => 
  setTimeout(() => reject(new Error('WASM load timeout')), 10000)
)
await Promise.race([initPromise, timeout])
```

2. Add retry mechanism for WASM loading
3. Propagate LSP errors to caller with proper error types

---

## 7. Missing Filename Context

### Location
- Multiple places where filename could improve error messages

### Issue
- Some functions accept `filename` parameter but don't consistently use it
- Error messages often show generic "input.sruja" instead of actual filename
- Designer app doesn't always pass filename when available

### Impact
- Poor error messages
- Harder to debug multi-file scenarios (future)

### Recommendation
1. Always pass filename from designer app when available
2. Use filename in all error messages from Go backend
3. Include filename in diagnostic locations

---

## 8. No Type Safety for WASM Functions

### Location
- `packages/shared/src/web/wasmAdapter.ts`

### Issue
- All WASM functions accessed via `(window as any)[name]`
- No TypeScript types for Go WASM function signatures
- No compile-time validation of function calls

### Impact
- Runtime errors if function signature changes
- No IDE autocomplete
- Hard to refactor

### Recommendation
Create TypeScript types for WASM function signatures:
```typescript
interface WasmFunctionResult {
  ok: boolean
  json?: string
  dsl?: string
  data?: string | null
  error?: string
}

type WasmParseDsl = (dsl: string, filename?: string) => WasmFunctionResult
type WasmDslToLikeC4 = (dsl: string, filename?: string) => WasmFunctionResult
// etc.
```

---

## Summary of Recommendations

### High Priority
1. ✅ Add `likec4Fn` validation check in `wasmAdapter.ts`
2. ✅ Expose `score` function in TypeScript WASM API
3. ✅ Integrate LSP features into designer Monaco editor

### Medium Priority
4. Standardize result format between `result()` and `lspResult()`
5. Improve error handling with timeouts and retries
6. Always pass filename for better error messages

### Low Priority
7. Add TypeScript types for WASM functions
8. Consider Web Worker for true async WASM execution

---

## Files to Modify

1. `packages/shared/src/web/wasmAdapter.ts`
   - Add `likec4Fn` validation
   - Add `score` method to API
   - Improve error handling

2. `apps/designer/src/components/Panels/DSLPanel.tsx`
   - Initialize WASM LSP for Monaco editor

3. `packages/shared/src/web/wasmAdapter.ts`
   - Add timeout/retry for WASM loading

4. `cmd/wasm/results.go` (optional)
   - Standardize result format

5. `apps/designer/src/hooks/useValidation.ts` (optional)
   - Consider using WASM score function
