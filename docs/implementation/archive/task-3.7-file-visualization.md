# Task 3.7: File Boundary Visualization

**Priority**: 🟢 Medium (Nice to have)
**Technology**: TypeScript + React
**Estimated Time**: 2-3 days
**Dependencies**: Task 3.1 (needs viewer library)

## Overview

Add visualization of logical file organization based on JSON structure. Since file names are standardized, we can group elements by logical category (architecture, requirements, stories, etc.) rather than tracking actual file paths.

## Design

File organization is determined by JSON structure:
- Architecture elements (systems, containers, components) → `architecture.sruja`
- Requirements → `requirements.sruja`
- User stories → `stories.sruja`
- ADRs → `decisions.sruja`
- Scenarios → `scenarios.sruja`

Studio/viewer can visualize these logical groupings.

## Implementation

### Visual File Organization

```typescript
// In Studio or Viewer
function visualizeFileOrganization(architecture: ArchitectureJSON) {
  // Group elements by logical file category
  const fileGroups = {
    architecture: {
      name: 'architecture.sruja',
      elements: [
        ...architecture.architecture.systems,
        ...architecture.architecture.containers,
        ...architecture.architecture.components,
        ...architecture.architecture.persons,
        ...architecture.architecture.relations,
        ...architecture.architecture.flows,
        // ... other architecture elements
      ]
    },
    requirements: {
      name: 'requirements.sruja',
      elements: architecture.requirements || []
    },
    stories: {
      name: 'stories.sruja',
      elements: architecture.userStories || []
    },
    decisions: {
      name: 'decisions.sruja',
      elements: architecture.adrs || []
    },
    scenarios: {
      name: 'scenarios.sruja',
      elements: architecture.scenarios || []
    }
  };
  
  // Visualize with:
  // 1. Color coding by logical file category
  // 2. Grouping/clustering by category
  // 3. File legend/panel showing logical organization
  // 4. Toggle visibility by category
}
```

### File Panel/Legend

```typescript
// local-studio/src/components/studio/FilePanel.tsx
export function FilePanel({ architecture }: FilePanelProps) {
  const fileGroups = getFileGroups(architecture);
  
  return (
    <div className="file-panel">
      <h3>File Organization</h3>
      {Object.entries(fileGroups).map(([key, group]) => (
        <div 
          key={key}
          className="file-item"
          style={{ borderColor: getFileColor(key) }}
        >
          <span>{group.name}</span>
          <span>{group.elements.length} elements</span>
        </div>
      ))}
    </div>
  );
}
```

### Color Coding Elements

```typescript
// Style elements based on logical file category
function getElementStyle(element: Element, architecture: ArchitectureJSON) {
  const category = getElementCategory(element, architecture);
  const color = getFileColor(category);
  
  return {
    'background-color': color,
    'border-color': darken(color),
    // Visual indicator for shared elements
    'border-style': element.metadata?.shared ? 'dashed' : 'solid'
  };
}

function getElementCategory(element: Element, architecture: ArchitectureJSON): string {
  // Determine which logical file category this element belongs to
  if (isArchitectureElement(element, architecture)) return 'architecture';
  if (isRequirement(element, architecture)) return 'requirements';
  if (isUserStory(element, architecture)) return 'stories';
  if (isADR(element, architecture)) return 'decisions';
  if (isScenario(element, architecture)) return 'scenarios';
  return 'unknown';
}
```

### Shared Services Visualization

```typescript
// Show shared services (referenced via naming convention)
function visualizeSharedServices(architecture: ArchitectureJSON) {
  const sharedElements = findSharedElements(architecture);
  
  // Show:
  // - Which elements are shared (marked with metadata.shared: true)
  // - Shared service indicators
  // - Reference relationships
  sharedElements.forEach(element => {
    addSharedAnnotation(element, {
      name: element.id, // e.g., "shared.AuthService"
      referencedFrom: findReferencesToElement(architecture, element.id)
    });
  });
}
```

## Features

* ✅ Visual file organization (color coding by logical category)
* ✅ File panel/legend showing logical file structure
* ✅ Shared service indicators (dashed borders for shared elements)
* ✅ Filter by category (show/hide architecture/requirements/stories/etc.)
* ✅ Toggle visibility by logical file category

## Acceptance Criteria

* [ ] Elements can be color-coded by logical file category
* [ ] File panel shows logical file organization (architecture.sruja, requirements.sruja, etc.)
* [ ] Shared elements are visually distinct (dashed borders)
* [ ] Can filter view by category (architecture/requirements/stories/decisions/scenarios)
* [ ] Shared service references can be visualized
