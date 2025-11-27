# Brownfield Adoption Strategy

Goal
- Make Sruja immediately useful in legacy (“brownfield”) environments with near-zero friction.

Principles
- Start by modeling what already exists (no forced redesign).
- Support partial/inferred models and low-friction validation.
- Deliver visible value (diagrams, drift, summaries) in <30 minutes.

Auto‑Discovery Pipeline
- Importers and scanners convert existing assets into Sruja IR:
  - Codebases (Go/Java/TS/Python): systems, modules, REST endpoints, events, entities, datastores, queues, relations
  - OpenAPI/Swagger → API contracts
  - AsyncAPI → event schemas, producers/consumers
  - Kubernetes manifests → deployments/services/ingress/state
  - Terraform → infrastructure components
  - Postman collections → mapped API contracts
  - GitHub repos → system boundaries via repo layout
  - Existing C4 diagrams → DSL placeholders

Onboarding Workflow
1. Run imports
   - `sruja import project ./backend`
   - `sruja import openapi ./api/openapi.yaml`
   - `sruja import asyncapi ./events/payment.yaml`
   → generates `architecture brownfield.sruja` with systems/containers/APIs/events/entities/relations
2. Generate outputs
   - C4 diagrams, domain/event flows, contract summaries, drift/inconsistency reports
3. Create legacy baseline
   - `snapshot legacy-baseline` (baseline architecture; no mandatory fixes)
4. Incremental refinement (AI‑assisted)
   - Boundaries, naming, documentation, merging/splitting components
5. Introduce governance progressively
   - Start with info/warning rules; move to blocking after confidence

Partial Specification Support
- Allow inferred/unknown placeholders and “accept for now” flags:
  - `inferred true`, `auto_discovered true`, `metadata { verified: false }`

Refactoring & Code Alignment Tools
- AI suggests boundaries, merges, consolidation, missing metadata.
- MCP tools:
  - `sruja.code.detect_mismatches`
  - `sruja.code.suggest_fixes`

Brownfield Mode
- `sruja brownfield init` → looser validation, non‑blocking rules, auto‑inferred elements
- `sruja brownfield tighten` → gradually increase rigor

Adoption Tactics & Incentives
- Start with architecture owners; provide templates; auto‑diagrams.
- “AI explains your architecture”; celebrate early wins (e.g., cycle reduction).
- Tangible value: diagrams, event flow, API summary, entity catalog, drift, snapshot baseline.

Summary
- Auto‑discovery + partial models + progressive rules + AI guidance + code alignment → practical brownfield adoption.

