# Sruja Designer Improvement Plan

## Inspired by Excalidraw & draw.io (Local-First Tools)

### 🎯 Priority 1: Essential Features (Implement First)

#### 1. **Export to Image Formats** ⭐⭐⭐

**Why**: Users need to share diagrams visually, embed in docs
**Implementation**:

- Add "Export as PNG" and "Export as SVG" to Actions menu
- Use html2canvas or react-flow's built-in export
- Add export button in diagram toolbar
- Options: Export current view, Export full diagram, Export with/without background

**Files to modify**:

- `apps/playground/src/App.tsx` - Add export handlers
- `apps/playground/src/components/Canvas/ArchitectureCanvas.tsx` - Export logic

#### 2. **Keyboard Shortcuts** ⭐⭐⭐

**Why**: Professional tools need keyboard shortcuts for speed
**Implementation**:

- `Cmd/Ctrl + K` - Command palette
- `Cmd/Ctrl + Z/Y` - Undo/Redo (when history is added)
- `Cmd/Ctrl + S` - Export/Save
- `Cmd/Ctrl + O` - Import/Open
- `Cmd/Ctrl + /` - Show shortcuts help
- `?` - Show shortcuts modal
- `Escape` - Close modals/panels
- `Tab` - Cycle through tabs
- `Cmd/Ctrl + F` - Focus search (in navigation)

**Files to create**:

- `apps/playground/src/hooks/useKeyboardShortcuts.ts`
- `apps/playground/src/components/shared/ShortcutsModal.tsx`

#### 3. **Command Palette (Cmd+K)** ⭐⭐

**Why**: Quick access to all actions, discoverability
**Implementation**:

- Fuzzy search through actions
- Categories: Navigation, Actions, Export, Settings
- Keyboard navigation (arrow keys, enter)
- Show keyboard shortcuts in results

**Files to create**:

- `apps/playground/src/components/shared/CommandPalette.tsx`

#### 4. **Undo/Redo System** ⭐⭐⭐

**Why**: Critical for user confidence, prevents data loss fear
**Implementation**:

- History stack for architecture changes
- Limit to last 50 actions
- Store in memory (local-first, no server)
- Visual indicator when undo/redo available

**Files to create**:

- `apps/playground/src/stores/historyStore.ts`
- Integrate with `useArchitectureStore`

### 🎯 Priority 2: Workflow Enhancements

#### 5. **Copy/Paste/Duplicate** ⭐⭐

**Why**: Speed up creation of similar elements
**Implementation**:

- `Cmd/Ctrl + C` - Copy selected node
- `Cmd/Ctrl + V` - Paste (with auto-increment ID)
- `Cmd/Ctrl + D` - Duplicate selected
- Right-click context menu: "Duplicate", "Copy", "Paste"

**Files to modify**:

- `apps/playground/src/components/Canvas/ArchitectureCanvas.tsx`
- Add clipboard state management

#### 6. **Enhanced Zoom Controls** ⭐⭐

**Why**: Better navigation for large diagrams
**Implementation**:

- "Fit to Screen" button (Cmd/Ctrl + 0)
- "Zoom to Selection" (Cmd/Ctrl + =)
- "Actual Size" (Cmd/Ctrl + 1)
- Zoom percentage display
- Mouse wheel zoom with modifier keys

**Files to modify**:

- `apps/playground/src/components/Canvas/ArchitectureCanvas.tsx`
- Enhance React Flow Controls

#### 7. **Grid & Snap-to-Grid** ⭐

**Why**: Better alignment, professional look
**Implementation**:

- Toggle grid overlay (View menu or toolbar)
- Snap-to-grid option
- Grid size settings (8px, 16px, 32px)
- Visual grid lines (subtle)

**Files to create**:

- `apps/playground/src/components/Canvas/GridOverlay.tsx`
- Add to settings/store

### 🎯 Priority 3: Polish & UX

#### 8. **Enhanced MiniMap** ⭐

**Why**: Better navigation for large diagrams
**Implementation**:

- Click minimap to jump to area
- Show current viewport rectangle
- Better node representation
- Toggle minimap visibility

**Files to modify**:

- `apps/playground/src/components/Canvas/ArchitectureCanvas.tsx`

#### 9. **Better Visual Feedback** ⭐

**Why**: More polished, professional feel
**Implementation**:

- Smooth transitions on node selection
- Hover effects on interactive elements
- Loading states with progress
- Success/error toast notifications
- Ripple effects on button clicks

**Files to modify**:

- Various component CSS files
- Add animation utilities

#### 10. **Template Gallery Enhancement** ⭐

**Why**: Easier discovery of starter templates
**Implementation**:

- More prominent template button
- Template preview thumbnails
- Categories with icons
- "New from Template" in empty state
- Template search/filter

**Files to modify**:

- `apps/playground/src/components/shared/ExamplesDropdown.tsx`
- `apps/playground/src/App.tsx` (empty state)

### 🎯 Additional Ideas from draw.io/Excalidraw

#### 11. **Right-Click Context Menu**

- Copy, Paste, Duplicate, Delete
- Edit Properties
- Navigate to (if in different view)
- Export selection

#### 12. **Multi-Select**

- Shift+Click to select multiple nodes
- Drag to select area
- Bulk operations (delete, copy, etc.)

#### 13. **Search & Jump**

- Global search (Cmd/Ctrl + F)
- Jump to node by ID
- Highlight search results

#### 14. **Better Empty State**

- Template cards with previews
- Recent files (localStorage)
- Quick start guide
- Video tutorial link

#### 15. **Diagram Layers**

- Show/hide layers (persons, systems, containers)
- Layer opacity controls
- Focus mode (hide non-essential)

#### 16. **Export Options Dialog**

- Format selection (PNG, SVG, PDF)
- Resolution/quality settings
- Include/exclude elements
- Background color option
- Transparent background option

#### 17. **Keyboard Navigation**

- Arrow keys to move selected node
- Tab to cycle through nodes
- Enter to edit selected node
- Delete to remove selected

#### 18. **Better Tooltips**

- Rich tooltips with shortcuts
- Contextual help
- Feature discovery hints

#### 19. **Performance Indicators**

- Show layout calculation progress
- Node count display
- Performance mode toggle
- Lazy loading indicators

#### 20. **Theme Customization**

- More theme options
- Custom color schemes
- High contrast mode
- Print-friendly theme

## Implementation Order Recommendation

**Week 1-2 (Critical)**:

1. Export PNG/SVG
2. Basic keyboard shortcuts (Cmd+S, Cmd+O, Cmd+Z/Y)
3. Undo/Redo system

**Week 3-4 (High Value)**: 4. Command Palette (Cmd+K) 5. Copy/Paste/Duplicate 6. Enhanced zoom controls

**Week 5-6 (Polish)**: 7. Grid & snap-to-grid 8. Right-click context menu 9. Better visual feedback 10. Template gallery enhancement

**Future**:

- Multi-select
- Diagram layers
- Advanced export options
- Performance optimizations
