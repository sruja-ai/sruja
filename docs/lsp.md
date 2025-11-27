# LSP — Completions and Hover (Approval Policy DSL)

Completions
- Block keywords: `applies_to`, `when`, `require`, `severity`, `auto_approve`, `except`, `metadata`.
- Targets: `architecture`, `domain`, `system`, `container`, `component`, `entity`, `field`, `event`, `contract`, `diagram`.
- Expressions: attribute paths (e.g., `field.pii`, `event.version.bump`), operators (`==`, `!=`, `contains`, `matches`, `and`, `or`, `not`).
- Approvers: team names and group quorum snippet.

Hover Docs
- Static keyword documentation for policy blocks.
- Operator and attribute hover content.
- IR-driven hovers for target identifiers (systems, entities, events, contracts).

Implementation
- Completions in `pkg/lsp/policy_rule_completion.go`.
- Hovers in `pkg/lsp/approval_hover.go` with `ProvideApprovalHover` integrated into `pkg/lsp/hover.go`.
