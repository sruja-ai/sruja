# Reverse Engineering Plan: JSON → Sruja DSL

## Overview

Enable **round-trip editing** by converting JSON architecture data back to Sruja DSL. This enables:

1. **Diagram-First Workflow**: Users create diagrams visually, then generate code
2. **Visual Studio**: Drag-and-drop architecture editor
3. **Code ↔ Diagram Sync**: Edit in either direction
4. **Migration Tool**: Convert existing diagrams to Sruja DSL

## Architecture

### Current Flow (Code → Diagram)
```
Sruja DSL → Parser → AST → JSON Exporter → JSON → Cytoscape.js → Interactive Diagram
```

### New Flow (Diagram → Code)
```
JSON → DSL Generator → Sruja DSL
```

### Round-Trip Flow
```
Sruja DSL ↔ AST ↔ JSON ↔ Interactive Diagram
```

## Implementation

### Phase 1: JSON → AST Converter

**File**: `pkg/language/json_to_ast.go`

Convert JSON architecture data back to AST structures.

```go
// JSONToAST converts JSON architecture data to AST
func JSONToAST(jsonData []byte) (*Architecture, error) {
    var archJSON ArchitectureJSON
    if err := json.Unmarshal(jsonData, &archJSON); err != nil {
        return nil, err
    }
    
    // Convert JSON to AST
    arch := &Architecture{
        Name: archJSON.Metadata.Name,
    }
    
    // Convert systems
    for _, sysJSON := range archJSON.Architecture.Systems {
        sys := convertSystemJSONToAST(sysJSON)
        arch.Systems = append(arch.Systems, sys)
    }
    
    // Convert persons, relations, etc.
    // ...
    
    return arch, nil
}
```

**Key Functions:**
- `convertSystemJSONToAST(json SystemJSON) *System`
- `convertContainerJSONToAST(json ContainerJSON) *Container`
- `convertComponentJSONToAST(json ComponentJSON) *Component`
- `convertRelationJSONToAST(json RelationJSON) *Relation`
- `convertDomainJSONToAST(json DomainJSON) *DomainBlock`
- `convertScenarioJSONToAST(json ScenarioJSON) *Scenario`
- `convertFlowJSONToAST(json FlowJSON) *Flow`
- `convertRequirementJSONToAST(json RequirementJSON) *Requirement`
- `convertADRJSONToAST(json ADRJSON) *ADR`
- `convertDeploymentJSONToAST(json DeploymentJSON) *DeploymentNode`
- `convertContractJSONToAST(json ContractJSON) *Contract`
- `convertSharedArtifactJSONToAST(json SharedArtifactJSON) *SharedArtifact`
- `convertLibraryJSONToAST(json LibraryJSON) *Library`
- `convertPolicyJSONToAST(json PolicyJSON) *Policy`
- `convertConstraintJSONToAST(json ConstraintJSON) *ConstraintEntry`
- `convertConventionJSONToAST(json ConventionJSON) *ConventionEntry`
- `convertViewJSONToAST(json ViewJSON) *View`
- `convertImportJSONToAST(json ImportJSON) *ImportSpec`

### Phase 2: AST → DSL Printer

**File**: `pkg/language/printer.go` (enhance existing)

Convert AST back to Sruja DSL text.

```go
// Print converts AST to Sruja DSL text
func Print(arch *Architecture) string {
    var sb strings.Builder
    
    // Print architecture header
    if arch.Name != "" {
        fmt.Fprintf(&sb, "architecture %q {\n", arch.Name)
    }
    
    // Print follows
    for _, follow := range arch.Follows {
        fmt.Fprintf(&sb, "  follows %q\n", follow)
    }
    
    // Print metadata
    if len(arch.Metadata) > 0 {
        sb.WriteString("  metadata {\n")
        for _, meta := range arch.Metadata {
            fmt.Fprintf(&sb, "    %s: %q\n", meta.Key, meta.Value)
        }
        sb.WriteString("  }\n")
    }
    
    // Print systems
    for _, sys := range arch.Systems {
        printSystem(&sb, sys, 1)
    }
    
    // Print persons
    for _, person := range arch.Persons {
        printPerson(&sb, person, 1)
    }
    
    // Print relations
    for _, rel := range arch.Relations {
        printRelation(&sb, rel, 1)
    }
    
    // Print domains
    for _, domain := range arch.Domains {
        printDomain(&sb, domain, 1)
    }
    
    // Print scenarios
    for _, scenario := range arch.Scenarios {
        printScenario(&sb, scenario, 1)
    }
    
    // Print requirements
    for _, req := range arch.Requirements {
        printRequirement(&sb, req, 1)
    }
    
    // Print ADRs
    for _, adr := range arch.ADRs {
        printADR(&sb, adr, 1)
    }
    
    // Print deployment
    for _, node := range arch.DeploymentNodes {
        printDeploymentNode(&sb, node, 1)
    }
    
    // Print contracts
    if len(arch.Contracts) > 0 {
        printContracts(&sb, arch.Contracts, 1)
    }
    
    // Print shared artifacts
    for _, artifact := range arch.SharedArtifacts {
        printSharedArtifact(&sb, artifact, 1)
    }
    
    // Print libraries
    for _, lib := range arch.Libraries {
        printLibrary(&sb, lib, 1)
    }
    
    // Print policies
    for _, policy := range arch.Policies {
        printPolicy(&sb, policy, 1)
    }
    
    // Print constraints
    if len(arch.Constraints) > 0 {
        printConstraints(&sb, arch.Constraints, 1)
    }
    
    // Print conventions
    if len(arch.Conventions) > 0 {
        printConventions(&sb, arch.Conventions, 1)
    }
    
    // Print views
    for _, view := range arch.Views {
        printView(&sb, view, 1)
    }
    
    // Print imports
    for _, imp := range arch.Imports {
        printImport(&sb, imp, 1)
    }
    
    if arch.Name != "" {
        sb.WriteString("}\n")
    }
    
    return sb.String()
}

func printSystem(sb *strings.Builder, sys *System, indent int) {
    indentStr := strings.Repeat("  ", indent)
    
    // System header
    if sys.Description != nil {
        fmt.Fprintf(sb, "%ssystem %s %q {\n", indentStr, sys.ID, sys.Label)
    } else {
        fmt.Fprintf(sb, "%ssystem %s {\n", indentStr, sys.ID)
    }
    
    // Print technology
    if sys.Technology != nil {
        fmt.Fprintf(sb, "%s  technology: %q\n", indentStr, *sys.Technology)
    }
    
    // Print description
    if sys.Description != nil {
        fmt.Fprintf(sb, "%s  description: %q\n", indentStr, *sys.Description)
    }
    
    // Print metadata
    if len(sys.Metadata) > 0 {
        fmt.Fprintf(sb, "%s  metadata {\n", indentStr)
        for _, meta := range sys.Metadata {
            fmt.Fprintf(sb, "%s    %s: %q\n", indentStr, meta.Key, meta.Value)
        }
        fmt.Fprintf(sb, "%s  }\n", indentStr)
    }
    
    // Print containers
    for _, cont := range sys.Containers {
        printContainer(sb, cont, indent+1)
    }
    
    // Print components (top-level in system)
    for _, comp := range sys.Components {
        printComponent(sb, comp, indent+1)
    }
    
    // Print data stores
    for _, ds := range sys.DataStores {
        printDataStore(sb, ds, indent+1)
    }
    
    // Print queues
    for _, q := range sys.Queues {
        printQueue(sb, q, indent+1)
    }
    
    // Print relations
    for _, rel := range sys.Relations {
        printRelation(sb, rel, indent+1)
    }
    
    // Print flows
    for _, flow := range sys.Flows {
        printFlow(sb, flow, indent+1)
    }
    
    // Print requirements
    for _, req := range sys.Requirements {
        printRequirement(sb, req, indent+1)
    }
    
    // Print ADRs
    for _, adr := range sys.ADRs {
        printADR(sb, adr, indent+1)
    }
    
    // Print contracts
    if len(sys.Contracts) > 0 {
        printContracts(sb, sys.Contracts, indent+1)
    }
    
    sb.WriteString(indentStr + "}\n")
}

func printContainer(sb *strings.Builder, cont *Container, indent int) {
    // Similar structure...
}

func printComponent(sb *strings.Builder, comp *Component, indent int) {
    // Similar structure...
}

// ... more print functions for each AST type
```

**Key Functions:**
- `Print(arch *Architecture) string` - Main entry point
- `printSystem(sb *strings.Builder, sys *System, indent int)`
- `printContainer(sb *strings.Builder, cont *Container, indent int)`
- `printComponent(sb *strings.Builder, comp *Component, indent int)`
- `printPerson(sb *strings.Builder, person *Person, indent int)`
- `printRelation(sb *strings.Builder, rel *Relation, indent int)`
- `printDomain(sb *strings.Builder, domain *DomainBlock, indent int)`
- `printContext(sb *strings.Builder, ctx *ContextBlock, indent int)`
- `printAggregate(sb *strings.Builder, agg *Aggregate, indent int)`
- `printEntity(sb *strings.Builder, entity *Entity, indent int)`
- `printValueObject(sb *strings.Builder, vo *ValueObject, indent int)`
- `printDomainEvent(sb *strings.Builder, event *DomainEvent, indent int)`
- `printScenario(sb *strings.Builder, scenario *Scenario, indent int)`
- `printFlow(sb *strings.Builder, flow *Flow, indent int)`
- `printRequirement(sb *strings.Builder, req *Requirement, indent int)`
- `printADR(sb *strings.Builder, adr *ADR, indent int)`
- `printDeploymentNode(sb *strings.Builder, node *DeploymentNode, indent int)`
- `printContract(sb *strings.Builder, contract *Contract, indent int)`
- `printSharedArtifact(sb *strings.Builder, artifact *SharedArtifact, indent int)`
- `printLibrary(sb *strings.Builder, lib *Library, indent int)`
- `printPolicy(sb *strings.Builder, policy *Policy, indent int)`
- `printConstraint(sb *strings.Builder, constraint *ConstraintEntry, indent int)`
- `printConvention(sb *strings.Builder, convention *ConventionEntry, indent int)`
- `printView(sb *strings.Builder, view *View, indent int)`
- `printImport(sb *strings.Builder, imp *ImportSpec, indent int)`

### Phase 3: JSON → DSL CLI Command

**File**: `cmd/sruja/json_to_dsl.go`

CLI command to convert JSON to DSL.

```go
// Command: sruja json-to-dsl <input.json> <output.sruja>
var jsonToDSLCmd = &cobra.Command{
    Use:   "json-to-dsl <input.json> <output.sruja>",
    Short: "Convert JSON architecture to Sruja DSL",
    Long:  "Convert JSON architecture data back to Sruja DSL format",
    Args:  cobra.ExactArgs(2),
    RunE: func(cmd *cobra.Command, args []string) error {
        inputJSON := args[0]
        outputDSL := args[1]
        
        // Read JSON
        jsonData, err := os.ReadFile(inputJSON)
        if err != nil {
            return fmt.Errorf("failed to read JSON: %w", err)
        }
        
        // Convert JSON to AST
        arch, err := language.JSONToAST(jsonData)
        if err != nil {
            return fmt.Errorf("failed to parse JSON: %w", err)
        }
        
        // Convert AST to DSL
        dsl := language.Print(arch)
        
        // Write DSL
        if err := os.WriteFile(outputDSL, []byte(dsl), 0644); err != nil {
            return fmt.Errorf("failed to write DSL: %w", err)
        }
        
        fmt.Printf("Converted %s to %s\n", inputJSON, outputDSL)
        return nil
    },
}
```

### Phase 4: Visual Studio Integration

**File**: `pkg/studio/studio.go` (new package)

Web-based visual studio that:
1. Loads JSON architecture
2. Renders with Cytoscape.js
3. Allows drag-and-drop editing
4. Saves changes back to JSON
5. Can export to DSL

```go
// Studio provides visual editing capabilities
type Studio struct {
    architecture *language.Architecture
    jsonData     []byte
}

// LoadJSON loads architecture from JSON
func (s *Studio) LoadJSON(jsonData []byte) error {
    arch, err := language.JSONToAST(jsonData)
    if err != nil {
        return err
    }
    s.architecture = arch
    s.jsonData = jsonData
    return nil
}

// SaveJSON saves architecture to JSON
func (s *Studio) SaveJSON() ([]byte, error) {
    // Convert AST to JSON
    return json.Marshal(s.architecture)
}

// ExportDSL exports architecture to DSL
func (s *Studio) ExportDSL() string {
    return language.Print(s.architecture)
}
```

## JSON Structure Alignment

The JSON structure must match what we export, so the reverse engineering works seamlessly.

**JSON Structure** (from `HTML_JSON_CDN_PLAN.md`):
```json
{
  "metadata": {
    "name": "E-commerce Platform",
    "version": "1.0.0",
    "generated": "2025-01-XX"
  },
  "architecture": {
    "systems": [...],
    "persons": [...],
    "relations": [...],
    "requirements": [...],
    "adrs": [...],
    "domains": [...],
    "scenarios": [...],
    "flows": [...],
    "deployment": [...],
    "contracts": [...],
    "sharedArtifacts": [...],
    "libraries": [...],
    "policies": [...],
    "constraints": [...],
    "conventions": [...],
    "views": [...],
    "imports": [...]
  }
}
```

**AST Structure** (from `pkg/language/ast.go`):
- `Architecture` struct with all fields
- Each element type has corresponding AST struct

**Mapping**: JSON fields must map 1:1 to AST fields for clean conversion.

## Round-Trip Guarantees

### What Preserves Perfectly

1. **Structure** - Systems, containers, components hierarchy
2. **Relations** - All relations between elements
3. **Metadata** - All metadata key-value pairs
4. **Properties** - All properties
5. **Documentation** - Descriptions, labels
6. **DDD Elements** - Domains, contexts, aggregates, entities, value objects, events
7. **Scenarios/Flows** - All steps and sequences
8. **Requirements/ADRs** - All content
9. **Deployment** - Infrastructure topology
10. **Contracts** - All contract definitions
11. **Shared Resources** - SharedArtifacts, Libraries
12. **Governance** - Policies, constraints, conventions
13. **Views** - Custom view definitions
14. **Imports** - Import specifications

### What May Differ (Formatting)

1. **Whitespace** - Indentation, spacing
2. **Order** - Element order may differ
3. **Comments** - Comments are lost (not in AST)
4. **String Quoting** - May use different quote styles

### Round-Trip Test

```go
func TestRoundTrip(t *testing.T) {
    // Original DSL
    originalDSL := `architecture "Test" {
        system API "API Service" {
            container WebApp "Web App" {
                component Handler "Request Handler"
            }
        }
    }`
    
    // Parse to AST
    parser, _ := language.NewParser()
    program, _ := parser.Parse("test.sruja", originalDSL)
    arch := program.Architecture
    
    // Convert AST to JSON
    jsonData, _ := json.Marshal(arch)
    
    // Convert JSON back to AST
    arch2, _ := language.JSONToAST(jsonData)
    
    // Convert AST back to DSL
    generatedDSL := language.Print(arch2)
    
    // Parse generated DSL
    program2, _ := parser.Parse("test2.sruja", generatedDSL)
    arch3 := program2.Architecture
    
    // Compare structures (ignore formatting)
    assert.Equal(t, arch.Name, arch3.Name)
    assert.Equal(t, len(arch.Systems), len(arch3.Systems))
    // ... more comparisons
}
```

## Use Cases

### 1. Visual Studio (Diagram-First)

**Workflow:**
1. User opens visual studio (web app)
2. Creates architecture by dragging/dropping elements
3. Studio saves to JSON
4. User exports to DSL when ready

**Benefits:**
- Non-technical users can create architectures
- Visual feedback during creation
- Can switch to code view anytime

### 2. Migration Tool

**Workflow:**
1. Import existing diagram (PlantUML, Mermaid, etc.)
2. Convert to JSON (via adapter)
3. Convert JSON to Sruja DSL
4. User refines DSL

**Benefits:**
- Migrate from other tools
- Preserve existing architectures

### 3. Code ↔ Diagram Sync

**Workflow:**
1. Edit DSL in code editor
2. Auto-generate JSON
3. Visual studio updates diagram
4. Edit diagram in visual studio
5. Auto-generate DSL
6. Code editor updates DSL

**Benefits:**
- Users can work in preferred mode
- Always in sync
- Best of both worlds

### 4. API/CLI Integration

**Workflow:**
1. External tool generates JSON
2. Convert to Sruja DSL
3. Use Sruja tooling

**Benefits:**
- Integrate with other tools
- Programmatic architecture generation

## Implementation Priority

### Phase 1: Core (High Priority)
1. ✅ JSON → AST converter (basic elements)
2. ✅ AST → DSL printer (basic elements)
3. ✅ CLI command `json-to-dsl`
4. ✅ Round-trip tests

### Phase 2: Complete Coverage (Medium Priority)
1. ✅ All element types (DDD, deployment, etc.)
2. ✅ All metadata and properties
3. ✅ All relations and flows
4. ✅ All governance elements

### Phase 3: Visual Studio (Low Priority)
1. ⏳ Web-based visual editor
2. ⏳ Drag-and-drop interface
3. ⏳ Real-time DSL preview
4. ⏳ Export/import functionality

## Benefits

1. **Flexibility** - Support both code-first and diagram-first workflows
2. **Accessibility** - Non-technical users can create architectures
3. **Migration** - Easy to migrate from other tools
4. **Integration** - Can integrate with external tools
5. **Round-Trip** - Edit in either direction without data loss
6. **Studio Foundation** - Enables visual studio development

## Challenges

1. **Formatting** - DSL formatting may differ (whitespace, order)
2. **Comments** - Comments are lost (not in AST)
3. **Validation** - Need to validate generated DSL
4. **Complexity** - Some structures may be hard to reverse (e.g., complex expressions)

## Solution

1. **Formatting** - Use consistent formatting rules (can be configurable)
2. **Comments** - Store comments separately if needed (future enhancement)
3. **Validation** - Always parse generated DSL to ensure it's valid
4. **Complexity** - Start with simple cases, expand gradually

