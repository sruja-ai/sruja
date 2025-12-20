# Change Visualization Overview

## Overview

Complete system for tracking, visualizing, and managing architectural changes over time using a change-based model similar to database schema migrations.

## Architecture

### Components

1. **Go Backend** (CLI Commands)
   - Change file creation and management
   - Snapshot generation
   - Diff calculation
   - Timeline generation

2. **TypeScript Frontend** (Visualization)
   - Visual diff rendering
   - Snapshot viewer
   - Change timeline UI
   - Integration with viewer and studio

### Flow

```
DSL Files
    │
    ├─→ Change Files (001-add-analytics.sruja, 002-add-payment.sruja)
    │
    ├─→ Base Snapshot (v1.0.0.sruja)
    │
    └─→ Apply Changes
            │
            ├─→ Generate Current Snapshot (current.sruja)
            │
            ├─→ Export to JSON (with change metadata)
            │
            └─→ Visualize Changes (TypeScript)
                    ├─→ Diff View
                    ├─→ Snapshot View
                    └─→ Timeline View
```

## Features

### 1. Change Management (Go CLI)

#### Create Change
```bash
sruja change create add-analytics --requirement "REQ-123"
```

Creates: `changes/003-add-analytics.sruja`

#### Apply Changes
```bash
sruja change apply
sruja change apply --snapshot v1.2.0
```

Generates current snapshot from base + changes

#### View Changes
```bash
sruja diff v1.0.0 v2.1.0
sruja change diff 003-add-analytics
sruja timeline --summary
```

### 2. Snapshot Management (Go CLI)

#### Generate Snapshot
```bash
sruja snapshot create v1.1.0
```

#### View Snapshot
```bash
sruja snapshot view v1.1.0
```

### 3. Visual Diff (TypeScript)

Compare two architecture versions:

**Side-by-Side Mode**:
- Left: Previous architecture
- Right: Current architecture
- Highlighted changes

**Overlay Mode**:
- Single diagram with color coding:
  - 🟢 Green: Added elements
  - 🔴 Red: Removed elements
  - 🟡 Yellow: Modified elements
  - ⚪ Gray: Unchanged elements

**Integration**:
```
viewer.html?diff=v1.0.0..v2.1.0
viewer.html?compare=v1.0.0&with=v2.1.0
```

### 4. Snapshot Viewer (TypeScript)

View architecture at specific point in time:

```
viewer.html?version=v1.1.0
studio.html?version=v1.1.0
```

### 5. Change Timeline (TypeScript)

Visual timeline showing:
- All versions/snapshots
- When each change occurred
- What changed in each change
- Links to view snapshot or diff

**UI Components**:
- Vertical timeline
- Expandable items showing change details
- Click to view snapshot
- Click to view diff

### 6. Change View (TypeScript)

Show what a specific change modified:
- Highlight affected elements
- Show change scope (system/container/component)
- Link to requirements/user stories

## Data Model

### Change File Structure

```sruja
change "003-add-analytics" {
  version "v1.1.0"
  requirement "REQ-123"
  userStory "US-456"
  
  add {
    system ShopSystem {
      container AnalyticsAPI "Analytics API" {
        component Dashboard "Dashboard"
      }
    }
  }
  
  modify {
    system ShopSystem {
      // Add new relation
      WebApp -> AnalyticsAPI "Sends analytics data"
    }
  }
  
  remove {
    // Nothing removed in this change
  }
}
```

### Snapshot Structure

Snapshot is a full architecture file at that point in time:

```sruja
snapshot "v1.1.0" {
  date "2025-02-01"
  architecture "E-commerce Platform" {
    // Complete architecture state at v1.1.0
  }
}
```

### JSON with Change Metadata

```json
{
  "metadata": {
    "name": "E-commerce Platform",
    "version": "v2.1.0",
    "snapshots": [
      {
        "version": "v1.0.0",
        "date": "2025-01-01",
        "description": "Initial architecture"
      },
      {
        "version": "v1.1.0",
        "date": "2025-02-01",
        "changeId": "001-add-analytics",
        "description": "Added analytics dashboard"
      }
    ]
  },
  "architecture": {
    // Current architecture state
  }
}
```

## User Workflows

### Workflow 1: Create and Apply Change

```bash
# 1. Create change file
sruja change create add-analytics --requirement "REQ-123"

# 2. Edit change file (changes/003-add-analytics.sruja)
#    Add new elements/relations

# 3. Validate change
sruja change validate 003-add-analytics

# 4. Apply change
sruja change apply

# 5. Generate snapshot
sruja snapshot create v1.1.0

# 6. View changes
sruja diff v1.0.0 v1.1.0
```

### Workflow 2: Visualize Changes

```bash
# 1. Export current architecture to JSON
sruja export json current.sruja current.json

# 2. Export previous version
sruja export json snapshots/v1.0.0.sruja v1.0.0.json

# 3. Open diff view
open viewer.html?diff=v1.0.0..current
```

### Workflow 3: Explore Timeline

```bash
# 1. View timeline
sruja timeline --summary

# 2. View specific snapshot
sruja snapshot view v1.1.0

# 3. View diff between versions
sruja diff v1.0.0 v1.1.0

# 4. Open in viewer
open viewer.html?version=v1.1.0
open viewer.html?diff=v1.0.0..v1.1.0
```

## Integration Points

### Viewer Integration

URL Parameters:
- `?version=v1.1.0` - View snapshot
- `?diff=v1.0.0..v2.1.0` - View diff
- `?compare=v1.0.0&with=v2.1.0` - Compare two versions
- `?change=003-add-analytics` - View change details

### Studio Integration

- Timeline panel showing all versions
- Snapshot selector dropdown
- Diff mode toggle
- Change view sidebar

### CLI Integration

All commands integrated into main `sruja` CLI:
- `sruja change ...` - Change management
- `sruja snapshot ...` - Snapshot management
- `sruja diff ...` - Change viewing
- `sruja timeline ...` - Timeline view

## Implementation Tasks

### Go Tasks

- [ ] Task 1.5: Change Commands (CLI)
  - Change file creation
  - Change application
  - Snapshot generation
  - Diff calculation
  - Timeline generation

### TypeScript Tasks

- [ ] Task 3.8: Change Visualization
  - Visual diff rendering
  - Snapshot viewer
  - Change timeline UI
  - Change view
  - Viewer/Studio integration

## Acceptance Criteria

* [ ] Users can create and apply changes
* [ ] Users can generate snapshots at specific versions
* [ ] Users can view diff between any two versions (CLI)
* [ ] Users can view visual diff in browser
* [ ] Users can view timeline of all changes
* [ ] Users can view snapshot at specific version
* [ ] Users can view what a change modified
* [ ] All changes are tracked and queryable
* [ ] Round-trip: change → snapshot → change works
