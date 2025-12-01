# Task 4.5: Studio Export Functionality (SVG/PNG)

**Priority**: 🟡 High (User-facing feature)
**Technology**: React + TypeScript
**Estimated Time**: 1-2 days
**Dependencies**: Task 3.6 (viewer export methods), Task 4.1 (studio core)

## Overview

Add export buttons to Studio toolbar for exporting architecture diagrams as SVG and PNG files.

## Implementation

### Studio Toolbar with Export Buttons

```typescript
// components/studio/StudioToolbar.tsx
import { Download } from 'lucide-react';

export function StudioToolbar({ 
  architecture,
  cy, // Cytoscape instance
  onLoad,
  onExportDSL 
}: StudioToolbarProps) {
  const handleExportPNG = async () => {
    if (!cy) return;
    
    // Use viewer export methods or direct Cytoscape API
    const blob = await cy.png({
      output: 'blob',
      full: true, // Export full graph
      scale: 2,   // High resolution
      bg: '#ffffff'
    });
    
    downloadFile(
      `architecture-${architecture?.metadata.name || 'diagram'}-${Date.now()}.png`,
      blob
    );
  };
  
  const handleExportSVG = async () => {
    if (!cy) return;
    
    const svgString = cy.svg({ full: true });
    const blob = new Blob([svgString], { type: 'image/svg+xml' });
    
    downloadFile(
      `architecture-${architecture?.metadata.name || 'diagram'}-${Date.now()}.svg`,
      blob
    );
  };
  
  return (
    <div className="studio-toolbar flex items-center gap-2 p-2 border-b">
      {/* Load button */}
      <button onClick={onLoad}>Load JSON</button>
      
      {/* Export buttons */}
      <div className="flex gap-1 ml-auto">
        <button 
          onClick={handleExportPNG}
          className="flex items-center gap-1"
          disabled={!cy}
        >
          <Download className="w-4 h-4" />
          Export PNG
        </button>
        
        <button 
          onClick={handleExportSVG}
          className="flex items-center gap-1"
          disabled={!cy}
        >
          <Download className="w-4 h-4" />
          Export SVG
        </button>
        
        <button 
          onClick={onExportDSL}
          disabled={!architecture}
        >
          Export DSL
        </button>
      </div>
    </div>
  );
}
```

### Export Options Dialog

```typescript
// components/studio/ExportDialog.tsx
export function ExportDialog({ 
  open, 
  onClose, 
  onExport 
}: ExportDialogProps) {
  const [format, setFormat] = useState<'png' | 'svg'>('png');
  const [full, setFull] = useState(true);
  const [scale, setScale] = useState(2);
  const [bgColor, setBgColor] = useState('#ffffff');
  
  return (
    <Dialog open={open} onClose={onClose}>
      <DialogTitle>Export Diagram</DialogTitle>
      
      <div className="space-y-4">
        <div>
          <label>Format</label>
          <Select value={format} onValueChange={(v) => setFormat(v as 'png' | 'svg')}>
            <option value="png">PNG</option>
            <option value="svg">SVG</option>
          </Select>
        </div>
        
        <div>
          <label>
            <input 
              type="checkbox" 
              checked={full}
              onChange={(e) => setFull(e.target.checked)}
            />
            Export full diagram (not just visible area)
          </label>
        </div>
        
        {format === 'png' && (
          <>
            <div>
              <label>Scale (for high-resolution)</label>
              <input 
                type="number" 
                value={scale}
                onChange={(e) => setScale(Number(e.target.value))}
                min="1"
                max="4"
                step="0.5"
              />
            </div>
            
            <div>
              <label>Background Color</label>
              <input 
                type="color" 
                value={bgColor}
                onChange={(e) => setBgColor(e.target.value)}
              />
            </div>
          </>
        )}
        
        <div className="flex gap-2 justify-end">
          <button onClick={onClose}>Cancel</button>
          <button onClick={() => onExport({ format, full, scale, bgColor })}>
            Export
          </button>
        </div>
      </div>
    </Dialog>
  );
}
```

### Utility Functions

```typescript
// utils/export.ts
export function downloadFile(filename: string, blob: Blob) {
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

export async function exportCytoscapeToPNG(
  cy: cytoscape.Core,
  options: {
    full?: boolean;
    scale?: number;
    bg?: string;
  } = {}
): Promise<Blob> {
  return cy.png({
    output: 'blob',
    full: options.full ?? true,
    scale: options.scale ?? 2,
    bg: options.bg ?? '#ffffff'
  });
}

export function exportCytoscapeToSVG(
  cy: cytoscape.Core,
  options: {
    full?: boolean;
  } = {}
): Blob {
  const svgString = cy.svg({
    full: options.full ?? true
  });
  
  return new Blob([svgString], { type: 'image/svg+xml' });
}
```

## Features

* ✅ Export as PNG from Studio
* ✅ Export as SVG from Studio
* ✅ Export full diagram (not just viewport)
* ✅ High-resolution PNG export (configurable scale)
* ✅ Background color customization (PNG)
* ✅ Export options dialog
* ✅ Custom filename based on architecture name
* ✅ Disabled state when no diagram loaded

## Acceptance Criteria

* [ ] Export PNG button works in Studio
* [ ] Export SVG button works in Studio
* [ ] Exported images match current view
* [ ] Full export includes all elements
* [ ] High-resolution export works
* [ ] Export options dialog works
* [ ] Filenames are meaningful (include architecture name)
* [ ] Export buttons are disabled when no diagram loaded

## Integration with Viewer Library

Studio can either:
1. Use SrujaViewer export methods directly (if viewer is used)
2. Use Cytoscape API directly (if managing Cytoscape instance separately)

Both approaches work - choose based on how Studio integrates with the viewer library.
