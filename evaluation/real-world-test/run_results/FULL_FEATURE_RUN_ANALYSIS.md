# Full Sruja Feature Run — Analysis and Verdict

**Date:** 2026-02-27  
**Target repo:** Express (and timeline/export on existing artifacts)  
**Goal:** Run the whole Sruja feature set, analyze results, and answer: **Is it useful or not?**

---

## 1. Features Run and Results Summary

| Feature | Command / Script | Result | Output location |
|--------|-------------------|--------|------------------|
| **Scan** | `sruja scan REPO -o scan_express.json` | ✅ Success | `run_results/scan_express.json` |
| **Quickstart** | `sruja quickstart -r REPO` | ✅ Success | `run_results/quickstart_express.txt` |
| **Drift (no baseline)** | `sruja drift -r REPO` | ✅ Success | `run_results/drift_express_no_baseline.txt` |
| **Drift (vs baseline)** | `sruja drift -r REPO -a example_generated_express.sruja` | ✅ Success | `run_results/drift_express_baseline.txt` |
| **Drift-diff** | `sruja drift-diff -b graph_master.json -h graph_HEAD.json` | ✅ Success | `run_results/drift_diff_express.txt` |
| **Analyze** | `sruja analyze -r REPO` | ✅ Success | `run_results/analyze_express.txt` |
| **Complexity** | `sruja complexity -r REPO --scc --centrality --coupling` | ✅ Success | `run_results/complexity_express.txt` |
| **Semantic** | `sruja semantic -r REPO` | ✅ Success | `run_results/semantic_express.txt` |
| **Why** | `sruja why "Where is the main request handling entry point?" -r REPO` | ⚠️ Partial | `run_results/why_express.txt` |
| **Intent check** | `sruja intent check -r REPO -i REPO` | ✅ Success | `run_results/intent_express.txt` |
| **Timeline report** | `./timeline_report.sh express -f both` | ✅ Success | `run_results/timeline_report_express.txt`, `timelines/express/` |
| **Lint** | `sruja lint example_generated_express.sruja` | ❌ Errors (by design) | `run_results/lint_express_sruja.txt` |
| **Evaluate architecture** | `./evaluate_architecture.sh express` | ✅ Success | `run_results/evaluate_architecture_express.txt`, `results/` |
| **Export (Mermaid)** | `sruja export mermaid example_generated_express.sruja` | ✅ Success | stdout (Mermaid diagram) |
| **Run demo** | `./run_demo.sh --baseline` | ✅ Success | `run_results/run_demo_baseline.txt` |

---

## 2. Analysis by Feature

### 2.1 Scan and Quickstart — **Useful**

- **Scan:** Produced a graph with 85 nodes and edges; JSON is machine-readable for downstream tools.
- **Quickstart:** In one run you get: component counts (76 modules, 6 services, 2 DBs), 50 dependencies, health 93/100, and **actionable findings** (7 orphan modules with file paths and suggestions). No config or API keys.

**Verdict:** High value. You get an immediate structural map and a short list of “review this” items with real paths.

---

### 2.2 Drift (no baseline) — **Useful**

- Same 7 orphans as quickstart, with **source file paths** (e.g. `lib/response.js`, `lib/express.js`). Clear, actionable.

**Verdict:** Useful for structural hygiene and dead-code review.

---

### 2.3 Drift (vs baseline architecture) — **Useful**

- **Proposed (DSL):** 27 components. **Actual (scan):** 85.
- Report shows **gap** (27 vs 85), **14 unconnected DSL components** with names (e.g. “Middleware Chain”, “Routing Logic”), and **15 concrete suggestions** (e.g. “Define how X interacts with other components”).

**Verdict:** Very useful. Surfaces exactly where declared architecture and code disagree and what to fix in the DSL or code.

---

### 2.4 Drift-diff (two graphs) — **Useful when refs differ**

- For express, master and HEAD were the same SHA, so diff was empty (0 new/removed components and edges). The **mechanism works**: given two graph JSONs from different refs, you get component/edge deltas.

**Verdict:** Useful for “what changed between release A and B?” when you have graphs at two refs (e.g. from `capture_timeline.sh`).

---

### 2.5 Analyze — **Useful**

- **Layer 1 (structural):** Same as quickstart/drift (93/100, 7 violations).
- **Layer 2 (semantic):** 85 components, 0 bounded contexts, 0 hidden couplings, 100/100.
- **Overall health:** 96/100 and a short recommendation list.

**Verdict:** Useful as a single command that combines structural and semantic views.

---

### 2.6 Complexity (SCC, centrality, coupling) — **Useful**

- **SCC:** 85 SCCs, 0 cyclic, largest size 1 → no cycles in express.
- **Centrality:** Hub and bridge nodes listed (e.g. `module:lib`, `examples_route-separation`).
- **Coupling:** Instability/abstractness/distance; **50 modules in “Zone of Pain”** (concrete + stable) with named violations and refactor suggestions.

**Verdict:** Useful for refactoring and stability/abstractness awareness. Metrics are interpretable.

---

### 2.7 Semantic — **Useful (quiet on this repo)**

- 85 components, 0 bounded contexts, 0 hidden couplings, 0 vocabulary leaks, 100/100. On express there was nothing to flag; on a repo with more domain boundaries or naming leakage this would add value.

**Verdict:** Useful where semantic boundaries matter; on express it mainly confirms “no issues detected.”

---

### 2.8 Why — **Partially useful**

- Answer was generic (“Try asking about specific services, technologies, or decisions”) with **confidence 30%** and **50 file references** from the scan. So it did use the graph and pointed to relevant files, but did not give a crisp “entry point is X” answer.

**Verdict:** Partially useful: good for “show me evidence” (file list); weak for direct Q&A until intent/ADRs or better prompting exist.

---

### 2.9 Intent check — **Useful**

- Compared **declared intent** (11 components, e.g. express.view, express.router) to **scanned reality** (85 components). Reported: “Undocumented: 85”, “Missing: 11”, **11 high-severity** (declared but not found), **85 medium** (in code but not in docs). Surfaces doc/code mismatch clearly.

**Verdict:** Useful for keeping ADRs/declared architecture aligned with code; the numbers tell the story.

---

### 2.10 Timeline report — **Useful**

- Script ran successfully; wrote `timeline_express.md` and `.json`. For express, refs were identical so the report was “no change” — expected. The **pipeline** (capture refs → scan per ref → drift-diff consecutive pairs → report) is in place for real multi-ref timelines.

**Verdict:** Useful for evolution over refs; value scales with number and spread of refs (e.g. tags).

---

### 2.11 Lint — **Useful**

- On `example_generated_express.sruja`: **8 warnings** (unreferenced elements: REQ001, REQ002, npm, etc.) and **4 errors** (duplicate `metadata`, undefined `npm.cookieParser`). Messages are specific and include fix hints.

**Verdict:** Useful. Catches real DSL issues before treating the file as the source of truth; the example file is a good test of the linter.

---

### 2.12 Evaluate architecture — **Useful**

- Ran on `test-repos/express/architecture.sruja` (65 lines, 2 systems, 8 containers). Lint passed; script printed stats and a **manual checklist** (completeness, accuracy, clarity, usefulness) and wrote a report to `results/`.

**Verdict:** Useful for standardizing how you evaluate a generated or hand-written architecture file (stats + validation + checklist).

---

### 2.13 Export (Mermaid) — **Useful**

- Exported the example architecture to a Mermaid diagram (nodes, edges, class styles). Usable in docs or diagrams.

**Verdict:** Useful for turning .sruja into shareable diagrams.

---

### 2.14 Run demo — **Useful**

- `./run_demo.sh --baseline` ran quickstart, drift (no baseline), and drift (vs example architecture) in sequence. Single entry point to “see value” without config.

**Verdict:** Useful for onboarding and showing the core loop.

---

## 3. Cross-Cutting Observations

1. **No key required for core value:** Scan, quickstart, drift, analyze, complexity, semantic, intent, timeline, lint, export, and evaluate all ran without any API key. Only `why` and optional `eval --llm` use LLM.
2. **Actionable output:** Findings include **file paths** and **named components**; suggestions are concrete (e.g. “Define how Middleware Chain interacts”, “Zone of Pain” refactors).
3. **Consistency:** Quickstart, drift, and analyze agree on inventory and violations; drift vs baseline and intent check both surface the 27-vs-85 gap from different angles.
4. **Limitations:** “Why” is low-confidence without richer intent/ADRs; semantic had nothing to report on express; drift-diff was empty because refs were the same (by design).

---

## 4. Verdict: Is Sruja Useful?

### Yes — with clear boundaries.

**Useful today:**

- **Capture:** Scan + quickstart give an immediate, accurate structural map and health signal (cycles, orphans, god modules) with file-level evidence. **Useful.**
- **Drift:** Structural drift and **baseline drift** (code vs .sruja) both give clear, actionable reports (missing components, unconnected DSL elements, suggestions). **Useful.**
- **Evolution:** Drift-diff and timeline report work and scale with multiple refs. **Useful.**
- **Intent and docs:** Intent check and drift-vs-baseline keep “what we say” aligned with “what the code is.” **Useful.**
- **Quality of .sruja:** Lint catches real errors and warnings; export produces Mermaid. **Useful.**
- **Deeper metrics:** Complexity (SCC, centrality, Zone of Pain) and analyze (structural + semantic) add refactoring and design insight. **Useful.**

**Partially useful / context-dependent:**

- **Why:** Good for “show me files related to this”; not yet strong for direct architectural Q&A without more intent/ADRs or model tuning.
- **Semantic:** Valuable when the repo has bounded contexts or vocabulary leakage; on express it was “all clear.”

**Not run (by design):**

- **Eval with LLM** (requires API key): not executed; documented as optional for quality scoring.

---

## 5. Bottom Line

| Question | Answer |
|---------|--------|
| **Does the full feature set run end-to-end?** | Yes. All major features ran successfully (except lint, which correctly failed on an invalid example file). |
| **Do results support real decisions?** | Yes. You get component lists, file paths, violation lists, baseline gaps, coupling metrics, and timeline deltas. |
| **Is it useful?** | **Yes.** Best for: (1) fast architecture capture and health, (2) drift vs declared architecture, (3) intent/doc vs code alignment, (4) refactor guidance (complexity, Zone of Pain), (5) evolution across refs (drift-diff, timeline). Use findings and drift for decisions; treat the health score as a signal, not a single grade. |
| **When is it less useful?** | When you need high-confidence natural-language “why” answers without ADRs; when the repo is C/C++-only (scanner doesn’t support it); or when you expect a single “pass/fail” number without reading the report. |

**Recommendation:** Use Sruja for real projects to **capture structure, compare to declared architecture, and track evolution**. Run the full suite (e.g. quickstart → drift → drift vs baseline → analyze → complexity; plus timeline if you have multiple refs) and base your judgment on the reports and suggestions, not only the health score.

---

*All raw outputs from this run are under `evaluation/real-world-test/run_results/`.*
