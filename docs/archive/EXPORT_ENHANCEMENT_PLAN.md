# Export Enhancement Plan: AI-Friendly Features

## Overview

Enhance existing markdown and JSON exports with AI-friendly features instead of creating a separate export system. This approach:
- Reuses existing code
- Maintains backward compatibility
- Adds value incrementally
- Requires less maintenance

## Goals

1. **Fix markdown export** (currently disabled in CLI)
2. **Add scoping** - export specific systems/containers/components
3. **Add token optimization** - smart filtering for AI token limits
4. **Add context hints** - format for different use cases (code generation, review, analysis)
5. **Add relationship prioritization** - focus on important connections

## Implementation Plan

### Phase 1: Fix & Enable Markdown Export (Week 1)

**Current State:**
- Markdown exporter exists in `pkg/export/markdown/`
- CLI shows error: "markdown export not yet updated for LikeC4 syntax"
- Exporter needs to work with LikeC4 AST structure

**Tasks:**
1. Update markdown exporter to work with LikeC4 AST
2. Enable markdown format in `cmd/sruja/export.go`
3. Add basic tests
4. Verify output quality

**Files to Modify:**
- `pkg/export/markdown/markdown.go` - Update to use LikeC4 AST
- `cmd/sruja/export.go` - Enable markdown case
- Add tests in `pkg/export/markdown/markdown_test.go`

### Phase 2: Add Scoping (Week 1-2)

**Feature:** Export only specific parts of architecture

**CLI Usage:**
```bash
# Export entire architecture (default)
sruja export markdown arch.sruja

# Export specific system
sruja export markdown arch.sruja --scope=system:OrderService

# Export specific container
sruja export markdown arch.sruja --scope=container:Web

# Export specific component
sruja export markdown arch.sruja --scope=component:API

# Works with JSON too
sruja export json arch.sruja --scope=system:OrderService
```

**Implementation:**
1. Add `Scope` field to export options
2. Add scope parsing logic (parse "system:Name" format)
3. Filter elements based on scope
4. Include related elements (containers within system, etc.)
5. Include relationships involving scoped elements

**Files to Modify:**
- `pkg/export/markdown/options.go` - Add Scope field
- `pkg/export/json/json.go` - Add scope support
- `pkg/export/markdown/markdown.go` - Implement filtering
- `cmd/sruja/export.go` - Add --scope flag

**Scope Format:**
```go
type Scope struct {
    Type string // "system", "container", "component", "full"
    ID   string // Element ID (e.g., "OrderService")
}

func ParseScope(scopeStr string) (*Scope, error) {
    // Parse "system:OrderService" format
}
```

### Phase 3: Token Optimization (Week 2)

**Feature:** Smart filtering to fit within token limits

**CLI Usage:**
```bash
# Limit to ~8000 tokens (rough estimate)
sruja export markdown arch.sruja --token-limit=8000

# No limit (default)
sruja export markdown arch.sruja
```

**Optimization Strategy:**
1. Estimate tokens (rough: ~4 chars per token)
2. If over limit, apply optimizations:
   - Remove less important elements (components before containers)
   - Truncate long descriptions
   - Summarize relationships (show count, not all)
   - Remove metadata if not critical
   - Prioritize: Systems > Containers > Components
3. Always include:
   - System/container names
   - Key relationships
   - Requirements and ADRs (summarized if needed)

**Implementation:**
```go
// pkg/export/optimizer.go
type TokenOptimizer struct {
    limit int
}

func (o *TokenOptimizer) Optimize(program *language.Program, scope *Scope) *language.Program {
    // Apply filtering and truncation
    // Return optimized program
}

func EstimateTokens(content string) int {
    // Rough estimate: ~4 characters per token
    return len(content) / 4
}
```

**Files to Create/Modify:**
- `pkg/export/optimizer.go` - Token optimization logic
- `pkg/export/markdown/options.go` - Add TokenLimit field
- `pkg/export/json/json.go` - Add token limit support
- `cmd/sruja/export.go` - Add --token-limit flag

### Phase 4: Context Hints (Week 2-3)

**Feature:** Format output for different use cases

**CLI Usage:**
```bash
# For code generation (focus on tech stack, relationships)
sruja export markdown arch.sruja --context=code_generation

# For architecture review (focus on ADRs, requirements)
sruja export markdown arch.sruja --context=review

# For analysis (focus on relationships, data flows)
sruja export markdown arch.sruja --context=analysis

# Default: balanced
sruja export markdown arch.sruja
```

**Context Types:**
- `code_generation`: Prioritize technology stack, interfaces, dependencies
- `review`: Prioritize ADRs, requirements, decisions
- `analysis`: Prioritize relationships, data flows, dependencies
- `default`: Balanced approach

**Implementation:**
```go
// pkg/export/markdown/options.go
type ContextType string

const (
    ContextDefault        ContextType = "default"
    ContextCodeGeneration ContextType = "code_generation"
    ContextReview         ContextType = "review"
    ContextAnalysis       ContextType = "analysis"
)

type Options struct {
    // ... existing fields
    Context ContextType
}

// In markdown exporter
func (e *Exporter) writeSystems(sb *strings.Builder, arch interface{}) {
    // Adjust content based on e.Options.Context
    switch e.Options.Context {
    case ContextCodeGeneration:
        // Emphasize technology, interfaces
    case ContextReview:
        // Emphasize ADRs, requirements
    case ContextAnalysis:
        // Emphasize relationships, flows
    }
}
```

**Files to Modify:**
- `pkg/export/markdown/options.go` - Add Context field
- `pkg/export/markdown/markdown.go` - Adjust formatting based on context
- `cmd/sruja/export.go` - Add --context flag

### Phase 5: Relationship Prioritization (Week 3)

**Feature:** Show most important relationships first

**Implementation:**
1. Score relationships by importance:
   - Direct relationships (higher priority)
   - Relationships with requirements/ADRs (higher priority)
   - Data flow relationships (higher priority)
   - Simple references (lower priority)
2. Sort and limit based on token budget
3. Show relationship summaries when space is limited

**Files to Modify:**
- `pkg/export/markdown/markdown.go` - Add relationship prioritization
- `pkg/export/optimizer.go` - Add relationship scoring

## File Structure

```
pkg/export/
├── markdown/
│   ├── markdown.go          # Main exporter (enhanced)
│   ├── options.go            # Options struct (enhanced)
│   ├── scope.go              # NEW: Scope parsing and filtering
│   └── markdown_test.go      # Tests
├── json/
│   ├── json.go               # Enhanced with scoping
│   └── ...
└── optimizer.go              # NEW: Token optimization logic
```

## CLI Changes

**Enhanced export command:**
```bash
sruja export <format> <file> [options]

Options:
  --scope=<type>:<id>     Scope to specific element (system:Name, container:Name)
  --token-limit=<n>       Limit output to approximately N tokens (0 = no limit)
  --context=<type>        Context type: default, code_generation, review, analysis
  --extended              Include pre-computed views (JSON only)
  --compact               Compact output (JSON only)
```

**Examples:**
```bash
# Basic usage (unchanged)
sruja export json arch.sruja
sruja export markdown arch.sruja

# Scoped export
sruja export markdown arch.sruja --scope=system:OrderService

# Token-optimized for AI
sruja export markdown arch.sruja --token-limit=8000 --context=code_generation

# Scoped + optimized
sruja export markdown arch.sruja --scope=container:Web --token-limit=4000 --context=code_generation
```

## Testing Strategy

1. **Unit Tests:**
   - Scope parsing and filtering
   - Token estimation
   - Optimization logic
   - Context-based formatting

2. **Integration Tests:**
   - End-to-end export with all options
   - Verify output quality
   - Verify token limits are respected

3. **Example Tests:**
   - Test with real architecture files
   - Compare outputs with/without optimizations
   - Verify scoping works correctly

## Success Criteria

1. ✅ Markdown export works with LikeC4 AST
2. ✅ Scoping works for system/container/component
3. ✅ Token optimization reduces output when needed
4. ✅ Context hints change output appropriately
5. ✅ Backward compatible (default behavior unchanged)
6. ✅ All features work with both markdown and JSON

## Migration Notes

- **Backward Compatibility:** All new features are opt-in
- **Default Behavior:** Unchanged (full export, no limits)
- **Breaking Changes:** None

## Future Enhancements (Out of Scope)

- XML format for Claude (can be added later if needed)
- Template system (can be added later)
- LSP integration (separate feature)
- Traceability (separate feature)

## Timeline

- **Week 1:** Fix markdown export, add scoping
- **Week 2:** Token optimization, context hints
- **Week 3:** Relationship prioritization, testing, polish

**Total: 3 weeks** (vs 6-8 weeks for separate AI export system)
