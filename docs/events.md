# Event DSL (Domain-Driven)

Purpose
- Define domain events with schema, semantics, lifecycle effects, guarantees, and versioning.

Syntax
- `events { event <Name> { version, entity, category, description, schema { ... }, metadata { ... }, guarantees { ... }, lifecycle_effect { Entity.State -> Entity.State }, causes { ... }, publishers { Qualified }, consumers { Qualified }, versioning { backwards_with "x.y" } } }`

Integration
- Lifecycle effects validated against entity FSM.
- Publishers/consumers reference components (`System.Container.Component` styles) for alignment.
- Policies enforce category-specific requirements (e.g., audit).
 - CLI: `sruja list events` lists events with versions.
 - MCP: `POST /list-events` returns events; `POST /validate-event` checks lifecycle_effect validity.

MCP
- Endpoints: `/list-events`, `/validate-event`.
