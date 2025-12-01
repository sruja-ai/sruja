# Task 3.3: Styling (D2-like Appearance)

**Priority**: 🟡 High (User experience)
**Technology**: TypeScript/JavaScript
**Estimated Time**: 2-3 days
**Dependencies**: Task 3.1

## Implementation

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

## Acceptance Criteria

* [ ] Styles match D2 appearance (or close)
* [ ] All element types styled correctly
* [ ] Colors are consistent
* [ ] Labels are readable
