# Change Modeling Summary: Architecture Evolution

## Core Principle: What IS an Architectural Change?

**Architectural changes modify the STRUCTURE, not implementation:**

### ✅ IS Architectural Change
- **Adding NEW elements** (systems, containers, components)
- **Removing/deprecating elements**
- **Creating new versions** (deprecating old + adding new)
- **Adding/removing relations**
- **Structural modifications**

### ❌ NOT Architectural Change
- Changing existing component implementation
- Performance optimizations (unless requires new structure)
- Bug fixes, refactoring
- Feature additions within existing structure

**Key Rule**:
- **NEW component** = ✅ Architectural change (track in architecture)
- **Changing existing component** = ❌ NOT architectural change (track in issue tracker)
- **Deprecating + new version** = ✅ Architectural change (track in architecture)

## Core Insight

**Most architecture changes happen at Container/System level:**
- ✅ **Architecture level**: Greenfield design (once per architecture)
- ✅ **Container/System level**: Common evolution patterns (most changes)
- ✅ **Component level**: Adding NEW components (not modifying existing)

## Change Levels in Practice

### Level 1: Architecture (Greenfield)

**When**: Initial design
**Example**: "Design the E-commerce Platform architecture"

```sruja
architecture "E-commerce Platform" {
  system ShopSystem {}
  system PaymentSystem {}
  system InventorySystem {}
}
```

### Level 2: Container/System (Evolution) ⭐ MOST COMMON

**When**: Architecture exists, adding new capabilities
**Example**: "Add analytics to E-commerce Platform"

```sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp {}
    container API {}
    
    // NEW - Container level change
    container AnalyticsAPI "Analytics API" {
      change "CHG-001" {
        requirement "REQ-123"
        scope "container"
      }
    }
  }
}
```

**Example**: "Add new inventory system"

```sruja
architecture "E-commerce Platform" {
  system ShopSystem {}  // Existing
  
  // NEW - System level change
  system InventorySystem "Inventory Management" {
    change "CHG-002" {
      requirement "REQ-124"
      scope "system"
    }
    
    container InventoryAPI {}
    datastore InventoryDB {}
  }
}
```

### Level 3: Component (New Components Only)

**When**: Adding NEW components (not modifying existing)
**Example**: "Add cache component to shopping cart"

**✅ Architectural Change**: Adding NEW component
```sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp {
      component ShoppingCart {}  // Existing
      
      // NEW component - architectural change
      component CartCache {
        change "CHG-003" {
          requirement "REQ-125"
          scope "component"
        }
      }
    }
  }
}
```

**❌ NOT Architectural Change**: Modifying existing component
```sruja
component ShoppingCart {
  // Changing implementation, adding features
  // This is implementation-level, tracked in issue tracker
  // NO change annotation - not architectural change
}
```

## Recommended Change Model

### 1. Requirements-First Approach

Define requirements separately, then link to architecture elements:

```sruja
// requirements/analytics.sruja
requirement "REQ-123" functional "As a business analyst, I need real-time sales analytics" {
  priority "high"
  status "approved"
  
  userStories [
    "US-456" "View daily sales metrics"
    "US-457" "Export analytics data"
  ]
}

userStory "US-456" "View daily sales metrics on dashboard" {
  requirement "REQ-123"
  acceptanceCriteria [
    "Dashboard shows real-time metrics"
    "Metrics update every 5 minutes"
  ]
}
```

### 2. Architecture with Change Annotations

Annotate new/modified elements with change information:

```sruja
// architecture/ecommerce-platform.sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp {}  // Existing
    container API {}     // Existing
    
    // NEW: Container-level change
    container AnalyticsAPI "Analytics API" {
      change {
        id "CHG-001"
        requirement "REQ-123"
        userStory "US-456"
        scope "container"
        status "planned"
        version "v2.1.0"
      }
      
      component MetricsCollector {}
      component DashboardAPI {}
    }
  }
  
  // NEW: Relation change
  relation ShopSystem.WebApp -> ShopSystem.AnalyticsAPI "Sends metrics" {
    change {
      id "CHG-001"
      requirement "REQ-123"
    }
  }
}
```

### 3. Change Scope Tracking

Elements track their change scope:

```json
{
  "id": "ShopSystem.AnalyticsAPI",
  "metadata": {
    "change": {
      "id": "CHG-001",
      "scope": "container",  // Architecture | System | Container | Component
      "requirement": "REQ-123",
      "status": "planned"
    }
  }
}
```

## Complete Example: Container-Level Change

### Step 1: Define Requirement

```sruja
// requirements/payment-integration.sruja
requirement "REQ-124" functional "Integrate payment gateway" {
  priority "high"
  status "approved"
  
  userStories [
    "US-458" "Process credit card payments"
  ]
}
```

### Step 2: Design Architecture Change

```sruja
// architecture/ecommerce-platform.sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp {}
    container API {}
    
    // NEW: Payment gateway integration (Container-level change)
    container PaymentGateway "Payment Gateway Client" {
      change {
        id "CHG-002"
        requirement "REQ-124"
        userStory "US-458"
        scope "container"
        status "in-progress"
      }
      
      component PaymentProcessor {}
      component GatewayAdapter {}
    }
  }
  
  // NEW: Relation for payment processing
  relation ShopSystem.API -> ShopSystem.PaymentGateway "Processes payments" {
    change {
      id "CHG-002"
    }
  }
}
```

### Step 3: Track Status

As work progresses, update change status:

```sruja
container PaymentGateway {
  change {
    status "in-progress"  // Changed from "planned"
    startedDate "2025-02-01"
  }
}
```

## JSON Structure

```json
{
  "metadata": {
    "name": "E-commerce Platform",
    "version": "2.1.0",
    "changes": [
      {
        "id": "CHG-001",
        "name": "Add Analytics Dashboard",
        "scope": "container",
        "requirement": "REQ-123",
        "status": "planned"
      },
      {
        "id": "CHG-002",
        "name": "Integrate Payment Gateway",
        "scope": "container",
        "requirement": "REQ-124",
        "status": "in-progress"
      }
    ]
  },
  "architecture": {
    "containers": [
      {
        "id": "ShopSystem.AnalyticsAPI",
        "label": "Analytics API",
        "metadata": {
          "change": {
            "id": "CHG-001",
            "scope": "container",
            "requirement": "REQ-123",
            "status": "planned"
          }
        }
      },
      {
        "id": "ShopSystem.PaymentGateway",
        "label": "Payment Gateway Client",
        "metadata": {
          "change": {
            "id": "CHG-002",
            "scope": "container",
            "requirement": "REQ-124",
            "status": "in-progress"
          }
        }
      }
    ]
  },
  "requirements": [
    {
      "id": "REQ-123",
      "description": "As a business analyst, I need real-time sales analytics",
      "priority": "high",
      "status": "approved",
      "changes": ["CHG-001"]
    },
    {
      "id": "REQ-124",
      "description": "Integrate payment gateway",
      "priority": "high",
      "status": "approved",
      "changes": ["CHG-002"]
    }
  ]
}
```

## Key Design Decisions

### 1. Change Annotations on Elements

**Why**: Elements directly know what change they're part of
**How**: `change` block in element metadata

```sruja
container AnalyticsAPI {
  change {
    id "CHG-001"
    requirement "REQ-123"
    scope "container"
  }
}
```

### 2. Requirements Separate from Architecture

**Why**: Requirements can be defined independently, linked to architecture
**How**: Separate requirements files, referenced by change annotations

```sruja
// requirements/analytics.sruja
requirement "REQ-123" functional "..."

// architecture/ecommerce-platform.sruja
container AnalyticsAPI {
  change {
    requirement "REQ-123"  // Reference
  }
}
```

### 3. Change Scope Tracking

**Why**: Understand what level is changing (Architecture/System/Container/Component)
**How**: `scope` field in change metadata

```json
{
  "change": {
    "scope": "container"  // Architecture | System | Container | Component
  }
}
```

### 4. Status Tracking

**Why**: Track change progress through lifecycle
**How**: `status` field in change metadata

```json
{
  "change": {
    "status": "planned"  // planned | in-progress | completed | cancelled
  }
}
```

## Visualization Needs

Studio/Viewer should support:

1. **Filter by change**: Show only elements with specific change ID
2. **Filter by requirement**: Show all elements related to a requirement
3. **Change scope highlighting**: Different visual treatment for Architecture/System/Container/Component changes
4. **Status visualization**: Color-code by change status (planned/in-progress/completed)
5. **Change timeline**: Show architecture evolution over versions
6. **Impact view**: Show all elements affected by a change

## Benefits

✅ **Traceability**: Link architecture changes to requirements/user stories  
✅ **Change visibility**: See what's changing and why  
✅ **Scope clarity**: Understand change level (Architecture/System/Container)  
✅ **Status tracking**: Track change progress  
✅ **Evolution view**: See architecture evolution over time  
✅ **Impact analysis**: Understand what elements are affected  
✅ **Requirement coverage**: See which requirements are addressed by architecture changes  

## Next Steps

1. ✅ Define change annotation syntax in DSL
2. ✅ Define requirements/user stories syntax (extend existing Requirement)
3. ⏳ Update JSON schema to include change metadata
4. ⏳ Update exporter/importer to handle changes
5. ⏳ Design Studio UI for change visualization
6. ⏳ Design change filtering and querying

## References

- [Architecture Changes Detailed Design](architecture-changes.md) - Complete design document
- [Architecture Isolation](architecture-isolation.md) - Architecture independence model
- [Architecture Model](architecture-model.md) - Overall architecture model

