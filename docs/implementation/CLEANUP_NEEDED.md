# Cleanup Needed Before Starting Implementation

## ✅ Cleanup Complete

All outdated files have been archived. See `archive/` directory.

## Files Removed or Archived

### 1. Outdated Proposal Files (Removed Feature)

**Status**: Proposals were removed, using ADRs + external systems (GitHub PRs, Cloud Studio) instead.

**Files to Remove**:
- `docs/implementation/go/task-1.6-proposal-commands.md` - Proposal commands (no longer needed)
- `docs/implementation/typescript/task-4.6-proposals.md` - Visual proposals (replaced by task-4.6-changes.md)
- `docs/implementation/go/teams-and-proposals.md` - Proposal workflow (outdated)

**Action**: ✅ **DONE** - Moved to `docs/implementation/archive/` (see archive/README.md for details)

### 2. Document Review Checklist

**File**: `docs/implementation/DOCUMENT_REVIEW_NEEDED.md`

**Status**: This appears to be a review checklist. Check if it's still relevant or if items are completed.

**Action**: Review and either:
- Delete if all items are addressed
- Update if some items still need work
- Move to archive if historical reference

### 3. Old Architecture Model Files

**Status**: We have simplified architecture model. Old complex model files may be outdated.

**Files to Check**:
- `docs/implementation/go/architecture-model.md` - Should reference simplified model
- `docs/implementation/go/architecture-changes.md` - Verify if still relevant
- `docs/implementation/go/architecture-isolation.md` - Verify if still relevant
- `docs/implementation/go/architecture-semantics.md` - Verify if still relevant
- `docs/implementation/go/architecture-simplification-summary.md` - May be redundant

**Action**: Review each file and either:
- Add deprecation note pointing to simplified model
- Delete if completely replaced
- Keep if still provides value

## Files to Update

### 1. Timeline References

**Check**: Does `timeline.md` reference Task 1.6 (proposals)?

**Action**: Remove any references to Task 1.6 if present.

### 2. README References

**Check**: Does `README.md` or `go/README.md` reference proposal files?

**Action**: Remove references to proposals, update to reflect changes workflow.

## Cleanup Actions Taken

### ✅ Completed

1. **Archived Proposal Files**:
   - ✅ Moved `task-1.6-proposal-commands.md` to `archive/`
   - ✅ Moved `task-4.6-proposals.md` to `archive/`
   - ✅ Moved `teams-and-proposals.md` to `archive/`
   - ✅ Created `archive/README.md` explaining why files were removed

2. **Timeline Check**:
   - ✅ Verified no Task 1.6 references in timeline
   - ✅ All task numbers are correct

3. **References**:
   - ✅ README files already mark Task 1.6 as REMOVED (strikethrough)
   - ✅ References point to SIMPLIFIED_CHANGE_WORKFLOW.md

### Optional (Can Do Later)

4. **Review DOCUMENT_REVIEW_NEEDED.md**:
   - Status: Review checklist - items may be completed or still relevant
   - Action: Can review during implementation if needed

### Optional (Can Do Later)

4. **Review Architecture Model Files**:
   - Add deprecation notes if needed
   - Consolidate if redundant

## Verification

After cleanup, verify:
- [ ] No references to "proposal" commands in active task files
- [ ] Timeline doesn't reference removed tasks
- [ ] README files don't reference removed features
- [ ] All active tasks are properly numbered (1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.7, 2.1, 3.x, 4.x, 5.x)

## Impact

**Low Risk**: These are documentation files only. Removing them won't affect code.

**Recommendation**: Do cleanup before starting Task 1.0 to avoid confusion.

