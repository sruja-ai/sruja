# Change Apply vs Commit: Clarification

## Current Confusion

The documentation mentions both:
- `sruja change apply` - Apply changes to generate snapshot
- `sruja change commit` - Commit change (unclear purpose)

## Proposed Simplification

Since we removed the proposal workflow, we should simplify to just **"apply"**:

### Apply = Generate Current State

**`sruja change apply`** - Computationally applies changes to generate the current architecture state:

```bash
# Apply all changes to generate current.sruja
sruja change apply

# Apply changes up to specific version
sruja change apply --snapshot v1.2.0

# Apply specific change
sruja change apply --change "003-add-analytics"
```

**What it does**:
1. **Validates all changes are in final state** (approved or deferred, not in-progress)
2. **Validates all ADRs referenced by changes are in final state** (decided or rejected, not pending)
3. Loads base snapshot (e.g., `v1.0.0.sruja`)
4. Loads all changes (or up to specific version)
5. Applies each change sequentially
6. Generates `current.sruja` (or snapshot file)

**This is a computational operation** - it generates the current state from base + changes.

**Critical Requirements**: 
- All changes must be in **final state** (approved or deferred) before applying
- **All ADRs referenced by changes must be in final state** (decided or rejected) before applying
- Changes in "in-progress" or "pending" state cannot be applied
- ADRs in "pending" or "in-progress" state cannot be applied

## Remove "Commit"

**`sruja change commit`** should be **removed** because:
- It's confusing (what does "commit" mean vs "apply"?)
- It came from the removed proposal workflow
- Changes are just files - they don't need "committing"
- Git handles actual commits

## Simplified Workflow

### 1. Create Change File
```bash
sruja change create add-analytics --requirement "REQ-123"
# Creates: changes/001-add-analytics.sruja
```

### 2. Edit Change File
Edit `changes/001-add-analytics.sruja` to define what changes

### 3. Apply Change
```bash
sruja change apply
# Validates all changes are in final state (approved/deferred)
# Generates: current.sruja (base + all changes)
```

**Validation**: `change apply` will fail if:
- Any change is in "in-progress" or "pending" state
- Any ADR referenced by a change is in "pending" or "in-progress" state

All changes and their referenced ADRs must be in final state before applying.

### 4. Git Commit (External)
```bash
git add changes/001-add-analytics.sruja current.sruja
git commit -m "Add analytics change"
git push
```

## Recommendation

**Remove `sruja change commit`** - it's not needed. Just use:
- `sruja change create` - Create change file
- `sruja change apply` - Apply changes to generate current state
- `git commit` - Actual version control (external)

## Updated Commands

```bash
# Change management
sruja change create <name> [--requirement REQ]
sruja change apply [--snapshot VERSION] [--change ID]
sruja change validate [--all] [--change ID]
sruja change diff <change-id>
sruja change rollback <change-id>

# No "commit" command needed
```

## Change States

Changes can have the following states:

- **`pending`** - Change is being created/edited (not ready)
- **`in-progress`** - Change is being worked on (not ready)
- **`approved`** - Change is approved and ready to apply (final state)
- **`deferred`** - Change is deferred/postponed (final state, won't be applied)

**Rule**: Only changes in **final state** (approved or deferred) can be applied.

## Apply Validation

When running `sruja change apply`, the system will:
1. Check all changes are in final state (approved or deferred)
2. Check all ADRs referenced by changes are in final state (decided or rejected)
3. Reject if any change is in-progress or pending
4. Reject if any ADR is pending or in-progress
5. Show error listing which changes and ADRs are not in final state

```bash
$ sruja change apply
Error: Cannot apply changes - some changes or ADRs are not in final state:

Changes not in final state:
  - changes/003-add-analytics.sruja: status "in-progress"
  - changes/004-add-payment.sruja: status "pending"

ADRs not in final state:
  - ADR-001 (referenced by change 003-add-analytics): status "pending"
  - ADR-002 (referenced by change 004-add-payment): status "in-progress"

All changes and their referenced ADRs must be in final state (approved/deferred for changes, decided/rejected for ADRs) before applying.
```

## ADR States

ADRs can have the following states:

- **`pending`** - ADR is being created/edited (not ready)
- **`in-progress`** - ADR is being discussed/reviewed (not ready)
- **`decided`** - ADR decision is made (final state)
- **`rejected`** - ADR decision is rejected (final state)

**Rule**: Only ADRs in **final state** (decided or rejected) can be referenced by changes that are being applied.

## Benefits

✅ **Clearer**: "Apply" is unambiguous (computational operation)  
✅ **Simpler**: One less command to understand  
✅ **Familiar**: Matches database migration terminology  
✅ **Separation**: Git handles version control, Sruja handles architecture state  
✅ **Safe**: Prevents applying incomplete changes  

