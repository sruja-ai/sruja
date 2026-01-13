# Designer Refactoring Priorities & Quick Wins

## Immediate Actions (Can Do Today)

### 1. Fix Duplicate `slugify` Functions ⚡

**Problem**: Builder components have duplicate `slugify` implementations

**Files to Fix**:

- `apps/designer/src/components/Panels/Builder/BuilderL1Context.tsx` (lines 12-17)
- `apps/designer/src/components/Panels/Builder/BuilderL2Container.tsx` (lines 10-15)
- `apps/designer/src/components/Panels/Builder/BuilderL3Component.tsx` (lines 9-14)

**Action**: Replace with:

```typescript
import { slugify } from "../../../utils/slugify";
```

**Impact**: Removes ~15 lines of duplicate code, ensures consistency

---

### 2. Consolidate Form Field Patterns

**Problem**: All Edit\*Form components have similar field patterns

**Quick Win**: Create shared field components:

- `NameField` - name input with validation
- `DescriptionField` - textarea for descriptions
- `IdField` - custom ID input with checkbox
- `TechnologyField` - technology select/input

**Files Affected**: All 19 form components

**Impact**: Reduces form code by ~20-30%

---

### 3. Extract Common Form Submit Pattern

**Problem**: All forms have similar submit handler structure

**Quick Win**: Create `useElementFormSubmit` hook:

```typescript
const { handleSubmit, isSubmitting } = useElementFormSubmit({
  element,
  kind: "system",
  onSuccess: onClose,
  updateArchitecture,
});
```

**Impact**: Reduces form code by ~30-40 lines per form

---

## High-Value Refactoring Opportunities

### 1. Form Component Consolidation (High Impact)

**Current**: 19 separate form components (~2,500 lines total)

**Proposed**: Generic form with schema/config

**Example Structure**:

```typescript
// Instead of EditSystemForm, EditContainerForm, etc.
<ElementForm
  element={element}
  kind="system"
  schema={systemFormSchema}
  isOpen={isOpen}
  onClose={onClose}
/>
```

**Estimated Reduction**: 60% of form code (~1,500 lines)

**Complexity**: Medium - Requires careful type system design

---

### 2. Builder Component Unification (High Impact)

**Current**: 3 separate builder components (~623 lines total)

**Proposed**: Single `BuilderForm` with level config

**Example Structure**:

```typescript
<BuilderForm
  level={1} // or 2, 3
  config={builderConfigs[level]}
/>
```

**Estimated Reduction**: 68% of builder code (~424 lines)

**Complexity**: Medium - Need to handle level-specific logic cleanly

---

### 3. Node Component Base Enhancement (Medium Impact)

**Current**: Some nodes use BaseCompoundNode, others don't

**Proposed**: Enhance BaseCompoundNode to support all node types

**Estimated Reduction**: 30% of node code (~360 lines)

**Complexity**: Low-Medium - Already have BaseCompoundNode pattern

---

## Code Duplication Analysis

### Duplicated Patterns Found

1. **Form Validation** (19x duplication)
   - Name validation: `if (!values.name.trim()) errors.name = "Name is required"`
   - ID validation: `if (values.customId && !values.idInput.trim()) errors.idInput = "ID is required"`
   - ID uniqueness check

2. **Element Update Pattern** (19x duplication)

   ```typescript
   await updateArchitecture((model) => {
     const newElements = { ...model.elements };
     // ... create/update logic
     return { ...model, elements: newElements };
   });
   ```

3. **Form State Initialization** (19x duplication)

   ```typescript
   initialValues: {
     name: element?.title || "",
     description: typeof element?.description === "string"
       ? element.description
       : (element?.description as unknown as { txt: string })?.txt || "",
     // ...
   }
   ```

4. **Builder Form State** (3x duplication)
   - Similar useState patterns
   - Similar submit handlers
   - Similar validation

---

## Recommended Refactoring Sequence

### Week 1: Quick Wins

1. ✅ Remove duplicate `slugify` (30 min)
2. ✅ Create shared form field components (2-3 hours)
3. ✅ Extract `useElementFormSubmit` hook (2-3 hours)

**Total Time**: ~1 day  
**Code Reduction**: ~200-300 lines

### Week 2: Form Consolidation

1. Create `BaseEditForm` component (1 day)
2. Create form schema system (1 day)
3. Migrate 3-4 forms as proof of concept (1 day)
4. Migrate remaining forms (2 days)

**Total Time**: ~5 days  
**Code Reduction**: ~1,500 lines

### Week 3: Builder Unification

1. Create `BaseBuilderForm` (1 day)
2. Extract level-specific configs (1 day)
3. Migrate all 3 builders (1 day)

**Total Time**: ~3 days  
**Code Reduction**: ~424 lines

### Week 4: Node Enhancement

1. Enhance `BaseCompoundNode` (1 day)
2. Migrate remaining nodes (1 day)

**Total Time**: ~2 days  
**Code Reduction**: ~360 lines

---

## Total Impact Estimate

**Current Component Code**: ~4,323 lines (forms + builders + nodes)

**After Refactoring**: ~1,839 lines

**Reduction**: ~2,484 lines (57% reduction)

**Time Investment**: ~11 days

**ROI**: High - Significant maintainability improvement

---

## Risk Assessment

### Low Risk

- ✅ Removing duplicate `slugify` - Already tested utility
- ✅ Creating shared form fields - Isolated changes
- ✅ Extracting hooks - Can be done incrementally

### Medium Risk

- ⚠️ Form consolidation - Need careful type system
- ⚠️ Builder unification - Need to handle level differences

### Mitigation

- Start with quick wins (low risk)
- Migrate incrementally (one form at a time)
- Keep old components until new ones are proven
- Comprehensive testing at each step

---

## Next Steps

1. **Start with Quick Wins** (Today)
   - Remove duplicate `slugify` functions
   - Create 2-3 shared form field components

2. **Proof of Concept** (This Week)
   - Create `BaseEditForm` for one form type
   - Validate approach works

3. **Incremental Migration** (Next 2-3 Weeks)
   - Migrate forms one by one
   - Test thoroughly at each step

4. **Builder & Node Refactoring** (Following Weeks)
   - Apply lessons learned from forms
   - Similar incremental approach
