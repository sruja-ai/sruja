# Task 3.2: Layout Configuration

**Priority**: 🟡 High (User experience)
**Technology**: TypeScript/JavaScript
**Estimated Time**: 2-3 days
**Dependencies**: Task 3.1

## Implementation

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

## Layout Strategies

- **Level 1/2/3**: Hierarchical layouts
- **Scenarios/Flows**: DAG layouts (dagre)
- **Domains**: Breadth-first layouts
- **Default**: Force-directed (cose)

## Acceptance Criteria

* [ ] Each view type has appropriate layout
* [ ] Layouts look good
* [ ] Performance is acceptable (1000+ nodes)
