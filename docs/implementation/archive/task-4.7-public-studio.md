# Task 4.7: Public Studio (Zero-Friction Web App)

**Priority**: 🟢 Medium (Nice to have, can be done after CLI Studio)
**Technology**: React + TypeScript + WebAssembly
**Estimated Time**: 5-7 days
**Dependencies**: Task 3.1 (Viewer Library), Task 1.1 (JSON Exporter), Task 1.2 (JSON to AST)

## Goal

Create a zero-friction web application hosted on `sruja.ai/studio/` that allows anyone to try Sruja without installation, authentication, or setup.

**Note**: Studio is already deployed to `https://sruja.ai/studio/` via GitHub Pages. This task focuses on adding the public-facing features.

## Key Features

1. **Visual Diagram Editor**
   - Drag-and-drop elements (systems, containers, components)
   - Create relations between elements
   - Edit element properties
   - Delete elements

2. **Import DSL** (via WASM)
   - Upload `.sruja` file
   - Paste DSL text
   - Load from URL (GitHub raw, etc.)
   - Load from Google Drive (future)

3. **Export DSL** (via WASM)
   - Download as `.sruja` file
   - Copy DSL text to clipboard
   - Save to Google Drive (future)

4. **Export HTML**
   - Download interactive HTML diagram
   - Self-contained (includes viewer library)

5. **Local Storage**
   - Auto-save to browser local storage
   - Restore on page reload

## Technology Stack

- **React 19** + TypeScript
- **Cytoscape.js** (diagram rendering via Viewer library)
- **Tailwind CSS** (styling)
- **WebAssembly (WASM)** - Go parser/printer compiled to WASM
- **Local Storage API** (persistence)

## Implementation

### WASM Setup

**Build Go Parser to WASM**:
```bash
# Build parser WASM
GOOS=js GOARCH=wasm go build -o public-studio/wasm/sruja-parser.wasm ./pkg/wasm/parser

# Build printer WASM
GOOS=js GOARCH=wasm go build -o public-studio/wasm/sruja-printer.wasm ./pkg/wasm/printer

# Copy Go WASM runtime
cp "$(go env GOROOT)/misc/wasm/wasm_exec.js" public-studio/wasm/
```

**WASM Go Code**:
```go
// pkg/wasm/parser.go
//go:build wasm
package wasm

import (
    "syscall/js"
    "github.com/sruja-ai/sruja/pkg/language"
    "github.com/sruja-ai/sruja/pkg/export/json"
)

func ParseDSL(this js.Value, args []js.Value) interface{} {
    dslText := args[0].String()
    
    // Parse DSL using Go parser
    program, err := language.ParseString(dslText)
    if err != nil {
        return map[string]interface{}{"error": err.Error()}
    }
    
    // Export to JSON
    exporter := json.NewExporter()
    jsonData, err := exporter.Export(program)
    if err != nil {
        return map[string]interface{}{"error": err.Error()}
    }
    
    return map[string]interface{}{"json": string(jsonData)}
}

func main() {
    js.Global().Set("SrujaParser", js.ValueOf(map[string]interface{}{
        "parseDSL": js.FuncOf(ParseDSL),
    }))
    <-make(chan bool)
}
```

**TypeScript Integration**:
```typescript
// src/lib/dsl-import.ts
import init, { parseDSL } from '../wasm/sruja-parser';

export async function importDSL(dslText: string): Promise<Architecture> {
    await init(); // Initialize WASM
    const result = parseDSL(dslText);
    
    if (result.error) {
        throw new Error(result.error);
    }
    
    return JSON.parse(result.json);
}
```

### Component Structure

```
public-studio/
  ├── src/
  │   ├── components/
  │   │   ├── DiagramEditor.tsx      # Main editor
  │   │   ├── ElementPalette.tsx     # Element types
  │   │   ├── PropertiesPanel.tsx    # Element editing
  │   │   ├── ImportDialog.tsx       # Import options
  │   │   ├── ExportDialog.tsx       # Export options
  │   │   └── Toolbar.tsx            # Actions
  │   ├── lib/
  │   │   ├── viewer.ts              # Viewer library
  │   │   ├── dsl-import.ts          # DSL import (WASM)
  │   │   ├── dsl-export.ts          # DSL export (WASM)
  │   │   ├── html-export.ts         # HTML export
  │   │   └── storage.ts             # Local storage
  │   ├── wasm/
  │   │   ├── sruja-parser.wasm      # Go parser compiled to WASM
  │   │   ├── sruja-printer.wasm     # Go printer compiled to WASM
  │   │   └── wasm_exec.js           # Go WASM runtime
  │   ├── App.tsx
  │   └── main.tsx
  └── public/
      └── index.html
```

### Error Handling

Follow [Error Reporting Strategy](../ERROR_REPORTING_STRATEGY.md) for Public Studio:
- Visual error indicators on diagram elements
- Error banner with collapsible details
- Inline error markers on canvas
- Quick fix suggestions (buttons)
- Toast notifications for network errors

## What's NOT Included

❌ **Authentication** - No user accounts  
❌ **Cloud Storage** - No saving to server  
❌ **Sharing** - No shareable links  
❌ **Collaboration** - No comments, reviews  
❌ **Change Management** - No changes, ADRs, snapshots  
❌ **Git Integration** - No repository connection  

## Deployment

**Static Hosting** (GitHub Pages, Netlify, Vercel):
- Build React app: `npm run build`
- Deploy `dist/` directory
- No backend required

**CDN Assets**:
- Viewer Library: `https://sruja.ai/viewer.js`
- WASM Modules: Hosted with app

## Acceptance Criteria

- [ ] Visual diagram editor works (drag-and-drop, edit, delete)
- [ ] Import DSL from file (via WASM)
- [ ] Import DSL from text paste (via WASM)
- [ ] Import DSL from URL (via WASM)
- [ ] Export DSL to file (via WASM)
- [ ] Export DSL to clipboard (via WASM)
- [ ] Export HTML (interactive diagram)
- [ ] Local storage auto-save/restore works
- [ ] Error handling displays errors clearly
- [ ] WASM modules load correctly
- [ ] Works offline (after initial load)
- [ ] No backend required

## Dependencies

- Task 3.1 (Viewer Library) - For diagram rendering
- Task 1.1 (JSON Exporter) - For WASM parser
- Task 1.2 (JSON to AST) - For WASM printer
- Error Reporting Strategy - For error handling

## Notes

- **Different from Local Studio**: Public Studio is standalone, no file operations, uses WASM
- **Different from Self-Hosted Studio**: Public Studio is public web app, no sharing features
- **WASM Bundle Size**: ~4-6 MB total (can be optimized with lazy loading)
- **Future**: Google Drive integration (Phase 3)

## Comparison

| Feature | Public Studio | Local Studio | Self-Hosted Studio |
|---------|--------------|--------------|-------------------|
| **Installation** | ❌ None | ✅ CLI install | ✅ Server deploy |
| **File Operations** | ❌ Local only | ✅ Read/write files | ❌ Export only |
| **DSL Import/Export** | ✅ WASM | ✅ Go API | ✅ Go API |
| **Git Connection** | ❌ None | ✅ Local repo | ❌ None |
| **Sharing** | ❌ None | ❌ Local only | ✅ Shareable links |

