# Task 3.1: Sruja Viewer Library (Core)

**Priority**: 🔴 Critical (Blocks HTML export)
**Technology**: TypeScript/JavaScript
**Estimated Time**: 5-7 days
**Dependencies**: Task 1.1 (needs JSON structure)

## Files to Create

* `learn/static/js/sruja-viewer.js` - Main library (hosted on GitHub Pages)
* `learn/static/css/sruja-viewer.css` - Styles (hosted on GitHub Pages)
* `learn/static/js/sruja-viewer.test.js` - Tests (optional)

## GitHub Pages Setup

* Files in `learn/static/js/` are served via GitHub Pages
* **URL**: `https://sruja.ai/static/js/sruja-viewer.js` (custom domain)
* Custom domain `sruja.ai` already configured (see `learn/CNAME`)
* Files are version-controlled and automatically deployed

## Library Structure

```javascript
// sruja-viewer.js
class SrujaViewer {
  constructor(options) {
    this.container = options.container;
    this.data = options.data;
    this.cy = null; // Cytoscape instance
  }
  
  async init() {
    // Check URL parameters for preview/change/PR
    const urlParams = new URLSearchParams(window.location.search);
    
    let data;
    if (urlParams.has('preview')) {
      // Load preview snapshot
      const previewName = urlParams.get('preview');
      data = await this.loadPreviewSnapshot(previewName);
    } else if (urlParams.has('change')) {
      // Load change preview
      const changeId = urlParams.get('change');
      data = await this.loadChangePreview(changeId);
    } else if (urlParams.has('pr')) {
      // Load PR preview
      const prNumber = urlParams.get('pr');
      data = await this.loadPRPreview(prNumber);
    } else if (urlParams.has('diff')) {
      // Load diff view
      const diff = urlParams.get('diff'); // e.g., "main...pr-123"
      data = await this.loadDiff(diff);
    } else {
      // Load JSON data normally
      data = await this.loadData();
    }
    
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

// Global API
window.SrujaViewer = SrujaViewer;
```

## Features

* ✅ Load JSON data (file or embedded)
* ✅ Convert JSON to Cytoscape elements
* ✅ Render graph with Cytoscape.js
* ✅ View switching (Level 1, 2, 3, scenarios, flows, etc.)
* ✅ Zoom/pan (built-in)
* ✅ Click/hover interactions
* ✅ Drill-down navigation
* ✅ Documentation panel
* ✅ Search functionality

## Acceptance Criteria

* [ ] Loads JSON correctly
* [ ] Renders all element types
* [ ] View switching works
* [ ] Interactions work (click, hover, zoom, pan)
* [ ] Documentation panel works
* [ ] Works in all modern browsers
