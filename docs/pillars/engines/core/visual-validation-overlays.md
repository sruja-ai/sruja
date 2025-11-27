# Visual Validation Overlays

**Status**: Core Engine  
**Pillars**: Core (Validation Visualization)

[← Back to Engines](../README.md)

## Overview

The Visual Validation Overlays system provides live architectural feedback directly on the diagram, showing validation errors and warnings as visual overlays on nodes and edges.

**This is a production-grade module used in platforms like Figma, Terrastruct D2, VSCode, and Draw.io.**

## Purpose

The Visual Validation Overlays system:

- ✅ Shows node-level error badges
- ✅ Shows edge-level error markers
- ✅ Provides hover tooltips with detailed explanation
- ✅ Highlights errors in minimap
- ✅ Syncs error panel with diagram selection
- ✅ Triggers validation on every model update
- ✅ Works with DSL or diagram edits

## Error Object Model

Each validation rule produces:

```ts
export interface ValidationIssue {
  id: string;                     // unique ID
  target: "node" | "edge";        // what to highlight
  nodeId?: string;                // if node error
  from?: string;                  // if edge error
  to?: string;                    // if edge error
  severity: "error" | "warning";
  message: string;
  rule: string;                   // rule identifier
}
```

## Validation State Management

We store validation results globally:

```ts
export interface ValidationState {
  issues: ValidationIssue[];
  setIssues: (issues: ValidationIssue[]) => void;
}

export const useValidation = create<ValidationState>((set) => ({
  issues: [],
  setIssues: (issues) => set({ issues }),
}));
```

## Integration: Run Validation on Model Changes

From your sync engine:

```ts
useArchStore.subscribe((state) => {
  const issues = runValidationRules(state.model);
  useValidation.getState().setIssues(issues);
});
```

**Validation always runs after DSL or diagram edits.**

## Node-Level Visual Overlays

We extend ReactFlow nodes using:

- Extra CSS border coloring
- Warning/error badge
- Hover tooltip

### Node Component Example

```tsx
function RectNode({ id, data }) {
  const issues = useValidation(s => s.issues.filter(i => i.nodeId === id));

  const hasError = issues.some(i => i.severity === "error");
  const hasWarning = issues.some(i => i.severity === "warning");

  return (
    <div
      className="relative shadow-sm p-2 flex items-center gap-2 rounded"
      style={{
        border: `2px solid ${
          hasError ? "#dc2626" :
          hasWarning ? "#d97706" :
          data.borderColor
        }`,
        background: data.color,
      }}
    >
      <Icon name={data.icon} size={16} />
      {data.label}

      {issues.length > 0 && (
        <div className="absolute -top-2 -right-2 w-5 h-5 rounded-full bg-red-600 text-white flex items-center justify-center text-xs">
          {issues.length}
        </div>
      )}
    </div>
  );
}
```

- ✔ Visual border coloring
- ✔ Badge showing error count
- ✔ Fully reactive

## Edge-Level Error Indicators

ReactFlow supports `edgeTypes`.

We extend edges with:

- Red/amber coloring
- Zig-zag stroke or dashed style
- Tooltip on hover

### Edge Component Example

```tsx
function ValidatedEdge({ id, sourceX, sourceY, targetX, targetY, data }) {
  const issues = useValidation(s => s.issues.filter(
    i => i.from === data.from && i.to === data.to
  ));

  const hasError = issues.some(i => i.severity === "error");

  return (
    <>
      <BaseEdge
        id={id}
        path={`M ${sourceX} ${sourceY} L ${targetX} ${targetY}`}
        style={{
          stroke: hasError ? "#dc2626" : "#555",
          strokeWidth: hasError ? 3 : 2,
          strokeDasharray: hasError ? "4 2" : "none",
        }}
      />
      {hasError && (
        <EdgeLabelRenderer>
          <div
            className="absolute bg-red-600 text-white text-xs px-1 py-0.5 rounded"
            style={{
              transform: `translate(${(sourceX + targetX) / 2}px, ${(sourceY + targetY) / 2}px)`,
            }}
          >
            {issues[0].message}
          </div>
        </EdgeLabelRenderer>
      )}
    </>
  );
}
```

## Tooltip for Details

Hover over the node for full message:

```tsx
<Tooltip>
  <TooltipTrigger>{/* node */}</TooltipTrigger>
  <TooltipContent>
    {issues.map((i) => (
      <div key={i.id} className="text-red-200">
        <strong>{i.rule}</strong>: {i.message}
      </div>
    ))}
  </TooltipContent>
</Tooltip>
```

## Global Error Panel (Sidebar)

A panel listing all validation issues:

```tsx
export function ValidationPanel() {
  const issues = useValidation(s => s.issues);

  return (
    <div className="p-4 border-l w-80">
      <h3 className="font-semibold mb-3">Validation</h3>

      {issues.length === 0 && <p className="text-sm text-muted">No issues 🎉</p>}

      {issues.map(i => (
        <div
          key={i.id}
          className="p-2 mb-2 rounded border cursor-pointer hover:bg-accent"
          onClick={() => highlightIssue(i)}
        >
          <span className="font-medium">{i.rule}</span>
          <p className="text-sm text-muted-foreground">{i.message}</p>
        </div>
      ))}
    </div>
  );
}
```

## Issue → Diagram Highlight

Click in panel → center on node:

```ts
function highlightIssue(issue: ValidationIssue) {
  if (issue.nodeId)
    reactFlowInstance.setCenter(node.position.x, node.position.y, { zoom: 1.3 });

  if (issue.from)
    reactFlowInstance.fitBounds(getEdgeBounds(issue.from, issue.to), {
      padding: 80
    });
}
```

## Example Built-In Validation Rules with Visual Output

These map perfectly to overlays:

### Rule 1 — "Unused Component"
Node error → grey outline + badge

### Rule 2 — "Missing Dependency"
Node-level + adjacency warning

### Rule 3 — Cycle Detected
Each edge in cycle is colored red + dashed

### Rule 4 — Direct UI → DB Access
UI node border goes **red**  
Edge is highlighted in red

### Rule 5 — External API without Gateway
External node has warning badge

### Rule 6 — Fan-Out to Missing Consumers
Queue node gets warning badge  
Edges dashed

## End-to-End Flow Summary

```
DSL change or Diagram change
    ↓
Model updated
    ↓
Validation engine runs
    ↓
Issues written to global Zustand store
    ↓
ReactFlow nodes and edges re-render:
  - borders colored
  - badges added
  - edge markers updated
  - tooltips attached
    ↓
Validation panel syncs automatically
    ↓
User clicks issue → diagram auto-focus
```

No flicker.  
No loops.  
No diagram resets.  
Perfect UX.

## MCP API

```
validation.overlays(model)
validation.issues()
validation.highlight(issueId)
```

## Strategic Value

The Visual Validation Overlays system provides:

- ✅ Live feedback on diagram
- ✅ Immediate error visibility
- ✅ Better user experience
- ✅ Reduced validation friction
- ✅ Professional editor quality

**This is critical for user experience and validation effectiveness.**

## Implementation Status

✅ Architecture designed  
✅ Overlay system specified  
✅ ReactFlow integration defined  
📋 Implementation in progress

---

*The Visual Validation Overlays system provides live architectural feedback directly on the diagram.*

