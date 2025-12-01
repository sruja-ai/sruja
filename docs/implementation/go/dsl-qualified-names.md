# DSL Qualified Names Enforcement (Scoping Rules)

## Overview

Qualified names follow **scoping rules** like programming languages:
- **Declarations**: Simple names (scope is obvious from nesting)
- **References within scope**: Simple names (scope is obvious)
- **References from outside scope**: Qualified names (must be explicit)

This makes the DSL cleaner while maintaining clarity for cross-scope references.

## Scoping Rules

### Declarations (Inside Parent Scope)

**Use simple names** - scope is obvious from nesting:

```sruja
system ShopSystem "Shop System" {
  container WebApp "Web Application" {}  // Simple name - inside ShopSystem
  container API "API Service" {
    component OrderService "Order Service" {}  // Simple name - inside API
    component PaymentService "Payment Service" {}
  }
  datastore Database "Database" {}  // Simple name - inside ShopSystem
}
```

### References (Depends on Scope)

**Within same scope**: Simple names are allowed
**From outside scope**: Must use qualified names

```sruja
system ShopSystem {
  container WebApp {
    component ShoppingCart {}
    component ProductCatalog {}
    
    // Within same container scope - can use simple names
    ShoppingCart -> ProductCatalog "Updates"
  }
  
  container API {
    component OrderService {}
  }
  
  // Cross-container within same system - qualified needed
  WebApp -> API "Calls"  // Could be qualified: ShopSystem.WebApp -> ShopSystem.API
  WebApp.ShoppingCart -> API.OrderService "Adds items"  // Qualified for clarity
}

// Top-level (outside all systems) - must use qualified names
Customer -> ShopSystem.WebApp "Uses"  // Qualified - outside ShopSystem scope
ShopSystem.API -> PaymentSystem.Gateway "Calls"  // Qualified - cross-system
```

### Relation References (Scoping Rules)

Relations can be defined **within their parent scope** - no need to force them outside:

```sruja
architecture "E-commerce" {
  person Customer "Customer"
  
  system ShopSystem {
    container WebApp {
      component ShoppingCart {}
      component ProductCatalog {}
      
      // Relations within container scope - simple names work
      ShoppingCart -> ProductCatalog "Updates"
      ShoppingCart -> WebApp "Uses parent"
    }
    
    container API {
      component OrderService {}
      
      // Relations within container scope
      OrderService -> API "Uses parent"
    }
    
    // Relations within system scope - simple names work
    WebApp -> API "Calls"
    API -> Database "Queries"
    
    // Cross-container component relations - qualified for clarity
    WebApp.ShoppingCart -> API.OrderService "Sends order"
  }
  
  system PaymentSystem {
    container Gateway {}
    
    // Relations within system scope
    Gateway -> PaymentSystem "Reports to"
  }
  
  // Top-level relations (outside systems) - qualified names required
  Customer -> ShopSystem.WebApp "Uses"
  ShopSystem.API -> PaymentSystem.Gateway "Calls"
}
```

**Key Rules**:
- Relations **within same container** can use simple names
- Relations **within same system** can use simple names
- Relations **between systems** must use qualified names
- Relations **at architecture level** must use qualified names
- Relations **from/to nested components** should use qualified names for clarity

## Validation Rules

### Parser Validation

The parser should:

1. **Accept simple names in declarations** (within parent scope)
2. **Build qualified names internally** from scope hierarchy
3. **Validate relation references** based on scope:
   - Within same system: Simple names allowed
   - Cross-system: Qualified names required
   - Top-level: Qualified names required
4. **Resolve references** to check if qualified name is needed

### Error Messages

Clear error messages when qualified names are needed:

```
Error: Reference 'WebApp' is ambiguous. Use qualified name 'ShopSystem.WebApp'
  at test.sruja:10:5
  Hint: Reference is outside 'ShopSystem' scope
```

```
Error: Cross-system reference must use qualified name
  Expected: 'ShopSystem.API -> PaymentSystem.Gateway'
  Found: 'API -> PaymentSystem.Gateway'
  at test.sruja:15:3
```

## Benefits

✅ **Unambiguous**: No confusion about which element is referenced  
✅ **Self-Documenting**: Names show hierarchy explicitly  
✅ **Consistent**: Same format in DSL and JSON  
✅ **Prevents Errors**: Catch mistakes at parse time  
✅ **Better Tooling**: Easier to implement autocomplete and validation

## Examples

### Complete Example

```sruja
architecture "E-commerce Platform" {
  person Customer "Customer"
  
  system ShopSystem "Shop System" {
    container WebApp "Web Application" {
      component ShoppingCart "Shopping Cart" {}
      component ProductCatalog "Product Catalog" {}
      
      // Relations within container scope - simple names
      ShoppingCart -> ProductCatalog "Updates"
      ShoppingCart -> WebApp "Uses parent"
    }
    
    container API "API Service" {
      component OrderService "Order Service" {}
      component PaymentService "Payment Service" {}
      
      // Relations within container scope
      OrderService -> PaymentService "Delegates to"
    }
    
    datastore Database "Database" {}
    
    // Relations within system scope - simple names work
    WebApp -> API "Calls"
    API -> Database "Queries"
    
    // Cross-container component references - qualified for clarity
    WebApp.ShoppingCart -> API.OrderService "Adds items"
  }
  
  system PaymentSystem "Payment System" {
    container Gateway "Payment Gateway" {}
    
    // Relations within system scope
    Gateway -> PaymentSystem "Reports to"
  }
  
  // Top-level relations (outside systems) - qualified names required
  Customer -> ShopSystem.WebApp "Uses"
  ShopSystem.API.OrderService -> PaymentSystem.Gateway "Processes payment"
}
```

### Scoping Summary

- **Declarations**: Always simple names (scope from nesting)
- **Relations can be nested**: Within container, system, or architecture level
- **Within same scope**: Simple names allowed
- **Cross-system scope**: Qualified names required
- **Top-level scope**: Qualified names required

## Implementation Notes

### Parser Changes

The parser needs to:

1. Accept **simple names in declarations** (within parent scope)
2. **Build qualified names internally** from scope hierarchy
3. Resolve references based on scope rules
4. Store both simple ID and qualified ID in AST
5. Validate cross-scope references use qualified names

### AST Structure

The AST stores both simple and qualified IDs:

```go
type Container struct {
    ID          string  // Stores "WebApp" (simple, from DSL)
    QualifiedID string  // Stores "ShopSystem.WebApp" (built from scope)
    // ...
}

type Component struct {
    ID          string  // Stores "ShoppingCart" (simple, from DSL)
    QualifiedID string  // Stores "ShopSystem.WebApp.ShoppingCart" (built from scope)
    // ...
}
```

### Qualified Name Generation

```go
// Build qualified name from scope hierarchy
func buildQualifiedName(element Element, parent Element) string {
    if parent == nil {
        return element.ID  // Top-level element
    }
    return fmt.Sprintf("%s.%s", parent.QualifiedID, element.ID)
}
```

### Validation

Add validation rule for scope-based references:

```go
type QualifiedNameRule struct{}

func (r *QualifiedNameRule) Validate(program *Program) []ValidationError {
    // Check relation references:
    // - Within same system: Simple names OK
    // - Cross-system: Must use qualified names
    // - Top-level: Must use qualified names
    // - Resolve references to check if qualified name is needed
}
```
