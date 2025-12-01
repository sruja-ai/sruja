# DSL Changes Required for Simplified Plan

## Overview

This document captures all DSL syntax changes needed to support the simplified implementation plan focusing on core value.

## Required DSL Changes

### 1. Change Block Syntax (Sprint 5 - Task 1.5)

**New DSL construct**: `change` block for tracking architectural changes.

#### Syntax

```sruja
change "<change-id>" {
  version "<version>"
  requirement "<requirement-id>"
  adr "<adr-id>"  // Optional: Reference to ADR
  status "<status>"  // pending, in-progress, approved, deferred
  
  metadata {
    owner "<owner>"  // Single owner (person/team)
    stakeholders ["<stakeholder1>", "<stakeholder2>", ...]  // List of stakeholders
  }
  
  add {
    // Architecture elements to add
    system MySystem {
      container NewContainer {}
    }
    relation MySystem.ExistingContainer -> MySystem.NewContainer "Uses"
  }
  
  modify {
    // Architecture elements to modify
    container ExistingContainer {
      // Modified properties
      description "Updated description"
    }
  }
  
  remove {
    // Architecture elements to remove
    container OldContainer {}
  }
}
```

#### Fields

- **`change "<id>"`**: Change identifier (e.g., "001-add-analytics")
- **`version`**: Version this change applies to (e.g., "v1.1.0")
- **`requirement`**: Requirement ID this change addresses (optional)
- **`adr`**: ADR ID this change references (optional)
- **`status`**: Change state - `pending`, `in-progress`, `approved`, `deferred`
- **`metadata.owner`**: Single owner (person email or team name)
- **`metadata.stakeholders`**: List of stakeholders (people/teams)
- **`add`**: Block containing elements to add
- **`modify`**: Block containing elements to modify
- **`remove`**: Block containing elements to remove

#### Status Values

- **`pending`**: Change is being created/edited (not ready to apply)
- **`in-progress`**: Change is being worked on (not ready to apply)
- **`approved`**: Change is approved and ready to apply (final state)
- **`deferred`**: Change is deferred/postponed (final state, won't be applied)

**Rule**: Only changes in **final state** (`approved` or `deferred`) can be applied.

#### Example

```sruja
change "003-add-analytics" {
  version "v1.1.0"
  requirement "REQ-123"
  adr "ADR-001"
  status "approved"
  
  metadata {
    owner "alice@example.com"
    stakeholders ["bob@example.com", "charlie@example.com", "Platform Team"]
  }
  
  add {
    system ShopSystem {
      container AnalyticsAPI "Analytics API" {
        component MetricsCollector {}
      }
    }
    relation ShopSystem.WebApp -> ShopSystem.AnalyticsAPI "Sends metrics"
  }
  
  modify {
    container ShopSystem.WebApp {
      description "Now includes analytics tracking"
    }
  }
}
```

### 2. Metadata Block for Changes (Sprint 5 - Task 1.5)

**New DSL construct**: `metadata` block inside `change` blocks.

#### Syntax

```sruja
metadata {
  owner "<owner>"
  stakeholders ["<stakeholder1>", "<stakeholder2>", ...]
}
```

#### Fields

- **`owner`**: Single owner (string - person email or team name)
- **`stakeholders`**: List of stakeholders (array of strings)

#### Notes

- `owner` is a single value (not a list)
- `stakeholders` is a list (can be empty)
- Both are optional (but recommended for tracking)

### 3. ADR Status Tracking (Sprint 5 - Task 1.5)

**Enhancement**: ADR status must support final states for change validation.

#### Current ADR Syntax (Verify)

```sruja
adr "<id>" "<title>" {
  tags [<element-refs>]
  status "<status>"  // pending, in-progress, decided, rejected
  context "<context>"
  decision "<decision>"
  consequences ["<consequence1>", ...]
}
```

#### Required Status Values

- **`pending`**: ADR is being created/edited (not final)
- **`in-progress`**: ADR is being worked on (not final)
- **`decided`**: ADR is decided (final state)
- **`rejected`**: ADR is rejected (final state)

**Rule**: Only ADRs in **final state** (`decided` or `rejected`) can be referenced by changes that are being applied.

#### Validation Rule

When applying changes:
- All changes must be in final state (`approved` or `deferred`)
- **All ADRs referenced by changes must be in final state** (`decided` or `rejected`)

### 4. Snapshot Block Syntax (Sprint 5 - Task 1.5)

**New DSL construct**: `snapshot` block for point-in-time views.

#### Syntax

```sruja
snapshot "<snapshot-name>" {
  version "<version>"
  description "<description>"
  timestamp "<iso-timestamp>"
  
  // Full architecture definition
  architecture MyArchitecture {
    // ... architecture elements ...
  }
}
```

#### Fields

- **`snapshot "<name>"`**: Snapshot identifier
- **`version`**: Version this snapshot represents
- **`description`**: Description of this snapshot
- **`timestamp`**: ISO timestamp when snapshot was created
- **Architecture**: Full architecture definition at this point in time

#### Example

```sruja
snapshot "v1.0.0-initial" {
  version "v1.0.0"
  description "Initial architecture"
  timestamp "2024-01-15T10:00:00Z"
  
  architecture ShopSystem {
    system ShopSystem {
      container WebApp {}
      container Database {}
    }
  }
}
```

### 5. Preview Snapshot Syntax (Sprint 5 - Task 1.5)

**Special snapshot type**: Preview snapshots can include in-progress changes.

#### Syntax

```sruja
snapshot "preview-<name>" {
  version "<version>"
  description "<description>"
  preview true  // Indicates this is a preview snapshot
  changes ["<change-id1>", "<change-id2>", ...]  // Changes included (can be in-progress)
  
  // Architecture with changes applied (for visualization only)
  architecture MyArchitecture {
    // ... architecture elements ...
  }
}
```

#### Fields

- **`preview true`**: Marks this as a preview snapshot (not validated)
- **`changes`**: List of change IDs included (can include in-progress changes)

#### Notes

- Preview snapshots are **not validated** (can include in-progress changes)
- Used for visualization and "what if" scenarios
- Cannot be applied (only for viewing)

## Parser Changes Required

### 1. Change Block Parser

**File**: `pkg/language/parser.go`

**Changes**:
- Add `parseChangeBlock()` function
- Parse `change` keyword followed by identifier
- Parse `version`, `requirement`, `adr`, `status` fields
- Parse `metadata` block with `owner` and `stakeholders`
- Parse `add`, `modify`, `remove` blocks containing architecture elements

**AST Node**: Add `ChangeBlock` node type

### 2. Metadata Block Parser (for Changes)

**File**: `pkg/language/parser.go`

**Changes**:
- Extend existing metadata parser to support `owner` and `stakeholders` in change context
- `owner`: Single string value
- `stakeholders`: Array of strings

### 3. ADR Status Validation

**File**: `pkg/language/parser.go` or `pkg/engine/`

**Changes**:
- Validate ADR status values: `pending`, `in-progress`, `decided`, `rejected`
- Add validation rule: ADRs referenced by changes must be in final state when applying

### 4. Snapshot Block Parser

**File**: `pkg/language/parser.go`

**Changes**:
- Add `parseSnapshotBlock()` function
- Parse `snapshot` keyword followed by identifier
- Parse `version`, `description`, `timestamp` fields
- Parse `preview` flag (optional)
- Parse `changes` array (for preview snapshots)
- Parse full architecture definition inside snapshot

**AST Node**: Add `SnapshotBlock` node type

## Printer Changes Required

### 1. Change Block Printer

**File**: `pkg/language/printer.go`

**Changes**:
- Add `printChangeBlock()` function
- Print `change` block with all fields
- Print `metadata` block with `owner` and `stakeholders`
- Print `add`, `modify`, `remove` blocks

### 2. Snapshot Block Printer

**File**: `pkg/language/printer.go`

**Changes**:
- Add `printSnapshotBlock()` function
- Print `snapshot` block with all fields
- Print architecture definition inside snapshot

## Validation Rules

### 1. Change Validation

- Change status must be one of: `pending`, `in-progress`, `approved`, `deferred`
- Only changes in final state (`approved`, `deferred`) can be applied
- ADRs referenced by changes must exist
- ADRs referenced by changes must be in final state when applying

### 2. ADR Validation

- ADR status must be one of: `pending`, `in-progress`, `decided`, `rejected`
- ADRs referenced by changes must be in final state (`decided`, `rejected`) when applying changes

### 3. Snapshot Validation

- Snapshot version must be valid semantic version
- Preview snapshots are not validated (can include in-progress changes)
- Regular snapshots must only include approved changes

## Implementation Priority

### Sprint 5 (Week 9-10): Change Tracking

1. **Change Block Parser** (Task 1.5)
   - Parse `change` blocks
   - Parse `metadata` block in changes
   - Parse `add`, `modify`, `remove` blocks

2. **Change Block Printer** (Task 1.5)
   - Print `change` blocks
   - Print `metadata` block in changes

3. **Snapshot Block Parser** (Task 1.5)
   - Parse `snapshot` blocks
   - Parse preview snapshots

4. **Snapshot Block Printer** (Task 1.5)
   - Print `snapshot` blocks

5. **ADR Status Validation** (Task 1.5)
   - Validate ADR status values
   - Validate ADR final state when applying changes

## Testing Requirements

### Unit Tests

- [ ] Parse change block with all fields
- [ ] Parse change block with metadata (owner, stakeholders)
- [ ] Parse change block with add/modify/remove blocks
- [ ] Parse snapshot block
- [ ] Parse preview snapshot
- [ ] Print change block (round-trip)
- [ ] Print snapshot block (round-trip)
- [ ] Validate change status values
- [ ] Validate ADR status values
- [ ] Validate change application (final state check)
- [ ] Validate ADR final state when applying changes

### Integration Tests

- [ ] Create change file via CLI
- [ ] Apply change (approved state)
- [ ] Fail to apply change (in-progress state)
- [ ] Fail to apply change (ADR not in final state)
- [ ] Create snapshot
- [ ] Create preview snapshot with in-progress changes
- [ ] Round-trip: DSL → JSON → DSL for changes
- [ ] Round-trip: DSL → JSON → DSL for snapshots

## Migration Notes

### Existing Files

- Existing `.sruja` files don't need changes (backward compatible)
- Change files are new (in `changes/` directory)
- Snapshot files are new (in `snapshots/` directory)

### Backward Compatibility

- Parser must handle files without change/snapshot blocks
- Existing ADR syntax remains unchanged
- Only enhancement: ADR status validation for change application

## Summary

**New DSL Constructs**:
1. ✅ `change` block - For tracking architectural changes
2. ✅ `metadata` block in changes - For owner/stakeholders
3. ✅ `snapshot` block - For point-in-time views
4. ✅ `preview` snapshot - For visualization with in-progress changes

**Enhanced DSL Constructs**:
1. ✅ ADR status validation - Must be in final state when referenced by changes

**No Breaking Changes**:
- All changes are backward compatible
- Existing files continue to work
- New constructs are additive only

