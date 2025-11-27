# MCP Server

Capabilities
- Architecture: `load`, `explain`, `diagram`, `query`, `diff`.
- Rules/Policies: `review-architecture`, `evaluate-policy`.
- Domain Model: `list-entities`, `list-events`, `validate-event`.
 - Approvals: `validate-approval-policy`, `validate-all`.

Usage
- `POST /list-entities { path }` → `{ entities: ["Payment", ...] }`
- `POST /list-events { path }` → `{ events: [{ name, version }] }`
- `POST /validate-event { path, event }` → `{ event, valid, issues: [] }`
 - `POST /validate-approval-policy { path, policy | policy_dsl, changes }` → `{ required_approvals, auto_approved, skipped_due_to_exception }`
 - `POST /validate-all { path, policies | policy_dsls, changes }` → consolidated `PolicyEvalReport` for CI and PR workflows.
