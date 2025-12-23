# WASM Caching and Development Notes

## Problem
After rebuilding the WASM file, browsers may cache the old version, requiring multiple refresh attempts ("refresh from source" + hard refresh) to see changes. This is confusing and slows down development.

## Solutions Implemented

### 1. Vite Dev Server Cache-Busting
- **Location**: `vite.config.ts`
- **Changes**:
  - Added `Cache-Control: no-cache, no-store, must-revalidate` headers for WASM files
  - Added ETag support based on file modification time
  - Properly handles query parameters in WASM URLs
  - Applied to both dev server and preview server

### 2. Client-Side Cache-Busting
- **Location**: `packages/shared/src/web/wasmAdapter.ts`
- **Changes**:
  - Improved cache-busting query parameter (uses `?v=` instead of `?t=`)
  - Added dev-mode console warnings with helpful instructions
  - Logs WASM load URL with cache-busting info

### 3. WASM Freshness Checker
- **Location**: `scripts/check-wasm-freshness.mjs`
- **Purpose**: Warns if WASM file is older than 5 minutes when starting dev server
- **Usage**: Automatically runs when you run `npm run dev`

### 4. New Dev Scripts
- `npm run dev:rebuild-wasm` - Rebuilds WASM and starts dev server in one command

## How It Works

1. **Server-Side**: Vite dev server sends no-cache headers for WASM files, preventing browser caching
2. **Client-Side**: WASM loader adds cache-busting query parameter (`?v=timestamp`)
3. **ETag Support**: Server uses file modification time as ETag for proper cache validation
4. **Developer Warnings**: Console logs help developers understand when WASM needs refreshing

## Best Practices

1. **After Rebuilding WASM**:
   - ✅ **No "reload from source" needed!** The cache-busting ensures fresh WASM loads automatically
   - The dev server sends no-cache headers, so a simple refresh should work
   - If changes don't appear (rare):
     - Hard refresh: `Ctrl+Shift+R` (Windows/Linux) or `Cmd+Shift+R` (Mac)
     - Or enable "Disable cache" in DevTools Network tab

2. **Quick Rebuild Workflow**:
   ```bash
   # Option 1: Rebuild and restart dev server
   npm run dev:rebuild-wasm
   
   # Option 2: Manual rebuild
   make wasm
   npm run copy:wasm
   # Then restart dev server or hard refresh browser
   ```

3. **Troubleshooting**:
   - Check console for `[WASM]` messages
   - Verify WASM file timestamp: `ls -lh public/wasm/sruja.wasm`
   - Check Network tab in DevTools to see if WASM is being cached

## Technical Details

- **ETag Format**: `"${mtime.getTime()}-${fileSize}"` - changes when file is modified
- **Cache-Busting**: Query parameter `?v=${timestamp}` ensures unique URL per load in dev mode
- **Headers**: `Cache-Control: no-cache` tells browser to revalidate, `no-store` prevents caching entirely

