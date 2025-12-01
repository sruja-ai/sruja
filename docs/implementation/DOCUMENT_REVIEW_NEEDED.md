# Document Review: Significant DSL Changes

## Summary of Major Changes

We've made significant changes to the DSL model that require updates across multiple documents:

### 1. Simplified Architecture Model
- ✅ **No complex file splitting** - Architecture stays in ONE file
- ✅ **Concept-based files only** - Standard names: `architecture.sruja`, `requirements.sruja`, `decisions.sruja`, `stories.sruja`, `scenarios.sruja`
- ✅ **No imports** - Shared services use naming convention (`shared.ServiceName`)
- ✅ **Relations & flows** always inside architecture block only

### 2. Key-Value Tag Syntax
- ✅ **New syntax**: `tags [system "name1", system "name2", container "SystemName.ContainerName"]`
- ✅ **Fully qualified containers**: `container "ShopSystem.AnalyticsAPI"` (not just `"AnalyticsAPI"`)
- ✅ **Multiple values**: Can tag multiple systems/containers/components per requirement/story/ADR

### 3. Tagging Model
- ✅ **Requirements/Stories/ADRs/Scenarios** use tags to link to elements
- ✅ **Flows and user stories** link via tags bidirectionally
- ✅ **No nested requirements** - Everything uses tags

### 4. Migration-Based Changes
- ✅ **Linear change history** - Like database migrations
- ✅ **Snapshots at any point** - Can generate current state

## Documents That Need Updates

### Critical Updates Needed

#### 1. `JSON_DESIGN_SUMMARY.md` ⚠️ NEEDS UPDATE
**Issues**:
- Still mentions "imports" and "shared elements" concept
- No mention of simplified architecture model
- No mention of key-value tag syntax
- No mention of concept-based files
- References old complex architecture model

**Required Changes**:
- Update to reflect simplified model (no imports, naming convention)
- Update tag examples to key-value syntax
- Update file organization to concept-based files
- Reference simplified-architecture-model.md

#### 2. `go/json-schema.md` ⚠️ NEEDS UPDATE
**Issues**:
- Still has `imports` concept (should be removed)
- Tag structure shows old array format `["tag1", "tag2"]`
- No mention of key-value tags
- References old complex architecture model

**Required Changes**:
- Remove imports references (except shared services naming convention)
- Update tag structure to key-value format: `[{type: "system", value: "ShopSystem"}, {type: "container", value: "ShopSystem.API"}]`
- Add requirements/stories/ADRs/scenarios to JSON structure
- Add flows to JSON structure
- Update to reflect concept-based files

#### 3. `go/task-1.1-json-exporter.md` ⚠️ NEEDS UPDATE
**Issues**:
- Still references imports
- Tag structure outdated
- No mention of requirements/stories/ADRs/scenarios export
- No mention of flows export

**Required Changes**:
- Remove import handling logic
- Update tag export to key-value format
- Add export for requirements/stories/ADRs/scenarios
- Add export for flows
- Update file organization handling

#### 4. `go/task-1.2-json-to-ast.md` ⚠️ NEEDS UPDATE
**Issues**:
- Still references imports reconstruction
- Tag structure outdated
- No mention of requirements/stories/ADRs/scenarios import
- No mention of flows import

**Required Changes**:
- Remove import reconstruction logic (use naming convention)
- Update tag import to key-value format
- Add import for requirements/stories/ADRs/scenarios
- Add import for flows
- Update file organization reconstruction

#### 5. `go/architecture-model.md` ⚠️ NEEDS UPDATE
**Issues**:
- Complex model with partial files, shared elements, imports
- Doesn't reflect simplified model

**Required Changes**:
- Add note at top referencing `simplified-architecture-model.md`
- Update examples to use concept-based files
- Remove or deprecate complex splitting examples
- Reference simplified model as primary

#### 6. `go/README.md` ⚠️ NEEDS UPDATE
**Issues**:
- References old architecture model documents
- Doesn't reference simplified architecture model

**Required Changes**:
- Add reference to simplified-architecture-model.md
- Update key design decisions to reflect simplified model

### Moderate Updates Needed

#### 7. `go/qualified-names.md`
**Status**: Mostly OK, but should verify tag examples

#### 8. `go/dsl-qualified-names.md`
**Status**: Mostly OK, but should verify examples

#### 9. `go/relation-flattening.md`
**Status**: Still valid - relations are flattened

#### 10. `go/file-metadata-design.md`
**Status**: May need updates for simplified model (concept-based files)

### Documentation References to Update

#### 11. `IMPLEMENTATION_PLAN.md`
- Should reference simplified architecture model
- Update file structure examples

#### 12. `overview.md`
- Update to mention simplified model
- Update file organization examples

## JSON Schema Changes Needed

### Tag Structure (Critical)

**Old format** (in json-schema.md):
```json
{
  "metadata": {
    "tags": ["tag1", "tag2"]
  }
}
```

**New format** (should be):
```json
{
  "metadata": {
    "tags": [
      {"type": "system", "value": "ShopSystem"},
      {"type": "container", "value": "ShopSystem.AnalyticsAPI"},
      {"type": "component", "value": "ShopSystem.AnalyticsAPI.MetricsCollector"}
    ]
  }
}
```

**OR simpler array format** (based on DSL syntax):
```json
{
  "tags": [
    {"system": "ShopSystem"},
    {"container": "ShopSystem.AnalyticsAPI"},
    {"flow": "OrderFlow"},
    {"story": "US-456"}
  ]
}
```

### Requirements/Stories/ADRs/Scenarios in JSON

**Need to add**:
```json
{
  "requirements": [
    {
      "id": "REQ-123",
      "description": "Analytics dashboard",
      "tags": [
        {"system": "ShopSystem"},
        {"container": "ShopSystem.AnalyticsAPI"}
      ],
      "priority": "high",
      "status": "approved"
    }
  ],
  "userStories": [
    {
      "id": "US-456",
      "description": "View metrics",
      "tags": [
        {"container": "ShopSystem.AnalyticsAPI"},
        {"flow": "OrderFlow"}
      ],
      "requirement": "REQ-123"
    }
  ],
  "adrs": [
    {
      "id": "ADR-001",
      "title": "Use REST API",
      "tags": [
        {"container": "ShopSystem.AnalyticsAPI"}
      ],
      "status": "decided"
    }
  ],
  "scenarios": [
    {
      "id": "High Traffic",
      "description": "System behavior during high traffic",
      "tags": [
        {"system": "ShopSystem"},
        {"container": "ShopSystem.AnalyticsAPI"}
      ]
    }
  ]
}
```

### Flows in JSON

**Need to add** (flows are in architecture block):
```json
{
  "architecture": {
    "flows": [
      {
        "id": "OrderFlow",
        "title": "Order Processing Flow",
        "tags": [
          {"story": "US-456"}
        ],
        "steps": [
          {
            "from": "Customer",
            "to": "ShopSystem.WebApp",
            "data": "Submits order"
          },
          {
            "from": "ShopSystem.WebApp",
            "to": "ShopSystem.API",
            "data": "Sends order request"
          }
        ]
      }
    ]
  }
}
```

## Priority Order for Updates

1. **🔴 Critical**: `json-schema.md` - This defines the contract
2. **🔴 Critical**: `JSON_DESIGN_SUMMARY.md` - High-level summary
3. **🔴 Critical**: `task-1.1-json-exporter.md` - Implementation details
4. **🔴 Critical**: `task-1.2-json-to-ast.md` - Implementation details
5. **🟡 High**: `architecture-model.md` - Add reference to simplified model
6. **🟡 High**: `go/README.md` - Update references
7. **🟢 Medium**: `IMPLEMENTATION_PLAN.md` - Update examples
8. **🟢 Medium**: `overview.md` - Update examples

## Next Steps

1. Update `json-schema.md` with new tag structure and requirements/stories/ADRs/scenarios
2. Update `JSON_DESIGN_SUMMARY.md` to reflect simplified model
3. Update task files to reflect new structure
4. Add deprecation notes to old complex model documents
5. Verify all examples use new syntax

