# WASM Size Impact of Builder API

## Current WASM Build

### What's Included
Looking at `cmd/wasm/main.go`, the WASM currently includes:

```go
import (
    "github.com/sruja-ai/sruja/pkg/engine"      // Validator
    "github.com/sruja-ai/sruja/pkg/export/json" // JSON exporter
    "github.com/sruja-ai/sruja/pkg/export/markdown" // Markdown exporter
    "github.com/sruja-ai/sruja/pkg/export/mermaid"  // Mermaid exporter
    "github.com/sruja-ai/sruja/pkg/language"        // Parser & AST
)
```

**Current WASM functions:**
- Parse DSL → JSON
- Validate DSL
- Export to JSON/Markdown/Mermaid/LikeC4
- LSP features (hover, completion, definition, etc.)

### Build Optimization
```bash
# From scripts/build-wasm.sh
GOOS=js GOARCH=wasm go build \
  -ldflags="-s -w" \      # Strip debug info and symbols
  -trimpath \             # Remove file system paths
  -o sruja.wasm

# Further optimization
wasm-opt -Oz sruja.wasm   # Maximum size optimization
```

**Current size:**
- Uncompressed: ~4.8MB (mentioned in package.json)
- Compressed (gzip): ~1.4MB
- Compressed (brotli): Even smaller

## Does Builder API Need to Be in WASM?

### Option 1: Builder Only in Backend/CLI ✅ **NO WASM Impact**

**Implementation:**
```go
// pkg/builder/builder.go (new package)
package builder

// Builder API for Go backend/CLI only
type Builder struct { ... }
```

**WASM impact:** **ZERO**
- Builder code is in a separate package
- `cmd/wasm/main.go` doesn't import it
- Go's dead code elimination removes it from WASM build
- Only included if explicitly imported

**Use cases:**
- CLI tools (init, code generation)
- Backend tests
- Import/export conversions
- Server-side code generation

**Verdict:** ✅ **Best approach** - No WASM size increase

---

### Option 2: Builder in WASM (Go) ❌ **Increases WASM Size**

**If you add to WASM:**
```go
// cmd/wasm/main.go
import (
    "github.com/sruja-ai/sruja/pkg/builder" // NEW
)

js.Global().Set("sruja_build_model", js.FuncOf(buildModel))
```

**Estimated size impact:**
- Builder implementation: ~50-100KB of Go code
- After compilation: ~200-500KB in WASM (Go runtime overhead)
- With optimization: ~100-300KB
- Compressed: ~30-100KB

**Total WASM size:**
- Current: ~4.8MB (1.4MB compressed)
- With Builder: ~5.0-5.3MB (~1.5MB compressed)
- **Increase: ~5-10%**

**Use cases:**
- Programmatic model creation in browser
- Designer app creating models without DSL
- But: Designer already has parser, so DSL works fine

**Verdict:** ❌ **Not recommended** - Adds size for limited benefit

---

### Option 3: Builder in TypeScript (Frontend) ✅ **NO WASM Impact**

**Implementation:**
```typescript
// apps/designer/src/builder/Builder.ts (new file)
export class Builder {
  static forSpecification(spec: Specification) {
    // Pure TypeScript implementation
  }
  
  toSrujaDsl(): string {
    // Generate DSL string
  }
  
  toModelDump(): SrujaModelDump {
    // Generate model structure directly
  }
}
```

**WASM impact:** **ZERO**
- Pure TypeScript, no Go code
- Not part of WASM build
- Bundled with frontend code (separate from WASM)

**Size impact:**
- TypeScript bundle: ~10-50KB (minified)
- Much smaller than Go WASM version
- Better performance (no WASM overhead)

**Use cases:**
- Designer app programmatic model creation
- UI-driven model generation
- Can generate DSL or model structure directly

**Verdict:** ✅ **Best for frontend** - No WASM impact, better performance

---

## Size Comparison

| Approach | WASM Size | Frontend Bundle | Total Impact |
|----------|----------|-----------------|--------------|
| **Current** | 4.8MB (1.4MB gz) | - | Baseline |
| **Go Builder in WASM** | +200-500KB | - | +5-10% WASM |
| **Go Builder (backend only)** | 0KB | - | No change |
| **TS Builder (frontend)** | 0KB | +10-50KB | Minimal |

## Recommendations

### 1. **Backend Builder (Go)** ✅ Recommended
- **Location:** `pkg/builder/` (new package)
- **WASM impact:** None (not imported in `cmd/wasm/main.go`)
- **Use cases:** CLI, tests, code generation, imports
- **Size:** No WASM impact

### 2. **Frontend Builder (TypeScript)** ✅ Recommended
- **Location:** `apps/designer/src/builder/` or `packages/shared/src/builder/`
- **WASM impact:** None (pure TypeScript)
- **Use cases:** Designer app, UI-driven creation
- **Size:** ~10-50KB in frontend bundle (separate from WASM)

### 3. **Builder in WASM (Go)** ❌ Not Recommended
- **WASM impact:** +200-500KB (~5-10% increase)
- **Use cases:** Limited (designer already has parser)
- **Size:** Significant increase for minimal benefit

## Implementation Strategy

### Phase 1: Backend Builder (No WASM Impact)
```go
// pkg/builder/builder.go
// Only used in cmd/sruja/, tests, CLI tools
// NOT imported in cmd/wasm/main.go
```

### Phase 2: Frontend Builder (No WASM Impact)
```typescript
// packages/shared/src/builder/Builder.ts
// Pure TypeScript, generates DSL or model structure
// Used in designer app
```

### Result
- ✅ Backend can create models programmatically
- ✅ Frontend can create models programmatically
- ✅ **Zero WASM size increase**
- ✅ Better performance (TypeScript vs WASM for frontend)

## Conclusion

**Implementing Builder API will NOT increase WASM size if:**

1. **Go Builder** is only in backend/CLI (not imported in `cmd/wasm/main.go`)
2. **TypeScript Builder** is used in frontend (separate from WASM)

**WASM size will increase if:**
- You add Builder to `cmd/wasm/main.go` imports
- You expose Builder functions via WASM

**Recommendation:**
- Use Go Builder for backend/CLI (no WASM impact)
- Use TypeScript Builder for frontend (no WASM impact)
- **Avoid adding Builder to WASM** (unnecessary size increase)

The current WASM is already optimized and includes only what's needed for parsing/validation/export. Builder API doesn't need to be in WASM since:
- Designer app can use TypeScript Builder
- Backend can use Go Builder
- Both are separate from WASM build
