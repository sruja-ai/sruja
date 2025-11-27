# AI‑First Brownfield Importer Architecture

Philosophy
- Replace complex static parsing with LLM‑assisted inference + a thin deterministic normalizer.

Pipeline Overview
1. Repo Collector (deterministic)
   - Collects file tree, Docker/K8s manifests, OpenAPI/AsyncAPI, SQL migrations, protobuf, README, CI configs, package manifests.
   - Produces a structured snapshot for AI.
2. AI Pass #1 — System & Container Inference
   - Uses repo structure, deployment configs, entrypoints and build files.
3. AI Pass #2 — API & Event Extraction
   - REST endpoints, gRPC services, pub/sub handlers, routes, producers/consumers, message schemas.
4. AI Pass #3 — Entity & Schema Modeling
   - SQL migrations, ORMs, DTOs, JSON schemas, GraphQL types.
5. AI Pass #4 — Relations, Lifecycles, Domains
   - Service calls, DB access, pub/sub, clients, env/service URLs, configs.
6. Deterministic IR Normalizer + Conflict Resolver
   - Deduplication, merge APIs/events, unify naming, fix references, resolve contradictions; applies Sruja rules.
7. DSL Generator + Notebook Review UI
   - Emits DSL with “inferred by AI” annotations, confidence, and explanation; human‑in‑the‑loop refinement.

Multi‑Agent AI
- Specialized agents for system, API, event, entity, relationship, domain, consistency, and explainer.
- Outputs merged deterministically; improves accuracy.

Outputs & Traceability
- IR fragments include source, confidence, and raw explanation.
- Normalizer ensures correctness for compile & diff engine.

Why it Works
- Low engineering cost; language‑agnostic; high‑quality inferred architecture.
- Immediate brownfield value; scalable with human review.

