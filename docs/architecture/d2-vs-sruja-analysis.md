# D2 vs Sruja: Can We Extend D2?

**Question**: Should we extend [D2](https://d2lang.com) for Sruja's vision, or build our own language?

[← Back to Architecture Documentation](./README.md)

## Executive Summary

**Recommendation**: **Build Sruja as a separate language**, but consider D2 as a **compilation target** (like Mermaid).

**Why?**
- D2 is a **general-purpose diagramming language**
- Sruja is an **architecture-specific modeling language** with domain semantics
- Sruja needs features D2 doesn't provide (requirements, ADRs, validation, extensions)
- We can compile Sruja → D2 (like we compile to Mermaid)

---

## D2 Overview

[D2](https://d2lang.com) is a modern declarative diagramming language that:
- ✅ Turns text to diagrams
- ✅ Has beautiful themes and styling
- ✅ Supports animations, sketch mode, LaTeX
- ✅ Has VS Code extension
- ✅ Exports to SVG, PNG, PDF
- ✅ Has local CLI with watch mode
- ✅ Supports containers, images, icons
- ✅ Has layout engines

**D2 is excellent for general diagramming.**

---

## Sruja's Unique Requirements

### 1. Architecture-Specific Semantics

**Sruja needs**:
- Systems, Containers, Components (C4 model concepts)
- Relationships with semantic meaning (uses, depends, publishes, subscribes)
- Multi-layer modeling (VHLD, HLD, LLD)
- Architecture-specific validation rules

**D2 provides**:
- Generic shapes and connections
- No built-in architecture semantics
- No validation rules

**Gap**: D2 doesn't understand architecture concepts - it's just shapes and lines.

---

### 2. Requirements & ADRs

**Sruja needs**:
```sruja
requirements {
  R1: functional "Must handle 10k concurrent users"
  R2: constraint "Must use PostgreSQL"
}

adrs {
  ADR001: "Use microservices architecture"
}
```

**D2 provides**:
- No first-class support for requirements
- No ADR support
- No traceability links

**Gap**: D2 is for diagrams, not requirements management.

---

### 3. Extensions System

**Sruja needs**:
- Resilience extensions (timeout, retry, circuit breaker)
- Performance extensions (latency, throughput, SLO)
- Security extensions (authentication, encryption)
- Cost extensions (cost models, budgets)
- Systems thinking (causal relationships, feedback loops)

**D2 provides**:
- Generic styling and theming
- No domain-specific extensions
- No architecture-specific concepts

**Gap**: D2 can't express architecture-specific properties.

---

### 4. Validation & Rules

**Sruja needs**:
- Semantic validation (no circular dependencies, valid references)
- Architecture rules (layers, boundaries, constraints)
- Custom validation plugins
- Cross-module validation

**D2 provides**:
- Syntax validation only
- No semantic validation
- No architecture rules

**Gap**: D2 doesn't validate architecture correctness.

---

### 5. Multi-Module Composition

**Sruja needs**:
- Import statements
- Cross-module references
- Global model composition
- Namespace management

**D2 provides**:
- Imports (basic)
- No cross-module reference resolution
- No global composition
- No namespace management

**Gap**: D2's imports are for code organization, not architecture composition.

---

### 6. Systems Thinking

**Sruja needs**:
- Causal relationships
- Feedback loops
- Stocks & flows
- System dynamics simulation

**D2 provides**:
- No systems thinking concepts
- No causal modeling
- No simulation capabilities

**Gap**: D2 can't model system dynamics.

---

## Comparison Table

| Feature | D2 | Sruja | Can D2 Support? |
|---------|----|----|-----------------|
| **General Diagramming** | ✅ Excellent | ⚠️ Basic | ✅ Yes |
| **Architecture Semantics** | ❌ No | ✅ Yes | ❌ No (not extensible) |
| **Requirements** | ❌ No | ✅ Yes | ❌ No |
| **ADRs** | ❌ No | ✅ Yes | ❌ No |
| **Validation Rules** | ❌ No | ✅ Yes | ❌ No |
| **Extensions** | ⚠️ Styling only | ✅ Domain-specific | ❌ No |
| **Multi-layer Modeling** | ❌ No | ✅ Yes | ❌ No |
| **Systems Thinking** | ❌ No | ✅ Yes | ❌ No |
| **Compilation Targets** | ✅ SVG/PNG/PDF | ✅ Mermaid/D2/PlantUML | ✅ Yes (we compile to D2) |

---

## Can We Extend D2?

### Option 1: Fork D2

**Pros**:
- ✅ Start with existing parser and tooling
- ✅ Get diagram rendering for free

**Cons**:
- ❌ Would need to add architecture semantics (major changes)
- ❌ Would need to add requirements/ADRs (not diagram concepts)
- ❌ Would need to add validation engine (not in D2)
- ❌ Would need to add extensions system (not in D2)
- ❌ Would need to add systems thinking (not in D2)
- ❌ Would diverge significantly from D2
- ❌ Maintenance burden (keeping up with D2 updates)

**Verdict**: ❌ **Not practical** - would require rewriting most of D2.

---

### Option 2: Build on Top of D2 (Preprocessor)

**Approach**: Create Sruja DSL that compiles to D2.

**Pros**:
- ✅ Use D2 for diagram rendering
- ✅ Keep Sruja's architecture semantics
- ✅ Can add requirements/ADRs as metadata
- ✅ Can add validation before compilation

**Cons**:
- ⚠️ D2 syntax might not map well to architecture concepts
- ⚠️ Would need to translate architecture semantics to D2 shapes
- ⚠️ Requirements/ADRs would need separate storage (not in diagram)
- ⚠️ Validation happens before D2 (not in D2)
- ⚠️ Extensions would need custom translation

**Verdict**: ⚠️ **Possible but limiting** - D2 becomes just a rendering target.

---

### Option 3: Use D2 as Compilation Target (Recommended)

**Approach**: Build Sruja as independent language, compile to D2 (like we compile to Mermaid).

**Pros**:
- ✅ Keep Sruja's full architecture semantics
- ✅ Support all Sruja features (requirements, ADRs, validation, extensions)
- ✅ Use D2 for beautiful diagram rendering
- ✅ Can also compile to Mermaid, PlantUML, etc.
- ✅ Independent language evolution
- ✅ No dependency on D2's roadmap

**Cons**:
- ⚠️ Need to build our own parser (but we're using `participle` - easy)
- ⚠️ Need to translate to D2 syntax (but we already do this for Mermaid)

**Verdict**: ✅ **Best approach** - Use D2 as a rendering target, not the language.

---

## Recommended Architecture

```
Sruja DSL (.sruja)
    ↓
Parser (participle) → AST
    ↓
Validation Engine
    ↓
Model Compiler
    ↓
┌─────────────────┬──────────────┬──────────────┐
│   Mermaid       │     D2       │   PlantUML   │
│   Compiler      │   Compiler   │   Compiler   │
└─────────────────┴──────────────┴──────────────┘
```

**Benefits**:
- ✅ Sruja keeps its architecture semantics
- ✅ Can render to multiple formats (Mermaid, D2, PlantUML)
- ✅ Users can choose their preferred diagram format
- ✅ D2 gets beautiful diagrams from Sruja
- ✅ No compromise on Sruja's features

---

## D2 Compiler Implementation

Add D2 as a compilation target (similar to Mermaid):

```go
// pkg/compiler/d2.go
package compiler

func CompileToD2(model *Model) string {
    var d2 strings.Builder
    
    // Convert architecture model to D2 syntax
    for _, element := range model.Elements {
        switch element.Type {
        case "system":
            d2.WriteString(fmt.Sprintf("%s: {\n", element.ID))
            d2.WriteString(fmt.Sprintf("  label: %q\n", element.Name))
            d2.WriteString("}\n")
        case "container":
            // Convert container to D2 shape
        case "component":
            // Convert component to D2 shape
        }
    }
    
    // Convert relationships
    for _, rel := range model.Relations {
        d2.WriteString(fmt.Sprintf("%s -> %s: %q\n", 
            rel.From, rel.To, rel.Label))
    }
    
    return d2.String()
}
```

**Usage**:
```bash
sruja compile --format d2 example.sruja
# Outputs: example.d2
```

---

## What We Can Learn from D2

Even though we're building our own language, we can learn from D2:

### 1. Beautiful Themes
- ✅ D2 has production-ready themes
- ✅ We can compile to D2 to get these themes
- ✅ Or create our own themes for Mermaid

### 2. Sketch Mode
- ✅ D2 has hand-drawn aesthetic
- ✅ We can compile to D2 for sketch mode
- ✅ Or add sketch mode to our Mermaid compiler

### 3. Animations
- ✅ D2 supports animations
- ✅ We can compile to D2 for animated diagrams
- ✅ Useful for showing architecture evolution

### 4. VS Code Extension
- ✅ D2 has official VS Code extension
- ✅ We can learn from their implementation
- ✅ Our extension can be similar

### 5. Watch Mode
- ✅ D2 CLI has watch mode
- ✅ We should add this to Sruja CLI
- ✅ Auto-compile on file changes

---

## Conclusion

**Build Sruja as an independent language**, but:

1. ✅ **Use D2 as a compilation target** (like Mermaid)
2. ✅ **Learn from D2's features** (themes, animations, watch mode)
3. ✅ **Compile to D2** for beautiful diagram rendering
4. ✅ **Keep Sruja's architecture semantics** (requirements, ADRs, validation, extensions)

**Why?**
- Sruja is **architecture-specific** (not general diagramming)
- D2 is **general-purpose** (not architecture-specific)
- They serve different purposes
- We can use D2 for what it's good at (rendering) while keeping Sruja for what it's good at (architecture modeling)

**Result**: Best of both worlds - Sruja's architecture semantics + D2's beautiful rendering.

---

## Action Items

1. ✅ **Keep building Sruja as independent language**
2. ✅ **Add D2 compiler** (similar to Mermaid compiler)
3. ✅ **Add `--format d2` flag** to `sruja compile`
4. ✅ **Learn from D2's features** (themes, animations, watch mode)
5. ✅ **Consider D2 for VS Code preview** (in addition to Mermaid)

---

*D2 is excellent for diagramming, but Sruja is for architecture modeling. Use D2 as a rendering target, not the language itself.*

