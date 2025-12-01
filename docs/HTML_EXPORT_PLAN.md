# Plan: Self-Contained HTML Export

## Overview

Switch from single SVG export to self-contained HTML export for better flexibility, interactivity, and layout control.

## Benefits of HTML over SVG

1. **Better Layout Control**
   - CSS Grid/Flexbox for responsive layouts
   - Multi-panel layouts (diagram + documentation side-by-side)
   - Better mobile responsiveness

2. **Enhanced Interactivity**
   - Full JavaScript capabilities (not limited to SVG DOM)
   - Can use modern frameworks if needed
   - Better event handling
   - More UI components (modals, dropdowns, tabs)

3. **Better Content Organization**
   - Separate panels for diagram, documentation, navigation
   - Collapsible sections
   - Search/filter capabilities
   - Better typography and readability

4. **Performance**
   - Can lazy-load views
   - Better rendering performance for complex diagrams
   - Can use Canvas/WebGL for large diagrams

5. **Accessibility**
   - Better screen reader support
   - Keyboard navigation
   - Semantic HTML structure

6. **Extensibility**
   - Easier to add features (export buttons, print, share)
   - Can embed external resources if needed
   - Better integration with other tools

## Architecture

### Current Structure (SVG)
```
<svg>
  <defs>...</defs>
  <g id="diagramArea">
    <g id="level1">...</g>
    <g id="view-*">...</g>
  </g>
  <g id="uiControls">...</g>
  <script>...</script>
</svg>
```

### Proposed Structure (HTML)
```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>Architecture: {name}</title>
  <style>/* Embedded CSS */</style>
</head>
<body>
  <div class="app-container">
    <header class="app-header">
      <!-- Title, search, filters -->
    </header>
    <div class="app-body">
      <aside class="sidebar">
        <!-- Navigation, level buttons, view list -->
      </aside>
      <main class="diagram-area">
        <div class="diagram-container">
          <!-- SVG diagrams embedded here -->
        </div>
        <div class="documentation-panel">
          <!-- Element documentation -->
        </div>
      </main>
    </div>
  </div>
  <script>/* Embedded JavaScript */</script>
</body>
</html>
```

## Implementation Plan

### Phase 1: Create HTML Exporter Structure

1. **New Package**: `pkg/export/html/`
   - `html.go` - Main exporter
   - `template.go` - HTML template generation
   - `styles.go` - CSS generation
   - `javascript.go` - JavaScript generation
   - `layout.go` - Layout structure

2. **Reuse Existing Logic**
   - Keep `pkg/export/svg/` for SVG generation
   - HTML exporter calls SVG exporter internally
   - Embed generated SVGs in HTML

### Phase 2: HTML Structure

1. **Header Section**
   - Architecture title
   - Search bar
   - Filter buttons (Requirements, ADRs, Technology)
   - Zoom controls

2. **Sidebar Navigation**
   - Level buttons (L1, L2, L3)
   - View list (scenarios, flows, domains, etc.)
   - Collapsible sections
   - Active view indicator

3. **Main Content Area**
   - Diagram panel (left/center)
   - Documentation panel (right, collapsible)
   - Responsive layout (stacks on mobile)

4. **Footer** (optional)
   - Export options
   - Metadata

### Phase 3: Enhanced Features

1. **Multi-Panel Layout**
   - Split view: diagram + docs
   - Tabbed interface for multiple views
   - Full-screen diagram mode

2. **Better Navigation**
   - Breadcrumbs
   - History (back/forward)
   - Keyboard shortcuts

3. **Search & Filter**
   - Search elements by name
   - Filter by type
   - Highlight search results

4. **Export Options**
   - Export current view as PNG
   - Export as PDF
   - Share link generation

### Phase 4: Migration

1. **CLI Command**
   - Add `sruja export html` command
   - Keep `sruja export svg` for backward compatibility
   - Default to HTML in future

2. **Backward Compatibility**
   - Keep SVG export working
   - Document migration path
   - Provide both options

## File Structure

```
pkg/export/
├── svg/           # Existing SVG exporter (keep)
│   ├── svg.go
│   ├── postprocess.go
│   └── ...
└── html/          # New HTML exporter
    ├── html.go    # Main exporter
    ├── template.go # HTML structure
    ├── styles.go  # CSS generation
    ├── javascript.go # JS generation
    └── layout.go  # Layout helpers
```

## Implementation Details

### HTML Exporter Interface

```go
type Exporter struct {
    svgExporter *svg.Exporter
}

func (e *Exporter) Export(arch *language.Architecture) (string, error) {
    // 1. Generate SVGs using existing SVG exporter
    // 2. Build HTML structure
    // 3. Embed SVGs
    // 4. Add CSS and JavaScript
    // 5. Return complete HTML
}
```

### Key Components

1. **Template System**
   - Use Go templates for HTML structure
   - Separate templates for header, sidebar, main, footer
   - Data-driven content

2. **CSS Architecture**
   - Modern CSS (Grid, Flexbox)
   - CSS variables for theming
   - Responsive breakpoints
   - Print styles

3. **JavaScript Architecture**
   - Modular JavaScript
   - View management
   - Event handling
   - State management

## Migration Strategy

### Option 1: Parallel Implementation
- Keep SVG export
- Add HTML export as new option
- Users choose which format

### Option 2: HTML as Default
- Make HTML the default export
- Keep SVG as `--format svg` option
- Update documentation

### Option 3: Gradual Migration
- Start with HTML export
- Keep SVG for simple use cases
- Migrate features over time

## Benefits Summary

| Feature | SVG | HTML |
|--------|-----|------|
| Layout Control | Limited | Full CSS |
| Multi-panel | Difficult | Easy |
| Responsive | Limited | Full support |
| Interactivity | SVG DOM | Full JS |
| Accessibility | Limited | Better |
| Performance | Good | Better for complex |
| File Size | Smaller | Larger (but minifiable) |
| Browser Support | Good | Excellent |

## Considerations

1. **File Size**
   - HTML will be larger than SVG
   - Can minify HTML/CSS/JS
   - Can use compression (gzip)

2. **Self-Contained Requirement**
   - All CSS/JS must be embedded
   - No external dependencies
   - All SVGs embedded

3. **Backward Compatibility**
   - Keep SVG export available
   - Provide migration guide
   - Support both formats

4. **Testing**
   - Test in multiple browsers
   - Test responsive layouts
   - Test accessibility

## Next Steps

1. ✅ Create plan document (this file)
2. ⏳ Create `pkg/export/html/` package structure
3. ⏳ Implement basic HTML template
4. ⏳ Integrate SVG generation
5. ⏳ Add CSS styling
6. ⏳ Add JavaScript interactivity
7. ⏳ Add CLI command
8. ⏳ Test and refine
9. ⏳ Update documentation
10. ⏳ Migration guide

## Timeline Estimate

- **Phase 1**: 2-3 days (basic HTML structure)
- **Phase 2**: 2-3 days (layout and navigation)
- **Phase 3**: 3-5 days (enhanced features)
- **Phase 4**: 1-2 days (migration and docs)

**Total**: ~2 weeks for full implementation



