# Policy Evaluation Engine

Pipeline
- Load old/new IR, compute diff, match targets, evaluate conditions, exceptions, auto-approve, produce approvals & diagnostics.

Data Structures
- `PolicyContext`, `PolicyEvalResult`, `PolicyEvalReport`, `ApprovalRequirement`, `Diagnostic` in `pkg/approval/pipeline.go`.

MCP Endpoints
- `POST /validate-approval-policy` evaluates a single policy with provided changes or DSL.
- `POST /validate-all` evaluates multiple policies and returns a consolidated report.

Integration
- CI uses `required_approvals` to block merges.
- LSP shows diagnostics and explanations; hover docs provide context.
