# Task 4.2: Drag-and-Drop Editor

**Priority**: 🟢 Medium (Nice to have)
**Technology**: React + TypeScript (`apps/studio/`)
**Estimated Time**: 5-7 days
**Dependencies**: Task 4.1

## Features

* Drag elements from palette
* Drop on canvas
* Create relations by connecting nodes
* Edit element properties
* Delete elements

## Implementation

### React Hook for Cytoscape

```typescript
// apps/studio/src/hooks/useCytoscape.ts
export function useCytoscape(containerRef: RefObject<HTMLDivElement>) {
  const [cy, setCy] = useState<cytoscape.Core | null>(null);
  
  useEffect(() => {
    if (!containerRef.current) return;
    
    const instance = cytoscape({
      container: containerRef.current,
      // ... config
    });
    
    setCy(instance);
    return () => instance.destroy();
  }, [containerRef]);
  
  return cy;
}
```

### Studio Canvas Component

```typescript
// apps/studio/src/components/studio/StudioCanvas.tsx
export function StudioCanvas({ 
  architecture, 
  onElementSelect 
}: StudioCanvasProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const cy = useCytoscape(containerRef);
  const [elements, setElements] = useState<ArchitectureJSON>(architecture);
  
  // Sync Cytoscape with React state
  useEffect(() => {
    if (!cy || !elements) return;
    
    // Convert JSON to Cytoscape elements
    const cytoscapeElements = convertToCytoscape(elements);
    cy.elements().remove();
    cy.add(cytoscapeElements);
  }, [cy, elements]);
  
  // Handle Cytoscape events
  useEffect(() => {
    if (!cy) return;
    
    cy.on('tap', 'node', (evt) => {
      onElementSelect(evt.target.id());
    });
    
    cy.on('add', 'node,edge', () => {
      // Update React state from Cytoscape
      const json = convertFromCytoscape(cy);
      setElements(json);
    });
    
    return () => {
      cy.off('tap');
      cy.off('add');
    };
  }, [cy, onElementSelect]);
  
  return <div ref={containerRef} className="studio-canvas" />;
}
```

### Element Palette with Drag-and-Drop

```typescript
// local-studio/src/components/studio/StudioSidebar.tsx
export function StudioSidebar({ onElementSelect }: StudioSidebarProps) {
  const elementTypes = ['system', 'container', 'component', 'person'];
  
  return (
    <div className="studio-sidebar">
      <h3>Elements</h3>
      {elementTypes.map(type => (
        <div
          key={type}
          draggable
          onDragStart={(e) => {
            e.dataTransfer.setData('element-type', type);
          }}
          className="palette-item"
        >
          {type}
        </div>
      ))}
    </div>
  );
}
```

### Canvas Drop Handler

```typescript
// Handle drop on canvas
const handleDrop = useCallback((e: DragEvent) => {
  e.preventDefault();
  const elementType = e.dataTransfer.getData('element-type');
  const position = cy.project({ x: e.clientX, y: e.clientY });
  
  // Add new element to Cytoscape
  const id = generateID(elementType);
  cy.add({
    data: { id, type: elementType, label: `New ${elementType}` },
    position
  });
  
  // State updates automatically via useEffect
}, [cy]);
```

## Acceptance Criteria

* [ ] Can add elements via drag-and-drop
* [ ] Can create relations by connecting nodes
* [ ] Can edit properties via property panel
* [ ] Can delete elements (keyboard or button)
* [ ] Changes update JSON state
* [ ] React state and Cytoscape stay in sync
