# Architecture Approval Policy DSL

Purpose
- Enforce guardrails with human + machine approvals integrated into Git/CI/MCP.

Syntax (JSON-compatible)
- `policy <ID> "<Label>" { applies_to: <TargetSelector>|[...], when: <Condition>, require: <Rule>|[<Rule>...], severity: "info|warning|error", auto_approve?: <Rule|Condition>, except?: <Condition>, metadata?: { key: value } }`

Targets
- `entity <ID|pattern>`, `system <ID|pattern>`, `container <ID|pattern>`, `component <ID|pattern>`, `event <ID|pattern>`, `contract <ID|pattern>`, `domain <ID>`, `field <Entity.field>`, `architecture`.

Conditions (examples)
- `field.pii == true`, `field.changed == true`, `entity.lifecycle.changed`, `event.version.bump == "major"`, `event.schema.breaking == true`, `contract.breaking_change == true`, `contract.compatible == true`, `system.boundaries.changed`, `metadata.business_critical == true`, `any_change == true`.

Approval Rules
- Strings (teams/users) or quorum: `{ group: "security_team", count: 2 }`.
- Tiers: `["domain_owner", { group: "security_team", count: 1 }, "chief_architect"]`.

Exceptions & Auto-Approve
- `except: when branch.name startswith "experiment/"`.
- `auto_approve: when contract.compatible == true` or `auto_approve: true`.

MCP Integration
- Endpoints:
  - `/validate-approval-policy` evaluates a single policy (JSON or DSL).
  - `/validate-all` batch evaluates multiple policies; returns consolidated `PolicyEvalReport` for CI gating.

Parser
- Full grammar implemented in `pkg/approval/dsl.go` with Regex token, target selectors, expressions, approval specs, auto-approve, exceptions.
