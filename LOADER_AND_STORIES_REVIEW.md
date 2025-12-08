# Loader Usage and Storybook Stories Review

## SrujaLoader Component

### Current Location
- **Defined**: `apps/website/src/shared/components/ui/SrujaLoader.tsx`
- **Status**: ❌ **Not in shared UI package** - Should be moved to `@sruja/ui` for consistency

### Usage Locations
1. `apps/website/src/features/viewer/components/PreviewPanels.tsx` - 3 uses
2. `apps/website/src/features/viewer/components/InteractiveViewer.tsx` - 4 uses
3. `apps/website/src/features/viewer/components/ViewerApp.tsx` - 2 uses
4. `apps/website/src/features/playground/components/LiveSrujaBlock.tsx` - 1 use
5. `apps/website/src/shared/components/ui/CodeBlockActions.tsx` - 1 use (dynamic creation)

**Total**: 11 uses across the website app

### Import Pattern
```typescript
import SrujaLoader from '@/shared/components/ui/SrujaLoader';
```

### Issues
1. ❌ **Not in shared package**: Component is website-specific but should be reusable
2. ❌ **No Storybook story**: Missing from Storybook documentation
3. ❌ **Inconsistent location**: Should be in `@sruja/ui` package for reuse

## Storybook Stories Coverage

### ✅ Components with Stories (24 stories)
- AppShell
- Badge
- Breadcrumb
- Button
- Card
- Combobox
- Dialog
- Editor (Monaco)
- Footer
- Header
- Input
- Listbox
- Menu
- MonacoVariants
- Popover
- RadioGroup
- SearchBar
- SearchDialog
- Skeleton
- Switch
- Tabs
- ThemeToggle
- Viewer
- ViewerInteractive

### ❌ Components Missing Stories
1. **SrujaLoader** - Used 11 times, no story
2. **Disclosure** - Component exists, no story
3. **Logo** - Component exists, no story
4. **MarkdownPreview** - Component exists, no story
5. **MermaidDiagram** - Component exists, no story
6. **MonacoEditor** - Component exists (Editor.stories.tsx might cover it)
7. **PosthogProvider** - Provider component, may not need story
8. **SrujaMonacoEditor** - Component exists, no story
9. **ThemeProvider** - Provider component, may not need story

## Recommendations

### 1. Move SrujaLoader to Shared Package
- Move `SrujaLoader.tsx` from `apps/website/src/shared/components/ui/` to `packages/ui/src/components/`
- Export from `packages/ui/src/components/index.ts`
- Update all imports to use `@sruja/ui`

### 2. Create Missing Stories
Priority order:
1. **SrujaLoader** (High - used 11 times)
2. **Disclosure** (Medium - UI component)
3. **Logo** (Low - simple component)
4. **MarkdownPreview** (Medium - used in docs)
5. **MermaidDiagram** (Medium - used in docs)
6. **SrujaMonacoEditor** (Low - variant of Editor)

### 3. Documentation Loader
- `packages/shared/src/documentation/loader.ts` - Different from SrujaLoader
- This is a utility for loading markdown docs, not a UI component
- ✅ **Status**: Correctly placed in `@sruja/shared`

## Action Items

1. ✅ Review loader usage consistency
2. ✅ Move SrujaLoader to `@sruja/ui` package
3. ✅ Create SrujaLoader.stories.tsx
4. ✅ Create stories for other missing components (Disclosure, Logo, MarkdownPreview, MermaidDiagram, SrujaMonacoEditor)
5. ✅ Update imports after moving SrujaLoader

## ✅ Status: COMPLETE

All tasks have been completed:
- SrujaLoader moved to `packages/ui/src/components/SrujaLoader.tsx`
- CSS animations added to `packages/ui/src/design-system/styles.css`
- All 7 import statements updated to use `@sruja/ui`
- 6 new Storybook stories created (30 total stories now)
- Old files deleted
- Component builds successfully

