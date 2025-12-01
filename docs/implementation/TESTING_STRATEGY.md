# Testing Strategy for Sruja

## Overview

Comprehensive testing strategy covering unit tests, integration tests, validation tests, and edge cases.

## Testing Levels

### 1. Unit Tests

**Purpose**: Test individual components in isolation.

**Go Unit Tests**:
- **Parser**: Test DSL → AST conversion
  - Valid syntax
  - Invalid syntax (error handling)
  - Edge cases (empty files, comments, whitespace)
- **Exporter**: Test AST → JSON conversion
  - All element types
  - Metadata preservation
  - File boundaries
- **Converter**: Test JSON → AST conversion
  - Round-trip preservation
  - Missing fields (defaults)
  - Invalid JSON (error handling)
- **Validator**: Test validation rules
  - Unique ID validation
  - Valid reference checking
  - Cycle detection
  - Orphan detection

**TypeScript Unit Tests**:
- **Viewer Core**: Test JSON → Diagram conversion
  - Element rendering
  - Relation rendering
  - Layout algorithms
- **Studio Core**: Test UI components
  - Drag-and-drop
  - Element editing
  - Relation creation

**Test Location**:
```
pkg/language/parser_test.go
pkg/export/json/exporter_test.go
pkg/export/json/converter_test.go
pkg/engine/validator_test.go
learn/assets/js/__tests__/
```

**Coverage Target**: 90%+

### 2. Integration Tests

**Purpose**: Test component interactions and workflows.

**Round-trip Tests**:
- DSL → JSON → DSL (preservation)
- DSL → JSON → AST → DSL (full cycle)
- Multiple files → JSON → Multiple files
- Import statements preserved

**Change Application Tests**:
- Apply single change
- Apply multiple changes (sequential)
- Apply changes with conflicts (error handling)
- Apply changes with invalid states (error handling)
- Preview snapshot generation

**File Operations Tests**:
- Load from file
- Save to file
- Modularization (split/merge)
- Import resolution

**Test Location**:
```
tests/integration/
  - roundtrip_test.go
  - change_application_test.go
  - file_operations_test.go
```

### 3. Validation Tests

**Purpose**: Test all validation rules comprehensively.

**Validation Rules**:
1. **Unique ID Validation**
   - Duplicate system IDs
   - Duplicate container IDs
   - Duplicate component IDs
   - Duplicate across scopes (allowed)

2. **Valid Reference Checking**
   - Valid relations
   - Invalid relations (missing target)
   - Self-references (allowed/blocked)
   - Cross-scope references

3. **Cycle Detection**
   - Simple cycles
   - Complex cycles
   - Self-loops
   - No cycles (valid)

4. **Orphan Detection**
   - Orphaned containers
   - Orphaned components
   - Orphaned relations
   - Valid nesting

**Test Location**:
```
tests/validation/
  - unique_id_test.go
  - valid_reference_test.go
  - cycle_detection_test.go
  - orphan_detection_test.go
```

### 4. Change Management Tests

**Purpose**: Test change workflow comprehensively.

**Change States**:
- Create change (pending)
- Update status (in-progress, approved, deferred)
- Apply approved changes
- Reject in-progress changes
- Validate ADR states

**Conflict Detection**:
- Overlapping elements
- Same qualified names
- Conflicting modifications
- No conflicts (valid)

**Preview Snapshots**:
- Generate with in-progress changes
- Generate with approved changes
- Generate with mixed states
- Export to HTML

**Test Location**:
```
tests/changes/
  - change_states_test.go
  - conflict_detection_test.go
  - preview_snapshots_test.go
```

### 5. Edge Case Tests

**Purpose**: Test boundary conditions and unusual scenarios.

**Large Files**:
- 1000+ elements
- Deep nesting (10+ levels)
- Many relations (1000+)
- Performance benchmarks

**Complex Imports**:
- Circular imports (error)
- Deep import chains
- Missing imports (error)
- Shared elements

**Unusual Syntax**:
- Empty files
- Files with only comments
- Files with only whitespace
- Unicode characters
- Special characters in IDs

**Invalid Input**:
- Malformed JSON
- Missing required fields
- Invalid types
- Null values

**Test Location**:
```
tests/edge_cases/
  - large_files_test.go
  - complex_imports_test.go
  - unusual_syntax_test.go
  - invalid_input_test.go
```

## Test Data

**See**: [Comprehensive Test Cases](TEST_CASES.md) for complete architecture examples with base + changes.

### Sample Architectures

**Simple Architecture** (`testdata/simple/`):
```sruja
architecture "Simple App" {
  system WebApp {}
  system Database {}
  relation WebApp -> Database "Reads/Writes"
}
```

**Medium Architecture** (`testdata/medium/`):
- 5-10 systems
- 20-30 containers
- 50-100 components
- 100+ relations
- **Includes**: Base architecture + sequential changes (add analytics, add payment, enhance API)

**Complex Architecture** (`testdata/complex/`):
- 20+ systems
- 100+ containers
- 500+ components
- 1000+ relations
- Multiple files (split)
- Shared elements
- **Includes**: Multi-system architecture + integration layer changes

**Change Test Data** (`testdata/changes/`):
- Single change (add, modify, remove)
- Multiple changes (sequential)
- Conflicting changes (overlapping elements, conflicting modifications)
- Changes with ADRs (approved, pending, in-progress)
- Preview snapshots (with in-progress changes)
- Change states (pending, in-progress, approved, deferred)
- Edge cases (empty changes, large changes, metadata-only)

### Test Fixtures

**Location**: `testdata/`

**Structure**:
```
testdata/
  ├── simple/
  │   ├── architecture.sruja
  │   └── expected.json
  ├── medium/
  │   ├── architecture.sruja
  │   └── expected.json
  ├── complex/
  │   ├── main.sruja
  │   ├── partials/
  │   └── expected.json
  ├── changes/
  │   ├── base.sruja
  │   ├── change-001.sruja
  │   └── expected-current.sruja
  └── invalid/
      ├── duplicate-id.sruja
      ├── invalid-reference.sruja
      └── cycle.sruja
```

## Test Execution

### Running Tests

**Go Tests**:
```bash
# All tests
go test ./...

# Specific package
go test ./pkg/language/...

# With coverage
go test -cover ./...

# Coverage report
go test -coverprofile=coverage.out ./...
go tool cover -html=coverage.out
```

**TypeScript Tests**:
```bash
cd learn
npm test

# With coverage
npm test -- --coverage
```

### CI/CD Integration

Tests run automatically:
- On every PR (validation workflow)
- On main branch (main validation workflow)
- Before release (release workflow)

## Performance Testing

**Benchmarks**:
- Parser performance (large files)
- Exporter performance (complex architectures)
- Viewer rendering (many elements)
- Change application (many changes)

**Targets**:
- Parse 1000-element file: < 1s
- Export 1000-element file: < 1s
- Render 1000-element diagram: < 2s
- Apply 100 changes: < 5s

**Location**: `tests/benchmarks/`

## Regression Testing

**Purpose**: Ensure changes don't break existing functionality.

**Approach**:
- Test suite runs on every commit
- Compare outputs (JSON, diagrams)
- Track performance metrics
- Visual regression tests (diagrams)

## Test Maintenance

**Best Practices**:
- Keep tests simple and focused
- Use descriptive test names
- Test one thing per test
- Use test fixtures (don't duplicate)
- Update tests when behavior changes
- Remove obsolete tests

## Acceptance Criteria

✅ **Unit Tests**: 90%+ coverage  
✅ **Integration Tests**: All workflows covered  
✅ **Validation Tests**: All rules tested  
✅ **Edge Cases**: Boundary conditions tested  
✅ **Performance**: Meets targets  
✅ **CI/CD**: Tests run automatically  

## Test Data Examples

### Example 1: Simple Round-trip

**Input** (`testdata/simple/input.sruja`):
```sruja
architecture "Test" {
  system API {}
}
```

**Expected JSON** (`testdata/simple/expected.json`):
```json
{
  "metadata": { "version": "1.0" },
  "systems": [
    { "id": "API", "name": "API" }
  ]
}
```

**Test**: Verify DSL → JSON → DSL preserves structure.

### Example 2: Change Application

**Base** (`testdata/changes/base.sruja`):
```sruja
architecture "App" {
  system WebApp {}
}
```

**Change** (`testdata/changes/change-001.sruja`):
```sruja
change "001-add-api" {
  status "approved"
  add {
    system API {}
    relation WebApp -> API "Calls"
  }
}
```

**Expected** (`testdata/changes/expected-current.sruja`):
```sruja
architecture "App" {
  system WebApp {}
  system API {}
  relation WebApp -> API "Calls"
}
```

**Test**: Verify change application produces expected result.

### 6. Manual Testing

**Purpose**: Human verification of functionality, usability, and edge cases that are difficult to automate.

**Manual Test Scenarios**:

**CLI Commands**:
- [ ] Test all CLI commands with various inputs
- [ ] Verify error messages are clear and helpful
- [ ] Test command help text and examples
- [ ] Verify file operations (read/write permissions)
- [ ] Test with real-world architecture files

**Studio (Visual Editor)**:
- [ ] Create architecture visually (drag-and-drop)
- [ ] Edit elements (rename, modify properties)
- [ ] Create relations between elements
- [ ] Export to DSL and verify output
- [ ] Export to HTML and verify in browser
- [ ] Export to PNG/SVG and verify quality
- [ ] Test undo/redo functionality
- [ ] Test keyboard shortcuts
- [ ] Test with large architectures (performance)
- [ ] Test on different browsers (Chrome, Firefox, Safari, Edge)

**Change Management**:
- [ ] Create change via CLI
- [ ] Create change via Studio
- [ ] Apply change and verify result
- [ ] Test conflict detection with real scenarios
- [ ] Test preview snapshots with in-progress changes
- [ ] Verify ADR validation works correctly

**HTML Export**:
- [ ] Open HTML in browser
- [ ] Test interactivity (zoom, pan, click)
- [ ] Test on mobile devices
- [ ] Test with different screen sizes
- [ ] Verify all view types render correctly

**Test Checklist**: `docs/testing/MANUAL_TEST_CHECKLIST.md`

### 7. MCP-Based Testing (Model Context Protocol)

**Purpose**: Use AI/LLM-based testing via Model Context Protocol for complex scenarios, edge cases, and validation that benefits from semantic understanding.

**MCP Testing Scenarios**:

**Semantic Validation**:
- [ ] Verify architecture semantics make sense
- [ ] Check for logical inconsistencies
- [ ] Validate naming conventions
- [ ] Verify architectural patterns are followed

**Complex Scenarios**:
- [ ] Test with domain-specific architectures (e-commerce, banking, etc.)
- [ ] Verify DDD concepts are correctly modeled
- [ ] Check that relations make logical sense
- [ ] Validate that changes don't break architectural principles

**Documentation Validation**:
- [ ] Verify descriptions are clear and complete
- [ ] Check that ADRs are properly linked
- [ ] Validate that requirements are traceable
- [ ] Ensure metadata is meaningful

**Round-trip Validation**:
- [ ] Use MCP to verify DSL → JSON → DSL preserves meaning
- [ ] Check that semantic information is preserved
- [ ] Validate that intent is maintained through transformations

**MCP Test Framework**:
- Use MCP servers for testing
- Create test scenarios that require semantic understanding
- Compare outputs for semantic equivalence (not just syntax)
- Generate test cases based on architectural patterns

**Test Location**: `tests/mcp/`

**MCP Test Examples**:
```go
// tests/mcp/semantic_validation_test.go
func TestArchitectureSemanticValidation(t *testing.T) {
    // Use MCP to validate architecture makes sense
    // Check for logical inconsistencies
    // Verify patterns are followed
}
```

## Summary

**Testing Strategy**:
- ✅ Unit tests for all components
- ✅ Integration tests for workflows
- ✅ Validation tests for all rules
- ✅ Edge case tests for robustness
- ✅ Performance benchmarks
- ✅ CI/CD integration
- ✅ **Manual testing for usability and real-world scenarios**
- ✅ **MCP-based testing for semantic validation and complex scenarios**

**Test Data**:
- ✅ Simple, medium, complex architectures
- ✅ Valid and invalid inputs
- ✅ Change scenarios
- ✅ Edge cases

**Coverage**: 90%+ for critical paths

**Manual Testing**: All user-facing features tested manually

**MCP Testing**: Complex scenarios and semantic validation tested via MCP

