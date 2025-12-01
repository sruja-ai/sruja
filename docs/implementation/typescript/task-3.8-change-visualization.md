# Task 3.8: Change Visualization and Diff

**Priority**: 🟡 High (Enables architecture evolution tracking)
**Technology**: TypeScript
**Dependencies**: Task 3.1 (Viewer Core), Change model from Go

## Overview

Visualize architectural changes over time with diff highlighting, snapshots, and timeline views.

## Features

### 1. Visual Diff View

**Compare two architecture versions** side-by-side or overlay:
- Previous architecture (baseline)
- Current architecture (target)
- Highlight added elements (green)
- Highlight removed elements (red)
- Highlight modified elements (yellow)
- Show unchanged elements (gray/faded)

**Requirements, Stories, ADRs, Scenarios, Policies**:
- **Separate change tracking**: Track changes to these concepts independently
- **Visual indicators**: Show added/removed/modified requirements, stories, ADRs, scenarios, policies
- **Tag changes**: Highlight when concepts are re-tagged (linked to different elements)
- **Linked element highlighting**: When viewing an element, show which requirements/stories/ADRs changed that affect it
- **Concept panel**: Side panel showing changes to requirements, stories, ADRs, scenarios, policies separately
- **Filter by concept**: Filter view to show only architectural changes, only requirement changes, etc.

### 2. Snapshot View

**View architecture at specific point in time**:
- Select version from timeline
- Show architecture state at that version
- All elements, relations, flows as they were

### 3. Change Timeline

**Visual timeline of changes**:
- List of all migrations/snapshots
- When each change was made
- What changed in each migration
- Link to diff view
- Show pending proposals alongside committed changes

### 3.5. Proposal Visualization

**Visualize pending proposals**:
- Show all active proposals in timeline
- Highlight which team owns each proposal
- Show proposal status (draft, in-review, approved, rejected)
- Visual diff for each proposal
- Show conflicts between parallel proposals
- Filter proposals by team

### 4. Change-Based Visualization

**Visualize individual migrations**:
- Show what a specific migration adds/removes/modifies
- Highlight affected elements
- Show change scope (system/container/component level)

### 5. Proposal Visualization (Detailed Overlay)

**Visualize proposed changes overlaid on existing architecture** (more detailed than version comparison):

**Purpose**: Preview what changes would look like before applying them
- Show existing architecture (base state)
- Overlay proposed additions (green, with dashed borders for "new")
- Overlay proposed modifications (yellow/orange, highlighting what changes)
- Mark proposed removals (red, with strikethrough or fading)
- Show unchanged elements (normal appearance)

**Requirements, Stories, ADRs, Scenarios, Policies in Proposals**:
- **Show proposed concept changes**: Display added/removed/modified requirements, stories, ADRs, scenarios, policies
- **Tag impact visualization**: Show how new tags link concepts to elements (highlight affected elements)
- **Concept-to-element links**: Visual indicators showing which elements would be linked to new/modified concepts
- **Requirement traceability**: Show how proposed requirements would link to architectural elements
- **Story-to-flow mapping**: Show how proposed user stories would link to flows
- **ADR impact**: Show which elements would be affected by proposed ADRs
- **Scenario coverage**: Show which elements/scenarios would be added or modified
- **Policy changes**: Show proposed policy additions/modifications and their scope

**Key Differences from Version Comparison**:
- **Version Comparison**: Compares two completed snapshots (both are final states)
- **Proposal Visualization**: Shows proposed changes on top of current state (one is existing, one is proposal)

**Use Cases**:
- Review a change before applying it
- Preview architectural changes before applying
- Validate proposed changes against current architecture
- See detailed impact of a change
- **Preview snapshots**: View future state with in-progress changes and pending ADRs

**ADR Status in Preview Snapshots** (Critical):
- When viewing preview snapshots with pending ADRs (multiple choices):
  - **Clear visual indicator**: Warning badge, yellow/orange styling
  - **Status message**: "ADR has multiple choices - decision pending"
  - **Show all options**: Display all available choices/options
  - **External discussion link**: "Discuss on GitHub" or "Cloud Studio" link
  - **Affected elements**: Highlight which elements depend on this ADR decision
  - **Note**: Pros/cons discussions happen in external systems (GitHub comments, Cloud Studio)

**Visual Details**:
- **Proposed additions**: 
  - Green background/border
  - Dashed border style (indicating "not yet applied")
  - Label prefix: "[NEW]" or similar
  - Hover shows: "Will be added: <element details>"
  
- **Proposed modifications**:
  - Yellow/orange highlight
  - Show original value vs new value (side-by-side or tooltip)
  - Highlight changed properties specifically
  - Hover shows: "Will change: <old> → <new>"
  
- **Proposed removals**:
  - Red border/background
  - Strikethrough or fade effect
  - Label prefix: "[REMOVE]" or similar
  - Hover shows: "Will be removed: <element details>"
  
- **Affected relations**:
  - Show new connections with dashed green lines
  - Show removed connections with dashed red lines
  - Highlight modified relations

**Interaction**:
- Toggle overlay on/off
- Filter by change type (only show additions, only removals, etc.)
- Click on proposed element to see detailed change information
- Side panel showing change summary

## Implementation

### JSON Structure for Changes

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
        "migration": "001-add-analytics",
        "description": "Added analytics dashboard"
      },
      {
        "version": "v2.1.0",
        "date": "2025-03-01",
        "migration": "003-add-inventory",
        "description": "Added inventory system"
      }
    ]
  },
  "architecture": {
    // Current state
  }
}
```

### Diff Calculation

```typescript
interface ArchitectureDiff {
  // Architectural elements
  added: Element[];      // New elements (systems, containers, components)
  removed: Element[];    // Deleted elements
  modified: Element[];   // Changed elements
  unchanged: Element[];  // Unchanged elements
  
  // Relations and flows
  relationsAdded: Relation[];
  relationsRemoved: Relation[];
  relationsModified: Relation[];
  
  flowsAdded: Flow[];
  flowsRemoved: Flow[];
  flowsModified: Flow[];
  
  // Requirements, Stories, ADRs, Scenarios, Policies
  requirementsAdded: Requirement[];
  requirementsRemoved: Requirement[];
  requirementsModified: Requirement[];
  
  userStoriesAdded: UserStory[];
  userStoriesRemoved: UserStory[];
  userStoriesModified: UserStory[];
  
  adrsAdded: ADR[];
  adrsRemoved: ADR[];
  adrsModified: ADR[];
  
  scenariosAdded: Scenario[];
  scenariosRemoved: Scenario[];
  scenariosModified: Scenario[];
  
  policiesAdded: Policy[];
  policiesRemoved: Policy[];
  policiesModified: Policy[];
  
  // Tag changes (which concepts link to which elements)
  tagChanges: TagChange[];  // Changes in element-tag relationships
}

interface TagChange {
  elementId: string;        // Which element was tagged/untagged
  conceptType: 'requirement' | 'userStory' | 'adr' | 'scenario';
  conceptId: string;        // Which concept (REQ-123, US-456, etc.)
  change: 'added' | 'removed' | 'modified';
  oldTags?: Tag[];
  newTags?: Tag[];
}

function calculateDiff(
  previous: ArchitectureJSON,
  current: ArchitectureJSON
): ArchitectureDiff {
  // Compare by qualified ID
  // Track additions, removals, modifications for:
  // - Architectural elements
  // - Relations and flows
  // - Requirements, user stories, ADRs, scenarios, policies
  // - Tag relationships (which concepts link to which elements)
}
```

### Visual Diff Rendering

```typescript
interface DiffViewOptions {
  mode: 'side-by-side' | 'overlay';
  highlightChanges: boolean;
  showUnchanged: boolean;
}

function renderDiff(
  diff: ArchitectureDiff,
  options: DiffViewOptions
): void {
  // Render with color coding
  // - Added: green border/background
  // - Removed: red border/strikethrough
  // - Modified: yellow border
  // - Unchanged: gray/faded
}
```

### Proposal Visualization

**Detailed overlay of proposed changes on existing architecture**:

```typescript
interface ProposalDiff {
  base: ArchitectureJSON;           // Existing architecture
  proposal: ArchitectureJSON;       // Proposed architecture
  changes: ArchitectureDiff;        // Calculated changes (includes all concept types)
  change?: ChangeJSON;              // Optional: change payload that defines proposal
}

// ProposalDiff includes changes to:
// - Architectural elements (systems, containers, components)
// - Relations and flows
// - Requirements, user stories, ADRs, scenarios, policies
// - Tag relationships

interface ProposalViewOptions {
  // Architectural elements
  showAdditions: boolean;           // Show proposed additions
  showRemovals: boolean;            // Show proposed removals
  showModifications: boolean;       // Show proposed modifications
  showUnaffected: boolean;          // Show unchanged elements
  overlayMode: 'highlight' | 'detailed'; // Highlight mode or detailed diff
  
  // Concept visibility
  showRequirementChanges: boolean;  // Show requirement changes
  showStoryChanges: boolean;        // Show user story changes
  showADRChanges: boolean;          // Show ADR changes
  showScenarioChanges: boolean;     // Show scenario changes
  showPolicyChanges: boolean;       // Show policy changes
  showTagChanges: boolean;          // Show tag/link changes
}

interface ProposedElement {
  element: Element;
  changeType: 'added' | 'removed' | 'modified';
  originalValue?: any;              // For modifications
  newValue?: any;                   // For modifications/additions
  affectedProperties?: string[];    // Which properties change
}

function renderProposal(
  proposalDiff: ProposalDiff,
  options: ProposalViewOptions
): void {
  // Render base architecture
  // Overlay proposed changes with detailed styling:
  // - Added: green, dashed borders, "[NEW]" label
  // - Modified: yellow/orange, show old→new in tooltip
  // - Removed: red, strikethrough, "[REMOVE]" label
  // - Affected relations: dashed lines
  
  // Also render concept changes:
  // - Highlight elements that will be linked to new requirements/stories/ADRs
  // - Show tag changes (which concepts link to which elements)
  // - Display concept panel showing proposed requirements/stories/ADRs/scenarios/policies
}
```

### Snapshot Loading

```typescript
async function loadSnapshot(version: string): Promise<ArchitectureJSON> {
  // Load snapshot from file system or API
  // Generate from base snapshot + migrations up to that version
}
```

### Timeline View

```typescript
interface TimelineItem {
  version: string;
  date: string;
  migration?: string;
  description: string;
  changes: {
    added: number;
    removed: number;
    modified: number;
  };
}

function renderTimeline(snapshots: TimelineItem[]): void {
  // Vertical timeline showing all changes
  // Click to view snapshot or diff
}
```

## UI Components

### 1. Diff View Component

```typescript
<DiffView
  previous={previousArchitecture}
  current={currentArchitecture}
  mode="overlay"
  highlightChanges={true}
/>
```

**Features**:
- Toggle between side-by-side and overlay mode
- Filter by change type (added/removed/modified)
- Highlight changed elements
- Show change details on hover

### 2. Snapshot Selector

```typescript
<SnapshotSelector
  snapshots={snapshots}
  currentVersion={currentVersion}
  onSelect={loadSnapshot}
/>
```

**Features**:
- Dropdown or timeline view
- Show version, date, description
- Quick jump to any version

### 3. Change Timeline

```typescript
<ChangeTimeline
  snapshots={snapshots}
  onSelectVersion={loadSnapshot}
  onSelectDiff={(from, to) => showDiff(from, to)}
/>
```

**Features**:
- Visual timeline of all changes
- Show change statistics
- Click to view snapshot
- Click to view diff between versions

### 4. Change View

```typescript
<ChangeView
  migration={migration}
  previousArchitecture={previous}
  currentArchitecture={current}
/>
```

**Features**:
- Show what the migration changed
- Highlight affected elements
- Show change scope
- Link to requirements/user stories

### 5. Proposal Visualization Component

```typescript
<ProposalView
  baseArchitecture={currentArchitecture}
  proposal={proposedArchitecture}
  migration={migration}  // Optional: if proposal comes from migration
  options={{
    showAdditions: true,
    showRemovals: true,
    showModifications: true,
    showUnaffected: false,
    overlayMode: 'detailed'
  }}
/>
```

**Features**:
- **Overlay proposed changes** on existing architecture
- **Detailed change annotations**:
  - Green dashed borders for new elements
  - Yellow/orange highlights for modified elements
  - Red strikethrough for removed elements
  - Tooltips showing old→new values for modifications
- **Filter controls**: Toggle additions/removals/modifications/unchanged
- **Change summary panel**: Side panel listing all proposed changes
- **Impact analysis**: Show which elements are affected by the changes
- **Interactive elements**: Click on proposed element to see detailed change info
- **Comparison**: Toggle between "current" and "proposed" view
- **Apply preview**: See what the architecture would look like after applying

**Visual Details**:
- Proposed additions show with `[NEW]` prefix and dashed green border
- Proposed modifications show property changes in tooltip (e.g., "Label: 'Old' → 'New'")
- Proposed removals show with `[REMOVE]` prefix and faded/red appearance
- Affected relations show as dashed lines (green for new, red for removed)
- Unchanged elements can be faded or hidden completely

**Requirements, Stories, ADRs, Scenarios, Policies Visualization**:
- **Concept panel**: Dedicated panel showing proposed changes to requirements, stories, ADRs, scenarios, policies
- **Tag indicators**: Visual badges on elements showing which concepts would be linked (via tags)
- **New concept links**: Highlight elements that would be linked to new requirements/stories/ADRs
- **Removed concept links**: Show elements that would lose concept tags
- **Requirement traceability**: Visual connection lines from requirements to linked elements
- **Story-to-flow visualization**: Show how proposed user stories link to flows
- **ADR impact**: Highlight elements affected by proposed ADRs

**ADR Status Indicators** (Critical for Preview Snapshots):
- **Pending ADRs with multiple choices**: Clearly indicate ADR is not finalized
  - Visual indicator: Warning icon, yellow/orange border, "Pending Decision" badge
  - Show all available choices/options
  - Display message: "This ADR has multiple choices - decision not yet finalized"
  - Link to external discussion: "Discuss on GitHub" or "Discuss in Cloud Studio"
  - Show which elements are affected by this pending ADR
- **Decided ADRs**: Show final decision clearly
  - Visual indicator: Checkmark, green border, "Decided" badge
  - Show the chosen option/decision
  - Show consequences of the decision
- **Scenario coverage**: Show which elements would be covered by new/modified scenarios
- **Policy scope**: Visualize scope of proposed policies (which elements they apply to)

### 6. Concept Change Panel

```typescript
<ConceptChangePanel
  changes={{
    requirements: diff.requirementsAdded | diff.requirementsModified | diff.requirementsRemoved,
    userStories: diff.userStoriesAdded | diff.userStoriesModified | diff.userStoriesRemoved,
    adrs: diff.adrsAdded | diff.adrsModified | diff.adrsRemoved,
    scenarios: diff.scenariosAdded | diff.scenariosModified | diff.scenariosRemoved,
    policies: diff.policiesAdded | diff.policiesModified | diff.policiesRemoved,
    tagChanges: diff.tagChanges
  }}
  onSelectConcept={(type, id) => highlightLinkedElements(type, id)}
/>
```

**Features**:
- **Tabs or sections** for Requirements, User Stories, ADRs, Scenarios, Policies
- **Change indicators**: Color-coded badges (green=added, red=removed, yellow=modified)
- **Tag visualization**: Show which elements each concept links to
- **Click to highlight**: Click on concept to highlight linked elements in diagram
- **Filter by concept type**: Show only requirements changes, only stories, etc.
- **Search/filter**: Search for specific concepts by ID or description

### 7. Proposal View Component

```typescript
<ProposalView
  proposal={proposal}
  currentArchitecture={currentArchitecture}
  showConflicts={true}
/>
```

**Features**:
- **Show proposal status**: Draft, in-review, approved, rejected
- **Team ownership**: Display which team owns the proposal
- **Proposed changes**: Visual diff of proposed changes
- **Review comments**: Show review feedback from other teams
- **Conflicts**: Highlight conflicts with other proposals
- **Impact analysis**: Show which teams/elements are affected
- **Modification history**: Show how proposal changed during review
- **Approve/Reject actions**: UI for approving/rejecting proposals
- **Commit button**: Convert approved proposal to migration

### 8. Team Ownership Visualization

```typescript
<TeamOwnershipView
  architecture={architecture}
  showProposals={true}
/>
```

**Features**:
- **Color coding**: Elements colored by team ownership
- **Team legend**: Panel showing teams and their colors
- **Ownership indicators**: Badges showing team on elements
- **Proposal indicators**: Show pending proposals by team
- **Filter by team**: Show/hide elements by team
- **Team boundaries**: Visual grouping by team ownership

## Viewer Integration

Integrate into Sruja Viewer:

```typescript
// Viewer options
interface ViewerOptions {
  // ... existing options
  showChanges?: boolean;
  compareWith?: string;  // Version to compare with
  highlightChanges?: boolean;
  proposal?: string;     // Migration or proposal ID to visualize
  proposalMode?: 'overlay' | 'preview'; // Proposal visualization mode
}

// URL parameters

// View specific version
// ?version=v1.0.0

// Compare two completed versions (diff)
// ?compare=v1.0.0&with=v2.1.0
// ?diff=v1.0.0..v2.1.0

// View migration proposal (detailed overlay)
// ?proposal=migration-003-add-analytics
// ?proposal=migration-003-add-analytics&mode=overlay
// ?proposal=migration-003-add-analytics&mode=preview

// View change from change file
// ?change=changes/003-add-analytics.sruja
```

## Acceptance Criteria

### Version Comparison (Two Completed Snapshots)
* [ ] Visual diff view works (side-by-side and overlay)
* [ ] Added elements highlighted in green
* [ ] Removed elements highlighted in red
* [ ] Modified elements highlighted in yellow
* [ ] Snapshot view loads architecture at specific version
* [ ] Change timeline shows all versions
* [ ] Timeline links to snapshots and diffs

### Proposal Visualization (Detailed Overlay)
* [ ] Proposal view overlays proposed changes on existing architecture
* [ ] Proposed additions shown with green dashed borders and `[NEW]` prefix
* [ ] Proposed modifications show old→new values in tooltips
* [ ] Proposed removals shown with red strikethrough and `[REMOVE]` prefix
* [ ] Affected relations shown as dashed lines (green for new, red for removed)
* [ ] Filter controls work (toggle additions/removals/modifications/unchanged)
* [ ] Change summary panel lists all proposed changes
* [ ] Click on proposed element shows detailed change information
* [ ] Toggle between "current" and "proposed" preview works
* [ ] Can visualize change from change file

### Concept Changes (Requirements, Stories, ADRs, Scenarios, Policies)
* [ ] Track changes to requirements, user stories, ADRs, scenarios, policies separately
* [ ] Show added/removed/modified requirements in concept panel
* [ ] Show added/removed/modified user stories in concept panel
* [ ] Show added/removed/modified ADRs in concept panel
* [ ] Show added/removed/modified scenarios in concept panel
* [ ] Show added/removed/modified policies in concept panel
* [ ] Visual indicators (badges) on elements showing concept links
* [ ] Highlight elements linked to new/modified concepts
* [ ] Tag changes tracked (which concepts link to which elements)
* [ ] Click on concept highlights linked elements in diagram
* [ ] Filter by concept type (only requirements, only stories, etc.)
* [ ] Requirement traceability visualization (requirements → elements)
* [ ] Story-to-flow mapping visualization
* [ ] ADR impact visualization (which elements affected)
* [ ] Pending ADR indicators (warning badges, yellow/orange styling)
* [ ] Display multiple choices for pending ADRs
* [ ] Show external discussion links (GitHub/Cloud Studio)
* [ ] Preview snapshot warnings for pending ADRs
* [ ] Clear messaging: "Pros/cons discussions happen in external systems"
* [ ] See [ADR UI Indicators](ADR_UI_INDICATORS.md) for detailed requirements
* [ ] Scenario coverage visualization
* [ ] Policy scope visualization

### General
* [ ] Change view shows what changed (including concepts)
* [ ] Integration with viewer (URL parameters for diff and proposal)
* [ ] Performance: Handle large architectures efficiently
* [ ] Both comparison modes work: version diff and proposal overlay
* [ ] Concept changes visible in both diff and proposal views
