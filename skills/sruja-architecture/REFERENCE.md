# Sruja Architecture Reference

Detailed reference for discovery workflow, modeling rules, and refinement with the Sruja architecture skill.

## Discovery Workflow

### Phase 1: Evidence Collection

Always start with deterministic evidence collection from the CLI:

```bash
sruja discover --context -r . --format json
```

**Output includes:**
- Repository structure (directories, file patterns)
- Detected technologies (languages, frameworks, libraries)
- Module boundaries (code organization)
- Entry points (main functions, API routes, etc.)
- External dependencies (package.json, requirements.txt, go.mod, etc.)
- Scan scope (what was included/excluded from analysis)

**What this tells you:**
- What programming languages are used
- What frameworks and libraries are present
- How the codebase is organized
- What external services are dependencies
- What files/directories were analyzed

### Phase 2: Scope Selection

Based on evidence, determine the scope of your architecture:

**C4 Context Level (Person + System):**
- External actors: humans (person), external software (system, optional tags)
- Major system boundaries

**C4 Container Level:**
- Deployable units (APIs, workers, apps)
- Datastores (databases, caches, queues)

**C4 Component Level:**
- Internal modules within containers
- Detailed data flows

**When to use each level:**
- Start with Context + Container for most projects
- Add Component level only for complex subsystems
- Don't add Component level globally just for completeness

### Phase 3: Ask Targeted Questions

Ask only when evidence is ambiguous. Examples:

**Scope Unclear:**
- "What are the main system boundaries? Are these services separate systems or containers?"
- "Is this a monolith or microservices architecture?"

**External Dependencies:**
- "What external services do you integrate with beyond what's in package.json?"
- "Are there manual deployments or infrastructure dependencies?"

**Datastores:**
- "What databases or caches are used?"
- "Is there a message queue for async operations?"

**Deployment:**
- "How are these components deployed?"
- "Are there separate services or a single deployable?"

**Data Flows:**
- "What are the main data flows between components?"
- "How does authentication work?"

**Stop asking when:** You have enough evidence to generate a minimal architecture.

### Phase 4: Generate Minimal DSL

Generate `architecture.sruja` covering only what evidence supports:

```sruja
// External actors
User = person "User" {
  description "End user of the application"
}

// Main system
Application = system "Application" {
  description "Main application system"

  // Containers (deployable units)
  API = container "API Service" {
    technology "Node.js + Express"
    description "REST API backend"
  }

  Frontend = container "Web Frontend" {
    technology "React"
    description "User interface"
  }

  Database = database "Database" {
    technology "PostgreSQL"
    description "Primary data storage"
  }
}

// Relationships
User -> Application.Frontend "HTTPS"
Application.Frontend -> Application.API "REST API"
Application.API -> Application.Database "SQL"
```

### Phase 5: Validate

Always lint:

```bash
sruja lint architecture.sruja
```

Fix all errors before considering complete.

## Modeling Rules

### Component Types

Use these types based on evidence:

**Person:** Human actors only (do not use for external software)
- Users (Admin, Customer, Guest)
- Administrators, operators, stakeholders
- Use **system** for external software (APIs, SaaS, control planes, destinations); optional `tags ["external"]`

**System:** Major boundaries
- High-level system boundaries representing major domains
- Separates your system from external systems
- Contains containers

**Container:** Deployable units
- APIs (REST, GraphQL, gRPC)
- Background workers/job processors
- Message consumers/producers
- Web applications
- Mobile apps

**Database:** Datastores
- Primary databases (PostgreSQL, MySQL)
- Caches (Redis)
- Queues (Kafka, RabbitMQ)
- Search indexes (Elasticsearch)

### Relationships

Be specific with relationships:

```sruja
// Good - specific protocol and purpose
Frontend -> API "HTTPS (REST API)"
API -> Database "PostgreSQL (JDBC)"
API -> Queue "publishes events"

// Bad - vague
Frontend -> API "uses"
API -> Database "connects"
```

Include:
- Protocol (HTTPS, gRPC, SQL, etc.)
- Purpose (reads, writes, publishes, etc.)
- Data flow direction

### Architectural Patterns

Choose patterns based on evidence:

**Monolith:** Single deployable unit
- Small teams (1-10 developers)
- Simple domain
- Fast time-to-market

**Microservices:** Multiple independent services
- Large teams
- Complex domain
- Independent scaling needs

**Event-Driven:** Async messaging
- Real-time processing
- Loose coupling
- Eventual consistency acceptable

**CQRS:** Separate read/write models
- Complex queries
- High throughput
- Different data models for read vs write

### Anti-Patterns to Avoid

**God Component:**
```sruja
// Don't do this
Everything = container "Everything" {
  technology "Node.js"
  description "Does auth, orders, payments, inventory, notifications"
}

// Do this instead
AuthService = container "Auth Service" { ... }
OrderService = container "Order Service" { ... }
PaymentService = container "Payment Service" { ... }
```

**Direct Database Access:**
```sruja
// Don't do this
Frontend -> Database "SQL"
Worker -> Database "SQL"
API -> Database "SQL"

// Do this instead
Frontend -> API "HTTPS"
Worker -> API "REST API"
API -> Database "SQL"
```

**Circular Dependencies:**
```sruja
// Don't do this
ServiceA -> ServiceB "calls"
ServiceB -> ServiceA "calls"

// Do this instead
ServiceA -> CommonService "uses"
ServiceB -> CommonService "uses"
```

## Refinement Workflow

### Drift Detection

When a baseline exists, detect drift:

```bash
sruja drift -r . -a architecture.sruja --format json
```

**Analyzes:**
- New circular dependencies
- New orphan components
- Layer violations
- Structural changes
- Suggested improvements

### Refinement Steps

1. **Review drift findings**
2. **Determine if drift is:**
   - Intentional (architecture evolved)
   - Unintentional (technical debt)
   - False positive (scope change)
3. **Update architecture.sruja**
4. **Run `sruja lint`**
5. **Commit changes**

### When to Refine

- After significant code changes
- When adding new features
- When refactoring code
- On a regular schedule (weekly/monthly)
- Before releases

### Minimal Updates

Refine incrementally:
- Fix errors first
- Add new components only if needed
- Remove outdated components
- Update relationships to match code
- Keep descriptions accurate

## Evidence Fidelity

### Trust the Evidence

**Always trust:**
- File structure (what exists)
- Technology detection (what's actually used)
- Dependencies (what's imported)
- Entry points (where execution starts)

**Never trust:**
- Heuristics about "this looks like X"
- Assumptions about deployment
- Guesses about external integrations
- Narratives without code backing

### Surface Uncertainty

When evidence is insufficient:

```sruja
/*
OPEN QUESTIONS:
- Authentication mechanism not clear from code
- Message queue purpose not documented
- External API endpoints not fully discovered
- Deployment model unknown
*/
```

Or add uncertainty markers:
```sruja
ExternalService = system "External Service" {
  description "External integration (evidence unclear)"
  // Add comment: "Need to verify service details"
}
```

## Scan Scope

### Default Excludes

The CLI excludes these by default to focus on production code:
- Generated code (node_modules, target, build, dist)
- Vendor directories (vendor, third_party)
- Fixtures and test data (fixtures, __mocks__)
- Documentation (docs, README, examples)
- Evaluation and benchmarks (evaluation, benchmark, perf)

### Why This Matters

- Keeps evidence focused on production-relevant code
- Avoids pollution from dependencies
- Ensures scan scope is reproducible
- Makes skill output more trustworthy

### Custom Scope

If you need custom scope:

```bash
# Scan specific directory
sruja discover --context -r ./src --format json

# Or configure in .sruja.yaml (if supported)
```

## Output Format

### JSON Structure

`discover --context --format json` returns:

```json
{
  "structure": { ... },
  "technologies": [ ... ],
  "modules": [ ... ],
  "entry_points": [ ... ],
  "dependencies": [ ... ],
  "scan_scope": {
    "included": [ ... ],
    "excluded": [ ... ],
    "total_files": 1234
  }
}
```

**Use this as:** The single source of truth for what the CLI actually analyzed.

## Common Mistakes

### Don't Guess

```
// Don't do this
Cache = database "Redis Cache" {
  description "Used for caching (assumed)"
}

// Do this instead
// Add to OPEN QUESTIONS:
// - Is there a cache layer? What caching strategy?
```

### Don't Over-Model

```
// Don't add components just for completeness
UserService = container "User Service" { ... }
OrderService = container "Order Service" { ... }
PaymentService = container "Payment Service" { ... }
NotificationService = container "Notification Service" { ... }
// ... 20 more containers

// Do this: Start minimal, add as needed
Application = system "Application" {
  API = container "API" { ... }
  Frontend = container "Frontend" { ... }
  Database = database "Database" { ... }
}
```

### Don't Skip Linting

Always lint after generating or editing:

```bash
sruja lint architecture.sruja
```

Fix errors before committing.

## Next Steps

- **Prompt patterns**: See PROMPTS.md
- **Compiled guide**: See AGENTS.md
- **Individual rules**: See rules/ directory
- **Install skill**: `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture`
