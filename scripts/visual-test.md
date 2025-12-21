# Visual Testing Guide

## Quick Visual Test Checklist

### ✅ Component Integration Verification

1. **Viewer App** (`apps/viewer`)
   - [ ] TopBar: Breadcrumb navigation renders correctly
   - [ ] TopBar: SearchBar shows results dropdown
   - [ ] TopBar: Level buttons (Context/Container/Component) toggle correctly
   - [ ] TopBar: Drag toggle button works
   - [ ] OverviewPage: Export buttons (PNG, SVG, JSON) render
   - [ ] OverviewPage: Metric cards display correctly
   - [ ] Requirements page: Cards with badges render
   - [ ] ADRs page: Cards with status badges render
   - [ ] Scenarios page: Cards with action buttons render
   - [ ] Systems page: Cards with container count badges render
   - [ ] InspectorPanel: Badges for type and tech stack render

2. **Studio App** (`apps/studio`)
   - [ ] Header: Export and Preview buttons render
   - [ ] Toolbar: All add element buttons (Person, System, Container, etc.) render
   - [ ] Toolbar: Level buttons (L1, L2, L3) render
   - [ ] Toolbar: Zoom controls render
   - [ ] Toolbar: Delete button (danger variant) renders
   - [ ] ExportDialog: Format selection buttons render
   - [ ] ExportDialog: Export and Cancel buttons render
   - [ ] AdrModal: Create and Cancel buttons render
   - [ ] InputModal: Confirm and Cancel buttons render
   - [ ] SearchDialog: Close button renders
   - [ ] CommandPalette: Close button renders
   - [ ] Toast: Close button renders

3. **Learn App** (`apps/learn`)
   - [ ] Homepage: CTA buttons (Get Started, Open Studio, Search) render
   - [ ] Code blocks: Copy and Open in Studio buttons render
   - [ ] ThemeToggle: Renders in navbar

## Running Visual Tests

### Option 1: Storybook (Recommended)
```bash
cd apps/storybook
npm run dev
# Open http://localhost:6006
# Navigate through all component stories
```

### Option 2: Run Individual Apps
```bash
# Viewer App
cd apps/viewer
npm run dev
# Test all pages and interactions

# Studio App  
cd apps/studio
npm run dev
# Test toolbar, dialogs, and modals

# Learn App
cd apps/learn
npm start
# Test homepage and code blocks
```

## Visual Test Checklist

### Button Component
- [ ] All variants render (primary, secondary, outline, ghost, danger)
- [ ] All sizes render (sm, md, lg)
- [ ] Loading state shows spinner
- [ ] Disabled state is visually distinct
- [ ] Hover states work correctly
- [ ] Focus states are visible

### Card Component
- [ ] Basic card with title renders
- [ ] Card with subtitle renders
- [ ] Card with footer renders
- [ ] Interactive card has hover effect
- [ ] Card grid layouts work

### Badge Component
- [ ] All color variants render
- [ ] Badges in card footers align correctly
- [ ] Badges in inspector panel render

### Breadcrumb Component
- [ ] Home icon renders
- [ ] Breadcrumb items render
- [ ] Separators render correctly
- [ ] Active item is highlighted
- [ ] Click handlers work

### SearchBar Component
- [ ] Input field renders
- [ ] Search results dropdown appears
- [ ] Results are selectable
- [ ] Keyboard navigation works

## Common Issues to Check

1. **Missing CSS Variables**
   - Check if `var(--color-*)` variables are defined
   - Verify theme provider is wrapping components

2. **Import Errors**
   - Verify all components are exported from `@sruja/ui`
   - Check for circular dependencies

3. **Type Errors**
   - Run TypeScript compiler: `npx tsc --noEmit`
   - Check for missing type definitions

4. **Styling Issues**
   - Verify Tailwind classes are working
   - Check for CSS conflicts
   - Ensure design system styles are loaded

## Automated Visual Testing

For CI/CD, consider:
- Chromatic for visual regression testing
- Playwright for E2E visual tests
- Screenshot comparison tests

