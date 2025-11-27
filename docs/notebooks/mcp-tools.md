# MCP Tools Reference

[← Back to Notebooks Index](./README.md)

Complete reference for all Sruja MCP tools that can be called by AI agents (Cursor, Copilot, etc.).

## Tool Design Principles

- ✅ **Stateless** - Kernel holds state; tools read/modify it
- ✅ **JSON-based** - Safe & compatible with LLMs
- ✅ **Patch-based** - Tools produce small deltas, not full rewrites
- ✅ **Deterministic** - Safe for automated application
- ✅ **Validation-first** - Validate before applying changes
- ✅ **Non-destructive** - Changes via diffs, snapshots, or variants

## Tool Groups

All tools are namespaced:

```
sruja.read.*      - Read architecture state
sruja.query.*     - Query architecture
sruja.write.*     - Modify architecture
sruja.validate.*  - Validate architecture
sruja.diff.*      - Compare versions
sruja.snapshot.*  - Manage snapshots
sruja.variant.*   - Manage variants
sruja.diagram.*   - Generate diagrams
sruja.event.*     - Event simulation
sruja.contract.*  - Contract management
sruja.lsp.*       - LSP features
sruja.code.*      - Code alignment
sruja.help        - Meta tool
```

## Read Tools

### sruja.read.model

Get the full architecture IR.

**Input:**
```json
{}
```

**Output:**
```json
{
  "ir": {
    "domains": {...},
    "systems": {...},
    "components": {...},
    "entities": {...},
    "events": {...}
  }
}
```

### sruja.read.element

Get a specific model element.

**Input:**
```json
{
  "id": "Payment"
}
```

**Output:**
```json
{
  "element": {
    "id": "Payment",
    "name": "Payment",
    "type": "entity",
    "fields": [...]
  }
}
```

### sruja.read.structure

Return high-level structure (domain → system → container → component).

**Input:**
```json
{}
```

**Output:**
```json
{
  "systems": [
    {"id": "Billing", "containers": [...]}
  ],
  "domains": [
    {"id": "Payments", "entities": [...]}
  ],
  "events": [...],
  "apis": [...],
  "components": [...]
}
```

### sruja.read.dependencies

Get dependencies for an element.

**Input:**
```json
{
  "id": "PaymentService"
}
```

**Output:**
```json
{
  "dependencies": [
    "UserService",
    "PaymentGateway"
  ]
}
```

## Query Tools

### sruja.query.run

Execute a SrujaQL query.

**Input:**
```json
{
  "query": "select systems where tags contains 'public'"
}
```

**Output:**
```json
{
  "results": [
    {"id": "Billing", "name": "Billing System", ...}
  ]
}
```

**Example queries:**
- `select systems where tags contains "public"`
- `select events where pii == true`
- `select components where depends_on contains "BillingDB"`
- `graph dependencies of Billing`
- `diff contracts BillingAPI 1.0 vs 2.0`

## Validation Tools

### sruja.validate.all

Validate entire architecture.

**Input:**
```json
{}
```

**Output:**
```json
{
  "diagnostics": [
    {
      "severity": "error",
      "message": "Missing required field 'currency'",
      "elementId": "Payment",
      "location": {"file": "cell", "line": 5}
    }
  ]
}
```

### sruja.validate.element

Validate a specific element.

**Input:**
```json
{
  "id": "Payment"
}
```

**Output:**
```json
{
  "diagnostics": [...]
}
```

### sruja.validate.contract

Validate a contract.

**Input:**
```json
{
  "id": "BillingAPI"
}
```

**Output:**
```json
{
  "violations": [
    {
      "rule": "no-breaking-changes",
      "message": "Field removed without deprecation",
      "severity": "error"
    }
  ]
}
```

## Write Tools

### sruja.write.apply_patch

Apply a patch to the architecture model (safe mode).

**Input:**
```json
{
  "patch": {
    "operation": "update",
    "elementType": "entity",
    "elementId": "Payment",
    "payload": {
      "fields": [
        {"name": "amount", "type": "Float", "required": true}
      ]
    }
  }
}
```

**Output:**
```json
{
  "applied": true,
  "diagnostics": [],
  "model": {...}
}
```

**Fail-safe:** `applied = false` if patch violates rules.

### sruja.write.apply_patches

Apply multiple patches.

**Input:**
```json
{
  "patches": [
    {"operation": "add", ...},
    {"operation": "update", ...}
  ]
}
```

**Output:**
```json
{
  "applied": true,
  "diagnostics": [...]
}
```

### sruja.write.update_element

Update an element directly.

**Input:**
```json
{
  "id": "Payment",
  "fields": {
    "currency": {"required": true}
  }
}
```

**Output:**
```json
{
  "applied": true,
  "diagnostics": [...]
}
```

### sruja.write.delete_element

Delete an element.

**Input:**
```json
{
  "id": "OldPayment"
}
```

**Output:**
```json
{
  "applied": true,
  "diagnostics": [
    {
      "severity": "warning",
      "message": "Deleting element may break dependencies"
    }
  ]
}
```

## Diff Tools

### sruja.diff.elements

Compare two elements.

**Input:**
```json
{
  "id1": "Payment_v1",
  "id2": "Payment_v2"
}
```

**Output:**
```json
{
  "diff": {
    "breaking": [
      "Field 'userId' removed"
    ],
    "additive": [
      "Field 'feeApplied' added"
    ],
    "compatible": false
  }
}
```

### sruja.diff.contract

Compare contract versions.

**Input:**
```json
{
  "versionA": "BillingAPI/1.0",
  "versionB": "BillingAPI/2.0"
}
```

**Output:**
```json
{
  "diff": {
    "breaking": [...],
    "additive": [...],
    "compatible": false
  }
}
```

## Snapshot Tools

### sruja.snapshot.create

Create a snapshot of current architecture state.

**Input:**
```json
{
  "name": "iteration-5",
  "description": "Before async payment changes"
}
```

**Output:**
```json
{
  "snapshot": {
    "id": "iteration-5",
    "name": "iteration-5",
    "timestamp": "2024-01-01T00:00:00Z",
    "model": {...}
  }
}
```

### sruja.snapshot.load

Load a snapshot.

**Input:**
```json
{
  "name": "iteration-5"
}
```

**Output:**
```json
{
  "model": {...}
}
```

### sruja.snapshot.list

List all snapshots.

**Input:**
```json
{}
```

**Output:**
```json
{
  "snapshots": [
    {
      "id": "iteration-5",
      "name": "iteration-5",
      "timestamp": "2024-01-01T00:00:00Z"
    }
  ]
}
```

## Variant Tools

### sruja.variant.create

Create a variant from a snapshot.

**Input:**
```json
{
  "name": "async-payments",
  "base": "iteration-5"
}
```

**Output:**
```json
{
  "variant": {
    "id": "async-payments",
    "name": "async-payments",
    "base": "iteration-5",
    "patches": []
  }
}
```

### sruja.variant.apply

Apply a variant to current model.

**Input:**
```json
{
  "name": "async-payments"
}
```

**Output:**
```json
{
  "model": {...}
}
```

### sruja.variant.diff

Get diff between variant and base.

**Input:**
```json
{
  "name": "async-payments"
}
```

**Output:**
```json
{
  "patches": [
    {"operation": "update", ...}
  ]
}
```

### sruja.variant.list

List all variants.

**Input:**
```json
{}
```

**Output:**
```json
{
  "variants": [
    {
      "id": "async-payments",
      "name": "async-payments",
      "base": "iteration-5"
    }
  ]
}
```

## Diagram Tools

### sruja.diagram.render

Generate a diagram.

**Input:**
```json
{
  "target": "system Billing",
  "format": "svg"
}
```

**Output:**
```json
{
  "diagram": "<svg>...</svg>"
}
```

**Formats:** `svg`, `png`, `mermaid`, `d2`

### sruja.diagram.system_map

Generate system map diagram.

**Input:**
```json
{}
```

**Output:**
```json
{
  "svg": "<svg>...</svg>"
}
```

### sruja.diagram.event_flow

Generate event flow diagram.

**Input:**
```json
{
  "entity": "Payment"
}
```

**Output:**
```json
{
  "svg": "<svg>...</svg>"
}
```

## Event Tools

### sruja.event.lifecycle

Get lifecycle FSM for an entity.

**Input:**
```json
{
  "entity": "Payment"
}
```

**Output:**
```json
{
  "fsm": {
    "states": ["PENDING", "AUTHORIZED", "COMPLETED"],
    "transitions": [
      {"from": "PENDING", "to": "AUTHORIZED", "event": "PaymentAuthorized"}
    ]
  }
}
```

### sruja.event.simulate

Simulate event lifecycle.

**Input:**
```json
{
  "entity": "Payment",
  "events": ["PaymentAuthorized", "PaymentCompleted"]
}
```

**Output:**
```json
{
  "finalState": "COMPLETED",
  "trace": [
    "PENDING → AUTHORIZED",
    "AUTHORIZED → COMPLETED"
  ]
}
```

### sruja.event.causes

Get events that cause another event.

**Input:**
```json
{
  "event": "PaymentCompleted"
}
```

**Output:**
```json
{
  "causes": [
    "PaymentAuthorized",
    "RefundProcessed"
  ]
}
```

## Contract Tools

### sruja.contract.get

Get a contract.

**Input:**
```json
{
  "id": "BillingAPI"
}
```

**Output:**
```json
{
  "contract": {
    "id": "BillingAPI",
    "version": "1.0",
    "endpoints": [...]
  }
}
```

### sruja.contract.evolve

Evolve a contract (create new version).

**Input:**
```json
{
  "id": "BillingAPI",
  "changes": {
    "add_field": {"name": "feeApplied", "type": "Float"}
  }
}
```

**Output:**
```json
{
  "new_version": {
    "id": "BillingAPI",
    "version": "2.0",
    ...
  }
}
```

## LSP Tools

### sruja.lsp.complete

Get autocomplete suggestions.

**Input:**
```json
{
  "code": "sys",
  "cursor": 3
}
```

**Output:**
```json
{
  "completions": [
    "system",
    "system Billing"
  ]
}
```

### sruja.lsp.hover

Get hover information.

**Input:**
```json
{
  "code": "Payment",
  "cursor": 7
}
```

**Output:**
```json
{
  "info": "Entity Payment\nFields: amount, currency, status"
}
```

### sruja.lsp.diagnostics

Get diagnostics for code.

**Input:**
```json
{
  "code": "entity Payment {\n  amount: Float\n}"
}
```

**Output:**
```json
{
  "diagnostics": [
    {
      "severity": "error",
      "message": "Missing required field 'currency'",
      "location": {"line": 1}
    }
  ]
}
```

## Code Alignment Tools

### sruja.code.detect_mismatches

Detect mismatches between code and architecture.

**Input:**
```json
{}
```

**Output:**
```json
{
  "mismatches": [
    {
      "type": "missing_api",
      "component": "PaymentService",
      "message": "API endpoint /payments not implemented",
      "location": "src/payment_service.go"
    }
  ]
}
```

### sruja.code.suggest_fixes

Suggest fixes for a mismatch.

**Input:**
```json
{
  "mismatchId": "missing_api_1"
}
```

**Output:**
```json
{
  "patches": [
    {
      "file": "src/payment_service.go",
      "patch": "func (s *Service) CreatePayment(...) {...}"
    }
  ]
}
```

### sruja.code.apply_fix

Apply a code fix.

**Input:**
```json
{
  "patch": {
    "file": "src/payment_service.go",
    "content": "..."
  }
}
```

**Output:**
```json
{
  "applied": true
}
```

## Meta Tool

### sruja.help

Get information about all available tools.

**Input:**
```json
{}
```

**Output:**
```json
{
  "tools": [
    {
      "name": "sruja.read.model",
      "description": "Get the full architecture IR",
      "input_schema": {...},
      "output_schema": {...}
    },
    ...
  ]
}
```

## Tool Usage Patterns

### Pattern 1: Read → Reason → Apply

```
1. Call sruja.read.model
2. Reason about changes
3. Produce DSL patches
4. Call sruja.write.apply_patch
5. Call sruja.validate.all
6. Call sruja.diagram.render
```

### Pattern 2: Self-Healing Architecture

```
1. Call sruja.list_violations
2. For each violation:
   - Generate fix
   - Call sruja.write.apply_patch
   - Call sruja.validate.all
3. Repeat until clean
```

### Pattern 3: Variant Exploration

```
1. Call sruja.snapshot.create (save checkpoint)
2. Call sruja.variant.create (create variant)
3. Make experimental changes
4. If successful: sruja.variant.merge
5. If not: sruja.snapshot.load (rollback)
```

## Security Considerations

- **Authentication** - MCP endpoints require authentication
- **Authorization** - Different tools may require different permissions
- **Rate Limiting** - Prevent abuse
- **Validation** - All model updates must be validated
- **Audit Logging** - Track all MCP tool usage

## References

- [MCP Integration](../integration/mcp.md) - General MCP integration guide
- [Architecture Kernel](./kernel.md) - Kernel details
- [Cursor AI Integration](./cursor-ai-integration.md) - Using tools with Cursor

