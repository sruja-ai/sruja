# Architecture Model Simplification Summary

## The Problem with Current Complexity

**Current issues:**
- Single vs multiple files creates confusion
- Complex import mechanisms
- Partial files, shared elements - too many concepts
- Requirements nested in elements - complex hierarchies
- Organization complexity doesn't add value

**User's insight**: 
- Organization is just organization - users shouldn't care
- Large architectures are visualization problems, not structure problems
- Keep it simple like database migrations
- Use simple tagging, not complex hierarchies

## Proposed Simplified Model

### 1. Migration-Based Changes (Like DB Migrations)

**Linear history with snapshots:**

```
architecture/
  ├── v1.0.0.sruja              # Initial snapshot
  ├── migrations/
  │   ├── 001-add-analytics.sruja
  │   ├── 002-add-payment.sruja
  │   └── 003-add-inventory.sruja
  └── current.sruja             # Current state (generated)
```

**Key concept**: Each migration is a change set that modifies the architecture. At any point, we can generate a snapshot showing the current state.

### 2. Simple Tagging (Not Nested Structures)

**Requirements, User Stories, ADRs, Scenarios use tags to link elements:**

```sruja
// Elements (simple structure)
system ShopSystem {
  container AnalyticsAPI {}
}

// Requirements (tagged, not nested)
requirement "REQ-123" "Analytics dashboard" {
  tags ["ShopSystem", "AnalyticsAPI"]
}

// User Stories (tagged, mostly container-level)
userStory "US-456" "View sales metrics" {
  tags ["AnalyticsAPI"]  // Container level
}

// ADRs (tagged decisions)
adr "ADR-001" "Use REST API" {
  tags ["AnalyticsAPI"]
  status "decided"
}

// Scenarios (tagged behavioral descriptions)
scenario "High Traffic" {
  tags ["ShopSystem", "AnalyticsAPI"]
  description "System behavior during high traffic"
}
```

### 3. File Organization (Two Simple Options)

**Option 1: Single file with clear sections:**
```
workspace/
  └── ecommerce-platform.sruja    # Everything in one file
```

**Option 2: Concept-based files (standard names):**
```
workspace/
  ├── architecture.sruja          # Architecture structure
  ├── requirements.sruja          # Requirements
  ├── decisions.sruja             # ADRs
  ├── stories.sruja               # User stories
  └── scenarios.sruja             # Scenarios
```

**Rules**:
- ✅ Architecture stays in ONE file
- ✅ Concept-based files use standard names (`requirements.sruja`, `decisions.sruja`, etc.)
- ❌ NO splitting architecture across multiple files
- ❌ NO custom file names

### 4. Shared Services Use Naming Convention (No Imports)

**No imports needed** - use naming convention:

```sruja
// shared/auth-service.sruja
architecture "Auth Service" {
  system AuthAPI {}
}

// architecture/ecommerce-platform.sruja
architecture "E-commerce Platform" {
  system ShopSystem {}
  
  // Reference shared service using naming convention
  external shared.AuthService "Auth Service"  // Convention: shared.ServiceName
  ShopSystem -> shared.AuthService "Uses"
}
```

**Convention**:
- Shared services in `shared/` directory
- Reference using `shared.ServiceName` (no import needed)
- Parser automatically resolves shared services from `shared/` directory
- Simple and clear - no import statements

### 5. Large Architectures = Visualization Problem

**Solution**: UI handles collapse/expand, not complex file structures

```
ShopSystem
  ├─ WebApp (collapsed)
  ├─ API (collapsed)
  └─ AnalyticsAPI (expanded)
      ├─ MetricsCollector
      └─ DashboardAPI
```

## Complete Example

### Architecture Structure

```sruja
// architecture/ecommerce-platform.sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp {}
    container API {}
    container AnalyticsAPI {
      component MetricsCollector {}
      component DashboardAPI {}
    }
    datastore Database {}
  }
  
  system PaymentSystem {
    container PaymentAPI {}
  }
  
  ShopSystem.API -> PaymentSystem.PaymentAPI "Processes payments"
  ShopSystem.WebApp -> ShopSystem.AnalyticsAPI "Sends metrics"
}
```

### Requirements (Tagged)

```sruja
// requirements/analytics.sruja
requirement "REQ-123" "As a business analyst, I need real-time analytics" {
  tags ["ShopSystem", "AnalyticsAPI"]
  priority "high"
  status "approved"
}
```

### User Stories (Tagged, Container-Level)

```sruja
// requirements/user-stories.sruja
userStory "US-456" "View daily sales metrics on dashboard" {
  tags ["AnalyticsAPI"]  // Container level - business users understand
  requirement "REQ-123"
  acceptanceCriteria [
    "Dashboard shows real-time metrics"
    "Metrics update every 5 minutes"
  ]
}

// Later can tag components for technical details
userStory "US-457" "Export analytics data" {
  tags ["AnalyticsAPI", "AnalyticsAPI.MetricsCollector"]  // Container + Component
  requirement "REQ-123"
}
```

### ADRs (Tagged Decisions)

```sruja
// decisions/adr-001.sruja
adr "ADR-001" "Use REST API for analytics data access" {
  tags ["AnalyticsAPI"]
  status "decided"
  context "Need to provide analytics data to external systems"
  decision "Use REST API with OAuth2"
  consequences [
    "Standard protocol - easy integration"
    "Requires authentication layer"
  ]
}

// Pending decisions
adr "ADR-002" "Choose database for analytics" {
  tags ["AnalyticsAPI"]
  status "pending"
  context "Need to store large volumes of analytics data"
}
```

### Scenarios (Tagged Behavioral Descriptions)

```sruja
// scenarios/high-traffic.sruja
scenario "High Traffic" {
  tags ["ShopSystem", "AnalyticsAPI"]
  description "System behavior during high traffic (Black Friday)"
  
  conditions [
    "10x normal traffic"
    "All systems operational"
  ]
  
  behavior [
    "AnalyticsAPI uses caching to reduce database load"
    "ShopSystem.API throttles requests to PaymentSystem"
    "Metrics collection continues but with reduced granularity"
  ]
}

scenario "Rollout" {
  tags ["AnalyticsAPI"]
  description "Analytics API rollout strategy"
  
  behavior [
    "Deploy to 10% of traffic initially"
    "Monitor error rates and performance"
    "Gradually increase to 100%"
  ]
}

scenario "Disaster Recovery" {
  tags ["ShopSystem", "PaymentSystem"]
  description "System behavior during disaster"
  
  behavior [
    "Analytics API fails gracefully (non-critical)"
    "ShopSystem continues without analytics"
    "PaymentSystem has redundant failover"
  ]
}
```

### Migration (Change Tracking)

```sruja
// migrations/001-add-analytics.sruja
migration "001-add-analytics" {
  version "v1.1.0"
  requirement "REQ-123"
  
  add {
    system ShopSystem {
      container AnalyticsAPI "Analytics API" {
        component MetricsCollector {}
        component DashboardAPI {}
      }
    }
    relation ShopSystem.WebApp -> ShopSystem.AnalyticsAPI "Sends metrics"
  }
}
```

## Key Simplifications

### ✅ Removed Complexity

1. **No architecture splitting**: Architecture stays in ONE file - no splitting into multiple files
2. **No nested requirements**: Just tags to link
3. **No partial files concept**: Concept-based files only (requirements, decisions, stories, scenarios)
4. **No import mechanism at all**: Shared services use naming convention (`shared.ServiceName`)
5. **No custom file names**: Standard names only (`architecture.sruja`, `requirements.sruja`, etc.)

### ✅ Added Simplicity

1. **Migration-based changes**: Linear history, clear snapshots
2. **Simple tagging**: Link requirements/stories/ADRs/scenarios to elements
3. **Transparent organization**: Users don't need to care about file structure
4. **Clear separation**: Architecture structure vs requirements/decisions/scenarios

## Benefits

✅ **Simple**: No complex hierarchies  
✅ **Clear**: Linear change history  
✅ **Visual**: Large architectures handled by UI  
✅ **Traceable**: Tags link everything  
✅ **Flexible**: Organization doesn't matter  
✅ **No imports**: Shared services use naming convention (`shared.ServiceName`)  
✅ **Opinionated**: Clear model, less confusion  

## Migration from Current Model

**Old way** (complex):
```sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    requirements {
      requirement "REQ-123" {}
    }
    container AnalyticsAPI {
      requirements {
        requirement "REQ-124" {}
      }
    }
  }
}
```

**New way** (simple):
```sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container AnalyticsAPI {}
  }
}

requirement "REQ-123" {
  tags ["ShopSystem"]
}

requirement "REQ-124" {
  tags ["AnalyticsAPI"]
}
```

## Visualization Support Needed

Studio/Viewer should support:

1. **Collapse/Expand**: Handle large architectures visually
2. **Tag filtering**: Show all elements with specific tags
3. **Requirement view**: Show all requirements for an element
4. **Timeline view**: Show architecture evolution through migrations
5. **Scenario view**: Show scenarios affecting an element
6. **ADR view**: Show decisions for an element

## Next Steps

1. ✅ Define simplified architecture model (this document)
2. ⏳ Define migration DSL syntax
3. ⏳ Define tagging model
4. ⏳ Define snapshot generation
5. ⏳ Simplify parser (remove complex import/organization logic)
6. ⏳ Design Studio visualization

