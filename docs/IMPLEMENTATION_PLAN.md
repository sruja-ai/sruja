# Implementation Plan: Complete Sruja Export & Studio

## Overview

This plan covers the complete implementation of:
1. **DSL → JSON** - Export Sruja DSL to JSON format
2. **JSON → DSL** - Reverse engineering JSON back to DSL
3. **JSON → HTML** - Generate interactive diagrams from JSON
4. **Web Studio** - Visual diagram editor (no backend, pure client-side)

## Architecture

```
┌─────────────┐
│  Sruja DSL  │
└──────┬──────┘
       │
       ├─────────────────┐
       │                 │
       ▼                 ▼
┌─────────────┐    ┌─────────────┐
│   Parser    │    │   Printer   │
│ (DSL→AST)  │    │ (AST→DSL)   │
└──────┬──────┘    └──────┬──────┘
       │                  │
       │                  │
       ▼                  │
┌─────────────┐           │
│     AST     │◄──────────┘
└──────┬──────┘
       │
       ├─────────────────┐
       │                 │
       ▼                 ▼
┌─────────────┐    ┌─────────────┐
│ JSON Exporter│    │JSON→AST Conv│
│ (AST→JSON)  │    │ (JSON→AST)  │
└──────┬──────┘    └──────┬──────┘
       │                  │
       │                  │
       ▼                  │
┌─────────────┐           │
│    JSON     │◄──────────┘
└──────┬──────┘
       │
       ├─────────────────┐
       │                 │
       ▼                 ▼
┌─────────────┐    ┌─────────────┐
│ HTML Exporter│   │  Web Studio │
│ (JSON→HTML) │   │ (JSON↔UI)   │
└──────┬──────┘    └──────┬──────┘
       │                  │
       ▼                  ▼
┌─────────────┐    ┌─────────────┐
│  .sruja.html│    │  Diagram UI │
│ (Interactive)│   │  (Cytoscape)│
└─────────────┘    └──────┬──────┘
                          │
                          ▼
                   ┌─────────────┐
                   │ Export DSL  │
                   │ (via Printer)│
                   └─────────────┘
```

## Phase Breakdown

### Phase 1: Core Data Transformations (Foundation)
**Goal**: Enable DSL ↔ JSON round-trip

### Phase 2: HTML Export (Interactive Diagrams)
**Goal**: Generate interactive HTML from JSON

### Phase 3: JavaScript Library (Cytoscape.js)
**Goal**: Reusable rendering library for all architectures

### Phase 4: Web Studio (Visual Editor)
**Goal**: Client-side diagram editor with DSL export

---

## Phase 1: Core Data Transformations

### Task 1.1: JSON Exporter (AST → JSON)
**Priority**: 🔴 Critical (Blocks everything)
**Estimated Time**: 2-3 days
**Dependencies**: None (uses existing AST)

**Files to Create:**
- `pkg/export/json/json.go` - Main exporter
- `pkg/export/json/json_types.go` - JSON type definitions
- `pkg/export/json/json_test.go` - Comprehensive tests

**Implementation:**
```go
// pkg/export/json/json.go
type Exporter struct{}

func (e *Exporter) Export(arch *language.Architecture) ([]byte, error) {
    // Convert AST to JSON structure
    jsonData := ArchitectureJSON{
        Metadata: MetadataJSON{
            Name:      arch.Name,
            Version:   "1.0.0",
            Generated: time.Now().Format(time.RFC3339),
        },
        Architecture: convertArchitectureToJSON(arch),
        Navigation:   buildNavigation(arch),
    }
    return json.MarshalIndent(jsonData, "", "  ")
}
```

**JSON Structure:**
```json
{
  "metadata": {
    "name": "Architecture Name",
    "version": "1.0.0",
    "generated": "2025-01-XX"
  },
  "architecture": {
    "imports": [
      {
        "path": "shared.sruja",
        "alias": null
      },
      {
        "path": "billing.sruja",
        "alias": "Billing"
      }
    ],
    "systems": [...],
    "persons": [...],
    "relations": [...],
    "domains": [...],
    "scenarios": [...],
    "flows": [...],
    "requirements": [...],
    "adrs": [...],
    "deployment": [...],
    "contracts": [...],
    "sharedArtifacts": [...],
    "libraries": [...],
    "policies": [...],
    "constraints": [...],
    "conventions": [...],
    "views": [...]
  },
  "navigation": {
    "levels": ["level1", "level2", "level3"],
    "scenarios": [...],
    "flows": [...],
    "domains": [...]
  }
}
```

**Note**: `imports` are at the top of `architecture` object (matching DSL structure where imports appear first in architecture block).

**Test Coverage:**
- ✅ All element types (systems, containers, components, etc.)
- ✅ **Imports** (with and without alias)
- ✅ All DDD elements (domains, contexts, aggregates, etc.)
- ✅ All relations and flows
- ✅ All metadata and properties
- ✅ Round-trip tests (DSL → JSON → DSL) - **imports must be preserved**

**Acceptance Criteria:**
- [ ] All AST types can be exported to JSON
- [ ] JSON structure matches specification
- [ ] 90%+ test coverage
- [ ] Round-trip tests pass

---

### Task 1.2: JSON to AST Converter (JSON → AST)
**Priority**: 🔴 Critical (Blocks reverse engineering)
**Estimated Time**: 2-3 days
**Dependencies**: Task 1.1 (needs JSON structure)

**Files to Create:**
- `pkg/language/json_to_ast.go` - Main converter
- `pkg/language/json_to_ast_test.go` - Comprehensive tests

**Implementation:**
```go
// pkg/language/json_to_ast.go
func JSONToAST(jsonData []byte) (*Architecture, error) {
    var archJSON ArchitectureJSON
    if err := json.Unmarshal(jsonData, &archJSON); err != nil {
        return nil, err
    }
    
    arch := &Architecture{
        Name: archJSON.Metadata.Name,
    }
    
    // Convert imports FIRST (matching DSL structure)
    arch.Imports = convertImportsJSONToAST(archJSON.Architecture.Imports)
    
    // Convert all elements
    arch.Systems = convertSystemsJSONToAST(archJSON.Architecture.Systems)
    arch.Persons = convertPersonsJSONToAST(archJSON.Architecture.Persons)
    arch.Relations = convertRelationsJSONToAST(archJSON.Architecture.Relations)
    // ... all other elements
    
    return arch, nil
}

func convertImportsJSONToAST(importsJSON []ImportJSON) []*ImportSpec {
    imports := make([]*ImportSpec, 0, len(importsJSON))
    for _, impJSON := range importsJSON {
        imp := &ImportSpec{
            Path: impJSON.Path,
        }
        if impJSON.Alias != nil {
            alias := *impJSON.Alias
            imp.Alias = &alias
        }
        imports = append(imports, imp)
    }
    return imports
}
```

**Test Coverage:**
- ✅ All JSON types can be converted to AST
- ✅ **Imports preserved** (path and alias)
- ✅ All element types preserved
- ✅ All metadata preserved
- ✅ Round-trip tests (JSON → AST → JSON)
- ✅ Round-trip tests (DSL → JSON → DSL) - **imports must match**
- ✅ Edge cases (missing fields, null values)
- ✅ Import with alias
- ✅ Import without alias

**Acceptance Criteria:**
- [ ] All JSON types can be converted to AST
- [ ] **Imports are preserved** (path and alias)
- [ ] Round-trip tests pass (JSON → AST → JSON)
- [ ] Round-trip tests pass (DSL → JSON → DSL) - **imports must match**
- [ ] 90%+ test coverage
- [ ] Handles missing/null fields gracefully
- [ ] Import statements appear first in generated DSL (matching DSL structure)

---

### Task 1.3: CLI Commands
**Priority**: 🟡 High (User-facing)
**Estimated Time**: 1 day
**Dependencies**: Task 1.1, Task 1.2

**Files to Create/Modify:**
- `cmd/sruja/json.go` - JSON export command
- `cmd/sruja/json_to_dsl.go` - JSON to DSL command

**Commands:**
```bash
# Export DSL to JSON
sruja export json <input.sruja> <output.json>

# Convert JSON to DSL
sruja json-to-dsl <input.json> <output.sruja>

# Round-trip test
sruja export json input.sruja output.json
sruja json-to-dsl output.json output2.sruja
diff input.sruja output2.sruja  # Should match (formatting may differ)

# Test with imports
cat > test.sruja <<EOF
architecture "Test" {
  import "shared.sruja"
  import "billing.sruja" as Billing
  system API "API Service" {}
}
EOF

sruja export json test.sruja test.json
sruja json-to-dsl test.json test2.sruja
# test2.sruja should have imports preserved at the top
grep -q 'import "shared.sruja"' test2.sruja
grep -q 'import "billing.sruja" as Billing' test2.sruja
```

**Acceptance Criteria:**
- [ ] `export json` command works
- [ ] `json-to-dsl` command works
- [ ] Commands handle errors gracefully
- [ ] Help text is clear

---

### Task 1.4: Modularization Command (Split Large Files)
**Priority**: 🟡 High (Developer Experience)
**Estimated Time**: 3-5 days
**Dependencies**: Task 1.1 (needs JSON), Task 1.2 (needs JSON-to-AST)

**Files to Create:**
- `pkg/refactor/modularize.go` - Modularization logic
- `pkg/refactor/splitter.go` - File splitting strategies
- `cmd/sruja/modularize.go` - CLI command

**Command:**
```bash
# Split by system (each system becomes its own file)
sruja modularize --strategy=system <input.sruja> <output-dir>

# Split by domain (each domain becomes its own file)
sruja modularize --strategy=domain <input.sruja> <output-dir>

# Split by feature (custom grouping)
sruja modularize --strategy=feature <input.sruja> <output-dir>

# Split with shared/common elements
sruja modularize --strategy=system --shared <input.sruja> <output-dir>
```

**Strategies:**

1. **By System** (`--strategy=system`)
   - Each system → separate file
   - Shared elements (persons, shared artifacts, etc.) → `shared.sruja`
   - Main file imports all system files
   - Relations preserved (cross-file references)

2. **By Domain** (`--strategy=domain`)
   - Each domain → separate file
   - Shared elements → `shared.sruja`
   - Main file imports all domain files

3. **By Feature** (`--strategy=feature`)
   - Groups related systems/domains together
   - Uses metadata/tags to determine grouping
   - Custom grouping rules

**Implementation:**
```go
// pkg/refactor/modularize.go
type Modularizer struct {
    Strategy string // "system", "domain", "feature"
    Shared   bool   // Extract shared elements
}

func (m *Modularizer) Modularize(arch *language.Architecture, outputDir string) error {
    switch m.Strategy {
    case "system":
        return m.splitBySystem(arch, outputDir)
    case "domain":
        return m.splitByDomain(arch, outputDir)
    case "feature":
        return m.splitByFeature(arch, outputDir)
    default:
        return fmt.Errorf("unknown strategy: %s", m.Strategy)
    }
}

func (m *Modularizer) splitBySystem(arch *language.Architecture, outputDir string) error {
    // 1. Extract shared elements (persons, shared artifacts, etc.)
    shared := extractSharedElements(arch)
    
    // 2. Create shared.sruja
    if len(shared) > 0 {
        sharedFile := createSharedFile(shared)
        writeFile(filepath.Join(outputDir, "shared.sruja"), sharedFile)
    }
    
    // 3. Split each system into its own file
    for _, sys := range arch.Systems {
        sysFile := createSystemFile(sys, shared)
        filename := fmt.Sprintf("%s.sruja", sys.ID)
        writeFile(filepath.Join(outputDir, filename), sysFile)
    }
    
    // 4. Create main.sruja that imports all system files
    mainFile := createMainFile(arch, arch.Systems)
    writeFile(filepath.Join(outputDir, "main.sruja"), mainFile)
    
    return nil
}

func createSystemFile(sys *language.System, shared *SharedElements) string {
    var sb strings.Builder
    
    // Import shared if needed
    if shared != nil && len(shared.Elements) > 0 {
        sb.WriteString("import \"shared.sruja\"\n\n")
    }
    
    // System definition
    printer := language.NewPrinter()
    // ... print system with its containers, components, etc.
    
    return sb.String()
}

func createMainFile(arch *language.Architecture, systems []*language.System) string {
    var sb strings.Builder
    
    sb.WriteString(fmt.Sprintf("architecture %q {\n", arch.Name))
    
    // Import shared
    if hasSharedElements(arch) {
        sb.WriteString("  import \"shared.sruja\"\n")
    }
    
    // Import all system files
    for _, sys := range systems {
        sb.WriteString(fmt.Sprintf("  import \"%s.sruja\"\n", sys.ID))
    }
    
    // Top-level relations (cross-system)
    for _, rel := range arch.Relations {
        if isCrossSystem(rel, systems) {
            sb.WriteString(fmt.Sprintf("  %s -> %s", rel.From, rel.To))
            if rel.Verb != nil {
                sb.WriteString(fmt.Sprintf(" %s", *rel.Verb))
            }
            if rel.Label != nil {
                sb.WriteString(fmt.Sprintf(" %q", *rel.Label))
            }
            sb.WriteString("\n")
        }
    }
    
    sb.WriteString("}\n")
    return sb.String()
}
```

**Example Output:**

**Input:** `monolith.sruja`
```sruja
architecture "E-commerce Platform" {
  person Customer "Customer"
  person Admin "Admin"
  
  system Shop "Shop System" {
    container WebApp "Web App" {}
    container API "API" {}
    datastore DB "Database" {}
  }
  
  system Payment "Payment System" {
    container Gateway "Payment Gateway" {}
  }
  
  system Inventory "Inventory System" {
    container Service "Inventory Service" {}
  }
  
  Customer -> Shop "Uses"
  Shop -> Payment "Processes payment"
  Shop -> Inventory "Checks stock"
}
```

**Output after `sruja modularize --strategy=system monolith.sruja ./modules`:**

**`modules/shared.sruja`**
```sruja
person Customer "Customer"
person Admin "Admin"
```

**`modules/Shop.sruja`**
```sruja
import "shared.sruja"

system Shop "Shop System" {
  container WebApp "Web App" {}
  container API "API" {}
  datastore DB "Database" {}
}
```

**`modules/Payment.sruja`**
```sruja
import "shared.sruja"

system Payment "Payment System" {
  container Gateway "Payment Gateway" {}
}
```

**`modules/Inventory.sruja`**
```sruja
import "shared.sruja"

system Inventory "Inventory System" {
  container Service "Inventory Service" {}
}
```

**`modules/main.sruja`**
```sruja
import "shared.sruja"
import "Shop.sruja"
import "Payment.sruja"
import "Inventory.sruja"

architecture "E-commerce Platform" {
  Customer -> Shop "Uses"
  Shop -> Payment "Processes payment"
  Shop -> Inventory "Checks stock"
}
```

**Test Coverage:**
- ✅ Split by system
- ✅ Split by domain
- ✅ Shared elements extraction
- ✅ Import generation
- ✅ Cross-file relations preserved
- ✅ Round-trip (modularize → combine → original)

**Acceptance Criteria:**
- [ ] Can split by system
- [ ] Can split by domain
- [ ] Shared elements extracted correctly
- [ ] Imports generated correctly
- [ ] Cross-file relations preserved
- [ ] Generated files are valid Sruja DSL
- [ ] Can combine split files back to original (round-trip)

---

## Phase 2: HTML Export

### Task 2.1: HTML Exporter (JSON → HTML)
**Priority**: 🟡 High (User-facing)
**Estimated Time**: 1-2 days
**Dependencies**: Task 1.1 (needs JSON structure)

**Files to Create:**
- `pkg/export/html/html.go` - HTML exporter
- `pkg/export/html/template.go` - HTML template

**Implementation:**
```go
// pkg/export/html/html.go
type Exporter struct {
    JSONPath string // Path to JSON file (relative or absolute)
    CDN      bool   // Use CDN for JS/CSS
}

func (e *Exporter) Export(jsonData []byte, outputPath string) error {
    // Generate minimal HTML
    html := generateHTML(e.JSONPath, e.CDN)
    return os.WriteFile(outputPath, []byte(html), 0644)
}
```

**HTML Template:**
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Architecture: {{.Name}}</title>
  <!-- Sruja Viewer from sruja.ai (GitHub Pages with custom domain) -->
  <link rel="stylesheet" href="https://sruja.ai/static/js/sruja-viewer.css">
</head>
<body>
  <div id="sruja-app"></div>
  <!-- Cytoscape.js from unpkg CDN -->
  <script src="https://unpkg.com/cytoscape@3.27.0/dist/cytoscape.min.js"></script>
  <script src="https://unpkg.com/cytoscape-dagre@2.5.0/cytoscape-dagre.js"></script>
  <!-- Sruja Viewer from sruja.ai -->
  <script src="https://sruja.ai/static/js/sruja-viewer.js"></script>
  <script>
    SrujaViewer.init({
      container: '#sruja-app',
      data: './{{.JSONFile}}'
    });
  </script>
</body>
</html>
```

**Note**: Using `sruja.ai` custom domain (GitHub Pages):
- **URL**: `https://sruja.ai/static/js/sruja-viewer.js`
- **Path**: `learn/static/js/sruja-viewer.js` (served via GitHub Pages)
- **Benefits**: 
  - ✅ Branded domain (`sruja.ai` instead of `sruja-ai.github.io`)
  - ✅ Free (GitHub Pages)
  - ✅ Easy to set up (already configured)
  - ✅ Version controlled
  - ✅ HTTPS by default
- **Future**: Can migrate to dedicated CDN later if needed

**Acceptance Criteria:**
- [ ] Generates valid HTML
- [ ] Links to JSON file correctly
- [ ] Supports CDN mode
- [ ] Supports embedded JSON mode (optional)

---

### Task 2.2: CLI Command for HTML Export
**Priority**: 🟡 High (User-facing)
**Estimated Time**: 0.5 days
**Dependencies**: Task 2.1

**Command:**
```bash
# Export to HTML (generates .sruja.html)
sruja export html <input.sruja> <output.sruja.html>

# Or from JSON
sruja export html-json <input.json> <output.sruja.html>
```

**Acceptance Criteria:**
- [ ] Command works
- [ ] Generates valid HTML
- [ ] HTML opens in browser correctly

---

## Phase 3: JavaScript Library (Cytoscape.js)

### Task 3.1: Sruja Viewer Library (Core)
**Priority**: 🔴 Critical (Blocks HTML export)
**Estimated Time**: 5-7 days
**Dependencies**: Task 1.1 (needs JSON structure)

**Files to Create:**
- `learn/static/js/sruja-viewer.js` - Main library (hosted on GitHub Pages)
- `learn/static/css/sruja-viewer.css` - Styles (hosted on GitHub Pages)
- `learn/static/js/sruja-viewer.test.js` - Tests (optional)

**GitHub Pages Setup:**
- Files in `learn/static/js/` are served via GitHub Pages
- **URL**: `https://sruja.ai/static/js/sruja-viewer.js` (custom domain)
- Custom domain `sruja.ai` already configured (see `learn/CNAME`)
- Files are version-controlled and automatically deployed

**Library Structure:**
```javascript
// sruja-viewer.js
class SrujaViewer {
  constructor(options) {
    this.container = options.container;
    this.data = options.data;
    this.cy = null; // Cytoscape instance
  }
  
  async init() {
    // Load JSON data
    const data = await this.loadData();
    
    // Convert JSON to Cytoscape elements
    const elements = this.convertToCytoscape(data);
    
    // Initialize Cytoscape
    this.cy = cytoscape({
      container: document.querySelector(this.container),
      elements: elements,
      style: this.getStyle(),
      layout: this.getLayout(),
    });
    
    // Setup interactions
    this.setupInteractions();
    
    // Setup views
    this.setupViews(data);
  }
  
  convertToCytoscape(data) {
    // Convert JSON architecture to Cytoscape nodes + edges
    const nodes = [];
    const edges = [];
    
    // Systems as nodes
    data.architecture.systems.forEach(sys => {
      nodes.push({
        data: { id: sys.id, label: sys.label, type: 'system' }
      });
    });
    
    // Relations as edges
    data.architecture.relations.forEach(rel => {
      edges.push({
        data: { 
          id: `${rel.from}-${rel.to}`,
          source: rel.from,
          target: rel.to,
          label: rel.label
        }
      });
    });
    
    return [...nodes, ...edges];
  }
  
  setupViews(data) {
    // Level 1, Level 2, Level 3 views
    // Scenarios, Flows, Domains, etc.
  }
  
  setupInteractions() {
    // Click, hover, zoom, pan
    // Drill-down navigation
    // Documentation panel
  }
}
```

**Features:**
- ✅ Load JSON data (file or embedded)
- ✅ Convert JSON to Cytoscape elements
- ✅ Render graph with Cytoscape.js
- ✅ View switching (Level 1, 2, 3, scenarios, flows, etc.)
- ✅ Zoom/pan (built-in)
- ✅ Click/hover interactions
- ✅ Drill-down navigation
- ✅ Documentation panel
- ✅ Search functionality

**Acceptance Criteria:**
- [ ] Loads JSON correctly
- [ ] Renders all element types
- [ ] View switching works
- [ ] Interactions work (click, hover, zoom, pan)
- [ ] Documentation panel works
- [ ] Works in all modern browsers

---

### Task 3.2: Layout Configuration
**Priority**: 🟡 High (User experience)
**Estimated Time**: 2-3 days
**Dependencies**: Task 3.1

**Implementation:**
```javascript
getLayout(viewType) {
  switch(viewType) {
    case 'level1':
    case 'level2':
    case 'level3':
      return { name: 'hierarchical', ... };
    case 'scenario':
    case 'flow':
      return { name: 'dagre', ... };
    case 'domain':
      return { name: 'breadthfirst', ... };
    default:
      return { name: 'cose', ... };
  }
}
```

**Acceptance Criteria:**
- [ ] Each view type has appropriate layout
- [ ] Layouts look good
- [ ] Performance is acceptable (1000+ nodes)

---

### Task 3.3: Styling (D2-like Appearance)
**Priority**: 🟡 High (User experience)
**Estimated Time**: 2-3 days
**Dependencies**: Task 3.1

**Implementation:**
```javascript
getStyle() {
  return [
    {
      selector: 'node[type="system"]',
      style: {
        'shape': 'round-rectangle',
        'background-color': '#4A90E2',
        'label': 'data(label)',
        'width': 'label',
        'height': 'label',
        'padding': '10px'
      }
    },
    {
      selector: 'node[type="container"]',
      style: {
        'shape': 'round-rectangle',
        'background-color': '#7ED321',
        // ...
      }
    },
    // ... more styles
  ];
}
```

**Acceptance Criteria:**
- [ ] Styles match D2 appearance (or close)
- [ ] All element types styled correctly
- [ ] Colors are consistent
- [ ] Labels are readable

---

### Task 3.4: View Management
**Priority**: 🟡 High (User experience)
**Estimated Time**: 2-3 days
**Dependencies**: Task 3.1

**Features:**
- Level 1, 2, 3 views
- Scenario views
- Flow views
- Domain/Context/Aggregate views
- Requirements/ADRs views
- Deployment views
- Custom views

**Acceptance Criteria:**
- [ ] All view types work
- [ ] View switching is smooth
- [ ] Views filter nodes/edges correctly

---

### Task 3.5: Interactivity (Click, Hover, Drill-down)
**Priority**: 🟡 High (User experience)
**Estimated Time**: 2-3 days
**Dependencies**: Task 3.1

**Features:**
- Click to select element
- Hover to highlight
- Click to drill-down
- Documentation panel
- Search functionality

**Acceptance Criteria:**
- [ ] All interactions work
- [ ] Drill-down navigation works
- [ ] Documentation panel shows correct content
- [ ] Search finds elements

---

## Phase 4: Web Studio (Visual Editor)

### Task 4.1: Studio Core (UI Framework)
**Priority**: 🟢 Medium (Nice to have)
**Estimated Time**: 3-5 days
**Dependencies**: Task 3.1 (needs viewer library)

**Files to Create:**
- `learn/studio/index.html` - Studio page
- `learn/static/js/studio.js` - Studio logic
- `learn/static/css/studio.css` - Studio styles

**Features:**
- Load JSON architecture
- Render with Sruja Viewer
- Sidebar with element palette
- Property panel
- Toolbar (add, delete, edit)

**Acceptance Criteria:**
- [ ] Studio UI loads
- [ ] Can load JSON
- [ ] Viewer renders correctly
- [ ] UI is responsive

---

### Task 4.2: Drag-and-Drop Editor
**Priority**: 🟢 Medium (Nice to have)
**Estimated Time**: 5-7 days
**Dependencies**: Task 4.1

**Features:**
- Drag elements from palette
- Drop on canvas
- Create relations by connecting nodes
- Edit element properties
- Delete elements

**Implementation:**
```javascript
// Use Cytoscape's built-in node creation
studio.cy.on('tap', 'node', (evt) => {
  // Show property panel
  showPropertyPanel(evt.target);
});

// Add node
function addNode(type, position) {
  const id = generateID(type);
  studio.cy.add({
    data: { id, type, label: `New ${type}` },
    position: position
  });
  updateJSON(); // Update JSON data
}
```

**Acceptance Criteria:**
- [ ] Can add elements
- [ ] Can create relations
- [ ] Can edit properties
- [ ] Can delete elements
- [ ] Changes update JSON

---

### Task 4.3: DSL Export from Studio
**Priority**: 🟢 Medium (Nice to have)
**Estimated Time**: 2-3 days
**Dependencies**: Task 4.2, Task 1.2

**Features:**
- Export JSON to DSL
- Download .sruja file
- Real-time DSL preview

**Implementation:**
```javascript
// In browser: JSON → AST → DSL
async function exportToDSL() {
  const jsonData = studio.getJSON();
  
  // Use WebAssembly or JS implementation of JSON → AST → DSL
  // Or call backend API (if available)
  // For now: Use JS implementation
  
  const ast = jsonToAST(jsonData);
  const dsl = printAST(ast);
  
  downloadFile('architecture.sruja', dsl);
}
```

**Options:**
1. **Pure JS** - Implement JSON → AST → DSL in JavaScript (port Go code)
2. **WebAssembly** - Compile Go code to WASM
3. **Backend API** - Call backend (but user wants no backend)

**Recommendation**: Start with pure JS implementation (simpler, no backend needed)

**Acceptance Criteria:**
- [ ] Can export to DSL
- [ ] DSL is valid
- [ ] Can download .sruja file
- [ ] DSL preview works

---

### Task 4.4: Studio Polish
**Priority**: 🟢 Low (Nice to have)
**Estimated Time**: 2-3 days
**Dependencies**: Task 4.3

**Features:**
- Undo/redo
- Save/load (localStorage)
- Keyboard shortcuts
- Help/documentation
- Export options (JSON, DSL, HTML)

**Acceptance Criteria:**
- [ ] Undo/redo works
- [ ] Save/load works
- [ ] Keyboard shortcuts work
- [ ] Export options work

---

## Task Dependencies & Parallelization

### Dependency Graph

```
Phase 1: Core Data Transformations
├── Task 1.1: JSON Exporter (AST → JSON)
│   └── No dependencies
│
├── Task 1.2: JSON to AST Converter (JSON → AST)
│   └── Depends on: Task 1.1 (needs JSON structure)
│
├── Task 1.3: CLI Commands
│   └── Depends on: Task 1.1, Task 1.2
│
└── Task 1.4: Modularization Command
    └── Depends on: Task 1.1, Task 1.2 (needs JSON and JSON-to-AST)

Phase 2: HTML Export
├── Task 2.1: HTML Exporter
│   └── Depends on: Task 1.1 (needs JSON structure)
│
└── Task 2.2: CLI Command for HTML
    └── Depends on: Task 2.1

Phase 3: JavaScript Library
├── Task 3.1: Sruja Viewer Library (Core)
│   └── Depends on: Task 1.1 (needs JSON structure)
│
├── Task 3.2: Layout Configuration
│   └── Depends on: Task 3.1
│
├── Task 3.3: Styling
│   └── Depends on: Task 3.1
│
├── Task 3.4: View Management
│   └── Depends on: Task 3.1
│
└── Task 3.5: Interactivity
    └── Depends on: Task 3.1

Phase 4: Web Studio
├── Task 4.1: Studio Core
│   └── Depends on: Task 3.1 (needs viewer library)
│
├── Task 4.2: Drag-and-Drop Editor
│   └── Depends on: Task 4.1
│
├── Task 4.3: DSL Export from Studio
│   └── Depends on: Task 4.2, Task 1.2
│
└── Task 4.4: Studio Polish
    └── Depends on: Task 4.3
```

### Parallelization Opportunities

**Can be done in parallel:**

1. **Task 1.1 (JSON Exporter) + Task 3.1 (Viewer Library)**
   - JSON Exporter defines JSON structure
   - Viewer Library can start implementing with mock JSON
   - Sync when JSON structure is finalized

2. **Task 1.3 (CLI Commands) + Task 1.4 (Modularization)**
   - Both depend on Task 1.1 and 1.2
   - Can be done in parallel once JSON round-trip works

3. **Task 3.2 (Layouts) + Task 3.3 (Styling) + Task 3.4 (Views) + Task 3.5 (Interactions)**
   - All depend on Task 3.1 (Core)
   - Can be done in parallel once core is ready

4. **Task 2.1 (HTML Exporter) + Task 3.1 (Viewer Library)**
   - HTML Exporter is simple (just template)
   - Can be done in parallel

**Must be sequential:**

1. **Task 1.1 → Task 1.2 → Task 1.3** (Core data transformations)
2. **Task 3.1 → Task 3.2/3.3/3.4/3.5** (Viewer library features)
3. **Task 4.1 → Task 4.2 → Task 4.3 → Task 4.4** (Studio features)

---

## Recommended Implementation Order

### Sprint 1: Foundation (Week 1-2)
**Goal**: Enable DSL ↔ JSON round-trip

1. ✅ **Task 1.1: JSON Exporter** (2-3 days)
   - Start immediately
   - Blocks everything else
   
2. ✅ **Task 1.2: JSON to AST Converter** (2-3 days)
   - Start after Task 1.1 JSON structure is defined
   - Can start with basic structure, iterate
   
3. ✅ **Task 1.3: CLI Commands** (1 day)
   - Start after Task 1.1 and 1.2 are done
   - Quick win, user-facing

4. ✅ **Task 1.4: Modularization Command** (3-5 days)
   - Start after Task 1.1 and 1.2 are done
   - Can be done in parallel with Task 1.3
   - Developer experience improvement

**Deliverable**: 
- `sruja export json` and `sruja json-to-dsl` commands work
- `sruja modularize` command works (split large files)

---

### Sprint 2: HTML Export (Week 2-3)
**Goal**: Generate interactive HTML from JSON

1. ✅ **Task 2.1: HTML Exporter** (1-2 days)
   - Start in parallel with Task 3.1 (simple template)
   
2. ✅ **Task 2.2: CLI Command** (0.5 days)
   - Quick after Task 2.1

**Deliverable**: `sruja export html` command works (but diagrams not interactive yet)

---

### Sprint 3: JavaScript Library Core (Week 3-5)
**Goal**: Interactive diagram rendering

1. ✅ **Task 3.1: Sruja Viewer Library (Core)** (5-7 days)
   - Start as soon as Task 1.1 JSON structure is defined
   - Use mock JSON initially, sync later
   - **Critical path** - blocks everything else

**Deliverable**: Basic interactive diagrams work

---

### Sprint 4: JavaScript Library Features (Week 5-7)
**Goal**: Complete viewer functionality

1. ✅ **Task 3.2: Layout Configuration** (2-3 days) - Parallel
2. ✅ **Task 3.3: Styling** (2-3 days) - Parallel
3. ✅ **Task 3.4: View Management** (2-3 days) - Parallel
4. ✅ **Task 3.5: Interactivity** (2-3 days) - Parallel

**Deliverable**: Full-featured interactive diagrams

---

### Sprint 5: Web Studio (Week 7-10)
**Goal**: Visual diagram editor

1. ✅ **Task 4.1: Studio Core** (3-5 days)
2. ✅ **Task 4.2: Drag-and-Drop Editor** (5-7 days)
3. ✅ **Task 4.3: DSL Export** (2-3 days)
4. ✅ **Task 4.4: Studio Polish** (2-3 days)

**Deliverable**: Complete web studio with DSL export

---

## Timeline Summary

| Sprint | Duration | Tasks | Deliverable |
|--------|----------|-------|-------------|
| **Sprint 1** | Week 1-2 | 1.1, 1.2, 1.3, 1.4 | DSL ↔ JSON round-trip + Modularization |
| **Sprint 2** | Week 2-3 | 2.1, 2.2 | HTML export (basic) |
| **Sprint 3** | Week 3-5 | 3.1 | Interactive diagrams (basic) |
| **Sprint 4** | Week 5-7 | 3.2, 3.3, 3.4, 3.5 | Interactive diagrams (complete) |
| **Sprint 5** | Week 7-10 | 4.1, 4.2, 4.3, 4.4 | Web studio |

**Total Estimated Time**: 10 weeks (2.5 months)

---

## Critical Path

**Must be done in order:**
1. Task 1.1 (JSON Exporter) - **BLOCKS EVERYTHING**
2. Task 1.2 (JSON to AST) - Blocks reverse engineering
3. Task 3.1 (Viewer Core) - Blocks HTML export and studio
4. Task 4.1 (Studio Core) - Blocks studio features

**Can be parallelized:**
- Task 1.1 + Task 3.1 (with mock JSON initially)
- Task 2.1 + Task 3.1
- Task 3.2/3.3/3.4/3.5 (all after 3.1)
- Task 4.2/4.3/4.4 (all after 4.1)

---

## Success Criteria

### Phase 1: Core Data Transformations ✅
- [ ] DSL → JSON works for all element types
- [ ] JSON → DSL works for all element types
- [ ] Round-trip tests pass (DSL → JSON → DSL)
- [ ] CLI commands work
- [ ] 90%+ test coverage

### Phase 2: HTML Export ✅
- [ ] HTML export generates valid files
- [ ] HTML opens in browser
- [ ] Links to JSON correctly

### Phase 3: JavaScript Library ✅
- [ ] Loads and renders JSON
- [ ] All view types work
- [ ] Interactions work (click, hover, zoom, pan)
- [ ] Documentation panel works
- [ ] Search works
- [ ] Works in all modern browsers

### Phase 4: Web Studio ✅
- [ ] Can load JSON
- [ ] Can add/edit/delete elements
- [ ] Can create relations
- [ ] Can export to DSL
- [ ] Can export to JSON
- [ ] Can export to HTML
- [ ] Works offline (no backend)

---

## Next Steps

1. **Start with Task 1.1** (JSON Exporter) - Critical path
2. **Define JSON structure** - Document in `docs/JSON_SCHEMA.md`
3. **Create JSON schema** - For validation
4. **Set up test infrastructure** - For comprehensive testing
5. **Begin Task 3.1** (Viewer Library) - In parallel with mock JSON

---

## Notes

- **No Backend Required**: Web studio is pure client-side (JSON → AST → DSL in JS)
- **CDN Strategy**: JS/CSS from CDN, JSON can be embedded or separate file
- **Testing**: JSON approach is much more testable (see `TESTABILITY_COMPARISON.md`)
- **Performance**: Cytoscape.js handles 1000+ nodes smoothly
- **Offline Support**: Can bundle JS locally for offline use

