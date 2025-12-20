# Task 3.1: Sruja Viewer Library (Core)

**Task Status**
- Status: Pending
- Owner: Unassigned
- Target Date: -
- Dependencies: Task 1.1
- Last Updated: 2025-12-01

**Priority**: 🔴 Critical (Blocks HTML export)
**Technology**: TypeScript + React (wrapper)
**Estimated Time**: 5-7 days
**Dependencies**: Task 1.1 (needs JSON structure)

## Files to Create

* `packages/viewer/src/viewer.ts` - Main viewer class (Cytoscape.js integration)
* `packages/viewer/src/types.ts` - TypeScript type definitions
* `packages/viewer/src/index.ts` - Package exports
* `packages/viewer/vite.config.ts` - Library build configuration
* `apps/learn/assets/js/components/Viewer.tsx` - React wrapper component (optional)
* `apps/studio/src/components/Viewer.tsx` - React wrapper component (optional)

## Package Structure

The viewer is a **shared library package** (`@sruja/viewer`) in the monorepo:

```
packages/viewer/
├── src/
│   ├── index.ts        # Main exports
│   ├── viewer.ts       # SrujaViewer class
│   └── types.ts        # Type definitions
├── dist/               # Build output
│   ├── index.js        # Compiled library
│   └── index.d.ts      # TypeScript declarations
├── package.json
└── vite.config.ts
```

## Build & Usage

### Building

```bash
# Build viewer package
npm run build --filter=@sruja/viewer

# Output: packages/viewer/dist/index.js
```

### Using in Apps

```typescript
// In Studio or Learn apps
import { createViewer, SrujaViewer } from '@sruja/viewer';

const viewer = createViewer({
  container: '#my-container',
  data: architectureJSON
});

await viewer.init();
```

### GitHub Pages Deployment

The viewer library is **not deployed separately**. It's:
- Built as part of the monorepo
- Bundled into Studio and Learn apps during their builds
- Available as an npm workspace package (`@sruja/viewer`)
- Imported directly in code (no CDN needed)

## Library Structure

```typescript
// sruja-viewer.ts
type ViewerOptions = {
  container: string;
  data?: ArchitectureJSON;
};

export class SrujaViewer {
  container: string;
  data?: ArchitectureJSON;
  cy: cytoscape.Core | null;

  constructor(options: ViewerOptions) {
    this.container = options.container;
    this.data = options.data;
    this.cy = null;
  }

  async init() {
    const urlParams = new URLSearchParams(window.location.search);

    let data: ArchitectureJSON;
    if (urlParams.has('preview')) {
      const previewName = urlParams.get('preview')!;
      data = await this.loadPreviewSnapshot(previewName);
    } else if (urlParams.has('change')) {
      const changeId = urlParams.get('change')!;
      data = await this.loadChangePreview(changeId);
    } else if (urlParams.has('pr')) {
      const prNumber = urlParams.get('pr')!;
      data = await this.loadPRPreview(prNumber);
    } else if (urlParams.has('diff')) {
      const diff = urlParams.get('diff')!; // e.g., "main...pr-123"
      data = await this.loadDiff(diff);
    } else if (this.data) {
      data = this.data;
    } else {
      data = await this.loadData();
    }

    const elements = this.convertToCytoscape(data);

    this.cy = cytoscape({
      container: document.querySelector(this.container) as HTMLDivElement,
      elements,
      style: this.getStyle(),
      layout: this.getLayout(),
    });

    this.setupInteractions();
    this.setupViews(data);
  }

  convertToCytoscape(data: ArchitectureJSON): cytoscape.ElementsDefinition {
    const nodes: cytoscape.NodeDefinition[] = [];
    const edges: cytoscape.EdgeDefinition[] = [];

    data.architecture.systems.forEach((sys) => {
      nodes.push({ data: { id: sys.id, label: sys.label, type: 'system' } });
    });

    data.architecture.relations.forEach((rel) => {
      edges.push({
        data: {
          id: `${rel.from}-${rel.to}`,
          source: rel.from,
          target: rel.to,
          label: rel.label,
        },
      });
    });

    return [...nodes, ...edges];
  }

  setupViews(data: ArchitectureJSON) {}
  setupInteractions() {}

  getStyle(): cytoscape.Stylesheet[] { return []; }
  getLayout(): cytoscape.LayoutOptions { return { name: 'cose' }; }

  async loadData(): Promise<ArchitectureJSON> { /* ... */ throw new Error('not implemented'); }
  async loadPreviewSnapshot(_: string): Promise<ArchitectureJSON> { /* ... */ throw new Error('not implemented'); }
  async loadChangePreview(_: string): Promise<ArchitectureJSON> { /* ... */ throw new Error('not implemented'); }
  async loadPRPreview(_: string): Promise<ArchitectureJSON> { /* ... */ throw new Error('not implemented'); }
  async loadDiff(_: string): Promise<ArchitectureJSON> { /* ... */ throw new Error('not implemented'); }
}

declare global {
  interface Window { SrujaViewer: typeof SrujaViewer }
}

window.SrujaViewer = SrujaViewer;
```

### React Wrapper Component

```tsx
// apps/studio/src/components/Viewer.tsx or apps/learn/assets/js/components/Viewer.tsx
import { createViewer } from '@sruja/viewer';
import { useEffect, useRef } from 'react';
import type { ArchitectureJSON } from '@sruja/viewer';

export function Viewer({ data }: { data?: ArchitectureJSON }) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!containerRef.current) return;
    const viewer = createViewer({ 
      container: containerRef.current, 
      data 
    });
    viewer.init();
    
    return () => {
      viewer.destroy();
    };
  }, [data]);

  return <div ref={containerRef} style={{ width: '100%', height: '100%' }} />;
}
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
* [ ] **Manual testing**: Test in Chrome, Firefox, Safari, Edge
* [ ] **Manual testing**: Test on mobile devices
* [ ] **MCP-based testing**: Verify semantic correctness of rendered diagrams
