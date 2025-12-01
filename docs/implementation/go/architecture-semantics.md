# Architecture Semantics: What Does "Architecture" Mean?

## Core Concept

An **architecture block** represents **"the architecture of X"** - it describes the complete structure and design of a specific system, product, or platform.

```sruja
architecture "E-commerce Platform" {
  // This describes: "the architecture of the E-commerce Platform"
  system ShopSystem {}
  system PaymentSystem {}
}
```

## What an Architecture Block Represents

An architecture block answers:
- **"What is this the architecture of?"** → The architecture name
- **"What are the components?"** → Systems, containers, components, etc.
- **"How do they relate?"** → Relations, flows, scenarios
- **"What are the constraints?"** → Requirements, ADRs, policies

## Multiple Architectures

### What "Multiple Architectures" Means

Multiple architectures represent **different, distinct systems/products**:
- "E-commerce Platform" architecture
- "Payment Platform" architecture
- "Customer Portal" architecture

These are **separate systems** with their own architectures.

### Product with Multiple Architectures

A product portfolio might have multiple architectures:

```
workspace/
  ├── ecommerce-platform.sruja    # Architecture of E-commerce Platform
  ├── payment-platform.sruja      # Architecture of Payment Platform
  ├── customer-portal.sruja       # Architecture of Customer Portal
  └── shared/                     # Shared elements
      ├── auth.sruja              # Shared authentication
      └── common.sruja            # Common libraries
```

**Rule**: **One architecture block per file**
- Each `.sruja` file with an architecture block describes ONE architecture
- Multiple architectures = multiple files

### Why Not Multiple Architectures in One File?

❌ **Confusing**: Which architecture are you describing?  
❌ **Complex**: Hard to understand scope and boundaries  
❌ **Unclear ownership**: Which elements belong to which architecture?  
❌ **Hard to maintain**: Changes affect multiple architectures  

✅ **Better**: One file = one architecture = clear ownership

## File Structure Rules

### Rule 1: One Architecture Block Per File

**✅ Allowed**:
```sruja
// ecommerce-platform.sruja
architecture "E-commerce Platform" {
  system ShopSystem {}
}
```

**❌ Not Allowed**:
```sruja
// platforms.sruja
architecture "E-commerce Platform" {
  system ShopSystem {}
}

architecture "Payment Platform" {
  system PaymentGateway {}
}
```

**Error**: Multiple architecture blocks in one file not allowed. Split into separate files.

### Rule 2: Shared Elements (No Architecture Block)

**✅ Allowed**:
```sruja
// shared/auth.sruja
// No architecture block - shared elements only
library AuthLib "Authentication Library" {}
system CommonAuth "Common Authentication" {}
```

These are **shared elements** that can be imported by multiple architectures.

### Rule 3: Architecture Imports Shared Elements

```sruja
// ecommerce-platform.sruja
architecture "E-commerce Platform" {
  import "shared/auth.sruja"
  
  system ShopSystem {}
  ShopSystem -> CommonAuth "Uses"
}
```

## Product Portfolio Modeling

### Scenario: Product with Multiple Architectures

You have a product portfolio with:
- E-commerce Platform
- Payment Platform
- Customer Portal

**File Structure**:
```
product/
  ├── ecommerce-platform.sruja    # Architecture of E-commerce Platform
  ├── payment-platform.sruja      # Architecture of Payment Platform
  ├── customer-portal.sruja       # Architecture of Customer Portal
  └── shared/
      ├── auth.sruja              # Shared: Authentication
      ├── common-libs.sruja       # Shared: Common libraries
      └── users.sruja             # Shared: User entities
```

**Each Architecture File**:
- Describes one complete architecture
- Can import shared elements
- Generates its own JSON file

**JSON Output**:
- `ecommerce-platform.json` - Architecture of E-commerce Platform
- `payment-platform.json` - Architecture of Payment Platform
- `customer-portal.json` - Architecture of Customer Portal

Each JSON is self-contained and represents one architecture.

## Workspace Concept

A **workspace** is a directory containing:
- Multiple architecture files (one architecture each)
- Shared element files (no architecture blocks)
- Organized in logical structure

**Example Workspace**:
```
workspace/
  ├── architectures/
  │   ├── ecommerce-platform.sruja
  │   ├── payment-platform.sruja
  │   └── customer-portal.sruja
  ├── shared/
  │   ├── auth.sruja
  │   └── common.sruja
  └── README.md
```

## Benefits of One Architecture Per File

✅ **Clear ownership**: Each file describes one architecture  
✅ **Simple model**: Easy to understand  
✅ **Easy to navigate**: Find architecture by file name  
✅ **Clear boundaries**: Each architecture is self-contained  
✅ **Better organization**: Logical file structure  
✅ **Simpler JSON**: One JSON per architecture file

## Validation

The parser should enforce:
- **Maximum one architecture block per file**
- **Error if multiple architecture blocks found**

```
Error: Multiple architecture blocks in one file not allowed
  Found: "E-commerce Platform" and "Payment Platform"
  Hint: Split into separate files: ecommerce-platform.sruja and payment-platform.sruja
  at platforms.sruja:1:1 and platforms.sruja:10:1
```

## Summary

- **Architecture block** = "the architecture of X"
- **One architecture block per file** = simple, clear model
- **Multiple architectures** = multiple files
- **Product portfolio** = workspace with multiple architecture files
- **Shared elements** = files without architecture blocks
