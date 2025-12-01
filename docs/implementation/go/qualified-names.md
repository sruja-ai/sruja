# Qualified Names Specification

## Overview

All element IDs use **qualified names** in both **DSL and JSON** for clarity, stability, and unambiguous references. This prevents naming conflicts when flattening imported files and makes element relationships explicit.

**Enforcement**: Qualified names are required in:
- ✅ DSL syntax (when referencing nested elements)
- ✅ JSON format (all element IDs)
- ✅ Relation references (from/to fields)

## Qualified Name Structure

### Naming Patterns

1. **Systems**: Simple name
   - Pattern: `SystemName`
   - Example: `ShopSystem`, `PaymentService`

2. **Containers**: System-scoped
   - Pattern: `SystemName.ContainerName`
   - Example: `ShopSystem.WebApp`, `ShopSystem.API`

3. **Components**: System and Container-scoped
   - Pattern: `SystemName.ContainerName.ComponentName`
   - Example: `ShopSystem.WebApp.ShoppingCart`, `ShopSystem.API.OrderService`

4. **Top-level Elements** (Persons, SharedArtifacts, etc.): Simple name
   - Pattern: `ElementName`
   - Example: `Customer`, `SharedLibrary`

5. **Nested Elements** (within Systems/Containers): Inherit parent qualification
   - Pattern: `SystemName.ElementName` or `SystemName.ContainerName.ElementName`
   - Example: `ShopSystem.Database`, `ShopSystem.WebApp.Queue`

## Examples

### DSL Structure (Scoping Rules)

```sruja
architecture "E-commerce" {
  person Customer "Customer"
  
  system ShopSystem "Shop System" {
    // Declarations: Simple names (scope is obvious)
    container WebApp "Web Application" {
      component ShoppingCart "Shopping Cart" {}
      component ProductCatalog "Product Catalog" {}
      
      // Relations within container scope - simple names
      ShoppingCart -> ProductCatalog "Updates"
    }
    container API "API Service" {
      component OrderService "Order Service" {}
      
      // Relations within container scope
      OrderService -> API "Uses"
    }
    datastore Database "Database" {}
    
    // Relations within system scope - simple names work
    WebApp -> API "Calls"
    API -> Database "Queries"
    
    // Cross-container components - qualified for clarity
    WebApp.ShoppingCart -> API.OrderService "Calls"
  }
  
  system PaymentSystem {
    container Gateway {}
    
    // Relations within system scope
    Gateway -> PaymentSystem "Reports to"
  }
  
  // Top-level relations - qualified names required (outside system scope)
  Customer -> ShopSystem.WebApp "Uses"
  ShopSystem.API -> PaymentSystem.Gateway "Calls"
}
```

**Key Points**:
- **Declarations**: Always use simple names (scope builds qualified internally)
- **Relations can be nested**: Within container, system, or architecture blocks
- **Within scope**: Simple names allowed for relations too
- **Outside scope**: Qualified names required

### JSON with Qualified Names

```json
{
  "metadata": {
    "name": "E-commerce",
    "sourceFiles": [
      {
        "path": "main.sruja",
        "elements": [
          "Customer",
          "ShopSystem",
          "ShopSystem.WebApp",
          "ShopSystem.WebApp.ShoppingCart",
          "ShopSystem.WebApp.ProductCatalog",
          "ShopSystem.API",
          "ShopSystem.API.OrderService",
          "ShopSystem.Database"
        ]
      }
    ]
  },
  "architecture": {
    "persons": [
      {
        "id": "Customer",
        "label": "Customer"
      }
    ],
    "systems": [
      {
        "id": "ShopSystem",
        "label": "Shop System"
      }
    ],
    "containers": [
      {
        "id": "ShopSystem.WebApp",
        "label": "Web Application"
      },
      {
        "id": "ShopSystem.API",
        "label": "API Service"
      }
    ],
    "components": [
      {
        "id": "ShopSystem.WebApp.ShoppingCart",
        "label": "Shopping Cart Component"
      },
      {
        "id": "ShopSystem.WebApp.ProductCatalog",
        "label": "Product Catalog Component"
      },
      {
        "id": "ShopSystem.API.OrderService",
        "label": "Order Service Component"
      }
    ],
    "datastores": [
      {
        "id": "ShopSystem.Database",
        "label": "Database"
      }
    ],
    "relations": [
      {
        "from": "Customer",
        "to": "ShopSystem.WebApp",
        "label": "Uses"
      },
      {
        "from": "ShopSystem.WebApp",
        "to": "ShopSystem.API",
        "label": "Calls"
      },
      {
        "from": "ShopSystem.API",
        "to": "ShopSystem.Database",
        "label": "Queries"
      }
    ]
  }
}
```

## Benefits

✅ **Cleaner DSL**: Simple names in declarations (scope-based)  
✅ **Unambiguous References**: Qualified names when outside scope  
✅ **Stability**: Qualified names prevent conflicts when flattening imports  
✅ **Self-Documenting**: Names show hierarchy and relationships  
✅ **Easy Parsing**: Can extract parent/child relationships from qualified names  
✅ **No Naming Conflicts**: Multiple files can have elements with same base name  
✅ **Familiar**: Scoping rules match programming language conventions

## Implementation

### Building Qualified Names from Scope (DSL → AST)

The parser builds qualified names from scope hierarchy:

```go
// Build qualified name from parent scope
func buildQualifiedName(element Element, parent Element) string {
    if parent == nil {
        return element.ID  // Top-level element (System, Person, etc.)
    }
    // Recursively build: parent.QualifiedID + "." + element.ID
    return fmt.Sprintf("%s.%s", parent.QualifiedID, element.ID)
}

// Example:
// - System "ShopSystem" -> QualifiedID: "ShopSystem"
// - Container "WebApp" inside ShopSystem -> QualifiedID: "ShopSystem.WebApp"
// - Component "ShoppingCart" inside WebApp -> QualifiedID: "ShopSystem.WebApp.ShoppingCart"
```

### Using Qualified Names (AST → JSON)

```go
// Use QualifiedID from AST (already built by parser)
func (e *Exporter) exportElementID(element Element) string {
    return element.QualifiedID  // Already qualified from scope
}
```

### Parsing Qualified Names (JSON → DSL)

```go
// Parse qualified name to extract parts
func parseQualifiedName(qualifiedID string) (parts []string) {
    return strings.Split(qualifiedID, ".")
}

// Example: "ShopSystem.WebApp.ShoppingCart" -> ["ShopSystem", "WebApp", "ShoppingCart"]

// Extract system name
func extractSystemName(qualifiedID string) string {
    parts := parseQualifiedName(qualifiedID)
    return parts[0]
}

// Extract container name
func extractContainerName(qualifiedID string) string {
    parts := parseQualifiedName(qualifiedID)
    if len(parts) >= 2 {
        return parts[1]
    }
    return ""
}

// Extract component name
func extractComponentName(qualifiedID string) string {
    parts := parseQualifiedName(qualifiedID)
    if len(parts) >= 3 {
        return parts[2]
    }
    return ""
}
```

## Relation References

All relation `from` and `to` fields use qualified names:

```json
{
  "relations": [
    {
      "from": "Customer",
      "to": "ShopSystem.WebApp",
      "label": "Uses"
    },
    {
      "from": "ShopSystem.WebApp.ShoppingCart",
      "to": "ShopSystem.API.OrderService",
      "label": "Calls"
    }
  ]
}
```

## DSL Syntax Rules

### Required Qualified Names

1. **Container definitions**: Must use qualified name
   ```sruja
   container ShopSystem.WebApp "Web Application" {}
   ```

2. **Component definitions**: Must use qualified name
   ```sruja
   component ShopSystem.WebApp.ShoppingCart "Shopping Cart" {}
   ```

3. **Relation references**: Must use qualified names
   ```sruja
   ShopSystem.WebApp -> ShopSystem.API "Calls"
   ```

4. **All nested element references**: Must use qualified names
   ```sruja
   ShopSystem.WebApp.ShoppingCart -> ShopSystem.API.OrderService
   ```

### Optional Shorthand (Context-Aware)

When defining nested elements, simple names can be used if context is clear:

```sruja
system ShopSystem {
  container ShopSystem.WebApp {  // Container must be qualified
    component ShoppingCart {}    // Can be simple - qualified from parent
  }
}
```

But when **referencing** elements (in relations, flows, etc.), **qualified names are required**:

```sruja
ShopSystem.WebApp.ShoppingCart -> ShopSystem.API.OrderService  // Must be qualified
```

## Migration Strategy

### DSL → JSON

1. Parse qualified names from DSL
2. Validate qualified names match hierarchy
3. Store qualified names directly in JSON
4. All relation references already use qualified names

### JSON → DSL

1. Parse qualified names from JSON
2. Extract hierarchy from qualified names
3. Generate DSL with qualified names (matching JSON)
4. Relations use qualified names as-is

**Key Point**: Qualified names are the source of truth in both DSL and JSON.
