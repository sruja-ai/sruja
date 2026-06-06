# Sruja Architecture Reference

Detailed reference for discovery workflow, modeling rules, and refinement with the Sruja architecture skill.

## Discovery Workflow

### Phase 1: Evidence Collection

Prefer `.sruja/context.json` when present and not stale (e.g. from **Sruja: Refresh repo context** or `sruja sync -r .`). If it is missing or stale, **run** the CLI yourself to get evidence—do not ask the user to run a command first. Optionally suggest Refresh repo context for faster results next time.

```bash
sruja sync -r .
```
or JSON only: `sruja discover --context -r . --format json`

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

Generate `repo.sruja` covering only what evidence supports:

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
sruja lint repo.sruja
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
sruja drift -r . -a repo.sruja --format json
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
3. **Update repo.sruja**
4. **Run `sruja lint repo.sruja`**
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

## SDLC update workflow

Canonical flow for keeping architecture in sync with code (guided automation, human-in-loop):

1. **Gather evidence** — `sruja sync -r .` (writes `.sruja/context.json` and runs drift).
2. **Detect drift** — `sruja drift -r . -a repo.sruja` (or use output from sync). Review violations and suggestions.
3. **Propose changes** — AI analyzes drift and proposes concrete DSL edits (new/missing components, relationship updates, removals). Do not apply automatically.
4. **Human approval** — User reviews diff or list of changes and approves or rejects.
5. **Apply and validate** — After approval, apply edits to repo.sruja, then run `sruja lint repo.sruja` and fix any errors.
6. **Optional** — User runs `git add` and `git commit` (or creates PR). Skill may suggest commands but does not execute them.

**Automation level:** Drift detection and proposal are automatic; applying changes to the DSL is always manual. Commit/PR is user-driven.

## Query patterns

Use these to answer questions without editing the DSL.

### Impact analysis

**User need:** "What breaks if I change or remove X?"

**Steps:**
- Run `sruja human explain <element_id> --file repo.sruja` to get incoming/outgoing relation counts and element description.
- Optionally run `sruja tree repo.sruja` to see full hierarchy (tree takes a file path, not an element).
- Summarize: dependents (incoming), dependencies (outgoing), and which parts of the system would be affected.

**Note:** The CLI does not support `sruja tree <element> --depth N`. Use `explain` for element-level impact; use `tree` with the file for structure.

### Requirement traceability

**User need:** "Which components implement requirement R1?" or "Trace requirement R1 to code."

**Steps:**
- Read repo.sruja for requirement definitions (e.g. `R1 = requirement functional "..."`).
- Identify links to elements (tags, references, or narrative in descriptions).
- If the DSL exports requirements (e.g. `sruja export markdown` with requirements section), use that to list requirements and suggest how to link them to elements if not already done.

### Compliance check

**User need:** "Are we within architectural rules?" or CI gate.

**Steps:**
- Run `sruja compliance -r . -a repo.sruja -f json` for a canonical report (status, health_score, drift_entries, policy_violations, remediation_checklist).
- Run `sruja validate repo.sruja --policy <rules>` when policy files exist.
- Present findings; do not auto-apply. Non-zero exit from `sruja compliance` indicates non-compliant state.

## Integrations

**Phase 1 (now):** CLI + skill. All SDLC actions work via CLI commands and skill prompts. No external dependencies. Works in any editor (Cursor, VS Code, Claude, etc.).

**Phase 2 (short-term):** GitHub. Use `sruja drift-pr -r . --base origin/<base> --head HEAD -f github-actions` for PR-scoped drift. Use templates in `templates/github-actions/` (e.g. sruja-architecture-pr.yml). Add prompt patterns for PR review with architecture context.

**Phase 3 (future):** PM tools. Import requirements from Jira/Linear; export ADRs to Confluence/Notion; link requirement IDs to external tickets. Deferred until core SDLC and query flows are solid.

## Evidence Fidelity

### Static graph (Tree-sitter)

Evidence comes from a **static analysis graph** produced by Tree-sitter parsing of source code. The CLI builds a nodes-and-edges graph (modules, imports, dependencies) from supported languages. This graph backs `sruja discover` and `sruja sync` (and `.sruja/context.json`). Use it to verify and assist the AI: stay evidence-first and avoid inventing components or relationships not present in the code.

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

`discover --context --format json` returns (matches CLI `DiscoverContextJson`):

```json
{
  "repo": "<repo path>",
  "scan_scope": { "included": [ ... ], "excluded": [ ... ], ... },
  "components": 42,
  "edges": 58,
  "primary_language": "TypeScript",
  "framework": "React",
  "architecture_style": "monolith",
  "domain": null,
  "suggested_areas": [ "src", "lib", "apps" ]
}
```

From this you get: repository path, scan scope, graph size (components/edges), primary language, framework, inferred architecture style and domain, and suggested areas for scoping. **Use this as** the single source of truth for what the CLI actually analyzed.

### Progressive discovery: summary and full graph

After `sruja sync -r .`, two artifacts are written:

- **`.sruja/context.json`** — Summary (Tier 1): the DiscoverContextJson fields above plus `updated_at`, `truth_status`, `baseline_path`, `git_commit`. Use this first for fast, small context.
- **`.sruja/graph.json`** — Full graph (Tier 2/3): complete `nodes` and `edges` from the scan. No information is dropped. When you need to reason about a specific area or module, read this file and use only the slice (e.g. filter nodes/edges by suggested_areas or path). For very large repos, prefer scoped access (filter by area) rather than loading the entire file into context at once.

Use summary first; when reasoning about a specific area or module, request that slice from `.sruja/graph.json` or run discover. This keeps large and multi-repo fully representable without blowing context limits.

## Export Coverage

The CLI and export crate support **DSL**, **Markdown**, and **Mermaid** output. Use `sruja export` (or the equivalent API) to produce documentation from `.sruja` files.

### DSL (round-trip)

The DSL printer pretty-prints the full AST back to Sruja source. All top-level constructs are covered:

- **Elements**: person, system, container, component, database, queue (with body: description, technology, metadata, scale, slo, nested elements)
- **Relations**: `From -> To "Label"` with optional tags
- **Governance**: requirements, ADRs, policies, constraints, conventions
- **Behavior**: scenarios, flows, feedback loops, causal loops
- **Structure**: overview block, views (with include/exclude), deployment tree, styles, kind/tag definitions, imports, extend

### Markdown

Markdown export produces a structured document with optional sections (each can be toggled via options):

| Section        | DSL source                    | Notes                                      |
|----------------|--------------------------------|--------------------------------------------|
| Overview       | `overview { ... }`             | Summary, audience, scope, goals, risks     |
| Systems        | systems + containers/components| Per-system L2 diagram; per-container L3    |
| Persons        | `person` elements             |                                            |
| Deployments    | `deployment` nodes            | Nested deployment tree                    |
| Requirements   | `requirement` items           | Type and description                       |
| ADRs           | `adr` items                   | Status, context, decision, consequences    |
| Policies       | `policy` items                | Category, enforcement                      |
| Constraints    | `constraints { ... }`         |                                            |
| Conventions    | `conventions { ... }`         |                                            |
| Scenarios      | `scenario` / `flow`           | Mermaid sequence diagram per scenario/flow |
| Feedback Loops | `feedbackLoop`                | Mermaid diagram                            |
| Causal Loops   | `causalLoop`                  | Mermaid diagram                            |
| Views          | `view` definitions            | When using view-driven export; diagram per view from resolved include/exclude |

When **view-driven export** is enabled (`use_views` or `view_name`), the document is organized by named views; each view's Mermaid diagram uses only that view's resolved elements and relations.

### Mermaid

Mermaid export produces flowchart-style C4 diagrams:

- **L1 (context)**: persons and systems only
- **L2 (container)**: one system and its containers/datastores/queues (optionally focused via `target_id`)
- **L3 (component)**: one container and its components

Styles are applied by element kind (person, system, container, database, queue, component, external). Relation labels are rendered on edges.

- **View-driven**: When exporting from a resolved view (e.g. from Markdown view-driven export), the diagram contains only the view's elements and relations, respecting `include`/`exclude` and scope.

Scenarios and flows can be exported as **sequence diagrams**; feedback and causal loops as dedicated Mermaid diagrams.

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
sruja lint repo.sruja
```

Fix errors before committing.

## Next Steps

- **Core skill**: See SKILL.md (skill root)
- **Prompt patterns**: See PROMPTS.md (this directory)
- **Compiled guide**: See AGENTS.md (this directory)
- **Individual rules**: See rules/ directory (skill root)
- **Install skill**: `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture`
