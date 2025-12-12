# Examples Migration to Shared Service

## Summary

All apps now use the shared examples service from `@sruja/shared` instead of maintaining their own examples lists.

## Changes Made

### 1. Architecture Visualizer (`apps/architecture-visualizer`)

**Before:**
- Had hardcoded `EXAMPLES` array with only 3 examples
- Only supported JSON files

**After:**
- Uses `getAvailableExamples()` from `@sruja/shared`
- Loads all examples from `manifest.json`
- Supports DSL files (converts via WASM)
- Added DSL tab to show source code
- Examples grouped by category in dropdown

**Files Updated:**
- `src/examples.ts` - Now uses shared service
- `src/components/shared/ExamplesDropdown.tsx` - Loads from shared service
- `src/components/Panels/DSLPanel.tsx` - New component to show DSL source
- `src/App.tsx` - Added DSL tab

### 2. Studio Core (`apps/studio-core`)

**Before:**
- Had hardcoded `EXAMPLES` object with 5 quick templates
- Templates were inline DSL strings

**After:**
- Uses `getAvailableExamples()` from `@sruja/shared` for all examples
- Keeps `QUICK_TEMPLATES` for fast-loading templates in welcome screen
- Loads DSL files from shared service when selected
- Combines quick templates + shared examples in dropdown

**Files Updated:**
- `src/examples.ts` - Now uses shared service + quick templates
- `src/components/WelcomeScreen.tsx` - Loads DSL from shared service
- `src/components/StudioToolbar.tsx` - Dynamically loads examples
- `src/App.tsx` - Updated to use new example loading
- `src/hooks/useViewer.ts` - Uses QUICK_TEMPLATES instead of EXAMPLES

### 3. Website Viewer (`apps/website`)

**Status:** ✅ Already using shared service
- `src/features/viewer/hooks/useExamples.ts` - Already uses `@sruja/shared`

## Shared Service API

The shared examples service (`@sruja/shared/src/examples/index.ts`) provides:

```typescript
// Get all available examples (filtered and sorted)
getAvailableExamples(): Promise<Example[]>

// Load example file content (DSL or JSON)
loadExampleFile(filename: string): Promise<string>

// Load examples manifest
loadExamplesManifest(): Promise<ExamplesManifest>
```

## Example Structure

Examples are defined in `examples/manifest.json`:

```json
{
  "examples": [
    {
      "file": "simple.sruja",
      "name": "Quick Start",
      "order": 1,
      "category": "basic",
      "description": "Simple architecture example",
      "skipPlayground": false
    }
  ]
}
```

## Benefits

1. **Single Source of Truth**: All examples defined in one place (`examples/manifest.json`)
2. **Consistency**: All apps show the same examples
3. **Easy Updates**: Add new examples by updating `manifest.json`
4. **DSL Support**: All apps can load and display DSL files
5. **Categorization**: Examples are organized by category
6. **Filtering**: Can filter examples (e.g., `skipPlayground`)

## Migration Checklist

- [x] Architecture Visualizer migrated
- [x] Studio Core migrated
- [x] Website Viewer already using shared service
- [x] DSL tab added to Architecture Visualizer
- [x] All examples from manifest.json now visible

## Testing

To verify the migration:

1. **Architecture Visualizer**: 
   - Open http://localhost:5173
   - Click "Examples" dropdown
   - Should see all examples from manifest.json grouped by category
   - Click DSL tab to see source code

2. **Studio Core**:
   - Open studio app
   - Check example dropdown in toolbar
   - Should see quick templates + all shared examples

3. **Website Viewer**:
   - Already working with shared service

## Future Improvements

- [ ] Add search/filter in examples dropdown
- [ ] Add example preview thumbnails
- [ ] Add example tags for better filtering
- [ ] Add example popularity/usage metrics
