# Diff Engine (Semantic)

Purpose
- Produce semantic diff between Old IR and New IR to drive policies, approvals, and diagnostics.

Core Maps in ModelDiff
- `Systems map[string]*SystemDiff`
- `Entities map[string]*EntityDiff`
- `Events map[string]*EventDiff`
- `Contracts map[string]*ContractDiff`
- `Fields map[string]*FieldDiff`

Key Computations
- Event version bump (`major|minor|patch`)
- Schema breaking detection
- Field added/removed/changed + PIIChanged
- System boundary changes
- Contract breaking/compatible

Integration
- Evaluator uses diff-aware attribute resolution in `pkg/approval/eval.go` via `matchCondition`.
