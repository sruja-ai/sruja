# Task 3.2: Layout Configuration

**Priority**: 🟡 High (User experience)
**Technology**: TypeScript
**Estimated Time**: 2-3 days
**Dependencies**: Task 3.1

## Implementation

```typescript
function getLayout(viewType: string): cytoscape.LayoutOptions {
  switch (viewType) {
    case 'level1':
    case 'level2':
    case 'level3':
      return { name: 'hierarchical' } as cytoscape.LayoutOptions;
    case 'scenario':
    case 'flow':
      return { name: 'dagre' } as cytoscape.LayoutOptions;
    case 'domain':
      return { name: 'breadthfirst' } as cytoscape.LayoutOptions;
    default:
      return { name: 'cose' } as cytoscape.LayoutOptions;
  }
}
```

## Layout Strategies

- **Level 1/2/3**: Hierarchical layouts
- **Scenarios/Flows**: DAG layouts (dagre)
- **Domains**: Breadth-first layouts
- **Default**: Force-directed (cose)

## Acceptance Criteria

* [ ] Each view type has appropriate layout
* [ ] Layouts look good
* [ ] Performance is acceptable (1000+ nodes)
