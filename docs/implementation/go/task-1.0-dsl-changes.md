# Task 1.0: DSL Parser/Printer Changes

**Priority**: 🔴 Critical (Required for all other tasks)
**Technology**: Go
**Estimated Time**: 3-4 days
**Dependencies**: None (uses existing parser/printer)
**Blocks**: Task 1.1, Task 1.5, and all tasks that use parser/printer

## Overview

Extend the DSL parser and printer to support new syntax for:
1. Change blocks (`change` keyword)
2. Snapshot blocks (`snapshot` keyword)
3. Metadata arrays (for `stakeholders`)
4. Remove colon requirement from metadata (see `PROPOSED_METADATA_SYNTAX_CHANGE.md`)

**Reference**: See [DSL_CHANGES_REQUIRED.md](DSL_CHANGES_REQUIRED.md) for complete syntax specifications.

## Files to Modify

* `pkg/language/ast.go` - Add AST nodes for change and snapshot blocks
* `pkg/language/parser.go` - Add parsing logic
* `pkg/language/printer.go` - Add printing logic
* `pkg/language/ast_test.go` - Add tests
* `pkg/language/parser_test.go` - Add parsing tests
* `pkg/language/printer_test.go` - Add printing tests

## Implementation

### 1. Remove Colon from Metadata (Prerequisite)

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

**File**: `pkg/language/printer.go`

**Change**:
```go
// BEFORE
fmt.Fprintf(sb, "%s%s: %q\n", indent, entry.Key, entry.Value)

// AFTER
fmt.Fprintf(sb, "%s%s %q\n", indent, entry.Key, entry.Value)
```

**Files to Update**:
- `examples/metadata_showcase.sruja`
- `examples/top_level_component.sruja`
- `examples/ddd_advanced.sruja`
- All test files with metadata examples

### 2. Add Metadata Array Support

**File**: `pkg/language/ast.go`

**Add**:
```go
// MetaArrayEntry represents a metadata entry with array value
type MetaArrayEntry struct {
    Key   string   `parser:"@Ident"`
    Value []string `parser:"'[' @String ( ',' @String )* ']'"`
}

func (m *MetaArrayEntry) Location() SourceLocation { return SourceLocation{} }

// Update MetadataBlock to support both
type MetadataBlock struct {
    LBrace  string       `parser:"'metadata' '{'"`
    Entries []MetaValue  `parser:"@@*"`  // Union of MetaEntry and MetaArrayEntry
    RBrace  string       `parser:"'}'"`
}

// MetaValue is a union type
type MetaValue struct {
    Entry      *MetaEntry      `parser:"@@"`
    ArrayEntry *MetaArrayEntry `parser:"| @@"`
}
```

**File**: `pkg/language/printer.go`

**Update** `printMetadata` to handle arrays:
```go
func (p *Printer) printMetadata(sb *strings.Builder, entries []*MetaEntry, arrayEntries []*MetaArrayEntry) {
    // Print string entries
    for _, entry := range entries {
        fmt.Fprintf(sb, "%s%s %q\n", indent, entry.Key, entry.Value)
    }
    // Print array entries
    for _, entry := range arrayEntries {
        fmt.Fprintf(sb, "%s%s [%s]\n", indent, entry.Key, strings.Join(quoteStrings(entry.Value), ", "))
    }
}
```

### 3. Add Change Block AST Node

**File**: `pkg/language/ast.go`

**Add**:
```go
// ChangeBlock represents a change block
type ChangeBlock struct {
    ID          string             `parser:"'change' @String"`
    Version     *string            `parser:"( 'version' @String )?"`
    Requirement *string            `parser:"( 'requirement' @String )?"`
    ADR         *string            `parser:"( 'adr' @String )?"`
    Status      string             `parser:"'status' @String"`
    Metadata    *MetadataBlock     `parser:"( 'metadata' '{' @@ '}' )?"`
    Add         *ArchitectureBlock `parser:"( 'add' '{' @@ '}' )?"`
    Modify      *ArchitectureBlock `parser:"( 'modify' '{' @@ '}' )?"`
    Remove      *ArchitectureBlock `parser:"( 'remove' '{' @@ '}' )?"`
    LBrace      string             `parser:"'{'"`
    RBrace      string             `parser:"'}'"`
}

func (c *ChangeBlock) Location() SourceLocation { return SourceLocation{} }

// ArchitectureBlock contains architecture elements (for add/modify/remove)
type ArchitectureBlock struct {
    Items []ArchitectureItem `parser:"@@*"`
}
```

**Update** `FileItem`:
```go
type FileItem struct {
    // ... existing fields ...
    Change      *ChangeBlock      `parser:"| @@"`
    Snapshot    *SnapshotBlock    `parser:"| @@"`
}
```

### 4. Add Snapshot Block AST Node

**File**: `pkg/language/ast.go`

**Add**:
```go
// SnapshotBlock represents a snapshot block
type SnapshotBlock struct {
    Name        string             `parser:"'snapshot' @String"`
    Version     *string            `parser:"( 'version' @String )?"`
    Description *string            `parser:"( 'description' @String )?"`
    Timestamp   *string            `parser:"( 'timestamp' @String )?"`
    Preview     *bool              `parser:"( 'preview' @('true'|'false') )?"`
    Changes     []string           `parser:"( 'changes' '[' @String ( ',' @String )* ']' )?"`
    Architecture *Architecture     `parser:"( 'architecture' @String '{' @@ '}' )?"`
    LBrace      string             `parser:"'{'"`
    RBrace      string             `parser:"'}'"`
}

func (s *SnapshotBlock) Location() SourceLocation { return SourceLocation{} }
```

### 5. Add Parser Logic

**File**: `pkg/language/parser.go`

**Add**:
```go
// parseChangeBlock parses a change block
func (p *Parser) parseChangeBlock(filename string, input string) (*ChangeBlock, error) {
    // Use participle to parse change block
    // Implementation details...
}

// parseSnapshotBlock parses a snapshot block
func (p *Parser) parseSnapshotBlock(filename string, input string) (*SnapshotBlock, error) {
    // Use participle to parse snapshot block
    // Implementation details...
}
```

### 6. Add Printer Logic

**File**: `pkg/language/printer.go`

**Add**:
```go
// printChangeBlock prints a change block
func (p *Printer) printChangeBlock(sb *strings.Builder, change *ChangeBlock) {
    fmt.Fprintf(sb, "change %q {\n", change.ID)
    p.IndentLevel++
    
    if change.Version != nil {
        fmt.Fprintf(sb, "%sversion %q\n", p.indent(), *change.Version)
    }
    if change.Requirement != nil {
        fmt.Fprintf(sb, "%srequirement %q\n", p.indent(), *change.Requirement)
    }
    if change.ADR != nil {
        fmt.Fprintf(sb, "%sadr %q\n", p.indent(), *change.ADR)
    }
    fmt.Fprintf(sb, "%sstatus %q\n", p.indent(), change.Status)
    
    if change.Metadata != nil {
        p.printMetadata(sb, change.Metadata.Entries, change.Metadata.ArrayEntries)
    }
    
    if change.Add != nil {
        sb.WriteString(p.indent() + "add {\n")
        p.IndentLevel++
        p.printArchitectureBlock(sb, change.Add)
        p.IndentLevel--
        sb.WriteString(p.indent() + "}\n")
    }
    
    // Similar for modify and remove...
    
    p.IndentLevel--
    sb.WriteString(p.indent() + "}\n")
}

// printSnapshotBlock prints a snapshot block
func (p *Printer) printSnapshotBlock(sb *strings.Builder, snapshot *SnapshotBlock) {
    // Implementation...
}
```

## Testing

### Unit Tests

**File**: `pkg/language/parser_test.go`

```go
func TestParseChangeBlock(t *testing.T) {
    dsl := `
change "001-add-api" {
  version "v1.1.0"
  requirement "REQ-001"
  status "approved"
  metadata {
    owner "alice@example.com"
    stakeholders ["bob@example.com", "charlie@example.com"]
  }
  add {
    system API {}
  }
}`
    parser, _ := language.NewParser()
    file, err := parser.Parse("test.sruja", dsl)
    // Assertions...
}

func TestParseMetadataArray(t *testing.T) {
    dsl := `
metadata {
  owner "alice@example.com"
  stakeholders ["bob@example.com", "charlie@example.com"]
}`
    // Test parsing...
}

func TestParseSnapshotBlock(t *testing.T) {
    dsl := `
snapshot "v1.0.0-release" {
  version "v1.0.0"
  description "Production release"
  timestamp "2025-01-15T10:00:00Z"
  architecture "My System" {
    system API {}
  }
}`
    // Test parsing...
}
```

### Round-Trip Tests

**File**: `pkg/language/printer_test.go`

```go
func TestChangeBlockRoundTrip(t *testing.T) {
    // Parse DSL → AST → Print DSL → Parse again
    // Verify output matches input
}

func TestSnapshotBlockRoundTrip(t *testing.T) {
    // Parse DSL → AST → Print DSL → Parse again
    // Verify output matches input
}

func TestMetadataArrayRoundTrip(t *testing.T) {
    // Parse metadata with arrays → Print → Parse again
    // Verify arrays are preserved
}
```

### Integration Tests

**File**: `pkg/language/integration_test.go`

```go
func TestChangeBlockWithArchitecture(t *testing.T) {
    // Test parsing change block with nested architecture elements
    // Test applying changes to base architecture
}

func TestPreviewSnapshotWithInProgressChanges(t *testing.T) {
    // Test creating preview snapshot with in-progress changes
    // Verify validation rules (preview snapshots don't require final states)
}
```

## Validation Rules

### Change Block Validation

**File**: `pkg/engine/change_validation.go` (new file)

```go
// ValidateChangeBlock validates a change block
func ValidateChangeBlock(change *ChangeBlock) error {
    // Validate status is one of: pending, in-progress, approved, deferred
    validStatuses := []string{"pending", "in-progress", "approved", "deferred"}
    if !contains(validStatuses, change.Status) {
        return fmt.Errorf("invalid status: %s", change.Status)
    }
    
    // Validate at least one of add/modify/remove is present
    if change.Add == nil && change.Modify == nil && change.Remove == nil {
        return fmt.Errorf("change block must have at least one of: add, modify, remove")
    }
    
    return nil
}
```

### ADR Status Validation (for apply)

**File**: `pkg/engine/change_apply.go` (new file)

```go
// ValidateChangeForApply validates that a change can be applied
func ValidateChangeForApply(change *ChangeBlock, adrs []*ADR) error {
    // Change must be in final state
    if change.Status != "approved" && change.Status != "deferred" {
        return fmt.Errorf("change %s is not in final state (status: %s)", change.ID, change.Status)
    }
    
    // If change references an ADR, ADR must be in final state
    if change.ADR != nil {
        adr := findADR(adrs, *change.ADR)
        if adr == nil {
            return fmt.Errorf("referenced ADR %s not found", *change.ADR)
        }
        if adr.Status != "decided" && adr.Status != "rejected" {
            return fmt.Errorf("referenced ADR %s is not in final state (status: %s)", *change.ADR, adr.Status)
        }
    }
    
    return nil
}
```

## Acceptance Criteria

- [ ] Parser accepts metadata without colon (matches constraints/conventions)
- [ ] Parser accepts metadata with array values (stakeholders)
- [ ] Parser parses change blocks with all fields
- [ ] Parser parses snapshot blocks with all fields
- [ ] Printer outputs metadata without colon
- [ ] Printer outputs metadata arrays correctly
- [ ] Printer outputs change blocks correctly
- [ ] Printer outputs snapshot blocks correctly
- [ ] Round-trip: parse → print → parse works for all new constructs
- [ ] All existing tests pass (backward compatibility)
- [ ] Validation rules are implemented
- [ ] Error messages are clear and helpful
- [ ] Examples updated to use new syntax
- [ ] Documentation updated

## Manual Testing

1. **Metadata Syntax**:
   ```bash
   # Test parsing metadata without colon
   echo 'metadata { team "Payments" }' | sruja parse
   ```

2. **Change Block**:
   ```bash
   # Test parsing change block
   cat changes/001-add-api.sruja | sruja parse
   ```

3. **Snapshot Block**:
   ```bash
   # Test parsing snapshot block
   cat snapshots/v1.0.0.sruja | sruja parse
   ```

## Dependencies

- None (uses existing parser/printer infrastructure)

## Blocks

- **Task 1.5** (Change Commands) - Cannot implement change commands without parser support

## Notes

- See `PROPOSED_METADATA_SYNTAX_CHANGE.md` for metadata colon removal details
- See `DSL_CHANGES_REQUIRED.md` for complete syntax specifications
- All examples in `examples/` directory need to be updated to remove colons from metadata
- Test files need to be updated to use new syntax

