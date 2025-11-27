# Sruja SQL and MCP — Focused Design

Purpose: distill the Sruja Query Language (SQL) and MCP server into a concise, implementation-ready plan.

Executive Summary
- SQL: human- and AI-friendly query DSL for the architecture model.
- MCP: structured server exposing load, explain, query, validate, diagram, diff, and safe modify operations.

MCP Capabilities
- `loadArchitecture`, `explain`, `diagram`, `query`, `validate`, `diff`, `modifyArchitecture`, `listPlugins`, `listVariants`, `analyzeMetadata`.

Compact API Examples
```json
{ "call": "query", "q": "find containers where metadata.team == \"Payments\"" }
```
```json
{ "call": "diagram", "type": "c4-container", "root": "Billing" }
```
```json
{ "call": "modifyArchitecture", "action": "addRelation", "from": "Frontend", "to": "BillingAPI", "label": "Requests checkout" }
```

SQL Overview
- Syntax: `find <type> [where <expr>]`
- Types: `systems`, `containers`, `components`, `persons`, `datastores`, `queues`, `relations`, `adrs`, `requirements`, `journeys`, `steps`, `anything`
- Operators: `==`, `!=`, `in`, `contains`, `starts_with`, `ends_with`, `matches`, `exists`, `and`, `or`, `not`
- Fields: core (`id`, `label`, `description`, `technology`, `type`, `tags[]`), `metadata.<key>`, plugin fields (e.g., `security.risk`, `cloud.region`)

SQL Grammar (EBNF)
```
query           = "find", type, [ whereClause ] ;
type            = ident ;
whereClause     = "where", expr ;
expr            = orExpr ;
orExpr          = andExpr, { "or", andExpr } ;
andExpr         = unaryExpr, { "and", unaryExpr } ;
unaryExpr       = [ "not" ], primaryExpr ;
primaryExpr     = comparison | "(" expr ")" ;
comparison      = field, comparator, value | field, matchOp, string ;
field           = ident, { ".", ident } ;
comparator      = "==" | "!=" | "in" | "not", "in" ;
matchOp         = "contains" | "starts_with" | "ends_with" | "matches" ;
value           = string | number | list ;
list            = "[", value, { ",", value }, "]" ;
```

Go Implementation Outline
- Parser: Participle structs for `Query`, `WhereClause`, `Expr`, `BinaryExpr`, `Field`, `Value`.
- Engine: `QueryEngine.Execute(query)` evaluates AST over semantic model with plugin field getters.
- Plugins: register computed fields and validation hooks used by SQL and MCP.

Query Language for Rules & Policies
- Shared expression engine across Rule DSL and Policy DSL.
- Supported ops in rule `when`/`ensure` and policy `applies_to`/`controls`: `==`, `!=`, `in`, `contains`, `starts_with`, `ends_with`, `matches`, `exists`, with `and`/`or`/`not`.
- Values: strings, numbers, lists; fields: `id`, `label`, `verb`, `from`, `to`, `metadata.<key>`.

Rule DSL Notes
- Term syntax allows `not` prefix: use `not <field> exists` or `not <field> in ["a","b"]`.
- Relation fields available in rule contexts: `from`, `to`, `verb`, `label`.

Policy DSL Notes
- `applies_to <type> where <expr>` selects target elements using the same expression ops.
- `controls { <expr>* }` checks metadata or fields using the same operators.

Entities DSL
- First-class domain model: `entities { entity <Name> { description, fields { key: Type }, relations { name -> Target }, invariants { "expr" }, lifecycle { StateA -> StateB }, versioning { current "x.y" backwards_with "1.x" }, constraints { key "value" } } }`.
- Exists at architecture, system, and container levels; `domain <Name> { entities { ... } }` supported.
- Contracts, components, and data can map to entities via `request_map`, `emits_schema`, `writes_schema`, etc.

Event DSL (Domain-driven)
- Domain events defined under `events { event <Name> { version, entity, category, description, schema { ... }, metadata { ... }, guarantees { ... }, lifecycle_effect { Entity.State -> Entity.State }, causes { ... }, publishers { Qualified }, consumers { Qualified }, versioning { ... } } }`.
- Events integrate with entities for lifecycle validation and schema alignment; publishers/consumers are qualified component references.

MCP Endpoints (extended)
- `/review-architecture` evaluates rules; `/evaluate-policy` computes compliance.
- `/list-entities`, `/list-events` enumerate parsed domain models.
- `/validate-event` checks lifecycle_effect states exist in entity lifecycle and schema alignment basics.
 - `/validate-approval-policy` evaluates a single approval policy (JSON or DSL).
 - `/validate-all` batch evaluates policies to produce CI gating approvals/diagnostics.

Policy Evaluation & Diff Engine
- Diff-aware evaluator and semantic diff structures implemented; see `docs/policy-eval.md` and `docs/diff-engine.md`.

LSP Completions & Hover
- Policy DSL completions and hover implemented; see `docs/lsp.md`.

LSP Integration
- Autocomplete types after `find `; fields after `where `; metadata keys and plugin fields; enum values from project metadata.

Examples
```text
find containers
find systems where id contains "Billing"
find relations where from == "Frontend"
find containers where metadata.team == "Payments"
```

Server Structure
```
mcp/
 ├── server.go
 ├── protocol/handlers.go
 ├── core/{parser.go, semantic.go, validator.go, query.go, diagram.go, ast-editor.go}
 ├── plugins/loader.go
 └── utils/{file.go, logging.go, config.go}
```

Implementation Phases
- Phase 1: load, explain, validate, diagram (JSON).
- Phase 2: safe AST modify operations.
- Phase 3: diagram SVG/Graph; C4 integration.
- Phase 4: diff/variants.
- Phase 5: plugin hooks for metadata, validation, explain, query, modify.

The sections below remain for reference; this document now prioritizes SQL and MCP.

This is exactly how tools like Terraform, GraphQL, Prettier, Structurizr, Husky, and TypeScript achieved rapid adoption.

Below is a **holistic adoption strategy** — technical, product, and organizational.

---

# 🏆 THE 7 FOUNDATIONS OF EASY ADOPTION

---

# **1. ZERO-FRICTION ONBOARDING (5 minutes → usable)**

Most tools fail because onboarding is heavy.

Your first experience MUST feel like:

> “Wow, this is easy.”

### ✔ Install with 1 command

```
curl -sSL https://sruja.ai/install | bash
```

Or via:

```
brew install sruja
npm install -g sruja
```

### ✔ VSCode Extension in Marketplace

People should just search “Sruja” and click install.

### ✔ First project in 30 seconds

```
sruja init
```

This generates:

* sample architecture
* sample diagrams
* sample ADR
* README
* VSCode workspace settings

### ✔ No config required

Defaults should “just work”.

---

# **2. Copy-Paste Friendly Syntax (Familiar, Minimal, Intuitive)**

Your DSL should feel like:

* Terraform
* GraphQL SDL
* Structurizr DSL

This means intuitive structure:

```
system API {
    container App "Web App"
}
```

This makes it **instantly relatable**.

Avoid complex syntax.
Avoid parentheses.
Avoid boilerplate.

---

# **3. Amazing Autocomplete (LSP First) → Makes You Look Smart**

Most teams adopt a DSL primarily because *autocomplete feels magical*.

Your LSP must be so good that typing feels effortless.

### ✔ Autocomplete EVERYTHING

* systems
* containers
* metadata
* relations
* references
* ADR IDs
* requirement types
* imported architecture elements

### ✔ Inline explanations

Hover should show:

* diagrams
* metadata
* relationships
* ADR summary
* description

This is **pure adoption fuel**.

---

# **4. Instant Visual Feedback → “Architecture Preview”**

The moment a dev writes:

```
system API {
  container App
}
```

They should see a diagram update in real time.

### Live preview pane

Like Markdown preview → ultra addictive.

### Visualizations:

* C4 diagrams
* journey flow diagrams
* dependency map
* architecture landscape
* cloud infra mapping (via plugins)

When people SEE what they type, adoption skyrockets.

---

# **5. Plays Nicely With Git & PR Workflows**

This is extremely important.

### ✔ Architectural Diff

```
sruja diff HEAD~1 HEAD
```

Developers see what changed in a diagram.
Reviewers love this.

### ✔ PR Comment Bot

On PR, auto-generate:

* updated diagrams
* ADR changes
* requirement changes
* warnings
* metadata errors

This integrates into *existing workflows*—no new process needed.

### ✔ Architecture as Code

Developers already love code reviews.

---

# **6. Integrate With Existing Architecture Artifacts (DON'T replace everything)**

Adoption fails when a tool tries to replace:

* wiki pages
* diagrams
* UML
* cloud diagrams
* Word docs

Your tool must **co-exist**, not replace.

### Integrate with:

* Confluence / Notion
* GitHub Wiki
* Markdown docs
* Mermaid
* PlantUML
* Cloud diagrams

Allow exporting:

```
sruja diagram --format mermaid
sruja export --format markdown
```

This reduces perceived disruption.

---

# **7. Start Small, Grow Naturally (Low commitment)**

Teams should be able to start with:

```
system API
```

Or even:

```
container WebApp
```

Then progressively expand.

Your DSL must support **partial models**:

* missing descriptions
* unresolved references
* minimal structure

This guarantees teams can start with whatever they have.

---

# 🧠 ADVANCED ADOPTION BOOSTERS

---

# **8. Templates & Examples**

```
sruja new microservices
sruja new event-driven
sruja new monolith
sruja new service-mesh
sruja new api-gateway
```

People LOVE templates.

Also provide:

* sample metadata
* sample plugin configs
* example architectures
* real-world patterns

---

# **9. Guided Docs (High-quality Learning Path)**

Better than long PDF docs.

Provide:

* Quickstart
* “Learn Sruja in 5 minutes”
* “Real examples”
* “Deep dive: Metadata”
* “Best practices for modeling”
* “Plugins 101”
* “Using Sruja in big companies”

---

# **10. Make it Playful (Dev Playground)**

Offer a web-based playground:

* type DSL
* see diagrams
* see explanations
* export JSON
* validate instantly
* try plugins

Zero installation means instant adoption.

---

# **11. Smart Explain Mode → AI built-in**

Huge adoption lever.

```
sruja explain system API
```

Outputs:

* purpose
* dependencies
* components
* metadata insights
* ADRs that apply
* risk warnings
* flow involvement

This makes the tool a **teaching assistant**, not a static checker.

---

# **12. Great Error Messages (Developer-friendly)**

Bad error:

```
Error: unexpected token
```

Great error:

```
Error: Unexpected 'BillingAPI' here.
  → relations must be inside a system or container
  → quick fix: wrap in system { ... }
```

Even better:

```
Did you mean: Billing.App?
```

Best DX comes from empathetic error UX.

---

# **13. Plugin Ecosystem → Let Community Multiply Value**

Make it **easy** to build plugins:

```
sruja plugin create my-plugin
```

Provide:

* examples
* templates
* testing API
* metadata registration

Plugins will drive adoption faster than anything else.

---

# **14. Allow Gradual Adoption Across Teams**

Don’t require a big-bang rollout.

Support:

* model only one system
* import existing diagrams
* slowly grow models
* adopt metadata later
* adopt plugins optionally

Make usage modular.

---

# **15. Focus on “Developer Joy”**

A tool adopted is one that sparks joy:

* instantaneous
* visual
* delightful autocomplete
* minimal syntax
* smart suggestions
* beautiful UI
* diagrams appear like magic

Make the tool feel like a **friendly assistant**.

Not a strict bureaucracy tool.

---

# 🏆 FINAL ANSWER — THE FORMULA FOR EASY ADOPTION

### ✔ ZERO-FRICTION onboarding

### ✔ BEAUTIFUL live diagrams

### ✔ INTELLIGENT autocomplete

### ✔ FRIENDLY CLI

### ✔ AWESOME docs + examples

### ✔ GIT + PR integration

### ✔ METADATA-driven extensions

### ✔ TEMPLATES & PLAYGROUND

### ✔ GREAT error messages

### ✔ PLUGINS to extend anything

### ✔ START small → scale up

**This is exactly how you make Sruja DSL a delight to use.
This will unlock rapid adoption in real teams.**


Great — **building an MCP (Model Context Protocol) server** for Sruja DSL is the *perfect* way to enable **AI integration**, IDE copilots, architecture analysis, automated refactoring, explanations, and natural-language queries.

Below is the full plan:

* What an MCP should do for an architecture DSL
* How to design the MCP capabilities
* What APIs you should expose
* Data models
* Plugin model
* Implementation phases
* How AI (ChatGPT / Copilots / Claude / Cursor) will use it

This is exactly how we make Sruja "AI-native".

---

# 🧠 **Why You Want an MCP for Sruja**

MCP gives your DSL:

### ✔ Universal AI interoperability

* ChatGPT
* Claude
* Cursor
* VSCode AI extensions
* IDE copilots

### ✔ Structured access to architecture

AI can query architecture, modify it, validate it.

### ✔ Real-time architecture explanations

“Explain why BillingAPI talks to InventoryDB.”

### ✔ Automated refactoring

“Split API system into API+Gateway.”

### ✔ Automatic ADR suggestion

“Propose ADR for changing authentication flow.”

### ✔ Natural-language modeling

“Create a new system for payments and connect it to BillingAPI.”

### ✔ Programmatic diagram generation

“Generate a C4 container diagram.”

### ✔ Metadata transformation

“Apply security metadata to all public-facing APIs.”

### ✔ Multi-variant comparison

“Compare OptionA and OptionB architectures.”

This transforms your DSL into a conversational modeling environment.

---

# 🏗 **MCP Server Design for Sruja**

Your MCP server should expose **high-level, AI-friendly functions** that hide DSL complexity.

Below are the recommended MCP capabilities.

---

# 🧩 **1. loadArchitecture(file)**

Read `.sruja` file → return:

* parsed AST
* semantic model
* list of systems/containers/components
* list of relations
* metadata

### MCP:

```
call("loadArchitecture", { path: "billing.sruja" })
```

---

# 🧩 **2. explain(elementId)**

AI can request explanations of any element.

```
call("explain", { id: "BillingAPI" })
```

Returns:

* what it does
* inbound/outbound dependencies
* metadata summary
* ADR context
* journeys involving it
* warnings
* diagram link

This is HUGE for experience.

---

# 🧩 **3. modifyArchitecture(action)**

AI can **edit the architecture file safely**.

Examples:

### Add system

```
call("modifyArchitecture", {
    action: "addSystem",
    system: { id: "Payments", label: "Payments Engine" }
})
```

### Add relation

```
call("modifyArchitecture", {
    action: "addRelation",
    from: "Frontend",
    to: "BillingAPI",
    label: "Requests checkout"
})
```

### Add metadata

```
call("modifyArchitecture", {
    action: "addMetadata",
    target: "BillingAPI",
    key: "team",
    value: "Payments"
})
```

AI != string manipulation.
Your MCP *modifies AST safely*.

---

# 🧩 **4. diagram(type, element)**

Generate diagrams via MCP.

```
call("diagram", { type: "c4-container", root: "Billing" })
```

Return:

* SVG / PNG / JSON graph
* or structured DiagramModel for frontends

---

# 🧩 **5. validate()**

Run all validations including plugin validations.

Return:

* errors
* warnings
* suggestions

AI can use this to self-correct.

---

# 🧩 **6. diff(variantA, variantB)**

```
call("diff", {
    a: "option-a.sruja",
    b: "option-b.sruja"
})
```

Return:

* added elements
* removed elements
* changed metadata
* changed relations
* variant rating

AI can summarize changes beautifully.

---

# 🧩 **7. query(queryExpression)**

Use a simple architecture query language:

Examples:

```
find containers where metadata.team == "Billing"
find relations where from == "Frontend"
find systems touching "InventoryDB"
```

MCP returns structured JSON.

---

# 🧩 **8. generateADR(subject)**

```
call("generateADR", {
    subject: "Switch from REST to gRPC"
})
```

MCP generates an ADR *template*, not text.
AI fills the text.

---

# 🧩 **9. search(term)**

Fuzzy search through:

* IDs
* labels
* metadata
* ADRs
* requirements

---

# 🧩 **10. plugin operations**

Expose plugin metadata schemas:

```
call("listPlugins")
call("getPluginMetadata", { plugin: "security" })
```

AI will use this to suggest metadata keys.

---

# 🚀 **MCP Server Implementation Plan (Go)**

---

## **Phase 1 — Core Infrastructure**

* Load file
* Parse with tolerant mode
* Build semantic model
* Expose MCP endpoints:

  * loadArchitecture
  * explain
  * validate
  * diagram (JSON mock for now)

Time: 1–2 days.

---

## **Phase 2 — Editing Engine**

Build safe AST modification APIs:

* add/remove/modify system
* containers
* relations
* metadata
* ADRs
* requirements

This unlocks AI “write” capabilities.

---

## **Phase 3 — Diagram Engine**

Integrate:

* Mermaid
* Graphviz
* C4 plugin

Return JSON or SVG.

---

## **Phase 4 — Variant Support**

Add `diff`.

---

## **Phase 5 — Plugin Integration**

Allow plugins to contribute:

* metadata keys
* validations
* explain hooks
* diagram augmentations

---

# 🧠 How AI Will Use This MCP

ChatGPT will:

* load the architecture
* understand its structure
* run validation before giving advice
* modify architecture safely
* generate diagrams
* compare variants
* find missing metadata
* suggest improvements
* generate ADRs
* fix errors

Developers can do:

> “Split BillingAPI into BillingAPI + BillingWorker.”

AI calls modify→diagram→validate automatically.

This is **true Architecture Copilot**.

---

# ⭐ FINAL ANSWER SUMMARY

MCP gives you:

### ✔ AI-driven architecture modeling

### ✔ Safe, correct AST editing

### ✔ Explainable architecture

### ✔ Automatic diagram generation

### ✔ Plugin-powered metadata intelligence

### ✔ Zero-friction natural-language workflows

### ✔ Variant diffing & UX

### ✔ Real-time validation

YOU become the first DSL with **native AI-first design**, not an afterthought.


Below is a **complete, production-level MCP Server Design for Sruja**, built specifically to support:

* AI-assisted architecture modeling
* Safe AST editing
* Query, validate, and diagram generation
* Metadata- and plugin-driven extensibility
* Multi-variant modeling
* Natural-language explanation
* Powerful IDE/LSP integrations
* Full support for ChatGPT, Cursor, Claude, VSCode Copilot tools

This is the **blueprint** you should follow to make Sruja an **AI-native architecture modeling platform**.

---

# 🏛️ **SRUJA MCP SERVER — FULL DESIGN**

We break the design into:

1. High-level Architecture
2. Capabilities
3. Actions
4. Data Models
5. Plugin Hooks
6. Directory structure
7. Server lifecycle
8. Example requests & responses
9. Full API specification

---

# 1. **HIGH-LEVEL ARCHITECTURE**

```
┌──────────────────────────┐
│     MCP Sruja Server     │
├──────────────────────────┤
│  File Loader              │
│  Parser (tolerant)        │
│  Semantic Model Builder   │
│  Variant Manager          │
│  Query Engine             │
│  Validation Engine        │
│  Diagram Generator        │
│  Plugin Framework         │
│  AST Edit Engine          │
└──────────────────────────┘
          ↑   ↓
     MCP Protocol
          ↑   ↓
┌──────────────────────────┐
│      AI Client           │     
│  ChatGPT / Cursor / IDE  │
└──────────────────────────┘
```

The AI calls structured MCP functions.
Your server does **semantic work**, not the AI.

---

# 2. **CORE CAPABILITIES (API SURFACE)**

These are the MCP functions your server must expose.

### **1. loadArchitecture**

Return AST + semantic model + metadata.

### **2. explain**

Explain any element.

### **3. diagram**

Generate diagram (C4 container, system view, graph, etc.)

### **4. query**

Query architecture using a small query DSL.

### **5. validate**

Run core + plugin validations.

### **6. diff**

Compare variants or files.

### **7. modifyArchitecture**

Safe AST editing operations.

### **8. analyzeMetadata**

Let plugins inspect metadata.

### **9. listVariants**

Return all architecture files and their variants.

### **10. listPlugins**

Return installed plugins + their capabilities.

---

# 3. **DETAILED ACTION DESIGN**

Below is the complete API for each action.

---

## 📘 **Action: loadArchitecture**

### Request

```json
{
  "path": "billing.sruja"
}
```

### Response

```json
{
  "architecture": {
    "name": "Billing",
    "systems": [...],
    "containers": [...],
    "components": [...],
    "relations": [...],
    "metadata": {...},
    "adrs": [...],
    "requirements": [...]
  },
  "ast": {},
  "semanticModel": {}
}
```

---

## 📘 **Action: explain**

### Request

```json
{
  "id": "BillingAPI"
}
```

### Response

```json
{
  "summary": "BillingAPI is the primary API container handling invoicing.",
  "inbound": [...],
  "outbound": [...],
  "metadata": {...},
  "inJourneys": [...],
  "relatedADRs": [...],
  "warnings": [...]
}
```

---

## 📘 **Action: diagram**

### Request

```json
{
  "type": "c4-container",
  "root": "Billing"
}
```

### Response

```json
{
  "diagram": {
    "format": "svg",
    "data": "<svg>...</svg>"
  }
}
```

Or JSON graph:

```json
{
  "diagram": {
    "nodes": [...],
    "edges": [...]
  }
}
```

---

## 📘 **Action: query**

### Request

```json
{
  "q": "find containers where metadata.team == \"billing\""
}
```

### Response

```json
{
  "results": [
    { "id": "BillingAPI", "label": "Billing API" }
  ]
}
```

---

## 📘 **Action: validate**

### Request

```json
{
  "path": "billing.sruja"
}
```

### Response

```json
{
  "errors": [...],
  "warnings": [...],
  "suggestions": [...]
}
```

---

## 📘 **Action: diff**

### Request

```json
{
  "a": "option-a.sruja",
  "b": "option-b.sruja"
}
```

### Response

```json
{
  "elementsAdded": [...],
  "elementsRemoved": [...],
  "relationsChanged": [...],
  "metadataChanged": [...]
}
```

---

## 📘 **Action: modifyArchitecture**

This is where the AI can safely mutate architecture.

### Example: Add System

```
{
  "action": "addSystem",
  "targetFile": "billing.sruja",
  "system": {
    "id": "Fraud",
    "label": "Fraud Detection"
  }
}
```

### Example: Add Relation

```
{
  "action": "addRelation",
  "from": "BillingAPI",
  "to": "FraudAPI",
  "label": "Validates charges"
}
```

### Response

```json
{ "status": "ok", "fileUpdated": true }
```

The server rewrites the AST, not the AI.

---

# 4. **DATA MODELS (Used in Requests & Responses)**

Your MCP server must use the following consistent internal structures:

### **Element**

```
id
label
description
metadata
type (system | container | component | person | datastore | queue)
location (file, line, col)
```

### **Relation**

```
from
to
label
verb
type
resolved pointers
```

### **Metadata**

```
map[string]string
```

### **Architecture**

```
systems
containers
components
persons
datastores
queues
adrs
requirements
relations
metadata
```

---

# 5. **PLUGIN FRAMEWORK FOR MCP**

Your MCP server must load plugins that can:

### ✔ Add metadata keys

### ✔ Add autocomplete suggestions

### ✔ Provide validation rules

### ✔ Provide diagram layers

### ✔ Provide explain hooks

### ✔ Provide query filters

### Plugin Interface (Go)

```go
type Plugin interface {
    Name() string
    MetadataSchema() []MetadataDescriptor
    Validate(*SemanticModel, *Diagnostics)
    Explain(Element) PluginExplain
    DiagramLayer(*DiagramModel) *DiagramLayer
    QueryHook(QueryContext) []QueryResult
    ModifyHook(ModifyContext) error
}
```

The MCP server will pass plugin-derived data into:

* autocomplete
* explain
* validate
* query
* modify

---

# 6. **DIRECTORY STRUCTURE FOR SERVER**

```
mcp/
 ├── server.go
 ├── protocol/
 │    ├── handlers.go
 │    ├── requests.go
 │    ├── responses.go
 │    └── schema.json
 ├── core/
 │    ├── parser.go
 │    ├── semantic.go
 │    ├── resolver.go
 │    ├── validator.go
 │    ├── query.go
 │    ├── diagram.go
 │    └── ast-editor.go
 ├── plugins/
 │    ├── loader.go
 │    └── *.so (runtime loaded)
 └── utils/
      ├── file.go
      ├── logging.go
      └── config.go
```

---

# 7. **SERVER LIFECYCLE**

### 1. Start server

Loads:

* plugins
* metadata schemas
* config

### 2. Connect over MCP

Handshake.

### 3. On each request

* load file (cached)
* parse -> semantic model
* run resolver
* run validations
* dispatch plugin hooks
* return structured JSON

### 4. On modify actions

* AST rewrite
* file save
* re-parse & revalidate
* return updated model

---

# 8. **EXAMPLE MCP INTERACTIONS**

---

## **AI: “Add a new container to Billing system.”**

MCP:

```
modifyArchitecture({
  action: "addContainer",
  system: "Billing",
  container: {
     id: "BillingWorker",
     label: "Background Processor"
  }
})
```

---

## **AI: “Explain how billing interacts with auth.”**

MCP:

```
explain({ id: "BillingAPI" })
```

AI uses returned data to generate explanation.

---

## **AI: “Generate a diagram of the billing architecture.”**

MCP:

```
diagram({
  type: "c4-container",
  root: "Billing"
})
```

---

## **AI: “List metadata keys supported in this project.”**

MCP:

```
listPlugins()
```

---

# 9. **FULL API SPECIFICATION (COMPACT VERSION)**

### Capabilities

```
loadArchitecture
explain
diagram
query
validate
diff
modifyArchitecture
listPlugins
listVariants
analyzeMetadata
```

### Modify Actions

```
addSystem
removeSystem
addContainer
removeContainer
addComponent
removeComponent
addRelation
removeRelation
addMetadata
updateMetadata
removeMetadata
addADR
addRequirement
```

---

# 🏆 **FINAL SUMMARY**

The Sruja MCP server should:

### ✔ Provide a high-level semantic API for AI

### ✔ Safely mutate architecture via AST editing

### ✔ Integrate metadata & plugin ecosystem

### ✔ Support diagrams, validation, querying

### ✔ Enable variant management

### ✔ Produce rich explanations & structured data

### ✔ Power IDE/LSP and AI copilot interactions

This makes Sruja **AI-native**, **safe**, and **enterprise-ready**.

Below is the **complete, production-quality Query Language Design** for Sruja — a minimal, intuitive, powerful DSL that lets MCP, IDEs, and AI tools query the architecture model.

This is equivalent to what Terraform CDK, K8s selectors, GraphQL filters, and Structurizr’s view expressions provide — but tailored for your architecture model and metadata-driven extensibility.

---

# 🧠 **GOALS of the Sruja Query Language (SQL)**

The query language must:

### ✔ Be simple and readable (developer-friendly)

### ✔ Be easily AI-generatable (LLM-friendly)

### ✔ Be fast to evaluate

### ✔ Support autocomplete in your LSP

### ✔ Integrate with metadata

### ✔ Support plugins to contribute new fields/predicates

### ✔ Work across architectures and variants

AND it must NOT:

* be Turing complete
* become a complex programming language
* require new DSL grammar

This is strictly a *query/filter language*, like Kubernetes selectors + SQL WHERE + GraphQL filters.

---

# 🌟 **THE SQL (Sruja Query Language) — Overview**

**Main syntax pattern:**

```
find <type> [where <expression>]
```

### Supported types:

```
systems
containers
components
persons
datastores
queues
relations
metadata
adrs
requirements
journeys
steps
anything      (global search across all)
```

### Examples:

```
find containers
find systems where id == "Billing"
find relations where from == "Frontend"
find containers where metadata.team == "Payments"
find components where technology == "Go"
find systems where tag contains "critical"
find steps where to == "AuthService"
```

This is both **human-readable** and **AI-friendly**.

---

# 🧩 **1. Query Types**

## **find <elementType>**

```
find systems
find containers
find relations
find anything
```

`anything` returns a best-effort list of matches across types.

---

# 🧩 **2. WHERE Expressions**

These expressions allow filtering based on:

### ✔ Built-in fields:

```
id
label
description
technology
type
tags[]
```

### ✔ Metadata:

```
metadata.<key>
```

### ✔ Plugin-exposed fields:

Plugins can register:

* derived fields
* computed fields
* traceability fields

Example:

```
security.risk
cloud.region
performance.tier
```

---

# 🧩 **3. Expression Operators**

### Equality:

```
==
!=
```

### String matching:

```
contains
starts_with
ends_with
matches (regex)
```

### Set membership:

```
in
not in
```

### Boolean:

```
and
or
not
```

### Parentheses:

```
( ... )
```

---

# 🧩 **4. Example Queries**

---

## 🔍 **Basic Structural Queries**

### All containers:

```
find containers
```

### Systems with Billing in name:

```
find systems where id contains "Billing"
```

### All components using Go:

```
find components where technology == "Go"
```

---

## 🔍 **Metadata Queries**

### Containers owned by Payments team:

```
find containers where metadata.team == "Payments"
```

### High-criticality systems:

```
find systems where metadata.criticality in ["high", "critical"]
```

### Public-facing APIs:

```
find containers where metadata.public == "true"
```

---

## 🔍 **Relationship Queries**

### Who does Frontend call?

```
find relations where from == "Frontend"
```

### Who calls BillingAPI?

```
find relations where to == "BillingAPI"
```

### All relations labeled “Reads”

```
find relations where verb == "reads"
```

---

## 🔍 **Cross-Architecture Queries**

### Elements in the Billing architecture:

```
find anything where namespace == "Billing"
```

### Imported system elements:

```
find containers where namespace != "Root"
```

---

## 🔍 **Journey Queries**

### What journeys involve the user?

```
find journeys where steps.from == "User"
```

### Who calls AuthService in flows?

```
find steps where to == "AuthService"
```

---

# 🧱 **5. Query Execution Model**

The query engine performs:

### **1. Load and resolve architecture model**

All imports, variants, plugins applied.

### **2. Normalize metadata**

Metadata becomes `map[string]string`.

### **3. Build a searchable index:**

* id index
* label index
* relation graph
* metadata index
* plugin fields

### **4. Execute AST of Query Expression**

Example query:

```
find containers where metadata.team == "Billing"
```

Query AST:

```
Find {
  Type: containers
  Filter: Equals(
      Field("metadata.team"),
      String("Billing")
  )
}
```

---

# 🧠 **6. Query Language Grammar (EBNF)**

This is the formal grammar for implementation.

```
query           = "find", type, [ whereClause ] ;
type            = ident ;
whereClause     = "where", expr ;
expr            = orExpr ;
orExpr          = andExpr, { "or", andExpr } ;
andExpr         = unaryExpr, { "and", unaryExpr } ;
unaryExpr       = [ "not" ], primaryExpr ;
primaryExpr     = comparison
                 | "(" expr ")" ;
comparison      = field, comparator, value
                 | field, matchOp, string ;
field           = ident, { ".", ident } ;
comparator      = "==" | "!=" | "in" | "not", "in" ;
matchOp         = "contains" | "starts_with" | "ends_with" | "matches" ;
value           = string | number | list ;
list            = "[", value, { ",", value }, "]" ;
```

---

# 🛠 **7. Query Engine API Design (Go)**

```
type QueryResult struct {
    Elements []Element
    Raw      any
}

type QueryEngine struct {
    Model        *SemanticModel
    PluginFields []PluginField
}

func (q *QueryEngine) Execute(query string) (QueryResult, error)
```

Plugin fields:

```
type PluginField struct {
    Name string                // e.g., "security.risk"
    Getter func(e Element) any // returns computed value
}
```

---

# 🔌 **8. Plugin Extension Hooks**

Plugins can extend the query language by:

### ✔ Adding new fields

Example:

```
cloud.region
security.classification
performance.slo
```

### ✔ Adding custom functions

Example:

```
within_hops(Frontend, 2)
is_exposed()
requires_encryption()
has_event_flow()
```

### ✔ Adding new query types

Example:

```
find dataflows where ...
find boundaries where ...
```

### ✔ Providing domain-specific filters

---

# 🎨 **9. Autocomplete and LSP Integration**

When typing:

```
find containers where meta|
```

LSP suggests:

* metadata keys from core
* plugin metadata keys

When typing:

```
find |
```

LSP suggests types:

* systems
* containers
* relations
* anything

When typing:

```
metadata.team == |
```

Autocomplete suggests values from:

* metadata annotations seen in project
* plugin enum values

---

# 🧩 **10. Edge Cases & Advanced Features**

### 10.1 Numeric metadata:

```
metadata.cost_per_tx > 0.02
```

### 10.2 Existence checks:

```
metadata.rate_limit exists
```

### 10.3 Negation:

```
not metadata.criticality in ["high"]
```

### 10.4 Relations queries via join syntax:

```
find systems where out.to == "AuthService"
find systems where in.from == "Frontend"
```

---

# 🏆 **FINAL SUMMARY — The SQL (Sruja Query Language)**

Your SQL is:

### ✔ Human-friendly

### ✔ AI-friendly

### ✔ Extensible

### ✔ Metadata-aware

### ✔ Plugin-integrated

### ✔ Fast

### ✔ Ideal for LSP

### ✔ Ideal for MCP

Syntax:

```
find <types> where <expr>
```

Expressions support:

* `==`, `!=`, `in`, `contains`, `starts_with`, regex
* metadata access (`metadata.<key>`)
* plugin fields (`cloud.region`, `security.risk`)
* relational introspection (via relations)
* deep nested conditions

This is the **perfect query language for architecture analysis**.

---
How to implement with Participle – step-by-step
1. Define grammar structs

E.g., in Go:

type Query struct {
    Find   string        `parser:"'find' @Ident"`
    Type   string        `parser:"@Ident"`
    Where  *WhereClause  `parser:"( 'where' @@ )?"`
}

type WhereClause struct {
    Expr Expr `parser:"@@"`
}

type Expr interface { /* base */ }

type BinaryExpr struct {
    Left  Expr   `parser:"@@"`
    Op    string `parser:"@('==' | '!=' | 'in' | 'contains' | ...)"`
    Right Value  `parser:"@@"`
}

type Field struct {
    Path []string `parser:"@Ident ( '.' @Ident )*"`
}

type Value struct {
    String *string  `parser:"@String"`
    Number *float64 `parser:"| @Float"`
    List   *[]Value `parser:"| '[' @@ ( ',' @@ )* ']'"`
}

2. Build parser
parser, err := participle.Build(&Query{})

3. Parse queries
q := &Query{}
err = parser.ParseString("", "find containers where metadata.team == \"Payments\"", q)

4. Evaluate AST against semantic model

Walk the AST, for each element of the requested type, check the Where condition by evaluating field paths, comparing values, checking metadata map, etc.

5. Integrate with LSP

When user types "find " → autocomplete types

After "where " → autocomplete fields (based on type + metadata keys)

Use parsed AST to validate syntax as user types

6. Support plugin-fields

When building field suggestions, include dynamic fields from plugins (e.g., security.risk, cloud.region). In evaluation, refer to plugin-provided value getters.

📝 Summary

Using Participle is a very good match for your query language requirement. It allows you to:

define the grammar cleanly in Go

parse queries into ASTs

integrate it with your semantic model

support LSP/autocomplete

support plugin-extension of fields
