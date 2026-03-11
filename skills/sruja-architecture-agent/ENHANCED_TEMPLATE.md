# Sruja Architecture Agent - Enhanced Skill Template

This enhanced template improves upon the base `sruja-architecture-agent` skill with better abstraction detection and pattern recognition.

## Improvements Over Base Skill

### 1. Intelligent Abstraction Detection
Automatically determines appropriate component granularity based on repository size and complexity.

### 2. Pattern Recognition
Detects common architectural patterns (microservices, monolith, layered, event-driven).

### 3. Technology Detection
Enhanced detection of frameworks, databases, and external services.

### 4. Relationship Inference
Infers semantic relationships between components (queries, publishes, calls).

---

## Enhanced Agent Instructions

When using `@sruja-architecture-agent` with this enhanced template:

### Phase 1: Repository Analysis

```markdown
Analyze the repository structure:

1. **Size Assessment**
   - Count total modules/files
   - If >1000 modules: Use HIGH abstraction (systems + containers only)
   - If 100-1000 modules: Use MEDIUM abstraction (systems + containers + key components)
   - If <100 modules: Use LOW abstraction (detailed components)

2. **Technology Stack Detection**
   - Check package.json, go.mod, Cargo.toml, requirements.txt
   - Identify frameworks (Express, Django, Spring Boot, etc.)
   - Identify databases (PostgreSQL, MongoDB, Redis, etc.)
   - Identify external services (Stripe, AWS, SendGrid, etc.)

3. **Architectural Pattern Recognition**
   - Look for microservices signals: multiple Dockerfiles, docker-compose services
   - Look for layered architecture: api/, service/, dao/ structure
   - Look for event-driven: message queues, event buses
   - Look for monolith: single application entry point
```

### Phase 2: Component Identification

```markdown
Identify components at appropriate abstraction level:

**HIGH Abstraction (for large repos >1000 modules):**
- Focus on SYSTEMS (bounded contexts, domains)
- Group modules into CONTAINERS (services, applications)
- Skip individual functions/classes
- Example: "User Service" container with 3-5 key components

**MEDIUM Abstraction (for repos 100-1000 modules):**
- Identify main SYSTEMS
- Break down into CONTAINERS
- Add key COMPONENTS per container (3-7 each)
- Example: "API Gateway" container with "Router", "Auth Middleware", "Rate Limiter"

**LOW Abstraction (for repos <100 modules):**
- Detailed component breakdown
- Include most important modules
- Show data flows and dependencies
- Example: Complete breakdown of all modules
```

### Phase 3: Architecture Generation Rules

```markdown
Follow these rules when generating architecture.sruja:

1. **Component Count Target**
   - Aim for 15-30 top-level elements
   - Maximum 50 components total
   - Group related functionality

2. **Naming Conventions**
   - Use business domain names, not technical names
   - "User Management Service" not "user_svc_module"
   - "Order Processing" not "order_processor.go"

3. **Technology Tags**
   - Add technology() tags to containers
   - Be specific: "Node.js/Express" not just "JavaScript"
   - Include versions if relevant

4. **Descriptions**
   - Add description() for every system and container
   - Explain purpose in 1-2 sentences
   - Use business language

5. **Relationships**
   - Use semantic relationship labels
   - "queries" for database access
   - "publishes events to" for message queues
   - "calls" for synchronous communication
   - "subscribes to" for event subscriptions
```

### Phase 4: Pattern Templates

#### Microservices Pattern

```sruja
system "E-Commerce Platform" {
  description "Microservices-based e-commerce platform"
  
  container "API Gateway" {
    technology "Node.js/Express"
    description "Single entry point for all client requests"
    
    component "Router"
    component "Auth Middleware"
    component "Rate Limiter"
  }
  
  container "User Service" {
    technology "Python/FastAPI"
    description "Manages user accounts and authentication"
    
    component "User API"
    component "Auth Service"
  }
  
  container "Order Service" {
    technology "Go"
    description "Handles order processing and management"
    
    component "Order API"
    component "Order Processor"
  }
  
  datastore "User Database" {
    technology "PostgreSQL"
    description "Stores user data"
  }
  
  datastore "Order Database" {
    technology "MongoDB"
    description "Stores order documents"
  }
  
  messagebus "Event Bus" {
    technology "RabbitMQ"
    description "Async event communication"
  }
  
  container "API Gateway" -> container "User Service" "routes to"
  container "API Gateway" -> container "Order Service" "routes to"
  container "User Service" -> datastore "User Database" "queries"
  container "Order Service" -> datastore "Order Database" "queries"
  container "Order Service" -> messagebus "Event Bus" "publishes events to"
  container "User Service" -> messagebus "Event Bus" "subscribes to"
}
```

#### Layered Architecture Pattern

```sruja
system "Web Application" {
  description "Traditional layered web application"
  
  container "Presentation Layer" {
    technology "React"
    description "User interface components"
    
    component "Components"
    component "Pages"
    component "State Management"
  }
  
  container "API Layer" {
    technology "Node.js/Express"
    description "REST API endpoints"
    
    component "Routes"
    component "Controllers"
    component "Middleware"
  }
  
  container "Business Logic Layer" {
    technology "Node.js"
    description "Core business rules and logic"
    
    component "Services"
    component "Domain Models"
    component "Business Rules"
  }
  
  container "Data Access Layer" {
    technology "Node.js/TypeORM"
    description "Database interactions"
    
    component "Repositories"
    component "ORM Models"
  }
  
  datastore "Primary Database" {
    technology "PostgreSQL"
  }
  
  container "Presentation Layer" -> container "API Layer" "calls"
  container "API Layer" -> container "Business Logic Layer" "delegates to"
  container "Business Logic Layer" -> container "Data Access Layer" "uses"
  container "Data Access Layer" -> datastore "Primary Database" "queries"
}
```

#### Event-Driven Pattern

```sruja
system "Event-Driven System" {
  description "Event-driven architecture with CQRS"
  
  container "Command API" {
    technology "Node.js/Express"
    description "Handles write operations"
    
    component "Command Handlers"
    component "Validators"
  }
  
  container "Query API" {
    technology "Node.js/Express"
    description "Handles read operations"
    
    component "Query Handlers"
    component "Projections"
  }
  
  container "Event Store" {
    technology "EventStoreDB"
    description "Stores all events"
  }
  
  container "Read Model" {
    technology "MongoDB"
    description "Optimized read projections"
  }
  
  messagebus "Event Bus" {
    technology "Kafka"
    description "Event streaming platform"
  }
  
  container "Command API" -> container "Event Store" "appends events to"
  container "Event Store" -> messagebus "Event Bus" "publishes to"
  messagebus "Event Bus" -> container "Query API" "delivers events to"
  container "Query API" -> container "Read Model" "updates"
}
```

---

## Quality Checklist

Before finalizing `architecture.sruja`, verify:

### Abstraction Level
- [ ] Total components between 15-30 (max 50)
- [ ] No individual functions listed as components
- [ ] Related modules grouped into containers
- [ ] Clear hierarchy (system → container → component)

### Completeness
- [ ] All major systems included
- [ ] Key services/applications identified
- [ ] Databases and data stores listed
- [ ] External dependencies mentioned

### Accuracy
- [ ] Technology tags are correct
- [ ] Relationships reflect actual dependencies
- [ ] Names match codebase terminology
- [ ] Descriptions are accurate

### Clarity
- [ ] Readable by non-technical stakeholders
- [ ] Clear business domain names
- [ ] Logical organization
- [ ] Consistent naming conventions

---

## Example Workflow

### Step 1: Quick Scan

```bash
# Get repository stats
find . -name "*.js" -o -name "*.ts" -o -name "*.go" | wc -l

# If >1000 files: HIGH abstraction
# If 100-1000 files: MEDIUM abstraction
# If <100 files: LOW abstraction
```

### Step 2: Technology Detection

```bash
# Check for Node.js
[ -f package.json ] && cat package.json | jq '.dependencies | keys'

# Check for Python
[ -f requirements.txt ] && cat requirements.txt

# Check for Go
[ -f go.mod ] && cat go.mod
```

### Step 3: Pattern Detection

```bash
# Microservices signals
find . -name "Dockerfile" | wc -l
grep -r "docker-compose" . | wc -l

# Layered architecture signals
ls -la | grep -E "api|service|dao|controller|model"

# Event-driven signals
grep -r "kafka\|rabbitmq\|event" . | wc -l
```

### Step 4: Generate Architecture

```markdown
Based on analysis:
- 2500 modules detected → HIGH abstraction
- Node.js/Express + PostgreSQL detected
- Microservices pattern detected (3 Dockerfiles, docker-compose)

Generate architecture.sruja with:
- 3-5 main systems
- 8-15 containers total
- 20-30 components total
```

---

## Common Mistakes to Avoid

### ❌ Too Granular

```sruja
# BAD: Function-level components
container "User Service" {
  component "getUserById()"
  component "createUser()"
  component "updateUser()"
  component "deleteUser()"
  # ... 50 more functions
}
```

### ✅ Appropriate Abstraction

```sruja
# GOOD: Logical components
container "User Service" {
  component "User Management API"
  component "Authentication Service"
  component "User Repository"
}
```

### ❌ Missing Context

```sruja
# BAD: No descriptions or technologies
container "Service A" {
  component "Module 1"
  component "Module 2"
}
```

### ✅ Rich Context

```sruja
# GOOD: Clear descriptions and technologies
container "User Service" {
  technology "Python/FastAPI"
  description "Handles user registration, authentication, and profile management"
  
  component "User API" {
    technology "REST/JSON"
  }
  
  component "Auth Service" {
    technology "JWT/OAuth2"
  }
}
```

### ❌ Generic Relationships

```sruja
# BAD: Generic "uses" relationship
container "API" -> container "Service" "uses"
```

### ✅ Semantic Relationships

```sruja
# GOOD: Clear relationship semantics
container "API Gateway" -> container "User Service" "routes requests to"
container "Order Service" -> messagebus "Event Bus" "publishes order events to"
```

---

## Validation

After generating `architecture.sruja`:

```bash
# 1. Lint check
sruja lint architecture.sruja

# 2. Validate component count
grep -c "component\|container" architecture.sruja
# Should be 15-50

# 3. Check drift
sruja drift -r . -a architecture.sruja
# Should have minimal violations

# 4. Export to review
sruja export markdown architecture.sruja -o architecture.md
# Review for clarity
```

---

## Integration with Testing Framework

When using with the automated testing framework:

1. **After CLI Analysis:** Read generated `AGENT_INSTRUCTIONS.md`
2. **During Agent Analysis:** Apply this enhanced template
3. **Before Evaluation:** Run validation checks
4. **After Evaluation:** Review recommendations and iterate

---

## Best Practices

### For Large Repositories (>1000 modules)
- Focus on bounded contexts
- Group by business domain
- Show integration patterns
- Skip implementation details

### For Medium Repositories (100-1000 modules)
- Balance overview and detail
- Show key architectural layers
- Include important services
- Highlight critical data flows

### For Small Repositories (<100 modules)
- Detailed component breakdown
- Show most modules
- Illustrate all relationships
- Include configuration details

---

**Version:** 2.0 (Enhanced)
**Compatible with:** sruja-architecture-agent v1.0+
**Last Updated:** 2026-03-09
