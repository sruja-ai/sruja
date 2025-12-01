# Relation Flattening in JSON

## Overview

Relations can be defined at multiple scopes in DSL:
- Within container blocks
- Within system blocks  
- At architecture/top level

In JSON, all relations are **flattened** into a single top-level `relations` array.

## DSL Structure

### Nested Relations

```sruja
architecture "E-commerce" {
  person Customer "Customer"
  
  system ShopSystem {
    container WebApp {
      component ShoppingCart {}
      component ProductCatalog {}
      
      // Relation within container scope
      ShoppingCart -> ProductCatalog "Updates"
    }
    
    container API {
      component OrderService {}
      
      // Relation within container scope
      OrderService -> API "Uses"
    }
    
    // Relation within system scope
    WebApp -> API "Calls"
    API -> Database "Queries"
  }
  
  system PaymentSystem {
    container Gateway {}
    
    // Relation within system scope
    Gateway -> PaymentSystem "Reports to"
  }
  
  // Top-level relation (architecture scope)
  Customer -> ShopSystem.WebApp "Uses"
  ShopSystem.API -> PaymentSystem.Gateway "Calls"
}
```

## JSON Structure (Flattened)

All relations are collected into a single array:

```json
{
  "architecture": {
    "relations": [
      {
        "from": "ShopSystem.WebApp.ShoppingCart",
        "to": "ShopSystem.WebApp.ProductCatalog",
        "label": "Updates",
        "metadata": {
          "sourceFile": "main.sruja",
          "scope": "container"
        }
      },
      {
        "from": "ShopSystem.API.OrderService",
        "to": "ShopSystem.API",
        "label": "Uses",
        "metadata": {
          "sourceFile": "main.sruja",
          "scope": "container"
        }
      },
      {
        "from": "ShopSystem.WebApp",
        "to": "ShopSystem.API",
        "label": "Calls",
        "metadata": {
          "sourceFile": "main.sruja",
          "scope": "system"
        }
      },
      {
        "from": "ShopSystem.API",
        "to": "ShopSystem.Database",
        "label": "Queries",
        "metadata": {
          "sourceFile": "main.sruja",
          "scope": "system"
        }
      },
      {
        "from": "PaymentSystem.Gateway",
        "to": "PaymentSystem",
        "label": "Reports to",
        "metadata": {
          "sourceFile": "main.sruja",
          "scope": "system"
        }
      },
      {
        "from": "Customer",
        "to": "ShopSystem.WebApp",
        "label": "Uses",
        "metadata": {
          "sourceFile": "main.sruja",
          "scope": "architecture"
        }
      },
      {
        "from": "ShopSystem.API",
        "to": "PaymentSystem.Gateway",
        "label": "Calls",
        "metadata": {
          "sourceFile": "main.sruja",
          "scope": "architecture"
        }
      }
    ]
  }
}
```

## Implementation

### Collecting Relations

```go
func collectAllRelations(arch *Architecture) []*Relation {
    var allRelations []*Relation
    
    // Top-level architecture relations
    allRelations = append(allRelations, arch.Relations...)
    
    // System-level relations
    for _, sys := range arch.Systems {
        allRelations = append(allRelations, sys.Relations...)
        
        // Container-level relations
        for _, cont := range sys.Containers {
            allRelations = append(allRelations, cont.Relations...)
        }
    }
    
    return allRelations
}
```

### Building Qualified References

When flattening, ensure all relation references use qualified names:

```go
func buildQualifiedRelation(rel *Relation, scope *Scope) *RelationJSON {
    return &RelationJSON{
        From: buildQualifiedReference(rel.From, scope),
        To:   buildQualifiedReference(rel.To, scope),
        Label: rel.Label,
        Metadata: MetadataJSON{
            SourceFile: scope.SourceFile,
            Scope:      scope.Type, // "container", "system", "architecture"
        },
    }
}
```

## Benefits

✅ **Simpler JSON structure**: Single relations array  
✅ **Scope information preserved**: Via metadata.scope  
✅ **All relations accessible**: No need to traverse hierarchy  
✅ **Consistent with element flattening**: Same approach for all nested structures

## Round-trip Reconstruction

When converting JSON → DSL:
1. Read `metadata.scope` to determine nesting level
2. Group relations by scope
3. Reconstruct nested structure:
   - Container-scope relations → inside container blocks
   - System-scope relations → inside system blocks
   - Architecture-scope relations → at top level
