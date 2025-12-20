# Task 1.2: JSON to AST Converter (JSON → AST)

**Task Status**
- Status: ✅ **COMPLETE**
- Owner: Unassigned
- Target Date: -
- Dependencies: Task 1.1
- Last Updated: 2025-12-01

**Priority**: 🔴 Critical (Blocks reverse engineering)
**Technology**: Go
**Estimated Time**: 2-3 days
**Dependencies**: Task 1.1 (needs JSON structure)

## Output Format Options

JSON → DSL conversion provides two options (user chooses):

1. **Single File**: All content in one file with clear sections
2. **Multiple Files**: Concept-based files with standard names:
   - `architecture.sruja` - Architecture structure (elements, relations, flows)
   - `requirements.sruja` - Requirements
   - `decisions.sruja` - ADRs
   - `stories.sruja` - User stories
   - `scenarios.sruja` - Scenarios

**No other customization** - these are the only two options.

## Files Created

* `pkg/import/json/json.go` - Main converter (ToArchitecture, ToDSL)
* `pkg/import/json/json_test.go` - Comprehensive tests (15 tests)
* `cmd/sruja/import.go` - CLI command implementation

## Implementation

```go
// pkg/language/json_to_ast.go

// OutputFormat specifies how DSL files should be organized
type OutputFormat string

const (
    OutputFormatSingleFile   OutputFormat = "single"   // All in one file
    OutputFormatMultipleFiles OutputFormat = "multiple" // Concept-based files
)

func JSONToAST(jsonData []byte, outputFormat OutputFormat) (*Architecture, []FileOutput, error) {
    var archJSON ArchitectureJSON
    if err := json.Unmarshal(jsonData, &archJSON); err != nil {
        return nil, nil, err
    }
    
    arch := &Architecture{
        Name: archJSON.Metadata.Name,
    }
    
    // Convert all elements from JSON to AST
    arch.Systems = convertSystemsJSONToAST(archJSON.Architecture.Systems)
    arch.Persons = convertPersonsJSONToAST(archJSON.Architecture.Persons)
    arch.Relations = convertRelationsJSONToAST(archJSON.Architecture.Relations)
    arch.Flows = convertFlowsJSONToAST(archJSON.Architecture.Flows) // Flow is architecture construct, should be implemented
    // ... all other elements
    
    // Convert requirements/stories/ADRs/scenarios
    requirements := convertRequirementsJSONToAST(archJSON.Requirements)
    userStories := convertUserStoriesJSONToAST(archJSON.UserStories)
    adrs := convertADRsJSONToAST(archJSON.ADRs)
    scenarios := convertScenariosJSONToAST(archJSON.Scenarios)
    
    // Generate output files based on format option
    files := generateDSLFiles(arch, requirements, userStories, adrs, scenarios, outputFormat)
    
    return arch, files, nil
}

// Generate DSL files based on output format
func generateDSLFiles(
    arch *Architecture,
    requirements []*Requirement,
    userStories []*UserStory,
    adrs []*ADR,
    scenarios []*Scenario,
    format OutputFormat,
) []FileOutput {
    var files []FileOutput
    
    switch format {
    case OutputFormatSingleFile:
        // Single file with all sections
        content := generateSingleFile(arch, requirements, userStories, adrs, scenarios)
        files = append(files, FileOutput{
            Path:    arch.Name + ".sruja",
            Content: content,
        })
        
    case OutputFormatMultipleFiles:
        // Concept-based files with standard names
        files = append(files, FileOutput{
            Path:    "architecture.sruja",
            Content: generateArchitectureFile(arch),
        })
        files = append(files, FileOutput{
            Path:    "requirements.sruja",
            Content: generateRequirementsFile(requirements),
        })
        files = append(files, FileOutput{
            Path:    "stories.sruja",
            Content: generateStoriesFile(userStories),
        })
        files = append(files, FileOutput{
            Path:    "decisions.sruja",
            Content: generateDecisionsFile(adrs),
        })
        files = append(files, FileOutput{
            Path:    "scenarios.sruja",
            Content: generateScenariosFile(scenarios),
        })
    }
    
    return files
}

type FileOutput struct {
    Path    string
    Content string
}
```

// Reconstruct file boundaries from metadata
func reconstructFileBoundaries(archJSON ArchitectureJSON) map[string][]string {
    fileElements := make(map[string][]string)
    rootArch := archJSON.Metadata.RootArchitecture
    
    // Use metadata.sourceFiles if available
    if archJSON.Metadata.SourceFiles != nil {
        for _, sf := range archJSON.Metadata.SourceFiles {
            fileElements[sf.Path] = sf.Elements
            
            // Determine if file should have architecture block
            if sf.Architecture == rootArch {
                // Main architecture file
            } else if sf.Architecture == nil {
                // Shared elements file (no architecture block)
            } else {
                // Imported architecture (if we support that)
            }
        }
    } else {
        // Fallback: group elements by metadata.sourceFile
        // Separate root elements from shared elements
        // ... implementation
    }
    
    return fileElements
}

// Separate root architecture elements from shared elements
func separateRootAndShared(elements []ElementJSON, rootArch string) (root []ElementJSON, shared []ElementJSON) {
    for _, elem := range elements {
        if elem.Metadata.Architecture == rootArch {
            root = append(root, elem)
        } else if elem.Metadata.Shared {
            shared = append(shared, elem)
        }
    }
    return root, shared
}

// Reconstruct relation nesting from metadata.scope
func reconstructRelationNesting(relationsJSON []RelationJSON) {
    for _, rel := range relationsJSON {
        scope := rel.Metadata.Scope // "container", "system", "architecture"
        
        switch scope {
        case "container":
            // Place relation inside appropriate container block
        case "system":
            // Place relation inside appropriate system block
        case "architecture":
            // Place relation at top level
        }
    }
}

// Parse qualified names - they come directly from JSON
func parseQualifiedName(qualifiedID string) []string {
    return strings.Split(qualifiedID, ".")
}

// Extract simple ID from qualified name (last part) - for display purposes only
func extractSimpleID(qualifiedID string) string {
    parts := parseQualifiedName(qualifiedID)
    return parts[len(parts)-1]
}

// Reconstruct nested structure from qualified names
func reconstructHierarchy(elements []ElementJSON) {
    // Group by hierarchy level
    systems := make(map[string]*System)
    containers := make(map[string]*Container)
    
    for _, elem := range elements {
        parts := parseQualifiedName(elem.ID)
        
        switch {
        case len(parts) == 1:
            // System or top-level element
            // Store qualified ID directly (DSL uses qualified names)
            
        case len(parts) == 2:
            // Container or nested element in system
            sysName := parts[0]
            elemName := parts[1]
            // Store qualified ID: sysName.elemName
            
        case len(parts) == 3:
            // Component or nested element in container
            sysName := parts[0]
            contName := parts[1]
            elemName := parts[2]
            // Store qualified ID: sysName.contName.elemName
        }
    }
}

// When generating DSL, use qualified names as-is
func generateDSL(arch *Architecture) string {
    // Use qualified IDs directly in DSL output
    // No need to convert to simple names
}

// Determine which files should be imports
func determineImports(fileElements map[string][]string) []*ImportSpec {
    imports := []*ImportSpec{}
    
    // Files with elements marked as imported should become imports
    // Main file (typically "main.sruja" or first file) is not an import
    // ... implementation
    
    return imports
}
```

## Test Coverage

* ✅ All JSON types can be converted to AST
* ✅ **File boundaries reconstructed** from metadata
* ✅ **Import statements reconstructed** from file metadata
* ✅ All element types preserved
* ✅ All metadata preserved (including sourceFile)
* ✅ Round-trip tests (JSON → AST → JSON)
* ✅ Round-trip tests (DSL → JSON → DSL) - **file organization must match**
* ✅ Edge cases (missing fields, null values, missing metadata)
* ✅ Self-contained JSON handling (no imports array)
* ✅ Source files tracking reconstruction

## Implementation Details

### Single File Output

Generates one file with all sections:
```sruja
// ecommerce-platform.sruja
architecture "E-commerce Platform" {
  system ShopSystem {}
  // ... relations, flows
}

// Requirements section
requirement "REQ-123" {...}

// Decisions section
adr "ADR-001" {...}

// User Stories section
userStory "US-456" {...}

// Scenarios section
scenario "High Traffic" {...}
```

### Multiple Files Output

Generates concept-based files with standard names:
- `architecture.sruja` - All architecture structure
- `requirements.sruja` - All requirements
- `decisions.sruja` - All ADRs
- `stories.sruja` - All user stories
- `scenarios.sruja` - All scenarios

**Important**: 
- No custom file names allowed
- Architecture stays in ONE file (no splitting)
- Standard file names enforced

## Acceptance Criteria

* [ ] All JSON types can be converted to AST
* [ ] **Output format option** works (single file or multiple files)
* [ ] **Single file output** generates one file with all sections
* [ ] **Multiple files output** generates standard concept-based files
* [ ] **No custom file names** - only standard names allowed
* [ ] **Qualified names are preserved** from JSON (DSL uses qualified names)
* [ ] **Qualified names are validated** against expected patterns
* [ ] **Relations and flows** are in architecture block only
* [ ] **Key-value tags** are converted correctly
* [ ] **Root architecture is identified** from metadata.rootArchitecture
* [ ] **Shared services** use naming convention (`shared.ServiceName`)
* [ ] Elements maintain their sourceFile metadata
* [ ] Round-trip tests pass (JSON → AST → JSON)
* [ ] Round-trip tests pass (DSL → JSON → DSL) - file organization matches
* [ ] Round-trip tests pass (DSL → JSON → DSL) - qualified names preserved
* [ ] Round-trip tests pass (DSL → JSON → DSL) - tags preserved
* [ ] 90%+ test coverage
* [ ] Handles missing/null fields gracefully
* [ ] Handles missing metadata gracefully

See [Qualified Names Specification](qualified-names.md) for naming rules.
