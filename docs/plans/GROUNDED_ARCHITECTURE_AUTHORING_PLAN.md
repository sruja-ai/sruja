# Grounded architecture authoring

**Status:** Implemented (Phases 0–3).  
**Not:** A backlog to over-index on — use this to keep static scan, LLM synthesis, and gates in the right lanes.

---

## Split of responsibilities

| Lane | Tool | Output | Trust |
|------|------|--------|-------|
| **Facts** | `sruja sync`, `sruja author evidence`, manifests | `.sruja/author_evidence.json`, `.sruja/context.json`, optional `repo.sruja.draft` | Evidence — citeable, versioned |
| **Synthesis** | Skill / MCP (default); `sruja author propose --enrich-cmd` (opt-in) | `.sruja/proposals/<id>.json` or `repo.sruja.working` | Hypothesis until promoted |
| **Enforcement** | `sruja lint`, `sruja drift`, `sruja propose approve` | Pass/fail, violations | Machine-checkable |

**Static** answers: workspace units, import communities (as hints), entrypoints, manifest deps, scan scope, drift vs baseline.  
**LLM** answers: domain container names, actors, runtime/data-flow edges — each with `evidence_refs[]` and `open_questions[]` where needed.  
**Never** promote import/call edges or community borders as reviewed C4 without human review.

---

## What ships today

- **`author_evidence/v1`** — `.sruja/author_evidence.json` from `sruja author evidence` and `sruja sync` (capped, ~80 KB).
- **`repo.sruja.draft`** — capped workspace map from `quickstart --generate-baseline` / `init --auto`; structural evidence only.
- **Proposals** — `.sruja/proposals/<id>.json` with optional `evidence_refs`, `open_questions`, `synthesis_notes`; promote via `sruja propose approve <id>`.
- **MCP** — `sruja_get_author_evidence` for synthesis input (prefer over full `graph.json`).
- **Skill** — `sruja-architecture`: synthesis step, proposals/working DSL, no default `repo.sruja` writes.
- **Headless synthesis** — `sruja author propose --enrich-cmd '…'` (JSON Proposal on stdout → saved under proposals).
- **Gates** — lint, drift, intent check, MCP `evaluate_proposal` (score + optional gate command).

---

## Authoring flow (default)

1. `sruja sync -r .` → `author_evidence.json` + `context.json` (avoid loading `graph.json` unless debugging deps).
2. Skill or MCP reads author evidence; **synthesize** domain map (communities are import-cluster hints, not containers).
3. Write `.sruja/proposals/<id>.json` (preferred) or `repo.sruja.working`; list open questions.
4. Human promotes → `sruja propose approve <id>` or copy working → `repo.sruja` → `sruja lint` → `sruja drift -r . -a repo.sruja`.

CLI does **not** call a hosted LLM by default; the editor skill or your `--enrich-cmd` runs the model.

---

## `author_evidence/v1` (summary)

Key fields: `schema_version`, `git_commit`, `truth_status`, `summary`, `workspace_units[]`, `communities[]` (`edge_basis: "import_graph"`), `entrypoints[]`, `data_stores[]`, `repomap_files[]`, `manifest_edges[]`, `open_questions_seed[]`, `excluded_from_default_context[]`.

---

## Anti-patterns

- “Run `--generate-baseline` and you have architecture.”
- Dumping full `graph.json` or 6k-line DSL into default LLM context.
- Using import communities or `Calls` edges as C4 relationships without review.
- Writing `repo.sruja` directly from LLM output without promotion + lint + drift.
- Agent loops that skip lint/drift/evidence.

---

## Later (only when demanded)

- Richer evidence: OpenAPI, K8s, `sruja sources` citations in the bundle.
- Lint rule: require evidence refs on DSL elements.
- `SRUJA_SYNC_WRITE_GRAPH=0` to skip writing `graph.json` on sync.

Track decisions in PR/issue comments; expand this file when a phase changes.
