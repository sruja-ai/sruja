# Mermaid Export Debugging Guide

## ✅ Confirmation: Using TypeScript Exporter

**The code IS using TypeScript, NOT WASM for mermaid export.**

### Verification
- ✅ `convertDslToMermaid()` calls `generateSystemDiagramForArch()` (TypeScript)
- ✅ Function is properly imported from `../export/mermaid`
- ✅ Function is exported from shared package

### Code Flow
```
CodeBlockActions.tsx
  ↓
convertDslToMermaid(dsl)  [from @sruja/shared]
  ↓
getWasmApi() → parseDslToJson()  [WASM - for parsing only]
  ↓
generateSystemDiagramForArch(archJson)  [TypeScript - for export]
  ↓
generateSystemDiagram(arch, config)  [TypeScript]
  ↓
Returns mermaid diagram string
```

---

## Why "Mermaid export not available" Error?

The error occurs when `convertDslToMermaid()` returns `null`. This can happen if:

1. **WASM not initialized** - `getWasmApi()` returns null
2. **DSL parsing failed** - `parseDslToJson()` throws or returns empty
3. **Invalid JSON structure** - Architecture JSON doesn't have `architecture` property
4. **TypeScript exporter error** - `generateSystemDiagramForArch()` throws an exception
5. **Empty result** - Exporter returns empty string

---

## Improved Error Handling

### In `convertDslToMermaid()` (packages/shared/src/web/wasmAdapter.ts)
- ✅ Checks if WASM is available
- ✅ Validates JSON parsing result
- ✅ Validates architecture structure
- ✅ Wraps TypeScript exporter in try-catch
- ✅ Logs detailed error information

### In `CodeBlockActions.tsx`
- ✅ Catches `convertDslToMermaid()` errors
- ✅ Provides fallback error diagnosis
- ✅ Shows specific error messages to user
- ✅ Logs errors to console for debugging

---

## How to Debug

### Step 1: Check Browser Console
Open DevTools Console and look for:
- `[CodeBlockActions] convertDslToMermaid error:` - Shows the actual error
- `WASM not available` - WASM initialization failed
- `WASM parser returned empty JSON` - Parsing failed
- `Invalid architecture JSON structure` - JSON structure issue
- `TypeScript mermaid exporter threw error` - Exporter exception
- `TypeScript mermaid exporter returned empty result` - Exporter returned empty

### Step 2: Check Network Tab
Verify WASM files are loading:
- `wasm/sruja.wasm` - Should load successfully
- `wasm/wasm_exec.js` - Should load successfully

### Step 3: Test Parsing Separately
```javascript
// In browser console
const api = await initWasm()
const json = await api.parseDslToJson('your dsl here')
console.log('Parsed JSON:', json)
```

### Step 4: Test Exporter Separately
```javascript
// In browser console (after parsing)
import { generateSystemDiagramForArch } from '@sruja/shared/export/mermaid'
const archJson = JSON.parse(json)
const mermaid = generateSystemDiagramForArch(archJson)
console.log('Mermaid:', mermaid)
```

---

## Common Issues & Solutions

### Issue: "WASM not available"
**Cause**: WASM files not loaded or initialization failed

**Solution**:
1. Check Network tab - are WASM files loading?
2. Check console for WASM load errors
3. Try refreshing the page
4. Check if WASM files exist in `public/wasm/` directory

### Issue: "WASM parser returned empty JSON"
**Cause**: DSL syntax error or invalid input

**Solution**:
1. Check DSL syntax
2. Verify DSL is not empty
3. Check console for parse errors

### Issue: "Invalid architecture JSON structure"
**Cause**: Parsed JSON doesn't have `architecture` property

**Solution**:
1. Check what JSON was returned
2. Verify DSL is valid architecture definition
3. Check if JSON structure matches expected format

### Issue: "TypeScript mermaid exporter threw error"
**Cause**: Exporter function encountered an error

**Solution**:
1. Check console for full error stack trace
2. Verify architecture JSON has required properties
3. Check if architecture has systems/persons/containers

### Issue: "TypeScript mermaid exporter returned empty result"
**Cause**: Exporter returned empty string

**Solution**:
1. Check if architecture has any elements (systems, persons, etc.)
2. Verify architecture structure is correct
3. Check console logs for architecture details

---

## Testing the Fix

After the improvements, you should see:

1. **Better Error Messages**: More specific error messages instead of generic "Mermaid export not available"
2. **Console Logging**: Detailed error information in browser console
3. **Error Diagnosis**: Automatic diagnosis of whether issue is parsing or export

---

## Next Steps

1. **Check Browser Console**: Look for the specific error message
2. **Share Error Details**: The improved error handling will show exactly what failed
3. **Verify WASM Loading**: Check Network tab to ensure WASM files load
4. **Test with Simple DSL**: Try with a minimal valid DSL to isolate the issue

---

## Code Location

- **Helper Function**: `packages/shared/src/web/wasmAdapter.ts` (line 436)
- **TypeScript Exporter**: `packages/shared/src/export/mermaid.ts` (line 1009)
- **Usage**: `apps/website/src/shared/components/ui/CodeBlockActions.tsx` (line 106)

---

**Status**: ✅ Code is correct - using TypeScript exporter. Error handling improved to help diagnose the actual issue.

