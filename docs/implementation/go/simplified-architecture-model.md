# Simplified Architecture Model: Change-Based with Linear History

## Core Philosophy

**Keep it simple and opinionated:**
- Like database migrations: linear history of changes, snapshots at any point (we call them "changes")
- Single vs multiple files is just **organization** - not a structural concern
- Large architectures are a **visualization problem** (collapse/expand), not a structure problem
- Use **simple tagging** for linking requirements/user stories/ADRs to elements
- Focus on what users actually need: tracking changes, requirements, decisions, scenarios

## Key Simplifications

### 1. Organization is Just Organization

**Simplified View**: Two options - single file OR concept-based files

```
// Option 1: Single file with clear sections
workspace/
  └── ecommerce-platform.sruja    # Everything in one file

// Option 2: Concept-based files (standard names)
workspace/
  ├── architecture.sruja          # Architecture structure
  ├── requirements.sruja          # Requirements
  ├── decisions.sruja             # ADRs
  ├── stories.sruja               # User stories
  └── scenarios.sruja             # Scenarios
```

**Rules**:
- ✅ Architecture stays in ONE file (`architecture.sruja` or single file)
- ✅ Concept-based files: `requirements.sruja`, `decisions.sruja`, `stories.sruja`, `scenarios.sruja`
- ❌ NO splitting architecture across multiple files
- ❌ NO custom file names - follow standard structure
- ✅ Shared services use naming convention (`shared.ServiceName`)

### 2. Change-Based Model

Like database migrations - linear history with snapshots:

```
architecture/
  ├── v1.0.0.sruja              # Initial snapshot
  ├── changes/
  │   ├── 001-add-analytics.sruja
  │   ├── 002-add-payment.sruja
  │   └── 003-add-inventory.sruja
  └── current.sruja             # Current state (generated from changes)
```

**Key Concept**: Each change is a **change set** that modifies the architecture.

### 3. Simple Tagging Model (Key-Value Pairs)

**No complex hierarchies** - just key-value tags to link things. Tags can have multiple values:

```sruja
// Requirements linked via tags (can link to multiple systems/containers/components)
requirement "REQ-123" "Analytics dashboard" {
  tags [
    system "ShopSystem"
    system "InventorySystem"
    container "ShopSystem.AnalyticsAPI"
  ]
  priority "high"
}

// Requirement affecting multiple containers/components
requirement "REQ-124" "Secure all payment endpoints" {
  tags [
    container "PaymentSystem.PaymentAPI"
    container "PaymentSystem.Gateway"
    component "PaymentSystem.PaymentAPI.PaymentProcessor"
    component "PaymentSystem.PaymentAPI.RefundHandler"
  ]
  priority "critical"
}

// User stories linked via tags (mostly container-level)
userStory "US-456" "View sales metrics" {
  tags [
    container "ShopSystem.AnalyticsAPI"
  ]
  requirement "REQ-123"
}

// User story spanning multiple containers
userStory "US-457" "Complete order with payment" {
  tags [
    container "ShopSystem.API"
    container "PaymentSystem.PaymentAPI"
  ]
  requirement "REQ-124"
}

// ADRs linked via tags
adr "ADR-001" "Use REST API for analytics" {
  tags [
    container "ShopSystem.AnalyticsAPI"
  ]
  status "decided"
}

// ADR affecting multiple systems
adr "ADR-002" "Use OAuth2 for all services" {
  tags [
    system "ShopSystem"
    system "PaymentSystem"
    system "InventorySystem"
  ]
  status "decided"
}

// Scenarios linked via tags
scenario "High Traffic" {
  tags [
    system "ShopSystem"
    system "PaymentSystem"
    container "ShopSystem.AnalyticsAPI"
  ]
  description "System behavior during high traffic"
}
```

### 4. Elements Don't Nest Requirements

**Instead**: Requirements tag the elements they relate to.

```sruja
// ❌ Complex nested structure
system ShopSystem {
  requirements {
    requirement "REQ-123" {}
  }
}

// ✅ Simple tagging with key-value pairs
system ShopSystem {}
requirement "REQ-123" {
  tags [
    system "ShopSystem"
  ]
}
```

## Change-Based Model

### Concept: Linear Change History

Each change creates/modifies/removes elements:

```sruja
// changes/001-add-analytics.sruja
change "001-add-analytics" {
  version "v1.1.0"
  requirement "REQ-123"
  
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

### Concept: Snapshots

At any point, we can generate a snapshot (current state):

```sruja
// Generated: current.sruja
architecture "E-commerce Platform" {
  // All elements after all changes applied
  system ShopSystem {
    container WebApp {}
    container API {}
    container AnalyticsAPI {}  // Added in change 001
  }
}
```

### Concept: Change History

Track changes over time:

```
Timeline:
  v1.0.0 (2025-01-01) - Initial architecture
  v1.1.0 (2025-02-01) - Added analytics (change 001)
  v1.2.0 (2025-03-01) - Added payment (change 002)
  v1.3.0 (2025-04-01) - Added inventory (change 003)
```

## Simplified DSL Structure

### Core Architecture Elements

```sruja
architecture "E-commerce Platform" {
  // Just the structure - simple
  system ShopSystem {
    container WebApp {}
    container API {}
    datastore Database {}
  }
  
  system PaymentSystem {
    container PaymentAPI {}
  }
  
  // Relations
  ShopSystem.API -> PaymentSystem.PaymentAPI "Processes payments"
}
```

### Requirements (Tagged, Not Nested)

```sruja
// requirements/analytics.sruja
requirement "REQ-123" "As a business analyst, I need real-time analytics" {
  tags ["ShopSystem", "AnalyticsAPI"]
  priority "high"
  status "approved"
}
```

### User Stories (Tagged, Mostly Container-Level)

```sruja
// requirements/user-stories.sruja
userStory "US-456" "View daily sales metrics on dashboard" {
  tags ["AnalyticsAPI"]  // Container level - business users understand this
  requirement "REQ-123"
  acceptanceCriteria [
    "Dashboard shows real-time metrics"
    "Metrics update every 5 minutes"
  ]
}

// Later can tag with components for technical tracking
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
  decision "Use REST API with OAuth2 authentication"
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
  options [
    "PostgreSQL - familiar, good performance"
    "ClickHouse - optimized for analytics"
  ]
}
```

### Scenarios (Tagged Behavioral Descriptions)

```sruja
// scenarios/high-traffic.sruja
scenario "High Traffic" {
  tags ["ShopSystem", "AnalyticsAPI"]
  description "System behavior during high traffic periods (Black Friday)"
  
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
  description "System behavior during disaster scenario"
  
  behavior [
    "Analytics API fails gracefully (non-critical)"
    "ShopSystem continues without analytics"
    "PaymentSystem has redundant failover"
  ]
}
```

## File Organization (Simple)

### Option 1: Single File with Clear Sections

**Everything in one file with clear sections:**

```sruja
// ecommerce-platform.sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp {}
    container API {}
  }
  system PaymentSystem {}
  
  // Relations (always inside architecture block)
  ShopSystem.API -> PaymentSystem "Uses"
  
  // Flows (always inside architecture block)
  flow OrderFlow "Order Processing" {
    Customer -> ShopSystem.WebApp "Submits order"
    ShopSystem.WebApp -> ShopSystem.API "Sends order"
    ShopSystem.API -> PaymentSystem "Processes payment"
  }
}

// Requirements section
requirement "REQ-123" "Analytics dashboard" {
  tags [
    system "ShopSystem"
  ]
  priority "high"
}

// Decisions section (ADRs)
adr "ADR-001" "Use REST API" {
  tags [
    container "ShopSystem.API"
  ]
  status "decided"
}

// User Stories section
userStory "US-456" "View metrics" {
  tags [
    container "ShopSystem.API"
  ]
  requirement "REQ-123"
}

// Scenarios section
scenario "High Traffic" {
  tags [
    system "ShopSystem"
  ]
  description "System behavior during high traffic"
}
```

### Option 2: Concept-Based Files (Organized by Type)

**Standard File Structure**:
```
workspace/
  ├── architecture.sruja      # Architecture structure (elements, relations, flows)
  ├── requirements.sruja      # Requirements
  ├── decisions.sruja         # ADRs (decisions)
  ├── stories.sruja           # User stories
  └── scenarios.sruja         # Scenarios
```

**OR single file with clear sections**:
```
workspace/
  └── ecommerce-platform.sruja  # Everything in one file
```

**Standard File Names** (not custom):
- `architecture.sruja` - Architecture structure (elements, relations, flows)
- `requirements.sruja` - Requirements
- `decisions.sruja` - ADRs
- `stories.sruja` - User stories
- `scenarios.sruja` - Scenarios

**Example: Concept-Based Files**:

**architecture.sruja** - All architecture structure in one file:
```sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp "Web Application" {
      component ShoppingCart "Shopping Cart" {}
      component ProductCatalog "Product Catalog" {}
      component Checkout "Checkout" {}
    }
    container API "API Service" {
      component OrderService "Order Service" {}
      component ProductService "Product Service" {}
      component UserService "User Service" {}
    }
    datastore Database "Database" {}
  }
  
  system PaymentSystem {
    container PaymentAPI "Payment API" {
      component PaymentProcessor "Payment Processor" {}
      component RefundHandler "Refund Handler" {}
    }
    container Gateway "Payment Gateway" {}
  }
  
  // Relations (always inside architecture block)
  ShopSystem.WebApp -> ShopSystem.API "Calls"
  ShopSystem.API -> PaymentSystem.PaymentAPI "Processes payments"
  ShopSystem -> PaymentSystem "Uses"
  
  // Flows (always inside architecture block)
  flow OrderFlow "Order Processing Flow" {
    Customer -> ShopSystem.WebApp "Submits order"
    ShopSystem.WebApp -> ShopSystem.API "Sends order request"
    ShopSystem.API -> PaymentSystem.PaymentAPI "Processes payment"
    ShopSystem.API -> ShopSystem.Database "Saves order"
  }
  
  flow PaymentFlow "Payment Processing Flow" {
    ShopSystem.API -> PaymentSystem.PaymentAPI "Payment request"
    PaymentSystem.PaymentAPI -> PaymentSystem.Gateway "Authorizes payment"
    PaymentSystem.Gateway -> PaymentSystem.PaymentAPI "Payment confirmation"
    PaymentSystem.PaymentAPI -> ShopSystem.API "Payment result"
  }
}
```

**requirements.sruja** - All requirements:
```sruja
requirement "REQ-123" "As a business analyst, I need real-time analytics" {
  tags [
    system "ShopSystem"
    container "ShopSystem.AnalyticsAPI"
  ]
  priority "high"
  status "approved"
}

requirement "REQ-124" "Payment processing must be secure" {
  tags [
    system "PaymentSystem"
  ]
  priority "critical"
  status "approved"
}
```

**decisions.sruja** - All ADRs:
```sruja
adr "ADR-001" "Use REST API for analytics data access" {
  tags [
    container "ShopSystem.AnalyticsAPI"
  ]
  status "decided"
  context "Need to provide analytics data to external systems"
  decision "Use REST API with OAuth2 authentication"
}

adr "ADR-002" "Choose database for analytics" {
  tags [
    container "ShopSystem.AnalyticsAPI"
  ]
  status "pending"
  context "Need to store large volumes of analytics data"
}
```

**stories.sruja** - All user stories:
```sruja
userStory "US-456" "View daily sales metrics on dashboard" {
  tags [
    container "ShopSystem.AnalyticsAPI"
  ]
  requirement "REQ-123"
  acceptanceCriteria [
    "Dashboard shows real-time metrics"
    "Metrics update every 5 minutes"
  ]
}
```

**scenarios.sruja** - All scenarios:
```sruja
scenario "High Traffic" {
  tags [
    system "ShopSystem"
    container "ShopSystem.AnalyticsAPI"
  ]
  description "System behavior during high traffic periods (Black Friday)"
  
  conditions [
    "10x normal traffic"
    "All systems operational"
  ]
  
  behavior [
    "AnalyticsAPI uses caching to reduce database load"
    "ShopSystem.API throttles requests to PaymentSystem"
  ]
}
```

**Key Rules**:
- ✅ Architecture stays in ONE file (`architecture.sruja` or in single file)
- ✅ Concept-based files use standard names: `requirements.sruja`, `decisions.sruja`, `stories.sruja`, `scenarios.sruja`
- ✅ OR everything in one file with clear sections
- ❌ NO splitting architecture across multiple files (no `main.sruja`, `systems.sruja`, `containers.sruja`)
- ❌ NO custom file names - follow standard structure

### Shared Services (Naming Convention)

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

## Change Example

### Initial Snapshot

```sruja
// v1.0.0.sruja (initial)
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp {}
    container API {}
    datastore Database {}
  }
}
```

### Change 1: Add Analytics

```sruja
// changes/001-add-analytics.sruja
change "001-add-analytics" {
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

### Generated Current Snapshot

```sruja
// current.sruja (generated from v1.0.0 + changes)
architecture "E-commerce Platform" {
  system ShopSystem {
    container WebApp {}
    container API {}
    container AnalyticsAPI "Analytics API" {  // Added
      component MetricsCollector {}
      component DashboardAPI {}
    }
    datastore Database {}
  }
  
  relation ShopSystem.WebApp -> ShopSystem.AnalyticsAPI "Sends metrics"
}
```

## Tagging Model

### Tags Link Everything (Multiple Values Supported)

**Key-Value tags can have single values or arrays (multiple values)**:

```sruja
// Elements
system ShopSystem {}
system PaymentSystem {}
container ShopSystem.AnalyticsAPI {}
container PaymentSystem.PaymentAPI {}

// Requirements (tag elements - can tag multiple systems/containers/components)
requirement "REQ-123" "Analytics dashboard" {
  tags [
    system "ShopSystem"
    container "ShopSystem.AnalyticsAPI"
  ]
}

// Requirement affecting multiple systems
requirement "REQ-124" "Secure all payment processing" {
  tags [
    system "ShopSystem"
    system "PaymentSystem"
    container "ShopSystem.API"
    container "PaymentSystem.PaymentAPI"
  ]
}

// Requirement affecting multiple components (fully qualified)
requirement "REQ-125" "Update all payment components" {
  tags [
    component "PaymentSystem.PaymentAPI.PaymentProcessor"
    component "PaymentSystem.PaymentAPI.RefundHandler"
    component "PaymentSystem.Gateway.GatewayAdapter"
  ]
}

// User Stories (tag containers - can tag multiple)
userStory "US-456" "View metrics" {
  tags [
    container "ShopSystem.AnalyticsAPI"
  ]
}

// User story spanning multiple containers
userStory "US-457" "Complete checkout flow" {
  tags [
    container "ShopSystem.WebApp"
    container "ShopSystem.API"
    container "PaymentSystem.PaymentAPI"
  ]
}

// ADRs (tag elements - can tag multiple)
adr "ADR-001" "Use REST API" {
  tags [
    container "ShopSystem.AnalyticsAPI"
  ]
}

// ADR affecting multiple systems
adr "ADR-002" "Use OAuth2 for all services" {
  tags [
    system "ShopSystem"
    system "PaymentSystem"
    system "InventorySystem"
  ]
}

// Scenarios (tag systems/containers - can tag multiple)
scenario "High Traffic" {
  tags [
    system "ShopSystem"
    system "PaymentSystem"
    container "ShopSystem.AnalyticsAPI"
  ]
}
```

### Querying by Tags

Studio/Viewer can:
- Show all requirements for a system/container/component (even if requirement tags multiple elements)
- Show all user stories for a container (even if story spans multiple containers)
- Show all ADRs for an element (even if ADR affects multiple elements)
- Show all scenarios affecting an element (even if scenario covers multiple elements)

**Query Examples**:
- Find all requirements affecting `ShopSystem`: Matches REQ-123, REQ-124 (both tag ShopSystem)
- Find all user stories for `PaymentSystem.PaymentAPI`: Matches US-457 (tags include PaymentSystem.PaymentAPI)
- Find all ADRs affecting `PaymentSystem`: Matches ADR-002 (tags include PaymentSystem)

## Relations vs Flows

**Key Rules**:
- ✅ **Relations** and **Flows** must always be inside the architecture block
- ✅ **Relations**: Simple connections between elements (`ShopSystem -> PaymentSystem`)
- ✅ **Flows**: Sequences of steps showing data/value flow through multiple elements

**Relations** (Simple connections):
```sruja
architecture "E-commerce Platform" {
  system ShopSystem {}
  system PaymentSystem {}
  
  // Relations inside architecture block
  ShopSystem -> PaymentSystem "Uses"
  ShopSystem.API -> PaymentSystem.PaymentAPI "Calls"
}
```

**Flows** (Sequences of steps):
```sruja
architecture "E-commerce Platform" {
  system ShopSystem {}
  system PaymentSystem {}
  
  // Flows inside architecture block
  flow OrderFlow "Order Processing Flow" {
    Customer -> ShopSystem.WebApp "Submits order"
    ShopSystem.WebApp -> ShopSystem.API "Sends order request"
    ShopSystem.API -> PaymentSystem.PaymentAPI "Processes payment"
    ShopSystem.API -> ShopSystem.Database "Saves order"
  }
}
```

**Difference**:
- **Relations**: Static connections showing "what connects to what"
- **Flows**: Dynamic sequences showing "how data/value flows through the system"

### Flows and User Stories Relationship

**Yes! Flows are like the architectural answer to user stories:**

- **User Stories**: Describe **WHAT** a user wants to do (business perspective)
  - "As a customer, I want to place an order"
  - "As a business analyst, I want to view sales metrics"

- **Flows**: Describe **HOW** the system processes that (technical perspective)
  - Shows the sequence of data/value flow through the system
  - Shows how user stories are implemented architecturally

**Example with Key-Value Tag Linking**:

```sruja
// stories.sruja
userStory "US-456" "Place an order" {
  tags [
    system "ShopSystem"
    flow "OrderFlow"  // Tag links to flow
  ]
  requirement "REQ-123"
  acceptanceCriteria [
    "Customer can submit order"
    "Order is saved to database"
    "Payment is processed"
  ]
}

// architecture.sruja
architecture "E-commerce Platform" {
  system ShopSystem {}
  system PaymentSystem {}
  
  // Flow shows HOW the user story is implemented
  // Tag links back to user story
  flow OrderFlow "Order Processing Flow" {
    tags [
      story "US-456"  // Link to user story via tag
    ]
    
    Customer -> ShopSystem.WebApp "Submits order"
    ShopSystem.WebApp -> ShopSystem.API "Sends order request"
    ShopSystem.API -> PaymentSystem.PaymentAPI "Processes payment"
    ShopSystem.API -> ShopSystem.Database "Saves order"
  }
}
```

**Key-Value Tag Linking Pattern**:
- **User Story** tags the flow: `tags [flow "OrderFlow"]` - "This story uses this flow"
- **Flow** tags the user story: `tags [story "US-456"]` - "This flow implements this story"
- **Bidirectional linking**: Easy to query both directions
  - "Show all flows for user story US-456" (query by `story "US-456"`)
  - "Show all user stories for flow OrderFlow" (query by `flow "OrderFlow"`)
- **Clear semantics**: Key-value pairs make it explicit what each tag represents

**Key Points**:
- ✅ **Key-value tag linking**: Flows and user stories link via key-value tags (bidirectional)
  - User story tags the flow: `tags [flow "OrderFlow"]`
  - Flow tags the user story: `tags [story "US-456"]`
- Flows show the **technical implementation** of how user stories flow through the system
- Multiple user stories might use the same flow (shared process)
- Flows show the **happy path** or **normal flow** of how something works
- **Querying**: Can find all flows for a story, or all stories for a flow using key-value queries

**Flows vs Scenarios**:
- **Flows**: Show normal flow - how things work in typical cases
- **Scenarios**: Show behavior under different conditions (high traffic, disaster, rollout, etc.)

## Benefits of Simplified Model

✅ **Simple**: No complex hierarchies, just tags  
✅ **Clear**: Linear change history  
✅ **Visual**: Large architectures handled by UI (collapse/expand)  
✅ **Traceable**: Tags link requirements/stories/ADRs to elements  
✅ **Flexible**: Organization doesn't matter to users  
✅ **No imports**: Shared services use naming convention (`shared.ServiceName`)  
✅ **Opinionated**: Clear model, less confusion  

## Key Design Decisions

1. **Change-based**: Linear history, snapshots at any point
2. **Tagging, not nesting**: Requirements/Stories/ADRs tag elements
3. **Organization is transparent**: Users organize files, parser handles it
4. **Shared services use naming convention**: `shared.ServiceName` - no imports needed
5. **Large = visualization problem**: Solve with UI, not complex structure
6. **User stories mostly container-level**: Business users understand containers
7. **Scenarios for behavioral descriptions**: How system behaves in different conditions
8. **Relations and flows inside architecture block only**: Both must be defined within the architecture block

## Next Steps

1. Define change DSL syntax
2. Define tagging model for requirements/user stories/ADRs/scenarios
3. Define snapshot generation from changes
4. Simplify architecture model (remove complex import/organization logic)
5. Design Studio visualization for large architectures (collapse/expand)
6. Design change timeline visualization

