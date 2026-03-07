# Is Sruja Useful on Real Projects? — Verdict

**Short answer: Yes.** Sruja is useful on real projects for **capturing structure, finding drift, and comparing code to declared architecture**. It is less useful when you need high-confidence natural-language Q&A without ADRs, or when the repo is outside supported languages (no C/C++ scanner).

---

## What Was Run (This Session)

| Test | Repo | Result |
|------|------|--------|
| **run_demo.sh** | Express | ✅ Quickstart (97/100), drift (7 orphans with file paths) |
| **quickstart** | Sruja (this repo) | ✅ 1253 components, 87/100, 19 orphans, 107 “god modules” with paths |
| **drift vs baseline** | Express | ✅ Proposed 27 vs actual 85; 14 unconnected DSL components + 15 concrete suggestions |

---

## Where Sruja Is Clearly Useful

1. **Fast architecture capture** — `sruja quickstart -r .` gives component counts, health score, and **actionable findings with file paths** (orphans, cycles, god modules). No config or API keys.

2. **Structural drift** — `sruja drift -r .` surfaces orphans and violations with **source file paths** (e.g. `lib/response.js`, `lib/express.js`). Good for dead-code and hygiene review.

3. **Declared vs actual** — `sruja drift -r . -a architecture.sruja` compares the DSL to the scan. You see **gap (e.g. 27 vs 85)**, which DSL components are unconnected, and concrete suggestions (“Define how X interacts with…”). Very useful for keeping docs and code aligned.

4. **Refactor guidance** — Complexity (SCC, centrality, Zone of Pain) and `sruja analyze` give interpretable metrics and named violations. Useful for prioritising refactors.

5. **Evolution over refs** — Drift-diff and timeline report work across refs (e.g. master vs HEAD, tags). Valuable when you have multiple refs to compare.

6. **DSL quality** — `sruja lint` catches real errors/warnings; export produces Mermaid. Useful for treating `.sruja` as source of truth.

---

## Limitations (When It’s Less Useful)

- **“Why” Q&A** — `sruja why "question"` can return low-confidence, generic answers without richer intent/ADRs. Better for “show me related files” than crisp architectural Q&A today.
- **Semantic layer** — On Express, semantic had nothing to flag (0 bounded contexts, 0 hidden couplings). Value increases on repos with clearer domain boundaries and naming leakage.
- **Scanner coverage** — No C/C++ support; value is highest on JS/TS, Rust, Go, Python.
- **Orphan false positives** — Entry points or dynamically required modules (e.g. Express `lib/express.js`) can appear as “orphans” because static analysis doesn’t see all edges. Still actionable as “review this file.”

---

## How to Reproduce

```bash
# From repo root: build CLI
make build

# Fast path (~2 min, no config)
cd evaluation/real-world-test
./run_demo.sh

# Optional: drift vs example architecture
./run_demo.sh --baseline

# On your own repo
sruja quickstart -r /path/to/your/repo
sruja drift -r /path/to/your/repo
sruja drift -r /path/to/your/repo -a architecture.sruja
```

---

## Recommendation

Use Sruja on real projects to **capture structure, compare to declared architecture, and track evolution**. Run quickstart → drift → drift vs baseline → analyze (and timeline if you have multiple refs). Base decisions on the **reports and suggestions**, not only the health score.

For a full feature-by-feature analysis and scores, see **run_results/FULL_FEATURE_RUN_ANALYSIS.md**.
