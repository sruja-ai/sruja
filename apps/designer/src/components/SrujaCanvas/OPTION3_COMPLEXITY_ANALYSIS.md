# Option 3 Complexity Analysis: Passing Graphviz JSON to Go

## Current Architecture

```
TypeScript (Frontend)          Go WASM (Backend)
─────────────────────          ─────────────────
SrujaCanvas.tsx
  ↓
convertDslToDot() ──────────→ sruja_dsl_to_dot()
  (DSL string)                  (WASM function)
                                ↓
                                DOT Generator
                                ↓
                                DOT String ────→ TypeScript
                                                  ↓
                                              Graphviz WASM
                                                  ↓
                                              JSON Output
                                                  ↓
                                              React Flow
```

## Option 3: Full Refinement Loop

```
TypeScript (Frontend)          Go WASM (Backend)
─────────────────────          ─────────────────
1. convertDslToDot() ───────→ sruja_dsl_to_dot()
   (DSL string)                 ↓
                                DOT Generator
                                ↓
                                DOT String ────→ TypeScript
                                                  ↓
                                              Graphviz WASM
                                                  ↓
                                              JSON Output
                                                  ↓
2. refineLayout() ──────────→ sruja_refine_layout()
   (Graphviz JSON)              ↓
                                Parse JSON
                                ↓
                                Measure Quality
                                ↓
                                Refine Constraints
                                ↓
                                Generate New DOT ─→ TypeScript
                                                      ↓
                                                  Graphviz WASM
                                                      ↓
                                                  JSON Output
                                                      ↓
                                                  React Flow
```

## Complexity Breakdown

### 1. **WASM Function Addition** ⚠️ LOW-MEDIUM

**What's needed:**

- Add new WASM export: `sruja_refine_layout(graphvizJson, dsl, viewLevel, focusNodeId)`
- Similar pattern to existing `sruja_dsl_to_dot`

**Complexity:**

- **Low**: Follow existing pattern (1-2 hours)
- **Files to modify:**
  - `cmd/wasm/main.go` - Add new function export
  - `packages/shared/src/web/wasmTypes.ts` - Add TypeScript type
  - `packages/shared/src/web/wasmAdapter.ts` - Add adapter function

**Example:**

```go
// cmd/wasm/main.go
func refineLayout(this js.Value, args []js.Value) interface{} {
    graphvizJson := args[0].String()
    dsl := args[1].String()
    viewLevel := args[2].Int()
    focusNodeId := ""
    if len(args) > 3 {
        focusNodeId = args[3].String()
    }

    // Parse Graphviz JSON, measure quality, refine, return new DOT
    // ...
}
```

### 2. **Graphviz JSON Parsing in Go** ⚠️ MEDIUM

**What's needed:**

- Parse Graphviz JSON structure in Go
- Extract node positions, dimensions, edge splines
- Build internal representation

**Complexity:**

- **Medium**: JSON parsing is straightforward, but Graphviz JSON structure is complex (4-6 hours)
- **Files to create:**
  - `pkg/export/dot/graphviz_json.go` - JSON parsing
  - Types for Graphviz JSON structure

**Graphviz JSON Structure:**

```json
{
  "bb": "0,0,1000,800",
  "objects": [
    {
      "_gvid": 0,
      "name": "system1",
      "pos": "100,200",
      "width": "2.77",
      "height": "1.66"
    }
  ],
  "edges": [
    {
      "_gvid": 1,
      "tail": 0,
      "head": 1,
      "_draw_": [
        {
          "op": "b",
          "points": [
            [100, 200],
            [150, 250],
            [200, 300],
            [250, 350]
          ]
        }
      ]
    }
  ]
}
```

**Go Types Needed:**

```go
type GraphvizJSON struct {
    BB      string          `json:"bb"`
    Objects []GraphvizNode  `json:"objects"`
    Edges   []GraphvizEdge  `json:"edges"`
}

type GraphvizNode struct {
    GVID   int     `json:"_gvid"`
    Name   string  `json:"name"`
    Pos    string  `json:"pos"`    // "x,y"
    Width  string  `json:"width"` // inches
    Height string  `json:"height"` // inches
}

type GraphvizEdge struct {
    GVID  int              `json:"_gvid"`
    Tail  int              `json:"tail"`
    Head  int              `json:"head"`
    Draw  []GraphvizDrawOp `json:"_draw_"`
}
```

### 3. **Quality Metrics Implementation** ⚠️ MEDIUM-HIGH

**What's needed:**

- Implement actual quality metric algorithms (not estimates)
- Edge crossing detection
- Node overlap detection
- Rank alignment measurement
- Edge length variance calculation

**Complexity:**

- **Medium-High**: Requires geometric algorithms (8-12 hours)
- **Files to create/modify:**
  - `pkg/export/dot/quality.go` - Replace estimates with real calculations
  - `pkg/export/dot/geometry.go` - Geometric algorithms

**Algorithms needed:**

1. **Edge Crossing Detection**: Line segment intersection (O(n²) for n edges)
2. **Node Overlap Detection**: Rectangle intersection (O(n²) for n nodes)
3. **Rank Alignment**: Measure Y-coordinate variance within ranks
4. **Edge Length Variance**: Calculate edge lengths, compute variance

**Example:**

```go
func countEdgeCrossings(edges []GraphvizEdge, nodes map[int]GraphvizNode) int {
    crossings := 0
    for i := 0; i < len(edges); i++ {
        for j := i + 1; j < len(edges); j++ {
            if edgesIntersect(edges[i], edges[j], nodes) {
                crossings++
            }
        }
    }
    return crossings
}
```

### 4. **Iterative Refinement Loop** ⚠️ MEDIUM

**What's needed:**

- Refine constraints based on quality metrics
- Generate new DOT
- Return to TypeScript for re-layout
- Handle iteration limit/timeout

**Complexity:**

- **Medium**: Logic exists, needs integration (4-6 hours)
- **Files to modify:**
  - `pkg/export/dot/refinement.go` - Already exists, needs real metrics
  - `cmd/wasm/main.go` - WASM function implementation

**Challenge:**

- Need to pass DSL back to Go (or cache it)
- Multiple round-trips: TypeScript → Go → TypeScript → Graphviz → TypeScript

### 5. **TypeScript Integration** ⚠️ LOW-MEDIUM

**What's needed:**

- Add `refineLayout()` function in TypeScript
- Call WASM function with Graphviz JSON
- Handle iteration loop in TypeScript
- Update React Flow with refined layout

**Complexity:**

- **Low-Medium**: Straightforward but needs careful state management (3-4 hours)
- **Files to modify:**
  - `apps/designer/src/components/SrujaCanvas/SrujaCanvas.tsx`
  - `packages/shared/src/web/wasmAdapter.ts`

**Example:**

```typescript
async function refineLayout(
  dsl: string,
  graphvizJson: any,
  viewLevel: number,
  focusNodeId?: string
): Promise<DotResult> {
  const api = await getWasmApi();
  const result = api.sruja_refine_layout(
    JSON.stringify(graphvizJson),
    dsl,
    viewLevel,
    focusNodeId || ""
  );
  return JSON.parse(result.data);
}
```

## Total Complexity Estimate

| Component              | Complexity      | Time Estimate   | Risk       |
| ---------------------- | --------------- | --------------- | ---------- |
| WASM Function Addition | Low-Medium      | 1-2 hours       | Low        |
| Graphviz JSON Parsing  | Medium          | 4-6 hours       | Medium     |
| Quality Metrics (Real) | Medium-High     | 8-12 hours      | Medium     |
| Refinement Loop        | Medium          | 4-6 hours       | Low        |
| TypeScript Integration | Low-Medium      | 3-4 hours       | Low        |
| **TOTAL**              | **Medium-High** | **20-30 hours** | **Medium** |

## Challenges & Risks

### 1. **Performance** ⚠️ MEDIUM RISK

- Multiple round-trips: TypeScript → Go → TypeScript → Graphviz → TypeScript
- Each iteration adds latency
- Graphviz layout is expensive (50-200ms per layout)
- Quality metrics (edge crossing detection) is O(n²)

**Mitigation:**

- Limit iterations (max 2-3)
- Cache DSL in Go
- Use web workers for quality metrics
- Early exit if quality is good enough

### 2. **Graphviz JSON Structure** ⚠️ LOW RISK

- Graphviz JSON format is stable but complex
- Need to handle edge cases (clusters, subgraphs)
- Coordinate system conversion (Graphviz uses bottom-left origin)

**Mitigation:**

- Use existing TypeScript parser as reference
- Add comprehensive tests
- Handle coordinate conversion carefully

### 3. **State Management** ⚠️ LOW-MEDIUM RISK

- Need to track iteration count
- Handle user interactions during refinement
- Cancel refinement if user changes view

**Mitigation:**

- Use React state/refs
- Add cancellation tokens
- Debounce refinement triggers

### 4. **WASM Size** ⚠️ LOW RISK

- Adding JSON parsing and quality metrics increases WASM size
- Current WASM: ~5-10MB
- Estimated increase: +500KB-1MB

**Mitigation:**

- Use efficient JSON parsing (encoding/json is fine)
- Optimize algorithms
- Consider lazy loading quality metrics

## Benefits vs Complexity

### Benefits ✅

- **Accurate quality metrics** - Real measurements, not estimates
- **True iterative refinement** - Data-driven optimization
- **FAANG-level quality** - Professional-grade layout
- **Consistent with architecture** - All logic in Go

### Complexity ⚠️

- **20-30 hours** of development
- **Medium risk** - Several moving parts
- **Performance overhead** - Multiple round-trips
- **Maintenance burden** - More code to maintain

## Recommendation

**Option 3 is viable but has significant complexity.**

**Consider if:**

- Layout quality is critical
- You have 20-30 hours available
- Performance is acceptable (2-3 iterations = 200-600ms overhead)

**Alternative: Hybrid Approach**

- Keep quality metrics in TypeScript (Option 2)
- Use Graphviz JSON directly (no parsing needed)
- Simpler, faster, still accurate
- **Complexity: 8-12 hours** instead of 20-30

## Implementation Phases

If proceeding with Option 3:

**Phase 1: Foundation (8 hours)**

1. Add WASM function stub
2. Implement Graphviz JSON parsing
3. Basic quality metrics (overlaps, alignment)

**Phase 2: Quality Metrics (8 hours)**

1. Edge crossing detection
2. Rank alignment measurement
3. Edge length variance

**Phase 3: Refinement (6 hours)**

1. Constraint refinement logic
2. Iteration loop
3. TypeScript integration

**Phase 4: Polish (4 hours)**

1. Performance optimization
2. Error handling
3. Testing

**Total: ~26 hours**
