# Entities DSL

Entities are first-class domain model definitions decoupled from API/Event/Data contracts.

Syntax
- `entities { entity <Name> { description, fields { key: Type }, relations { name -> Target }, invariants { "expr" }, lifecycle { StateA -> StateB }, versioning { current "x.y" backwards_with "1.x" }, constraints { key "value" } } }`
- Supported locations: architecture, system, container, and `domain <Name> { entities { ... } }`.

Integration
- Contracts map via `request_map`, `response_map`, `emits_schema`, `writes_schema`.
- Validators align event schemas and lifecycle transitions with entity definitions.
 - CLI: `sruja list entities` lists all entities (architecture + systems + containers).
 - MCP: `POST /list-entities { path }` returns entity names; `POST /validate-event` validates lifecycle transitions against entity FSM.
