# Next Consistency Checks

## ✅ Completed

1. **SrujaLoader Migration** - Moved to `@sruja/ui`, all imports updated, Storybook stories created
2. **Storybook Coverage** - All 30 UI components now have stories
3. **Structured Logging** - Enhanced logger with structured logging, migrated social-publish apps
4. **ESLint v9** - Upgraded, shared config created, all packages using it
5. **TypeScript Migration** - Scripts converted to TypeScript with ZX where beneficial

## 🔍 Potential Next Areas

### 1. Website-Specific Components
Components in `apps/website/src/shared/components/ui/` that might be candidates for `@sruja/ui`:
- `TagList.tsx` - Used in website, could be reusable
- `EmptyState.tsx` - Generic component, could be shared
- `EngagementTracker.tsx` - Website-specific, keep here
- `ThemeWrapper.tsx` - Wrapper component, might be website-specific
- `CodeBlockActions.tsx` - Website-specific functionality, keep here

**Recommendation**: Review `TagList` and `EmptyState` - if they're generic enough, consider moving to `@sruja/ui`.

### 2. Import Path Consistency
- ✅ `@sruja/ui` - 34 imports (good)
- ⚠️ `@/shared` - 20 imports (website-specific utilities, appropriate)

**Status**: Current usage is appropriate - `@/shared` is for website-specific utilities, `@sruja/ui` for shared UI components.

### 3. Linting Issues
- ⚠️ `packages/html-viewer` has 3 errors and 36 warnings (mostly `any` types)
- These are non-blocking but could be improved

**Recommendation**: Address `any` types in html-viewer package (low priority).

### 4. Build Output Consistency
- ⚠️ Warning: `no output files found for task @sruja/shared#build`
- Check `turbo.json` outputs configuration

**Recommendation**: Review turbo.json outputs for shared package.

### 5. Component Export Consistency
- ✅ All components use named exports (no default exports)
- ✅ Consistent with TypeScript best practices

## Summary

**Current Status**: ✅ **Good**
- SrujaLoader migration complete
- Storybook coverage complete
- Import patterns are appropriate
- Minor improvements possible but not critical

**Next Steps** (Optional):
1. Review `TagList` and `EmptyState` for potential move to `@sruja/ui`
2. Address `any` types in html-viewer (low priority)
3. Fix turbo.json outputs warning (low priority)

