# Playground Migration: Learn App → Public Studio

## Overview

Migrate the Playground component from the `learn/` Hugo documentation site to `public-studio/` as a core feature. This consolidates interactive Sruja tools into one place.

## Current State

**Playground in `learn/`**:
- Location: `learn/assets/js/components/Playground.tsx`
- Purpose: Interactive DSL editor with live preview
- Features:
  - Code editor (textarea)
  - Live diagram rendering
  - Example selection
  - Export options
- Used in: Hugo content pages (`learn/content/playground/`)

## Target State

**Playground in `public-studio/`**:
- Location: `public-studio/src/components/Playground.tsx`
- Purpose: Same interactive DSL editor, but as part of Public Studio
- Features:
  - Code editor (textarea or Monaco editor)
  - Live diagram rendering (via viewer library)
  - Example selection
  - Import/Export DSL (via WASM)
  - Local storage
- Access: `sruja.ai/studio` (main Public Studio) or `sruja.ai/studio/playground` (playground mode)

## Migration Plan

### Phase 1: Public Studio Implementation (Prerequisite)

1. ✅ Build Public Studio with core features
2. ✅ Implement WASM import/export
3. ✅ Integrate viewer library
4. ✅ Add local storage

### Phase 2: Playground Migration

**Step 1: Extract Playground Component**

```typescript
// public-studio/src/components/Playground.tsx
import { useState, useEffect } from 'react';
import { SrujaViewer } from '@sruja/viewer';
import { importDSL, exportDSL } from '../lib/dsl-import-export';

export function Playground() {
  const [dslCode, setDslCode] = useState<string>('');
  const [architecture, setArchitecture] = useState<Architecture | null>(null);
  const [error, setError] = useState<string | null>(null);
  
  // Load from local storage or examples
  useEffect(() => {
    const saved = loadFromLocalStorage();
    if (saved) {
      setDslCode(saved);
      handleCodeChange(saved);
    } else {
      // Load default example
      loadExample('simple');
    }
  }, []);
  
  const handleCodeChange = async (code: string) => {
    setDslCode(code);
    saveToLocalStorage(code);
    
    try {
      const arch = await importDSL(code); // Uses WASM
      setArchitecture(arch);
      setError(null);
    } catch (err) {
      setError(err.message);
      setArchitecture(null);
    }
  };
  
  return (
    <div className="playground-container">
      <div className="playground-editor">
        <textarea
          value={dslCode}
          onChange={(e) => handleCodeChange(e.target.value)}
          placeholder="Enter Sruja DSL code..."
        />
        {error && <div className="error">{error}</div>}
      </div>
      <div className="playground-preview">
        {architecture && (
          <SrujaViewer
            container="#diagram"
            data={architecture}
          />
        )}
      </div>
    </div>
  );
}
```

**Step 2: Add Playground Mode to Public Studio**

```typescript
// public-studio/src/App.tsx
import { Playground } from './components/Playground';
import { Studio } from './components/Studio';

export function App() {
  const [mode, setMode] = useState<'studio' | 'playground'>('studio');
  
  // Check URL for mode
  useEffect(() => {
    const path = window.location.pathname;
    if (path.includes('/playground')) {
      setMode('playground');
    }
  }, []);
  
  return (
    <div className="app">
      <Navigation mode={mode} onModeChange={setMode} />
      {mode === 'playground' ? <Playground /> : <Studio />}
    </div>
  );
}
```

**Step 3: Update Routes**

- `sruja.ai/studio` → Full Studio (visual editor)
- `sruja.ai/studio/playground` → Playground mode (code editor)

**Step 4: Migrate Examples**

Move examples from `learn/assets/js/examples.generated.ts` to `public-studio/src/examples/`:

```typescript
// public-studio/src/examples/index.ts
export const examples = {
  simple: `architecture "Simple App" {
    system WebApp {}
    system Database {}
    relation WebApp -> Database "Reads/Writes"
  }`,
  // ... other examples
};
```

### Phase 3: Update Learn App

**After Migration**:

1. **Remove Playground from `learn/`**:
   - Delete `learn/assets/js/components/Playground.tsx`
   - Delete `learn/content/playground/`
   - Update Hugo content to link to Public Studio

2. **Update Learn Content**:
   ```markdown
   <!-- learn/content/tutorials/basic/dsl-basics.md -->
   Try it out in the [Public Studio Playground](https://sruja.ai/studio/playground)!
   ```

3. **Keep WASM in `learn/static/`**:
   - Playground in learn still uses WASM for rendering
   - Can share WASM files between learn and public-studio
   - Or remove WASM from learn if not needed

## Benefits

✅ **Consolidation** - All interactive tools in one place  
✅ **Better UX** - Playground gets full Studio features (import/export, local storage)  
✅ **Simpler Learn App** - Learn focuses on documentation  
✅ **Shared Code** - Playground uses same viewer library as Studio  
✅ **WASM Support** - Full DSL parsing via WASM (not just rendering)  

## Implementation Details

### Playground Features in Public Studio

**Enhanced Features** (beyond current playground):
- ✅ Import DSL from file (via WASM)
- ✅ Export DSL to file (via WASM)
- ✅ Copy DSL to clipboard
- ✅ Local storage (auto-save)
- ✅ Error handling (visual indicators)
- ✅ Example library (same examples)
- ✅ Live preview (same as before)

**UI Options**:

**Option 1: Separate Route**
- `/studio` → Full Studio (visual editor)
- `/studio/playground` → Playground (code editor)

**Option 2: Toggle Mode**
- Toggle button in Studio: "Code Editor" ↔ "Visual Editor"
- Same route, different view

**Recommendation**: Option 1 (separate routes) - clearer separation, better for sharing links.

### Code Sharing

**Shared Components**:
- Viewer library (npm package)
- WASM modules (same parser/printer)
- Example data (can be shared)

**Public Studio Specific**:
- Playground component (code editor)
- Studio component (visual editor)
- Import/Export dialogs

## Migration Checklist

- [ ] Public Studio implemented (core features)
- [ ] WASM import/export working
- [ ] Viewer library integrated
- [ ] Playground component created in Public Studio
- [ ] Examples migrated to Public Studio
- [ ] Routes configured (`/studio` and `/studio/playground`)
- [ ] Local storage working
- [ ] Error handling implemented
- [ ] Playground removed from `learn/`
- [ ] Learn content updated (links to Public Studio)
- [ ] Documentation updated

## Timeline

**After Public Studio (Task 4.7) is complete**:
- Week 1: Extract and migrate Playground component
- Week 2: Update routes, integrate examples
- Week 3: Remove from learn, update documentation
- Week 4: Testing and polish

**Estimated Time**: 1-2 weeks after Public Studio is done

## Notes

- **Backward Compatibility**: Old playground links in learn can redirect to Public Studio
- **WASM Sharing**: Can share WASM files between learn and public-studio (if learn still needs rendering)
- **Examples**: Can keep examples in both places initially, migrate gradually
- **Documentation**: Update all playground references to point to Public Studio

