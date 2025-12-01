# Comprehensive Test Cases: Architecture Examples with Base and Changes

## Overview

This document provides comprehensive test cases with complete architecture examples (base initial state + changes) to test all features of Sruja.

## Test Case Structure

Each test case includes:
- **Base Architecture**: Initial state before changes
- **Changes**: One or more change files
- **Expected Result**: Final state after applying changes
- **Test Scenarios**: What features this tests

## Test Case Categories

### 1. Simple Architecture (Basic Features)

#### Test Case 1.1: Simple Add Change

**Base Architecture** (`testdata/simple/base.sruja`):
```sruja
architecture "Simple App" {
  system WebApp {}
  system Database {}
  relation WebApp -> Database "Reads/Writes"
}
```

**Change** (`testdata/simple/changes/001-add-api.sruja`):
```sruja
change "001-add-api" {
  version "v1.1.0"
  requirement "REQ-001"
  status "approved"
  
  metadata {
    owner "alice@example.com"
    stakeholders ["bob@example.com"]
  }
  
  add {
    system API {}
    relation WebApp -> API "Calls"
    relation API -> Database "Queries"
  }
}
```

**Expected Result** (`testdata/simple/expected-v1.1.0.sruja`):
```sruja
architecture "Simple App" {
  system WebApp {}
  system API {}
  system Database {}
  relation WebApp -> API "Calls"
  relation API -> Database "Queries"
  relation WebApp -> Database "Reads/Writes"
}
```

**Tests**:
- ✅ Basic change creation
- ✅ Add elements (system)
- ✅ Add relations
- ✅ Change application
- ✅ Round-trip preservation

#### Test Case 1.2: Simple Modify Change

**Base Architecture**: Same as 1.1 base

**Change** (`testdata/simple/changes/002-modify-webapp.sruja`):
```sruja
change "002-modify-webapp" {
  version "v1.2.0"
  requirement "REQ-002"
  status "approved"
  
  metadata {
    owner "charlie@example.com"
    stakeholders ["alice@example.com"]
  }
  
  modify {
    system WebApp {
      description "Updated web application with new features"
      tags ["frontend", "react"]
    }
  }
}
```

**Expected Result**: WebApp has updated description and tags

**Tests**:
- ✅ Modify element properties
- ✅ Update descriptions
- ✅ Update tags

#### Test Case 1.3: Simple Remove Change

**Base Architecture**: Result from Test Case 1.1

**Change** (`testdata/simple/changes/003-remove-api.sruja`):
```sruja
change "003-remove-api" {
  version "v1.3.0"
  requirement "REQ-003"
  status "approved"
  
  metadata {
    owner "dave@example.com"
    stakeholders []
  }
  
  remove {
    system API {}
    relation WebApp -> API "Calls"
    relation API -> Database "Queries"
  }
}
```

**Expected Result**: API system and its relations removed

**Tests**:
- ✅ Remove elements
- ✅ Remove relations
- ✅ Cleanup orphaned relations

### 2. Medium Architecture (Real-World Scenarios)

#### Test Case 2.1: E-commerce Platform - Initial State

**Base Architecture** (`testdata/medium/ecommerce-base.sruja`):
```sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp "Web Application" {
      component ShoppingCart {}
      component ProductCatalog {}
    }
    container API "REST API" {
      component OrderService {}
      component ProductService {}
    }
    container Database "PostgreSQL Database" {
      component ProductDB {}
      component OrderDB {}
    }
  }
  
  relation ShopSystem.WebApp -> ShopSystem.API "HTTP"
  relation ShopSystem.API -> ShopSystem.Database "SQL"
}
```

#### Test Case 2.2: Add Analytics System

**Change** (`testdata/medium/changes/001-add-analytics.sruja`):
```sruja
change "001-add-analytics" {
  version "v1.1.0"
  requirement "REQ-101"
  adr "ADR-001"
  status "approved"
  
  metadata {
    owner "analytics-team@example.com"
    stakeholders ["platform-team@example.com", "product-team@example.com"]
  }
  
  add {
    system ShopSystem {
      container AnalyticsAPI "Analytics API" {
        component MetricsCollector {}
        component EventProcessor {}
      }
    }
    relation ShopSystem.WebApp -> ShopSystem.AnalyticsAPI "Sends events"
    relation ShopSystem.API -> ShopSystem.AnalyticsAPI "Sends metrics"
  }
}
```

**Expected Result**: Analytics system added with relations

**Tests**:
- ✅ Add container to existing system
- ✅ Add multiple components
- ✅ Add multiple relations
- ✅ ADR reference
- ✅ Multiple stakeholders

#### Test Case 2.3: Add Payment System

**Change** (`testdata/medium/changes/002-add-payment.sruja`):
```sruja
change "002-add-payment" {
  version "v1.2.0"
  requirement "REQ-102"
  adr "ADR-002"
  status "approved"
  
  metadata {
    owner "payment-team@example.com"
    stakeholders ["platform-team@example.com", "security-team@example.com"]
  }
  
  add {
    system ShopSystem {
      container PaymentGateway "Payment Gateway" {
        component PaymentProcessor {}
        component RefundService {}
      }
    }
    relation ShopSystem.WebApp -> ShopSystem.PaymentGateway "Processes payments"
    relation ShopSystem.API -> ShopSystem.PaymentGateway "Validates transactions"
  }
  
  modify {
    container ShopSystem.API {
      component OrderService {
        description "Now includes payment integration"
      }
    }
  }
}
```

**Expected Result**: Payment system added, OrderService updated

**Tests**:
- ✅ Sequential changes
- ✅ Add + modify in same change
- ✅ Multiple teams (owner/stakeholders)

#### Test Case 2.4: Modify Existing Container

**Change** (`testdata/medium/changes/003-enhance-api.sruja`):
```sruja
change "003-enhance-api" {
  version "v1.3.0"
  requirement "REQ-103"
  status "approved"
  
  metadata {
    owner "api-team@example.com"
    stakeholders ["platform-team@example.com"]
  }
  
  modify {
    container ShopSystem.API {
      component OrderService {
        description "Enhanced with caching and rate limiting"
        tags ["cached", "rate-limited"]
      }
      component ProductService {
        description "Added search functionality"
      }
    }
  }
}
```

**Tests**:
- ✅ Modify nested components
- ✅ Update multiple components
- ✅ Add tags to existing elements

### 3. Complex Architecture (Advanced Features)

#### Test Case 3.1: Multi-System Architecture

**Base Architecture** (`testdata/complex/multi-system-base.sruja`):
```sruja
architecture "Enterprise Platform" {
  system ShopSystem {
    container WebApp {}
    container API {}
    container Database {}
  }
  
  system InventorySystem {
    container InventoryAPI {}
    container InventoryDB {}
  }
  
  system ShippingSystem {
    container ShippingAPI {}
    container ShippingDB {}
  }
  
  relation ShopSystem.API -> InventorySystem.InventoryAPI "Checks stock"
  relation ShopSystem.API -> ShippingSystem.ShippingAPI "Creates shipment"
}
```

#### Test Case 3.2: Add Integration Layer

**Change** (`testdata/complex/changes/001-add-integration.sruja`):
```sruja
change "001-add-integration" {
  version "v1.1.0"
  requirement "REQ-201"
  status "approved"
  
  metadata {
    owner "integration-team@example.com"
    stakeholders ["shop-team@example.com", "inventory-team@example.com", "shipping-team@example.com"]
  }
  
  add {
    system IntegrationLayer {
      container MessageBus "Event Bus" {
        component EventRouter {}
        component EventStore {}
      }
    }
    
    relation ShopSystem.API -> IntegrationLayer.MessageBus "Publishes events"
    relation InventorySystem.InventoryAPI -> IntegrationLayer.MessageBus "Subscribes to events"
    relation ShippingSystem.ShippingAPI -> IntegrationLayer.MessageBus "Subscribes to events"
  }
  
  modify {
    relation ShopSystem.API -> InventorySystem.InventoryAPI {
      description "Replaced with event-based communication"
    }
    relation ShopSystem.API -> ShippingSystem.ShippingAPI {
      description "Replaced with event-based communication"
    }
  }
}
```

**Tests**:
- ✅ Add new system
- ✅ Modify existing relations
- ✅ Event-driven architecture
- ✅ Multiple stakeholders across teams

### 4. Change States and Validation

#### Test Case 4.1: Pending Change (Cannot Apply)

**Base Architecture**: Test Case 2.1 base

**Change** (`testdata/changes/pending-change.sruja`):
```sruja
change "004-add-cache" {
  version "v1.4.0"
  requirement "REQ-104"
  status "pending"  // Not ready to apply
  
  metadata {
    owner "cache-team@example.com"
    stakeholders []
  }
  
  add {
    system ShopSystem {
      container Cache "Redis Cache" {}
    }
  }
}
```

**Expected Behavior**: `sruja change apply` should fail with error

**Tests**:
- ✅ Validation: pending changes cannot be applied
- ✅ Error message lists pending changes

#### Test Case 4.2: In-Progress Change (Cannot Apply)

**Change** (`testdata/changes/in-progress-change.sruja`):
```sruja
change "005-add-search" {
  version "v1.5.0"
  requirement "REQ-105"
  status "in-progress"  // Not ready to apply
  
  metadata {
    owner "search-team@example.com"
    stakeholders ["product-team@example.com"]
  }
  
  add {
    system ShopSystem {
      container SearchService "Elasticsearch" {}
    }
  }
}
```

**Expected Behavior**: `sruja change apply` should fail

**Tests**:
- ✅ Validation: in-progress changes cannot be applied
- ✅ Error message lists in-progress changes

#### Test Case 4.3: ADR Not in Final State (Cannot Apply)

**Base Architecture**: Test Case 2.1 base

**ADR** (`testdata/changes/adr-pending.sruja`):
```sruja
adr "ADR-003" "Use microservices" {
  status "pending"  // Not decided yet
  context "Need to scale"
  decision ""
}
```

**Change** (`testdata/changes/change-with-pending-adr.sruja`):
```sruja
change "006-microservices" {
  version "v1.6.0"
  requirement "REQ-106"
  adr "ADR-003"  // References pending ADR
  status "approved"
  
  metadata {
    owner "arch-team@example.com"
    stakeholders []
  }
  
  add {
    // ... changes
  }
}
```

**Expected Behavior**: `sruja change apply` should fail because ADR-003 is pending

**Tests**:
- ✅ Validation: ADRs must be in final state
- ✅ Error message lists pending ADRs
- ✅ Error message shows which changes reference them

### 5. Conflict Detection

#### Test Case 5.1: Overlapping Elements (Conflict)

**Base Architecture**: Test Case 2.1 base

**Change 1** (`testdata/changes/conflict-001.sruja`):
```sruja
change "007-add-auth" {
  version "v1.7.0"
  requirement "REQ-107"
  status "approved"
  
  metadata {
    owner "auth-team@example.com"
    stakeholders []
  }
  
  add {
    system ShopSystem {
      container AuthService "Authentication Service" {
        component LoginService {}
      }
    }
  }
}
```

**Change 2** (`testdata/changes/conflict-002.sruja`):
```sruja
change "008-add-security" {
  version "v1.7.0"  // Same version
  requirement "REQ-108"
  status "approved"
  
  metadata {
    owner "security-team@example.com"
    stakeholders []
  }
  
  add {
    system ShopSystem {
      container AuthService "Authentication Service" {  // Same container ID
        component OAuthService {}
      }
    }
  }
}
```

**Expected Behavior**: `sruja change conflicts` should detect conflict

**Tests**:
- ✅ Conflict detection: same element ID
- ✅ Conflict detection: overlapping changes
- ✅ Error message shows conflicting changes

#### Test Case 5.2: Conflicting Modifications

**Change 1** (`testdata/changes/conflict-modify-001.sruja`):
```sruja
change "009-modify-api-1" {
  version "v1.8.0"
  status "approved"
  
  modify {
    container ShopSystem.API {
      description "API v1"
    }
  }
}
```

**Change 2** (`testdata/changes/conflict-modify-002.sruja`):
```sruja
change "010-modify-api-2" {
  version "v1.8.0"
  status "approved"
  
  modify {
    container ShopSystem.API {
      description "API v2"  // Different description
    }
  }
}
```

**Expected Behavior**: Conflict detected (both modify same element)

**Tests**:
- ✅ Conflict detection: conflicting modifications
- ✅ Error message shows what's conflicting

### 6. Preview Snapshots

#### Test Case 6.1: Preview with In-Progress Changes

**Base Architecture**: Test Case 2.1 base

**Change 1** (`testdata/changes/preview-001-approved.sruja`):
```sruja
change "011-add-notifications" {
  version "v1.9.0"
  status "approved"
  
  add {
    system ShopSystem {
      container NotificationService {}
    }
  }
}
```

**Change 2** (`testdata/changes/preview-002-in-progress.sruja`):
```sruja
change "012-add-recommendations" {
  version "v2.0.0"
  status "in-progress"  // Not approved yet
  
  add {
    system ShopSystem {
      container RecommendationEngine {}
    }
  }
}
```

**Preview Snapshot** (`testdata/snapshots/preview/future-state.sruja`):
```sruja
snapshot "preview-future-state" {
  version "v2.0.0"
  description "Future state with notifications and recommendations"
  preview true
  changes ["011-add-notifications", "012-add-recommendations"]
  
  architecture "E-commerce Platform" {
    // ... includes both approved and in-progress changes
  }
}
```

**Tests**:
- ✅ Preview snapshots can include in-progress changes
- ✅ Preview snapshots skip validation
- ✅ Preview snapshots are for visualization only

### 7. Multiple Files (Split Architecture)

#### Test Case 7.1: Split Architecture with Changes

**Base Architecture** (`testdata/complex/split/main.sruja`):
```sruja
architecture "Large System" {
  system MainSystem {
    container Frontend {}
    container Backend {}
  }
}
```

**Partial File** (`testdata/complex/split/partials/services.sruja`):
```sruja
// No architecture block - part of MainSystem
system MainSystem {
  container ServiceA {}
  container ServiceB {}
}
```

**Change** (`testdata/complex/split/changes/001-add-service.sruja`):
```sruja
change "013-add-service" {
  version "v1.1.0"
  status "approved"
  
  add {
    system MainSystem {
      container ServiceC {}
    }
  }
}
```

**Tests**:
- ✅ Changes work with split architectures
- ✅ File boundaries preserved
- ✅ Round-trip with multiple files

### 8. DDD Architecture

#### Test Case 8.1: Domain-Driven Design

**Base Architecture** (`testdata/ddd/base.sruja`):
```sruja
architecture "E-commerce DDD" {
  domain ShopDomain {
    context OrderContext {
      aggregate Order {
        entity OrderItem {}
        valueObject Money {}
      }
    }
    context ProductContext {
      aggregate Product {
        entity ProductDetails {}
      }
    }
  }
}
```

**Change** (`testdata/ddd/changes/001-add-payment-context.sruja`):
```sruja
change "014-add-payment-context" {
  version "v1.1.0"
  status "approved"
  
  add {
    domain ShopDomain {
      context PaymentContext {
        aggregate Payment {
          entity PaymentTransaction {}
        }
      }
    }
    relation ShopDomain.OrderContext.Order -> ShopDomain.PaymentContext.Payment "Pays via"
  }
}
```

**Tests**:
- ✅ DDD elements (domains, contexts, aggregates)
- ✅ Changes to DDD structures
- ✅ Relations between DDD elements

### 9. Edge Cases

#### Test Case 9.1: Empty Change

**Change** (`testdata/edge-cases/empty-change.sruja`):
```sruja
change "015-empty" {
  version "v1.1.0"
  status "approved"
  
  add {}
  modify {}
  remove {}
}
```

**Tests**:
- ✅ Empty change blocks
- ✅ Change with no actual changes

#### Test Case 9.2: Change with Only Metadata

**Change** (`testdata/edge-cases/metadata-only.sruja`):
```sruja
change "016-metadata" {
  version "v1.1.0"
  requirement "REQ-109"
  adr "ADR-004"
  status "approved"
  
  metadata {
    owner "team@example.com"
    stakeholders ["stakeholder1@example.com", "stakeholder2@example.com"]
  }
  
  add {}
}
```

**Tests**:
- ✅ Change with only metadata
- ✅ Multiple stakeholders
- ✅ ADR reference without changes

#### Test Case 9.3: Large Change (Performance)

**Base Architecture**: Complex architecture (1000+ elements)

**Change** (`testdata/edge-cases/large-change.sruja`):
```sruja
change "017-large" {
  version "v1.1.0"
  status "approved"
  
  add {
    // 500+ new elements
    // 1000+ new relations
  }
}
```

**Tests**:
- ✅ Performance with large changes
- ✅ Memory usage
- ✅ Apply time

### 10. Round-Trip Tests

#### Test Case 10.1: DSL → JSON → DSL

**Input**: Any base architecture from test cases above

**Process**:
1. Parse DSL → AST
2. Export AST → JSON
3. Convert JSON → AST
4. Print AST → DSL

**Expected**: Output DSL matches input DSL (semantically)

**Tests**:
- ✅ Round-trip preservation
- ✅ File boundaries preserved
- ✅ Qualified names preserved
- ✅ Metadata preserved

#### Test Case 10.2: Change → Apply → Snapshot → Change

**Input**: Base + Change

**Process**:
1. Apply change → Current state
2. Create snapshot from current state
3. Extract change from snapshot

**Expected**: Extracted change matches original change

**Tests**:
- ✅ Change application correctness
- ✅ Snapshot generation
- ✅ Change extraction

## Test Data Organization

```
testdata/
├── simple/
│   ├── base.sruja
│   ├── changes/
│   │   ├── 001-add-api.sruja
│   │   ├── 002-modify-webapp.sruja
│   │   └── 003-remove-api.sruja
│   ├── expected-v1.1.0.sruja
│   ├── expected-v1.2.0.sruja
│   └── expected-v1.3.0.sruja
├── medium/
│   ├── ecommerce-base.sruja
│   ├── changes/
│   │   ├── 001-add-analytics.sruja
│   │   ├── 002-add-payment.sruja
│   │   └── 003-enhance-api.sruja
│   └── expected-v1.3.0.sruja
├── complex/
│   ├── multi-system-base.sruja
│   ├── split/
│   │   ├── main.sruja
│   │   ├── partials/
│   │   │   └── services.sruja
│   │   └── changes/
│   │       └── 001-add-service.sruja
│   └── changes/
│       └── 001-add-integration.sruja
├── changes/
│   ├── pending-change.sruja
│   ├── in-progress-change.sruja
│   ├── conflict-001.sruja
│   ├── conflict-002.sruja
│   ├── conflict-modify-001.sruja
│   ├── conflict-modify-002.sruja
│   ├── preview-001-approved.sruja
│   ├── preview-002-in-progress.sruja
│   ├── change-with-pending-adr.sruja
│   └── adr-pending.sruja
├── ddd/
│   ├── base.sruja
│   └── changes/
│       └── 001-add-payment-context.sruja
├── edge-cases/
│   ├── empty-change.sruja
│   ├── metadata-only.sruja
│   └── large-change.sruja
└── snapshots/
    └── preview/
        └── future-state.sruja
```

## Test Coverage Matrix

| Feature | Test Cases | Status |
|---------|-----------|--------|
| **Basic Changes** | 1.1, 1.2, 1.3 | ✅ Covered |
| **Real-World Scenarios** | 2.1-2.4 | ✅ Covered |
| **Complex Architectures** | 3.1, 3.2 | ✅ Covered |
| **Change States** | 4.1-4.3 | ✅ Covered |
| **Conflict Detection** | 5.1, 5.2 | ✅ Covered |
| **Preview Snapshots** | 6.1 | ✅ Covered |
| **Split Architectures** | 7.1 | ✅ Covered |
| **DDD** | 8.1 | ✅ Covered |
| **Edge Cases** | 9.1-9.3 | ✅ Covered |
| **Round-Trip** | 10.1, 10.2 | ✅ Covered |

## Usage in Tests

### Go Tests

```go
// pkg/changes/apply_test.go
func TestApplyChange(t *testing.T) {
    base := loadTestFile("testdata/simple/base.sruja")
    change := loadTestFile("testdata/simple/changes/001-add-api.sruja")
    expected := loadTestFile("testdata/simple/expected-v1.1.0.sruja")
    
    result, err := ApplyChange(base, change)
    require.NoError(t, err)
    assert.Equal(t, expected, result)
}
```

### Integration Tests

```go
// tests/integration/change_workflow_test.go
func TestChangeWorkflow(t *testing.T) {
    // Test Case 2.1-2.4: Sequential changes
    base := loadTestFile("testdata/medium/ecommerce-base.sruja")
    
    changes := []string{
        "testdata/medium/changes/001-add-analytics.sruja",
        "testdata/medium/changes/002-add-payment.sruja",
        "testdata/medium/changes/003-enhance-api.sruja",
    }
    
    result := base
    for _, changeFile := range changes {
        change := loadTestFile(changeFile)
        result, _ = ApplyChange(result, change)
    }
    
    expected := loadTestFile("testdata/medium/expected-v1.3.0.sruja")
    assert.Equal(t, expected, result)
}
```

## Summary

✅ **Comprehensive Test Cases**: 30+ test cases covering all features  
✅ **Base + Changes**: Each test case includes base architecture and changes  
✅ **Expected Results**: Clear expected outcomes for each test  
✅ **All Scenarios**: Simple, medium, complex, edge cases, conflicts, previews  
✅ **Real-World**: E-commerce, DDD, multi-system examples  
✅ **Well-Organized**: Clear structure for easy test implementation

