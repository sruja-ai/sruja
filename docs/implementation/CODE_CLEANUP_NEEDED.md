# Code Cleanup Needed Before Starting Implementation

## 🔴 Critical: Compilation Errors

### 1. `pkg/language/ast_postprocess.go` - Undefined Types

**Status**: Code doesn't compile - blocks all work

**Errors**:
```
pkg/language/ast_postprocess.go:47:11: item.Policy undefined (type ArchitectureItem has no field or method Policy)
pkg/language/ast_postprocess.go:473:10: undefined: ContextBlock
pkg/language/ast_postprocess.go:529:10: undefined: Aggregate
pkg/language/ast_postprocess.go:564:10: undefined: ValueObject
pkg/language/ast_postprocess.go:587:10: undefined: Policy
pkg/language/ast_postprocess.go:591:10: undefined: Flow
```

**Root Cause**:
- `ArchitectureItem` struct (line 185 in ast.go) does NOT have `Policy` or `Flow` fields
- Types `ContextBlock`, `Aggregate`, `ValueObject`, `Policy`, `Flow` are NOT defined in `ast.go`
- Code in `ast_postprocess.go` references these undefined types

**Action Required**: 
1. **Remove Policy handling** (lines 47-58 in ast_postprocess.go) - Policy not in ArchitectureItem
2. **Remove or fix ContextBlock** (line 473) - Type doesn't exist
3. **Remove or fix Aggregate** (line 529) - Type doesn't exist  
4. **Remove or fix ValueObject** (line 564) - Type doesn't exist
5. **Remove Policy PostProcess** (line 587) - Type doesn't exist
6. **Remove or fix Flow** (line 591) - Type doesn't exist, but Flow is referenced in ArchitectureItem (line 119)

**Quick Fix**: Comment out all references to undefined types, then verify what's actually needed.

## 🟡 Incomplete Features

### 2. Journey Feature (Removed but Test Remains)

**File**: `pkg/language/features_test.go:107`
**File**: `pkg/language/parser_journey_test.go`

**Status**: Feature was removed but test files remain

**Action**: 
- Remove or update `parser_journey_test.go` 
- Clean up `features_test.go` test placeholder

### 3. Policy Handling (Incomplete)

**Files**:
- `pkg/language/ast_postprocess.go:47-55` - References Policy but undefined
- `pkg/export/svg/svg.go:604` - Placeholder comment "future implementation"

**Status**: Policy feature partially implemented

**Action**: 
- Either complete Policy implementation or remove all references
- Decide if Policy is needed for simplified plan

### 4. Flow Handling (May be Incomplete)

**File**: `pkg/language/ast_postprocess.go:119-121`

**Status**: References `item.Flow` - verify if Flow type exists

**Action**: 
- Check if Flow type is defined
- Remove if not needed, fix if needed

### 5. DDD Types (ContextBlock, Aggregate, ValueObject)

**Files**: `pkg/language/ast_postprocess.go:473, 529, 564`

**Status**: Referenced but may be undefined or have wrong names

**Action**: 
- Verify if these types exist in `ast.go`
- Check if they're named differently (e.g., `Context` vs `ContextBlock`)
- Fix references or remove if not needed

### 6. Metadata Extraction (Incomplete)

**Files**: 
- `pkg/dx/explainer_metadata_test.go:12, 48, 102`

**Status**: Tests note that metadata extraction is not implemented for:
- DataStore
- Queue  
- Person

**Action**: 
- Either implement or update tests to reflect current state
- Not blocking for Task 1.0, but should be addressed

## 🟢 Optional Cleanup

### 7. Investigate D2 Command

**File**: `cmd/investigate_d2/main.go`

**Status**: Appears to be a debugging/investigation tool

**Action**: 
- Keep if useful for debugging
- Remove if no longer needed
- Move to `cmd/tools/` or similar if keeping

### 8. Placeholder Comments

**Files**:
- `pkg/export/svg/ui.go:120` - "Initial Placeholder"
- `pkg/export/json/json_types.go:106` - "Placeholder types"
- `pkg/export/svg/svg.go:604` - "placeholder for future implementation"

**Action**: 
- Review and either implement or remove
- Not blocking, but good to clean up

## Priority Actions

### Before Starting Task 1.0

1. **🔴 CRITICAL**: Fix compilation errors in `ast_postprocess.go`
   - Remove or fix undefined type references
   - Ensure code compiles
   - Run tests to verify nothing breaks

2. **🟡 HIGH**: Clean up removed Journey feature
   - Remove `parser_journey_test.go`
   - Clean up test placeholder in `features_test.go`

### During Task 1.0 (Can Fix While Working)

3. **🟡 MEDIUM**: Decide on Policy/Flow/DDD types
   - Either implement or remove references
   - Update placeholder comments

### After Task 1.0 (Nice to Have)

4. **🟢 LOW**: Complete metadata extraction
5. **🟢 LOW**: Clean up placeholder comments
6. **🟢 LOW**: Organize investigation tools

## Recommended Fix Strategy

### Step 1: Fix Compilation Errors

**Option A: Remove Undefined References** (Quick fix)
```go
// Comment out or remove Policy handling in ast_postprocess.go
// if item.Policy != nil {
//     ... (remove this block)
// }
```

**Option B: Find Correct Type Names** (Better fix)
- Search `ast.go` for actual type names
- Update references to match

**Option C: Implement Missing Types** (If needed)
- Only if these features are required
- Check if they're in simplified plan

### Step 2: Verify What's Actually Needed

Check `SIMPLIFIED_PLAN.md` to see if these features are needed:
- Policy - Not mentioned in simplified plan
- Flow - Not mentioned in simplified plan  
- ContextBlock/Aggregate/ValueObject - DDD features, check if needed

**Recommendation**: Remove references to features not in simplified plan.

## Verification

After cleanup:
- [ ] Code compiles: `go build ./pkg/...`
- [ ] Tests pass: `go test ./pkg/...`
- [ ] No undefined type references
- [ ] No broken imports
- [ ] Removed features are cleaned up

## Impact

**High Impact**: Compilation errors block all work
**Medium Impact**: Incomplete features may cause confusion
**Low Impact**: Placeholders and investigation tools don't block work

**Recommendation**: Fix compilation errors before starting Task 1.0.

