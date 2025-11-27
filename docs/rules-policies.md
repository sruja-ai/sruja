# Rules and Policies

Shared Expr
- Operators: `==`, `!=`, `in`, `contains`, `starts_with`, `ends_with`, `matches`, `exists` with `and`/`or`/`not`.
- Fields: `id`, `label`, `from`, `to`, `verb`, `label`, `metadata.<key>`.

Rule DSL
- `rule <ID> { applies_to <type> when { <expr> } ensure { <expr> } severity <level> message "..." }`
- Types: `system`, `container`, `component`, `relation`.

Policy DSL
- `policy <ID> { applies_to <type> where <expr> rules { <RuleID>* } controls { <expr>* } severity <level> ... }`
- Evaluator computes compliance and violations per target elements.

MCP
- Endpoints: `/review-architecture`, `/evaluate-policy`, `/validate-approval-policy`, `/validate-all`.
  - See `docs/approval-policy.md` for approval-specific DSL.
