# JSON Schema Specification

This document defines the JSON format that serves as the **contract** between Go (DSL ↔ JSON) and TypeScript (JSON ↔ Diagram).

## Design Principles

1. **Self-Contained**: JSON is standalone - no imports or external dependencies
2. **Single Root Architecture**: Each JSON represents one root architecture (multiple architectures in file → separate JSON files)
3. **Structure-Based File Organization**: File organization is determined by JSON structure (no file name tracking needed)
4. **Standard File Names**: When converting JSON → DSL, use standard file names (architecture.sruja, requirements.sruja, etc.)
5. **Qualified Names**: All element IDs use qualified names (e.g., `SystemName.ContainerName.ComponentName`) for clarity and stability
6. **Round-trip Preservation**: All information must survive DSL → JSON → DSL (file organization is reconstructible from structure)

See [Simplified Architecture Model](simplified-architecture-model.md) for details on the simplified model.

## Complete JSON Structure

```json
{
  "metadata": {
    "name": "Architecture Name",
    "rootArchitecture": "Architecture Name",  // Root architecture this JSON represents
    "version": "1.0.0",
    "generated": "2025-01-XXT00:00:00Z"
  },
  "architecture": {
    "systems": [
      {
        "id": "ShopSystem",
        "label": "Shop System",
        "metadata": {
          "team": "Platform Team"  // Team ownership - just team ID
        },
        // ... other system properties
      }
    ],
    "containers": [
      {
        "id": "ShopSystem.WebApp",
        "label": "Web Application",
        "metadata": {
          "team": "Platform Team",
          "owners": ["alice@example.com"]
        },
        // ... other container properties
      }
    ],
    "components": [
      {
        "id": "ShopSystem.WebApp.ShoppingCart",
        "label": "Shopping Cart Component",
        "metadata": {
          // No sourceFile tracking - file organization determined by JSON structure
        },
        // ... other component properties
      }
    ],
    "persons": [
      {
        "id": "Customer",
        "label": "Customer",
        "metadata": {
          "shared": true  // Mark as shared element (referenced via naming convention)
        }
      }
    ],
    "relations": [...],
    "flows": [...],  // Always inside architecture block
    "domains": [...],
    "deployment": [...],
    "contracts": [...],
    "sharedArtifacts": [...],
    "libraries": [...],
    "policies": [...],
    "constraints": [...],
    "conventions": [...],
    "views": [...]
  },
  "requirements": [...],  // Separate from architecture (concept-based files)
  "userStories": [...],   // Separate from architecture (concept-based files)
  "adrs": [...],          // Separate from architecture (concept-based files)
  "scenarios": [...],     // Separate from architecture (concept-based files)
  "navigation": {
    "levels": ["level1", "level2", "level3"],
    "scenarios": [...],
    "flows": [...],
    "domains": [...]
  }
}
```

## Key Points

1. **No imports array**: JSON is self-contained - all elements are flattened
2. **Single root architecture**: Each JSON represents one root architecture (one architecture block per file enforced)
3. **No file name tracking**: File organization is determined by JSON structure:
   - `architecture` block → `architecture.sruja`
   - `requirements` array → `requirements.sruja`
   - `userStories` array → `stories.sruja`
   - `adrs` array → `decisions.sruja`
   - `scenarios` array → `scenarios.sruja`
4. **Shared elements**: Shared services use naming convention (`shared.ServiceName`) - marked with `metadata.shared: true`
5. **Standard file names**: JSON → DSL conversion uses standard file names only (no custom file names)
7. **All fields optional**: Missing fields should be handled gracefully
8. **Round-trip preservation**: File boundaries and architecture ownership can be reconstructed from metadata when converting back to DSL
9. **Versioned**: Metadata includes version for future compatibility

## Element Metadata Structure

Each architecture element can have metadata:

```json
{
  "id": "ElementID",
  "label": "Element Label",
  "metadata": {
    "sourceFile": "path/to/file.sruja",  // File where element is defined
    "architecture": "Architecture Name",  // Which architecture owns this element (null if shared)
    "imported": false,                     // Whether element was imported
    "importedFrom": null,                 // Original file if imported (for tracking)
    "shared": false,                       // Whether element is shared (from file without architecture block)
    "tags": [                              // Key-value tags
      {"type": "system", "value": "ShopSystem"},
      {"type": "container", "value": "ShopSystem.AnalyticsAPI"}
    ],
    // ... other metadata fields
  }
}
```

### Metadata Fields

- **`sourceFile`**: File path where element is defined
- **`architecture`**: Architecture name that owns this element
  - Root architecture name: Element belongs to root architecture
  - `null`: Element is shared (from file without architecture block)
- **`imported`**: `true` if element came from import
- **`importedFrom`**: Source file path if imported
- **`shared`**: `true` if element is shared (from file without architecture block)

## File Boundary Reconstruction

When converting JSON → DSL:
- Group elements by `metadata.sourceFile`
- Separate root architecture elements from shared elements using `metadata.architecture`
- Reconstruct import statements based on `metadata.imported` and `metadata.sourceFile`
- Preserve file organization using `metadata.sourceFiles` mapping
- Shared elements (`metadata.shared: true`) go into files without architecture blocks

## Qualified Names

All element IDs use qualified names for clarity and stability:
- Systems: `SystemName`
- Containers: `SystemName.ContainerName`
- Components: `SystemName.ContainerName.ComponentName`

See [Qualified Names Specification](qualified-names.md) for complete naming rules and examples.

## One Architecture Per File

**Rule**: **One architecture block per file** (enforced at parser level)

Each `.sruja` file can have:
- **One architecture block** → Generates one JSON file
- **No architecture block** (shared elements only) → No JSON (elements are imported)

**Multiple architectures** are modeled as **multiple files** (workspace concept):

```
workspace/
  ├── ecommerce-platform.sruja    # One architecture
  ├── payment-platform.sruja      # Another architecture
  └── shared/
      └── auth.sruja              # Shared elements (no architecture block)
```

Each architecture file generates its own JSON:
- `ecommerce-platform.json` (root architecture: "E-commerce Platform")
- `payment-platform.json` (root architecture: "Payment Platform")

Each JSON is self-contained and represents one root architecture.

## Shared Services (Naming Convention)

Shared services use naming convention instead of explicit imports:

**shared/auth-service.sruja**:
```sruja
architecture "Auth Service" {
  system AuthAPI {}
}
```

**architecture/ecommerce-platform.sruja**:
```sruja
architecture "E-commerce Platform" {
  external shared.AuthService "Auth Service"  // Naming convention
  system ShopSystem {}
  ShopSystem -> shared.AuthService "Uses"
}
```

No explicit imports needed - parser resolves `shared.ServiceName` from `shared/` directory.

## Key-Value Tags Structure

Requirements, user stories, ADRs, and scenarios use key-value tags:

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

**JSON Structure**:
```json
{
  "requirements": [
    {
      "id": "REQ-123",
      "description": "Analytics dashboard",
      "tags": [
        {"type": "system", "value": "ShopSystem"},
        {"type": "container", "value": "ShopSystem.AnalyticsAPI"}
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
        {"type": "container", "value": "ShopSystem.AnalyticsAPI"},
        {"type": "flow", "value": "OrderFlow"}
      ],
      "requirement": "REQ-123"
    }
  ],
  "adrs": [
    {
      "id": "ADR-001",
      "title": "Use REST API",
      "tags": [
        {"type": "container", "value": "ShopSystem.AnalyticsAPI"}
      ],
      "status": "decided"
    }
  ],
  "scenarios": [
    {
      "id": "High Traffic",
      "description": "System behavior during high traffic",
      "tags": [
        {"type": "system", "value": "ShopSystem"},
        {"type": "container", "value": "ShopSystem.AnalyticsAPI"}
      ]
    }
  ]
}
```

**Tag Types**:
- `system`: System ID (e.g., "ShopSystem")
- `container`: Fully qualified container ID (e.g., "ShopSystem.AnalyticsAPI")
- `component`: Fully qualified component ID (e.g., "ShopSystem.AnalyticsAPI.MetricsCollector")
- `flow`: Flow ID (e.g., "OrderFlow")
- `story`: User story ID (e.g., "US-456")

**Important**: Containers and components in tags must be fully qualified.

## Flows Structure

Flows are part of the architecture block and can have tags:

**JSON Structure**:
```json
{
  "architecture": {
    "flows": [
      {
        "id": "OrderFlow",
        "title": "Order Processing Flow",
        "tags": [
          {"type": "story", "value": "US-456"}
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

## Relations and Flows Location

**Rule**: Relations and flows must always be inside the architecture block:
- Relations: Simple connections between elements
- Flows: Sequences showing data/value flow through system
- Both are exported in the `architecture` object, not at root level

## Simplified File Organization

**Concept-Based Files** (standard names):
- `architecture.sruja` - Architecture structure (elements, relations, flows)
- `requirements.sruja` - Requirements
- `decisions.sruja` - ADRs
- `stories.sruja` - User stories
- `scenarios.sruja` - Scenarios

**OR Single File**: Everything in one file with clear sections

**No splitting**: Architecture stays in ONE file - no `main.sruja`, `systems.sruja`, etc.

See [Simplified Architecture Model](simplified-architecture-model.md) for complete details.

## Type Definitions

See individual task files for detailed type definitions:
- [Task 1.1: JSON Exporter](task-1.1-json-exporter.md) - Go type definitions
- [Qualified Names Specification](qualified-names.md) - Naming rules
- [Architecture Model](architecture-model.md) - Root architecture and shared elements
- TypeScript types will be generated from JSON schema (future work)
