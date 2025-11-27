# Model-Aware Code Generation Engine

**Status**: Advanced Engine  
**Pillars**: Core (Code Generation)

[← Back to Engines](../README.md)

## Overview

The Model-Aware Code Generation Engine generates code based on the true architecture model, not templates alone. It makes your architecture platform AI-native, MCP-compatible, refactoring-safe, and enforces architecture constraints during coding.

**This system is much more powerful than typical scaffolding tools (Yeoman, Plop, Nest schematics).**

You are essentially building *Nx + Backstage + Structurizr + AI + Architecture DSL → actual code enforcement*.

## What the Codegen Engine Does

Given a global architecture model:

```
Contexts → Containers → Components → Interfaces
Relations → Boundaries → Requirements → ADRs
```

The engine generates:

- ✅ Project structures
- ✅ Service scaffolds
- ✅ Domain modules
- ✅ API interfaces
- ✅ DTOs
- ✅ Event definitions
- ✅ Infrastructure boilerplate
- ✅ Integration adaptors
- ✅ Stubs for relationships
- ✅ Tests
- ✅ README per module
- ✅ Language-specific files (TS/Go/Rust/Java)

And critically:

- ✔ Everything is tied back to the architecture spec
- ✔ MCP tools can validate and regenerate missing parts
- ✔ AI assistants use the architecture to generate "architecturally correct code"

## Architecture → Code: Fundamental Design

Pipeline:

```
GlobalModel     ← composed model (IR)
   ↓
CodegenPlan     ← what must be generated
   ↓
TemplateEngine  ← fills templates with model data
   ↓
OutputFiles     ← FS or Git target
```

## Codegen Concepts

### Generators
Generators transform specific architecture elements → code.

Examples:

- `ServiceGenerator`
- `ControllerGenerator`
- `EventGenerator`
- `IntegrationGenerator`
- `InfrastructureGenerator`

### Language Targets
Each generator registers support for **TS, Go, Rust, Java**.

Example:

```
generate(service, sruja="ts")
generate(service, sruja="go")
```

### Templates
Each generator uses template files + template functions.

Example:

```
templates/
   ts/
     service.ts.hbs
     dto.ts.hbs
   go/
     service.go.hbs
     dto.go.hbs
```

### Model Input
Generators receive **fully resolved IR**:

- full paths
- layer info
- boundaries
- relationships
- events
- required deps

## Codegen Engine API (TypeScript)

```ts
interface CodegenRequest {
  model: GlobalModel;
  target: "ts" | "go" | "rust" | "java";
  modules: string[]; // optional subset
  outputDir: string; // local or git
}

interface CodegenEngine {
  generate(req: CodegenRequest): Promise<GeneratedFile[]>;
}
```

### Output file:

```ts
interface GeneratedFile {
  path: string;      // "services/payments/PaymentService.ts"
  content: string;
  existsConflict?: boolean;
}
```

## MCP Integration — THIS IS KEY

Your MCP server exposes:

### Tool #1: codegen.generate

```
Input:
{
  "modules": ["payments", "auth"],
  "target": "ts"
}

Output:
[
  {
    "path": "payments/PaymentService.ts",
    "content": "export class PaymentService { ... }"
  }
]
```

### Tool #2: codegen.plan

```
"plan": [
  "Create Service PaymentsAPI",
  "Generate DTO: ChargeRequest",
  "Generate Event: PaymentSucceeded",
  "Generate Integration Adapter for Auth"
]
```

### Tool #3: codegen.validate

Checks:

- outdated files
- missing integration stubs
- DTO mismatch
- drift between architecture and code

## Generators (Detailed)

### Generator 1: Service Generator

Input:

```
component: { name: "PaymentService", kind: "service" }
dependencies: 
   - auth.UserService
   - notifications.EmailSender
```

Output:

#### TS

```
export class PaymentService {
    constructor(
       private auth: UserService,
       private email: EmailSender
    ) {}

    async processPayment(input: ChargeRequest): Promise<ChargeResponse> { }
}
```

#### Go

```
type PaymentService struct {
   Auth UserService
   Email EmailSender
}

func (s *PaymentService) ProcessPayment(req ChargeRequest) (ChargeResponse, error) { }
```

### Generator 2: API Generator (HTTP / gRPC)

Component with tag:

```
component PaymentApi endpoint "/payments"
```

Outputs:

- controller file
- route file
- OpenAPI spec
- request/response DTOs

### Generator 3: Event Generator

Architecture:

```
event PaymentSucceeded {
   paymentId: string
   userId: string
}
```

Generated:

#### TS

```
export interface PaymentSucceededEvent {
   paymentId: string;
   userId: string;
}
```

#### Go

```
type PaymentSucceeded struct {
   PaymentID string `json:"paymentId"`
   UserID    string `json:"userId"`
}
```

### Generator 4: Infrastructure Generator

Given architecture:

```
db PaymentsDB type postgresql
queue PaymentEvents type kafka
```

Output:

- migrations directory
- connection factory
- repository interfaces
- repository templates

### Generator 5: Integration Generator

For every relation:

```
PaymentService -> NotificationService
```

Generates:

- client interface
- adapter stub
- error handling
- retry logic
- circuit breaker setup

## MCP API

```
codegen.generate({model, target, modules})
codegen.plan(model)
codegen.validate(codebase, model)
codegen.regenerate(module)
codegen.sync(model, codebase)
```

## Strategic Value

The Code Generation Engine provides:

- ✅ Architecture-driven code generation
- ✅ MCP integration for AI assistants
- ✅ Architecture constraint enforcement
- ✅ Multi-language support
- ✅ Refactoring safety
- ✅ Consistency across codebase

**This is critical for bridging architecture and implementation.**

## Implementation Status

✅ Architecture designed  
✅ Generator framework specified  
✅ MCP integration defined  
📋 Implementation in progress

---

*The Code Generation Engine bridges architecture models and actual code implementation.*

