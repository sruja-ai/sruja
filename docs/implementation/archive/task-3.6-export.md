# Task 3.6: Export Functionality (SVG/PNG)

**Priority**: 🟡 High (User-facing feature)
**Technology**: TypeScript/JavaScript
**Estimated Time**: 2-3 days
**Dependencies**: Task 3.1 (needs viewer library)

## Overview

Add export functionality to export architecture diagrams as SVG and PNG files. This will be available in both:
1. Standalone HTML viewer (via SrujaViewer API)
2. Studio (React component)

## Implementation

### Viewer Library Export Methods

```typescript
// packages/viewer/src/viewer.ts
import { SrujaViewer } from '@sruja/viewer';

// The SrujaViewer class already exists in packages/viewer/
// Add export methods to it:
  // ... existing code ...
  
  /**
   * Export current view as PNG
   * @param options - Export options (width, height, scale, etc.)
   * @returns Promise<Blob> - PNG image blob
   */
  async exportPNG(options = {}) {
    const {
      width = this.cy.width(),
      height = this.cy.height(),
      scale = 1,
      bg = '#ffffff',
      full = false
    } = options;
    
    return this.cy.png({
      output: 'blob',
      bg: bg,
      full: full,
      scale: scale,
      maxWidth: width,
      maxHeight: height
    });
  }
  
  /**
   * Export current view as SVG
   * @param options - Export options
   * @returns Promise<Blob> - SVG blob
   */
  async exportSVG(options = {}) {
    const {
      full = false
    } = options;
    
    const svgString = this.cy.svg({
      full: full
    });
    
    return new Blob([svgString], { type: 'image/svg+xml' });
  }
  
  /**
   * Download exported image
   * @param blob - Image blob (PNG or SVG)
   * @param filename - Output filename
   */
  downloadImage(blob: Blob, filename: string) {
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  }
  
  /**
   * Export as PNG and download
   */
  async exportAndDownloadPNG(filename?: string) {
    const blob = await this.exportPNG();
    const name = filename || `architecture-${Date.now()}.png`;
    this.downloadImage(blob, name);
  }
  
  /**
   * Export as SVG and download
   */
  async exportAndDownloadSVG(filename?: string) {
    const blob = await this.exportSVG();
    const name = filename || `architecture-${Date.now()}.svg`;
    this.downloadImage(blob, name);
  }
}
```

### Standalone HTML UI

Add export buttons to the viewer UI:

```html
<!-- Auto-generated in HTML or added via viewer -->
<div class="sruja-viewer-controls">
  <button onclick="viewer.exportAndDownloadPNG()">Export PNG</button>
  <button onclick="viewer.exportAndDownloadSVG()">Export SVG</button>
</div>
```

### Export Options

```typescript
interface ExportOptions {
  // PNG options
  width?: number;        // Output width
  height?: number;       // Output height
  scale?: number;        // Scale factor (for high-DPI)
  bg?: string;          // Background color
  full?: boolean;       // Export full graph (including off-screen)
  
  // SVG options
  full?: boolean;       // Export full graph
  
  // Common options
  filename?: string;    // Custom filename
}
```

## Features

* ✅ Export current view as PNG
* ✅ Export current view as SVG
* ✅ Export full graph (including off-screen elements)
* ✅ High-resolution export (scale factor)
* ✅ Custom filename
* ✅ Automatic download
* ✅ Background color customization (PNG)
* ✅ Works in standalone HTML
* ✅ Works in Studio

## Acceptance Criteria

* [ ] Can export as PNG from standalone HTML
* [ ] Can export as SVG from standalone HTML
* [ ] Can export full graph (not just viewport)
* [ ] Export buttons appear in viewer UI
* [ ] High-resolution export works (scale > 1)
* [ ] Custom filename works
* [ ] Background color can be customized
* [ ] Exported images are valid and open correctly

## Notes

- Cytoscape.js has built-in PNG/SVG export capabilities
- PNG export requires canvas, SVG export is native
- Full export (`full: true`) includes all nodes/edges, not just visible viewport
- High-DPI displays benefit from `scale: 2` or higher
