# Plan: HTML + JSON Data + Cytoscape.js (CDN)

## Overview

**No D2 dependency!** Use:
- **Minimal HTML** - Just structure
- **JSON data file** - Architecture data only (no SVG, no D2 scripts)
- **Cytoscape.js from CDN** - Graph visualization library
- **Custom Sruja Viewer JS** - Fixed library for all Sruja architectures

**Key Benefits:**
- ✅ **No D2 dependency** - Eliminates WASM overhead
- ✅ **No compilation step** - Direct rendering from JSON
- ✅ **Much smaller** - No WASM (~500 KB+ saved)
- ✅ **Better performance** - Native JS rendering
- ✅ **Fully interactive** - Built-in zoom, pan, click, hover

## Architecture

### File Structure

```
architecture.html          # Minimal HTML (~5-10 KB)
architecture.json         # Architecture data (~20-100 KB, no SVG)
```

### HTML Structure (Minimal)

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Architecture: {name}</title>
  <link rel="stylesheet" href="https://cdn.sruja.ai/v1/sruja-viewer.css">
</head>
<body>
  <div id="sruja-app"></div>
  <!-- Cytoscape.js from CDN -->
  <script src="https://unpkg.com/cytoscape@3.27.0/dist/cytoscape.min.js"></script>
  <!-- Cytoscape layout extensions -->
  <script src="https://unpkg.com/cytoscape-dagre@2.5.0/cytoscape-dagre.js"></script>
  <!-- Sruja Viewer (fixed library for all architectures) -->
  <script src="https://cdn.sruja.ai/v1/sruja-viewer.js"></script>
  <script>
    SrujaViewer.init({
      container: '#sruja-app',
      data: './architecture.json'
    });
  </script>
</body>
</html>
```

### JSON Data Structure (NO SVG EMBEDDED)

```json
{
  "metadata": {
    "name": "E-commerce Platform",
    "version": "1.0.0",
    "generated": "2025-01-XX"
  },
  "architecture": {
    "systems": [
      {
        "id": "Shop",
        "label": "E-commerce Shop",
        "containers": [...],
        "components": [...]
      }
    ],
    "persons": [...],
    "relations": [...],
    "requirements": [...],
    "adrs": [...],
    "domains": [...],
    "scenarios": [...],
    "flows": [...]
  },
  "navigation": {
    "levels": ["level1", "level2", "level3"],
    "scenarios": [...],
    "flows": [...],
    "domains": [...]
  }
}
```

**Key Point**: JSON contains **architecture data only**, NOT SVG strings. JS library generates SVG from data.

## Benefits

### 1. **Much Smaller File Sizes (No WASM!)**
- HTML: ~5-10 KB (minimal structure)
- JSON: ~20-100 KB (just data, no SVG markup)
- Cytoscape.js: ~150 KB (from CDN, cached globally)
- cytoscape-dagre: ~50 KB (from CDN, cached globally)
- Sruja Viewer JS: ~50-150 KB (loaded once, cached)
- **Total**: ~275-460 KB first load, ~75-160 KB subsequent (Cytoscape cached)
- **vs Current**: 500 KB-2 MB (embedded SVG + D2 WASM overhead)
- **WASM Elimination**: Saves ~500 KB+ (no D2 WASM needed!)

### 2. **Shared JS Library**
- One JS file serves all architectures
- Browser caching (load once, use everywhere)
- Version control (update JS, all files benefit)
- CDN benefits (fast delivery, edge caching)

### 3. **Separation of Concerns**
- Data (JSON) separate from presentation (JS)
- Easy to update rendering without regenerating files
- Can swap rendering engines (D2, custom, etc.)

### 4. **Better Performance**
- JSON parsing is fast
- JS can lazy-load/generate views on demand
- Can use modern frameworks (React, Vue, etc.)
- Better code splitting

### 5. **Flexibility**
- Multiple rendering modes (SVG, Canvas, WebGL)
- Easy to add features (just update JS)
- Can support plugins/extensions
- Can change rendering style without regenerating files

## Implementation Plan

### Phase 1: JSON Exporter (Architecture Data Only)

1. **Create JSON Exporter**
   - `pkg/export/json/json.go`
   - Export architecture to structured JSON
   - **NO SVG strings** - just data
   - Include metadata and navigation

2. **JSON Structure**
   ```go
   type ArchitectureJSON struct {
       Metadata    MetadataJSON
       Architecture ArchitectureData  // Systems, containers, etc.
       Navigation  NavigationJSON
   }
   ```

### Phase 2: JavaScript Library (Cytoscape.js-based)

1. **Create JS Library** (`sruja-viewer.js`)
   - Location: Separate repo or `learn/static/js/`
   - Uses **Cytoscape.js** for rendering (via CDN)
   - **Fixed library** - works for all Sruja architectures
   - Features:
     - Parse architecture data from JSON
     - **Cytoscape graph rendering** (nodes + edges)
     - **Built-in layouts** (hierarchical, dagre, breadthfirst, cose)
     - View management
     - Navigation
     - Search/filter
     - Zoom/pan (built-in)
     - Interactive elements (click, hover - built-in)
     - Documentation panel
     - Animations and transitions

2. **Library API**
   ```javascript
   SrujaViewer.init({
     container: '#sruja-app',
     data: './architecture.json',
     options: {
       theme: 'default',
       defaultView: 'level1',
       enableSearch: true,
       layout: 'hierarchical', // or 'dagre', 'breadthfirst', 'cose'
       animations: true
     }
   });
   ```

### Phase 3: HTML Template

1. **Minimal HTML Generator**
   - `pkg/export/html/html.go`
   - Generate minimal HTML
   - Link to CDN JS/CSS
   - Reference JSON file

2. **HTML Options**
   - Standalone: JSON embedded in HTML (single file)
   - Split: Separate HTML + JSON files
   - CDN mode: Use CDN for JS/CSS

### Phase 4: CDN Setup

1. **CDN Strategy**
   - Option A: npm + unpkg (Recommended)
   - Option B: jsDelivr
   - Option C: GitHub Releases
   - Option D: Self-hosted CDN

2. **Versioning**
   - Semantic versioning
   - Multiple versions available
   - Latest version alias

## Rendering: Cytoscape.js (Recommended)

### Why Cytoscape.js Instead of D2?

**D2 (Current):**
- Declarative diagramming language
- Requires compilation (D2 → SVG via WASM)
- **WASM overhead** (~500 KB+)
- Limited interactivity
- Fixed layouts
- Server-side compilation needed

**Cytoscape.js (Proposed):**
- ✅ **No D2 dependency** - Eliminates WASM completely
- ✅ **Direct rendering** from JSON data (no compilation step)
- ✅ **Purpose-built for graphs** - Perfect for architecture diagrams
- ✅ **Built-in layouts** - hierarchical, dagre, breadthfirst, cose
- ✅ **Better interactivity** - Built-in zoom, pan, click, hover
- ✅ **Better performance** - Optimized for graph rendering
- ✅ **Smaller bundle** - ~150 KB vs D2 WASM ~500 KB+
- ✅ **CDN available** - unpkg/jsDelivr
- ✅ **No compilation** - Pure JavaScript rendering

### Cytoscape.js Implementation

1. **Layout Algorithms (Built-in)**
   - **hierarchical**: Perfect for C4 model (systems → containers → components)
   - **dagre**: For flow diagrams and DAGs
   - **breadthfirst**: For tree structures
   - **cose**: Force-directed for relationship visualization

2. **Rendering Pipeline**
   ```
   JSON Data → Cytoscape Elements → Layout Calculation → SVG/Canvas Rendering → Interactive Features
   ```

3. **Cytoscape Features Used**
   - Graph rendering (nodes + edges)
   - Built-in layouts (no need to implement)
   - Built-in zoom/pan
   - Built-in event handling (click, hover, etc.)
   - Extensions (cytoscape-dagre for layouts)
   - Styling (CSS-like selectors)

## File Size Comparison

### Current (Embedded SVG)
- Single SVG: ~500 KB - 2 MB
- Everything embedded

### Cytoscape.js-based Rendering (Recommended)
- HTML: ~5-10 KB
- JSON: ~20-100 KB (just data)
- Cytoscape.js: ~150 KB (CDN, cached globally)
- cytoscape-dagre: ~50 KB (CDN, cached globally)
- Sruja Viewer: ~50-150 KB (loaded once, cached)
- **First load**: ~275-460 KB
- **Subsequent loads**: ~75-160 KB (Cytoscape + Viewer cached)
- **WASM eliminated**: No D2 WASM (~500 KB+ saved)

**Note**: Cytoscape.js is cached globally, and users likely already have it cached from other sites.

## Implementation Details

### JSON Exporter

```go
// pkg/export/json/json.go
type Exporter struct{}

func (e *Exporter) Export(arch *language.Architecture) (string, error) {
    // 1. Structure architecture data
    // 2. Include metadata
    // 3. Include navigation info
    // 4. NO SVG - just data
    // 5. Return JSON string
}
```

### HTML Exporter

```go
// pkg/export/html/html.go
type Exporter struct{}

func (e *Exporter) Export(arch *language.Architecture, jsonPath string) (string, error) {
    // Generate minimal HTML
    // Reference JSON file
    // Link to CDN JS/CSS
}
```

### JavaScript Library Structure

```
sruja-viewer/
├── src/
│   ├── viewer.js         # Main viewer (Cytoscape wrapper)
│   ├── data-parser.js    # Parse JSON → Cytoscape elements
│   ├── layouts.js        # Layout configuration (hierarchical, dagre, etc.)
│   ├── styles.js         # Cytoscape styling (CSS-like)
│   ├── interactions.js   # Event handlers (click, hover, etc.)
│   ├── views.js          # View management (level1, level2, scenarios, etc.)
│   ├── navigation.js     # Navigation
│   ├── search.js         # Search/filter
│   └── ui.js             # UI components (sidebar, panels, etc.)
├── styles/
│   └── viewer.css        # Styles
└── dist/
    ├── sruja-viewer.js
    └── sruja-viewer.css
```

**Dependencies (CDN):**
- Cytoscape.js v3.27+ (~150 KB)
  - Core graph rendering
  - Built-in layouts
  - Built-in interactions
- cytoscape-dagre (~50 KB)
  - Additional layout algorithms
  - Flow diagram support

## CLI Command

```bash
# Generate HTML + JSON (data only)
sruja export html architecture.sruja

# Output:
# - architecture.html (references CDN JS)
# - architecture.json (architecture data, no SVG)

# Or generate standalone (JSON embedded)
sruja export html --standalone architecture.sruja
# Output:
# - architecture.html (JSON embedded in <script> tag)
```

## Recommendation: Data-Only JSON

**Store architecture data only in JSON**, let JS library generate SVG:

1. ✅ **Smallest file size** (~20-100 KB vs 50-200 KB with SVG)
2. ✅ **Most flexible** - can change rendering without regenerating
3. ✅ **True separation** - data vs presentation
4. ✅ **Future-proof** - can add new rendering modes easily

**Implementation (D3-based)**:
- JSON: Architecture structure (systems, containers, relations, etc.)
- JS: Uses D3.js to generate SVG directly from data
- D3 layouts: Hierarchical (C4), force-directed, tree, dagre
- D3 interactions: Zoom, pan, click, hover, animations

## Cytoscape.js Advantages Over D2

1. **No D2 Dependency**
   - D2: Requires WASM (~500 KB+), compilation step
   - Cytoscape: Pure JavaScript, no WASM, no compilation

2. **No Compilation Step**
   - D2: Data → D2 Script → WASM Compile → SVG
   - Cytoscape: Data → Graph (direct rendering)

3. **Better Interactivity**
   - Built-in zoom, pan, drag
   - Built-in click, hover events
   - Smooth animations
   - Better than D2 for interactive graphs

4. **Built-in Layouts**
   - Hierarchical (perfect for C4 model)
   - Dagre (for flows)
   - Breadthfirst (for trees)
   - Cose (force-directed)
   - No need to implement layouts

5. **Smaller Bundle**
   - Cytoscape: ~150 KB
   - D2 WASM: ~500 KB+
   - **Saves ~350 KB+**

6. **Better Performance**
   - Optimized for graph rendering
   - Handles 1000+ nodes smoothly
   - Better than D2 for large graphs

7. **Purpose-Built**
   - Designed for graphs/networks
   - Perfect for architecture diagrams
   - Better than D2 for this use case

## Next Steps

1. Create JSON exporter (architecture data only)
2. Create minimal HTML exporter
3. Design JS library API (Cytoscape.js-based)
4. Implement JS library:
   - JSON → Cytoscape elements converter
   - Cytoscape layout configuration (hierarchical, dagre, etc.)
   - Cytoscape styling (CSS-like)
   - Event handlers (click, hover, etc.)
   - View management
5. Publish to npm/CDN
6. Update CLI commands
7. Update documentation

## Key Benefits Summary

✅ **No D2 dependency** - Eliminates WASM completely
✅ **No compilation** - Direct rendering from JSON
✅ **Much smaller** - ~275-460 KB vs 500 KB-2 MB (saves ~500 KB+ WASM)
✅ **Better performance** - Native JS, optimized for graphs
✅ **Fully interactive** - Built-in zoom, pan, click, hover
✅ **Fixed JS library** - One library works for all Sruja architectures
✅ **CDN delivery** - Fast, cached globally
