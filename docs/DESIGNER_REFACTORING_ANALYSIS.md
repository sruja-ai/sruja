# Designer Component Refactoring Analysis

## Executive Summary

**Total Components**: 158 TypeScript/TSX files  
**Key Findings**: Significant code duplication across forms, builders, and some node components  
**Refactoring Priority**: High - Multiple opportunities to reduce code by ~30-40%

## Major Duplication Patterns

### 1. Form Components (High Priority) ⚠️

**Location**: `apps/designer/src/components/shared/forms/`

**Duplicated Code**:

- **19 form components** with nearly identical structure:
  - `EditSystemForm.tsx`
  - `EditContainerForm.tsx`
  - `EditComponentForm.tsx`
  - `EditPersonForm.tsx`
  - `EditQueueForm.tsx`
  - `EditDataStoreForm.tsx`
  - `EditRequirementForm.tsx`
  - `EditADRForm.tsx`
  - `EditPolicyForm.tsx`
  - `EditConstraintForm.tsx`
  - `EditConventionForm.tsx`
  - `EditFlowForm.tsx`
  - `EditMetadataForm.tsx`
  - `EditOverviewForm.tsx`

**Common Patterns**:

```typescript
// Repeated in EVERY form:
- useFormState hook with similar structure
- FormField components with same props
- SidePanel wrapper
- Similar validation logic
- Similar submit handlers
- ID generation/slugify logic
- Element update patterns
```

**Refactoring Opportunity**:

- Create a generic `BaseEditForm<T>` component
- Extract common form fields into reusable components
- Create form schema/config system
- **Estimated Reduction**: ~60% of form code

### 2. Builder Components (High Priority) ⚠️

**Location**: `apps/designer/src/components/Panels/Builder/`

**Duplicated Code**:

- `BuilderL1Context.tsx` (258 lines)
- `BuilderL2Container.tsx` (204 lines)
- `BuilderL3Component.tsx` (161 lines)

**Common Patterns**:

```typescript
// Repeated in ALL builders:
- slugify function (duplicated 3x!)
- useState for form fields
- Similar submit handlers
- Similar validation
- Similar UI structure
- useBuilderProgress hook
```

**Refactoring Opportunity**:

- Extract `slugify` to shared utils (already exists in `utils/slugify.ts` but not used!)
- Create `BaseBuilderForm` component
- Use shared form components
- **Estimated Reduction**: ~50% of builder code

### 3. Node Components (Medium Priority)

**Location**: `apps/designer/src/components/Nodes/`

**Current State**:

- ✅ `ContainerNode` already uses `BaseCompoundNode` (good!)
- ⚠️ `SystemNode` has 221 lines with custom logic
- ⚠️ `ComponentNode` has custom implementation
- ⚠️ `PersonNode`, `DataStoreNode`, `QueueNode` have similar patterns

**Duplication**:

- Similar node styling logic
- Similar handle positioning
- Similar badge/icon rendering
- Similar color scheme usage

**Refactoring Opportunity**:

- Create `BaseNode` component for common patterns
- Extract node rendering logic
- **Estimated Reduction**: ~30% of node code

### 4. Panel Components (Low Priority)

**Location**: `apps/designer/src/components/Panels/`

**Duplication**:

- Similar panel structure
- Similar empty states
- Similar loading states

**Note**: Some duplication is acceptable for clarity, but could be improved with shared components.

## Specific Code Duplication Examples

### Example 1: slugify Function (Duplicated 3x)

**Found in**:

- `BuilderL1Context.tsx` (lines 12-17)
- `BuilderL2Container.tsx` (lines 10-15)
- `BuilderL3Component.tsx` (lines 9-14)

**Solution**: Already exists in `utils/slugify.ts` - just import it!

### Example 2: Form Validation Pattern

**Repeated in all Edit\*Form components**:

```typescript
validate: (values) => {
  const errors: FormErrors = {};
  if (!values.name.trim()) errors.name = "Name is required";
  if (values.customId && !values.idInput.trim()) errors.idInput = "ID is required";
  // ... more validation
  return errors;
};
```

**Solution**: Create validation schema system or shared validators.

### Example 3: Element Update Pattern

**Repeated in all forms**:

```typescript
await updateArchitecture((model) => {
  const newElements = { ...model.elements };
  // ... element creation/update logic
  return { ...model, elements: newElements };
});
```

**Solution**: Extract to shared utility functions.

## Recommended Refactoring Plan

### Phase 1: Quick Wins (1-2 days)

1. **Remove duplicate `slugify` functions**
   - Replace with import from `utils/slugify.ts`
   - Files: `BuilderL1Context.tsx`, `BuilderL2Container.tsx`, `BuilderL3Component.tsx`

2. **Extract common form utilities**
   - Create `formUtils.ts` with:
     - ID generation logic
     - Element update helpers
     - Common validators

3. **Consolidate form field components**
   - Create reusable field components
   - Reduce repetition in form definitions

### Phase 2: Form Component Refactoring (3-5 days)

1. **Create generic form base**
   - `BaseEditForm<T>` component
   - Schema-driven form generation
   - Reduce 19 forms to ~5-7 with configs

2. **Extract form schemas**
   - Define form structure declaratively
   - Type-safe form generation

### Phase 3: Builder Component Refactoring (2-3 days)

1. **Create `BaseBuilderForm`**
   - Shared form structure
   - Level-specific configs
   - Reduce 3 builders to 1 with configs

2. **Unify builder logic**
   - Shared submit handlers
   - Shared validation
   - Shared UI components

### Phase 4: Node Component Refactoring (2-3 days)

1. **Enhance `BaseCompoundNode`**
   - Support more node types
   - Extract common patterns

2. **Create node factory**
   - Config-driven node creation
   - Reduce custom node components

## Metrics

### Current State

- **Total Components**: 158 files
- **Form Components**: 19 files (~2,500 lines)
- **Builder Components**: 3 files (~623 lines)
- **Node Components**: 11 files (~1,200 lines)

### After Refactoring (Estimated)

- **Form Components**: ~7 files (~1,000 lines) - **60% reduction**
- **Builder Components**: ~1 file (~200 lines) - **68% reduction**
- **Node Components**: ~8 files (~840 lines) - **30% reduction**
- **Total Reduction**: ~1,283 lines (~30% of component code)

## Benefits

1. **Maintainability**: Single source of truth for common patterns
2. **Consistency**: Unified behavior across similar components
3. **Testing**: Easier to test shared logic
4. **Performance**: Better code splitting opportunities
5. **Developer Experience**: Less code to understand and modify

## Risks & Considerations

1. **Over-abstraction**: Don't create too many layers
2. **Type Safety**: Ensure TypeScript types remain strong
3. **Backward Compatibility**: Ensure no breaking changes
4. **Testing**: Update tests for refactored components
5. **Gradual Migration**: Refactor incrementally, not all at once

## Priority Recommendations

### Immediate (This Week)

1. ✅ Remove duplicate `slugify` functions
2. ✅ Extract common form utilities
3. ✅ Create shared form field components

### Short Term (This Month)

1. Create `BaseEditForm` for form components
2. Refactor builder components to use shared base
3. Extract common node patterns

### Long Term (Next Quarter)

1. Schema-driven form generation
2. Config-driven component creation
3. Further consolidation where it makes sense

## Files to Review First

1. `apps/designer/src/components/shared/forms/` - All form components
2. `apps/designer/src/components/Panels/Builder/` - All builder components
3. `apps/designer/src/utils/slugify.ts` - Already exists, not used everywhere
4. `apps/designer/src/components/Nodes/` - Node component patterns
