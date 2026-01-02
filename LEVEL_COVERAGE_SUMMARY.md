# Level Coverage Summary

## ✅ All Levels (L1, L2, L3) Now Covered

### Universal Improvements (Apply to All Levels)

1. **Dynamic Spacing Scaling**
   - Based on node count (applies to all levels)
   - Logarithmic scaling: `1.0 + 0.25 * (nodeCount / 8.0)`
   - Caps at 2.2x for very large diagrams

2. **Very Complex Diagrams (20+ nodes)**
   - Extra 30% horizontal spacing (NodeSep)
   - Extra 35% vertical spacing (RankSep)
   - Applies to all levels

3. **Edge Routing Improvements**
   - Increased `minlen` for complex diagrams (all levels)
   - Better spline selection (all levels)
   - Enhanced edge separation (all levels)

4. **Label Positioning**
   - Adaptive label distance (2.0-2.5 inches)
   - Better edge separation for many relations
   - Applies to all levels

5. **Cluster Margins & Compound Nodes**
   - Extra margins for clusters with 5+ children
   - Increased padding for complex compound nodes
   - Applies to all levels with hierarchies

---

## Level-Specific Improvements

### L1 (Context View) - Persons & Systems

**Specific Improvements:**

- ✅ Base scaling: 15% horizontal, 20% vertical (L1NodeSepScale, L1RankSepScale)
- ✅ Complex L1 (8+ nodes): Additional 20% horizontal, 25% vertical
- ✅ Rank constraints: Persons at top rank
- ✅ Handles different node sizes (person vs system)

**Why Needed:**

- Different node sizes cause overlaps
- Person-system relationships need clear separation
- Example: "User overlaps WebApp" issue

**Code Location:**

```go
if viewLevel == 1 || viewLevel == 0 {
    constraints.Global.NodeSep *= L1NodeSepScale // 1.15
    constraints.Global.RankSep *= L1RankSepScale // 1.20
    if len(elements) >= 8 {
        constraints.Global.NodeSep *= 1.20 // Additional 20%
        constraints.Global.RankSep *= 1.25 // Additional 25%
    }
}
```

---

### L2 (Container View) - Containers within Systems

**Specific Improvements:**

- ✅ Complex L2 (8+ containers): 15% horizontal, 20% vertical
- ✅ Very complex L2 (15+ containers): Additional 10% horizontal, 15% vertical
- ✅ Rank constraints: Datastores/queues at bottom
- ✅ Handles dense container layouts

**Why Needed:**

- Many containers in complex systems
- Need spacing for container hierarchies
- Example: SaaS platform with many containers

**Code Location:**

```go
if viewLevel == 2 {
    if len(elements) >= 8 {
        constraints.Global.NodeSep *= 1.15 // 15% boost
        constraints.Global.RankSep *= 1.20 // 20% vertical
    }
    if len(elements) >= 15 {
        constraints.Global.NodeSep *= 1.10 // Additional 10%
        constraints.Global.RankSep *= 1.15 // Additional 15%
    }
}
```

---

### L3 (Component View) - Components within Containers

**Specific Improvements:**

- ✅ Dense L3 (6+ components): 15% horizontal, 18% vertical
- ✅ Complex L3 (12+ components): Additional 10% horizontal, 12% vertical
- ✅ Rank constraints: Components aligned
- ✅ Handles very dense component layouts

**Why Needed:**

- Components are typically smaller and more numerous
- Dense layouts need careful spacing
- Example: API container with many components

**Code Location:**

```go
if viewLevel == 3 {
    if len(elements) >= 6 {
        constraints.Global.NodeSep *= 1.15 // 15% boost
        constraints.Global.RankSep *= 1.18 // 18% vertical
    }
    if len(elements) >= 12 {
        constraints.Global.NodeSep *= 1.10 // Additional 10%
        constraints.Global.RankSep *= 1.12 // Additional 12%
    }
}
```

---

## Improvement Matrix

| Level  | Base Scaling  | 8+ Nodes          | 15+ Nodes         | 20+ Nodes       | Rank Constraints         |
| ------ | ------------- | ----------------- | ----------------- | --------------- | ------------------------ |
| **L1** | 1.15x / 1.20x | +1.20x / +1.25x   | -                 | +1.30x / +1.35x | Persons at top           |
| **L2** | -             | +1.15x / +1.20x   | +1.10x / +1.15x   | +1.30x / +1.35x | Infrastructure at bottom |
| **L3** | -             | +1.15x / +1.18x\* | +1.10x / +1.12x\* | +1.30x / +1.35x | Components aligned       |

\*L3 uses 6+ and 12+ thresholds instead of 8+ and 15+

---

## Examples Covered

### L1 Examples

- ✅ `project_ecommerce.sruja` - Persons, systems, external systems
- ✅ `reference_c4_model.sruja` - Context view examples
- ✅ Any L1 diagram with 8+ nodes gets extra spacing

### L2 Examples

- ✅ `project_saas_platform.sruja` - Many containers
- ✅ `project_iot_platform.sruja` - Complex container hierarchies
- ✅ Any L2 diagram with 8+ containers gets extra spacing

### L3 Examples

- ✅ Component views within any container
- ✅ Dense component layouts (6+ components)
- ✅ Very complex component views (12+ components)

---

## Testing Recommendations

### Test Each Level

1. **L1 Testing:**

   ```bash
   # Open in browser: project_ecommerce.sruja at L1
   # Check: No person-system overlaps
   ```

2. **L2 Testing:**

   ```bash
   # Open in browser: project_saas_platform.sruja at L2
   # Check: Container spacing, no overlaps
   ```

3. **L3 Testing:**
   ```bash
   # Open in browser: Any container's L3 view
   # Check: Component spacing, alignment
   ```

---

## Summary

✅ **All three C4 levels (L1, L2, L3) now have:**

- Level-specific spacing improvements
- Progressive scaling based on complexity
- Universal improvements (edge routing, labels, clusters)
- Rank constraints optimized per level

The improvements are **adaptive** - they apply more spacing as diagrams get more complex, ensuring quality across all levels and sizes.
