# Diagram Layout Generation - Gap Analysis

## Research: How LikeC4 and Similar Tools Create Better Diagrams

### LikeC4's Approach

LikeC4 is a toolchain for architecture-as-code that generates well-structured C4 diagrams. Key techniques:

#### 1. **Architecture-as-Code DSL**

- Single source of truth for all diagrams
- Model-driven approach ensures consistency
- Live preview with instant feedback
- Version control integration (diagrams stored in Git)

#### 2. **Multiple Layout Algorithms**

- Supports multiple layout engines (Graphviz, Mermaid, D2)
- Allows users to choose the best layout for their use case
- Can export to different formats (PNG, SVG, DOT, Mermaid)

#### 3. **Hierarchical Structure**

- Uses nested models (System → Container → Component)
- Maintains parent-child relationships explicitly
- Clusters are properly sized based on content

#### 4. **Consistent Styling**

- Uniform styles across all diagram levels
- Theme-aware (light/dark mode)
- Customizable but with sensible defaults

#### 5. **View Definitions**

- Allows defining specific views (System Context, Container, Component)
- Can filter and focus on specific parts of the model
- Supports custom views with specific rules

### Graphviz Best Practices (Used by LikeC4, PlantUML)

#### 1. **Rank Constraints**

```dot
// LikeC4 uses rank=same for nodes at the same level
{ rank=same; "Person1"; "Person2"; "Person3" }
{ rank=same; "System1"; "System2"; "System3" }
```

#### 2. **Node Grouping**

```dot
// Group related nodes together
subgraph cluster_system1 {
  label="System 1";
  "Container1";
  "Container2";
}
```

#### 3. **Edge Weights**

```dot
// Important relationships get higher weight
"System1" -> "System2" [weight=10];
"System1" -> "System3" [weight=1];
```

#### 4. **Direction Hints**

```dot
// Use rankdir based on diagram type
rankdir=TB;  // Top to bottom (default for C4)
rankdir=LR;  // Left to right (for wide diagrams)
```

#### 5. **Node Ordering**

```dot
// Control node order within ranks
ordering=out;  // Order by outgoing edges
ordering=in;   // Order by incoming edges
```

#### 6. **Port Constraints**

```dot
// Control edge attachment points
"Node1":e -> "Node2":w;  // East to West
"Node1":s -> "Node2":n;  // South to North
```

### Mermaid's Approach

#### 1. **Automatic Layout**

- Uses Dagre layout algorithm (hierarchical)
- Automatically handles node positioning
- Optimizes for readability

#### 2. **Text Measurement**

- Measures text to calculate node sizes
- Supports multi-line text with proper wrapping
- Accounts for font size and weight

#### 3. **Edge Routing**

- Uses orthogonal routing by default
- Minimizes edge crossings
- Supports curved edges for better readability

#### 4. **Subgraph Handling**

- Properly sizes subgraphs based on content
- Maintains hierarchy visually
- Handles nested structures well

### PlantUML's Approach

#### 1. **Layout Algorithms**

- Uses Graphviz for layout
- Supports multiple algorithms (dot, neato, fdp, sfdp, twopi, circo)
- Allows users to choose algorithm per diagram

#### 2. **Skin Parameters**

- Extensive customization options
- Consistent styling across elements
- Theme support

#### 3. **Layout Hints**

- Uses `left to right direction` for horizontal layouts
- Supports `top to bottom direction` for vertical
- Can specify node positions explicitly

### Structurizr's Approach

#### 1. **View-Specific Layouts**

- Different layouts for different view types
- System Context: Focus on external actors
- Container: Focus on internal structure
- Component: Detailed component view

#### 2. **Automatic Positioning**

- Uses force-directed layout algorithms
- Minimizes edge crossings
- Optimizes for visual clarity

#### 3. **Relationship Routing**

- Smart edge routing to avoid overlaps
- Uses curved edges for better readability
- Groups related relationships

### Key Learnings

#### 1. **Text Measurement is Critical**

- All successful tools measure text before layout
- Node sizes should adapt to content
- Multi-line text requires proper line-height calculation

#### 2. **Rank Constraints are Essential**

- Same-level nodes should use `rank=same`
- Prevents random vertical positioning
- Ensures consistent alignment

#### 3. **Edge Weights Matter**

- Important relationships should have higher weight
- Helps Graphviz prioritize layout decisions
- Reduces edge crossings

#### 4. **Adaptive Spacing**

- Spacing should scale with diagram size
- Too tight = overlapping
- Too loose = wasted space

#### 5. **Multiple Layout Attempts**

- Some tools try multiple algorithms
- Pick the best result
- Fallback to simpler layouts if needed

#### 6. **View-Specific Rules**

- Different views need different layouts
- System Context: Persons at top, Systems below
- Container: Containers in logical groups
- Component: Components in sequence

#### 7. **Cluster Optimization**

- Clusters should size based on content
- Proper margins prevent overlap
- Nested clusters need special handling

#### 8. **Edge Routing Optimization**

- Use Graphviz edge points when available
- Or ensure React Flow routing matches expectations
- Minimize crossings and overlaps

## Current Pipeline

1. **Projection** (`projectionEngine.ts`) → Creates C4Nodes and C4Edges
2. **DOT Generation** (`dotGenerator.ts`) → Converts to Graphviz DOT format
3. **Layout** (`layoutEngine.ts`) → Runs Graphviz WASM to compute positions
4. **React Flow Mapping** (`SrujaCanvas.tsx`) → Maps to React Flow nodes/edges

## Identified Gaps

### 1. **Fixed Node Sizes** ⚠️ CRITICAL

**Problem**: All nodes have fixed dimensions regardless of content:

- L1: 250x150 (all persons/systems)
- L2: 200x120 (all containers)
- L3: 180x100 (all components)

**Impact**:

- Long titles get truncated or overflow
- Short titles waste space
- Inconsistent visual hierarchy
- Poor readability

**Solution**:

- Measure text content and calculate dynamic sizes
- Use minimum/maximum constraints
- Consider multi-line text with proper line-height
- Account for technology tags and descriptions

### 2. **No Rank Constraints** ⚠️ HIGH

**Problem**: Graphviz doesn't know which nodes should be on the same level.

**Impact**:

- Persons and Systems may not be on the same rank in L1
- Containers may not align properly in L2
- Components may not align in L3
- Inconsistent vertical alignment

**Solution**:

- Add `rank=same` constraints for nodes of the same type/level
- Use `rankdir` more intelligently based on diagram structure
- Consider using `rank` attribute for explicit level control

### 3. **No Edge Weights** ⚠️ MEDIUM

**Problem**: All edges are treated equally in layout.

**Impact**:

- Important relationships don't get priority
- Layout doesn't optimize for critical paths
- No distinction between primary and secondary relationships

**Solution**:

- Assign weights based on relationship importance
- Use `weight` attribute in DOT for edge prioritization
- Consider relationship direction and frequency

### 4. **Fixed Spacing** ⚠️ MEDIUM

**Problem**: `nodesep=110px` and `ranksep=120px` are fixed.

**Impact**:

- Too tight for large diagrams
- Too loose for small diagrams
- Doesn't adapt to content density
- May cause overlapping in dense areas

**Solution**:

- Calculate spacing based on node count and diagram size
- Use adaptive spacing: `nodesep = base + (nodeCount * factor)`
- Consider minimum/maximum spacing constraints
- Scale with zoom level or viewport size

### 5. **No Layout Hints** ⚠️ HIGH

**Problem**: Missing constraints for better hierarchical structure.

**Impact**:

- No control over node ordering
- No hints for preferred positions
- No grouping of related nodes
- Random node placement

**Solution**:

- Add `constraint=false` for edges that shouldn't affect layout
- Use `group` attribute to keep related nodes together
- Add `ordering` hints for node sequence
- Consider `headport` and `tailport` for edge attachment points

### 6. **Edge Routing Not Used** ⚠️ LOW

**Problem**: Graphviz edge points are computed but not used.

**Impact**:

- React Flow routes edges independently
- May not match Graphviz's optimized routing
- Potential edge overlaps
- Less optimal edge paths

**Solution**:

- Parse Graphviz edge spline points
- Use custom edge component with Graphviz paths
- Or ensure React Flow edge types match Graphviz expectations

### 7. **No Cluster Optimization** ⚠️ MEDIUM

**Problem**: Clusters (subgraphs) may not be optimally sized or positioned.

**Impact**:

- Clusters may be too large or too small
- Poor spacing around clusters
- Cluster boundaries may overlap nodes
- Inconsistent cluster margins

**Solution**:

- Calculate cluster size based on contained nodes
- Add proper padding/margins
- Use `clusterrank` attribute
- Consider `newrank=true` for better cluster handling

### 8. **No Feedback Loop** ⚠️ LOW

**Problem**: Can't detect and fix layout quality issues.

**Impact**:

- No detection of overlapping nodes
- No detection of edge crossings
- No metrics for layout quality
- Can't automatically improve bad layouts

**Solution**:

- Add layout quality metrics (crossings, overlaps, edge lengths)
- Implement iterative improvement
- Add layout validation
- Provide user feedback on layout quality

### 9. **Missing Node Constraints** ⚠️ MEDIUM

**Problem**: No constraints to prevent nodes from being too close or too far.

**Impact**:

- Nodes may overlap
- Nodes may be unnecessarily far apart
- No minimum distance enforcement
- No maximum distance constraints

**Solution**:

- Add `minlen` for edges
- Use `nodesep` and `ranksep` more effectively
- Consider `overlap` attribute
- Add custom constraints

### 10. **No Direction Hints** ⚠️ MEDIUM

**Problem**: Layout doesn't consider preferred flow direction.

**Impact**:

- May not follow natural reading flow
- Persons may not be at top
- Systems may not be in logical order
- No consideration for data flow direction

**Solution**:

- Use `rankdir` based on diagram type
- Add explicit rank ordering
- Consider `ordering=out` or `ordering=in`
- Use `rank` attribute for explicit levels

## Recommendations Based on Research

### Immediate (High Impact, Low Effort) - Inspired by LikeC4/Graphviz

1. **Rank Constraints** - Add `rank=same` for same-level nodes (LikeC4 pattern)
   - L1: All persons on same rank, all systems on same rank
   - L2: All containers on same rank
   - L3: All components on same rank

2. **Edge Weights** - Prioritize important relationships (Graphviz best practice)
   - Primary relationships: weight=10
   - Secondary relationships: weight=1
   - Helps Graphviz make better layout decisions

3. **Adaptive Spacing** - Calculate spacing based on node count (Mermaid pattern)
   - Scale nodesep and ranksep with diagram density
   - Prevent overlapping in dense diagrams
   - Avoid excessive spacing in sparse diagrams

### Short-term (High Impact, Medium Effort) - Inspired by Mermaid/PlantUML

4. **Dynamic Node Sizing** - Measure text and calculate sizes (Mermaid approach)
   - Measure actual text width/height
   - Account for multi-line text
   - Set min/max constraints
   - This is critical for readability

5. **Layout Hints** - Add grouping and ordering constraints (Graphviz advanced)
   - Use `group` attribute for related nodes
   - Use `ordering=out` or `ordering=in` for node sequence
   - Add `constraint=false` for edges that shouldn't affect layout

6. **Cluster Optimization** - Better cluster sizing (LikeC4 pattern)
   - Calculate cluster size based on contained nodes
   - Add proper padding/margins
   - Use `newrank=true` for better cluster handling

### Medium-term (High Impact, Medium-High Effort) - Inspired by Structurizr

7. **View-Specific Layout Rules** - Different layouts per view type
   - System Context: Persons at top, Systems below
   - Container: Logical grouping of containers
   - Component: Sequential component layout

8. **Edge Routing** - Use Graphviz edge points (LikeC4 approach)
   - Parse Graphviz spline points
   - Use custom edge component with Graphviz paths
   - Or ensure React Flow routing matches Graphviz expectations

### Long-term (Medium Impact, High Effort) - Research-based

9. **Multiple Layout Attempts** - Try different algorithms (PlantUML pattern)
   - Try dot, neato, fdp algorithms
   - Pick the best result based on quality metrics
   - Fallback to simpler layouts if needed

10. **Feedback Loop** - Add layout quality metrics (Research-based)
    - Detect overlapping nodes
    - Count edge crossings
    - Measure edge lengths
    - Automatically improve bad layouts

11. **Advanced Constraints** - Custom layout rules
    - User-defined constraints
    - Preferred positions
    - Forbidden positions
    - Custom grouping rules

## Decision Record (2025-12-23)

**Decision**: Stick with Graphviz (WASM).
**Rationale**:

1. The backend (`pkg/export/dot`) already implements sophisticated layout logic (text measurement, constraints) that matches or exceeds base D2 capabilities for C4 constraints.
2. The "gaps" identified above were largely due to a stale WASM build rather than engine limitations.
3. Switching to ELK is a high-cost rewrite with minimal immediate ROI for an "adoption" tool.

**Status**: Resolved by rebuilding WASM with latest Go backend logic.
