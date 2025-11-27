# Parser Decision: Off-the-Shelf vs Custom

**Decision Date**: 2025-01-XX  
**Status**: ✅ **DECIDED**

## Executive Summary

**Decision**: **Use `participle` for all parsing needs - core DSL, extensions, and systems thinking.**

**Confirmed**: All engines and extensions can be implemented using `participle`. No migration needed.

This provides the best balance of:
- ✅ Fast development velocity (critical for solo founder)
- ✅ Good enough for MVP and v1.0
- ✅ Easy migration path if we outgrow it
- ✅ Strong error messages and source location tracking

---

## DSL Complexity Analysis

### Current Requirements

1. **Basic Structure** (MVP)
   - Workspace, model, systems, containers, components
   - Relationships (->, uses, depends)
   - Metadata blocks
   - Comments

2. **Extensions** (v1.0+)
   - Resilience patterns (timeout, retry, circuit breaker)
   - Performance (latency, throughput, SLO)
   - Security (authentication, encryption)
   - Cost (cost models, budgets)
   - Systems thinking (causal relationships, loops, stocks/flows)

3. **Advanced Features** (v2.0+)
   - Multi-module imports
   - Cross-module references
   - Requirements traceability
   - ADRs
   - User journeys

### Parsing Requirements

- ✅ **Source location tracking** - Critical for error messages and LSP
- ✅ **Incremental parsing** - For editor integration (future LSP)
- ✅ **Good error messages** - Developer experience
- ✅ **Extensible grammar** - For extensions
- ✅ **Fast parsing** - CLI tool performance
- ✅ **Bidirectional sync** - Round-trip DSL ↔ Model

---

## Parser Options Comparison

### 1. **participle** (Recommended for MVP)

**What it is**: Struct-based parser combinators for Go

**Pros**:
- ✅ **Go-idiomatic** - Uses struct tags, feels natural
- ✅ **Fast to implement** - Get parser working in days, not weeks
- ✅ **Good error messages** - Built-in error reporting
- ✅ **Source location tracking** - Built-in support
- ✅ **Active maintenance** - Well-maintained library
- ✅ **Easy to test** - Simple test cases
- ✅ **No code generation** - Pure Go code

**Cons**:
- ⚠️ **Limited for complex grammars** - May struggle with very complex constructs
- ⚠️ **Less control** - Can't optimize as much as hand-written
- ⚠️ **Incremental parsing** - May need custom implementation

**Example**:
```go
type Workspace struct {
    Model      *Model      `"workspace" "{" @@ "}"`
    Requirements []Requirement `"requirements" "{" @@* "}"`
}

type System struct {
    Name        string     `@Ident`
    Label       string     `@String`
    Containers  []Container `"{" @@* "}"`
}
```

**Best for**: MVP, v1.0, getting to market fast

---

### 2. **goyacc** (YACC for Go)

**What it is**: Parser generator, generates Go code from grammar

**Pros**:
- ✅ **Powerful** - Handles complex grammars
- ✅ **Proven** - YACC is battle-tested
- ✅ **Good for complex languages** - Can handle Systems Thinking DSL

**Cons**:
- ❌ **Code generation** - Extra build step
- ❌ **Steeper learning curve** - Need to learn YACC syntax
- ❌ **More complex** - Harder to debug
- ❌ **Slower development** - Takes longer to implement

**Best for**: Complex languages, if we outgrow participle

---

### 3. **gocc** (LALR Parser Generator)

**What it is**: LALR parser generator for Go

**Pros**:
- ✅ **Powerful** - Handles complex grammars
- ✅ **Pure Go** - No C dependencies

**Cons**:
- ❌ **Less popular** - Smaller community
- ❌ **Code generation** - Extra build step
- ❌ **Steeper learning curve**

**Best for**: If we need LALR parsing power

---

### 4. **Hand-Written Parser** (Future Option)

**What it is**: Custom recursive descent parser

**Pros**:
- ✅ **Full control** - Optimize for our exact needs
- ✅ **No dependencies** - Pure Go, no external libs
- ✅ **Incremental parsing** - Can implement exactly what we need
- ✅ **Best performance** - Can optimize hot paths
- ✅ **Custom error messages** - Perfect error messages

**Cons**:
- ❌ **Time-consuming** - Weeks to implement properly
- ❌ **More code to maintain** - More surface area for bugs
- ❌ **Slower to develop** - Delays MVP

**Best for**: v2.0+ if we need maximum control/performance

---

## Decision Matrix

| Criteria | participle | goyacc | gocc | Hand-Written |
|----------|-----------|--------|------|--------------|
| **Development Speed** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐ | ⭐ |
| **Grammar Complexity** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Error Messages** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Source Location** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Incremental Parsing** | ⭐⭐ | ⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Maintainability** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| **Performance** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Learning Curve** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐ | ⭐⭐⭐ |

---

## Recommendation: Start with `participle`

### Phase 1: MVP (Current)
**Use `participle`** because:
1. **Fast to market** - Get parser working in days
2. **Good enough** - Handles basic DSL + extensions
3. **Easy to test** - Simple test cases
4. **Good DX** - Error messages and source locations work well

### Phase 2: v1.0 (If Needed)
**Evaluate migration** if:
- We hit grammar limitations
- Performance becomes an issue
- We need advanced incremental parsing

**Migration path**:
- `participle` → `goyacc` (if we need more grammar power)
- `participle` → Hand-written (if we need maximum control)

### Phase 3: v2.0+ (If Needed)
**Consider hand-written parser** if:
- We need advanced incremental parsing for LSP
- Performance is critical
- We want perfect error messages
- We have time to invest

---

## Implementation Strategy

### Step 1: Start with `participle` (MVP)

```go
// pkg/language/parser.go
package language

import "github.com/alecthomas/participle/v2"

type Workspace struct {
    Model       *Model       `"workspace" "{" @@ "}"`
    Requirements []Requirement `("requirements" "{" @@* "}")?`
    ADRs        []ADR        `("adrs" "{" @@* "}")?`
}

type Model struct {
    Systems     []System     `"model" "{" @@* "}"`
}

type System struct {
    Name        string       `@Ident`
    Label       string       `@String`
    Containers  []Container  `("{" @@* "}")?`
    Location    participle.Position
}

// ... more types

func Parse(input string) (*Workspace, error) {
    parser, err := participle.Build[Workspace](
        participle.UseLookahead(2),
        participle.Unquote("String"),
    )
    if err != nil {
        return nil, err
    }
    return parser.ParseString("", input)
}
```

### Step 2: Add Source Location Tracking

`participle` has built-in position tracking via `participle.Position`:

```go
type System struct {
    Name     string              `@Ident`
    Location participle.Position `parser:""`
}
```

### Step 3: Add Error Handling

`participle` provides good error messages:

```go
workspace, err := Parse(input)
if err != nil {
    // participle provides detailed error with position
    return fmt.Errorf("parse error: %w", err)
}
```

### Step 4: Test Extensibility

Test that `participle` can handle:
- Basic DSL ✅
- Extensions (resilience, performance) ✅
- Systems thinking (if needed later) ⚠️

If Systems Thinking DSL is too complex, we can:
- Use `participle` for main DSL
- Hand-write Systems Thinking parser
- Or migrate to `goyacc` for that part

---

## Migration Plan (If Needed)

### When to Migrate

**Migrate from `participle` if**:
1. Grammar becomes too complex for struct tags
2. Performance is a bottleneck (unlikely for CLI tool)
3. We need advanced incremental parsing
4. Error messages aren't good enough

### Migration Path

**Option A: Migrate to `goyacc`**
- Keep same AST structure
- Rewrite grammar in YACC syntax
- Generate parser code
- Update tests

**Option B: Migrate to Hand-Written**
- Keep same AST structure
- Write recursive descent parser
- Implement incremental parsing
- Update tests

**Option C: Hybrid**
- Use `participle` for main DSL
- Hand-write parser for Systems Thinking (if too complex)
- Combine results

---

## Real-World Examples

### Similar Projects Using `participle`

- **HCL (HashiCorp)** - Uses `participle` for configuration language
- **CUE** - Uses similar approach (though custom parser)
- **Many Go DSLs** - Use `participle` for simplicity

### When Hand-Written Makes Sense

- **Go compiler** - Hand-written for performance
- **Rust compiler** - Hand-written for control
- **TypeScript** - Hand-written for complex grammar

**For Sruja**: We're not building a general-purpose language compiler. We're building a DSL for architecture modeling. `participle` is perfect for this.

---

## Final Recommendation

**Start with `participle`** because:

1. ✅ **Fastest path to MVP** - Get working parser in days
2. ✅ **Good enough for v1.0** - Handles our requirements
3. ✅ **Easy migration** - Can switch later if needed
4. ✅ **Good DX** - Error messages and source locations work well
5. ✅ **Active maintenance** - Well-maintained library

**Re-evaluate in v2.0** if:
- We need advanced incremental parsing for LSP
- Performance becomes critical
- Grammar becomes too complex

**Don't optimize prematurely** - `participle` will get us to market faster, and we can always migrate later if needed.

---

## Action Items

1. ✅ **Start with `participle`** - Implement basic parser
2. ✅ **Test with MVP grammar** - Verify it handles our needs
3. ✅ **Add source location tracking** - Use `participle.Position`
4. ✅ **Test extensions** - Verify resilience/performance DSL works
5. ⚠️ **Monitor complexity** - Watch for grammar limitations
6. ⚠️ **Plan migration** - If we outgrow `participle`, plan migration

---

*Start simple, optimize later. `participle` gets us to market faster.*

