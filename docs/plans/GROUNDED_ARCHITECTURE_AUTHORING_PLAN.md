# Grounded architecture authoring

**Status:** Active principles (implementation in progress).  
**Not:** A backlog to over-index on — use this to keep static scan, LLM synthesis, and gates in the right lanes.

---

## Split of responsibilities

| Lane | Tool | Output | Trust |
|------|------|--------|-------|
| **Facts** | `sruja scan`, sync, manifests | Graph, `.sruja/context.json`, `repo.sruja.draft` | Evidence — citeable, versioned |
| **Intent** | Human + editor LLM (skill, MCP) | Proposal → `repo.sruja` | Reviewed truth after lint + drift + merge |
| **Enforcement** | `sruja lint`, `sruja drift`, CI, `propose` | Pass/fail, violations | Machine-checkable |

**Static** answers: what packages exist, declared deps, scan scope, drift vs declared model.  
**LLM** answers: naming, boundaries, actors, flows, open questions — grounded in evidence, never auto-promoted.

Do not treat call/import graphs, folder clusters, or health scores as architecture relationships without review.

---

## What ships today (this repo)

- **`repo.sruja.draft`** — capped workspace map from Cargo/npm (`draft_summary.rs`); no call-graph edges in DSL.
- **`--generate-baseline`** / **`init --auto`** — write draft only; skip if `repo.sruja` or draft already exists (unless `--force`).
- **Reviewed baseline** — `repo.sruja` via skill + human edit or `sruja propose approve`.
- **Gates** — lint, drift, intent check, MCP `evaluate_proposal` (optional).

---

## Authoring flow (default)

1. `sruja sync -r .` and/or `sruja quickstart -r . --generate-baseline` → evidence + optional draft.
2. `sruja focus --file <path>` or MCP ladder for task scope (not full DSL dump).
3. LLM proposes DSL or proposal artifact; list **open questions** where evidence is thin.
4. Human promotes to `repo.sruja` → `sruja lint` → `sruja drift -r . -a repo.sruja` → CI.

CLI does **not** call an LLM in v1; it prepares ground. The editor skill runs the model.

---

## Anti-patterns

- “Run `--generate-baseline` and you have architecture.”
- Dumping full `graph.json` or 6k-line DSL into default LLM context.
- Using `Calls` / import edges as C4 relationships without review.
- Agent loops that skip lint/drift/evidence.

---

## Later (only when demanded)

- `sruja author evidence` — formal evidence bundle JSON (schema in original review doc if needed).
- Richer evidence: OpenAPI, K8s, ingest citations — still **evidence**, not auto-truth.
- Optional headless LLM with explicit opt-in.

Track decisions in PR/issue comments; expand this file when a phase actually lands.
