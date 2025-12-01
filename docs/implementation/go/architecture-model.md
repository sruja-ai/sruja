# Architecture Model: Root Architecture and Shared Elements

## Overview

The architecture model supports:
1. **Root Architecture**: One primary architecture describing "architecture of what"
2. **Shared Elements**: Libraries, components, systems that can be reused across architectures
3. **One Architecture Per File**: Each `.sruja` file can have at most one architecture block
4. **Import Model**: How elements from different files are composed

## Core Principle

**An architecture block represents "the architecture of X"** - it describes the complete structure of a specific system/product.

**Rule**: **One architecture block per file** (enforced at parser level)

- Each architecture file describes ONE architecture
- Multiple architectures = multiple files (workspace concept)
- Shared elements = files without architecture blocks

See [Architecture Semantics](architecture-semantics.md) for detailed explanation of what "architecture" means and why multiple blocks in one file are not allowed.

## File Organization Patterns

| Pattern | Architecture Block | File Role | `metadata.architecture` | `metadata.shared` | Use Case |
|---------|-------------------|-----------|------------------------|-------------------|----------|
| **Single Architecture** | ✅ Main file only | Main file | Architecture name | `false` | Small to medium architectures |
| **Split Architecture** | ✅ Main file only | Partial files | Architecture name (same) | `false` | Large architectures split for organization |
| **Shared Elements** | ❌ No block | Shared file | `null` | `true` | Reusable across architectures |
| **Multiple Architectures** | ✅ Each file | Separate files | Different names | `false` | Product portfolio (workspace) |

**Key Distinction**:
- **Split Architecture**: All files contribute to ONE architecture (same `metadata.architecture`)
- **Multiple Architectures**: Each file defines a DIFFERENT architecture (different `metadata.architecture`)
- **Shared Elements**: Elements can be used by MULTIPLE architectures (`metadata.architecture: null`)

**Complete Workspace Structure** (hybrid: some single files, some split):
```
workspace/
  ├── architectures/
  │   ├── arch1/                 # Architecture 1 directory (split)
  │   │   ├── main.sruja        # ✅ Has architecture block
  │   │   └── partials/         # ❌ No blocks (same architecture)
  │   ├── arch2.sruja           # Architecture 2 (single file)
  │   │                          # ✅ Has architecture block
  │   ├── arch3/                 # Architecture 3 directory (split)
  │   │   ├── main.sruja        # ✅ Has architecture block
  │   │   └── partials/         # ❌ No blocks (same architecture)
  │   └── arch4.sruja           # Architecture 4 (single file)
  │                              # ✅ Has architecture block
  └── shared/                    # Shared across architectures
      └── *.sruja               # ❌ No blocks (reusable)
```
- **Each main file** generates its own JSON (one JSON per architecture)
- **Single-file architectures**: Simple architectures in one file
- **Split architectures**: Complex architectures in directory with partial files
- **Partial files** are imported into their main architecture
- **Shared files** can be imported by multiple architectures
- See [Example 4](#example-4-workspace-with-multiple-architectures-each-split-across-multiple-files) for split-only example
- See [Example 5](#example-5-hybrid-workspace-some-single-files-some-split) for hybrid example

## What is an Architecture?

An **architecture block** represents **"the architecture of X"** - it describes the complete structure and design of a specific system, product, or platform.

```sruja
architecture "E-commerce Platform" {
  // This describes: "the architecture of the E-commerce Platform"
  system ShopSystem {}
  system PaymentSystem {}
}
```

The architecture name answers: **"What is this the architecture of?"**

## Architecture Hierarchy

### Root Architecture

Each file has **one root architecture** (the architecture block in that file):

```sruja
architecture "E-commerce Platform" {
  // This is the root - describes the E-commerce Platform architecture
  system ShopSystem {}
  system PaymentSystem {}
}
```

**Rule**: One architecture block per file (enforced at parser level).

### Shared Elements

Shared elements are reusable across multiple architectures:

**shared.sruja**:
```sruja
// Shared library - no architecture block
library AuthLib "Authentication Library" {
  // Shared across multiple architectures
}

// Shared system - can be used by multiple architectures
system CommonAuth "Common Authentication" {
  container AuthService {}
}
```

**Main architecture imports shared elements**:
```sruja
architecture "E-commerce Platform" {
  import "shared.sruja"
  
  system ShopSystem {}
  // Can reference CommonAuth from shared.sruja
  ShopSystem -> CommonAuth "Uses"
}
```

## File Structure Models

### Model 1: Single Root Architecture

**main.sruja**:
```sruja
architecture "E-commerce Platform" {
  import "shared.sruja"
  system ShopSystem {}
}
```

**JSON Structure**:
```json
{
  "metadata": {
    "name": "E-commerce Platform",
    "rootArchitecture": "E-commerce Platform"
  },
  "architecture": {
    "systems": [...],
    "libraries": [...]  // From shared.sruja
  }
}
```

### Model 2: Multiple Architectures (Multiple Files)

**❌ NOT ALLOWED**: Multiple architecture blocks in one file

**✅ CORRECT**: Separate files, one architecture each

**ecommerce-platform.sruja**:
```sruja
architecture "E-commerce Platform" {
  system ShopSystem {}
}
```

**payment-platform.sruja**:
```sruja
architecture "Payment Platform" {
  system PaymentGateway {}
}
```

**Processing**: Each file generates its own JSON:
- `ecommerce-platform.json` - Root architecture: "E-commerce Platform"
- `payment-platform.json` - Root architecture: "Payment Platform"

### Model 3: Shared Elements File (No Architecture Block)

**shared.sruja**:
```sruja
// No architecture block - just shared elements
library AuthLib "Authentication Library" {}

system CommonAuth "Common Authentication" {
  container AuthService {}
}

person Admin "Administrator"
```

**When imported**: Elements become part of the importing architecture's scope.

### Model 4: One Architecture Split Across Multiple Files

**Pattern**: A large architecture can be split into multiple files for better organization.

**Rule**: Only ONE file has the architecture block (declares the architecture). Other files are "partial" files containing parts of the same architecture.

**Note on Directory Organization**: Directory names for partial files are flexible and user's choice. Common patterns include:
- `systems/` - When organizing by systems
- `services/` - When organizing by services  
- `modules/` - When organizing by modules
- `components/` - When organizing by components
- Or any other organizational structure that fits your project

**Main file (with architecture block)**:
```sruja
// ecommerce-platform.sruja - Main architecture file
architecture "E-commerce Platform" {
  import "systems/payment.sruja"
  import "systems/inventory.sruja"
  import "shared/auth.sruja"
  
  // Top-level systems declared here
  system ShopSystem {}
  system AdminSystem {}
  
  // Relations between top-level systems
  ShopSystem -> AdminSystem "Sends reports"
}
```

**Partial files (no architecture block - part of same architecture)**:
```sruja
// systems/payment.sruja - Part of "E-commerce Platform" architecture
// No architecture block - this is part of the main architecture

system PaymentSystem "Payment Processing" {
  container PaymentAPI {}
  container PaymentGateway {}
  
  PaymentAPI -> PaymentGateway "Processes"
}

// Relations between imported system and main architecture elements
PaymentSystem -> ShopSystem "Handles payments"
```

```sruja
// systems/inventory.sruja - Part of "E-commerce Platform" architecture
system InventorySystem "Inventory Management" {
  container InventoryAPI {}
  datastore InventoryDB {}
  
  InventoryAPI -> InventoryDB "Queries"
}

InventorySystem -> ShopSystem "Provides stock info"
```

**File structure**:
```
ecommerce-platform/
  ├── ecommerce-platform.sruja    # Main file with architecture block
  ├── systems/
  │   ├── payment.sruja            # Partial: Payment system details
  │   └── inventory.sruja          # Partial: Inventory system details
  └── shared/
      └── auth.sruja               # Shared: Authentication (reusable)
```

**JSON Structure** (all elements belong to same architecture):
```json
{
  "metadata": {
    "name": "E-commerce Platform",
    "rootArchitecture": "E-commerce Platform",
    "sourceFiles": [
      {
        "path": "ecommerce-platform.sruja",
        "architecture": "E-commerce Platform",  // Defines this architecture
        "elements": ["ShopSystem", "AdminSystem"]
      },
      {
        "path": "systems/payment.sruja",
        "architecture": "E-commerce Platform",  // Part of same architecture
        "elements": ["PaymentSystem", "PaymentSystem.PaymentAPI"]
      },
      {
        "path": "systems/inventory.sruja",
        "architecture": "E-commerce Platform",  // Part of same architecture
        "elements": ["InventorySystem"]
      },
      {
        "path": "shared/auth.sruja",
        "architecture": null,  // Shared (reusable across architectures)
        "elements": ["AuthLib"]
      }
    ]
  },
  "architecture": {
    "systems": [
      {
        "id": "ShopSystem",
        "metadata": {
          "sourceFile": "ecommerce-platform.sruja",
          "architecture": "E-commerce Platform",
          "imported": false
        }
      },
      {
        "id": "PaymentSystem",
        "metadata": {
          "sourceFile": "systems/payment.sruja",
          "architecture": "E-commerce Platform",  // Same architecture
          "imported": true  // Imported from partial file
        }
      },
      {
        "id": "InventorySystem",
        "metadata": {
          "sourceFile": "systems/inventory.sruja",
          "architecture": "E-commerce Platform",  // Same architecture
          "imported": true
        }
      }
    ]
  }
}
```

**Key Points**:
- ✅ **One architecture block** in main file only
- ✅ **Partial files** have no architecture block (they're parts of the main architecture)
- ✅ **All elements** from partial files have `metadata.architecture: "E-commerce Platform"` (same as main)
- ✅ **All elements** from partial files have `metadata.imported: true`
- ✅ **One JSON** generated per architecture (contains elements from all files)
- ✅ **File organization** preserved via `metadata.sourceFile`

**When to use**:
- Large architectures that need better organization
- Team collaboration (different teams own different files)
- Modular architecture definitions

**Difference from shared elements**:
- **Partial files**: `metadata.architecture: "E-commerce Platform"` (belongs to this architecture)
- **Shared files**: `metadata.architecture: null`, `metadata.shared: true` (reusable across architectures)

## JSON Structure Design

### Current Design: Single Root Architecture

```json
{
  "metadata": {
    "name": "E-commerce Platform",  // Root architecture name
    "rootArchitecture": "E-commerce Platform",
    "version": "1.0.0",
    "generated": "2025-01-XXT00:00:00Z",
    "sourceFiles": [
      {
        "path": "main.sruja",
        "architecture": "E-commerce Platform",  // Which architecture this file defines
        "elements": ["ShopSystem"]
      },
      {
        "path": "shared.sruja",
        "architecture": null,  // No architecture - shared elements
        "elements": ["CommonAuth", "AuthLib"]
      }
    ]
  },
  "architecture": {
    // All elements flattened - from root + shared
    "systems": [
      {
        "id": "ShopSystem",
        "metadata": {
          "sourceFile": "main.sruja",
          "architecture": "E-commerce Platform"
        }
      },
      {
        "id": "CommonAuth",
        "metadata": {
          "sourceFile": "shared.sruja",
          "architecture": null,  // Shared element
          "imported": true
        }
      }
    ],
    "libraries": [
      {
        "id": "AuthLib",
        "metadata": {
          "sourceFile": "shared.sruja",
          "architecture": null,
          "imported": true
        }
      }
    ]
  }
}
```


## Design Decision: One Architecture Per File

**Rule**: **One architecture block per file** - enforced at parser level

1. **One architecture block per file**: Each `.sruja` file can have at most one architecture block
2. **One JSON per architecture**: Files with architecture blocks generate JSON (one JSON per architecture)
3. **Architecture can span multiple files**: Main file has architecture block, partial files (no block) can be imported
4. **Shared elements are imported**: Files without architecture blocks containing shared elements (reusable across architectures)
5. **Multiple architectures = multiple files**: Product portfolio is a workspace with multiple architecture files

### Benefits

✅ **Simpler JSON structure**: Single `architecture` object  
✅ **Clear ownership**: All elements belong to one root architecture  
✅ **Self-contained**: Each JSON describes one complete architecture  
✅ **Easier rendering**: TypeScript doesn't need to handle multiple architectures

### Parser Validation

The parser should enforce one architecture per file:

```go
// Validate one architecture per file
func validateFile(file *File) error {
    architectureCount := 0
    for _, item := range file.Items {
        if item.Architecture != nil {
            architectureCount++
            if architectureCount > 1 {
                return fmt.Errorf("multiple architecture blocks not allowed in one file. Found: %d", architectureCount)
            }
        }
    }
    return nil
}
```

### Processing

**Main file (with architecture block)** generates JSON:
```go
// Process file - generates one JSON (one architecture per file)
func processFile(file *File) (*ArchitectureJSON, error) {
    if len(file.Architectures) == 0 {
        // Shared-only or partial file - no JSON generated (elements are imported)
        return nil, nil
    }
    
    if len(file.Architectures) > 1 {
        return nil, fmt.Errorf("multiple architecture blocks not allowed")
    }
    
    arch := file.Architectures[0]
    // Resolve all imports (partial files + shared files)
    return exportArchitecture(arch, file)
}
```

**Partial files (no architecture block - part of same architecture)**:
```go
// Partial files are imported into main architecture
func importPartialFile(rootArch *Architecture, partialFile *File) {
    // Add elements to root architecture
    // Mark as imported: true
    // Set metadata.architecture: rootArch.Name (same architecture)
}
```

**Shared files (no architecture block - reusable)**:
```go
// Shared elements are imported into root architecture
func importSharedElements(rootArch *Architecture, sharedFile *File) {
    // Add shared elements to root architecture
    // Mark as imported: true
    // Set metadata.shared: true
    // Set metadata.architecture: null (not owned by any architecture)
}
```

**Processing flow**:
1. Parse main file (has architecture block)
2. Resolve imports:
   - Partial files → Elements belong to same architecture (`metadata.architecture: rootArch.Name`)
   - Shared files → Elements are shared (`metadata.architecture: null`, `metadata.shared: true`)
3. Generate single JSON with all elements flattened

## Element Ownership

### Root Architecture Elements

Elements defined in the root architecture block:
- `metadata.architecture`: Root architecture name
- `metadata.imported`: false

### Shared Elements

Elements from shared files (no architecture block):
- `metadata.architecture`: null
- `metadata.imported`: true
- `metadata.shared`: true


## Examples

### Example 1: Root with Shared Elements

**main.sruja**:
```sruja
architecture "E-commerce Platform" {
  import "shared.sruja"
  system ShopSystem {}
  ShopSystem -> CommonAuth "Uses"
}
```

**shared.sruja**:
```sruja
system CommonAuth "Common Authentication" {}
library AuthLib {}
```

**JSON** (single root architecture):
```json
{
  "metadata": {
    "name": "E-commerce Platform",
    "rootArchitecture": "E-commerce Platform"
  },
  "architecture": {
    "systems": [
      {
        "id": "ShopSystem",
        "metadata": {
          "architecture": "E-commerce Platform",
          "imported": false
        }
      },
      {
        "id": "CommonAuth",
        "metadata": {
          "architecture": null,  // Shared element
          "imported": true,
          "shared": true
        }
      }
    ],
    "libraries": [
      {
        "id": "AuthLib",
        "metadata": {
          "architecture": null,
          "imported": true,
          "shared": true
        }
      }
    ]
  }
}
```

### Example 2: Multiple Architectures (Separate Files)

**Product Portfolio Structure**:
```
workspace/
  ├── ecommerce-platform.sruja
  ├── payment-platform.sruja
  └── shared/
      └── auth.sruja
```

**ecommerce-platform.sruja**:
```sruja
architecture "E-commerce Platform" {
  import "shared/auth.sruja"
  system ShopSystem {}
}
```

**payment-platform.sruja**:
```sruja
architecture "Payment Platform" {
  import "shared/auth.sruja"
  system PaymentGateway {}
}
```

**Processing**: Each file generates its own JSON:
- `ecommerce-platform.json` - Root: "E-commerce Platform"
- `payment-platform.json` - Root: "Payment Platform"

### Example 3: One Architecture Split Across Multiple Files

**Large E-commerce Platform split for better organization**:

**File structure**:
```
ecommerce-platform/
  ├── ecommerce-platform.sruja    # Main file - defines architecture
  ├── systems/
  │   ├── shop.sruja              # Shop system details
  │   ├── payment.sruja           # Payment system details
  │   └── inventory.sruja         # Inventory system details
  └── shared/
      └── auth.sruja              # Shared authentication (reusable)
```

**ecommerce-platform.sruja** (Main - has architecture block):
```sruja
architecture "E-commerce Platform" {
  import "systems/shop.sruja"
  import "systems/payment.sruja"
  import "systems/inventory.sruja"
  import "shared/auth.sruja"
  
  // Top-level declarations
  system AdminSystem "Admin Portal" {}
  
  // Relations between systems from different files
  ShopSystem -> PaymentSystem "Uses"
  ShopSystem -> InventorySystem "Checks stock"
  AdminSystem -> ShopSystem "Manages"
}
```

**systems/shop.sruja** (Partial - no architecture block):
```sruja
// Part of "E-commerce Platform" architecture
system ShopSystem "Shopping System" {
  container WebApp "Web Application" {
    component ShoppingCart {}
    component ProductCatalog {}
  }
  container API "API Service" {}
  datastore Database {}
  
  WebApp -> API "Calls"
  API -> Database "Queries"
}
```

**systems/payment.sruja** (Partial - no architecture block):
```sruja
// Part of "E-commerce Platform" architecture
system PaymentSystem "Payment Processing" {
  container PaymentAPI {}
  container Gateway {}
  
  PaymentAPI -> Gateway "Processes"
}
```

**systems/inventory.sruja** (Partial - no architecture block):
```sruja
// Part of "E-commerce Platform" architecture
system InventorySystem "Inventory Management" {
  container InventoryAPI {}
  datastore InventoryDB {}
}
```

**shared/auth.sruja** (Shared - reusable):
```sruja
// Shared across multiple architectures
library AuthLib "Authentication Library" {}
```

**JSON Output** (single JSON - one architecture):
```json
{
  "metadata": {
    "name": "E-commerce Platform",
    "rootArchitecture": "E-commerce Platform",
    "sourceFiles": [
      {
        "path": "ecommerce-platform.sruja",
        "architecture": "E-commerce Platform",
        "elements": ["AdminSystem"]
      },
      {
        "path": "systems/shop.sruja",
        "architecture": "E-commerce Platform",  // Same architecture
        "elements": ["ShopSystem", "ShopSystem.WebApp", "ShopSystem.API"]
      },
      {
        "path": "systems/payment.sruja",
        "architecture": "E-commerce Platform",  // Same architecture
        "elements": ["PaymentSystem"]
      },
      {
        "path": "systems/inventory.sruja",
        "architecture": "E-commerce Platform",  // Same architecture
        "elements": ["InventorySystem"]
      },
      {
        "path": "shared/auth.sruja",
        "architecture": null,  // Shared (reusable)
        "elements": ["AuthLib"]
      }
    ]
  },
  "architecture": {
    "systems": [
      {
        "id": "AdminSystem",
        "metadata": {
          "sourceFile": "ecommerce-platform.sruja",
          "architecture": "E-commerce Platform",
          "imported": false
        }
      },
      {
        "id": "ShopSystem",
        "metadata": {
          "sourceFile": "systems/shop.sruja",
          "architecture": "E-commerce Platform",  // Same architecture
          "imported": true
        }
      },
      {
        "id": "PaymentSystem",
        "metadata": {
          "sourceFile": "systems/payment.sruja",
          "architecture": "E-commerce Platform",  // Same architecture
          "imported": true
        }
      },
      {
        "id": "InventorySystem",
        "metadata": {
          "sourceFile": "systems/inventory.sruja",
          "architecture": "E-commerce Platform",  // Same architecture
          "imported": true
        }
      }
    ],
    "libraries": [
      {
        "id": "AuthLib",
        "metadata": {
          "sourceFile": "shared/auth.sruja",
          "architecture": null,  // Shared
          "imported": true,
          "shared": true
        }
      }
    ]
  }
}
```

**Key observations**:
- ✅ **One JSON** generated (all files contribute to same architecture)
- ✅ **Main file** has architecture block: `ecommerce-platform.sruja`
- ✅ **Partial files** have no architecture block but `metadata.architecture: "E-commerce Platform"`
- ✅ **Shared file** has `metadata.architecture: null` and `metadata.shared: true`
- ✅ **File organization** preserved in `metadata.sourceFile`

### Example 4: Workspace with Multiple Architectures (Each Split Across Multiple Files)

**Complete workspace structure** with multiple architectures, each split across multiple files:

```
workspace/
  ├── architectures/
  │   ├── ecommerce-platform/
  │   │   ├── ecommerce-platform.sruja    # Main: Defines "E-commerce Platform"
  │   │   └── systems/
  │   │       ├── shop.sruja              # Partial: Part of E-commerce Platform
  │   │       ├── payment.sruja           # Partial: Part of E-commerce Platform
  │   │       └── inventory.sruja         # Partial: Part of E-commerce Platform
  │   ├── payment-platform/
  │   │   ├── payment-platform.sruja      # Main: Defines "Payment Platform"
  │   │   └── services/
  │   │       ├── gateway.sruja           # Partial: Part of Payment Platform
  │   │       ├── processing.sruja        # Partial: Part of Payment Platform
  │   │       └── reconciliation.sruja    # Partial: Part of Payment Platform
  │   └── customer-portal/
  │       ├── customer-portal.sruja       # Main: Defines "Customer Portal"
  │       └── modules/
  │           ├── profile.sruja           # Partial: Part of Customer Portal
  │           └── dashboard.sruja         # Partial: Part of Customer Portal
  ├── shared/
  │   ├── auth.sruja                      # Shared: Reusable across architectures
  │   ├── common-libs.sruja               # Shared: Common libraries
  │   └── users.sruja                     # Shared: User entities
  └── README.md
```

**Architecture 1: E-commerce Platform** (split across multiple files)

**architectures/ecommerce-platform/ecommerce-platform.sruja** (Main):
```sruja
architecture "E-commerce Platform" {
  import "systems/shop.sruja"
  import "systems/payment.sruja"
  import "systems/inventory.sruja"
  import "../../shared/auth.sruja"
  import "../../shared/users.sruja"
  
  system AdminSystem "Admin Portal" {}
  
  // Relations between systems
  ShopSystem -> PaymentSystem "Uses"
  ShopSystem -> InventorySystem "Checks stock"
  AdminSystem -> ShopSystem "Manages"
  Customer -> ShopSystem.WebApp "Shops"
}
```

**architectures/ecommerce-platform/systems/shop.sruja** (Partial):
```sruja
// Part of "E-commerce Platform" architecture
system ShopSystem "Shopping System" {
  container WebApp "Web Application" {
    component ShoppingCart {}
    component ProductCatalog {}
  }
  container API "API Service" {}
  datastore Database {}
}
```

**architectures/ecommerce-platform/systems/payment.sruja** (Partial):
```sruja
// Part of "E-commerce Platform" architecture
system PaymentSystem "Payment Processing" {
  container PaymentAPI {}
  container Gateway {}
}
```

**architectures/ecommerce-platform/systems/inventory.sruja** (Partial):
```sruja
// Part of "E-commerce Platform" architecture
system InventorySystem "Inventory Management" {
  container InventoryAPI {}
  datastore InventoryDB {}
}
```

**Architecture 2: Payment Platform** (split across multiple files)

**architectures/payment-platform/payment-platform.sruja** (Main):
```sruja
architecture "Payment Platform" {
  import "services/gateway.sruja"
  import "services/processing.sruja"
  import "services/reconciliation.sruja"
  import "../../shared/auth.sruja"
  
  system AdminPortal "Admin Portal" {}
  
  GatewayService -> ProcessingService "Routes"
  ProcessingService -> ReconciliationService "Reconciles"
}
```

**architectures/payment-platform/services/gateway.sruja** (Partial):
```sruja
// Part of "Payment Platform" architecture
system GatewayService "Payment Gateway" {
  container GatewayAPI {}
  container Router {}
}
```

**architectures/payment-platform/services/processing.sruja** (Partial):
```sruja
// Part of "Payment Platform" architecture
system ProcessingService "Payment Processing" {
  container Processor {}
  datastore TransactionDB {}
}
```

**architectures/payment-platform/services/reconciliation.sruja** (Partial):
```sruja
// Part of "Payment Platform" architecture
system ReconciliationService "Reconciliation" {
  container Reconciler {}
  datastore ReconciliationDB {}
}
```

**Architecture 3: Customer Portal** (split across multiple files)

**architectures/customer-portal/customer-portal.sruja** (Main):
```sruja
architecture "Customer Portal" {
  import "modules/profile.sruja"
  import "modules/dashboard.sruja"
  import "../../shared/auth.sruja"
  import "../../shared/users.sruja"
  
  system PortalGateway "Portal Gateway" {}
  
  ProfileModule -> PortalGateway "Uses"
  DashboardModule -> PortalGateway "Uses"
  Customer -> PortalGateway "Accesses"
}
```

**architectures/customer-portal/modules/profile.sruja** (Partial):
```sruja
// Part of "Customer Portal" architecture
system ProfileModule "Profile Management" {
  container ProfileAPI {}
  datastore ProfileDB {}
}
```

**architectures/customer-portal/modules/dashboard.sruja** (Partial):
```sruja
// Part of "Customer Portal" architecture
system DashboardModule "Dashboard" {
  container DashboardAPI {}
}
```

**Shared Files** (reusable across architectures)

**shared/auth.sruja**:
```sruja
// Shared across multiple architectures
library AuthLib "Authentication Library" {}
system CommonAuth "Common Authentication" {
  container AuthService {}
}
```

**shared/users.sruja**:
```sruja
// Shared user entities
person Customer "Customer"
person Admin "Administrator"
```

**shared/common-libs.sruja**:
```sruja
// Shared libraries
library LoggingLib "Logging Library" {}
library MonitoringLib "Monitoring Library" {}
```

**Processing & JSON Output**

Each main architecture file generates its own JSON:

**ecommerce-platform.json**:
```json
{
  "metadata": {
    "name": "E-commerce Platform",
    "rootArchitecture": "E-commerce Platform",
    "sourceFiles": [
      {
        "path": "architectures/ecommerce-platform/ecommerce-platform.sruja",
        "architecture": "E-commerce Platform",
        "elements": ["AdminSystem"]
      },
      {
        "path": "architectures/ecommerce-platform/systems/shop.sruja",
        "architecture": "E-commerce Platform",
        "elements": ["ShopSystem", "ShopSystem.WebApp"]
      },
      {
        "path": "architectures/ecommerce-platform/systems/payment.sruja",
        "architecture": "E-commerce Platform",
        "elements": ["PaymentSystem"]
      },
      {
        "path": "architectures/ecommerce-platform/systems/inventory.sruja",
        "architecture": "E-commerce Platform",
        "elements": ["InventorySystem"]
      },
      {
        "path": "shared/auth.sruja",
        "architecture": null,
        "elements": ["AuthLib", "CommonAuth"]
      },
      {
        "path": "shared/users.sruja",
        "architecture": null,
        "elements": ["Customer"]
      }
    ]
  },
  "architecture": {
    "systems": [
      {
        "id": "AdminSystem",
        "metadata": {
          "sourceFile": "architectures/ecommerce-platform/ecommerce-platform.sruja",
          "architecture": "E-commerce Platform",
          "imported": false
        }
      },
      {
        "id": "ShopSystem",
        "metadata": {
          "sourceFile": "architectures/ecommerce-platform/systems/shop.sruja",
          "architecture": "E-commerce Platform",
          "imported": true
        }
      },
      {
        "id": "CommonAuth",
        "metadata": {
          "sourceFile": "shared/auth.sruja",
          "architecture": null,
          "imported": true,
          "shared": true
        }
      }
    ],
    "persons": [
      {
        "id": "Customer",
        "metadata": {
          "sourceFile": "shared/users.sruja",
          "architecture": null,
          "imported": true,
          "shared": true
        }
      }
    ]
  }
}
```

**payment-platform.json**:
```json
{
  "metadata": {
    "name": "Payment Platform",
    "rootArchitecture": "Payment Platform",
    "sourceFiles": [
      {
        "path": "architectures/payment-platform/payment-platform.sruja",
        "architecture": "Payment Platform",
        "elements": ["AdminPortal"]
      },
      {
        "path": "architectures/payment-platform/services/gateway.sruja",
        "architecture": "Payment Platform",
        "elements": ["GatewayService"]
      },
      {
        "path": "shared/auth.sruja",
        "architecture": null,
        "elements": ["AuthLib", "CommonAuth"]
      }
    ]
  },
  "architecture": {
    "systems": [
      {
        "id": "AdminPortal",
        "metadata": {
          "sourceFile": "architectures/payment-platform/payment-platform.sruja",
          "architecture": "Payment Platform",
          "imported": false
        }
      },
      {
        "id": "GatewayService",
        "metadata": {
          "sourceFile": "architectures/payment-platform/services/gateway.sruja",
          "architecture": "Payment Platform",
          "imported": true
        }
      },
      {
        "id": "CommonAuth",
        "metadata": {
          "sourceFile": "shared/auth.sruja",
          "architecture": null,
          "imported": true,
          "shared": true
        }
      }
    ]
  }
}
```

**customer-portal.json**:
```json
{
  "metadata": {
    "name": "Customer Portal",
    "rootArchitecture": "Customer Portal",
    "sourceFiles": [
      {
        "path": "architectures/customer-portal/customer-portal.sruja",
        "architecture": "Customer Portal",
        "elements": ["PortalGateway"]
      },
      {
        "path": "architectures/customer-portal/modules/profile.sruja",
        "architecture": "Customer Portal",
        "elements": ["ProfileModule"]
      },
      {
        "path": "shared/auth.sruja",
        "architecture": null,
        "elements": ["AuthLib", "CommonAuth"]
      }
    ]
  },
  "architecture": {
    "systems": [
      {
        "id": "PortalGateway",
        "metadata": {
          "sourceFile": "architectures/customer-portal/customer-portal.sruja",
          "architecture": "Customer Portal",
          "imported": false
        }
      },
      {
        "id": "ProfileModule",
        "metadata": {
          "sourceFile": "architectures/customer-portal/modules/profile.sruja",
          "architecture": "Customer Portal",
          "imported": true
        }
      }
    ]
  }
}
```

**Key Structure Rules**:

1. **One architecture per directory** (e.g., `ecommerce-platform/`, `payment-platform/`)
2. **One main file per architecture** (has architecture block)
3. **Multiple partial files per architecture** (no architecture block, part of same architecture)
4. **Shared directory** (contains reusable elements across architectures)
5. **One JSON per architecture** (generated from main file + imports)
6. **Clear separation**: Each architecture is self-contained but can import shared elements

**File Organization Summary**:

```
workspace/
  ├── architectures/           # Multiple architectures
  │   ├── arch1/              # Architecture 1 directory
  │   │   ├── main.sruja      # Main file (has architecture block)
  │   │   └── partials/       # Partial files (no block, same architecture)
  │   └── arch2/              # Architecture 2 directory
  │       ├── main.sruja      # Main file (has architecture block)
  │       └── partials/       # Partial files (no block, same architecture)
  └── shared/                 # Shared elements (reusable)
      └── *.sruja            # Shared files (no architecture blocks)
```

### Example 5: Hybrid Workspace (Some Single Files, Some Split)

**Realistic workspace** mixing simple single-file architectures with complex split architectures:

**Workspace structure**:
```
workspace/
  ├── architectures/
  │   ├── ecommerce-platform/        # ✅ Split architecture (large, complex)
  │   │   ├── ecommerce-platform.sruja    # Main file
  │   │   └── systems/
  │   │       ├── shop.sruja              # Partial file
  │   │       ├── payment.sruja           # Partial file
  │   │       └── inventory.sruja         # Partial file
  │   │
  │   ├── customer-portal.sruja      # ✅ Single file architecture (simple)
  │   │
  │   ├── analytics-platform/        # ✅ Split architecture (large, complex)
  │   │   ├── analytics-platform.sruja    # Main file
  │   │   └── systems/
  │   │       ├── data-collection.sruja   # Partial file
  │   │       └── reporting.sruja         # Partial file
  │   │
  │   ├── auth-service.sruja         # ✅ Single file architecture (simple)
  │   │
  │   └── notification-service.sruja # ✅ Single file architecture (simple)
  │
  └── shared/
      ├── auth.sruja                # Shared elements
      ├── users.sruja               # Shared elements
      └── common-libs.sruja         # Shared elements
```

**Architecture 1: E-commerce Platform** (Split - Large & Complex)

**architectures/ecommerce-platform/ecommerce-platform.sruja** (Main):
```sruja
architecture "E-commerce Platform" {
  import "systems/shop.sruja"
  import "systems/payment.sruja"
  import "systems/inventory.sruja"
  import "../../shared/auth.sruja"
  
  system AdminSystem "Admin Portal" {}
  
  ShopSystem -> PaymentSystem "Uses"
  ShopSystem -> InventorySystem "Checks stock"
}
```

**architectures/ecommerce-platform/systems/shop.sruja** (Partial):
```sruja
// Part of "E-commerce Platform" architecture
system ShopSystem "Shopping System" {
  container WebApp {}
  container API {}
  datastore Database {}
}
```

**architectures/ecommerce-platform/systems/payment.sruja** (Partial):
```sruja
// Part of "E-commerce Platform" architecture
system PaymentSystem "Payment Processing" {
  container PaymentAPI {}
}
```

**Architecture 2: Customer Portal** (Single File - Simple)

**architectures/customer-portal.sruja**:
```sruja
architecture "Customer Portal" {
  import "../shared/auth.sruja"
  import "../shared/users.sruja"
  
  system PortalGateway "Portal Gateway" {
    container GatewayAPI {}
    datastore PortalDB {}
  }
  
  system ProfileModule "Profile Management" {
    container ProfileAPI {}
  }
  
  system DashboardModule "Dashboard" {
    container DashboardAPI {}
  }
  
  PortalGateway -> ProfileModule "Routes"
  PortalGateway -> DashboardModule "Routes"
  Customer -> PortalGateway "Accesses"
}
```

**Architecture 3: Analytics Platform** (Split - Large & Complex)

**architectures/analytics-platform/analytics-platform.sruja** (Main):
```sruja
architecture "Analytics Platform" {
  import "systems/data-collection.sruja"
  import "systems/reporting.sruja"
  import "../shared/common-libs.sruja"
  
  system AdminDashboard "Admin Dashboard" {}
  
  DataCollectionService -> ReportingService "Sends data"
}
```

**architectures/analytics-platform/systems/data-collection.sruja** (Partial):
```sruja
// Part of "Analytics Platform" architecture
system DataCollectionService "Data Collection" {
  container Collector {}
  datastore RawDataDB {}
}
```

**Architecture 4: Auth Service** (Single File - Simple)

**architectures/auth-service.sruja**:
```sruja
architecture "Auth Service" {
  import "../shared/common-libs.sruja"
  
  system AuthService "Authentication Service" {
    container AuthAPI {}
    container TokenService {}
    datastore UserDB {}
    
    AuthAPI -> TokenService "Generates tokens"
    AuthAPI -> UserDB "Validates users"
  }
}
```

**Architecture 5: Notification Service** (Single File - Simple)

**architectures/notification-service.sruja**:
```sruja
architecture "Notification Service" {
  system NotificationService "Notification Service" {
    container NotificationAPI {}
    container EmailSender {}
    container SMSSender {}
    
    NotificationAPI -> EmailSender "Sends emails"
    NotificationAPI -> SMSSender "Sends SMS"
  }
}
```

**Shared Files**

**shared/auth.sruja**:
```sruja
library AuthLib "Authentication Library" {}
```

**shared/users.sruja**:
```sruja
person Customer "Customer"
person Admin "Administrator"
```

**Processing & JSON Output**

Each architecture generates its own JSON (regardless of whether it's single file or split):

**ecommerce-platform.json** (from split architecture):
```json
{
  "metadata": {
    "name": "E-commerce Platform",
    "rootArchitecture": "E-commerce Platform",
    "sourceFiles": [
      {
        "path": "architectures/ecommerce-platform/ecommerce-platform.sruja",
        "architecture": "E-commerce Platform",
        "elements": ["AdminSystem"]
      },
      {
        "path": "architectures/ecommerce-platform/systems/shop.sruja",
        "architecture": "E-commerce Platform",
        "elements": ["ShopSystem"]
      },
      {
        "path": "shared/auth.sruja",
        "architecture": null,
        "elements": ["AuthLib"]
      }
    ]
  },
  "architecture": {
    "systems": [
      {
        "id": "AdminSystem",
        "metadata": {
          "sourceFile": "architectures/ecommerce-platform/ecommerce-platform.sruja",
          "architecture": "E-commerce Platform",
          "imported": false
        }
      },
      {
        "id": "ShopSystem",
        "metadata": {
          "sourceFile": "architectures/ecommerce-platform/systems/shop.sruja",
          "architecture": "E-commerce Platform",
          "imported": true
        }
      }
    ]
  }
}
```

**customer-portal.json** (from single file architecture):
```json
{
  "metadata": {
    "name": "Customer Portal",
    "rootArchitecture": "Customer Portal",
    "sourceFiles": [
      {
        "path": "architectures/customer-portal.sruja",
        "architecture": "Customer Portal",
        "elements": ["PortalGateway", "ProfileModule", "DashboardModule"]
      },
      {
        "path": "shared/auth.sruja",
        "architecture": null,
        "elements": ["AuthLib"]
      },
      {
        "path": "shared/users.sruja",
        "architecture": null,
        "elements": ["Customer"]
      }
    ]
  },
  "architecture": {
    "systems": [
      {
        "id": "PortalGateway",
        "metadata": {
          "sourceFile": "architectures/customer-portal.sruja",
          "architecture": "Customer Portal",
          "imported": false
        }
      },
      {
        "id": "ProfileModule",
        "metadata": {
          "sourceFile": "architectures/customer-portal.sruja",
          "architecture": "Customer Portal",
          "imported": false
        }
      }
    ]
  }
}
```

**auth-service.json** (from single file architecture):
```json
{
  "metadata": {
    "name": "Auth Service",
    "rootArchitecture": "Auth Service",
    "sourceFiles": [
      {
        "path": "architectures/auth-service.sruja",
        "architecture": "Auth Service",
        "elements": ["AuthService"]
      },
      {
        "path": "shared/common-libs.sruja",
        "architecture": null,
        "elements": ["LoggingLib"]
      }
    ]
  },
  "architecture": {
    "systems": [
      {
        "id": "AuthService",
        "metadata": {
          "sourceFile": "architectures/auth-service.sruja",
          "architecture": "Auth Service",
          "imported": false
        }
      }
    ]
  }
}
```

**Key Observations**:

✅ **Flexible organization**: Mix single-file and split architectures as needed  
✅ **Single-file architectures**: Simple architectures stay in one file (`customer-portal.sruja`, `auth-service.sruja`)  
✅ **Split architectures**: Complex architectures split across multiple files (`ecommerce-platform/`, `analytics-platform/`)  
✅ **One JSON per architecture**: Regardless of file organization, each architecture generates one JSON  
✅ **Same processing**: Both patterns work the same way - main file + imports → JSON  
✅ **Shared elements**: All architectures can import from shared directory  

**When to use single file vs split**:
- **Single file**: Simple architectures (few systems, straightforward structure)
- **Split**: Large/complex architectures (many systems, better organization needed, team collaboration)

## Recommendations

1. **One Architecture Per File**: Enforce at parser level (validation error if multiple found)
2. **One JSON Per Architecture File**: Each file generates its own JSON
3. **Shared Elements as Imports**: Shared elements become part of importing architecture
4. **Multiple Architectures = Multiple Files**: Product portfolio is a workspace
5. **Metadata Tracking**: Track which architecture each element belongs to
6. **Shared Flag**: Mark shared elements with `metadata.shared: true`

## Product Portfolio = Workspace

A product with multiple architectures is modeled as a **workspace**:

**Simple workspace structure** (one file per architecture):
```
product-workspace/
  ├── architectures/
  │   ├── ecommerce-platform.sruja    # One architecture
  │   ├── payment-platform.sruja      # Another architecture
  │   └── customer-portal.sruja       # Another architecture
  ├── shared/
  │   ├── auth.sruja                  # Shared elements
  │   └── common.sruja                # Shared elements
  └── README.md
```

**Complex workspace structure** (multiple architectures, each split across multiple files):
See [Example 4](#example-4-workspace-with-multiple-architectures-each-split-across-multiple-files) for complete structure.

**Hybrid workspace structure** (mix of single-file and split architectures):
See [Example 5](#example-5-hybrid-workspace-some-single-files-some-split) for realistic hybrid structure.

Each architecture file:
- Contains one architecture block
- Can import partial files (same architecture)
- Can import shared elements (reusable across architectures)
- Generates its own JSON file

**Workspace Rules**:
1. **Multiple architectures** = Multiple main files (each has architecture block)
2. **Split architecture** = Main file + partial files (partial files have no architecture block)
3. **Shared elements** = Files without architecture blocks (can be imported by multiple architectures)
4. **One JSON per architecture** = Generated from main file + all imports (partial + shared)
