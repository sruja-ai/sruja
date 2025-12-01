# Proposed Metadata Syntax Change: Remove Colon Requirement

## Overview

Remove the colon (`:`) requirement from metadata syntax to make it consistent with `ConstraintEntry` and `ConventionEntry`, which use `@Ident @String` (no colon).

## Current State

### Parser Definition
```go
// pkg/language/ast.go
type MetaEntry struct {
    Key   string `parser:"@Ident ':'"`  // Colon is REQUIRED
    Value string `parser:"@String"`
}
```

### Current Syntax (with colon)
```sruja
metadata {
  team: "Payments"
  tier: "critical"
}
```

### Comparison with Similar Constructs
- **MetaEntry**: `@Ident ':' @String` (with colon)
- **ConstraintEntry**: `@Ident @String` (no colon)
- **ConventionEntry**: `@Ident @String` (no colon)

## Proposed Change

### 1. Update Parser Definition

**File**: `pkg/language/ast.go`

**Change**:
```go
// BEFORE
type MetaEntry struct {
    Key   string `parser:"@Ident ':'"`
    Value string `parser:"@String"`
}

// AFTER
type MetaEntry struct {
    Key   string `parser:"@Ident"`
    Value string `parser:"@String"`
}
```

**Impact**: Parser will accept metadata without colon, matching constraints/conventions syntax.

### 2. Update Printer

**File**: `pkg/language/printer.go`

**Change**:
```go
// BEFORE
fmt.Fprintf(sb, "%s%s: %q\n", indent, entry.Key, entry.Value)

// AFTER
fmt.Fprintf(sb, "%s%s %q\n", indent, entry.Key, entry.Value)
```

**Impact**: Printer will output metadata without colon.

### 3. Update Examples

**Files to Update**:
- `examples/metadata_showcase.sruja`
- `examples/top_level_component.sruja`
- `examples/ddd_advanced.sruja`

**Change**: Remove colons from all metadata entries
```sruja
// BEFORE
metadata {
  team: "Payments"
  tier: "critical"
}

// AFTER
metadata {
  team "Payments"
  tier "critical"
}
```

### 4. Update Tests

**Files to Check**:
- `pkg/language/metadata_test.go` - Update test DSL to remove colons
- `pkg/language/ast_metadata_test.go` - Verify tests still pass
- `pkg/language/metadata_parsing_test.go` - Update parsing tests

**Example Test Update**:
```go
// BEFORE
dsl := `
  metadata {
    owner: "team-a"
    tier: "gold"
  }
`

// AFTER
dsl := `
  metadata {
    owner "team-a"
    tier "gold"
  }
`
```

### 5. Update Documentation

**Files to Update**:
- `docs/implementation/DSL_SYNTAX_CONSISTENCY.md` - Update syntax reference
- `docs/implementation/go/DSL_CHANGES_REQUIRED.md` - Update change block examples
- `docs/implementation/TEST_CASES.md` - Update all metadata examples
- `docs/implementation/go/task-1.5-change-commands.md` - Update examples
- `docs/implementation/go/SIMPLIFIED_CHANGE_WORKFLOW.md` - Update examples
- `docs/implementation/go/PARALLEL_CHANGES.md` - Update examples

**Change**: All metadata examples should use syntax without colon:
```sruja
metadata {
  owner "alice@example.com"
  stakeholders ["bob@example.com", "charlie@example.com"]
}
```

## Benefits

1. **Consistency**: Matches `ConstraintEntry` and `ConventionEntry` syntax
2. **Simplicity**: One less character to type
3. **Clarity**: Key-value pairs are still clear without colon
4. **Alignment**: All key-value blocks use same pattern

## Migration Path

Since metadata is not widely used yet (pre-alpha), we can:
1. Update parser and printer
2. Update all examples
3. Update all tests
4. Update all documentation

**No backward compatibility needed** - this is a breaking change but acceptable in pre-alpha.

## Verification Checklist

- [ ] Update `pkg/language/ast.go` - Remove colon from parser tag
- [ ] Update `pkg/language/printer.go` - Remove colon from output
- [ ] Update all example files in `examples/` directory
- [ ] Update all test files in `pkg/language/`
- [ ] Run tests to ensure everything passes
- [ ] Update all documentation files
- [ ] Verify round-trip parsing (parse → print → parse) works

## Testing

After changes, verify:
1. Parser accepts metadata without colon
2. Parser rejects metadata with colon (or make it optional?)
3. Printer outputs without colon
4. Round-trip: parse → print → parse works correctly
5. All existing tests pass

## Optional: Make Colon Optional

Alternatively, we could make colon optional to support both syntaxes:

```go
type MetaEntry struct {
    Key   string `parser:"@Ident ( ':' )?"`
    Value string `parser:"@String"`
}
```

This would allow both:
- `team "Payments"` (no colon)
- `team: "Payments"` (with colon)

**Recommendation**: Remove colon entirely for consistency, unless there's a strong reason to support both.

