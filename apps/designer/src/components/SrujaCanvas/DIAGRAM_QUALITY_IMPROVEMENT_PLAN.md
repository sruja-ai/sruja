# Diagram Quality Improvement Plan

## Fundamental Analysis & Implementation Strategy

## Executive Summary

This plan addresses fundamental diagram quality issues in both **structure** (node positioning, alignment, spacing) and **label placement** (edge labels, node text layout). The improvements are based on research from LikeC4, Mermaid, PlantUML, and Graphviz best practices, applied to our Go-based DOT generator.

## Current State Analysis

### Strengths

- ✅ Basic rank constraints implemented (`rank=same` for same-kind nodes)
- ✅ Edge weights implemented (weight=10 for labeled, weight=1 for unlabeled)
- ✅ Adaptive spacing based on node count (basic scaling)
- ✅ Cluster support with subgraphs
- ✅ Graphviz spline edge routing available

### Critical Gaps

#### 1. **Node Sizing (CRITICAL - Structure)**

**Problem**: Heuristic-based sizing (`len(title)*8 + 40`) doesn't account for:

- Actual font metrics (Arial 12pt/14pt)
- Multi-line text wrapping
- Technology tags
- Description text
- Padding requirements

**Impact**:

- Long titles overflow or get truncated
- Short titles waste space
- Inconsistent visual hierarchy
- Poor readability

#### 2. **Rank Constraints (HIGH - Structure)**

**Problem**: Current implementation groups by `kind` only, missing:

- View-level specific ranks (L1: persons→systems, L2: containers→datastores)
- Parent-child rank relationships
- Cross-cluster rank alignment
- Explicit rank ordering (min/max/same)

**Impact**:

- Persons and systems may not align properly in L1
- Containers may not be visually grouped with their system
- No clear vertical hierarchy

#### 3. **Label Placement (HIGH - Labels)**

**Problem**: Graphviz edge labels lack constraints:

- No `labeldistance` or `labelangle` hints
- No `headlabel`/`taillabel` positioning
- No `decorate` for label-line association
- Edge labels may overlap nodes or other edges

**Impact**:

- Edge labels hard to read
- Labels may overlap nodes
- No control over label position along edge

#### 4. **Layout Hints (MEDIUM - Structure)**

**Problem**: Missing Graphviz layout hints:

- No `group` attribute for sibling clustering
- No `ordering=out`/`ordering=in` for node sequence
- No `headport`/`tailport` for edge attachment points
- No `constraint=false` for non-layout edges

**Impact**:

- Random node ordering within ranks
- Edges attach at suboptimal points
- Related nodes not visually grouped

#### 5. **Cluster Optimization (MEDIUM - Structure)**

**Problem**: Clusters use fixed margins, missing:

- Dynamic sizing based on contained nodes
- Proper padding calculation
- Cluster label positioning
- Nested cluster handling

**Impact**:

- Clusters too large or too small
- Poor spacing around clusters
- Cluster boundaries may overlap nodes

#### 6. **Edge Routing (MEDIUM - Structure)**

**Problem**: While splines are computed, missing:

- `splines` attribute optimization (ortho vs spline vs polyline)
- `concentrate` for edge bundling
- `overlap` handling
- Port-based routing hints

**Impact**:

- Suboptimal edge paths
- Edge crossings not minimized
- No edge bundling for parallel edges

## FAANG-Level Strategy: How Big Tech Solves This

### Core Philosophy: Constraint-Based Layout with Separation of Concerns

FAANG companies treat diagram layout as a **constraint satisfaction problem**, not a heuristic-based positioning task. Key principles:

#### 1. **Constraint-Based Architecture**

**How FAANG does it**: Define constraints, let the solver optimize.

- **Constraints are declarative**: "Persons must be at rank=min", not "Place persons at y=0"
- **Solver handles optimization**: Graphviz's dot algorithm is a constraint solver
- **Constraints are composable**: Add more constraints = better layout

**Our approach**:

```go
// Define constraints explicitly
type LayoutConstraints struct {
    RankConstraints    []RankConstraint
    SizeConstraints   []SizeConstraint
    PositionHints      []PositionHint
    EdgeConstraints   []EdgeConstraint
}

// Let Graphviz solve
dot := generateDOT(model, constraints)
```

#### 2. **Separation of Concerns: Layout vs Rendering**

**How FAANG does it**: Layout is pure computation, rendering is separate.

- **Layout engine**: Pure function `Layout(model) -> positions`
- **Rendering engine**: Takes positions + styles -> pixels
- **No coupling**: Layout doesn't know about React Flow, React Flow doesn't know about Graphviz internals

**Our current state**: ✅ Already separated (DOT → Graphviz → React Flow)

**Enhancement needed**: Make layout constraints explicit, not implicit in DOT generation.

#### 3. **Iterative Refinement, Not Single-Pass**

**How FAANG does it**: Multiple passes with quality metrics.

- **Pass 1**: Rough layout with basic constraints
- **Pass 2**: Refine based on quality metrics (crossings, overlaps)
- **Pass 3**: Optimize label placement
- **Pass 4**: Final polish

**Our approach**:

```go
// Phase 1: Basic layout
layout1 := runGraphviz(dot, basicConstraints)

// Phase 2: Measure quality
quality := measureLayoutQuality(layout1)
if quality.crossings > threshold {
    // Add more constraints
    refinedConstraints := addAntiCrossingConstraints(constraints)
    layout2 := runGraphviz(dot, refinedConstraints)
}
```

#### 4. **Metrics-Driven Optimization**

**How FAANG does it**: Measure everything, optimize based on data.

- **Quality metrics**: Edge crossings, node overlaps, label overlaps, edge length variance
- **Performance metrics**: Layout time, memory usage
- **User metrics**: Zoom level, pan frequency (indicates confusion)

**Our metrics**:

```go
type LayoutQuality struct {
    EdgeCrossings    int
    NodeOverlaps     int
    LabelOverlaps    int
    AvgEdgeLength    float64
    EdgeLengthVariance float64
    RankAlignment    float64  // 0-1, how well ranks align
    ClusterBalance   float64  // 0-1, how balanced clusters are
}
```

#### 5. **Hierarchical Decomposition**

**How FAANG does it**: Break complex graphs into subgraphs, layout recursively.

- **Top-level**: Layout systems and persons
- **Per-system**: Layout containers within system
- **Per-container**: Layout components within container
- **Compose**: Merge sub-layouts with proper spacing

**Our approach**:

```go
// L1: Layout top-level
l1Layout := layoutLevel1(persons, systems)

// L2: For each system, layout its containers
for system := range systems {
    l2Layout := layoutLevel2(system.containers)
    mergeLayout(l1Layout, l2Layout)
}
```

#### 6. **Caching and Incremental Updates**

**How FAANG does it**: Layout is expensive, cache aggressively.

- **Cache key**: Model hash + view level + focus node
- **Invalidation**: Only re-layout when model changes
- **Incremental**: If one node changes, only re-layout affected subgraph

**Our current state**: ✅ Basic caching exists in React component

**Enhancement needed**: Cache at DOT generation level, not just React Flow level.

#### 7. **User Override with Constraints**

**How FAANG does it**: Allow manual adjustments, but preserve constraints.

- **User drags node**: Update position, but maintain rank constraints
- **User pins node**: Add `pin=true` constraint, re-layout around it
- **User adjusts spacing**: Update `nodesep`/`ranksep`, re-layout

**Future enhancement**: Allow users to pin nodes, then re-layout with pinned constraints.

#### 8. **Proven Algorithms, Not Custom Solutions**

**How FAANG does it**: Use battle-tested libraries.

- **Graphviz**: 30+ years of refinement
- **ELK (Eclipse Layout Kernel)**: Used by VS Code, Eclipse
- **Dagre**: Used by Mermaid, many other tools
- **Don't reinvent**: These libraries have solved edge cases we haven't thought of

**Our approach**: ✅ Using Graphviz (correct choice)

**Enhancement**: Use Graphviz more effectively with proper constraints.

#### 9. **Performance at Scale**

**How FAANG does it**: Handle 1000+ node graphs efficiently.

- **Lazy layout**: Only layout visible nodes
- **Level-of-detail**: Simplify for overview, detail on zoom
- **Web workers**: Layout computation in background thread
- **Progressive rendering**: Show nodes as they're positioned

**Our current state**: ✅ Graphviz WASM runs in background

**Future enhancement**: Implement level-of-detail for very large graphs.

#### 10. **Deterministic Results**

**How FAANG does it**: Same input = same output, always.

- **No randomness**: Use deterministic algorithms
- **Fixed seeds**: If using random, use fixed seed
- **Versioned algorithms**: Algorithm changes = version bump

**Our approach**: ✅ Graphviz dot algorithm is deterministic

**Enhancement**: Ensure all our constraints are deterministic.

### FAANG Implementation Pattern

```go
// 1. Define constraints explicitly
constraints := BuildConstraints(model, viewLevel)

// 2. Generate DOT with all constraints
dot := GenerateDOT(model, constraints)

// 3. Layout with Graphviz (constraint solver)
layout := RunGraphviz(dot)

// 4. Measure quality
quality := MeasureQuality(layout)

// 5. If quality insufficient, refine constraints
if quality.Score < threshold {
    constraints = RefineConstraints(constraints, quality)
    layout = RunGraphviz(GenerateDOT(model, constraints))
}

// 6. Return deterministic result
return layout
```

## Fundamental Principles

### 1. **Text Measurement is Non-Negotiable**

All successful diagram tools (Mermaid, LikeC4, PlantUML) measure actual text before layout. We must:

- Measure text using font metrics (not character count)
- Account for font size, weight, and family
- Handle multi-line text with proper line-height
- Calculate padding based on content

**FAANG approach**: Use actual font rendering engine, cache metrics, measure at layout time.

### 2. **Rank Constraints Define Structure**

Graphviz's `rank` attribute is the primary tool for hierarchical layout:

- `rank=min` for top-level (persons in L1)
- `rank=same` for same-level alignment
- `rank=max` for bottom-level
- Invisible edges with high weight for ordering

**FAANG approach**: Constraints are data structures, not strings. Generate DOT from constraints.

### 3. **Edge Labels Need Explicit Positioning**

Graphviz provides label placement attributes:

- `labeldistance` controls distance from edge
- `labelangle` controls angle
- `headlabel`/`taillabel` for end-specific labels
- `decorate` for visual association

**FAANG approach**: Label placement is a constraint problem. Define label constraints, let solver optimize.

### 4. **Layout Hints Guide Graphviz**

Graphviz makes better decisions with hints:

- `group` keeps related nodes together
- `ordering` controls node sequence
- `headport`/`tailport` control edge attachment
- `constraint=false` for non-structural edges

**FAANG approach**: Hints are weak constraints. Use them to guide, not force.

## FAANG Implementation Pattern

### Constraint-Based DOT Generation

Instead of generating DOT strings directly, we define constraints and generate DOT from constraints:

```go
// pkg/export/dot/constraints.go

type RankConstraint struct {
    Type    string   // "min", "max", "same"
    NodeIDs []string
}

type SizeConstraint struct {
    NodeID string
    MinWidth, MaxWidth, MinHeight, MaxHeight float64
    PreferredWidth, PreferredHeight float64
}

type EdgeConstraint struct {
    From, To string
    Weight   int
    MinLen   int
    Ports    struct {
        Tail string // "n", "s", "e", "w"
        Head string
    }
    Label struct {
        Text      string
        Distance  float64
        Angle     float64
        Position  float64 // 0.0-1.0
    }
}

type LayoutConstraints struct {
    Ranks   []RankConstraint
    Sizes   []SizeConstraint
    Edges   []EdgeConstraint
    Global  GlobalConstraints
}

type GlobalConstraints struct {
    NodeSep    float64
    RankSep    float64
    Splines    string // "spline", "ortho", "polyline"
    RankDir    string // "TB", "LR"
}

// GenerateDOT generates DOT from model + constraints
func GenerateDOT(model *Model, constraints LayoutConstraints) string {
    // Build DOT programmatically from constraints
    // This is more maintainable than string concatenation
}
```

### Quality Metrics System

```go
// pkg/export/dot/quality.go

type LayoutQuality struct {
    EdgeCrossings    int
    NodeOverlaps     int
    LabelOverlaps    int
    AvgEdgeLength    float64
    EdgeLengthVariance float64
    RankAlignment    float64  // 0.0-1.0
    ClusterBalance   float64  // 0.0-1.0
    Score            float64  // Overall 0.0-1.0
}

func MeasureQuality(layout *GraphvizLayout) LayoutQuality {
    // Analyze layout result
    // Count crossings, overlaps, measure alignment
}

func (q LayoutQuality) NeedsRefinement() bool {
    return q.Score < 0.7 || q.EdgeCrossings > 5 || q.NodeOverlaps > 0
}
```

### Iterative Refinement

```go
// pkg/export/dot/layout.go

func LayoutWithRefinement(model *Model, viewLevel int) (*GraphvizLayout, error) {
    // Phase 1: Build initial constraints
    constraints := BuildInitialConstraints(model, viewLevel)

    // Phase 2: Generate DOT and layout
    dot := GenerateDOT(model, constraints)
    layout := RunGraphviz(dot)

    // Phase 3: Measure quality
    quality := MeasureQuality(layout)

    // Phase 4: Refine if needed
    maxIterations := 3
    for i := 0; i < maxIterations && quality.NeedsRefinement(); i++ {
        constraints = RefineConstraints(constraints, quality, layout)
        dot = GenerateDOT(model, constraints)
        layout = RunGraphviz(dot)
        quality = MeasureQuality(layout)
    }

    return layout, nil
}

func RefineConstraints(
    constraints LayoutConstraints,
    quality LayoutQuality,
    layout *GraphvizLayout,
) LayoutConstraints {
    // Add anti-crossing constraints
    if quality.EdgeCrossings > 0 {
        constraints = AddAntiCrossingConstraints(constraints, layout)
    }

    // Improve rank alignment
    if quality.RankAlignment < 0.9 {
        constraints = StrengthenRankConstraints(constraints)
    }

    // Adjust spacing
    if quality.NodeOverlaps > 0 {
        constraints.Global.NodeSep *= 1.2
        constraints.Global.RankSep *= 1.2
    }

    return constraints
}
```

## Implementation Plan

### Phase 1: Foundation - Text Measurement & Node Sizing (CRITICAL)

**Goal**: Replace heuristic sizing with accurate text measurement.

#### 1.1 Create Text Measurement Utility

**File**: `pkg/export/dot/text_measure.go`

```go
package dot

import (
    "image"
    "github.com/golang/freetype/truetype"
    "golang.org/x/image/font"
    "golang.org/x/image/math/fixed"
)

// FontMetrics holds font measurement data
type FontMetrics struct {
    FontSize    float64
    FontWeight  string
    FontFamily  string
    LineHeight  float64
    CharWidth   map[rune]float64 // Cache for common characters
}

// MeasureText measures actual text width using font metrics
func MeasureText(text string, metrics FontMetrics) float64 {
    // Implementation using truetype or similar
    // Returns width in pixels
}

// MeasureMultiline measures multi-line text with wrapping
func MeasureMultiline(text string, maxWidth float64, metrics FontMetrics) (width, height float64, lines []string) {
    // Wraps text and measures each line
    // Returns total width, height, and line array
}
```

**Rationale**: Accurate text measurement is the foundation for proper node sizing. Without this, all other improvements are compromised.

#### 1.2 Update Node Size Calculation

**File**: `pkg/export/dot/extractors.go`

**Changes**:

- Replace `calculateNodeSize()` heuristic with text measurement
- Measure title (bold, 14pt)
- Measure technology tag (normal, 12pt)
- Measure description (wrapped, 12pt)
- Calculate padding (20px horizontal, 15px vertical)
- Set min/max constraints (min: 180x100, max: 500x300)

**Expected Impact**:

- Nodes sized to content
- No text overflow
- Consistent visual hierarchy

### Phase 2: Structure - Enhanced Rank Constraints (HIGH)

**Goal**: Implement view-specific rank constraints for proper hierarchical alignment.

#### 2.1 View-Specific Rank Rules

**File**: `pkg/export/dot/writers.go` - `writeRankConstraints()`

**L1 (Context View)**:

```go
// Persons at top (rank=min)
{ rank=min; "person1"; "person2" }

// Systems below persons (rank=same)
{ rank=same; "system1"; "system2" }

// Invisible edge to enforce ordering
"person1" -> "system1" [style=invis, weight=1000]
```

**L2 (Container View)**:

```go
// Containers in same rank (within their system cluster)
{ rank=same; "system1.container1"; "system1.container2" }

// Datastores below containers
{ rank=same; "system1.db1"; "system1.db2" }

// Queues with datastores or separate rank
{ rank=same; "system1.queue1" }
```

**L3 (Component View)**:

```go
// Components in sequence (ordering=out)
// Use invisible edges to enforce left-to-right order
```

**Rationale**: View-specific ranks ensure proper visual hierarchy matching C4 model expectations.

#### 2.2 Cross-Cluster Rank Alignment

**Enhancement**: Ensure nodes in different clusters but same level align horizontally.

```go
// For L2: All containers across all systems should align
// Use rank=same across cluster boundaries
```

### Phase 3: Labels - Edge Label Placement (HIGH)

**Goal**: Improve edge label positioning and readability.

#### 3.1 Edge Label Attributes

**File**: `pkg/export/dot/writers.go` - `writeEdge()`

**Add**:

```go
// Label distance from edge (in inches)
labeldistance=1.5

// Label angle (0=parallel to edge, 90=perpendicular)
labelangle=0

// Decorate: draw line from label to edge
decorate=true

// Label position (0.0=head, 1.0=tail, 0.5=middle)
labelpos=0.5
```

**Rationale**: Explicit label positioning prevents overlap and improves readability.

#### 3.2 Long Label Handling

**Enhancement**: For long labels, use `headlabel`/`taillabel` or split label.

```go
if len(rel.Label) > 30 {
    // Use headlabel/taillabel or wrap
    // Or use labelangle=90 for perpendicular placement
}
```

### Phase 4: Layout Hints - Grouping & Ordering (MEDIUM)

**Goal**: Add Graphviz layout hints for better node positioning.

#### 4.1 Group Attribute

**File**: `pkg/export/dot/writers.go` - `writeNode()`

**Enhancement**: Use `group` to keep siblings together:

```go
// Siblings in same parent get same group
if elem.ParentID != "" {
    fmt.Fprintf(sb, ",\n%s  group=\"%s\"", indent, escapeID(elem.ParentID))
}
```

**Rationale**: `group` attribute tells Graphviz to keep related nodes close together.

#### 4.2 Ordering Hints

**Enhancement**: Add `ordering=out` for nodes with many outgoing edges.

```go
// For nodes with >3 outgoing edges, prefer ordering=out
if outgoingEdgeCount > 3 {
    attrs = append(attrs, "ordering=out")
}
```

#### 4.3 Port Constraints

**Enhancement**: Use `headport`/`tailport` for better edge attachment.

```go
// Determine optimal ports based on node positions
// Use headport/tailport in edge definition
"node1":e -> "node2":w [label="..."]
```

### Phase 5: Cluster Optimization (MEDIUM)

**Goal**: Improve cluster sizing and spacing.

#### 5.1 Dynamic Cluster Sizing

**File**: `pkg/export/dot/writers.go` - `writeCluster()`

**Enhancement**: Calculate cluster size based on contained nodes:

```go
// Calculate bounding box of all children
minX, minY, maxX, maxY := calculateClusterBounds(children)

// Add padding (40px = ~0.56 inches at 72 DPI)
padding := 0.56
clusterWidth := (maxX - minX) + (padding * 2)
clusterHeight := (maxY - minY) + (padding * 2)

// Use margin based on cluster size
margin := calculateMargin(clusterWidth, clusterHeight)
```

#### 5.2 Cluster Label Positioning

**Enhancement**: Position cluster labels better:

```go
// Use labelloc and labeljust
fmt.Fprintf(sb, "    labelloc=\"t\";  // Top\n")
fmt.Fprintf(sb, "    labeljust=\"l\";  // Left\n")
```

### Phase 6: Edge Routing Optimization (MEDIUM)

**Goal**: Optimize edge paths and reduce crossings.

#### 6.1 Spline Type Selection

**File**: `pkg/export/dot/writers.go` - `writeGraphHeader()`

**Enhancement**: Choose spline type based on diagram density:

```go
// For dense diagrams, use ortho (orthogonal)
// For sparse diagrams, use spline (curved)
if nodeCount > 10 {
    sb.WriteString("    splines=ortho,\n")
} else {
    sb.WriteString("    splines=spline,\n")
}
```

#### 6.2 Edge Concentration

**Enhancement**: Bundle parallel edges:

```go
// For multiple edges between same nodes
if parallelEdgeCount > 2 {
    sb.WriteString("    concentrate=true,\n")
}
```

#### 6.3 Overlap Handling

**Enhancement**: Prevent node overlap:

```go
sb.WriteString("    overlap=false,\n")  // Or overlap=scale
sb.WriteString("    sep=0.2,\n")        // Minimum separation
```

## Implementation Priority (FAANG Approach)

### Week 1: Foundation - Constraint Architecture

**Goal**: Establish constraint-based architecture before adding features.

1. **Architectural Refactoring** ⚠️ CRITICAL
   - Extract constraint building from DOT generation
   - Create `constraints.go` with constraint data structures
   - Refactor `Export()` to use constraints
   - Impact: Enables all future improvements
   - Effort: 2-3 days
   - Risk: Medium (refactoring existing code)

2. **Text Measurement & Node Sizing** ⚠️ CRITICAL
   - Implement font metrics measurement
   - Update size constraints based on actual text
   - Impact: 50-70% improvement in readability
   - Effort: 2-3 days
   - Risk: Medium (requires font library integration)

### Week 2: Quality & Refinement

3. **Quality Metrics System** ⚠️ HIGH
   - Implement `MeasureQuality()` function
   - Count crossings, overlaps, measure alignment
   - Impact: Enables data-driven optimization
   - Effort: 1-2 days
   - Risk: Low

4. **Enhanced Rank Constraints** ⚠️ HIGH
   - View-specific rank rules (L1/L2/L3)
   - Cross-cluster alignment
   - Impact: Proper hierarchical alignment
   - Effort: 1-2 days
   - Risk: Low (using constraint architecture)

5. **Iterative Refinement** ⚠️ MEDIUM
   - Implement `RefineConstraints()` based on quality
   - Add anti-crossing constraints
   - Impact: Automatic layout improvement
   - Effort: 2 days
   - Risk: Medium (complex logic)

### Short-term (Week 2)

3. **Phase 3**: Edge Label Placement ⚠️ HIGH
   - Impact: Better label readability
   - Effort: 1 day
   - Risk: Low

4. **Phase 4**: Layout Hints ⚠️ MEDIUM
   - Impact: Better node positioning
   - Effort: 2 days
   - Risk: Low

### Medium-term (Week 3-4)

5. **Phase 5**: Cluster Optimization ⚠️ MEDIUM
   - Impact: Better cluster appearance
   - Effort: 2 days
   - Risk: Medium

6. **Phase 6**: Edge Routing ⚠️ MEDIUM
   - Impact: Cleaner edge paths
   - Effort: 1-2 days
   - Risk: Low

## Architectural Refactoring: Constraint-Based Architecture

### Current Architecture (String-Based)

```
Model → String concatenation → DOT string → Graphviz → Layout
```

**Problems**:

- Constraints are implicit in string generation
- Hard to test constraints independently
- Hard to refine constraints based on quality
- No way to measure constraint effectiveness

### Target Architecture (Constraint-Based)

```
Model → Constraints (data) → DOT generator → DOT string → Graphviz → Layout → Quality metrics → Refine constraints
```

**Benefits**:

- Constraints are explicit and testable
- Can refine constraints based on quality metrics
- Can cache constraints separately from DOT
- Can version constraints independently

### Migration Path

#### Step 1: Extract Constraint Building (Week 1)

**File**: `pkg/export/dot/constraints.go` (new)

```go
// BuildConstraints extracts constraints from model
func BuildConstraints(elements []*Element, relations []*Relation, viewLevel int) LayoutConstraints {
    constraints := LayoutConstraints{
        Global: GlobalConstraints{
            NodeSep: 120,
            RankSep: 130,
            Splines: "spline",
            RankDir: "TB",
        },
    }

    // Build rank constraints
    constraints.Ranks = buildRankConstraints(elements, viewLevel)

    // Build size constraints
    constraints.Sizes = buildSizeConstraints(elements)

    // Build edge constraints
    constraints.Edges = buildEdgeConstraints(relations)

    return constraints
}
```

#### Step 2: Refactor DOT Generation (Week 1)

**File**: `pkg/export/dot/dot.go`

```go
// Export now uses constraints
func (e *Exporter) Export(prog *language.Program) *ExportResult {
    // ... extract elements and relations ...

    // Build constraints
    constraints := BuildConstraints(elements, relations, e.Config.ViewLevel)

    // Generate DOT from constraints
    dot := GenerateDOTFromConstraints(elements, relations, constraints)

    return &ExportResult{
        DOT: dot,
        Elements: elements,
        Relations: relations,
        Constraints: constraints, // Expose for testing
    }
}
```

#### Step 3: Add Quality Metrics (Week 2)

**File**: `pkg/export/dot/quality.go` (new)

```go
// MeasureQuality analyzes Graphviz layout result
func MeasureQuality(layout *GraphvizLayout) LayoutQuality {
    // Count edge crossings
    crossings := countEdgeCrossings(layout)

    // Check node overlaps
    overlaps := countNodeOverlaps(layout)

    // Measure rank alignment
    alignment := measureRankAlignment(layout)

    // Calculate overall score
    score := calculateScore(crossings, overlaps, alignment)

    return LayoutQuality{
        EdgeCrossings: crossings,
        NodeOverlaps: overlaps,
        RankAlignment: alignment,
        Score: score,
    }
}
```

#### Step 4: Implement Iterative Refinement (Week 2)

**File**: `pkg/export/dot/refinement.go` (new)

```go
// RefineConstraints improves constraints based on quality
func RefineConstraints(
    original LayoutConstraints,
    quality LayoutQuality,
) LayoutConstraints {
    refined := original // Copy

    // Add anti-crossing constraints
    if quality.EdgeCrossings > 0 {
        refined = addAntiCrossingHints(refined)
    }

    // Strengthen rank constraints
    if quality.RankAlignment < 0.9 {
        refined = strengthenRanks(refined)
    }

    // Adjust spacing
    if quality.NodeOverlaps > 0 {
        refined.Global.NodeSep *= 1.2
        refined.Global.RankSep *= 1.2
    }

    return refined
}
```

### Benefits of This Architecture

1. **Testability**: Can test constraint building independently
2. **Debuggability**: Can inspect constraints before DOT generation
3. **Optimization**: Can refine constraints based on quality metrics
4. **Extensibility**: Easy to add new constraint types
5. **Caching**: Can cache constraints separately from DOT

## Technical Considerations

### Font Measurement Options

**Option 1: Go Font Libraries**

- `golang.org/x/image/font` + `github.com/golang/freetype/truetype`
- Pros: Native Go, good performance
- Cons: Requires font file management

**Option 2: WASM Text Measurement**

- Measure in browser, pass to Go
- Pros: Uses browser font rendering
- Cons: Requires WASM round-trip

**Option 3: Font Metrics Table**

- Pre-calculated metrics for Arial
- Pros: Simple, fast
- Cons: Less accurate, font-specific

**Recommendation**: Start with Option 3 (metrics table) for MVP, migrate to Option 1 for accuracy.

### Testing Strategy

1. **Unit Tests**: Text measurement functions
2. **Integration Tests**: DOT output validation
3. **Visual Tests**: Compare before/after diagrams
4. **Regression Tests**: Ensure existing diagrams still work

### Backward Compatibility

- All changes should be additive (new attributes)
- Default behavior should remain same if config not set
- Use feature flags for new behaviors

## Success Metrics

### Quantitative

- Node size accuracy: ±5% of actual text size
- Rank alignment: 100% of same-level nodes aligned
- Label overlap: <5% of labels overlapping nodes
- Edge crossings: 20% reduction

### Qualitative

- Visual hierarchy clearly visible
- Labels readable without zoom
- Professional appearance matching LikeC4 quality
- Consistent spacing and alignment

## Risk Mitigation

### High Risk: Font Measurement

- **Mitigation**: Start with metrics table, validate with sample diagrams
- **Fallback**: Keep heuristic as fallback if measurement fails

### Medium Risk: Rank Constraints Complexity

- **Mitigation**: Implement incrementally, test each view level
- **Fallback**: Keep existing rank constraints if new ones fail

### Low Risk: Label Placement

- **Mitigation**: Graphviz handles most cases, we're just adding hints
- **Fallback**: Default Graphviz placement is acceptable

## Next Steps

1. **Review & Approve Plan**: Get stakeholder approval
2. **Set Up Font Measurement**: Choose approach, implement utility
3. **Implement Phase 1**: Text measurement + node sizing
4. **Test & Validate**: Visual comparison with existing diagrams
5. **Iterate**: Refine based on feedback

## Summary: FAANG-Level Approach

### Key Insight

FAANG companies don't solve layout problems with heuristics—they solve them with **constraint-based systems** that:

1. Define constraints explicitly (data structures, not strings)
2. Let proven solvers optimize (Graphviz dot algorithm)
3. Measure quality and refine iteratively
4. Separate concerns (layout vs rendering)

### Our Transformation

**Before (Current)**:

```
Model → String concatenation → DOT → Graphviz → Layout
```

- Constraints implicit in string generation
- No quality measurement
- No refinement capability
- Hard to test and debug

**After (FAANG Approach)**:

```
Model → Constraints (data) → DOT Generator → DOT → Graphviz → Layout → Quality Metrics → Refine Constraints
```

- Constraints explicit and testable
- Quality metrics drive optimization
- Iterative refinement improves results
- Easy to extend and maintain

### Implementation Strategy

1. **Week 1**: Build constraint architecture foundation
   - Extract constraints from string generation
   - Implement text measurement
   - Establish testing framework

2. **Week 2**: Add quality and refinement
   - Implement quality metrics
   - Add iterative refinement
   - Enhance rank constraints

3. **Week 3-4**: Polish and optimize
   - Edge label placement
   - Layout hints
   - Cluster optimization

### Expected Outcomes

- **50-70% improvement** in diagram readability (Phase 1)
- **Professional quality** matching LikeC4 (Phase 2)
- **Automatic optimization** through iterative refinement (Phase 3)
- **Maintainable codebase** with constraint-based architecture

### Success Criteria

✅ Constraints are explicit data structures  
✅ Quality metrics are measurable  
✅ Layout improves through refinement  
✅ Code is testable and maintainable  
✅ Diagrams match professional tools in quality

## References

- [Graphviz Attributes](https://graphviz.org/docs/attrs/)
- [LikeC4 Layout Strategy](https://github.com/likec4/likec4)
- [Mermaid Text Measurement](https://github.com/mermaid-js/mermaid)
- [ELK (Eclipse Layout Kernel)](https://www.eclipse.org/elk/)
- [Dagre Layout Algorithm](https://github.com/dagrejs/dagre)
- LAYOUT_GAPS_ANALYSIS.md (this directory)
