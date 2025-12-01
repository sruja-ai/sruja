# Parallel Changes: Multiple Teams, Multiple In-Progress Changes

## Design Decision

**YES - Allow multiple in-progress changes owned by multiple teams**

This enables parallel development, which is essential for larger organizations where multiple teams work on different parts of the architecture simultaneously.

## Why Allow Parallel In-Progress Changes?

### Benefits

✅ **Parallel Development** - Teams don't block each other  
✅ **Faster Development** - Multiple teams can work simultaneously  
✅ **Realistic Workflow** - Matches how teams actually work  
✅ **Scalability** - Works for organizations with many teams  
✅ **Flexibility** - Teams can work on independent changes  

### Challenges

⚠️ **Conflict Detection** - Need to detect when changes overlap  
⚠️ **Coordination** - Teams need to be aware of other teams' changes  
⚠️ **Merge Strategy** - Need to handle conflicts when applying  

## Rules

### 1. Multiple In-Progress Changes Allowed

```bash
# Team A creates change
sruja change create add-analytics --owner "alice@example.com" --stakeholders "Analytics Team"
# Status: in-progress

# Team B creates change (parallel, allowed)
sruja change create add-payment --owner "bob@example.com" --stakeholders "Payment Team"
# Status: in-progress

# Both can exist simultaneously
```

### 2. Conflict Detection

When applying changes, detect conflicts:

```bash
$ sruja change apply
Warning: Potential conflicts detected between in-progress changes:

Conflicts:
  - change 003-add-analytics (owner: alice@example.com) modifies ShopSystem.API
  - change 004-add-payment (owner: bob@example.com) modifies ShopSystem.API

These changes modify overlapping elements. Consider coordinating before applying.
```

### 3. Apply Validation

When applying, only approved changes are applied (in-progress changes are skipped):

```bash
# Only approved changes are applied
sruja change apply
# Applies: 001-add-analytics (approved)
# Skips: 003-add-analytics (in-progress)
# Skips: 004-add-payment (in-progress)
```

### 4. Conflict Resolution

Conflicts are resolved through:
- **Git workflow** - Teams coordinate via PRs
- **Manual coordination** - Teams discuss and adjust changes
- **Sequential application** - Apply one change, then the other (if compatible)

## Change File Structure

Changes can specify team ownership:

```sruja
change "003-add-analytics" {
  version "v1.1.0"
  requirement "REQ-123"
  adr "ADR-001"
  status "in-progress"   // Can be in-progress while other teams work
  
  metadata {
    owner: "alice@example.com"  // Change owner
    stakeholders: ["Analytics Team", "bob@example.com"]  // Stakeholders
  }
  
  add {
    system ShopSystem {
      container AnalyticsAPI {}
    }
  }
}
```

## Conflict Detection

### What Constitutes a Conflict?

Changes conflict if they:
- Modify the same element (system/container/component)
- Add elements with the same qualified name
- Remove elements that the other change modifies
- Modify overlapping relations

### Conflict Detection Command

```bash
# Check for conflicts between changes
sruja change conflicts

# Check conflicts for specific change
sruja change conflicts 003-add-analytics

# Check conflicts between specific changes
sruja change conflicts 003-add-analytics 004-add-payment
```

**Output**:
```
Conflicts detected:

Change 003-add-analytics (Analytics Team) conflicts with:
  - 004-add-payment (Payment Team): Both modify ShopSystem.API

Change 004-add-payment (Payment Team) conflicts with:
  - 003-add-analytics (Analytics Team): Both modify ShopSystem.API
```

## Workflow Example

### Scenario: Two Teams Working in Parallel

**Team A (Analytics Team)**:
```bash
# Create change
sruja change create add-analytics --team "Analytics Team"
# Edit change file (status: in-progress)
# Work on change...
```

**Team B (Payment Team)**:
```bash
# Create change (parallel, allowed)
sruja change create add-payment --team "Payment Team"
# Edit change file (status: in-progress)
# Work on change...
```

**Both teams work independently** - no blocking.

**When ready to apply**:
```bash
# Team A approves their change
# Edit change file: status "approved"

# Team B approves their change
# Edit change file: status "approved"

# Check for conflicts
sruja change conflicts
# Shows if changes conflict

# Apply (only approved changes)
sruja change apply
# Applies both if no conflicts, or shows conflicts if they exist
```

## Conflict Resolution Strategies

### 1. Git-Based Coordination

Teams coordinate through Git PRs:
- Each team creates PR with their change
- Reviewers check for conflicts
- Teams adjust changes based on feedback
- Merge sequentially or together

### 2. Sequential Application

Apply changes one at a time:
```bash
# Apply Team A's change first
sruja change apply --change 003-add-analytics

# Team B updates their change based on new state
# Then apply Team B's change
sruja change apply --change 004-add-payment
```

### 3. Manual Coordination

Teams discuss and adjust:
- Teams communicate about overlapping changes
- One team adjusts their change to avoid conflict
- Both changes can then be applied

## Implementation

### Conflict Detection Logic

```go
func DetectConflicts(changes []Change) []Conflict {
    var conflicts []Conflict
    
    for i, change1 := range changes {
        for j, change2 := range changes {
            if i >= j {
                continue // Avoid duplicate checks
            }
            
            if hasOverlap(change1, change2) {
                conflicts = append(conflicts, Conflict{
                    Change1: change1,
                    Change2: change2,
                    Elements: findOverlappingElements(change1, change2),
                })
            }
        }
    }
    
    return conflicts
}

func hasOverlap(change1, change2 Change) bool {
    // Check if changes modify same elements
    elements1 := getAllModifiedElements(change1)
    elements2 := getAllModifiedElements(change2)
    
    return hasIntersection(elements1, elements2)
}
```

### Apply with Conflict Check

```go
func ApplyChanges(baseSnapshot string, targetVersion string) (*Architecture, error) {
    changes := loadChangesUpTo(targetVersion)
    
    // Validate final states (as before)
    // ...
    
    // Check for conflicts between approved changes
    approvedChanges := filterApproved(changes)
    conflicts := DetectConflicts(approvedChanges)
    
    if len(conflicts) > 0 {
        return nil, formatConflictError(conflicts)
    }
    
    // Apply changes
    // ...
}
```

## Benefits of This Approach

✅ **Enables Parallel Development** - Teams work independently  
✅ **Realistic** - Matches how teams actually work  
✅ **Scalable** - Works for many teams  
✅ **Flexible** - Teams coordinate as needed  
✅ **Safe** - Conflict detection prevents issues  
✅ **Clear** - Conflicts are visible before applying  

## Recommendations

1. **Allow parallel in-progress changes** - Essential for team productivity
2. **Detect conflicts** - Warn teams about overlapping changes
3. **Coordinate via Git** - Use PRs for coordination (external system)
4. **Apply only approved** - In-progress changes are skipped during apply
5. **Show conflicts clearly** - Help teams understand what overlaps

## Summary

**YES - Allow multiple in-progress changes by multiple teams**

- Teams can work in parallel
- Conflict detection warns about overlaps
- Apply only processes approved changes
- Teams coordinate through Git PRs
- This enables realistic, scalable workflows


