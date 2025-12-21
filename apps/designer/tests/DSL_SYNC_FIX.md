# DSL Sync Fix - Tab Switching Issue

## Problem
When loading an example for the first time, DSL is shown correctly. However, after switching tabs (away from Code tab and back), the DSL panel becomes blank.

## Root Cause
1. **Component Remounting**: When switching between view tabs (Code/Diagram/Builder), the `CodePanel` component unmounts and remounts
2. **State Initialization**: When `DSLPanel` remounts, the local state `dslSource` was initialized to empty string `""` instead of reading from the store
3. **Sync Effect Timing**: The sync effect depended on both `storeDslSource` and `dslSource`, which could cause issues when the component remounts with empty state

## Fix Applied

### 1. Proper State Initialization
Changed from:
```typescript
const [dslSource, setDslSourceLocal] = useState<string>("");
```

To:
```typescript
const [dslSource, setDslSourceLocal] = useState<string>(() => {
  // Get current store value at initialization time
  const currentStoreDsl = useArchitectureStore.getState().dslSource;
  return currentStoreDsl || "";
});
```

This ensures that when the component remounts, it immediately gets the current DSL source from the store.

### 2. Improved Sync Effect
- Removed `dslSource` from the dependency array to avoid circular updates
- Only depends on `storeDslSource` so it always syncs when the store changes
- Added better logging to track sync operations

### 3. Added Keys to CodePanel Sub-components
Added keys to ensure proper remounting:
```typescript
{activeTab === "dsl" && <DSLPanel key="dsl-panel" />}
```

## Testing
Verified with Playwright browser automation:
1. ✅ Load example - DSL shows correctly
2. ✅ Switch to Diagram tab - DSLPanel unmounts
3. ✅ Switch back to Code tab - DSLPanel remounts with content visible
4. ✅ DSL content persists after tab switches

## Files Modified
- `apps/designer/src/components/Panels/DSLPanel.tsx` - Fixed state initialization and sync logic
- `apps/designer/src/components/Panels/CodePanel.tsx` - Added keys for proper remounting
- `apps/designer/tests/dsl-sync.spec.ts` - Added e2e tests for this scenario

