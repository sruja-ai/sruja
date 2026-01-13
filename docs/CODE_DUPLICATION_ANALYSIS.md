# Code Duplication Analysis

## Executive Summary

Analysis of the `apps/designer/src/components/shared/forms/` directory reveals significant code duplication across multiple Edit\*Form components. While some shared utilities exist (`FormFields.tsx`, `useElementFormSubmit.ts`), they are not fully utilized, leading to ~70% code duplication across form components.

## Duplication Categories

### 1. **Form Component Structure (High Duplication - ~80% similar)**

All Edit\*Form components follow nearly identical patterns:

**Affected Files:**

- `EditSystemForm.tsx`
- `EditPersonForm.tsx`
- `EditContainerForm.tsx`
- `EditComponentForm.tsx`
- `EditDataStoreForm.tsx`
- `EditQueueForm.tsx`
- `EditADRForm.tsx`
- `EditPolicyForm.tsx`
- `EditOverviewForm.tsx`
- `EditMetadataForm.tsx`
- `EditRequirementForm.tsx`
- `EditConstraintForm.tsx`
- `EditConventionForm.tsx`
- `EditFlowForm.tsx`

**Duplicated Patterns:**

#### a) Component Structure

```typescript
// Repeated in ALL forms
export function EditXForm({ isOpen, onClose, element, initialName }: EditXFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const data = useArchitectureStore((s) => s.model);
  const formRef = useRef<HTMLFormElement>(null);

  // ... form setup
}
```

#### b) SidePanel Wrapper

```typescript
// Repeated in ALL forms
<SidePanel
  isOpen={isOpen}
  onClose={onClose}
  title={element ? `Edit ${Type}` : `Add ${Type}`}
  size="lg"
  footer={
    <>
      <Button variant="secondary" onClick={onClose} type="button">
        Cancel
      </Button>
      <Button
        variant="primary"
        type="submit"
        form={`edit-${type}-form`}
        isLoading={form.isSubmitting}
      >
        {element ? "Update" : "Create"}
      </Button>
    </>
  }
>
```

#### c) Form Element Structure

```typescript
// Repeated in ALL forms
<form ref={formRef} id={`edit-${type}-form`} onSubmit={form.handleSubmit} className="edit-form">
  {/* Fields */}
  {form.errors.submit && (
    <div className="text-red-500 text-sm mt-2">{form.errors.submit}</div>
  )}
</form>
```

---

### 2. **Description Extraction Logic (100% Duplication)**

The same complex type-checking logic appears in **every form component**:

```typescript
// Found in: EditSystemForm, EditPersonForm, EditContainerForm, EditComponentForm,
//           EditDataStoreForm, EditQueueForm, and others
description: typeof element?.description === "string"
  ? element.description
  : (element?.description as unknown as { txt: string })?.txt || "";
```

**Occurrences:** 14+ files, 2-3 times per file (initialValues + useEffect)

**Impact:** ~40 lines of duplicated code across all forms

---

### 3. **Form Reset Logic (100% Duplication)**

Identical `useEffect` pattern in all forms:

```typescript
// Repeated in ALL forms
useEffect(() => {
  if (isOpen) {
    form.setValues({
      name: element?.title || initialName || "",
      description: /* same complex extraction */,
      // ... other fields
    });
    form.clearErrors();
  }
}, [isOpen, element, initialName]); // eslint-disable-line react-hooks/exhaustive-deps
```

**Occurrences:** 14+ files

---

### 4. **ID Generation Logic (High Duplication - ~90% similar)**

Similar ID generation patterns across forms, despite `useElementFormSubmit` hook existing:

```typescript
// Pattern in EditSystemForm, EditPersonForm
let targetId = element?.id;
if (!element) {
  targetId = values.customId ? values.idInput.trim() : slugify(values.name) || "type";
  let i = 1;
  const originalId = targetId;
  while (newElements[targetId as string]) {
    targetId = `${originalId}-${i++}`;
  }
}

// Pattern in EditContainerForm, EditComponentForm, EditDataStoreForm, EditQueueForm
let targetId = element?.id;
if (!element) {
  const baseId = values.customId ? values.idInput : slugify(values.name) || "type";
  if (!values.selectedSystemId) return model;
  targetId = `${values.selectedSystemId}.${baseId}`;
  let i = 1;
  const originalId = targetId;
  while (newElements[targetId as string]) {
    targetId = `${originalId}-${i++}`;
  }
}
```

**Note:** `useElementFormSubmit.ts` exists but is **not used** by any form components.

---

### 5. **Validation Logic (High Duplication - ~70% similar)**

Similar validation patterns:

```typescript
// Repeated pattern
validate: (values) => {
  const errors: FormErrors = {};
  if (!values.name.trim()) errors.name = "Name is required";
  if (values.customId && !values.idInput.trim()) errors.idInput = "ID is required";
  if (values.customId && values.idInput.trim() && !element) {
    if (data?.elements?.[values.idInput.trim()]) {
      errors.idInput = "ID already exists";
    }
  }
  return errors;
};
```

**Variations:**

- Some forms check for parent system/container
- Some forms have additional field validations
- Core pattern is identical

---

### 6. **Element Creation/Update Logic (High Duplication - ~80% similar)**

Similar element creation patterns:

```typescript
// Pattern in EditSystemForm, EditPersonForm
newElements[targetId as string] = {
  id: targetId,
  kind: "type",
  title: values.name,
  description: typeof values.description === "string" ? values.description : undefined,
  tags: tags.length > 0 ? tags : undefined,
  links: element?.links,
  style: {},
};

// Pattern in EditContainerForm, EditComponentForm, EditDataStoreForm, EditQueueForm
newElements[targetId as string] = {
  id: targetId,
  kind: "container" | "component",
  title: values.name,
  description: typeof values.description === "string" ? values.description : undefined,
  technology: values.technology || undefined,
  tags: tags,
  links: element?.links,
  style: {},
};
```

---

### 7. **Form Initial Values Pattern (High Duplication - ~75% similar)**

Similar initial values setup:

```typescript
// Repeated pattern
initialValues: {
  name: element?.title || initialName || "",
  description: /* complex extraction */,
  customId: false,
  idInput: element?.id || "",
  // ... type-specific fields
}
```

---

## Quantification

### Lines of Code Analysis

| Component         | Total Lines | Duplicated Logic | Unique Logic | Duplication % |
| ----------------- | ----------- | ---------------- | ------------ | ------------- |
| EditSystemForm    | 196         | ~140             | ~56          | 71%           |
| EditPersonForm    | 181         | ~135             | ~46          | 75%           |
| EditContainerForm | 233         | ~180             | ~53          | 77%           |
| EditComponentForm | 252         | ~195             | ~57          | 77%           |
| EditDataStoreForm | 213         | ~165             | ~48          | 77%           |
| EditQueueForm     | 208         | ~160             | ~48          | 77%           |

**Total Estimated Duplication:** ~970 lines of duplicated code across 6 main forms

---

## Root Causes

1. **Incomplete Refactoring**: `useElementFormSubmit` hook exists but is not integrated
2. **Missing Utility Functions**: No shared utilities for:
   - Description extraction
   - Form reset logic
   - Common validation patterns
3. **No Base Form Component**: Each form reimplements the same structure
4. **Copy-Paste Development**: Forms were likely created by copying and modifying

---

## Recommendations

### Priority 1: Extract Common Utilities

1. **Create `formUtils.ts`**:

   ```typescript
   export function extractDescription(element?: ElementDump): string {
     if (!element?.description) return "";
     return typeof element.description === "string"
       ? element.description
       : (element.description as unknown as { txt: string })?.txt || "";
   }
   ```

2. **Create `useFormReset.ts` hook**:
   ```typescript
   export function useFormReset<T>(
     form: FormState<T>,
     isOpen: boolean,
     initialValues: T,
     dependencies: unknown[]
   ) {
     useEffect(() => {
       if (isOpen) {
         form.setValues(initialValues);
         form.clearErrors();
       }
     }, [isOpen, ...dependencies]);
   }
   ```

### Priority 2: Create Base Form Component

Create `BaseElementForm.tsx` that handles:

- SidePanel wrapper
- Form structure
- Footer buttons
- Error display
- Form submission wrapper

### Priority 3: Integrate Existing Hook

Refactor forms to use `useElementFormSubmit` hook that already exists.

### Priority 4: Create Form Configuration System

Use a configuration-driven approach:

```typescript
interface FormConfig {
  kind: string;
  fields: FieldConfig[];
  validation: ValidationConfig;
  idGeneration: IdGenerationConfig;
}
```

---

## Impact Assessment

### Maintenance Burden

- **High**: Changes to form behavior require updates in 14+ files
- **Risk**: Inconsistent behavior across forms
- **Time**: ~2-3x longer to implement form changes

### Testing Burden

- **High**: Each form needs separate tests for common logic
- **Risk**: Missing edge cases in some forms

### Code Quality

- **Low**: High duplication violates DRY principle
- **Risk**: Bugs fixed in one form may not be fixed in others

---

## Estimated Refactoring Effort

| Task                           | Estimated Hours | Priority |
| ------------------------------ | --------------- | -------- |
| Extract description utility    | 1               | P1       |
| Create form reset hook         | 2               | P1       |
| Create base form component     | 8               | P2       |
| Integrate useElementFormSubmit | 12              | P3       |
| Refactor all forms             | 16              | P3       |
| Testing                        | 8               | All      |
| **Total**                      | **47 hours**    |          |

---

## Next Steps

1. ✅ **Document duplication** (this document)
2. ⏳ Extract description utility function
3. ⏳ Create form reset hook
4. ⏳ Create base form component
5. ⏳ Refactor forms incrementally
6. ⏳ Update tests

---

## Notes

- `useElementFormSubmit.ts` exists but appears unused - investigate why
- Some forms have unique requirements (e.g., EditComponentForm needs parent container)
- Consider backward compatibility during refactoring
- Ensure TypeScript types are properly maintained
