# Task 1.1: JSON Exporter (AST → JSON)

**Priority**: 🔴 Critical (Blocks everything)
**Technology**: Go
**Estimated Time**: 2-3 days
**Dependencies**: None (uses existing AST)

## Files to Create

* `pkg/export/json/json.go` - Main exporter
* `pkg/export/json/json_types.go` - JSON type definitions
* `pkg/export/json/json_test.go` - Comprehensive tests

## Implementation

```go
// pkg/export/json/json.go
type Exporter struct{}

func (e *Exporter) Export(arch *language.Architecture) ([]byte, error) {
    // Convert AST to JSON structure
    jsonData := ArchitectureJSON{
        Metadata: MetadataJSON{
            Name:      arch.Name,
            Version:   "1.0.0",
            Generated: time.Now().Format(time.RFC3339),
        },
        Architecture: convertArchitectureToJSON(arch),
        Navigation:   buildNavigation(arch),
    }
    return json.MarshalIndent(jsonData, "", "  ")
}
```

## JSON Structure (Self-Contained)

```json
{
  "metadata": {
    "name": "Architecture Name",
    "version": "1.0.0",
    "generated": "2025-01-XXT00:00:00Z",
    "sourceFiles": [
      {
        "path": "main.sruja",
        "elements": ["System1", "Person1"]
      },
      {
        "path": "shared.sruja",
        "elements": ["Person1", "SharedArtifact1"]
      }
    ]
  },
  "architecture": {
    "systems": [
      {
        "id": "System1",
        "label": "Main System",
        "metadata": {
          "sourceFile": "main.sruja"
        }
      }
    ],
    "persons": [
      {
        "id": "Person1",
        "label": "User",
        "metadata": {
          "sourceFile": "shared.sruja",
          "imported": true,
          "importedFrom": "shared.sruja"
        }
      }
    ],
    "relations": [...],
    "domains": [...],
    "scenarios": [...],
    "flows": [...],
    "requirements": [...],
    "adrs": [...],
    "deployment": [...],
    "contracts": [...],
    "sharedArtifacts": [...],
    "libraries": [...],
    "policies": [...],
    "constraints": [...],
    "conventions": [...],
    "views": [...]
  },
  "navigation": {
    "levels": ["level1", "level2", "level3"],
    "scenarios": [...],
    "flows": [...],
    "domains": [...]
  }
}
```

**Key Design Decisions**:

1. **No imports in JSON**: JSON is self-contained - all elements are flattened
2. **File information in metadata**: Each element has `metadata.sourceFile` indicating its origin
3. **Source files tracking**: `metadata.sourceFiles` maps files to their elements for reconstruction
4. **Flattened structure**: All imported elements are included directly in the JSON arrays

## Implementation Notes

When exporting DSL → JSON:

1. **Identify root architecture**: Determine which architecture is the root (from main file)
2. **Resolve all imports**: Load all imported files and their elements
   - If imported file has architecture block: Import elements from that architecture
   - If imported file has no architecture block: Treat as shared elements
3. **Build qualified names from scope**: DSL uses simple names, parser builds qualified
   - Systems: `SystemName` (unchanged)
   - Containers: `SystemName.ContainerName` (built from scope)
   - Components: `SystemName.ContainerName.ComponentName` (built from scope)
   - Nested elements inherit parent qualification
4. **Flatten elements**: Include all elements (from main file + imported files) in JSON
5. **Flatten relations**: All relations (whether nested in containers/systems or top-level) go into top-level `relations` array. Preserve scope information in `metadata.scope`.
6. **Add file metadata**: Each element and relation gets `metadata.sourceFile` indicating origin
7. **Add architecture metadata**: Each element gets `metadata.architecture` (root architecture name or null for shared)
8. **Mark shared elements**: Set `metadata.shared: true` for elements from files without architecture blocks
9. **Track source files**: Build `metadata.sourceFiles` mapping (including architecture info)
10. **Mark imported elements**: Set `metadata.imported: true` for elements from imports
11. **Preserve import paths**: Store original import path in `metadata.importedFrom`
12. **Update relation references**: All relation `from`/`to` use qualified names (built from scope). See [Relation Flattening](relation-flattening.md) for details.

### Handling Multiple Architectures

**Rule**: One architecture block per file (enforced at parser level)

If parser validation passes, each file generates exactly one JSON:
- File with one architecture block → one JSON file
- File with no architecture block (shared-only) → no JSON (elements are imported)
- Multiple architectures = multiple files (workspace concept)

See [Architecture Model](architecture-model.md) and [Architecture Semantics](architecture-semantics.md) for details.

### Handling Shared Elements

If a file has no architecture block (shared elements only), elements are imported into root architecture with `metadata.shared: true`. See [Architecture Model](architecture-model.md) for details.

### Qualified Name Handling

Since DSL already uses qualified names, we can use them directly:

```go
// Qualified names come from DSL - use as-is
func exportElementID(element Element) string {
    // If element has QualifiedID, use it (already qualified from DSL)
    if element.QualifiedID != "" {
        return element.QualifiedID
    }
    
    // Fallback: generate qualified name (shouldn't happen if DSL enforces it)
    return generateQualifiedName(element)
}

// Validate qualified names match expected patterns
func validateQualifiedName(id string, expectedPattern string) error {
    // Validate container: SystemName.ContainerName
    // Validate component: SystemName.ContainerName.ComponentName
    // etc.
}
```

**Note**: Since DSL enforces qualified names, the exporter should use them directly from AST rather than generating them.

See [Qualified Names Specification](qualified-names.md) for complete naming rules.

## Test Coverage

* ✅ All element types (systems, containers, components, etc.)
* ✅ **File metadata preservation** (sourceFile annotations)
* ✅ **Flattened import resolution** (all imported elements included)
* ✅ **Qualified names** (all element IDs use qualified format)
* ✅ **Qualified relation references** (from/to use qualified names)
* ✅ **Flattened relations** (relations from nested scopes are flattened into top-level array)
* ✅ All DDD elements (domains, contexts, aggregates, etc.)
* ✅ All relations and flows (from all scopes)
* ✅ All metadata and properties
* ✅ Round-trip tests (DSL → JSON → DSL) - **file boundaries must be reconstructable**
* ✅ Round-trip tests (DSL → JSON → DSL) - **qualified names must be preserved**
* ✅ Self-contained JSON (no external dependencies)
* ✅ Source files tracking in metadata

## Acceptance Criteria

* [ ] All AST types can be exported to JSON
* [ ] JSON is self-contained (no imports, all elements flattened)
* [ ] **All element IDs use qualified names** (e.g., `SystemName.ContainerName.ComponentName`)
* [ ] **All relations are flattened** (from nested scopes into top-level relations array)
* [ ] **All relation references use qualified names** (from/to fields)
* [ ] **Root architecture identified** and stored in `metadata.rootArchitecture`
* [ ] **Architecture ownership tracked** in `metadata.architecture` for each element
* [ ] **Shared elements marked** with `metadata.shared: true`
* [ ] **Multiple architectures handled** (separate JSON files generated)
* [ ] File source information preserved in element and relation metadata
* [ ] Source files tracked in metadata.sourceFiles
* [ ] JSON structure matches specification
* [ ] 90%+ test coverage
* [ ] Round-trip tests pass (file boundaries can be reconstructed)
* [ ] Round-trip tests pass (qualified names preserved)
* [ ] Error handling follows [Error Reporting Strategy](../ERROR_REPORTING_STRATEGY.md)
* [ ] All errors include location, code, message, and suggestions
* [ ] **Manual testing**: Test with real-world architecture files
* [ ] **MCP-based testing**: Verify semantic preservation in round-trip
