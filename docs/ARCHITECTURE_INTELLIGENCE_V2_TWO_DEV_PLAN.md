# Architecture Intelligence V2: Two-Developer Execution Plan

**Purpose:** Enable 2 developers to execute the ARCHITECTURE_INTELLIGENCE_V2.md roadmap in parallel with minimal coordination overhead.

**Principle:** Split by **domain** (structural vs semantic/runtime) and **shared contract** (interfaces, not implementation).

---

## Summary: Track Ownership

| Track | Developer | Primary Focus | Crates | Depends On |
|-------|-----------|---------------|--------|------------|
| **A — Structural & Intent** | Dev 1 | Graph algorithms, drift, intent comparison | sruja-graph, sruja-diff, sruja-intent, sruja-language | — |
| **B — Semantic & Runtime** | Dev 2 | Embeddings, clustering, traces, adoption | sruja-semantic, sruja-runtime, sruja-cli (why), docs | — |

Both tracks can start **immediately**; coordination is limited to agreed interfaces and CLI wiring.

---

## Week-by-Week Execution Plan

### Phase 0 + Phase 1 (Weeks 1–4)

| Week | Dev 1 (Structural & Intent) | Dev 2 (Semantic & Runtime) | Coordination |
|------|-----------------------------|----------------------------|--------------|
| **1** | Unify drift heuristics; cycle deduplication; god-module threshold configurable (ScanConfig/drift opts) | `sruja why` evidence templates; README/docs reorder | None |
| **2** | Build `Program → Graph` converter | New crate `sruja-semantic` scaffold; `EmbeddingProvider` trait; import resolution `./dir` → `./dir/index.ts` (sruja-scan) | Agree on `Graph` / `NodeId` / `EdgeId` types (existing) |
| **3** | Wire `sruja drift --baseline foo.sruja`; SCC module | Embedding implementations (OpenAI, local stub); vocabulary extractor | Dev 1: confirm `compare_graphs` contract for baseline |
| **4** | Treewidth module; centrality module | Domain clustering; bounded context detector | None |

### Phase 2 (Weeks 5–7)

| Week | Dev 1 | Dev 2 | Coordination |
|------|-------|-------|--------------|
| **5** | Coupling metrics; `sruja complexity` CLI | Semantic coupling analyzer; vocabulary leakage | None |
| **6** | Begin `sruja-intent` crate; ADR parser | `sruja semantic analyze` CLI; integration tests | None |
| **7** | Intent model; drift detector (no semantic alignment yet) | Polish semantic reports; mock embedding for CI | None |

### Phase 3 + Phase 4 (Weeks 8–14)

| Week | Dev 1 | Dev 2 | Coordination |
|------|-------|-------|--------------|
| **8** | Design doc parser; intent aligner | `sruja-runtime` crate scaffold; trace types | None |
| **9** | Drift detector with boundary violations | OTLP collector; execution graph processor | None |
| **10** | `sruja intent check` CLI; ADR generation | Agent execution trees; tool invocation graph | None |
| **11** | Optional: semantic alignment (consumes Dev 2’s SemanticIntelligence) | Emergent cycle detector; `sruja runtime analyze` | **Handoff:** Dev 2 documents `SemanticIntelligence` API for Dev 1 |
| **12** | Intent integration tests; drift report polish | Runtime analysis; hotspot detection | None |
| **13** | — | Runtime report; OTLP integration tests | None |
| **14** | **sruja-report** crate scaffold; `ComprehensiveReport` schema (structural, intent); unit tests for structural/intent | **sruja-report** contributions (semantic, runtime sections); **sruja-config** scaffold or agree `sruja.toml` schema | **Joint:** Agree ComprehensiveReport fields and `Recommendation` (priority, category, source_layer, effort); agree config owner |
| **15** | `sruja analyze` structural + intent wiring; wire report from sruja-report | `sruja analyze` semantic + runtime wiring; load config (sruja-config or env) | **Joint:** Integrate into single `sruja analyze --all-layers`; implement report aggregation |
| **16** | Docs; AGENTS.md; example projects; structural/intent integration tests | Docs; AGENTS.md; performance tuning; semantic/runtime integration tests | Joint review and release prep |

---

## Track A: Structural & Intent (Dev 1)

### Scope

- **Phase 0:** Drift unification, `Program → Graph`, `sruja drift --baseline`
- **Phase 1:** Treewidth, SCC, centrality, coupling; `sruja complexity`
- **Phase 3:** `sruja-intent` crate (ADR parser, intent model, drift detector, `sruja intent check`)
- **Short-term:** Cycle deduplication, god-module threshold config

### Crates Owned

- `sruja-graph` (treewidth, scc, centrality, coupling)
- `sruja-diff` (drift unification, baseline wiring)
- `sruja-language` (Program → Graph converter)
- `sruja-intent` (new crate)
- `sruja-cli` (drift, complexity, intent commands)
- `sruja-report` (lead: ComprehensiveReport schema, structural/intent sections; joint in Week 14–15)
- `sruja-config` (lead: sruja.toml schema and load; both consume)

### Deliverables

1. `sruja drift --baseline foo.sruja` (optional baseline)
2. `sruja complexity` with treewidth, SCC, centrality, coupling
3. `sruja intent check --repo . --intent ./docs/architecture`
4. ADR parser and drift report generation

### Dependencies on Dev 2

- **Optional:** Semantic alignment in drift (Week 11+). Can work with a stub until `SemanticIntelligence` is ready.
- **Phase 5:** Joint integration of `sruja analyze --all-layers`.

---

## Track B: Semantic & Runtime (Dev 2)

### Scope

- **Phase 0:** `sruja why` templates, README/docs, blast_radius
- **Phase 2:** `sruja-semantic` crate (embeddings, clustering, coupling, leakage)
- **Phase 4:** `sruja-runtime` crate (traces, execution trees, OTLP, emergent cycles)
- **Short-term:** Import resolution (`./dir` → `./dir/index.ts`), test consolidation

### Crates Owned

- `sruja-semantic` (new crate)
- `sruja-runtime` (new crate)
- `sruja-cli` (why, semantic, runtime commands)
- `sruja-report` (contribute semantic/runtime sections; joint in Week 14–15)
- `sruja-config` (contribute if separate crate; both consume)
- `docs/`, README

### Deliverables

1. `sruja why` with deterministic evidence templates
2. `sruja semantic analyze --repo .`
3. `sruja runtime analyze --traces ./traces.json`
4. Updated adoption docs and no-key-first messaging

### Dependencies on Dev 1

- **None for Phase 2/4.** Uses existing `sruja_scan::Graph`; no Program→Graph required.
- **Phase 5:** Joint integration; Dev 1 may consume `SemanticIntelligence` for intent alignment.

---

## Shared Contracts (Must Align)

### 1. Graph Types

- Both use `sruja_scan::Graph`, `NodeId`, `EdgeId` from existing codebase.
- No changes needed; confirm usage in Week 1 kickoff.

### 2. Program → Graph Converter (Dev 1)

- **Output:** `sruja_scan::Graph` (or equivalent) so `compare_graphs` can run.
- **Consumer:** Dev 1’s drift baseline; Dev 2 does not use it.

### 3. ComprehensiveReport (Phase 5)

- **Owner:** `sruja-report` crate. Dev 1 leads schema and structural/intent sections; Dev 2 contributes semantic/runtime sections.
- **Fields:** `structural`, `semantic`, `intent`, `runtime`, `overall_health`, `recommendations`.
- **Recommendation type:** `priority`, `category`, `description`, `affected_components`, `source_layer`, `estimated_effort` (per main roadmap).
- **Agreement:** Week 14; implementation Week 15.

### 4. SemanticIntelligence API (optional)

- If Dev 1 adds semantic alignment to intent drift: Dev 2 exposes `sruja_semantic::analyze()`.
- Contract: input components + structural_edges (from scan graph), output `SemanticReport` (contexts, coupling, summary).
- **Document:** `docs/SEMANTIC_INTELLIGENCE_API.md` (Week 11 handoff).

---

## Conflict Prevention

| Area | Risk | Mitigation |
|------|------|------------|
| `sruja-cli` | Both add subcommands | Dev 1: drift, complexity, intent. Dev 2: why, semantic, runtime. Separate files/modules. |
| `sruja-graph` | Dev 1 adds modules | Dev 2 does not modify sruja-graph. Semantic uses new crate. |
| Config | Both need config | Dev 1 leads `sruja-config` (or agreed `sruja.toml` schema) in Week 14; both consume. |
| Docs | Overlapping edits | Dev 1: architecture, drift, intent. Dev 2: adoption, semantic, runtime, getting-started. |

---

## Independent Start Checklist

### Dev 1 — Can Start Day 1

- [ ] Unify `calculate_health_score` and `calculate_drift_health_score` in sruja-diff
- [ ] Deduplicate cycle reporting in `find_circular_dependencies`
- [ ] Add `sruja-graph/src/scc.rs` stub
- [ ] Create `sruja-graph/src/treewidth.rs` with min-fill heuristic stub

### Dev 2 — Can Start Day 1

- [ ] Create `crates/sruja-semantic/` with Cargo.toml and `lib.rs`
- [ ] Add `EmbeddingProvider` trait in `embedding/provider.rs`
- [ ] Improve `sruja why` evidence templates (no LLM)
- [ ] Reorder README for no-key-first value

---

## Handoff / Integration Points

| Week | From | To | Artifact |
|------|------|----|----------|
| 2 | Both | Both | Agreed `Graph`/node/edge usage |
| 10 | Dev 2 | Dev 1 | `SemanticIntelligence` API (if semantic alignment used) |
| 14 | Both | Both | `ComprehensiveReport` schema |
| 15 | Both | Both | `sruja analyze --all-layers` integration |

---

## Success Criteria for Independent Execution

- **No merge conflicts** on the same files (clear crate/module ownership)
- **No blocking waits** (each track has unblocked work every week)
- **Single integration week** (Week 15) for full-stack wiring
- **Documented APIs** at handoff points to avoid rework

---

## Success Metrics (Product)

Align with ARCHITECTURE_INTELLIGENCE_V2.md; measure after Phase 5.

| Category | Metric | Target |
|----------|--------|--------|
| **Adoption** | Zero-key usage (quickstart, drift, why without API key) | Supported and documented |
| **Adoption** | Time to first value | &lt; 5 min (scan + quickstart) |
| **Quality** | Scan correctness (node/edge extraction) | Unit + integration tests; spot-checks on known repos |
| **Quality** | Drift baseline accuracy | compare_graphs matches expected diff on fixture |
| **Health** | ComprehensiveReport produced | `sruja analyze --all-layers` runs and outputs all four layers |

---

## Testing Expectations

- **Dev 1:** Unit tests for treewidth, SCC, centrality, coupling; unit tests for Program→Graph and compare_graphs; integration tests for drift baseline and intent check (e.g. fixture repos).
- **Dev 2:** Unit tests for embedding provider (stub), clustering, vocabulary; integration tests for `sruja semantic` and `sruja runtime analyze`; mock embedding for CI.
- **Week 15–16:** Joint integration tests for `sruja analyze`; regression tests for ComprehensiveReport shape.
- **Short-term:** Move duplicate cycle/orphan tests from sruja-scan into sruja-diff; add unit tests for `detect_architectural_drift`.

---

## Optional / Deferred

- **Blast radius:** Listed in Track B scope; implement when capacity allows (post–Phase 5 or parallel).
- **Design doc parser:** Week 8; can be minimal (e.g. extract intent from markdown headings) if ADR + .sruja cover most needs.

---

## Document History

| Date | Change |
|------|--------|
| 2026-02-23 | Initial plan based on ARCHITECTURE_INTELLIGENCE_V2.md |
| 2026-02-23 | Assign sruja-report/sruja-config ownership; add short-term items to week plan (god-module threshold, import resolution); add Success Metrics and Testing expectations; clarify Recommendation type; optional/deferred (blast_radius, design doc parser). |
