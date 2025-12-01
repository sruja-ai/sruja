# Task 4.1: Studio Core (UI Framework)

**Priority**: 🟢 Medium (Nice to have)
**Technology**: React + TypeScript
**Estimated Time**: 3-5 days
**Dependencies**: Task 3.1 (needs viewer library)

## Technology Stack

- **React 19** - UI framework
- **TypeScript** - Type safety
- **Tailwind CSS** - Styling (already configured)
- **Radix UI** - Component primitives (already installed)
- **Cytoscape.js** - Graph rendering (via Sruja Viewer library)

## Files to Create

* `learn/assets/js/components/Studio.tsx` - Main Studio component
* `learn/assets/js/components/studio/` - Studio-specific components
  * `StudioCanvas.tsx` - Cytoscape canvas wrapper
  * `StudioSidebar.tsx` - Element palette
  * `StudioPropertyPanel.tsx` - Property editor
  * `StudioToolbar.tsx` - Toolbar with actions
* `learn/content/studio/_index.md` - Studio page route
* `learn/layouts/studio.html` - Studio layout template

## Component Structure

```typescript
// Studio.tsx
export function Studio() {
  const [architecture, setArchitecture] = useState<ArchitectureJSON | null>(null);
  const [selectedElement, setSelectedElement] = useState<string | null>(null);
  
  // Handle URL parameters for PR previews
  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search);
    
    // Load preview snapshot
    if (urlParams.has('preview')) {
      const previewName = urlParams.get('preview');
      loadPreviewSnapshot(previewName);
    }
    
    // Load change preview
    if (urlParams.has('change')) {
      const changeId = urlParams.get('change');
      loadChangePreview(changeId);
    }
    
    // Load PR preview (auto-detect changes)
    if (urlParams.has('pr')) {
      const prNumber = urlParams.get('pr');
      loadPRPreview(prNumber);
    }
  }, []);
  
  const loadPreviewSnapshot = async (previewUrl: string) => {
    // Handle different URL formats
    let url = previewUrl;
    
    // Expand github:// convention to GitHub Pages URL
    if (previewUrl.startsWith('github://')) {
      const [, org, repo, , prNumber] = previewUrl.split('/');
      url = `https://${org}.github.io/${repo}/previews/pr-${prNumber}/preview.json`;
    }
    
    // Load from customer's GitHub Pages (or other public URL)
    try {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`Failed to load preview: ${response.statusText}`);
      }
      const json = await response.json();
      setArchitecture(json);
    } catch (error) {
      console.error('Failed to load preview:', error);
      // Show error message to user
      alert(`Failed to load preview from ${url}. Make sure the preview is deployed to GitHub Pages.`);
    }
  };
  
  return (
    <div className="studio-container">
      <StudioToolbar onLoad={handleLoadJSON} />
      <div className="studio-main">
        <StudioSidebar onElementSelect={handleAddElement} />
        <StudioCanvas 
          architecture={architecture}
          selectedElement={selectedElement}
          onElementSelect={setSelectedElement}
        />
        {selectedElement && (
          <StudioPropertyPanel 
            element={selectedElement}
            onUpdate={handleUpdateElement}
          />
        )}
      </div>
    </div>
  );
}
```

## Features

* Load JSON architecture (file upload or paste)
* **URL parameter support**: Load previews/changes directly from URL
  - `?preview=pr-123` - Load preview snapshot
  - `?change=003-add-analytics` - Load change preview
  - `?pr=123` - Load PR preview (auto-detects changes)
* Render with Sruja Viewer (React wrapper around Cytoscape)
* Sidebar with element palette (drag-and-drop ready)
* Property panel (edits selected element)
* Toolbar (add, delete, edit, save, export DSL, export PNG, export SVG)
* **File construct visualization**: Visual grouping by source file (using metadata.sourceFile)
* **File boundaries**: Show/hide elements by file, color-code by file

## Integration with Existing Learn Site

The Studio will be integrated into the existing Hugo + React setup:
- Uses existing React/TypeScript configuration
- Follows existing component patterns (see `Playground.tsx`)
- Uses existing UI components from `assets/js/components/ui/`

## Acceptance Criteria

* [ ] Studio UI loads as React component
* [ ] Can load JSON (file upload or paste)
* [ ] Viewer renders correctly via React wrapper
* [ ] UI is responsive (Tailwind CSS)
* [ ] Follows existing code patterns

## Getting Started

- Launch local Studio: `sruja studio --port 5173` and open `http://localhost:5173`
- Load JSON via file upload or paste in the Studio toolbar
- For file operations (read/write change files) see `go/STUDIO_API.md`
