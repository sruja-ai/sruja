# Public Studio: Zero-Friction Try-Out

## Overview

A simple, public web application hosted on `sruja.ai/studio` that allows anyone to try Sruja without installation, authentication, or setup. Perfect for quick exploration and evaluation.

## Goals

✅ **Zero Friction** - No installation, no sign-up, no authentication  
✅ **Quick Start** - Create DSL from scratch using visual diagrams  
✅ **Import/Export** - Import existing DSL, export DSL or HTML  
✅ **Simple** - Focus on core functionality only  
✅ **Playground Mode** - Code editor with live preview (future: migrate from learn app)  
❌ **No Collaboration** - No sharing, no comments, no auth  
❌ **No Persistence** - No saving to cloud (local storage only)  

## Features

### Core Features

1. **Visual Diagram Editor**
   - Drag-and-drop elements (systems, containers, components)
   - Create relations between elements
   - Edit element properties
   - Delete elements

2. **Import DSL**
   - Upload `.sruja` file
   - Paste DSL text
   - Load from URL (GitHub raw, etc.)
   - Load from Google Drive (future)

3. **Export DSL**
   - Download as `.sruja` file
   - Copy DSL text to clipboard
   - Save to Google Drive (future)

4. **Export HTML**
   - Download interactive HTML diagram
   - Self-contained (includes viewer library)
   - Can be shared/opened anywhere

5. **Local Storage**
   - Auto-save to browser local storage
   - Restore on page reload
   - Clear/reset option

### What's NOT Included

❌ **Authentication** - No user accounts  
❌ **Cloud Storage** - No saving to server  
❌ **Sharing** - No shareable links  
❌ **Collaboration** - No comments, reviews  
❌ **Change Management** - No changes, ADRs, snapshots  
❌ **Git Integration** - No repository connection  

## Architecture

### Frontend Only

**Technology**:
- React 19 + TypeScript
- Cytoscape.js (diagram rendering)
- Tailwind CSS (styling)
- Local Storage API (persistence)
- WebAssembly (WASM) for DSL import/export (Go parser/printer compiled to WASM)

**Deployment**:
- Static hosting (GitHub Pages, Netlify, Vercel)
- CDN for fast global access
- No backend required

### Data Flow

```
User Interaction
    ↓
React UI (Studio)
    ↓
JSON (in-memory)
    ↓
┌───────────┬───────────┬───────────┐
│           │           │           │
Import DSL  Export DSL  Export HTML  Local Storage
(WASM)      (WASM)      (Client)     (Browser)
```

**No Server Required**:
- All processing client-side
- **DSL import via WASM** (Go parser compiled to WASM)
- **DSL export via WASM** (Go printer compiled to WASM)
- HTML export includes embedded viewer
- No API calls (except optional URL import for Google Drive)

## Implementation

### Component Structure

```
public-studio/
  ├── src/
  │   ├── components/
  │   │   ├── DiagramEditor.tsx      # Main editor
  │   │   ├── ElementPalette.tsx      # Element types
  │   │   ├── PropertiesPanel.tsx     # Element editing
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

### Key Components

#### 1. Diagram Editor

**File**: `src/components/DiagramEditor.tsx`

```typescript
import { useCytoscape } from '../lib/viewer';
import { Architecture } from '../types';

export function DiagramEditor() {
  const [architecture, setArchitecture] = useState<Architecture | null>(null);
  const { cytoscapeRef, addElement, addRelation, updateElement, deleteElement } = useCytoscape(architecture);
  
  // Drag-and-drop handlers
  // Element creation/editing
  // Relation creation
}
```

**Features**:
- Visual canvas (Cytoscape)
- Drag elements from palette
- Click to select/edit
- Drag to create relations
- Delete with keyboard/button

#### 2. Import Dialog

**File**: `src/components/ImportDialog.tsx`

```typescript
export function ImportDialog({ onImport }: { onImport: (arch: Architecture) => void }) {
  const [mode, setMode] = useState<'file' | 'text' | 'url' | 'googledrive'>('file');
  
  // File upload
  const handleFileUpload = async (file: File) => {
    const text = await file.text();
    const architecture = await importDSL(text); // Uses WASM
    onImport(architecture);
  };
  
  // Text paste
  const handleTextPaste = async (text: string) => {
    const architecture = await importDSL(text); // Uses WASM
    onImport(architecture);
  };
  
  // URL load
  const handleUrlLoad = async (url: string) => {
    const response = await fetch(url);
    const text = await response.text();
    const architecture = await importDSL(text); // Uses WASM
    onImport(architecture);
  };
  
  // Google Drive load (future)
  const handleGoogleDriveLoad = async (fileId: string) => {
    // Authenticate with Google Drive API
    const fileContent = await loadFromDrive(fileId);
    const architecture = await importDSL(fileContent); // Uses WASM
    onImport(architecture);
  };
}
```

**Options**:
- **Upload File**: Select `.sruja` file
- **Paste Text**: Paste DSL directly
- **Load from URL**: Fetch from URL (GitHub raw, etc.)
- **Load from Google Drive**: Select file from Google Drive (future)

#### 3. Export Dialog

**File**: `src/components/ExportDialog.tsx`

```typescript
export function ExportDialog({ architecture }: { architecture: Architecture }) {
  // Export DSL
  const handleExportDSL = async () => {
    const dslText = await exportDSL(architecture); // Uses WASM
    downloadFile('architecture.sruja', dslText);
  };
  
  // Export HTML
  const handleExportHTML = () => {
    // Generate HTML with embedded viewer
    // Download as .html file
  };
  
  // Copy DSL
  const handleCopyDSL = async () => {
    const dslText = await exportDSL(architecture); // Uses WASM
    await navigator.clipboard.writeText(dslText);
  };
  
  // Save to Google Drive (future)
  const handleSaveToGoogleDrive = async () => {
    // Authenticate with Google Drive API
    // Convert to DSL
    // Upload to Google Drive
  };
}
```

**Options**:
- **Download DSL**: `.sruja` file
- **Download HTML**: Interactive `.html` file
- **Copy DSL**: Copy to clipboard
- **Save to Google Drive**: Upload to Google Drive (future)

#### 4. Local Storage

**File**: `src/lib/storage.ts`

```typescript
const STORAGE_KEY = 'sruja-studio-architecture';

export function saveToLocalStorage(architecture: Architecture) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(architecture));
}

export function loadFromLocalStorage(): Architecture | null {
  const data = localStorage.getItem(STORAGE_KEY);
  return data ? JSON.parse(data) : null;
}

export function clearLocalStorage() {
  localStorage.removeItem(STORAGE_KEY);
}
```

**Features**:
- Auto-save on changes
- Restore on page load
- Clear option

### Google Drive Integration (Future)

**How it works**:
- Uses Google OAuth (no Sruja account required)
- User authenticates with Google account
- Browse Google Drive for `.sruja` files
- Load files directly from Drive
- Save files back to Drive

**Implementation**:
```typescript
// src/lib/googledrive.ts
import { gapi } from 'gapi-script';

export async function initGoogleDrive() {
  await gapi.load('client:auth2', async () => {
    await gapi.client.init({
      apiKey: process.env.GOOGLE_API_KEY,
      clientId: process.env.GOOGLE_CLIENT_ID,
      discoveryDocs: ['https://www.googleapis.com/discovery/v1/apis/drive/v3/rest'],
      scope: 'https://www.googleapis.com/auth/drive.file'
    });
  });
}

export async function listDSLFiles() {
  const response = await gapi.client.drive.files.list({
    q: "mimeType='text/plain' and name contains '.sruja'",
    fields: 'files(id, name)'
  });
  return response.result.files;
}

export async function loadFromDrive(fileId: string): Promise<string> {
  const response = await gapi.client.drive.files.get({
    fileId: fileId,
    alt: 'media'
  });
  return response.body;
}

export async function saveToDrive(filename: string, content: string): Promise<string> {
  const file = new Blob([content], { type: 'text/plain' });
  const metadata = { name: filename };
  const form = new FormData();
  form.append('metadata', new Blob([JSON.stringify(metadata)], { type: 'application/json' }));
  form.append('file', file);
  
  const response = await gapi.client.request({
    path: 'https://www.googleapis.com/upload/drive/v3/files',
    method: 'POST',
    params: { uploadType: 'multipart' },
    body: form
  });
  return response.result.id;
}
```

**Note**: Google Drive integration uses OAuth for file access only - no Sruja user accounts or authentication required. Users authenticate with Google to access their own Drive files.

### DSL Import/Export (WASM)

**Both import and export use WASM** - Compile Go parser/printer to WebAssembly for full feature support.

#### Import (DSL → JSON) via WASM

**Implementation**: Compile Go parser to WASM

```go
// pkg/wasm/parser.go
//go:build wasm
// +build wasm

package wasm

import (
    "syscall/js"
    "github.com/sruja-ai/sruja/pkg/language"
)

// ParseDSL parses DSL text and returns JSON
// Exported to JavaScript as parseDSL
func ParseDSL(this js.Value, args []js.Value) interface{} {
    dslText := args[0].String()
    
    // Parse DSL using Go parser
    program, err := language.ParseString(dslText)
    if err != nil {
        return map[string]interface{}{
            "error": err.Error(),
        }
    }
    
    // Export to JSON
    exporter := json.NewExporter()
    jsonData, err := exporter.Export(program)
    if err != nil {
        return map[string]interface{}{
            "error": err.Error(),
        }
    }
    
    // Return JSON string
    return map[string]interface{}{
        "json": string(jsonData),
    }
}

func main() {
    js.Global().Set("SrujaParser", js.ValueOf(map[string]interface{}{
        "parseDSL": js.FuncOf(ParseDSL),
    }))
    <-make(chan bool) // Keep alive
}
```

**Build WASM**:
```bash
GOOS=js GOARCH=wasm go build -o sruja-parser.wasm ./pkg/wasm
```

**TypeScript Usage**:
```typescript
// src/lib/dsl-import.ts
import init, { parseDSL } from '../wasm/sruja-parser';

export async function importDSL(dslText: string): Promise<Architecture> {
    // Initialize WASM module
    await init();
    
    // Parse DSL to JSON
    const result = parseDSL(dslText);
    
    if (result.error) {
        throw new Error(result.error);
    }
    
    // Parse JSON to Architecture object
    return JSON.parse(result.json);
}
```

#### Export (JSON → DSL) via WASM

**Implementation**: Compile Go printer to WASM

```go
// pkg/wasm/printer.go
//go:build wasm
// +build wasm

package wasm

import (
    "encoding/json"
    "syscall/js"
    "github.com/sruja-ai/sruja/pkg/language"
)

// ExportDSL converts JSON to DSL
// Exported to JavaScript as exportDSL
func ExportDSL(this js.Value, args []js.Value) interface{} {
    jsonText := args[0].String()
    
    // Parse JSON
    var archJSON json.ArchitectureJSON
    if err := json.Unmarshal([]byte(jsonText), &archJSON); err != nil {
        return map[string]interface{}{
            "error": err.Error(),
        }
    }
    
    // Convert JSON to AST
    converter := json.NewConverter()
    program, err := converter.Convert(&archJSON)
    if err != nil {
        return map[string]interface{}{
            "error": err.Error(),
        }
    }
    
    // Print AST to DSL
    printer := language.NewPrinter()
    dslText, err := printer.Print(program)
    if err != nil {
        return map[string]interface{}{
            "error": err.Error(),
        }
    }
    
    return map[string]interface{}{
        "dsl": dslText,
    }
}

func main() {
    js.Global().Set("SrujaPrinter", js.ValueOf(map[string]interface{}{
        "exportDSL": js.FuncOf(ExportDSL),
    }))
    <-make(chan bool) // Keep alive
}
```

**TypeScript Usage**:
```typescript
// src/lib/dsl-export.ts
import init, { exportDSL } from '../wasm/sruja-printer';

export async function exportDSL(architecture: Architecture): Promise<string> {
    // Initialize WASM module
    await init();
    
    // Convert Architecture to JSON
    const json = JSON.stringify(architecture);
    
    // Convert JSON to DSL
    const result = exportDSL(json);
    
    if (result.error) {
        throw new Error(result.error);
    }
    
    return result.dsl;
}
```

**Benefits**:
- ✅ **Full feature support** - Uses same Go parser/printer as CLI
- ✅ **Consistent behavior** - Same parsing logic everywhere
- ✅ **No backend required** - Runs entirely client-side
- ✅ **Round-trip guarantee** - Same code ensures preservation
- ✅ **Error handling** - Same error messages as CLI

**Build Process**:
```bash
# Build parser WASM
GOOS=js GOARCH=wasm go build -o public-studio/wasm/sruja-parser.wasm ./pkg/wasm/parser

# Build printer WASM
GOOS=js GOARCH=wasm go build -o public-studio/wasm/sruja-printer.wasm ./pkg/wasm/printer

# Copy Go WASM runtime
cp "$(go env GOROOT)/misc/wasm/wasm_exec.js" public-studio/wasm/
```

**Bundle Size**:
- Parser WASM: ~2-3 MB (compressed: ~500-800 KB)
- Printer WASM: ~2-3 MB (compressed: ~500-800 KB)
- Total: ~4-6 MB (compressed: ~1-1.5 MB)

**Optimization**:
- Use `tinygo` for smaller WASM size (optional)
- Lazy load WASM modules (load on first import/export)
- Compress WASM files (gzip/brotli)

#### Export (JSON → DSL)

**WASM Export (Recommended)**:
- Compile Go printer to WASM
- Full feature support
- Consistent with CLI

**Implementation**:
```typescript
// src/lib/dsl-export.ts
import init, { jsonToDSL } from '../wasm/sruja_wasm';

export async function exportDSL(architecture: Architecture): Promise<string> {
  await init(); // Initialize WASM
  const json = JSON.stringify(architecture);
  return jsonToDSL(json);
}
```

### HTML Export

**File**: `src/lib/html-export.ts`

```typescript
export function exportHTML(architecture: Architecture): string {
  const json = JSON.stringify(architecture);
  const viewerScript = '<script src="https://sruja.ai/viewer.js"></script>';
  
  return `<!DOCTYPE html>
<html>
<head>
  <title>Sruja Architecture</title>
  ${viewerScript}
</head>
<body>
  <div id="sruja-viewer"></div>
  <script>
    const architecture = ${json};
    SrujaViewer.render('sruja-viewer', architecture);
  </script>
</body>
</html>`;
}
```

**Features**:
- Self-contained HTML
- Embedded viewer library (CDN)
- Interactive diagram
- Can be shared/opened anywhere

## User Flow

### 1. First Visit

1. User opens `sruja.ai/studio`
2. Empty canvas shown
3. Element palette visible
4. Toolbar with Import/Export options

### 2. Create Architecture

1. Drag system from palette
2. Click to edit name/properties
3. Add containers/components
4. Create relations (drag between elements)
5. Auto-saved to local storage

### 3. Import Existing DSL

1. Click "Import" button
2. Choose: Upload / Paste / URL / Google Drive (future)
3. DSL parsed and loaded
4. Diagram rendered
5. Auto-saved to local storage

### 4. Export

1. Click "Export" button
2. Choose: DSL / HTML / Google Drive (future)
3. File downloaded (or saved to Google Drive)
4. (Or copy DSL to clipboard)

### 5. Return Visit

1. User opens `sruja.ai/studio`
2. Architecture restored from local storage
3. Continue editing

## Deployment

### Static Hosting

**Option 1: GitHub Pages**
- Free, simple
- Automatic deployment from main branch
- URL: `sruja-ai.github.io/studio`

**Option 2: Netlify**
- Free tier
- Automatic deployment
- Custom domain support

**Option 3: Vercel**
- Free tier
- Fast CDN
- Easy deployment

**Recommendation**: GitHub Pages (simple, free, integrated with repo)

### Build Process

```bash
# Build React app
cd public-studio
npm install
npm run build

# Output: dist/ directory (static files)
# Deploy dist/ to hosting
```

### CDN Assets

**Viewer Library**: `https://sruja.ai/viewer.js` (CDN)  
**WASM Module**: `https://sruja.ai/sruja.wasm` (CDN)  

## Comparison: Public Studio vs Other Options

| Feature | Public Studio | CLI Studio | Self-Hosted Studio |
|---------|--------------|------------|-------------------|
| **Installation** | ❌ None | ✅ CLI install | ✅ Server deploy |
| **Authentication** | ❌ None | ❌ None | ❌ None (simple) |
| **File Operations** | ❌ Local only | ✅ Read/write files | ❌ Export only |
| **Git Connection** | ❌ None | ✅ Local repo | ❌ None |
| **Sharing** | ❌ None | ❌ Local only | ✅ Shareable links |
| **Collaboration** | ❌ None | ❌ None | ❌ None |
| **Change Management** | ❌ None | ✅ Full support | ❌ None |
| **Use Case** | Try out | Local dev | Team sharing |

## Benefits

✅ **Zero Friction** - No installation, no sign-up  
✅ **Quick Evaluation** - Try before installing  
✅ **Easy Sharing** - Export HTML, share anywhere  
✅ **Simple** - Focus on core functionality  
✅ **Fast** - Static hosting, CDN  
✅ **Free** - No cost to users  

## Limitations

⚠️ **No Persistence** - Local storage only (cleared if browser data cleared)  
⚠️ **No Collaboration** - No sharing, no comments  
⚠️ **No Change Management** - No changes, ADRs, snapshots  
⚠️ **No Git Integration** - No repository connection  

## Future Enhancements (Optional)

**If needed later**:
- Export to multiple formats (PNG, SVG)
- Template library (common patterns)
- Example architectures
- Tutorial/guide
- **Google Drive integration** - Save/load DSL files from Google Drive (future)

**Not planned**:
- Authentication (no user accounts)
- Cloud storage (except Google Drive for file access)
- Collaboration features (no comments, reviews)

## Implementation Priority

**Phase 1** (MVP):
- ✅ Visual editor (basic)
- ✅ Import DSL (file, text) via WASM
- ✅ Export DSL via WASM
- ✅ Export HTML
- ✅ Local storage

**Phase 2** (Enhancements):
- ✅ Import from URL (via WASM)
- ✅ Better UI/UX
- ✅ Export PNG/SVG
- ✅ Lazy load WASM modules

**Phase 3** (Future):
- ⏳ Google Drive integration (save/load DSL files)
  - OAuth authentication with Google (no Sruja account needed)
  - Browse and select `.sruja` files from Google Drive
  - Save new/updated DSL files to Google Drive
  - Still no Sruja authentication - uses Google OAuth only
- ⏳ **Playground migration** - Migrate playground from `learn/` app to Public Studio
  - Code editor mode in Public Studio
  - Live preview with diagram rendering
  - Example library
  - Route: `sruja.ai/studio/playground`
  - See [Playground Migration](../PLAYGROUND_MIGRATION.md) for details

## Summary

**Public Studio** is a simple, zero-friction web app for trying Sruja:
- ✅ No installation
- ✅ No authentication
- ✅ Visual diagram editor
- ✅ Import/Export DSL
- ✅ Export HTML
- ✅ Local storage only

**Perfect for**: Quick evaluation, learning, simple diagrams, sharing HTML exports.

