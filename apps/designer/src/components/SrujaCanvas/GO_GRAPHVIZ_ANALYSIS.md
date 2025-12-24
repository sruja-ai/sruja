# go-graphviz Analysis: Benefits for Option 3

## What is go-graphviz?

`github.com/goccy/go-graphviz` is a **pure Go library** that:

- Embeds Graphviz as WebAssembly
- Provides Go bindings for Graphviz functionality
- Supports DOT language encoding/decoding
- Can generate layouts directly in Go

## Current Architecture (Option 3 Without go-graphviz)

```
Go WASM (Backend)              TypeScript (Frontend)
─────────────────              ─────────────────────
1. Generate DOT
   ↓
2. Return DOT ──────────────→ 3. Call Graphviz WASM
                                (@hpcc-js/wasm-graphviz)
                                ↓
                              4. Get JSON Output
                                ↓
                              5. Pass JSON back ──→ 6. Parse JSON
                                                     ↓
                                                    7. Measure Quality
                                                     ↓
                                                    8. Refine Constraints
                                                     ↓
                                                    9. Generate New DOT
```

**Problems:**

- Multiple round-trips (Go → TS → Go)
- JSON parsing in Go (complex Graphviz JSON structure)
- State management across WASM boundary
- Performance overhead

## Architecture WITH go-graphviz

```
Go WASM (Backend)              TypeScript (Frontend)
─────────────────              ─────────────────────
1. Generate DOT
   ↓
2. Call Graphviz (go-graphviz)
   ↓
3. Get JSON Output (directly in Go)
   ↓
4. Measure Quality (in Go)
   ↓
5. Refine Constraints (in Go)
   ↓
6. Generate New DOT
   ↓
7. Repeat if needed
   ↓
8. Return Final DOT ─────────→ 9. Use DOT for React Flow
                                 (or return positions directly)
```

**Benefits:**

- ✅ **No round-trips** - Everything happens in Go
- ✅ **No JSON parsing** - Go types directly
- ✅ **Simpler architecture** - Single language
- ✅ **Better performance** - No WASM boundary crossing
- ✅ **Easier quality metrics** - Direct access to layout data

## Detailed Benefits

### 1. **Eliminates JSON Parsing Complexity** ⚠️ HIGH BENEFIT

**Without go-graphviz:**

- Need to parse complex Graphviz JSON structure
- Handle coordinate conversions
- Map Graphviz IDs to node names
- 4-6 hours of work

**With go-graphviz:**

- Direct Go types from Graphviz
- No JSON parsing needed
- Type-safe access to layout data
- **Saves: 4-6 hours**

### 2. **Eliminates Round-Trips** ⚠️ HIGH BENEFIT

**Without go-graphviz:**

```
Go → TypeScript → Graphviz → TypeScript → Go → TypeScript
```

- Each iteration = 3 round-trips
- 2-3 iterations = 6-9 round-trips
- Each round-trip = ~10-20ms overhead

**With go-graphviz:**

```
Go → Graphviz → Go → Graphviz → Go
```

- Everything in Go
- No round-trips
- **Saves: 60-180ms per refinement cycle**

### 3. **Simpler Quality Metrics** ⚠️ MEDIUM BENEFIT

**Without go-graphviz:**

- Parse JSON to extract node positions
- Parse JSON to extract edge splines
- Convert coordinates
- Build internal representation

**With go-graphviz:**

- Direct access to node positions (Go types)
- Direct access to edge splines (Go types)
- No conversion needed
- **Saves: 2-3 hours**

### 4. **Better Performance** ⚠️ MEDIUM BENEFIT

**Without go-graphviz:**

- WASM boundary crossing overhead
- JSON serialization/deserialization
- Multiple async operations

**With go-graphviz:**

- Single Go execution context
- Direct memory access
- Synchronous operations
- **Faster by 20-30%**

### 5. **Type Safety** ⚠️ LOW-MEDIUM BENEFIT

**Without go-graphviz:**

- JSON parsing can fail
- Type assertions needed
- Runtime errors possible

**With go-graphviz:**

- Compile-time type safety
- No JSON parsing errors
- Better error handling

## Complexity Reduction

| Component              | Without go-graphviz | With go-graphviz | Savings    |
| ---------------------- | ------------------- | ---------------- | ---------- |
| WASM Function          | 1-2h                | 1-2h             | 0h         |
| JSON Parsing           | 4-6h                | **0h**           | **4-6h**   |
| Quality Metrics        | 8-12h               | **6-8h**         | **2-4h**   |
| Refinement Loop        | 4-6h                | **2-3h**         | **2-3h**   |
| TypeScript Integration | 3-4h                | **1-2h**         | **2h**     |
| **TOTAL**              | **20-30h**          | **10-15h**       | **10-15h** |

## Challenges & Considerations

### 1. **Maintenance Status** ⚠️ MEDIUM RISK

**Issue:**

- Last release: Over 1 year ago
- Limited recent activity
- May not be actively maintained

**Impact:**

- Bug fixes may be slow
- New Graphviz features may not be available
- Security updates may lag

**Mitigation:**

- Fork and maintain if needed
- Use as-is if it works (Graphviz is stable)
- Monitor for critical issues

### 2. **WASM Compatibility** ⚠️ LOW-MEDIUM RISK

**Issue:**

- go-graphviz embeds WASM
- We're already building Go to WASM
- Need to ensure it works in browser WASM context

**Questions:**

- Does go-graphviz's embedded WASM work when Go itself is compiled to WASM?
- Or does it use cgo (which doesn't work in WASM)?

**Investigation Needed:**

- Check if go-graphviz uses cgo
- Test if it works in browser WASM
- Verify performance

### 3. **WASM Size** ⚠️ LOW RISK

**Issue:**

- go-graphviz embeds Graphviz WASM
- Our WASM already includes Graphviz (via @hpcc-js)
- Potential duplication

**Impact:**

- WASM size may increase
- But we could remove @hpcc-js dependency

**Mitigation:**

- Measure actual size increase
- Consider if worth it (probably yes)

### 4. **API Differences** ⚠️ LOW RISK

**Issue:**

- go-graphviz API may differ from @hpcc-js
- Need to adapt code

**Impact:**

- Minor refactoring needed
- Should be straightforward

## Implementation Approach

### Phase 1: Proof of Concept (2-3 hours)

1. Add go-graphviz dependency
2. Test if it works in WASM build
3. Create simple test: Generate DOT → Layout → Get positions
4. Verify it works in browser

### Phase 2: Integration (4-6 hours)

1. Replace @hpcc-js Graphviz calls with go-graphviz
2. Update quality metrics to use Go types
3. Implement refinement loop in Go
4. Update TypeScript to use new API

### Phase 3: Optimization (2-3 hours)

1. Optimize performance
2. Add caching
3. Error handling
4. Testing

**Total: 8-12 hours** (vs 20-30 hours without)

## Code Example

### Without go-graphviz (Current)

```go
// cmd/wasm/main.go
func refineLayout(this js.Value, args []js.Value) interface{} {
    graphvizJson := args[0].String() // JSON string from TypeScript

    // Parse JSON (complex)
    var gvData GraphvizJSON
    json.Unmarshal([]byte(graphvizJson), &gvData)

    // Extract positions (manual parsing)
    nodes := make(map[string]NodePos)
    for _, obj := range gvData.Objects {
        // Parse "x,y" string, convert coordinates...
    }

    // Measure quality
    quality := measureQuality(nodes, gvData.Edges)

    // Refine...
}
```

### With go-graphviz

```go
import "github.com/goccy/go-graphviz"

func refineLayout(dot string, maxIterations int) (string, LayoutQuality) {
    g := graphviz.New()
    defer g.Close()

    graph, _ := graphviz.ParseBytes([]byte(dot))
    defer graph.Close()

    for i := 0; i < maxIterations; i++ {
        // Layout directly in Go
        renderer, _ := g.Render(graph, "dot", "json")
        jsonData, _ := io.ReadAll(renderer)

        // Parse to Go structs (type-safe)
        var layout LayoutResult
        json.Unmarshal(jsonData, &layout)

        // Measure quality (direct access to positions)
        quality := measureQuality(layout)

        if !quality.NeedsRefinement() {
            break
        }

        // Refine constraints
        dot = refineConstraints(dot, quality)
        graph, _ = graphviz.ParseBytes([]byte(dot))
    }

    return dot, quality
}
```

## Recommendation

### ✅ **Use go-graphviz IF:**

1. It works in WASM build (needs testing)
2. Maintenance status is acceptable (or you can fork)
3. Performance is acceptable

### ❌ **Don't use go-graphviz IF:**

1. It doesn't work in WASM (uses cgo)
2. Maintenance is a concern
3. You need latest Graphviz features

## Next Steps

1. **Test WASM Compatibility** (1 hour)

   ```bash
   go get github.com/goccy/go-graphviz
   # Try to build for WASM
   GOOS=js GOARCH=wasm go build ./cmd/wasm
   ```

2. **Create Proof of Concept** (2 hours)
   - Simple DOT → Layout → Positions
   - Test in browser

3. **If it works: Proceed with integration** (8-10 hours)
4. **If it doesn't: Fall back to Option 2** (TypeScript quality metrics)

## Conclusion

**go-graphviz could reduce Option 3 complexity by 50%** (10-15 hours saved), but:

- ⚠️ **Needs testing** - May not work in WASM
- ⚠️ **Maintenance risk** - Inactive project
- ✅ **High benefit** - If it works, significantly simpler

**Recommendation: Test first, then decide.**
