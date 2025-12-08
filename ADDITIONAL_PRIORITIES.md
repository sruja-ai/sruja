# Additional Priority Items

## 🔴 High-Value Quick Wins

### 1. Test Coverage Reporting
**Priority**: High
**Effort**: Low-Medium
**Impact**: High

**Current State**:
- Tests exist but no coverage reporting
- Can't track coverage trends
- No visibility into untested code

**Action Items**:
- Add coverage reporting to CI (Codecov or Codacy)
- Set coverage thresholds
- Generate coverage reports for TypeScript/JavaScript
- Track coverage over time

**Benefits**:
- Identify untested code paths
- Prevent coverage regression
- Guide testing efforts

---

### 2. Type Safety Audit & Improvements
**Priority**: High
**Effort**: Medium
**Impact**: High

**Current State**:
- Found 12+ instances of `any` types and `@ts-expect-error`
- Missing type definitions for some integrations
- Type safety could be improved

**Action Items**:
- Audit all `any` types in `packages/`
- Replace `@ts-expect-error` with proper types where possible
- Add type definitions for external integrations
- Enable stricter TypeScript checks

**Files to Review**:
- `packages/viewer/src/viewer.ts` (multiple `any` types)
- `packages/shared/src/web/wasmAdapter.ts` (WASM types)
- All `@ts-expect-error` comments

**Benefits**:
- Better IDE support
- Catch bugs at compile time
- Improved developer experience

---

### 3. API Documentation (JSDoc)
**Priority**: Medium-High
**Effort**: Medium
**Impact**: Medium-High

**Current State**:
- Public APIs lack JSDoc comments
- No generated API documentation
- Developers must read source code to understand APIs

**Action Items**:
- Add JSDoc to all public exports in `packages/shared`
- Add JSDoc to all public exports in `packages/ui`
- Add JSDoc to all public exports in `packages/viewer`
- Set up TypeDoc for API documentation generation
- Add examples to JSDoc comments

**Key APIs to Document**:
- `packages/shared/src/utils/logger.ts` - Logger API
- `packages/shared/src/analytics/errorTracking.ts` - Error tracking
- `packages/shared/src/web/wasmAdapter.ts` - WASM API
- `packages/ui/src/components/*` - All component props
- `packages/viewer/src/viewer.ts` - Viewer API

**Benefits**:
- Better developer experience
- Self-documenting code
- Easier onboarding

---

### 4. Bundle Size Monitoring
**Priority**: Medium
**Effort**: Low
**Impact**: Medium
**Status**: ✅ Infrastructure Setup Complete

**Current State**:
- ✅ `size-limit` tool installed
- ✅ Size limits configured
- ✅ CI integration added
- ⚠️ Note: Packages export TypeScript source, not bundles
- ⚠️ May need adjustment for actual bundle monitoring

**Completed**:
- Added `size-limit` to root package.json
- Configured size limits in `.size-limit.json`
- Added bundle size check to CI (non-blocking)
- Created documentation

**Future Enhancements**:
- Monitor actual app bundles (website, studio-core, viewer-core)
- Use webpack-bundle-analyzer for detailed analysis
- Set up Lighthouse CI for performance budgets
- Track bundle size trends over time

**Benefits**:
- Prevent bundle bloat
- Track performance impact
- Guide optimization efforts

---

### 5. More Test Coverage
**Priority**: Medium-High
**Effort**: Medium-High
**Impact**: High

**Current State**:
- 41 tests total (good start)
- Missing tests for critical packages:
  - `@sruja/viewer` - No tests
  - `packages/shared/src/web/wasmAdapter.ts` - No tests
  - More UI components need tests

**Action Items**:
- Add tests for `packages/viewer` core functionality
- Add tests for WASM adapter
- Add tests for more UI components (Dialog, Menu, Tabs, etc.)
- Add integration tests for component interactions

**Priority Test Areas**:
1. Viewer core (layout, rendering, interactions)
2. WASM adapter (all functions)
3. UI components (all interactive components)

---

### 6. Developer Experience Improvements
**Priority**: Medium
**Effort**: Low-Medium
**Impact**: Medium

**Current State**:
- Good setup, but could be improved

**Action Items**:
- Add pre-commit hooks (Husky + lint-staged)
- Add VS Code workspace settings for consistent formatting
- Add debugging configurations
- Improve error messages in development
- Add development scripts for common tasks

**Benefits**:
- Faster development cycles
- Consistent code style
- Better debugging experience

---

### 7. Code Quality Metrics Dashboard
**Priority**: Low-Medium
**Effort**: Medium
**Impact**: Medium

**Current State**:
- Codacy exists but no dashboard
- No visibility into code quality trends

**Action Items**:
- Set up code quality dashboard
- Track metrics over time:
  - Test coverage
  - Code complexity
  - Technical debt
  - Security issues
- Add quality gates to CI

**Benefits**:
- Track code quality trends
- Identify technical debt
- Guide refactoring efforts

---

### 8. Performance Budgets & Monitoring
**Priority**: Medium
**Effort**: Medium
**Impact**: Medium-High

**Current State**:
- No performance budgets
- No automated performance testing
- Performance issues discovered late

**Action Items**:
- Add Lighthouse CI to test performance
- Set performance budgets (LCP, FID, CLS)
- Monitor Core Web Vitals
- Add performance regression tests
- Track bundle size impact on performance

**Benefits**:
- Catch performance regressions early
- Maintain good user experience
- Guide optimization efforts

---

### 9. Visual Regression Testing
**Priority**: Low-Medium
**Effort**: Medium
**Impact**: Medium

**Current State**:
- No visual regression testing
- UI changes not automatically validated

**Action Items**:
- Set up Percy or Chromatic
- Add visual tests for UI components
- Add visual tests for critical pages
- Integrate into CI workflow

**Benefits**:
- Catch visual regressions
- Document UI changes
- Confidence in UI refactoring

---

### 10. Error Boundary Coverage
**Priority**: Medium
**Effort**: Low
**Impact**: Medium

**Current State**:
- Error boundary exists in Studio
- May be missing in other apps

**Action Items**:
- Audit all apps for error boundaries
- Add error boundaries where missing
- Improve error boundary UI/UX
- Test error boundary behavior

**Benefits**:
- Better error handling
- Improved user experience
- Easier debugging

---

## 📊 Priority Matrix

### Quick Wins (Low Effort, High Impact)
1. ✅ Test Coverage Reporting
2. ✅ Bundle Size Monitoring
3. ✅ Developer Experience Improvements

### High Impact (Medium-High Effort)
1. ✅ Type Safety Audit
2. ✅ More Test Coverage
3. ✅ API Documentation

### Long-term Value (Medium Effort)
1. ✅ Performance Budgets
2. ✅ Code Quality Dashboard
3. ✅ Visual Regression Testing

---

## Recommended Next Steps

**If you want quick wins:**
1. Test Coverage Reporting (1-2 hours)
2. Bundle Size Monitoring (1 hour)
3. Developer Experience (pre-commit hooks) (1 hour)

**If you want high impact:**
1. Type Safety Audit (4-6 hours)
2. API Documentation (6-8 hours)
3. More Test Coverage (8-12 hours)

**If you want long-term value:**
1. Performance Budgets (4-6 hours)
2. Code Quality Dashboard (2-4 hours)
3. Visual Regression Testing (4-6 hours)

---

## Summary

All of these items would improve code quality, developer experience, and maintainability. The choice depends on your immediate needs:

- **Quick wins**: Coverage reporting, bundle size, DX improvements
- **High impact**: Type safety, tests, documentation
- **Long-term**: Performance, quality metrics, visual testing

