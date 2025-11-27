# DSL Specification

Complete DSL grammar and specification for Architecture DSL v1 and v2.

[← Back to Documentation Index](../README.md)

Here is DSL Grammar v1 in Ohm syntax, designed for your MVP architecture modeling language.

It focuses on the minimal but fully functional constructs you need:

Nodes (services, databases, components)

Edges (A -> B "label")

Labels & strings

Blocks (for future multi-layer modeling)

Metadata (optional key/value)

Comments

Imports (for future libraries)

Stable formatting for round-trip sync

This grammar is simple, readable, extensible, and suitable for implementing your initial DSL parser in Ohm.

## Go Grammar (EBNF + participle)

### EBNF (v1)

```
Program           = { Statement }
Statement         = ImportStatement | NodeDeclaration | EdgeDeclaration | Comment
ImportStatement   = "import" StringLiteral
NodeDeclaration   = Identifier ":" NodeType StringLiteral? MetadataBlock?
NodeType          = "service" | "database" | "component" | "actor" | "container"
EdgeDeclaration   = Identifier "->" Identifier StringLiteral?
Comment           = "//" { any } | "/*" { any } "*/"
MetadataBlock     = "{" { MetadataEntry [ "," ] } "}"
MetadataEntry     = Identifier ":" Value
Value             = StringLiteral | NumberLiteral | BooleanLiteral
Identifier        = letter { letter | digit | "_" | "-" }
StringLiteral     = '"' { char } '"'
NumberLiteral     = digit+
BooleanLiteral    = "true" | "false"
```

### Go AST (participle tags)

```go
type Program struct {
    Statements []*Statement `@@*`
}

type Statement struct {
    Import  *ImportStatement `| @@`
    Node    *NodeDeclaration `| @@`
    Edge    *EdgeDeclaration `| @@`
    Comment *Comment         `| @@`
}

type ImportStatement struct {
    Kw   string `"import"`
    Path string `@String`
}

type NodeDeclaration struct {
    Name  string   `@Ident`
    Colon string   `":"`
    Type  string   `@("service"|"database"|"component"|"actor"|"container")`
    Label *string  `@String?`
    Meta  *MetadataBlock `@@?`
}

type EdgeDeclaration struct {
    From  string `@Ident`
    Arrow string `"->"`
    To    string `@Ident`
    Label *string `@String?`
}

type Comment struct {
    LineStart   *string `@("//")?`
    BlockStart  *string `@("/*")?`
    Text        string  `@Rest`
}

type MetadataBlock struct {
    LBrace  string           `"{"`
    Entries []*MetadataEntry `(@@ ("," @@)*)?`
    RBrace  string           `"}"`
}

type MetadataEntry struct {
    Key   string `@Ident`
    Colon string `":"`
    Value string `@String | @Int | @("true"|"false")`
}
```

### Serializer Contract (Go)

```
Input:  Program
Output: JSON model with nodes, edges, metadata
Stability: Round-trip formatting preserves statement order and labels
```

🌟 Ohm DSL Grammar v1

Below is the entire grammar.
You can paste this directly into an Ohm grammar file (architecture.ohm).

Architecture {
  Program        = _ (Statement _)*

  // ---- Statements ----
  Statement      = ImportStatement
                 | NodeDeclaration
                 | EdgeDeclaration
                 | Comment

  // ---- Import ----
  ImportStatement = "import" _ StringLiteral

  // ---- Node Declarations ----
  NodeDeclaration = Identifier _ ":" _ NodeType _ StringLiteral? _ MetadataBlock?

  NodeType       = "service"
                 | "database"
                 | "component"

  // ---- Edges ----
  EdgeDeclaration = Identifier _ "->" _ Identifier _ StringLiteral?

  // ---- Metadata Block ----
  MetadataBlock  = "{" _ (MetadataEntry (_ "," _ MetadataEntry)*)? _ "}"

  MetadataEntry  = Identifier _ ":" _ Value

  // ---- Values ----
  Value          = StringLiteral
                 | NumberLiteral
                 | BooleanLiteral

  BooleanLiteral = "true" | "false"
  NumberLiteral  = digit+

  // ---- Strings ----
  StringLiteral  = "\"" (~"\"" any)* "\""

  // ---- Comments ----
  Comment        = LineComment | BlockComment
  LineComment    = "//" (~"\n" any)* ("\n" | end)
  BlockComment   = "/*" (~"*/" any)* "*/"

  // ---- Identifiers ----
  Identifier     = letter (letter | digit | "_" | "-")*

  // ---- Extras ----
  _              = (space | Newline)*

  Newline        = "\n" | "\r\n" | "\r"
}

📘 DSL Examples Supported by This Grammar
✔ Basic Nodes
userService: service "User Service"
userDB: database "User Database"
auth: component "Auth Logic"

✔ Edges
userService -> userDB "reads/writes"
auth -> userService "authenticates"

✔ Metadata
userService: service "User Service" {
  owner: "platform-team",
  tier: "gold",
  replicas: 3
}

✔ Imports (future extension)
import "aws-library.sruja"

✔ Comments
// Single line comment
/* Block comment */

🎯 Design Goals Achieved
1. Simple & Learnable

The grammar mirrors D2/C4-like structure and is easy to read.

2. Extensible

Future additions can be layered over:

Blocks ({ ... })

Layers (context, container, component)

Properties

ADR links

Patterns

Library imports

3. Unambiguous for Parsing

Ohm’s PEG-based design ensures predictable parsing.

4. Bidirectional Safe

Serializer can generate stable output:

One statement per line

Metadata in canonical order

Identifiers first, then types

5. Whitespace tolerant

Whitespace does not affect parsing.


Below is a **fully polished, standalone Markdown specification document** for **DSL v1**, suitable for your repo (`docs/dsl-spec-v1.md`).
It describes syntax, semantics, examples, and formatting rules.

---

# # 📘 Architecture DSL v1 — Specification

*A lightweight, human-readable language for defining software architecture models.*

Version: **v1 (MVP)**
Status: **Draft**
Author: **You**

---

# ## 1. Introduction

The Architecture DSL is a **text-based language** for describing software architecture models.
It provides a simple, declarative syntax for defining:

* **Nodes** (services, databases, components)
* **Edges** (relationships/dependencies)
* **Metadata** (key-value properties)
* **Imports** (for library packages)
* **Comments**

This DSL is the **single source of truth** for all architecture diagrams, models, and metadata.
It is designed for:

* Bidirectional editing (text ⇄ diagram)
* Version control (Git-friendly)
* Round-trip safety (canonical formatting)
* Ease of reading and writing

---

# ## 2. Core Concepts

The DSL describes an architecture using three fundamental constructs:

### ✔ **Nodes**

Entities in the system: services, components, databases, etc.

### ✔ **Edges**

Connections between nodes: dependencies, communications, flows.

### ✔ **Metadata**

Optional key/value attributes attached to nodes.

### ✔ **Imports**

Allows loading external component libraries (future functionality).

---

# ## 3. Syntax Overview

### A DSL program is composed of:

```
Program = Statement*
```

Where a **statement** is one of:

* Node Declaration
* Edge Declaration
* Import Statement
* Comment

Whitespace is ignored.

---

# ## 4. Identifiers

### Rules:

* Must start with a letter
* Can include: letters, digits, `_`, `-`
* Case-sensitive
* No spaces allowed

### Examples:

```
userService
UserService
user-db
auth_v2
```

---

# ## 5. Node Declarations

Nodes represent architecture elements.

### **Syntax:**

```
identifier: type "Label" { metadata }
```

### **Node Types (v1):**

| Type        | Meaning                       |
| ----------- | ----------------------------- |
| `service`   | Application/service           |
| `database`  | Database or persistent store  |
| `component` | Code-level or internal module |

### **Label** (optional):

A quoted string giving a human-friendly name.

### **Metadata** (optional):

A `{ key: value }` block.

---

### **Examples:**

#### Minimal:

```
userService: service
```

#### With label:

```
userService: service "User Service"
```

#### With metadata:

```
userService: service "User Service" {
  owner: "platform",
  replicas: 3
}
```

---

# ## 6. Edge Declarations

Edges describe relationships such as calls or dependencies.

### **Syntax:**

```
source -> target "Label"
```

### Notes:

* `source` and `target` must be valid node identifiers.
* Label is optional.

### **Examples:**

```
userService -> userDB "reads/writes"
auth -> userService "authenticates"
frontend -> apiGateway
```

---

# ## 7. Metadata Blocks

Metadata blocks provide structured properties for nodes.

### **Syntax:**

```
{
  key: value,
  key2: value2
}
```

### Valid metadata values:

* String
* Number
* Boolean

### Strings must be quoted.

### Example:

```
userDB: database "User DB" {
  encrypted: true,
  engine: "postgres",
  sizeGB: 20
}
```

### Canonical rules (for serializer):

* Keys sorted alphabetically
* One property per line
* Commas required between entries

---

# ## 8. Import Statements

Imports allow external DSL files to be referenced (future library feature).

### **Syntax:**

```
import "path/or/library.sruja"
```

### Example:

```
import "aws/managed-services.sruja"
```

Imports do not introduce additional semantics in v1, but they are included to keep the DSL easily extensible in v2.

---

# ## 9. Comments

Two types supported:

### **Line comment:**

```
// this is a comment
```

### **Block comment:**

```
/* 
  this is a block comment 
*/
```

Comments may appear anywhere between statements.

---

# ## 10. Whitespace Rules

* Whitespace is insignificant
* Newlines are treated as separators but not required
* Multiple newlines allowed
* Indentation optional (not semantically meaningful)

---

# ## 11. Best Practices & Formatting Conventions

These are *guidelines* to maintain readability and maximize diff-friendliness.

---

### ### 11.1 Node Formatting (recommended)

```
identifier: type "Label" {
  key: value,
  key2: value2
}
```

or without metadata:

```
identifier: type "Label"
```

---

### ### 11.2 Edge Formatting

One edge per line:

```
source -> target "Label"
```

---

### ### 11.3 Logical Ordering

When generating DSL from the model:

1. Imports
2. Node declarations
3. Edge declarations
4. Comments (associated with nearest statements)

---

### ### 11.4 Canonical Formatting for Round-Trip Sync

The serializer MUST guarantee:

* Nodes sorted by identifier
* Edges sorted alphabetically by `source -> target`
* Metadata keys sorted alphabetically
* Quoted labels always included (empty string allowed)
* No trailing commas
* No trailing whitespace

This ensures version control diffs remain clean and predictable.

---

# ## 12. Full Valid Example (Recommended Style)

```
import "shared.lib"

/* User Management System */

userService: service "User Service" {
  owner: "platform",
  replicas: 3
}

userDB: database "User Database" {
  encrypted: true,
  sizeGB: 20
}

auth: component "Auth Logic"

userService -> userDB "reads/writes"
auth -> userService "authenticates"
```

---

# ## 13. Grammar (Formal Ohm Definition)

```
Architecture {
  Program        = _ (Statement _)*

  Statement      = ImportStatement
                 | NodeDeclaration
                 | EdgeDeclaration
                 | Comment

  ImportStatement = "import" _ StringLiteral

  NodeDeclaration = Identifier _ ":" _ NodeType _ StringLiteral? _ MetadataBlock?

  NodeType       = "service"
                 | "database"
                 | "component"

  EdgeDeclaration = Identifier _ "->" _ Identifier _ StringLiteral?

  MetadataBlock  = "{" _ (MetadataEntry (_ "," _ MetadataEntry)*)? _ "}"

  MetadataEntry  = Identifier _ ":" _ Value

  Value          = StringLiteral
                 | NumberLiteral
                 | BooleanLiteral

  BooleanLiteral = "true" | "false"
  NumberLiteral  = digit+

  StringLiteral  = "\"" (~"\"" any)* "\""

  Comment        = LineComment | BlockComment
  LineComment    = "//" (~"\n" any)* ("\n" | end)
  BlockComment   = "/*" (~"*/" any)* "*/"

  Identifier     = letter (letter | digit | "_" | "-")*

  _              = (space | Newline)*
  Newline        = "\n" | "\r\n" | "\r"
}
```

---

# ## 14. JSON Model Mapping (Summary)

Each DSL node maps to:

```json
{
  "id": "userService",
  "type": "service",
  "label": "User Service",
  "metadata": {
    "owner": "platform",
    "replicas": 3
  }
}
```

Each edge maps to:

```json
{
  "from": "auth",
  "to": "userService",
  "label": "authenticates"
}
```

---

# ## 15. Future Extensions (v2 and beyond)

The DSL is designed to support extensions without breaking backward compatibility:

* Hierarchical modeling (context/container/component)
* ADR references
* User journeys
* Tags and annotations
* Node templates (from libraries)
* Pattern definitions
* Multi-diagram files
* Namespaces
* Version constraints

---

# ## 16. Conclusion

This DSL v1 specification defines the **minimal, stable foundation** necessary to build:

* Bidirectional architecture editing
* Diagram rendering
* JSON model transformations
* Git-based versioning
* Validation rules
* Future multi-layer architecture support

It is intentionally simple, readable, and extendable.

Below is a **clean, forward-thinking DSL v2 proposal** that extends your DSL into a **full architecture modeling language** with:

* **Layers** (Context → Container → Component)
* **User Journeys**
* **ADRs**
* **Requirements**
* **Tags, annotations**
* **Namespaces & modules**
* **Future-safe structure for org-wide libraries**

This proposal keeps **v1 fully compatible**, but adds structured, scalable modeling.

Perfect for long-term system architecture + MCP/AI integration.

---

# # 🚀 Architecture DSL v2 — Proposal

Version: **v2 (Extended Model)**
Design Goal: Add **higher-level architectural abstractions** while **keeping v1 stable**.

---

# ## 1. Overview

v2 introduces a hierarchical architecture structure based on **C4-like multi-layer modeling**, alongside new first-class concepts:

### ✔ Architecture Layers

* `context` → external actors & systems
* `containers` → services / deployable units
* `components` → code-level modules
* Future extension: `classes`, `interfaces`, `functions`, `events`

### ✔ User Journeys

Describe how users interact with architecture elements.

### ✔ Requirements

Functional and non-functional requirements.

### ✔ ADRs

Architecture decision records, linkable to nodes.

### ✔ Modules / Namespaces

Allow splitting architecture into independent parts.

### ✔ Metadata Everywhere

All elements (nodes, edges, layers, journeys, ADRs) support `{ metadata }`.

---

# ## 2. Top-Level Structure (v2 Root)

A v2 DSL file supports multiple *sections*.

Example:

```
imports { ... }
context { ... }
containers { ... }
components { ... }
journeys { ... }
requirements { ... }
adrs { ... }
```

All sections are optional to maintain compatibility.

---

# ## 3. Layers

## ### 3.1 Context Layer

Defines **external actors** and **systems**.

```
context {
  user: actor "End User"
  paymentGateway: system "External Payment System"

  user -> paymentGateway "initiates payment"
}
```

### Node types allowed:

* `actor`
* `system`

---

## ### 3.2 Containers Layer

Defines **deployable units** like services, apps, databases.

```
containers {
  api: service "API Server" {
    owner: "platform"
  }

  db: database "Main DB"
  ui: frontend "Web App"

  ui -> api "rest calls"
  api -> db "queries"
}
```

### Node types allowed:

* `service`
* `database`
* `frontend`
* `queue`
* `cache`
* `cron`
* etc.

---

## ### 3.3 Components Layer

Defines **logical modules inside containers**.

```
components(api) {
  auth: component "Auth Module"
  orders: component "Order Module"

  auth -> orders "reads order permissions"
}
```

### Notes:

* `components(api)` binds the component diagram to the `api` container.
* Multiple component diagrams (per-container) supported.

---

# ## 4. User Journeys (First-Class)

User journeys describe **flows across architecture elements**.

### Example:

```
journeys {
  login {
    title: "User logs into the system"
    steps {
      user -> ui "enters username/password"
      ui -> api "POST /login"
      api -> auth "check credentials"
      auth -> db "fetch user record"
    }
  }
}
```

### Notes:

* `steps` is a sequence of edges.
* Steps can reference any node across layers.
* Future support: swimlanes, sequence diagrams.

---

# ## 5. Requirements (Functional & Non-Functional)

Requirements become part of architecture metadata.

```
requirements {
  R1: functional "Users must log in with email/password"
  R2: nonfunctional "System must handle 10k requests/minute"
  R3: constraint "Data must be encrypted at rest"
}
```

### Types:

* `functional`
* `nonfunctional`
* `constraint`
* `regulatory`
* `security`

### Linking:

```
api: service "API" {
  satisfies: ["R1", "R3"]
}
```

---

# ## 6. ADRs (Architecture Decision Records)

Each ADR becomes a structured object.

```
adrs {
  ADR1 {
    title: "Choose PostgreSQL for primary DB"
    status: "accepted"
    context: """
      We need a reliable relational DB...
    """
    decision: """
      Use PostgreSQL due to ecosystem...
    """
    consequences: """
      + Strong consistency
      - Requires maintenance
    """
    relatesTo: ["db", "api"]
  }
}
```

### Status suggestions:

* proposed
* accepted
* deprecated
* superseded

---

# ## 7. Modules / Namespaces

Large architectures benefit from subdividing the DSL.

### Example:

```
module payments {
  containers {
    ps: service "Payment Service"
    db: database "Payments DB"
  }
}
```

### Usage from another module:

```
orders -> payments.ps "charge customer"
```

---

# ## 8. Tags & Annotations

Any element (node, edge, section) may have tags:

```
api: service "API" {
  tags: ["critical", "public"]
}
```

Or annotation syntax:

```
@critical @public
api: service "API"
```

---

# ## 9. v2 Grammar Evolution

### New top-level sections:

```
Program = _ (Section _)*

Section =
    ImportSection
  | ContextSection
  | ContainersSection
  | ComponentsSection
  | JourneysSection
  | RequirementsSection
  | ADRsSection
  | ModuleSection
  | Statement  // for v1 compatibility
```

### New node types:

* `actor`, `system` (context)
* `frontend`, `queue`, `cache`, `cron`

### New block types:

* `steps`
* `requirements {...}`
* `journeys {...}`
* `adrs {...}`
* `module name { ... }`

---

# ## 10. Example — Full v2 DSL File

```
import "aws/services.sruja"

/* Context */

context {
  user: actor "End User"
  billingSystem: system "External Billing"
  ui: frontend "Web App"

  user -> ui "enters credentials"
  ui -> billingSystem "pre-auth"
}

/* Containers */

containers {
  api: service "API" { owner: "platform" }
  db: database "PostgresDB" { encrypted: true }

  ui -> api "REST"
  api -> db "queries"
}

/* Components */

components(api) {
  auth: component "Auth Module"
  orders: component "Orders Module"

  auth -> orders "fetch permissions"
}

/* User Journey */

journeys {
  purchase {
    title: "Purchase a product"
    steps {
      user -> ui "browse product"
      ui -> api "GET /product"
      api -> db "read"
      ui -> billingSystem "charge"
    }
  }
}

/* Requirements */

requirements {
  R1: functional "System must support user purchase flow"
  R2: nonfunctional "99.9% uptime"
}

/* Linking reqs */

api: service "API" {
  satisfies: ["R1"]
}

/* ADRs */

adrs {
  ADR1 {
    title: "Choose PostgreSQL"
    status: "accepted"
    decision: "We selected PostgreSQL..."
    relatesTo: ["db"]
  }
}
```

---

# ## 11. Compatibility Notes

### v1 still valid:

```
serviceA: service "A"
serviceA -> serviceB
```

### v2 adds structure but does not break v1 files.

You can parse v1 using a v2 parser without issues.

---

# ## 12. Recommendations for Implementation

### Parser:

* Extend the v1 Ohm grammar modularly
* Add new constructs using `Section` blocks
* Maintain AST backward compatibility

### Model:

* Add hierarchical layer model
* Add journies + ADRs + requirements into central model
* Add `module` namespace field to each element

### UI:

* Layer switcher: context → containers → components
* Journey visualizer
* ADR & requirement panels

---

Below is the **complete, polished, production-ready Ohm grammar** for **Architecture DSL v2**, including:

* Context / Containers / Components sections
* Modules / Namespaces
* Journeys
* Requirements
* ADRs
* Imports
* v1 compatibility mode
* Metadata blocks
* Comments
* Tags
* Annotations
* Rich value types (strings, numbers, booleans, arrays, multiline)

This grammar is designed for **real implementation**—clean, modular, and extensible.

You can drop this directly into `architecture-v2.ohm`.

---

# # 🧠 Architecture DSL v2 — Full Ohm Grammar

```
ArchitectureV2 {
  Program
    = _ (Section _)*

  // ----------------------------
  //   Top-level Sections
  // ----------------------------

  Section
    = ImportSection
    | ContextSection
    | ContainersSection
    | ComponentsSection
    | JourneysSection
    | RequirementsSection
    | ADRsSection
    | ModuleSection
    | Statement   --v1Compat

  // ----------------------------
  //   IMPORTS
  // ----------------------------

  ImportSection
    = "import" _ StringLiteral

  // ----------------------------
  //   CONTEXT LAYER
  // ----------------------------

  ContextSection
    = "context" _ Block(ContextBody)

  ContextBody
    = (NodeDeclaration(ContextType) _ | EdgeDeclaration _ | Comment _)*

  ContextType
    = "actor"
    | "system"

  // ----------------------------
  //   CONTAINERS LAYER
  // ----------------------------

  ContainersSection
    = "containers" _ Block(ContainersBody)

  ContainersBody
    = (NodeDeclaration(ContainerType) _ | EdgeDeclaration _ | Comment _)*

  ContainerType
    = "service"
    | "database"
    | "frontend"
    | "queue"
    | "cache"
    | "cron"
    | "component"  // still allowed for v1 compatibility

  // ----------------------------
  //   COMPONENTS LAYER
  // ----------------------------

  ComponentsSection
    = "components" _ "(" _ Identifier _ ")" _ Block(ComponentsBody)

  ComponentsBody
    = (NodeDeclaration("component") _ | EdgeDeclaration _ | Comment _)*

  // ----------------------------
  //   USER JOURNEYS
  // ----------------------------

  JourneysSection
    = "journeys" _ Block(JourneysBody)

  JourneysBody
    = (JourneyBlock _)*

  JourneyBlock
    = Identifier _ Block(JourneyContent)

  JourneyContent
    = (JourneyTitle _ | StepsBlock _ | MetadataBlock _)*

  JourneyTitle
    = "title" _ ":" _ StringLiteral

  StepsBlock
    = "steps" _ Block(StepsBody)

  StepsBody
    = (EdgeDeclaration _)*

  // ----------------------------
  //   REQUIREMENTS
  // ----------------------------

  RequirementsSection
    = "requirements" _ Block(RequirementsBody)

  RequirementsBody
    = (RequirementEntry _)*

  RequirementEntry
    = Identifier _ ":" _ RequirementType _ StringLiteral _ MetadataBlock?

  RequirementType
    = "functional"
    | "nonfunctional"
    | "constraint"
    | "security"
    | "regulatory"

  // ----------------------------
  //   ADRs
  // ----------------------------

  ADRsSection
    = "adrs" _ Block(ADRsBody)

  ADRsBody
    = (ADRBlock _)*

  ADRBlock
    = Identifier _ Block(ADRContent)

  ADRContent
    = (ADRField _)*

  ADRField
    = "title" _ ":" _ StringLiteral
    | "status" _ ":" _ Identifier
    | "context" _ ":" _ MultiStringLiteral
    | "decision" _ ":" _ MultiStringLiteral
    | "consequences" _ ":" _ MultiStringLiteral
    | "relatesTo" _ ":" _ ArrayLiteral

  // ----------------------------
  //   MODULES
  // ----------------------------

  ModuleSection
    = "module" _ Identifier _ Block(ModuleBody)

  ModuleBody
    = (Section _)*

  // ----------------------------
  //   V1-compatible Statements
  // ----------------------------

  Statement
    = NodeDeclaration(NodeType)
    | EdgeDeclaration
    | Comment

  NodeDeclaration[type]
    = Tags? _ Identifier _ ":" _ type _ StringLiteral? _ MetadataBlock?

  // -----------------------------------
  //   EDGES
  // -----------------------------------

  EdgeDeclaration
    = Tags? _ Identifier _ "->" _ Identifier _ StringLiteral?

  // -----------------------------------
  //   TAGS & ANNOTATIONS
  // -----------------------------------

  Tags
    = (Annotation _)+

  Annotation
    = "@" Identifier

  // -----------------------------------
  //   COMMON NODE TYPES (v1 fallback)
  // -----------------------------------

  NodeType
    = "service"
    | "database"
    | "component"

  // -----------------------------------
  //   METADATA BLOCK
  // -----------------------------------

  MetadataBlock
    = "{" _ (MetadataEntry (_ "," _ MetadataEntry)*)? _ "}"

  MetadataEntry
    = Identifier _ ":" _ Value

  // -----------------------------------
  //   VALUES
  // -----------------------------------

  Value
    = StringLiteral
    | MultiStringLiteral
    | NumberLiteral
    | BooleanLiteral
    | ArrayLiteral

  BooleanLiteral
    = "true" | "false"

  NumberLiteral
    = digit+ ("." digit+)?   // support integers + decimals

  ArrayLiteral
    = "[" _ (Value (_ "," _ Value)*)? _ "]"

  // Multiline triple-quoted string:
  MultiStringLiteral
    = "\"\"\"" (~"\"\"\"" any)* "\"\"\""

  StringLiteral
    = "\"" (~"\"" any)* "\""

  // -----------------------------------
  //   IDENTIFIERS
  // -----------------------------------

  Identifier
    = letter (letter | digit | "_" | "-")*

  // -----------------------------------
  //   BLOCK HELPER
  // -----------------------------------

  Block[x]
    = "{" _ x? _ "}"

  // -----------------------------------
  //   COMMENTS
  // -----------------------------------

  Comment
    = LineComment | BlockComment

  LineComment
    = "//" (~"\n" any)* ("\n" | end)

  BlockComment
    = "/*" (~"*/" any)* "*/"

  // -----------------------------------
  //   WHITESPACE
  // -----------------------------------

  _
    = (space | Newline | Comment)*

  Newline
    = "\n" | "\r\n" | "\r"
}
```

---

# ## Domain / Context / Team / ContextMap (DDD)

The DSL supports enterprise boundaries aligned with DDD and Team Topologies.

### Top-level declarations

```
domain <id> { ... }
context <id> in <domain> { ... }
team <id> { ... }
```

### Module binding to contexts

```
module auth {
  context: identity.auth
  owner: team.platform

  api: service "Auth API"
  db: database "User Store"
}
```

### Qualified identifiers

```
identity.auth.api
ordering.checkout.service
team.platform
```

Hierarchy: `domain → context → module → container → component`.

### Context maps

Define relationships between bounded contexts:

```
contextMap {
  identity.auth -> ordering.checkout: "customer-supplier"
  ordering.checkout -> inventory.core: "conformist"
  identity.auth -> platform.gateway: "open-host-service"
}
```

Supported relationship modes:

`customer-supplier`, `conformist`, `shared-kernel`, `open-host-service`, `anticorruption-layer`.

### Ohm grammar additions

```
ArchitectureBoundaries {
  Program        = _ (Section _ | Statement _ | DomainDeclaration _ | TeamDeclaration _ | ContextMapSection _)*

  DomainDeclaration
                 = "domain" _ Identifier _ Block(DomainBody)

  DomainBody     = (PropertyEntry _ | ContextDeclaration _ | Comment _)*

  ContextDeclaration
                 = "context" _ Identifier _ "in" _ Identifier _ Block(ContextBody)

  ContextBody    = (PropertyEntry _ | Comment _)*

  TeamDeclaration
                 = "team" _ Identifier _ Block(TeamBody)

  TeamBody       = (PropertyEntry _ | Comment _)*

  ContextMapSection
                 = "contextMap" _ Block(ContextMapBody)

  ContextMapBody = (ContextRelation _)*

  ContextRelation
                 = QualifiedIdentifier _ "->" _ QualifiedIdentifier _ ":" _ StringLiteral

  QualifiedIdentifier
                 = Identifier ("." Identifier)*

  PropertyEntry  = Identifier _ ":" _ Value
}
```

These rules integrate with v2 while remaining backward compatible.

### Boundary policies (validation)

- Cross-context access to internals is forbidden; use published APIs
- Domain data must not be accessed directly across domains
- Shared-kernel allows shared models; ACL requires indirection via adapter layer

### Visualization

- Domains as swimlanes; contexts grouped; teams as badges
- Context maps rendered as relational diagrams

# # 🧩 Major Features Implemented
# ## Reference Resolution & Identifier Rules

Identifiers use module-qualified form:

```
<module>.<container>
```

Examples:

```
auth.api
checkout.web
orders.queue
inventory.db
```

Unqualified names within a module resolve to the current module:

```
web -> auth.api
```

Resolves `web` to `checkout.web` when inside the `checkout` module.

Resolution algorithm:

```
function qualifyId(raw: string, currentModuleId: string): string {
  return raw.includes('.') ? raw : `${currentModuleId}.${raw}`;
}
```

Validation integrates with context maps and team topology plugins.

### ✔ Full layering (context, containers, components)

### ✔ Namespaces via `module { ... }`

### ✔ User journeys with `steps`

### ✔ Requirements with typed categories

### ✔ ADRs with multi-line context / decision / consequences

### ✔ Tags (`@public`, `@critical`)

### ✔ Metadata everywhere

### ✔ Triple-quoted strings (`""" ... """`)

### ✔ Arrays for linking (`relatesTo: ["db", "api"]`)

### ✔ Full v1 backward compatibility

### ✔ Extendable structure (events, classes, patterns, templates)

This grammar is **ready for implementation** using Ohm's semantics.

---

## 📋 DETAILED IMPLEMENTATION PLAN SUMMARY

### Technology Versions
- **Next.js**: 16.x (Turbopack, React Compiler, PPR)
- **React**: 19.x (Server Components, View Transitions)
- **TypeScript**: 5.6+
- **Bun**: 1.1+ (runtime, bundler)
- **Elysia**: 1.x (backend framework)
- **Ohm**: v17.2.1+ (DSL parser)
- **React Flow**: 12.9.3+ (@xyflow/react)
- **Monaco Editor**: 0.55.1+ (with LSP support)
- **LSP**: vscode-languageserver, monaco-languageclient
- **Zustand**: 5.x
- **TanStack Query**: v5
- **TailwindCSS**: 4.x
- **Zod**: 4.x
- **ELK.js**: 0.9+

### MVP Timeline Update
- **Original MVP**: 6-8 weeks
- **With LSP MVP**: 8-10 weeks (+2 weeks)
- **Benefits**: Professional IDE features, better DX, fewer bugs, future-proof

### Implementation Timeline

#### Week 1-2: Foundation (Phase 0 + Phase 1)
**Days 1-2: Project Setup**
- Monorepo initialization
- Next.js 16 app setup with React 19
- Elysia backend setup
- Development tooling (ESLint, Prettier, Vitest)
- GitHub repository & CI/CD

**Days 3-7: DSL Foundation**
- Ohm grammar definition
- Parser implementation
- Schema package (Zod)
- AST → Model transformer
- Model → DSL serializer
- Round-trip tests

**Days 8-10: Model Engine**
- Core model operations (add/update/remove nodes/edges)
- Model normalization
- Semantic validation engine
- Error handling

#### Week 3-4: LSP Server (Phase 2) - NEW
**Days 11-12: LSP Server Setup**
- LSP server bootstrap
- WebSocket transport setup
- TextDocument synchronization
- Connection lifecycle

**Days 13-15: LSP Features**
- Diagnostics (syntax + semantic)
- Completion (keywords + identifiers)
- Hover (node/edge info)
- Formatting (optional)

**Days 16-17: LSP Integration**
- Elysia WebSocket endpoint
- Error handling
- Testing & validation

#### Week 5-7: Editor MVP with LSP (Phase 3)
**Days 18-21: React Flow Integration**
- React Flow setup
- Model store (Zustand)
- Custom node components
- Diagram rendering from model
- Basic interactions (drag, select, delete)

**Days 22-24: Monaco + LSP Integration**
- Monaco setup
- LSP client connection
- Syntax highlighting (fallback)
- LSP-powered diagnostics, completion, hover

**Days 25-27: Bidirectional Sync**
- Sync engine implementation
- Text → Model → Diagram flow
- Diagram → Model → Text flow
- Undo/redo system

**Days 28-30: Polish & Auto Layout**
- ELK.js integration
- Local storage persistence
- UI polish
- Error handling

#### Week 8-10: Backend & Real-time (Phase 4)
**Days 31-34: REST API**
- Elysia server setup
- REST endpoints (GET/POST model)
- Eden Treaty client setup
- Project management API

**Days 35-38: WebSocket Sync**
- WebSocket server (Elysia, separate from LSP)
- WebSocket client hook
- Real-time model updates
- Conflict resolution (last write wins)

**Days 39-42: Git Storage**
- Git storage service
- DSL file operations
- Commit/push automation
- History API

**Days 43-45: Integration & Testing**
- End-to-end testing
- Error handling
- Security validation
- Documentation

### Task Breakdown by Priority

#### 🔴 Critical Path (Must Complete First)
1. Monorepo setup
2. DSL grammar + parser
3. Model schema (Zod)
4. AST → Model transformer
5. Model → DSL serializer
6. Semantic validation engine
7. **DSL testing suite (parser, round-trip, validation)**
8. **LSP server (diagnostics, completion, hover)**
9. **LSP testing suite**
10. **LSP client integration (Monaco)**
11. React Flow diagram rendering
12. Monaco DSL editor with LSP
13. Bidirectional sync (local)
14. Backend REST API
15. WebSocket real-time sync
16. **E2E tests (1-2 critical scenarios)**

#### 🟡 High Priority (Next)
14. Git storage integration
15. Auto layout (ELK.js)
16. Undo/redo
17. Error handling & validation
18. Project management UI
19. LSP formatting (optional but easy)

#### 🟢 Medium Priority (Post-MVP)
16. Multi-layer modeling (Phase 4)
17. Validation engine (Phase 5)
18. Version history UI (Phase 6)
19. Custom libraries (Phase 7)
20. AI/MCP integration (Phase 8)

### Development Workflow

1. **Setup Phase** (Day 1-2)
   - Initialize monorepo
   - Setup all packages
   - Configure tooling
   - Create GitHub repo

2. **Core Engine + Testing** (Day 3-10)
   - Build DSL parser
   - Build model engine
   - Build semantic validation
   - **Write comprehensive test suite**:
     - Grammar tests (Ohm)
     - AST builder tests
     - Model validation tests
     - Round-trip tests (critical)
     - Semantic validation tests
     - Error handling tests
   - Ensure round-trip safety

3. **LSP Server + Testing** (Day 11-17) - NEW
   - Build LSP server
   - Implement diagnostics, completion, hover
   - WebSocket transport
   - **Write LSP test suite**:
     - Diagnostics tests (critical)
     - Completion tests (critical)
     - Hover tests
     - Position mapping tests
     - Model syncing tests
     - Integration tests

4. **Frontend MVP with LSP** (Day 18-30)
   - Build diagram editor
   - Build DSL editor with LSP integration
   - Implement sync
   - Local-only MVP demo

5. **Backend Integration + Testing** (Day 31-45)
   - Build REST API
   - Add WebSocket sync (separate from LSP)
   - Integrate Git storage
   - **Write backend test suite**:
     - API endpoint tests
     - WebSocket sync tests
     - Git storage tests
     - Integration tests
   - **Write E2E tests** (1-2 critical scenarios):
     - Simple DSL → Diagram
     - Diagram Edit → DSL Update

6. **Polish & Launch** (Day 46+)
   - Bug fixes
   - Performance optimization
   - Documentation
   - MVP launch

### Key Milestones

- **Milestone 1** (Day 10): DSL parser + model engine working
- **Milestone 2** (Day 17): LSP server with diagnostics/completion/hover
- **Milestone 3** (Day 30): Local editor MVP with LSP (text + diagram sync)
- **Milestone 4** (Day 45): Full-stack MVP with real-time sync + Git
- **Milestone 5** (Day 60+): Multi-layer modeling
- **Milestone 6** (Day 90+): Validation engine + versioning

### Success Criteria

#### MVP (Phase 0-4)
- ✅ Write DSL → see diagram
- ✅ Edit diagram → DSL updates
- ✅ **LSP diagnostics (syntax + semantic errors)**
- ✅ **LSP autocomplete (keywords + node names)**
- ✅ **LSP hover (node/edge information)**
- ✅ **Comprehensive test suite (DSL, LSP, model engine)**
- ✅ **Round-trip tests ensure canonical DSL**
- ✅ Real-time sync across tabs
- ✅ Git persistence
- ✅ Auto layout
- ✅ Undo/redo
- ✅ **E2E tests for critical workflows**

#### Post-MVP (Phase 4-8)
- ✅ Multi-layer modeling (context/container/component)
- ✅ Validation rules engine
- ✅ Version history & diff
- ✅ Component libraries
- ✅ AI/MCP integration

### Risk Mitigation

1. **Parser Complexity**: Start with minimal grammar, extend incrementally
2. **Sync Conflicts**: Use simple "last write wins" for MVP, upgrade to CRDT later
3. **Performance**: Use React 19 compiler, optimize re-renders
4. **Git Operations**: Handle errors gracefully, add retry logic
5. **Browser Compatibility**: Test on Chrome, Firefox, Safari

### Next Steps After MVP

1. User testing & feedback
2. Performance optimization
3. Multi-layer modeling
4. Advanced validation rules
5. Component library system
6. AI integration for architecture generation

---

## 🎯 Quick Start Checklist

Before starting development, ensure:

- [ ] Node.js 18.18.0+ installed
- [ ] Bun 1.1+ installed
- [ ] pnpm 9.x installed
- [ ] Git configured
- [ ] GitHub repository created
- [ ] IDE configured (VS Code recommended)
- [ ] All dependencies compatible with React 19

---

## 🎯 Why LSP in MVP? (Strategic Decision)

### The Trade-off
- **Time Cost**: +2 weeks (8-10 weeks total instead of 6-8)
- **Complexity**: Medium (manageable for solo developer)
- **Value**: **10× improvement** in developer experience

### The Benefits

1. **Professional IDE Features**
   - Real-time diagnostics (catch errors as you type)
   - Intelligent autocomplete (suggest keywords, node names)
   - Hover information (quick context about nodes/edges)
   - Foundation for future features (go-to-definition, rename, etc.)

2. **Better Quality**
   - Catch syntax errors immediately
   - Validate semantics before sync
   - Prevent invalid models from propagating
   - Reduce debugging time significantly

3. **Future-Proof Architecture**
   - Ready for multi-file architectures
   - Enables AI/MCP integration earlier
   - Supports advanced refactoring later
   - Extensible for new language features

4. **Competitive Advantage**
   - Stands out in demos
   - Professional feel vs. basic text editor
   - Shows technical sophistication
   - Attractive to enterprise customers

5. **Text as Source of Truth**
   - LSP validates DSL correctness
   - Ensures bidirectional sync reliability
   - Maintains model integrity
   - Prevents corruption from invalid edits

### Implementation Reality

| Aspect | Reality |
|--------|---------|
| **Difficulty** | Medium (1.5 weeks for core features) |
| **Dependencies** | Standard LSP libraries (well-documented) |
| **Maintenance** | Low (LSP protocol is stable) |
| **Testing** | Straightforward (LSP has test utilities) |
| **ROI** | Very High (massive UX improvement) |

### MVP LSP Scope (What's In vs. Out)

**✅ IN (MVP)**:
- Diagnostics (syntax + semantic)
- Completion (keywords + identifiers)
- Hover (basic info)
- Formatting (optional, easy if serializer exists)

**❌ OUT (Post-MVP)**:
- Go to Definition (not needed yet)
- Rename Symbol (complex, can wait)
- Code Actions (future value)
- Cross-module indexing (after modules exist)

### Conclusion

**Including LSP in MVP is the right strategic call** because:
- Text is the source of truth for the DSL
- LSP ensures text correctness
- Professional IDE experience from day one
- Foundation for future features
- Manageable implementation (~1.5 weeks)
- High ROI (10× better DX)

**The 2-week extension is absolutely worth it.**

---

*Last Updated: 2025*
*Next.js Version: 16.x*
*React Version: 19.x*
*LSP: Included in MVP*
## DSL v0.1 — Simple Mode

Minimal, intuitive DSL for quick onboarding.

- Object types: `service`, `db`, `queue`, `external`, `ui`
- Flat structure, no modules/domains/contexts/imports
- Relations: `<from> -> <to>` with optional label `: "..."`
- Optional `@tag <name>` attached to next object (stored, ignored in Simple Mode)

### Syntax

```
<id>: <type> "Human Name"
<from> -> <to>
<from> -> <to> : "Label"
@tag critical
api: service "Auth API"
```

### Examples

```
userApi: service "User API"
userDB: db "User Database"
userApi -> userDB
```

```
orderApi: service "Order API"
paymentApi: service "Payment Service"
stripe: external "Stripe API"
ordersDB: db "Orders DB"
orderApi -> ordersDB
orderApi -> paymentApi
paymentApi -> stripe
```

```
cartApi: service "Cart API"
checkoutApi: service "Checkout API"
orderEvents: queue "Order Events"
ordersService: service "Orders Service"
checkoutApi -> orderEvents
orderEvents -> ordersService
```

### JSON Model (Simple Mode)

```
type SimpleModel = {
  components: { id: string; type: "service" | "db" | "queue" | "external" | "ui"; name: string; tags?: string[] }[]
  relations: { from: string; to: string; label?: string }[]
}
```

### Ohm Grammar (v0.1)

```
SimpleDSL {
  Program        = _ (Statement _)*
  Statement      = ComponentDecl | RelationDecl | TagDecl | Comment

  ComponentDecl  = Identifier _ ":" _ Type _ StringLiteral?
  Type           = "service" | "db" | "queue" | "external" | "ui"

  RelationDecl   = Identifier _ "->" _ Identifier (_ ":" _ StringLiteral)?

  TagDecl        = "@tag" _ Identifier

  StringLiteral  = "\"" (~"\"" any)* "\""
  Identifier     = letter (letter | digit | "_" | "-")*

  Comment        = LineComment | BlockComment
  LineComment    = "//" (~"\n" any)* ("\n" | end)
  BlockComment   = "/*" (~"*/" any)* "*/"

  _              = (space | Newline)*
  Newline        = "\n" | "\r\n" | "\r"
}
```

## Architecture Wizard Templates (Simple Mode)

Ready-to-use blueprints for onboarding, AI generation, and visual auto-layout.

### 1) Microservices Architecture (Basic)

```
web: ui "Web App"
apiGateway: service "API Gateway"

usersApi: service "Users Service"
usersDB: db "Users DB"

ordersApi: service "Orders Service"
ordersDB: db "Orders DB"

paymentsApi: service "Payments Service"
stripe: external "Stripe API"

web -> apiGateway
apiGateway -> usersApi
usersApi -> usersDB

apiGateway -> ordersApi
ordersApi -> ordersDB

apiGateway -> paymentsApi
paymentsApi -> stripe
```

### 2) Microservices (Intermediate, with Auth + Email)

```
web: ui "Frontend"
api: service "Backend API"
authApi: service "Auth Service"
emailApi: service "Email Service"

usersApi: service "Users Service"
usersDB: db "Users DB"

web -> api
api -> authApi
api -> usersApi
api -> emailApi
usersApi -> usersDB
emailApi -> externalSmtp
```

### 3) Event-Driven Architecture (Basic)

```
checkoutApi: service "Checkout API"
ordersTopic: queue "Orders Topic"
ordersWorker: service "Orders Processor"
ordersDB: db "Orders DB"

checkoutApi -> ordersTopic
ordersTopic -> ordersWorker
ordersWorker -> ordersDB
```

### 4) Event-Driven (With Multiple Consumers)

```
cartApi: service "Cart API"
checkoutApi: service "Checkout API"
orderEvents: queue "Order Events"

ordersService: service "Orders Service"
inventoryService: service "Inventory Service"
billingService: service "Billing Service"

ordersDB: db "Orders DB"
inventoryDB: db "Inventory DB"

checkoutApi -> orderEvents
orderEvents -> ordersService
orderEvents -> inventoryService
orderEvents -> billingService

ordersService -> ordersDB
inventoryService -> inventoryDB
billingService -> stripe
```

### 5) Monolith + Workers (Legacy/MVP)

```
web: ui "UI"
api: service "Monolith API"
db: db "Main Database"

worker: service "Background Worker"
jobs: queue "Job Queue"

web -> api
api -> db
api -> jobs
jobs -> worker
worker -> db
```

### 6) Serverless Architecture (AWS/GCP style)

```
client: ui "Client App"
apiGateway: service "API Gateway"
lambdaA: service "Lambda A"
lambdaB: service "Lambda B"
lambdaC: service "Lambda C"

events: queue "Event Stream"
mainDB: db "Primary DB"

client -> apiGateway
apiGateway -> lambdaA
lambdaA -> lambdaB
lambdaB -> mainDB
lambdaA -> events
events -> lambdaC
lambdaC -> mainDB
```

### 7) ETL / Data Pipeline

```
ingestionApi: service "Ingestion API"
rawBucket: db "Raw Data Storage"
kafka: queue "Event Stream (Kafka)"

etlWorker: service "ETL Worker"
warehouseDB: db "Data Warehouse"

ingestionApi -> rawBucket
ingestionApi -> kafka
kafka -> etlWorker
etlWorker -> warehouseDB
```

### 8) Machine Learning System (Basic)

```
collector: service "Data Collector"
rawDB: db "Raw Data"
preprocessor: service "Feature Preprocessor"
featuresDB: db "Feature Store"

trainer: service "Model Trainer"
modelRegistry: db "Model Registry"

predictor: service "Prediction API"

collector -> rawDB
collector -> preprocessor
preprocessor -> featuresDB
trainer -> featuresDB
trainer -> modelRegistry
predictor -> modelRegistry
```

### 9) E-Commerce Architecture

```
web: ui "Web Store"
api: service "Backend API"

productsApi: service "Products Service"
productsDB: db "Products DB"

cartApi: service "Cart Service"
cartDB: db "Cart DB"

ordersApi: service "Orders Service"
ordersDB: db "Orders DB"

paymentsApi: service "Payments Service"
stripe: external "Stripe"

web -> api
api -> productsApi
api -> cartApi
api -> ordersApi
api -> paymentsApi

productsApi -> productsDB
cartApi -> cartDB
ordersApi -> ordersDB
paymentsApi -> stripe
```

### 10) SaaS Multi-Tenant Architecture

```
frontend: ui "SaaS Frontend"
gateway: service "API Gateway"

tenantApi: service "Tenant Service"
tenantDB: db "Tenant Metadata"

billingApi: service "Billing Service"
stripe: external "Stripe Billing"

usageApi: service "Usage Tracker"
usageDB: db "Usage DB"

frontend -> gateway
gateway -> tenantApi
gateway -> billingApi
gateway -> usageApi

tenantApi -> tenantDB
usageApi -> usageDB
billingApi -> stripe
```

### Bonus: Minimal Useful Architecture (Default Template)

```
web: ui "App"
api: service "API"
db: db "Database"

web -> api
api -> db
```
