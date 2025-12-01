# ADR UI Indicators: Pending Decisions with Multiple Choices

## Overview

When viewing changes or preview snapshots that reference ADRs with multiple choices (not yet decided), the UI must clearly indicate that the ADR is not finalized and show available choices.

## Problem

Preview snapshots can include changes that reference ADRs with multiple choices (pending decisions). Users need to understand:
1. The ADR is not finalized
2. There are multiple choices available
3. Discussions happen in external systems (GitHub, Cloud Studio)

## UI Requirements

### 1. Visual Indicators for Pending ADRs

**When ADR has multiple choices (status: pending/in-progress)**:

#### Element-Level Indicators
- **Warning badge** on elements affected by pending ADR
  - Yellow/orange icon (⚠️ or ⚡)
  - Tooltip: "Pending ADR decision affects this element"
  - Click to show ADR details panel

#### ADR Panel Indicators
- **Status badge**: "Pending Decision" (yellow/orange)
- **Warning message**: "This ADR has multiple choices - decision not yet finalized"
- **Choices section**: Display all available options
  ```typescript
  {
    status: "pending",
    choices: [
      { option: "PostgreSQL", pros: [...], cons: [...] },
      { option: "ClickHouse", pros: [...], cons: [...] }
    ]
  }
  ```
- **External discussion link**: 
  - "Discuss on GitHub" (links to GitHub issue/PR)
  - "Discuss in Cloud Studio" (links to Cloud Studio discussion)
- **Note**: "Pros/cons discussions happen in external systems"

### 2. ADR Details Panel

When viewing an ADR (especially in preview snapshots):

```typescript
// ADR Details Component
interface ADRDetails {
  id: string;
  title: string;
  status: "pending" | "in-progress" | "decided" | "rejected";
  choices?: Choice[];  // Multiple choices if pending
  decision?: string;   // Final decision if decided
  context: string;
  consequences?: string[];
  externalDiscussion?: {
    type: "github" | "cloud-studio";
    url: string;
  };
}

interface Choice {
  option: string;
  pros?: string[];
  cons?: string[];
  discussion?: string; // Link to external discussion
}
```

**UI Display**:

```tsx
// ADR Details Panel
function ADRDetailsPanel({ adr }: { adr: ADRDetails }) {
  return (
    <div className="adr-panel">
      <div className="adr-header">
        <h3>{adr.title}</h3>
        {adr.status === "pending" && (
          <Badge variant="warning">Pending Decision</Badge>
        )}
        {adr.status === "decided" && (
          <Badge variant="success">Decided</Badge>
        )}
      </div>
      
      {adr.status === "pending" && adr.choices && (
        <div className="adr-pending-section">
          <Alert variant="warning">
            <strong>This ADR has multiple choices - decision not yet finalized</strong>
            <p>Pros/cons discussions happen in external systems (GitHub comments or Cloud Studio)</p>
          </Alert>
          
          <div className="adr-choices">
            <h4>Available Choices:</h4>
            {adr.choices.map((choice, idx) => (
              <div key={idx} className="choice-card">
                <h5>{choice.option}</h5>
                {choice.pros && (
                  <div className="pros">
                    <strong>Pros:</strong>
                    <ul>{choice.pros.map((p, i) => <li key={i}>{p}</li>)}</ul>
                  </div>
                )}
                {choice.cons && (
                  <div className="cons">
                    <strong>Cons:</strong>
                    <ul>{choice.cons.map((c, i) => <li key={i}>{c}</li>)}</ul>
                  </div>
                )}
                {choice.discussion && (
                  <a href={choice.discussion} target="_blank">
                    Discuss on GitHub →
                  </a>
                )}
              </div>
            ))}
          </div>
          
          {adr.externalDiscussion && (
            <div className="external-discussion">
              <a href={adr.externalDiscussion.url} target="_blank">
                {adr.externalDiscussion.type === "github" 
                  ? "Discuss on GitHub →" 
                  : "Discuss in Cloud Studio →"}
              </a>
            </div>
          )}
        </div>
      )}
      
      {adr.status === "decided" && adr.decision && (
        <div className="adr-decision-section">
          <Alert variant="success">
            <strong>Decision:</strong> {adr.decision}
          </Alert>
          {adr.consequences && (
            <div className="consequences">
              <h4>Consequences:</h4>
              <ul>{adr.consequences.map((c, i) => <li key={i}>{c}</li>)}</ul>
            </div>
          )}
        </div>
      )}
      
      <div className="adr-context">
        <h4>Context:</h4>
        <p>{adr.context}</p>
      </div>
    </div>
  );
}
```

### 3. Preview Snapshot Warnings

When viewing preview snapshots:

```tsx
// Preview Snapshot Warning Banner
function PreviewSnapshotBanner({ snapshot }: { snapshot: PreviewSnapshot }) {
  const pendingADRs = snapshot.changes
    .flatMap(c => c.adrs)
    .filter(adr => adr.status === "pending" || adr.status === "in-progress");
  
  if (pendingADRs.length === 0) return null;
  
  return (
    <Alert variant="warning" className="preview-warning">
      <strong>Preview Snapshot - Contains Pending Decisions</strong>
      <p>
        This preview includes {pendingADRs.length} ADR(s) with multiple choices that are not yet finalized.
        These decisions may change before the changes are applied.
      </p>
      <ul>
        {pendingADRs.map(adr => (
          <li key={adr.id}>
            <a href={`#adr-${adr.id}`}>{adr.title}</a> - {adr.choices?.length || 0} choices pending
          </li>
        ))}
      </ul>
      <p>
        <strong>Note:</strong> Pros/cons discussions happen in external systems (GitHub comments or Cloud Studio).
      </p>
    </Alert>
  );
}
```

### 4. Change View Indicators

When viewing a change that references pending ADRs:

```tsx
// Change View with ADR Indicators
function ChangeView({ change }: { change: Change }) {
  const pendingADRs = change.adrs.filter(adr => 
    adr.status === "pending" || adr.status === "in-progress"
  );
  
  return (
    <div className="change-view">
      <ChangeHeader change={change} />
      
      {pendingADRs.length > 0 && (
        <Alert variant="warning">
          <strong>This change references {pendingADRs.length} pending ADR(s)</strong>
          <p>These ADRs have multiple choices and are not yet finalized.</p>
          <ul>
            {pendingADRs.map(adr => (
              <li key={adr.id}>
                <ADRLink adr={adr} /> - {adr.choices?.length || 0} choices available
              </li>
            ))}
          </ul>
        </Alert>
      )}
      
      <ChangeDetails change={change} />
    </div>
  );
}
```

### 5. Element Highlighting

Elements affected by pending ADRs should be visually distinct:

```tsx
// Element with Pending ADR Indicator
function ArchitectureElement({ element, pendingADRs }: Props) {
  const hasPendingADR = pendingADRs.some(adr => 
    adr.tags.some(tag => tag.value === element.id)
  );
  
  return (
    <div className={`element ${hasPendingADR ? 'pending-adr' : ''}`}>
      {hasPendingADR && (
        <Tooltip content="Affected by pending ADR decision">
          <WarningIcon className="pending-adr-icon" />
        </Tooltip>
      )}
      <ElementContent element={element} />
    </div>
  );
}
```

**CSS**:
```css
.element.pending-adr {
  border: 2px dashed #f59e0b; /* Orange/yellow dashed border */
  background-color: #fef3c7; /* Light yellow background */
}

.pending-adr-icon {
  color: #f59e0b;
  margin-left: 4px;
}
```

## JSON Schema Updates

### ADR in JSON

```json
{
  "id": "ADR-001",
  "title": "Choose database for analytics",
  "status": "pending",
  "context": "Need to store large volumes of analytics data",
  "choices": [
    {
      "option": "PostgreSQL",
      "pros": ["Familiar to team", "Good performance"],
      "cons": ["May need optimization for scale"],
      "discussion": "https://github.com/org/repo/issues/123"
    },
    {
      "option": "ClickHouse",
      "pros": ["Optimized for analytics", "Better performance at scale"],
      "cons": ["Team needs to learn", "More complex setup"],
      "discussion": "https://github.com/org/repo/issues/123"
    }
  ],
  "externalDiscussion": {
    "type": "github",
    "url": "https://github.com/org/repo/issues/123"
  },
  "tags": [
    { "type": "container", "value": "ShopSystem.AnalyticsAPI" }
  ]
}
```

### Change with Pending ADR

```json
{
  "id": "003-add-analytics",
  "status": "in-progress",
  "adrs": ["ADR-001"],
  "metadata": {
    "pendingADRs": [
      {
        "id": "ADR-001",
        "status": "pending",
        "choicesCount": 2,
        "warning": "This change references an ADR with multiple choices - decision not yet finalized"
      }
    ]
  }
}
```

## User Experience Flow

### Viewing Preview Snapshot

1. User opens preview snapshot
2. **Warning banner appears** at top: "Preview Snapshot - Contains Pending Decisions"
3. Lists all pending ADRs with choice counts
4. User clicks on ADR → ADR details panel opens
5. Shows all choices with pros/cons
6. Shows link to external discussion (GitHub/Cloud Studio)
7. Elements affected by pending ADR are highlighted (yellow/orange border)

### Viewing Change

1. User views change that references pending ADR
2. **Warning alert** in change view: "This change references X pending ADR(s)"
3. Lists pending ADRs with links
4. User can click to see ADR details
5. Clear indication that decision is not finalized

## Implementation Checklist

- [ ] Add ADR status indicators (pending/decided badges)
- [ ] Display multiple choices for pending ADRs
- [ ] Show warning messages for pending ADRs
- [ ] Add external discussion links (GitHub/Cloud Studio)
- [ ] Highlight elements affected by pending ADRs
- [ ] Preview snapshot warning banner
- [ ] Change view ADR warnings
- [ ] ADR details panel with choices display
- [ ] Tooltips explaining pending ADR status
- [ ] Clear messaging: "Pros/cons discussions happen in external systems"

## Benefits

✅ **Clear communication** - Users understand ADR is not finalized  
✅ **Transparency** - All choices are visible  
✅ **External integration** - Links to where discussions happen  
✅ **Visual clarity** - Pending ADRs are clearly marked  
✅ **Informed decisions** - Users see all options before deciding  


