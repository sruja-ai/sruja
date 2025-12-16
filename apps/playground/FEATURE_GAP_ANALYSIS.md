# Feature Gap Analysis: Sruja Designer vs. Excalidraw & draw.io

## Current State Assessment

### ✅ What We Have:

- Examples/Templates dropdown
- React Flow controls (zoom, pan)
- MiniMap
- Export to .sruja file
- Import from .sruja file
- Share via URL
- Local storage persistence
- View/Edit mode toggle
- Multiple tabs (Builder, Diagram, Details, Code)

### ❌ What's Missing (High Priority):

#### 1. **Export to Image Formats** (Critical)

- **Excalidraw**: PNG, SVG with embedded data
- **draw.io**: PNG, JPG, PDF, SVG, XML
- **Our Gap**: Only exports .sruja DSL file
- **Impact**: Users can't embed diagrams in docs, presentations, or share visually

#### 2. **Keyboard Shortcuts** (High Value)

- **Excalidraw**: `?` shows shortcuts, `R` for rectangle, `A` for arrow, etc.
- **draw.io**: Extensive shortcuts for all actions
- **Our Gap**: No keyboard shortcuts at all
- **Impact**: Slower workflow, less professional feel

#### 3. **Command Palette** (High Value)

- **Excalidraw**: Quick actions via shortcuts
- **draw.io**: Search-based actions
- **Our Gap**: No command palette
- **Impact**: Hard to discover features, slower navigation

#### 4. **Copy/Paste/Duplicate** (High Value)

- **Excalidraw**: Full copy/paste with Ctrl+C/V
- **draw.io**: Copy/paste/duplicate nodes
- **Our Gap**: No copy/paste functionality
- **Impact**: Can't quickly duplicate similar systems/containers

#### 5. **Undo/Redo** (Critical)

- **Excalidraw**: Full undo/redo history
- **draw.io**: Undo/redo with history
- **Our Gap**: No undo/redo
- **Impact**: Fear of making mistakes, no safety net

#### 6. **Better Zoom Controls** (Medium)

- **Excalidraw**: Smooth zoom with mouse wheel, fit-to-screen
- **draw.io**: Multiple zoom options, fit-to-selection
- **Our Gap**: Basic React Flow controls
- **Impact**: Harder to navigate large diagrams

#### 7. **Grid & Snap-to-Grid** (Medium)

- **Excalidraw**: Grid overlay option
- **draw.io**: Grid with snap-to-grid
- **Our Gap**: No grid option
- **Impact**: Manual alignment is harder

#### 8. **Enhanced MiniMap** (Low)

- **Excalidraw**: N/A (different paradigm)
- **draw.io**: Interactive minimap with click-to-jump
- **Our Gap**: Basic minimap
- **Impact**: Less useful for large diagrams

#### 9. **Better Visual Feedback** (Medium)

- **Excalidraw**: Smooth animations, hover states
- **draw.io**: Clear selection indicators
- **Our Gap**: Basic feedback
- **Impact**: Less polished feel

#### 10. **Template Gallery** (Medium)

- **Excalidraw**: Graphics library
- **draw.io**: Extensive template library
- **Our Gap**: Examples dropdown exists but could be more prominent
- **Impact**: Harder to discover starter templates

## Recommended Implementation Priority

### Phase 1: Core Productivity (Week 1-2)

1. **Export to PNG/SVG** - Most requested, enables sharing
2. **Undo/Redo** - Safety net for users
3. **Keyboard Shortcuts** - Basic ones (Cmd+Z/Y, Cmd+S, etc.)

### Phase 2: Workflow Enhancement (Week 3-4)

4. **Copy/Paste/Duplicate** - Speed up creation
5. **Command Palette (Cmd+K)** - Feature discovery
6. **Better Zoom Controls** - Fit-to-screen, zoom-to-selection

### Phase 3: Polish (Week 5-6)

7. **Grid & Snap-to-Grid** - Better alignment
8. **Enhanced MiniMap** - Better navigation
9. **Visual Feedback Improvements** - Animations, hover states
10. **Template Gallery Enhancement** - More prominent, better UX

## Quick Wins (Can implement immediately):

- Add "?" keyboard shortcut to show shortcuts help
- Add Cmd+S to save/export
- Add Cmd+Z/Y for undo/redo (if we add history)
- Add export PNG button to Actions menu
- Improve minimap styling and interaction
