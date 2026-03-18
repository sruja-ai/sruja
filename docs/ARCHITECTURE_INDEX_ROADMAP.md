# Sruja Architecture Index Roadmap

> Sruja is a structured architecture index and relationship graph that helps AI agents and developers navigate complex software ecosystems through canonical identities, links, and source references.

---

## 1. Product Definition

### What Sruja Is

- **Structure and linkage layer** - owns relationships, identities, navigation
- **Index to source artifacts** - points to OpenAPI, Kubernetes, docs, code
- **Graph for queries** - fast structural questions for AI agents

### What Sruja Is Not

- Not a replacement for OpenAPI (source of truth for API contracts)
- Not a replacement for Kubernetes (source of truth for deployment)
- Not a knowledge base that duplicates content from elsewhere

### Core Principle

> **Sruja owns structure and linkage, not detailed operational content.**

---

## 2. What Goes Into Sruja

### 2.1 Canonical Entities (existing)

Sruja already has these via C4 model:
- `system` - top-level grouping
- `container` - deployable service/unit
- `component` - module within container
- `database` - data store
- `queue` - message broker
- `person` - human actor

### 2.2 Identity and Aliases (NEW)

Cross-system naming problem: same service called "payments", "payments-api", "payments-service" in different places.

```sruja
Payments = container "Payment Service" {
  technology "Go"
  description "Handles payment processing"
  
  // NEW: canonical ID for cross-system reference
  id "svc.payments"
  
  // NEW: aliases found in codebase
  aliases ["payments-api", "payments-service", "PAYMENTS_SVC"]
}
```

### 2.3 Source Bindings (NEW)

Link to actual artifacts - AI follows these to get details.

```sruja
Payments = container "Payment Service" {
  // NEW: source bindings
  source openapi "./specs/payments.yaml"
  source kubernetes "./k8s/payments/"
  source docs "./docs/services/payments.md"
  source readme "./services/payments/README.md"
}
```

### 2.4 Structured Metadata (NEW)

Architectural metadata that doesn't live elsewhere:

```sruja
Payments = container "Payment Service" {
  // NEW: structured fields
  owner "team-payments"
  domain "commerce"
  criticality high        // low | medium | high | critical
}
```

### 2.5 Relationships (existing)

Current syntax works well - no changes needed:
```sruja
Checkout -> Payments "calls via HTTP"
Payments -> Database "persists to"
```

---

## 3. What Does NOT Go Into Sruja

- Full OpenAPI schemas (just path to them)
- Full Kubernetes YAML (just path to them)
- Docker details (just path to Dockerfile)
- Environment variables
- Secrets
- Low-level infra configuration
- Frequently changing runtime metrics

---

## 4. DSL Extensions

### 4.1 New Element Body Fields

```sruja
ContainerName = container "Display Name" {
  technology "Go"
  description "..."
  
  // Identity
  id "svc.container-name"           // canonical ID
  aliases ["alt-name", "ALT_NAME"]  // aliases from codebase
  
  // Structured metadata  
  owner "team-name"
  domain "business-domain"
  criticality high
  
  // Source bindings
  source openapi "./path/to/spec.yaml"
  source kubernetes "./path/to/k8s/"
  source docs "./path/to/docs.md"
  source readme "./path/README.md"
  source proto "./path/to/api.proto"
  source config "./path/to/config.yaml"
  
  // Existing fields continue to work
  doc ".sruja/knowledge/ContainerName.md"
  tags ["api", "backend"]
  metadata {
    custom "value"
  }
  slo { ... }
  scale { ... }
}
```

### 4.2 Source Types (extensible)

Standard source types AI agents should recognize:

| Type | Description | AI Action |
|------|-------------|-----------|
| `openapi` | OpenAPI/Swagger spec | Parse for endpoints, schemas |
| `asyncapi` | AsyncAPI spec | Parse for events, channels |
| `kubernetes` | K8s manifests directory | Parse for deployment config |
| `dockerfile` | Container build file | Parse for dependencies |
| `terraform` | IaC files | Parse for infra resources |
| `docs` | Documentation | Read for context |
| `readme` | Service README | Read for quick context |
| `proto` | Protobuf definitions | Parse for gRPC APIs |
| `config` | Configuration files | Context for settings |
| `graphql` | GraphQL schema | Parse for queries/mutations |
| `helm` | Helm charts | Parse for K8s templating |

### 4.3 Criticality Levels

```sruja
criticality low       // acceptable downtime
criticality medium    // degraded experience on failure
criticality high      // significant business impact on failure
criticality critical  // immediate business/finance impact on failure
```

---

## 5. CLI Commands

### 5.1 Phase 1 Commands

```bash
# List all source bindings across all elements
sruja sources

# List sources for specific element
sruja sources Payments

# Validate all source paths exist
sruja sources --validate

# Filter by source type
sruja sources --type openapi
```

### 5.2 Phase 2+ Commands

```bash
# Query architecture (Phase 3)
sruja query "what calls Payments"
sruja query "dependencies of Checkout"
sruja query "sources for API"
sruja query "owner of Payments"
sruja query "critical services"

# Discover from codebase (Phase 2)
sruja discover --extract openapi,kubernetes,docs
```

### 5.3 Enhanced Export (Phase 1)

```bash
# Export for AI consumption (enhanced JSON)
sruja export ai-context --output context.json

# Output includes:
# - elements with canonical_id, aliases
# - source bindings with types
# - relationships as graph
```

---

## 6. JSON Export Schema

### 6.1 Element with Extensions

```json
{
  "elements": {
    "Payments": {
      "id": "Payments",
      "kind": "container",
      "title": "Payment Service",
      "technology": "Go",
      "description": "Handles payment processing",
      
      "canonical_id": "svc.payments",
      "aliases": ["payments-api", "payments-service"],
      
      "owner": "team-payments",
      "domain": "commerce",
      "criticality": "high",
      
      "sources": [
        {"type": "openapi", "path": "./specs/payments.yaml"},
        {"type": "kubernetes", "path": "./k8s/payments/"},
        {"type": "docs", "path": "./docs/services/payments.md"}
      ],
      
      "doc": ".sruja/knowledge/Payments.md",
      "tags": ["api", "backend"]
    }
  }
}
```

**Note:** The element `id` ("Payments") is the Sruja identifier used in relationships. The `canonical_id` ("svc.payments") is for cross-system reference - use this when the same service has different names in code, configs, or external systems.

### 6.2 Relationship Graph

```json
{
  "relations": [
    {
      "id": "rel_1",
      "source": "Checkout",
      "target": "Payments",
      "label": "calls via HTTP",
      "kind": "calls"
    }
  ],
  
  "graph": {
    "adjacency": {
      "Checkout": ["Payments", "Cart"],
      "Payments": ["Database", "StripeAPI"]
    }
  }
}
```

---

## 7. Implementation Phases

### Phase 1: Core DSL Extensions (MVP)

**Goal:** Prove structural navigation works.

**Scope:**
1. Add `id` field to element body
2. Add `aliases` field to element body
3. Add `source <type> <path>` syntax
4. Add `owner`, `domain`, `criticality` fields
5. Extend JSON export with new fields
6. Add `sruja sources` CLI command

**Files to modify:**
- `crates/sruja-language/src/ast.rs` - add new AST fields
- `crates/sruja-language/src/parser/elements.rs` - parse new fields
- `crates/sruja-export/src/json/types.rs` - extend JSON types
- `crates/sruja-export/src/json/exporter.rs` - export new fields
- `crates/sruja-cli/src/commands/` - add sources command

**Deliverable:**
```bash
# List sources for an element
sruja sources Payments
# Output:
#   openapi: ./specs/payments.yaml
#   kubernetes: ./k8s/payments/
#   docs: ./docs/services/payments.md

# Validate all source paths exist
sruja sources --validate
# Output:
#   ✓ Payments: all sources valid
#   ✗ Checkout: ./specs/checkout.yaml not found
```

### Phase 2: Extractors

**Goal:** Auto-discover sources from codebase.

**Scope:**
1. OpenAPI extractor (finds .yaml/.json specs)
2. Kubernetes extractor (finds k8s manifests)
3. Doc extractor (finds README, docs)
4. Alias inference from codebase
5. Relationship inference from imports/calls
6. `sruja discover --sources` command

**New crate:**
- `crates/sruja-extract/` - extractor framework

**Deliverable:**
```bash
sruja discover --sources
# Generates draft with discovered source bindings
```

### Phase 3: Query Engine

**Goal:** Fast structural queries for AI.

**Scope:**
1. Graph-based query implementation
2. Natural language query parsing (simple patterns)
3. Query result formatting (JSON, text)
4. Integration with existing graph crate

**Deliverable:**
```bash
sruja query "what depends on Payments"
sruja query "critical services in commerce domain"
```

### Phase 4: AI Integration

**Goal:** First-class AI agent support.

**Scope:**
1. MCP server for Sruja queries
2. AI context export optimization
3. Streaming query results
4. Change impact queries

**Deliverable:**
- AI agents use Sruja as first architecture entry point

---

## 8. AST Changes

### 8.1 ElementDefBody Extensions

```rust
// In crates/sruja-language/src/ast.rs

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ElementDefBody {
    // Existing fields
    pub description: Option<String>,
    pub technology: Option<String>,
    pub doc: Option<String>,
    pub metadata: Vec<MetaEntry>,
    pub constraints: Vec<ConstraintEntry>,
    pub conventions: Vec<ConventionEntry>,
    pub style: Option<StyleBlock>,
    pub scale: Option<ScaleBlock>,
    pub slo: Option<SloBlock>,
    pub items: Vec<ElementDefBodyItem>,
    
    // NEW: Identity
    pub id: Option<String>,
    pub aliases: Vec<String>,
    
    // NEW: Structured metadata
    pub owner: Option<String>,
    pub domain: Option<String>,
    pub criticality: Option<Criticality>,
    
    // NEW: Source bindings
    pub sources: Vec<SourceBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceBinding {
    pub kind: SourceKind,
    pub path: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Criticality {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceKind {
    OpenApi,
    AsyncApi,
    Kubernetes,
    Dockerfile,
    Terraform,
    Docs,
    Readme,
    Proto,
    Config,
    GraphQL,
    Helm,
    Custom(String),
}
```

### 8.2 New ElementDefBodyItem Variants

```rust
pub enum ElementDefBodyItem {
    // Existing variants...
    ElementDef(Box<ElementDef>),
    Relation(Relation),
    Description(String),
    Technology(String),
    Doc(String),
    Metadata(MetadataBlock),
    // ...
    
    // NEW
    Id(String),
    Aliases(Vec<String>),
    Owner(String),
    Domain(String),
    Criticality(Criticality),
    Source(SourceBinding),
}
```

---

## 9. Parser Changes

### 9.1 New Parsing Functions

```rust
// In crates/sruja-language/src/parser/elements.rs

pub(crate) fn parse_id_field(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("id").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, value) = parse_string(input)?;
    Ok((input, value))
}

pub(crate) fn parse_aliases_field(input: &str) -> IResult<&str, Vec<String>> {
    let (input, _) = tag("aliases").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, value) = parse_string_array(input)?;
    Ok((input, value))
}

pub(crate) fn parse_source_binding(input: &str) -> IResult<&str, SourceBinding> {
    let (input, _) = tag("source").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, kind_str) = parse_identifier(input)?;
    let (input, _) = ws1(input)?;
    let (input, path) = parse_string(input)?;
    
    let kind = match kind_str.to_lowercase().as_str() {
        "openapi" => SourceKind::OpenApi,
        "asyncapi" => SourceKind::AsyncApi,
        "kubernetes" | "k8s" => SourceKind::Kubernetes,
        "dockerfile" | "docker" => SourceKind::Dockerfile,
        "terraform" | "tf" => SourceKind::Terraform,
        "docs" | "doc" => SourceKind::Docs,
        "readme" => SourceKind::Readme,
        "proto" | "protobuf" => SourceKind::Proto,
        "config" => SourceKind::Config,
        "graphql" | "gql" => SourceKind::GraphQL,
        "helm" => SourceKind::Helm,
        _ => SourceKind::Custom(kind_str.to_string()),
    };
    
    Ok((input, SourceBinding {
        kind,
        path,
        description: None,
    }))
}

pub(crate) fn parse_criticality(input: &str) -> IResult<&str, Criticality> {
    let (input, _) = tag("criticality").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, level) = parse_identifier(input)?;
    
    let crit = match level.to_lowercase().as_str() {
        "low" => Criticality::Low,
        "medium" | "med" => Criticality::Medium,
        "high" => Criticality::High,
        "critical" => Criticality::Critical,
        _ => return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))),
    };
    
    Ok((input, crit))
}
```

---

## 10. Success Criteria

### For Developers
- Add `id`, `aliases`, `owner`, `domain`, `criticality` to elements
- Use `sruja sources` to validate and list source paths
- Run `sruja discover --sources` (Phase 2) to auto-generate source bindings

### For AI Agents
- Read JSON export to find artifact paths
- Follow source paths to OpenAPI, K8s, docs
- Query `sruja query "sources for X"` (Phase 3) and get artifact paths
- Understand relationships between services

### For Maintainability
- Sruja files stay small (just structure + links)
- No duplication of OpenAPI/K8s content
- Architecture truth is version-controlled

### For Architecture Clarity
- Cross-system naming resolved via `id` + `aliases`
- Ownership clear via `owner` field
- Criticality visible for prioritization

---

## 11. Strong Rules

1. **Never mirror full source artifacts** - only paths
2. **Every entity should have canonical identity** - use `id` field
3. **Source bindings are first-class** - use `source` keyword
4. **Relationships are highest-value** - maintain with care
5. **Developer review before changes** - use diff/apply workflow
6. **Prefer deterministic extraction** - structured parsers over LLM

---

## 12. Final Positioning

> **Sruja is a structured architecture index and relationship graph that helps AI agents and developers navigate complex software ecosystems through canonical identities, links, and source references.**

### Three Core Promises

1. **Discover architecture artifacts automatically** (via extractors)
2. **Organize into canonical entities and relationships** (via DSL)
3. **Guide AI agents to the right source artifact** (via queries + source bindings)
