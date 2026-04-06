# AppShell Component Architecture and Adoption Guide

## Overview

AppShell is a layout component that provides consistent application structure across the Sruja ecosystem. It implements a standardized header-sidebar-main-footer layout pattern with theme support and responsive design.

## Core Component

### AppShell (`packages/ui/src/components/AppShell.tsx`)

```typescript
export type AppShellProps = {
  header?: any
  sidebar?: any
  children: any
  footer?: any
}

export function AppShell({ header, sidebar, children, footer }: AppShellProps) {
  return (
    <div className="min-h-screen bg-[var(--color-surface)] text-[var(--color-text-primary)]">
      {header && <div className="border-b border-[var(--color-border)] bg-[var(--color-background)]">{header}</div>}
      <div className="flex">
        {sidebar && <aside className="w-64 border-r border-[var(--color-border)] bg-[var(--color-background)]">{sidebar}</aside>}
        <main className="flex-1 p-6">{children}</main>
      </div>
      {footer && <div className="border-t border-[var(--color-border)] bg-[var(--color-background)]">{footer}</div>}
    </div>
  )
}
```

## Adoption in Studio Application

### Current Implementation Status

The Studio application has adopted AppShell architecture with custom implementation:

**Location**: `apps/studio/src/App.tsx` (lines 1022-1528)

**Structure**:
```typescript
<div className="app h-screen flex flex-col overflow-hidden">
  {/* Header */}
  <Header
    title="Sruja Studio"
    subtitle="Architecture Visualization Tool"
    version={formatVersion('0.1.0')}
    logoLoading={isWasmLoading}
    // ... additional props
  />
  
  {/* Main Layout */}
  <div className="flex flex-1 overflow-hidden">
    {/* Activity Bar (VSCode-style) */}
    <div className="w-12 bg-slate-800 flex flex-col items-center pt-2 border-r border-slate-700">
      {/* Navigation buttons */}
    </div>
    
    {/* Sidebar Panel (Explorer) */}
    {showExplorer && (
      <div className="bg-slate-50 border-r border-slate-200 flex flex-col relative" style={{ width: `${sidebarWidth}px` }}>
        {/* Resizable explorer panel */}
      </div>
    )}
    
    {/* Main Content */}
    <div className="flex flex-1 flex-col min-w-0">
      {/* Editor/Viewer Tabs */}
      {/* Content Area */}
    </div>
  </div>
  
  {/* Footer */}
  <Footer
    leftContent={<span>© {new Date().getFullYear()} Sruja</span>}
    centerContent={<span>Architecture as Code</span>}
    rightContent={<a href="https://sruja.ai" target="_blank" rel="noopener noreferrer">sruja.ai</a>}
  />
</div>
```

### Studio-Specific Enhancements

#### 1. Activity Bar Pattern
- **Purpose**: VSCode-style vertical navigation
- **Implementation**: Custom sidebar with icon-based navigation
- **Features**: Toggle buttons for Explorer and Properties panels

#### 2. Resizable Panels
- **Explorer Panel**: Width-adjustable sidebar (200-600px range)
- **Mouse Resize**: Drag-to-resize functionality with visual feedback
- **State Management**: `sidebarWidth` state with min/max constraints

#### 3. Tab-based Content Switching
- **Editor Tab**: Full Monaco editor view
- **Split Tab**: Side-by-side editor and viewer
- **Dynamic Tabs**: State-driven tab visibility

## Adoption in Learn Application

### Current Implementation

**Location**: `apps/learn/assets/js/components/AppShell.tsx`

**Implementation Pattern**:
```typescript
export function AppShell() {
  const section = getSection();
  useEffect(() => {
    if (section === 'playground') {
      document.body.classList.add('playground-full');
    }
  }, [section]);

  return (
    <TopNavigation section={section} />
  );
}
```

**Mounting System**:
```typescript
export function mountAppShell(): void {
  let container = document.getElementById('sruja-app-root');
  if (!container) {
    const el = document.createElement('div');
    el.id = 'sruja-app-root';
    document.body.insertBefore(el, document.body.firstChild);
    container = el;
  }
  if ((container as any)._mounted) return;
  const root = createRoot(container);
  (container as any)._mounted = true;
  root.render(<AppShell />);
}
```

### Learn-Specific Features

#### 1. Section-based Navigation
- **Dynamic Section Detection**: `getSection()` utility
- **Conditional Styling**: Playground-specific CSS classes
- **Hugo Integration**: Works with static site generation

#### 2. Progressive Enhancement
- **Non-blocking**: AppShell mounts after page load
- **Graceful Degradation**: Works without JavaScript
- **Mount Prevention**: Prevents duplicate mounting

## Integration Patterns

### Basic AppShell Integration

```typescript
import { AppShell, Header, Footer } from '@sruja/ui'

function MyApp() {
  return (
    <AppShell
      header={
        <Header
          title="My Application"
          subtitle="Application description"
          rightContent={<UserMenu />}
        />
      }
      sidebar={<NavigationMenu />}
      footer={
        <Footer
          leftContent="© 2024 My Company"
          centerContent="Built with Sruja"
          rightContent={<SocialLinks />}
        />
      }
    >
      <MainContent />
    </AppShell>
  )
}
```

### Custom Layout Extensions

#### Resizable Sidebar Pattern
```typescript
const [sidebarWidth, setSidebarWidth] = useState(280)

<div className="flex flex-1 overflow-hidden">
  <aside 
    className="border-r border-[var(--color-border)] bg-[var(--color-background)]"
    style={{ width: `${sidebarWidth}px` }}
  >
    {/* Sidebar content */}
  </aside>
  
  {/* Resize handle */}
  <div
    className="w-1 cursor-col-resize hover:bg-blue-200 transition-colors"
    onMouseDown={(e) => {
      // Implement resize logic
    }}
  />
  
  <main className="flex-1 p-6">
    {/* Main content */}
  </main>
</div>
```

#### Activity Bar Pattern
```typescript
<div className="flex flex-1 overflow-hidden">
  {/* Activity Bar */}
  <div className="w-12 bg-slate-800 flex flex-col items-center border-r border-slate-700">
    <button className="w-10 h-10 rounded flex items-center justify-center">
      <Icon size={20} />
    </button>
  </div>
  
  {/* Main content area */}
  <div className="flex flex-1">
    {/* Sidebar and main content */}
  </div>
</div>
```

## Styling System

### CSS Variables
```css
:root {
  --color-surface: #f8fafc;
  --color-background: #ffffff;
  --color-text-primary: #1e293b;
  --color-border: #e2e8f0;
}
```

### Theme Integration
- **Dark Mode**: Automatic theme switching support
- **Custom Themes**: Override CSS variables for branding
- **Responsive**: Mobile-first responsive design

## Best Practices

### 1. Component Composition
- Use AppShell as the root layout container
- Keep header/sidebar/footer content modular
- Avoid deep nesting of AppShell components

### 2. State Management
- Manage layout state at the application level
- Use context providers for shared layout state
- Implement proper cleanup for event listeners

### 3. Performance
- Lazy load sidebar content when possible
- Implement virtual scrolling for large lists
- Use CSS containment for layout performance

### 4. Accessibility
- Ensure keyboard navigation works throughout
- Provide proper ARIA labels for screen readers
- Maintain focus management during transitions

## Migration Guide

### From Custom Layouts

1. **Identify Layout Components**: Map existing header/sidebar/footer elements
2. **Extract Content**: Separate layout structure from content
3. **Implement AppShell**: Wrap content with AppShell component
4. **Migrate Styling**: Convert to CSS variable system
5. **Test Responsiveness**: Verify mobile and desktop layouts

### Common Migration Patterns

#### Legacy Header Migration
```typescript
// Before
<div className="custom-header">
  <h1>Title</h1>
  <nav>...</nav>
</div>

// After
<Header
  title="Title"
  leftContent={<Navigation />}
  rightContent={<UserActions />}
/>
```

#### Sidebar Migration
```typescript
// Before
<div className="sidebar">
  <ul className="nav">...</ul>
</div>

// After
<aside className="w-64 border-r border-[var(--color-border)]">
  <NavigationMenu />
</aside>
```

## Testing Strategy

### Unit Tests
- Component rendering with different prop combinations
- CSS variable application
- Responsive behavior
- Accessibility compliance

### Integration Tests
- Cross-component communication
- Theme switching functionality
- Keyboard navigation flows
- Mobile touch interactions

### Visual Regression Tests
- Screenshot testing for different screen sizes
- Theme variation testing
- Component state variations

## Troubleshooting

### Common Issues

#### Layout Breaks on Mobile
- **Cause**: Missing responsive classes
- **Solution**: Add responsive utility classes
- **Example**: `className="flex flex-col md:flex-row"`

#### CSS Variables Not Applied
- **Cause**: Missing CSS variable definitions
- **Solution**: Import theme CSS or define variables
- **Check**: Browser DevTools for computed styles

#### Sidebar Not Collapsing
- **Cause**: Fixed width without responsive handling
- **Solution**: Use responsive width classes
- **Example**: `className="w-64 md:w-80 lg:w-96"`

### Debug Tools
- Browser DevTools for layout inspection
- React DevTools for component hierarchy
- CSS Grid/Flexbox visualization tools
- Accessibility audit tools

## Future Enhancements

### Planned Features
1. **Built-in Resizable Panels**: Native resize support
2. **Advanced Theme System**: More customization options
3. **Layout Presets**: Common layout configurations
4. **Animation Support**: Smooth transitions between states
5. **Mobile-First Redesign**: Enhanced mobile experience

### API Considerations
- Layout state persistence
- Dynamic layout switching
- Plugin architecture for extensions
- Performance monitoring integration