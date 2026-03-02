# Proving Sruja’s Value on Real Projects

**Purpose:** Address “Is this project actually useful?” with concrete evidence from real codebases.

---

## 1. What “value” means here

Sruja is **architecture-as-code + code-as-truth**. Value is:

| # | Value | What it gives you |
|---|--------|-------------------|
| 1 | **Capture** | Inferred architecture from code (modules, deps, services) without writing docs first. |
| 2 | **Ask questions** | “Why is this structured this way?” over the same graph (`sruja why`). |
| 3 | **Drift** | Compare **code vs declared architecture**; catch violations and missing pieces. |
| 4 | **Timeline / ADR** | Snapshots and drift **across refs**; optional ADR index per ref. |

The health score is **indicative**, not the product. Decisions should use **inventory + findings + drift** (see [TEST_ON_REAL_PROJECTS.md](./TEST_ON_REAL_PROJECTS.md)).

---

## 2. Evidence from real runs (this repo)

### 2.1 Express (framework, ~85 components)

**Command:** From repo root:

```bash
./target/release/sruja quickstart -r evaluation/real-world-test/test-repos/express
```

**Representative output:**

- **Inventory:** 76 modules, 6 services, 2 databases, 1 external API, 50 dependencies.
- **Findings:** Orphan modules (e.g. `index_js`, `lib_response_js`, `lib_request_js`) with concrete suggestions.
- **Health:** 93/100 (capped orphan penalty).
- **Evidence:** Paths like `./index.js`, `./examples/mvc/db.js`, `./examples/mvc/controllers/user/index.js` — real files, real structure.

**Value:** In under a minute you get a structural map and a short list of “review this” items (e.g. unused-looking modules) with no `.sruja` file.

---

### 2.2 Gitea (large product, 15k+ components)

**Command:**

```bash
./target/release/sruja quickstart -r evaluation/real-world-test/test-repos/gitea
```

**Representative output:**

- **Inventory:** 14,742 modules, 176 services, 101 databases, 7 external APIs, 29,260 dependencies.
- **Findings:**
  - **Circular dependency:** `web_src_js_features_comp_ComboMarkdownEditor_ts` ↔ `web_src_js_features_comp_EasyMDEToolbarActions_ts` with a concrete suggestion (interfaces / event-based).
  - **God modules:** 892 modules with >10 deps (e.g. `routers_install_install_go`, `routers_web_*`, `services_*`).
  - **Orphans:** e.g. migration packages, tooling; some are expected (migrations), some worth reviewing.
- **Health:** 78/100.
- **Evidence:** Real paths (`routers/install/routes.go`, `routers/install/install.go`, etc.).

**Value:** On a real product repo you get a dependency graph, cycles, and hotspots (god modules, orphans) without manual diagramming.

---

### 2.3 Drift: code vs declared architecture (Express)

**Command:**

```bash
./target/release/sruja drift -r evaluation/real-world-test/test-repos/express \
  -a evaluation/real-world-test/examples/example_generated_express.sruja
```

**Representative output:**

- **Proposed (DSL):** 27 components. **Actual (scan):** 85.
- **Gap:** “Missing: 85” — the declared architecture is a high-level view; the scan shows the full code shape.
- **Warnings:** 14 components in the DSL with no connections (e.g. “Layer System”, “Middleware Chain”), plus suggestions to define relationships.

**Value:** You see exactly where the **declared** architecture and **code** disagree. That’s the basis for keeping docs aligned or refining the DSL.

---

### 2.4 Timeline and ADR (already in this repo)

- **Captured data:** `evaluation/real-world-test/timelines/express/` has `manifest.json`, `graph_master.json`, `graph_HEAD.json`, `timeline_express.md`, and ADR indices.
- **Timeline report:** `timeline_express.md` describes refs (e.g. master → HEAD) and per-step component/edge deltas. For express, master and HEAD were the same SHA so the diff is empty; the mechanism is proven on real repos.
- **Flow:** `./capture_timeline.sh [REPO] [refs...]` then `./timeline_report.sh [REPO]` (see [TEST_ON_REAL_PROJECTS.md](./TEST_ON_REAL_PROJECTS.md) and [TIMELINE_PLAN.md](./TIMELINE_PLAN.md)).

**Value:** Architecture evolution across refs (and optional ADR context) without re-scanning by hand each time.

---

## 3. How to see it yourself (minimal path)

**Prereqs:** `make build` from Sruja repo root; test repos under `evaluation/real-world-test/test-repos/` (e.g. `./setup_repos.sh` or `./setup_repos.sh --complex`).

```bash
# From Sruja repo root
make build
SRUJA=./target/release/sruja

# 1) Capture + findings (no .sruja)
$SRUJA quickstart -r evaluation/real-world-test/test-repos/express
$SRUJA quickstart -r evaluation/real-world-test/test-repos/gitea

# 2) Drift vs a declared architecture
$SRUJA drift -r evaluation/real-world-test/test-repos/express \
  -a evaluation/real-world-test/examples/example_generated_express.sruja

# 3) Optional: timeline on a repo (from evaluation/real-world-test)
cd evaluation/real-world-test
./capture_timeline.sh express master HEAD
./timeline_report.sh express
cat timelines/express/timeline_express.md
```

---

## 4. Where this leaves “usefulness”

- **Capture:** Useful on real projects: Express and Gitea (and the other test repos) show that Sruja produces a structured inventory and findings from real code (Go, JS/TS, etc.).
- **Drift:** Useful: the Express run shows a clear gap between 27 declared vs 85 actual components and gives targeted suggestions.
- **Timeline/ADR:** Implemented and runnable on real repos; value scales with ref range (e.g. multiple tags) and with ADR presence.

So the project **does deliver value on real projects** in the form of:

1. **Fast, code-based architecture capture** (no docs required to start).
2. **Actionable findings** (cycles, god modules, orphans) with file/module references.
3. **Drift between “what we say” and “what the code is”** with concrete suggestions.
4. **Timeline and ADR capture** across refs for evolution and context.

**Caveats** (from [TEST_ON_REAL_PROJECTS.md](./TEST_ON_REAL_PROJECTS.md)): Scanner supports Go, JS/TS, Python, Rust (not C/C++). Orphan and health rules can be strict (e.g. migrations, tools); use findings and drift for decisions, not the raw score alone.

---

## 5. References

- [TEST_ON_REAL_PROJECTS.md](./TEST_ON_REAL_PROJECTS.md) — What matters (capture, questions, drift, commits), repo tiers, E2E steps, correctness notes.
- [TIMELINE_PLAN.md](./TIMELINE_PLAN.md) — Timeline capture/report design and status.
- [EVALUATION_GUIDE.md](./EVALUATION_GUIDE.md) — How to test “useful architecture docs” (e.g. generated `.sruja`) on real codebases.
