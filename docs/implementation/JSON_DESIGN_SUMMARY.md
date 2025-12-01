# JSON Design Summary: Simplified Self-Contained Model

## Key Understanding

✅ **JSON is self-contained** - no imports, all elements flattened  
✅ **Single root architecture** - each JSON represents one root architecture  
✅ **One architecture per file** - enforced at parser level (simpler, clearer)  
✅ **Simplified file organization** - concept-based files only (architecture.sruja, requirements.sruja, decisions.sruja, stories.sruja, scenarios.sruja)  
✅ **No imports** - shared services use naming convention (`shared.ServiceName`)  
✅ **Multiple architectures = multiple files** - product portfolio is a workspace  
✅ **Team ownership** - teams own systems/containers via metadata  
✅ **No file name tracking** - file organization determined by JSON structure  
✅ **Qualified names enforced** - DSL uses scoping rules (simple names in scope, qualified when outside). JSON uses fully qualified format (e.g., `SystemName.ContainerName.ComponentName`)  
✅ **Key-value tags** - Requirements/stories/ADRs/scenarios use tags `[system "name", container "System.Container"]` to link elements  
✅ **Relations & flows** - Always inside architecture block only  
✅ **Round-trip safe** - file boundaries and architecture ownership can be reconstructed from metadata

See [Simplified Architecture Model](go/simplified-architecture-model.md) for complete details.

## Design Decisions

### 1. No Imports in JSON Structure

**Why**: JSON should be portable and standalone
- No import resolution required
- Can be shared/moved without file system dependencies
- Simpler for TypeScript rendering

**Note**: Shared services use naming convention (`shared.ServiceName`) - no explicit imports needed. Parser automatically resolves from `shared/` directory.

### 2. Simplified File Organization

**Two Options**:
- **Single file**: Everything in one file with clear sections
- **Concept-based files**: Standard names - `architecture.sruja`, `requirements.sruja`, `decisions.sruja`, `stories.sruja`, `scenarios.sruja`

**Rules**:
- Architecture stays in ONE file (no splitting)
- Concept-based files use standard names only
- No custom file names

### 3. Team Ownership via Metadata

**How**: Teams own elements via metadata (simple, no duplication):
```json
{
  "id": "ShopSystem",
  "metadata": {
    "team": "Platform Team"
  }
}
```

**No root-level teams array** - team ownership is tracked directly on each element.

### 4. Key-Value Tags

**New Tag Syntax**: Requirements/stories/ADRs/scenarios use key-value tags:

**DSL Syntax**:
```sruja
requirement "REQ-123" "Analytics dashboard" {
  tags [
    system "ShopSystem"
    container "ShopSystem.AnalyticsAPI"
  ]
}

userStory "US-456" "View metrics" {
  tags [
    container "ShopSystem.AnalyticsAPI"
    flow "OrderFlow"
  ]
}
```

**JSON Format**:
```json
{
  "requirements": [
    {
      "id": "REQ-123",
      "tags": [
        {"type": "system", "value": "ShopSystem"},
        {"type": "container", "value": "ShopSystem.AnalyticsAPI"}
      ]
    }
  ]
}
```

**Key Points**:
- Tags use key-value pairs: `[system "name1", container "System.Container"]`
- Containers/components must be fully qualified in tags
- Multiple values allowed (multiple systems/containers per requirement)
- Flows and user stories link bidirectionally via tags

### 5. Relations and Flows

**Always Inside Architecture Block**:
- Relations: Simple connections between elements
- Flows: Sequences showing data/value flow through system
- Both must be defined inside architecture block

### 6. Qualified Names

**Why**: Unambiguous references, no naming conflicts, self-documenting  
**Where**: 
- **DSL**: Uses scoping rules (simple names in scope, qualified when outside)
- **JSON**: Always uses fully qualified format  
**How**: 
- DSL declarations: Simple names (scope builds qualified internally)
- DSL references: Qualified when outside scope
- JSON: All IDs use qualified format:
- Systems: `SystemName`
- Containers: `SystemName.ContainerName`
- Components: `SystemName.ContainerName.ComponentName`

Example: `ShopSystem.WebApp.ShoppingCart`

**DSL Syntax** (Scoping Rules):
```sruja
system ShopSystem {
  container WebApp {
    component ShoppingCart {}
    
    // Relations within container scope - simple names
    ShoppingCart -> WebApp "Uses"
  }
  
  container API {
    component OrderService {}
  }
  
  // Relations within system scope - simple names
  WebApp -> API "Calls"
  
  // Relations can be defined within their parent scope
  API -> Database "Queries"
}

// References from outside scope need qualification
Customer -> ShopSystem.WebApp "Uses"  // Qualified - outside system scope
ShopSystem.API -> PaymentSystem.Gateway "Calls"  // Qualified - cross-system
```

### 7. Root Architecture

**How**: Each JSON represents one root architecture:
```json
{
  "metadata": {
    "name": "E-commerce Platform",
    "rootArchitecture": "E-commerce Platform"
  }
}
```

### 8. Architecture Ownership

**How**: Each element tracks which architecture owns it:
```json
{
  "id": "ShopSystem",
  "metadata": {
    "architecture": "E-commerce Platform",  // Root architecture
    "imported": false
  }
},
{
  "id": "shared.AuthService",
  "metadata": {
    "architecture": null,  // Shared service (referenced via naming convention)
    "shared": true
  }
}
```

### 9. File Organization (No Tracking Needed)

**How**: File organization is determined by JSON structure - no file name tracking needed:
- `architecture` block → `architecture.sruja` (or single file)
- `requirements` array → `requirements.sruja`
- `userStories` array → `stories.sruja`
- `adrs` array → `decisions.sruja`
- `scenarios` array → `scenarios.sruja`

**JSON → DSL conversion**: User chooses single file OR multiple files (standard names only).

## Conversion Flow

### DSL → JSON

1. **Identify root architecture** from architecture file
2. Load architecture file + concept-based files (requirements.sruja, decisions.sruja, stories.sruja, scenarios.sruja)
3. Resolve shared services from `shared/` directory (using naming convention: `shared.ServiceName`)
4. **Build qualified names from scope** (DSL uses simple names, scope builds qualified)
   - DSL declarations use simple names: `WebApp`, `ShoppingCart`
   - Parser builds qualified names: `ShopSystem.WebApp`, `ShopSystem.WebApp.ShoppingCart`
5. Flatten all elements into JSON arrays (using qualified IDs from AST)
6. **Flatten all relations and flows** (relations/flows are always inside architecture block)
7. **Export requirements/stories/ADRs/scenarios** with key-value tags
8. **Relation references** use qualified names (built from scope or explicit in DSL)
9. Export self-contained JSON with `metadata.rootArchitecture`

**No file name tracking**: File organization is reconstructible from JSON structure.

**Multiple architectures**: Modeled as multiple files (workspace concept). Each architecture file generates its own JSON.

### JSON → DSL

**Output Format Options** (user chooses):
1. **Single File**: All content in one file with clear sections
2. **Multiple Files**: Concept-based files with standard names

**No other customization** - these are the only two options.

**Conversion Steps**:

1. Use qualified names from JSON (already qualified)
2. **Identify root architecture** from `metadata.rootArchitecture`
3. Convert all JSON elements to AST
4. Convert requirements/stories/ADRs/scenarios from JSON
5. **Generate DSL files** based on output format option:

   **Option 1: Single File**
   - Generate one file: `{architecture-name}.sruja`
   - Contains all sections: architecture, requirements, decisions, stories, scenarios

   **Option 2: Multiple Files**
   - Generate `architecture.sruja` (elements, relations, flows)
   - Generate `requirements.sruja` (requirements)
   - Generate `decisions.sruja` (ADRs)
   - Generate `stories.sruja` (user stories)
   - Generate `scenarios.sruja` (scenarios)

6. Reconstruct nested structure from qualified names
7. Extract simple names from qualified names for DSL declarations (scope-based syntax)
8. **Generate tags** in key-value format: `tags [system "name", container "System.Container"]`
9. Handle shared services using naming convention (`shared.ServiceName`)
10. Generate `.sruja` files with standard names only:
    - Architecture elements → `architecture.sruja` (or single file)
    - Requirements → `requirements.sruja`
    - User stories → `stories.sruja`
    - ADRs → `decisions.sruja`
    - Scenarios → `scenarios.sruja`

**No file name tracking needed**: File organization determined by JSON structure.

## Studio Visualization

Studio can visualize file organization using metadata:

- **File construct**: Visual grouping/overlay (optional)
- **Color coding**: Elements colored by source file
- **File panel**: Sidebar showing all source files
- **Filter by file**: Show/hide elements from specific files
- **Import indicators**: Visual marks on imported elements

**Important**: File visualization is **visual only** - it doesn't change JSON structure. JSON remains self-contained.

## Benefits

✅ Self-contained JSON (portable, shareable)  
✅ File information preserved (can reconstruct file organization)  
✅ Qualified names (unambiguous, self-documenting, no conflicts)  
✅ Simple rendering (no import resolution in TypeScript)  
✅ Round-trip safe (DSL → JSON → DSL preserves structure)  
✅ Studio visualization (can show file boundaries visually)

## References

- **[Simplified Architecture Model](go/simplified-architecture-model.md)** - ⭐ Primary reference for simplified model
- [JSON Schema](go/json-schema.md) - Complete JSON structure (needs update for key-value tags)
- [Architecture Model](go/architecture-model.md) - Original complex model (see simplified model for current approach)
- [Architecture Changes](go/architecture-changes.md) - Change tracking and requirements
- [Qualified Names Specification](go/qualified-names.md) - Naming rules and examples
- [DSL Qualified Names](go/dsl-qualified-names.md) - DSL syntax enforcement
- [Relation Flattening](go/relation-flattening.md) - How relations are flattened
- [Task 1.1: JSON Exporter](go/task-1.1-json-exporter.md) - Implementation (needs update)
- [Task 1.2: JSON to AST](go/task-1.2-json-to-ast.md) - Reverse conversion (needs update)
