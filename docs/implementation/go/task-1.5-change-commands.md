# Task 1.5: Change Commands

**Note**: Metadata syntax uses no colon (consistent with constraints/conventions):
- `owner "value"` (not `owner: "value"`)
- See `PROPOSED_METADATA_SYNTAX_CHANGE.md` for details and Snapshot Management

**Priority**: 🟡 High (Enables change tracking)
**Technology**: Go
**Estimated Time**: 2-3 days
**Dependencies**: Task 1.1 (JSON Exporter), Change model

## Overview

CLI commands for managing architectural changes: changes, snapshots, and viewing changes over time.

**Note**: Multiple teams can have multiple in-progress changes simultaneously. See [Parallel Changes](PARALLEL_CHANGES.md) for details on conflict detection and coordination.

## Commands

### 1. Create Change

```bash
# Create new change file
sruja change create <change-name> --requirement "REQ-123" [--owner OWNER] [--stakeholders STAKEHOLDERS]

# Example
sruja change create add-analytics --requirement "REQ-123" --owner "alice@example.com" --stakeholders "bob@example.com,charlie@example.com"
# Creates: changes/003-add-analytics.sruja
```

**Output**: Creates change file with template:
```sruja
change "003-add-analytics" {
  version "v1.1.0"
  requirement "REQ-123"
  adr "ADR-001"  // ADR referenced by this change
  status "pending"  // pending, in-progress, approved, deferred
  
  metadata {
    owner "alice@example.com"  // Change owner (single person/team)
    stakeholders ["bob@example.com", "charlie@example.com", "Platform Team"]  // Stakeholders (list)
  }
  
  add {
    // Elements to add
  }
  
  modify {
    // Elements to modify
  }
  
  remove {
    // Elements to remove
  }
}
```

**Change States**:
- `pending` - Change is being created/edited (not ready to apply)
- `in-progress` - Change is being worked on (not ready to apply)
- `approved` - Change is approved and ready to apply (final state)
- `deferred` - Change is deferred/postponed (final state, won't be applied)

**Rule**: Only changes in **final state** (approved or deferred) can be applied.

### 2. Apply Changes

```bash
# Apply all changes to generate current snapshot
sruja change apply [--snapshot <version>]

# Apply changes up to specific version
sruja change apply --snapshot v1.2.0

# Apply specific change
sruja change apply --change "003-add-analytics"
```

**Output**: Generates `current.sruja` (or snapshot file)

**Validation**: 
- ✅ All changes must be in **final state** (approved or deferred)
- ✅ **All ADRs referenced by changes must be in final state** (approved or deferred)
- ❌ Changes in "in-progress" or "pending" state will cause apply to fail
- ❌ ADRs in "pending" or "in-progress" state will cause apply to fail
- Error message lists which changes and ADRs are not in final state

**Example Error**:
```bash
$ sruja change apply
Error: Cannot apply changes - some changes or ADRs are not in final state:

Changes not in final state:
  - changes/003-add-analytics.sruja: status "in-progress"
  - changes/004-add-payment.sruja: status "pending"

ADRs not in final state:
  - ADR-001 (referenced by change 003-add-analytics): status "pending"
  - ADR-002 (referenced by change 004-add-payment): status "in-progress"

All changes and their referenced ADRs must be in final state (approved or deferred) before applying.
```

### 3. Generate Snapshot

```bash
# Generate snapshot at current state (only approved changes)
sruja snapshot create <version>

# Example
sruja snapshot create v1.1.0
# Creates: snapshots/v1.1.0.sruja
```

### 3.1. Generate Preview Snapshot (Visualization Only)

```bash
# Create preview snapshot with selected changes (including in-progress)
sruja snapshot preview <name> --changes <change1,change2,...>

# Example: Preview with in-progress changes
sruja snapshot preview "future-with-analytics" --changes 003-add-analytics,004-add-payment

# Example: Preview with specific approved + in-progress changes
sruja snapshot preview "q2-roadmap" --changes 001-add-analytics,003-add-payment,005-add-inventory

# Creates: snapshots/preview/future-with-analytics.sruja
```

**Preview Snapshots**:
- ✅ Can include in-progress changes (for visualization)
- ✅ Can include pending changes (for visualization)
- ✅ Can mix approved and in-progress changes
- ⚠️ **Visualization only** - Not applied to current state
- ⚠️ **Not validated** - ADR states not checked (preview only)
- ✅ Can be shared/viewed in Studio/Viewer
- ✅ Useful for "what if" scenarios and planning
- ⚠️ **Pending ADRs**: UI must clearly indicate when ADRs have multiple choices (not finalized)
  - Show warning badges and status indicators
  - Display all available choices/options
  - Link to external discussions (GitHub/Cloud Studio)
  - Note: Pros/cons discussions happen in external systems

**Use Cases**:
- Preview future architecture state
- Share "what if" scenarios with stakeholders
- Visualize combined effects of multiple changes
- Plan and coordinate between teams

### 4. View Changes

```bash
# View diff between two versions
sruja diff <version1> <version2>

# View diff between snapshot and current
sruja diff v1.0.0 current

# View what changed in a change
sruja change diff <change-name>

# View changes for a requirement
sruja changes --requirement "REQ-123"
```

### 5. Timeline View

```bash
# List all versions/snapshots
sruja timeline

# Show timeline with change summary
sruja timeline --summary

# Output:
# v1.0.0 (2025-01-01) - Initial architecture
# v1.1.0 (2025-02-01) - Added analytics (change 001)
#   - Added: ShopSystem.AnalyticsAPI
#   - Modified: ShopSystem
# v1.2.0 (2025-03-01) - Added payment (change 002)
#   - Added: PaymentSystem
```

### 6. Validate Change

```bash
# Validate change can be applied
sruja change validate <change-name>

# Validate all changes
sruja change validate --all
```

### 7. Check Conflicts

```bash
# Check for conflicts between changes
sruja change conflicts

# Check conflicts for specific change
sruja change conflicts <change-name>

# Check conflicts between specific changes
sruja change conflicts <change1> <change2>
```

**Output**: Shows conflicts between changes (overlapping elements, same qualified names, etc.)

**Note**: Multiple teams can have in-progress changes simultaneously. Conflicts are detected when applying approved changes. See [Parallel Changes](PARALLEL_CHANGES.md) for details.

### 9. Rollback Change

```bash
# Rollback specific change
sruja change rollback <change-name>

# Rollback to specific version
sruja rollback <version>
```

## File Structure

```
architecture/
  ├── v1.0.0.sruja              # Initial snapshot
  ├── snapshots/
  │   ├── v1.0.0.sruja          # Snapshot at v1.0.0
  │   ├── v1.1.0.sruja          # Snapshot at v1.1.0
  │   ├── v1.2.0.sruja          # Snapshot at v1.2.0
  │   └── preview/              # Preview snapshots (visualization only)
  │       ├── future-state.sruja
  │       ├── q2-roadmap.sruja
  │       └── both-teams-complete.sruja
  ├── changes/
  │   ├── 001-add-analytics.sruja
  │   ├── 002-add-payment.sruja
  │   └── 003-add-inventory.sruja
  ├── current.sruja             # Current state (generated)
  └── .sruja-changes.json       # Change metadata
```

## Change Metadata File

```json
{
  "currentVersion": "v1.3.0",
  "snapshots": [
    {
      "version": "v1.0.0",
      "date": "2025-01-01",
      "snapshotFile": "snapshots/v1.0.0.sruja"
    }
  ],
  "changes": [
    {
      "id": "001-add-analytics",
      "version": "v1.1.0",
      "date": "2025-02-01",
      "file": "changes/001-add-analytics.sruja",
      "requirement": "REQ-123",
      "status": "approved",
      "applied": true,
      "owner": "alice@example.com",
      "stakeholders": ["bob@example.com", "charlie@example.com", "Platform Team"]
    },
    {
      "id": "002-add-payment",
      "version": "v1.2.0",
      "date": "2025-03-01",
      "file": "changes/002-add-payment.sruja",
      "requirement": "REQ-124",
      "applied": true,
      "owner": "dave@example.com",
      "stakeholders": ["eve@example.com"]
    }
  ]
}
```

## Implementation

### Change File Creation

```go
func CreateChange(name string, requirement string, owner string, stakeholders []string) error {
    // Generate change number (next in sequence)
    changeNum := getNextChangeNumber()
    changeID := fmt.Sprintf("%03d-%s", changeNum, name)
    
    // Create change file with template
    template := generateChangeTemplate(changeID, requirement, owner, stakeholders)
    
    // Write to changes/ directory
    return writeChangeFile(changeID, template)
}

func generateChangeTemplate(changeID string, requirement string, owner string, stakeholders []string) string {
    // Generate DSL template with metadata
    // Includes owner and stakeholders in metadata block
}
```

### Apply Changes

```go
func ApplyChanges(baseSnapshot string, targetVersion string) (*Architecture, error) {
    // Load base snapshot
    baseArch := loadSnapshot(baseSnapshot)
    
    // Load and apply changes up to target version
    changes := loadChangesUpTo(targetVersion)
    
    // Validate all changes are in final state
    var changesNotFinal []string
    for _, change := range changes {
        if change.Status != "approved" && change.Status != "deferred" {
            changesNotFinal = append(changesNotFinal, fmt.Sprintf("%s: status \"%s\"", change.File, change.Status))
        }
    }
    
    // Validate all ADRs referenced by changes are in final state
    var adrsNotFinal []string
    allADRs := loadAllADRs()
    adrMap := make(map[string]*ADR)
    for _, adr := range allADRs {
        adrMap[adr.ID] = adr
    }
    
    for _, change := range changes {
        for _, adrID := range change.ADRs {
            adr, exists := adrMap[adrID]
            if !exists {
                return nil, fmt.Errorf("ADR %s referenced by change %s not found", adrID, change.ID)
            }
            if adr.Status != "decided" && adr.Status != "rejected" {
                adrsNotFinal = append(adrsNotFinal, fmt.Sprintf("%s (referenced by change %s): status \"%s\"", adrID, change.ID, adr.Status))
            }
        }
    }
    
    // Build error message if validation fails
    if len(changesNotFinal) > 0 || len(adrsNotFinal) > 0 {
        var errorParts []string
        errorParts = append(errorParts, "Cannot apply changes - some changes or ADRs are not in final state:\n")
        
        if len(changesNotFinal) > 0 {
            errorParts = append(errorParts, "\nChanges not in final state:")
            for _, msg := range changesNotFinal {
                errorParts = append(errorParts, "  - "+msg)
            }
        }
        
        if len(adrsNotFinal) > 0 {
            errorParts = append(errorParts, "\nADRs not in final state:")
            for _, msg := range adrsNotFinal {
                errorParts = append(errorParts, "  - "+msg)
            }
        }
        
        errorParts = append(errorParts, "\nAll changes and their referenced ADRs must be in final state (approved/deferred for changes, decided/rejected for ADRs) before applying.")
        
        return nil, fmt.Errorf(strings.Join(errorParts, "\n"))
    }
    
    currentArch := baseArch
    for _, change := range changes {
        // Only apply approved changes (skip deferred)
        if change.Status == "approved" {
            currentArch = applyChange(currentArch, change)
        }
    }
    
    // Generate current.sruja or snapshot
    return currentArch, nil
}
```

### Generate Preview Snapshot

```go
func GeneratePreviewSnapshot(name string, changeIDs []string) (*Architecture, error) {
    // Load base snapshot (current or specified version)
    baseArch := loadSnapshot("current") // or loadSnapshot(baseVersion)
    
    // Load selected changes (including in-progress - no validation)
    changes := loadChangesByIDs(changeIDs)
    
    // Apply changes (no validation - preview only)
    currentArch := baseArch
    for _, change := range changes {
        // Apply regardless of status (preview only)
        // This allows visualizing in-progress changes
        currentArch = applyChange(currentArch, change)
    }
    
    // Save as preview snapshot
    previewPath := fmt.Sprintf("snapshots/preview/%s.sruja", name)
    return currentArch, saveSnapshot(previewPath, currentArch)
}
```

**Key Differences from Regular Apply**:
- ✅ No validation of change states (can include in-progress)
- ✅ No validation of ADR states (preview only)
- ✅ Applies all selected changes regardless of status
- ✅ Saved in `snapshots/preview/` directory
- ⚠️ For visualization only - not applied to current state

### Generate Diff

```go
func GenerateDiff(version1 string, version2 string) (*ArchitectureDiff, error) {
    arch1 := loadSnapshot(version1)
    arch2 := loadSnapshot(version2)
    
    diff := &ArchitectureDiff{
        Added:    findAddedElements(arch1, arch2),
        Removed:  findRemovedElements(arch1, arch2),
        Modified: findModifiedElements(arch1, arch2),
        Unchanged: findUnchangedElements(arch1, arch2),
    }
    
    return diff, nil
}
```

## CLI Command Structure

```bash
# Change management
sruja change create <name> [--requirement REQ] [--team TEAM]
sruja change apply [--snapshot VERSION] [--change ID]
sruja change validate [--all] [--change ID]
sruja change conflicts [<change1>] [<change2>]
sruja change diff <change-id>
sruja change rollback <change-id>

# Snapshot management
sruja snapshot create <version>
sruja snapshot preview <name> --changes <change1,change2,...>
sruja snapshot list
sruja snapshot view <version>

# Change viewing
sruja diff <version1> <version2>
sruja changes [--requirement REQ] [--version VERSION]
sruja timeline [--summary]

# Rollback
sruja rollback <version>
```

## Acceptance Criteria

* [ ] `change create` command creates change file
* [ ] `change apply` validates all changes are in final state (approved/deferred)
* [ ] `change apply` validates all ADRs referenced by changes are in final state (decided/rejected)
* [ ] `change apply` rejects changes in in-progress or pending state
* [ ] `change apply` rejects ADRs in pending or in-progress state
* [ ] `change apply` detects conflicts between approved changes
* [ ] `change apply` shows clear error messages listing both changes and ADRs not in final state
* [ ] `change apply` shows conflicts if detected between approved changes
* [ ] `change apply` applies only approved changes (skips deferred and in-progress)
* [ ] `change apply` generates snapshot
* [ ] `change conflicts` detects conflicts between changes
* [ ] Multiple teams can have in-progress changes simultaneously
* [ ] `snapshot create` generates snapshot at specific version (only approved changes)
* [ ] `snapshot preview` creates preview snapshot with selected changes (can include in-progress)
* [ ] `snapshot preview` allows selecting multiple changes (approved + in-progress)
* [ ] Preview snapshots are saved in `snapshots/preview/` directory
* [ ] Preview snapshots can be viewed in Studio/Viewer
* [ ] Preview snapshots can be exported to HTML for sharing
* [ ] Preview snapshots skip validation (can include in-progress changes and pending ADRs)
* [ ] `diff` command shows changes between versions
* [ ] `timeline` command shows all versions
* [ ] `change validate` validates changes can be applied
* [ ] `rollback` command can rollback to previous version
* [ ] Change metadata file is maintained
* [ ] All changes are tracked and ordered
* [ ] Round-trip: change → snapshot → change works
* [ ] Error handling follows [Error Reporting Strategy](../ERROR_REPORTING_STRATEGY.md)
* [ ] Validation errors include clear messages and suggestions
* [ ] **Manual testing**: Test change workflow end-to-end with real scenarios
* [ ] **MCP-based testing**: Verify change semantics are preserved correctly

