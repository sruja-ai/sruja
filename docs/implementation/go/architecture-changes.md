# Modeling Architectural Changes and Requirements

## Core Principle: Architectural Change vs Implementation Change

**Architectural changes modify the STRUCTURE of the architecture:**
- ✅ Adding NEW elements (systems, containers, components)
- ✅ Removing/deprecating elements
- ✅ Creating new versions (deprecating old + adding new)
- ✅ Adding/removing relations
- ✅ Structural modifications

**Implementation changes modify the BEHAVIOR/CODE of existing elements:**
- ❌ Changing existing component implementation
- ❌ Performance optimizations (unless requires new structure)
- ❌ Bug fixes, refactoring
- ❌ Feature additions within existing structure

**Key Rule**: 
- **NEW component** = ✅ Architectural change (track in architecture)
- **Changing existing component** = ❌ NOT architectural change (track in issue tracker)
- **Deprecating + new version** = ✅ Architectural change (track in architecture)

## The Problem

When designing architecture changes, we need to model:

1. **What is changing?** (Change scope)
   - Architecture level (greenfield - initial design)
   - Container/System level (common - evolution)
   - Component level (rare - team-level, not architecture-level)

2. **Why is it changing?** (Change drivers)
   - New requirements
   - User stories
   - Business needs
   - Technical constraints

3. **How is it changing?** (Change details)
   - New systems/containers/components
   - Modified systems/containers/components
   - New relations
   - Removed elements

4. **Context**: When, who, status

## What IS an Architectural Change?

**Key Principle**: Architectural changes modify the **structure** of the architecture, not implementation details.

### ✅ IS an Architectural Change

1. **Adding new elements**: New systems, containers, or components
2. **Removing elements**: Deprecating/deleting systems, containers, or components
3. **Creating new versions**: Deprecating old version, creating new version of an element
4. **Adding new relations**: New interactions between elements
5. **Removing relations**: Removing interactions
6. **Structural modifications**: Changing element boundaries, splitting/merging elements

### ❌ NOT an Architectural Change

1. **Modifying existing component implementation**: Changing code, algorithms, internal logic
2. **Performance optimizations**: Internal improvements without structure changes
3. **Bug fixes**: Correcting implementation issues
4. **Refactoring**: Code restructuring without architectural impact
5. **Configuration changes**: Settings, parameters, tuning

### Important Distinction

**New Component** = ✅ Architectural Change
```sruja
container WebApp {
  component ShoppingCart {}      // Existing
  component RecommendationEngine {}  // NEW - architectural change
}
```

**Changing Existing Component** = ❌ NOT Architectural Change
```sruja
component ShoppingCart {
  // Modifying internal implementation
  // Adding new methods, changing algorithms
  // This is implementation-level, not architectural
}
```

**Deprecating + New Version** = ✅ Architectural Change
```sruja
container WebApp {
  component ShoppingCart {
    deprecation {
      reason "Replaced by ShoppingCartV2"
      replacedBy "ShoppingCartV2"
    }
  }
  component ShoppingCartV2 {  // NEW - architectural change (deprecation + replacement)
    change {
      id "CHG-001"
      replaces "ShoppingCart"
    }
  }
}
```

## Change Levels

### Level 1: Architecture Level (Greenfield)

**When**: Initial architecture design
**Scope**: Entire architecture
**Frequency**: Once per architecture

```sruja
architecture "E-commerce Platform" {
  // Initial design - whole architecture
  system ShopSystem {}
  system PaymentSystem {}
}
```

### Level 2: Container/System Level (Evolution)

**When**: Architecture exists, needs evolution
**Scope**: Specific containers or systems
**Frequency**: Common - most changes happen here

```sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp {}  // Existing
    container API {}     // Existing
    container AnalyticsAPI {}  // NEW - architectural change (adding new container)
  }
}
```

### Level 3: Component Level (New Components Only)

**When**: Adding new components (not modifying existing)
**Scope**: New components only
**Frequency**: Less common - adding new capabilities

**✅ Architectural Change**: Adding new component
```sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp {
      component ShoppingCart {}      // Existing
      component RecommendationEngine {}  // NEW - architectural change (new component)
    }
  }
}
```

**❌ NOT Architectural Change**: Modifying existing component
```sruja
component ShoppingCart {
  // Changing implementation details
  // Adding features, fixing bugs
  // This is implementation-level, tracked in issue tracker, not architecture
}
```

## Change Model

### Concept: Change Scope Annotation

Elements can be annotated to indicate they are part of a change:

```sruja
change "Add Analytics Dashboard" {
  requirement "REQ-123" "As a business analyst, I need real-time sales analytics"
  userStory "US-456" "View daily sales metrics on dashboard"
  
  // Change scope: Container level
  scope {
    system ShopSystem {
      container AnalyticsAPI "Analytics API" {  // NEW
        component MetricsCollector {}
        component DashboardAPI {}
      }
    }
  }
  
  // Impact: Relations
  impact {
    ShopSystem.WebApp -> ShopSystem.AnalyticsAPI "Sends metrics"
    ShopSystem.AnalyticsAPI -> ShopSystem.Database "Queries"
  }
}
```

### Concept: Requirements and User Stories

Link architectural changes to requirements:

```sruja
requirement "REQ-123" "As a business analyst, I need real-time sales analytics" {
  priority "high"
  status "in-progress"
  
  userStories [
    "US-456" "View daily sales metrics"
    "US-457" "Export analytics data"
  ]
  
  changes [
    "Add Analytics Dashboard"
  ]
}
```

## Proposed DSL Syntax

### Option 1: Change Blocks (Explicit Changes)

```sruja
architecture "E-commerce Platform" {
  // Existing architecture
  system ShopSystem {
    container WebApp {}
    container API {}
    datastore Database {}
  }
  
  // Change: Add Analytics
  change "Add Analytics Dashboard" {
    requirement "REQ-123"
    userStory "US-456"
    
    // What's changing
    add {
      system ShopSystem {
        container AnalyticsAPI "Analytics API" {
          component MetricsCollector {}
        }
      }
      relation ShopSystem.WebApp -> ShopSystem.AnalyticsAPI "Sends metrics"
    }
    
    // Optional: What's being modified
    modify {
      container ShopSystem.WebApp {
        // Add new component
        component MetricsSender {}
      }
    }
    
    // Optional: What's being removed
    remove {
      // component ShopSystem.WebApp.OldAnalytics {}
    }
  }
}
```

### Option 2: Annotations on Elements (Implicit Changes)

```sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp {}
    container API {}
    
    // Change annotation on new element
    container AnalyticsAPI "Analytics API" {
      change "Add Analytics Dashboard" {
        requirement "REQ-123"
        userStory "US-456"
        status "planned"
      }
      
      component MetricsCollector {}
    }
  }
}
```

### Option 3: Separate Change File (Change Tracking)

```sruja
// changes/add-analytics.sruja
change "Add Analytics Dashboard" {
  requirement "REQ-123" "As a business analyst, I need real-time sales analytics"
  userStory "US-456" "View daily sales metrics"
  
  architecture "E-commerce Platform"
  
  scope "container"  // Architecture | System | Container | Component
  
  add {
    system ShopSystem {
      container AnalyticsAPI "Analytics API" {
        component MetricsCollector {}
      }
    }
    relation ShopSystem.WebApp -> ShopSystem.AnalyticsAPI "Sends metrics"
  }
}
```

### Option 4: Requirements-First Model

```sruja
// requirements/analytics-requirements.sruja
requirement "REQ-123" "As a business analyst, I need real-time sales analytics" {
  priority "high"
  status "in-progress"
  
  userStories [
    "US-456" "View daily sales metrics"
    "US-457" "Export analytics data"
  ]
}

// architecture/ecommerce-platform.sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp {}
    
    // Link to requirement
    container AnalyticsAPI "Analytics API" {
      requirement "REQ-123"
      component MetricsCollector {}
    }
  }
}
```

## Recommended Approach: Hybrid Model

Combine **annotations** (for change tracking) with **requirements** (for traceability):

### 1. Requirements File (Separate)

```sruja
// requirements/analytics-requirements.sruja
requirement "REQ-123" "As a business analyst, I need real-time sales analytics" {
  priority "high"
  status "in-progress"
  epic "Analytics Features"
  
  userStories [
    "US-456" "View daily sales metrics on dashboard"
    "US-457" "Export analytics data to CSV"
  ]
}

userStory "US-456" "View daily sales metrics on dashboard" {
  requirement "REQ-123"
  acceptanceCriteria [
    "Dashboard shows real-time sales metrics"
    "Metrics update every 5 minutes"
  ]
}
```

### 2. Architecture File with Change Annotations

```sruja
// architecture/ecommerce-platform.sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp {}
    container API {}
    
    // Change annotation on new/modified elements
    container AnalyticsAPI "Analytics API" {
      change {
        id "CHG-001"
        name "Add Analytics Dashboard"
        requirement "REQ-123"
        userStory "US-456"
        scope "container"  // Architecture | System | Container | Component
        status "planned"   // planned | in-progress | completed | cancelled
        version "v2.1.0"
      }
      
      component MetricsCollector {}
      component DashboardAPI {}
    }
  }
  
  // Change annotation on new relations
  relation ShopSystem.WebApp -> ShopSystem.AnalyticsAPI "Sends metrics" {
    change {
      id "CHG-001"
      requirement "REQ-123"
    }
  }
}
```

### 3. Change Summary (Optional Workspace View)

```sruja
// workspace/changes-summary.sruja
changes {
  change "CHG-001" "Add Analytics Dashboard" {
    requirement "REQ-123"
    architecture "E-commerce Platform"
    scope "container"
    status "planned"
    
    affectedElements [
      "ShopSystem.AnalyticsAPI"
      "ShopSystem.WebApp -> ShopSystem.AnalyticsAPI"
    ]
  }
}
```

## JSON Structure for Changes

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
        "status": "planned",
        "version": "v2.1.0"
      }
    ]
  },
  "architecture": {
    "containers": [
      {
        "id": "ShopSystem.AnalyticsAPI",
        "label": "Analytics API",
        "metadata": {
          "sourceFile": "architecture/ecommerce-platform.sruja",
          "change": {
            "id": "CHG-001",
            "requirement": "REQ-123",
            "userStory": "US-456",
            "scope": "container",
            "status": "planned"
          }
        }
      }
    ],
    "relations": [
      {
        "from": "ShopSystem.WebApp",
        "to": "ShopSystem.AnalyticsAPI",
        "label": "Sends metrics",
        "metadata": {
          "change": {
            "id": "CHG-001",
            "requirement": "REQ-123"
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
      "status": "in-progress",
      "userStories": ["US-456", "US-457"],
      "changes": ["CHG-001"]
    }
  ]
}
```

## Real-World Examples

### Example 1: Adding New Container (Container-Level Change)

**Requirement**: "As a developer, I need to integrate payment gateway"

**Change**:
```sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp {}
    container API {}
    
    // NEW: Payment Gateway integration
    container PaymentGateway "Payment Gateway Client" {
      change {
        id "CHG-002"
        requirement "REQ-124"
        scope "container"
        status "in-progress"
      }
      
      component PaymentProcessor {}
      component GatewayAdapter {}
    }
  }
  
  relation ShopSystem.API -> ShopSystem.PaymentGateway "Processes payments" {
    change {
      id "CHG-002"
    }
  }
}
```

### Example 2: Modifying Existing System (System-Level Change)

**Requirement**: "As a business owner, I need inventory management"

**Change**:
```sruja
architecture "E-commerce Platform" {
  system ShopSystem {}  // Existing
  system PaymentSystem {}  // Existing
  
  // NEW: Inventory system
  system InventorySystem "Inventory Management" {
    change {
      id "CHG-003"
      requirement "REQ-125"
      scope "system"
      status "planned"
    }
    
    container InventoryAPI {}
    datastore InventoryDB {}
  }
  
  relation ShopSystem -> InventorySystem "Checks stock" {
    change {
      id "CHG-003"
    }
  }
}
```

### Example 3: Adding New Component (Architectural Change)

**Requirement**: "Add recommendation engine to shopping experience"

**Change**: Adding NEW component is architectural change
```sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp {
      component ShoppingCart {}  // Existing
      
      // NEW: Adding new component - architectural change
      component RecommendationEngine {
        change {
          id "CHG-004"
          requirement "REQ-126"
          scope "component"
          status "in-progress"
        }
      }
    }
  }
}
```

### Example 4: Modifying Existing Component (NOT Architectural Change)

**Requirement**: "Improve shopping cart performance"

**Implementation Change**: Modifying existing component is NOT architectural change
```sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp {
      component ShoppingCart {
        // This is implementation-level change
        // Performance optimization, bug fixes, feature additions
        // Tracked in issue tracker, not in architecture
        // NO change annotation - this is not architectural change
      }
    }
  }
}
```

**Note**: If performance improvement requires NEW component (like a cache), then adding the new component IS an architectural change:
```sruja
container WebApp {
  component ShoppingCart {}  // Existing (implementation improvements tracked elsewhere)
  
  // NEW component - this IS architectural change
  component CartCache {
    change {
      id "CHG-005"
      requirement "REQ-127"
      scope "component"
    }
  }
}
```

### Example 5: Deprecating Component + New Version (Architectural Change)

**Requirement**: "Redesign shopping cart with new architecture"

**Change**: Deprecating old version + creating new version
```sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp {
      component ShoppingCart {
        deprecation {
          reason "Replaced by ShoppingCartV2 with improved architecture"
          replacedBy "ShoppingCartV2"
        }
      }
      
      // NEW version - architectural change
      component ShoppingCartV2 {
        change {
          id "CHG-006"
          requirement "REQ-128"
          scope "component"
          replaces "ShoppingCart"
        }
      }
    }
  }
}
```

## Change Tracking Workflow

### 1. Requirements Phase

```sruja
// requirements/epic-2.sruja
requirement "REQ-123" "Analytics Dashboard" {
  priority "high"
  status "approved"
  
  userStories [
    "US-456" "View metrics"
    "US-457" "Export data"
  ]
}
```

### 2. Architecture Design Phase

```sruja
// architecture/ecommerce-platform.sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container AnalyticsAPI {
      change {
        id "CHG-001"
        requirement "REQ-123"
        scope "container"
        status "planned"
      }
    }
  }
}
```

### 3. Implementation Phase

```sruja
// Update status as work progresses
container AnalyticsAPI {
  change {
    status "in-progress"  // Changed from "planned"
  }
}
```

### 4. Completion Phase

```sruja
container AnalyticsAPI {
  change {
    status "completed"
    completedDate "2025-02-15"
  }
}
```

## Visualization

Studio/Viewer should be able to:
1. **Filter by change**: Show only elements with specific change ID
2. **Filter by requirement**: Show all elements related to a requirement
3. **Show change status**: Color-code elements by change status
4. **Show change scope**: Highlight architecture/system/container/component level changes
5. **Change timeline**: Show evolution of architecture over time

## Benefits

✅ **Traceability**: Link architecture changes to requirements/user stories  
✅ **Change visibility**: See what's changing and why  
✅ **Scope clarity**: Understand change level (architecture/container/system)  
✅ **Status tracking**: Track change progress  
✅ **Evolution view**: See architecture evolution over time  
✅ **Impact analysis**: Understand what elements are affected  

## Next Steps

1. Define DSL syntax for change annotations
2. Define DSL syntax for requirements/user stories
3. Update JSON schema to include change metadata
4. Update exporter/importer to handle changes
5. Design Studio UI for change visualization
6. Design change filtering and querying

