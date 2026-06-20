---
date: 2026-06-20
topic: sruja-strategy-tracks
focus: improvement ideas across Sruja's three strategy tracks (grounded authoring, grading engine depth, bounded agent delivery)
mode: repo-grounded
---

# Ideation: Sruja Strategy Tracks

## Grounding Context

**Codebase context:** Sruja is a CLI-first autonomous coding agent for architecture-as-code. Rust workspace, 13 crates in 3 tiers (Core: diagnostics, language, engine, export, scan, graph-core, graph, extract; Delivery: wasm, cli; Secondary: diff, intent, agent, memory). Has `.sruja` DSL + linter, MCP server (`sruja mcp`), closed-loop agent (`sruja agent loop`), code-embedded metadata (`@element`/`@layer`/`@boundary`), grounded authoring (`.sruja/author_evidence.json`, proposals → review → `repo.sruja`), agentic memory (`.sruja/agent_memory.json`).

**Gaps / leverage:** Telemetry is greenfield (2/5 strategy metrics need instrumentation: escape-rate, active-consumption). Documented test-coverage gaps (CLI handlers, LSP, WASM, tree-sitter). Heavy `docs/plans` churn (7 run-histories for one task). `docs/solutions/` does not exist yet (no compounding memory).

**External context:** ThoughtWorks Radar Vol 34 (Apr 2026) validates the thesis — "Architecture drift reduction with LLMs," "Feedback sensors for coding agents," "Zero trust for agents," "Mutation testing," "DORA rework rate." Greptile is the closest competitor ($30/seat, 9000+ teams, "Independence" feature, `/greploop`, TREX) — Sruja's differentiator is the formal DSL contract + grounded extraction from legacy. Prior art: Structurizr/C4 (MCP + drift detection), ArchUnit/dependency-cruiser (baseline-violations file), Aider repomap (canonical context distillation), SWE-bench/SWE-agent (hidden-test grading, Agent-Computer Interface), cargo-mutants.

## Topic Axes

1. contract-authoring — extraction/bootstrapping of the `.sruja` contract from messy code
2. grading-rigor — what the deterministic grader checks + how sharp/complete
3. agent-loop — the closed-loop bounded actor (observe→act→verify→critique→replan)
4. host-delivery — reaching devs where they work (MCP, extension, progressive disclosure)
5. measurement — making the grading bet provable (telemetry, escape-rate, adoption, drift)

## Ranked Ideas

### 1. Verify-step grep-gaming
**Description:** The agent loop's "independent grader" (`crates/sruja-agent/src/verify/mod.rs`) grades success via `expected: Option<String>` = "Expected substring in stdout/stderr." A real `.sruja/loop.toml` verifies "dotenvy added" by grepping `Cargo.toml` — an agent can satisfy that with a comment. Add stronger verify-step kinds: AST-contains / symbol-resolves / behavior-runs.
**Axis:** grading-rigor
**Basis:** `direct:` verify/mod.rs `expected` field + loop.toml acceptance checks; `external:` ThoughtWorks "Mutation testing — honest signal for perpetually-green tests."
**Rationale:** If the grader is gameable by string-matching, the entire "agents never grade themselves" thesis is hollow.
**Downsides:** Stronger verify kinds raise authoring cost per manifest.
**Confidence:** 92% **Complexity:** Medium **Status:** Unexplored

### 2. Mutation-test the grader (recall audit)
**Description:** There is no proof the drift/intent/violation detectors actually catch real violations. `behavioral_drift.rs` is 105 lines next to `critique.rs` at 522 — the high-value semantic check is the thinnest, and nothing tests that checks fire on injected drift. Run cargo-mutants-style injection: mutate the code/contract, confirm the grader notices. Pass-rate = the grader's measured recall; blind spots found this way are filled permanently.
**Axis:** grading-rigor
**Basis:** `direct:` line counts (`behavioral_drift.rs` 105 vs `critique.rs` 522) + TEST_COVERAGE_PLAN coverage gaps for drift helpers; `external:` cargo-mutants/Stryker + ThoughtWorks Radar "Mutation testing" (Trial).
**Rationale:** Without an honest signal, "no violations found" is indistinguishable from "the grader is broken." This is the only cheap proof of detector sharpness, and it embodies the product's thesis (nothing grades itself) applied to the tool itself.
**Downsides:** Building a violation-injection harness is non-trivial; recall is a moving target as new violation types are added.
**Confidence:** 85% **Complexity:** Medium **Status:** Explored

### 3. Pre-flight intent gate
**Description:** The loop spends tokens acting then verifying. Wire `sruja intent check` as a hard gate at the `agent plan` → `agent apply` seam: a plan that crosses a forbidden boundary is rejected before any file write. Turns the grader from a critic into a permit system.
**Axis:** agent-loop
**Basis:** `direct:` the plan/apply seam + `intent check` already exist; converged across 3 ideation frames. `external:` SWE-agent Agent-Computer Interface; mini-SWE-agent "simplicity wins."
**Rationale:** A violation caught at intent costs one rejection; caught after implementation costs the full implement→critique→replan cycle.
**Downsides:** Over-tight intent gates could block legitimate evolution (needs the drift-is-the-signal companion).
**Confidence:** 80% **Complexity:** Medium **Status:** Unexplored

### 4. Escape-rate loop governor
**Description:** `docs/plans/` holds 7 run-history JSONs (~39MB) for one task ("add-a-CLI-subcommand"); `max_iterations = 3` yet 7 full runs landed — the loop restarts rather than converges, with no telemetry on why each attempt was abandoned. Make escape-rate (runs-to-convergence + abandon-reason) a first-class loop metric with a hard governor.
**Axis:** agent-loop
**Basis:** `direct:` `docs/plans/run_*` (7 files, Jun 16) + `.sruja/runs/` (10 dirs); strategy names escape-rate as uninstrumented.
**Rationale:** Every restart costs tokens and operator trust, and nobody can see why convergence failed.
**Downsides:** Governor tuning is fiddly; too-aggressive caps abandon solvable tasks.
**Confidence:** 88% **Complexity:** Low-Medium **Status:** Unexplored

### 5. Baseline-ratchet adoption gate
**Description:** Stop requiring teams to reach "zero violations" before Sruja is useful. On `sruja bootstrap`, freeze the current violation set into a baseline; the grader blocks only new violations. Baseline entries decay out as code is touched. Adoption becomes "1 command" not "fix 200 violations first."
**Axis:** contract-authoring
**Basis:** `external:` ArchUnit/dependency-cruiser `.dependency-cruiser-known-violations.json` baseline pattern (proven); converged across 2 ideation frames.
**Rationale:** Cold-start cleanup is the #1 reason teams abandon architecture tooling.
**Downsides:** Table stakes (competitors do this) — adoption unlock, not a differentiator. Fingerprint-rot bug must be fixed first for baselines to be stable.
**Confidence:** 90% **Complexity:** Low-Medium **Status:** Unexplored

### 6. Escape/rework telemetry slice
**Description:** Instrument the two metrics that falsify the bet: escape-rate (changes that passed all gates but were reverted/hot-fixed) and rework-rate (grader-passed changes substantially rewritten within 7 days). This is the in-scope Track 2 light-telemetry slice — no full platform, just the two numbers.
**Axis:** measurement
**Basis:** `direct:` strategy names both as "needs instrumentation"; `external:` DORA rework rate (Radar Adopt) + Langfuse traces→scores.
**Rationale:** "Independence" and "deterministic grading" are untestable claims without rework-rate. If grader-passed code is rewritten at the same rate as ungraded code, the thesis collapses — and you'd have no way to know.
**Downsides:** Needs CI + post-merge drift comparison; the genuine instrumentation gap. Escapes depend on recall (#2) being meaningful.
**Confidence:** 82% **Complexity:** Medium **Status:** Unexplored

### 7. Solo-dev grounding (DSL invisible)
**Description:** Greptile's edge is "no DSL to learn." Counter it: a grounding mode that assumes the user can't author/read a C4 model. The agent drafts the contract from repomap + topology, then asks only yes/no questions ("should CLI depend on the language crate?"). The DSL is invisible by default.
**Axis:** host-delivery
**Basis:** `direct:` `sruja focus`/repomap/agent-assisted extraction exist; `reasoned:` flipping team-size to 1 forces removal of the DSL-authoring tax.
**Rationale:** Keeps the rigor (formal contract) while removing the friction — widens the addressable market and protects the differentiator vs Greptile.
**Downsides:** High complexity (UX overhaul); risk of low-quality contracts from yes/no authoring. Speculative (reasoned basis).
**Confidence:** 70% **Complexity:** High **Status:** Unexplored

## Rejection Summary

| # | Idea | Reason Rejected |
|---|------|-----------------|
| 1 | Orphan false-positive suppression | Real correctness bug — track as an issue, not an ideation survivor |
| 2 | Grounded-authoring synthesis gate | Near-miss — folds into the evidence-tiered authoring synthesis; revisit in brainstorm |
| 3 | Contract-from-drift-history | Novel but lower-confidence than baseline-ratchet for the same adoption problem |
| 4 | Auto-promote proposals on stable evidence | Overlaps with self-promoting; auto-promotion risks integrity before recall audit (#2) exists |
| 5 | Ambiguity-map as primary product | Strategic reframe — better handled in ce-brainstorm than as an improvement idea |
| 6 | Code-leading harvest | Near-miss — part of "drift-is-the-signal" synthesis; recommend brainstorm |
| 7 | Append-only evidence ledger | Infrastructure plumbing; low-novelty, folds into evidence-tiered authoring |
| 8 | Genome-annotation evidence tiers | Near-miss — strong analogy, part of evidence-tiered authoring synthesis; recommend brainstorm |
| 9 | Photogrammetry triangulation | Elegant but over-engineered for near-term; revisit when multi-source evidence matures |
| 10 | Code-embedded contract as first-class | Inverts the source-of-truth model; high-risk architectural change, needs brainstorm |
| 11 | Per-PR ephemeral contracts | Workflow variant worth exploring but lower priority than baseline-ratchet for adoption |
| 12 | Baseline fingerprint rot | Real correctness bug — track as an issue |
| 13 | Intent-from-tests extraction | Near-miss — recommend brainstorm; pairs with multi-surface contract |
| 14 | Decision-trajectory grading | Ambitious reframe — part of "drift-is-the-signal" synthesis; recommend brainstorm |
| 15 | Multi-surface contract | Scope expansion beyond architecture — warrants its own brainstorm to bound |
| 16 | Self-sharpening grader (learning-to-grader promotion) | **Top near-miss** — compounding moat; first brainstorm follow-up to #2 |
| 17 | Contract-derived executable boundary tests | Table stakes — ArchUnit/dependency-cruiser already ship this |
| 18 | Difficulty/Execution panel split | Novel framing but speculative; unclear the split changes agent behavior |
| 19 | Risk-limiting audits | Premature scaling optimization; revisit when repos are large enough to need it |
| 20 | Mutation-graded boundaries | Duplicate of #2 with a contract-vacuity twist; merge into #2 |
| 21 | Grader-as-teacher feedback control | Folds into self-sharpening grader; near-miss |
| 22 | FOQA black box | Near-miss — part of zero-trust admission control synthesis; recommend brainstorm |
| 23 | Phased occupancy permits | Heavy process change; value uncertain before the simpler intent gate (#3) lands |
| 24 | Self-promoting contract (zero-human) | Auto-merge risks integrity before recall audit (#2) and telemetry (#6) exist |
| 25 | Prose-vs-verify gap | Overlaps with #1; the stronger framing is the gameable-grader survivor |
| 26 | Push-only boundary injection | Near-miss — clean DX improvement; recommend brainstorm if #7 isn't pursued |
| 27 | Governance rail (platform buyer) | GTM/positioning reframe — strong but belongs in strategy revision or brainstorm |
| 28 | Curated shared instructions | Already partially shipped (sync-ide-rules); incremental, low novelty |
| 29 | Customs trusted-trader fast lanes | Premature optimization; revisit when grading cost is a measured bottleneck |
| 30 | Attestation-debt metric | Novel but hard to operationalize without signed-attestation infra; revisit post-telemetry |
| 31 | Pharmacovigilance (post-merge incidents) | Needs incident ingestion — edges toward deferred external-context scope |
| 32 | Drift ledger (sanctioned vs unsanctioned) | Near-miss — part of "drift-is-the-signal" synthesis; recommend brainstorm |
| 33 | Actor skill profile | Part of zero-trust admission control synthesis; secondary to intent gate (#3) |

## Cross-cutting syntheses (recommended brainstorm clusters)
- **Self-sharpening grader** — verdicts feed back as negative examples (planner), contract amendments, recall audit, escape governor. Top compounding moat.
- **Drift-is-the-signal** — drift classified sanctioned/unsanctioned; sanctioned = authoring queue; trajectory grading separates evolution from regression. Long-term safety.
- **Evidence-tiered grounded authoring** — confidence tiers + multi-source triangulation + accumulating ledger + ambiguity map. Cold-start fix.
- **Zero-trust admission control** — intent gate + per-actor trust routing + risk-based depth. Bounded delivery operationalized.

## Notes
- Survivors #1 and #2 are existential to the thesis: if the grader is gameable and its recall is unmeasured, the "independent grading" bet is hollow. These should likely land before adoption/delivery work.
- Idea #2 selected for ce-brainstorm handoff (marked Explored).
