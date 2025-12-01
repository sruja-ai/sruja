# Implementation Simplification Summary

## Decision: Remove Proposal Workflow, Use ADRs + External Systems

**Date**: 2025-01-XX  
**Rationale**: Keep the language simple and focused on architecture, not project management.

## What Changed

### ❌ Removed: Proposal Workflow (Task 1.6)

**Removed Commands**:
- `sruja proposal create`
- `sruja proposal comment`
- `sruja proposal respond`
- `sruja proposal action-item`
- `sruja proposal review`
- `sruja proposal modify`
- `sruja proposal conflicts`
- `sruja proposal approve`
- `sruja proposal commit`

**Why Removed**:
1. **Too complex** for language core - adds project management features
2. **Duplicates ADR functionality** - ADRs already track decisions
3. **Comments don't belong in files** - better handled by external systems
4. **Studio can create changes directly** - no need for separate proposal workflow

### ✅ Kept: Essential Features

1. **Change/Migration Commands** (Task 1.5)
   - `sruja change create`
   - `sruja migration create`
   - `sruja migration apply`
   - `sruja snapshot create`
   - `sruja diff`

2. **ADRs** (already in language)
   - Track decisions
   - Link to elements via tags
   - Status tracking (pending/decided/rejected)

3. **Studio Changes** (Task 4.6, simplified)
   - Create changes visually
   - Export to change files
   - Link to ADRs and requirements
   - Visual diff

4. **External Systems**
   - GitHub PRs for reviews and discussions
   - Cloud Studio for collaboration (future)
   - Comments/reviews in external systems

## New Workflow

### Developer Workflow

1. **Create change in Studio**:
   ```bash
   sruja studio
   # Visually add/modify architecture
   # Export as change file
   ```

2. **Create ADR for decision**:
   ```sruja
   adr "ADR-001" "Use REST API" {
     tags [container "ShopSystem.AnalyticsAPI"]
     status "decided"
   }
   ```

3. **Create PR in GitHub**:
   - PR includes change file + ADR
   - Discussion happens in GitHub comments
   - Reviews happen in GitHub reviews

4. **After approval, commit**:
   ```bash
   sruja migration apply
   ```

### Team Workflow (Future: Cloud Studio)

1. **Create change in Cloud Studio**
2. **Create ADR in Cloud Studio**
3. **Discussion in Cloud Studio** (threads, mentions)
4. **Review in Cloud Studio** (approvals)
5. **Auto-generate PR** on approval
6. **Merge PR** → change committed

## Benefits

✅ **Simpler language**: No proposal workflow complexity  
✅ **Better separation**: Decisions in ADRs, discussions external  
✅ **Familiar tools**: Use GitHub PRs (everyone knows them)  
✅ **Flexible**: Can use GitHub, cloud studio, or other systems  
✅ **Maintainable**: Less code, less complexity  
✅ **Focused**: Language stays focused on architecture  

## Alignment with Objectives

| Objective | Impact |
|-----------|--------|
| **Empowering developers** | ✅ Better - simpler workflow, familiar tools |
| **Making language simple** | ✅ Better - removed complex proposal workflow |
| **Easier to maintain files** | ✅ Better - less complexity in language |

## Updated Files

1. **Created**:
   - `docs/implementation/go/SIMPLIFIED_CHANGE_WORKFLOW.md` - New workflow documentation
   - `docs/implementation/SIMPLIFICATION_SUMMARY.md` - This file

2. **Updated**:
   - `docs/implementation/go/README.md` - Removed Task 1.6 reference
   - `docs/implementation/README.md` - Updated phase summary
   - `docs/implementation/typescript/task-4.6-proposals.md` → `task-4.6-changes.md` - Simplified to changes only

3. **Deprecated** (keep for reference):
   - `docs/implementation/go/task-1.6-proposal-commands.md` - Marked as removed

## Next Steps

1. ✅ Document simplified workflow
2. ✅ Update implementation plan
3. ⏳ Update timeline (remove Task 1.6)
4. ⏳ Update success criteria
5. ⏳ Enhance ADR support (better linking, status tracking)

## Conclusion

**Simpler is better**. By removing the proposal workflow and using:
- **ADRs** for decisions (already in language)
- **Studio** for creating changes (visual editor)
- **External systems** for collaboration (GitHub, cloud studio)

We keep the language focused on architecture while still enabling collaboration through familiar, external tools.


