# SearchBar & SearchDialog Integration Guide

## Overview

This document describes the integration of SearchBar and SearchDialog components across the Sruja application ecosystem. These components provide unified search functionality with keyboard shortcuts and consistent UI patterns.

## Component Architecture

### Core Components

#### SearchBar (`packages/ui/src/components/SearchBar.tsx`)
- **Purpose**: Standalone search input with autocomplete functionality
- **Key Features**:
  - Built with Headless UI Combobox for accessibility
  - Supports loading states and custom placeholders
  - Keyboard navigation with arrow keys
  - Auto-focus on mount
  - Customizable styling with CSS variables

#### SearchDialog (`packages/ui/src/components/SearchDialog.tsx`)
- **Purpose**: Modal dialog wrapper for SearchBar with async result fetching
- **Key Features**:
  - Transition animations using Headless UI
  - Async result fetching with loading states
  - Click-outside-to-close functionality
  - Responsive design with max-width constraints

### Integration Points

#### 1. Studio Application (`apps/studio`)

**Implementation Location**: `apps/studio/src/App.tsx` (lines 253-255, 1350-1360)

**Usage Pattern**:
```typescript
const [searchDialogOpen, setSearchDialogOpen] = useState(false);

// Keyboard shortcut integration
useKeyboardShortcuts({
  onFind: () => setSearchDialogOpen(true),
});

// Component integration
<SearchDialog
  isOpen={searchDialogOpen}
  archData={archData}
  onSelect={(id) => {
    if (viewerRef.current) {
      viewerRef.current.selectNode(id);
      setSelectedNodeId(id);
    }
  }}
  onClose={() => setSearchDialogOpen(false)}
/>
```

**Features**:
- Searches through architecture elements (persons, systems, containers, components)
- Hierarchical path display (e.g., "System > Container > Component")
- Keyboard navigation with arrow keys and Enter
- Element selection and highlighting in diagram

#### 2. Learn Application (`apps/learn`)

**Implementation Location**: `apps/learn/assets/js/components/AppShell.tsx`

**Integration Pattern**: AppShell component provides navigation context for search functionality

#### 3. Storybook Documentation (`apps/storybook`)

**Implementation Location**: 
- `apps/storybook/src/stories/SearchBar.stories.tsx`
- `apps/storybook/src/stories/SearchDialog.stories.tsx`

**Purpose**: Component documentation and testing

## Technical Implementation

### Search Algorithm

The Studio's SearchDialog implements a comprehensive search algorithm:

```typescript
const searchTerm = query.toLowerCase();
const found: SearchResult[] = [];

// Search persons
if (arch.persons) {
  arch.persons.forEach((person) => {
    if (
      person.id.toLowerCase().includes(searchTerm) ||
      person.label?.toLowerCase().includes(searchTerm)
    ) {
      found.push({
        id: person.id,
        label: person.label || person.id,
        type: 'person',
        path: person.label || person.id,
      });
    }
  });
}

// Search systems and nested elements
if (arch.systems) {
  arch.systems.forEach((system) => {
    // Search containers within systems
    if (system.containers) {
      system.containers.forEach((container) => {
        if (
          container.id.toLowerCase().includes(searchTerm) ||
          container.label?.toLowerCase().includes(searchTerm)
        ) {
          found.push({
            id: `${system.id}.${container.id}`,
            label: container.label || container.id,
            type: 'container',
            path: `${system.label || system.id} > ${container.label || container.id}`,
          });
        }
      });
    }
  });
}
```

### Styling System

Components use CSS variables for theming:

```css
--color-text-primary: #1e293b
--color-text-tertiary: #64748b
--color-border: #e2e8f0
--color-background: #ffffff
--color-surface: #f8fafc
```

### Type Safety

```typescript
export type SearchItem = {
  id: string
  label: string
  subLabel?: string
}

export type SearchBarProps = {
  query: string
  onQueryChange: (q: string) => void
  results: SearchItem[]
  loading?: boolean
  onSelect: (item: SearchItem | null) => void
  placeholder?: string
  className?: string
}
```

## Usage Patterns

### Basic SearchBar Integration

```typescript
import { SearchBar } from '@sruja/ui'

function MyComponent() {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<SearchItem[]>([])

  return (
    <SearchBar
      query={query}
      onQueryChange={setQuery}
      results={results}
      onSelect={(item) => console.log('Selected:', item)}
      placeholder="Search items..."
    />
  )
}
```

### SearchDialog with Async Results

```typescript
import { SearchDialog } from '@sruja/ui'

function MyApp() {
  const [isOpen, setIsOpen] = useState(false)

  const fetchResults = async (query: string): Promise<SearchItem[]> => {
    const response = await fetch(`/api/search?q=${query}`)
    return response.json()
  }

  return (
    <SearchDialog
      isOpen={isOpen}
      onClose={() => setIsOpen(false)}
      fetchResults={fetchResults}
      onSelect={(item) => console.log('Selected:', item)}
    />
  )
}
```

## Keyboard Shortcuts

### Global Shortcuts
- **Cmd/Ctrl + K**: Open command palette (Studio)
- **Cmd/Ctrl + F**: Open search dialog (Studio)

### Search Dialog Shortcuts
- **↑/↓**: Navigate through results
- **Enter**: Select highlighted result
- **Escape**: Close dialog

## Best Practices

### Performance
1. **Debouncing**: Implement query debouncing for async searches
2. **Result Limiting**: Limit search results to prevent UI performance issues
3. **Memoization**: Cache search results when appropriate

### UX Guidelines
1. **Loading States**: Always show loading indicators during async operations
2. **Empty States**: Provide helpful messages when no results are found
3. **Keyboard Navigation**: Ensure full keyboard accessibility
4. **Visual Feedback**: Use consistent highlighting and selection indicators

### Integration Checklist
- [ ] Import components from `@sruja/ui`
- [ ] Implement keyboard shortcuts
- [ ] Handle loading and error states
- [ ] Style with CSS variables for theme consistency
- [ ] Test keyboard navigation
- [ ] Add appropriate TypeScript types

## Testing

### Unit Tests
- Component rendering and prop handling
- Keyboard event handling
- Async result fetching
- Loading state management

### Integration Tests
- Keyboard shortcut functionality
- Cross-component communication
- Theme consistency
- Accessibility compliance

## Future Enhancements

### Planned Features
1. **Advanced Filtering**: Filter by element type, date, or custom criteria
2. **Search History**: Remember recent searches
3. **Fuzzy Matching**: Implement fuzzy search algorithms
4. **Search Analytics**: Track popular search terms
5. **Saved Searches**: Allow users to save complex queries

### API Considerations
- Consider implementing server-side search for large datasets
- Add search result ranking/scoring
- Support for search result pagination
- Integration with external search services (Algolia, Elasticsearch)

## Troubleshooting

### Common Issues

1. **Search Results Not Updating**
   - Verify `archData` is properly passed to SearchDialog
   - Check that search algorithm covers all element types
   - Ensure re-rendering triggers when data changes

2. **Keyboard Shortcuts Not Working**
   - Confirm shortcut registration in `useKeyboardShortcuts` hook
   - Check for conflicting global shortcuts
   - Verify event listener cleanup

3. **Styling Inconsistencies**
   - Ensure CSS variables are properly defined
   - Check for conflicting global styles
   - Verify component-specific class names

### Debug Tools
- Browser DevTools for keyboard event monitoring
- React DevTools for component state inspection
- Console logging for search algorithm debugging