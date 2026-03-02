# When We Analyze a Repo, What Are We Doing?

A step-by-step explanation of what Sruja does when you run `sruja quickstart -r <repo>`, `sruja scan`, or `sruja drift` on a repository.

---

## 1. Scan: Build an architecture graph from source code

**Goal:** Turn the repo’s source tree into a **graph** of **nodes** (modules, files, components) and **edges** (dependencies).

### 1.1 Walk the repository

- We **walk the directory tree** under the repo root (respecting `.gitignore`).
- We **skip** (by default): `node_modules`, `target`, `dist`, `build`, and paths containing `test` / `spec` / `__tests__`.
- We only consider **supported languages** (Go, JavaScript, TypeScript, Python, Rust) and **supported files** (by extension and, where needed, content).

**Where:** `sruja-scan` (e.g. `tree_sitter::build_walker`, `scan_with_tree_sitter`).

### 1.2 Detect language per file

- For each file we **detect language** from path/extension (and sometimes content).
- If the language is not supported, we skip the file.

**Where:** `sruja-scan/src/tree_sitter/detector.rs` (e.g. `detect_language`).

### 1.3 Parse each source file

- We **read the file** (up to a max size, e.g. 500 KB) and **parse it** with **Tree-sitter** (language-specific grammar).
- From the parse tree we extract:
  - **Imports** (e.g. `import X from 'Y'`, `from Z import W`, `use crate::...`, `#include`)
  - **Exports** and sometimes **definitions** (e.g. functions, classes) to infer “components” and to classify nodes (module vs service vs database by heuristics).

**Where:** `sruja-scan/src/tree_sitter/` and language-specific parsers under `tree_sitter/languages/` (e.g. `go.rs`, `rust.rs`, `python.rs`, JS/TS).

### 1.4 Build nodes and edges

- **Nodes:** We create one node per **file** (and, in the same pass, we ensure a **parent module** node per directory). Node kind (e.g. Module, Service, Database) is inferred from the parsed file (e.g. DB-like names, “api” paths).
- **Edges:** For each import we add an edge **from the importer (file or module) to the imported target**. Targets are resolved to node IDs (same repo or external). So the graph is “who depends on whom.”

Result: a **Graph** = `{ nodes: [...], edges: [...] }` with metadata. This is what `sruja scan -o out.json` writes.

**Where:** `sruja-scan/src/tree_sitter.rs` (aggregate `file_imports`, create `nodes` and `edges`), `sruja-scan/src/graph.rs` (Graph type).

### 1.5 Optional: package-manifest-only scan

- For **npm** (package.json) or **Cargo** (Cargo.toml) we can also do a **manifest-only** scan: we read the manifest and build a graph from package/dependency lists only (no Tree-sitter). This is used when full source parsing isn’t used.

**Where:** `sruja-scan/src/npm.rs`, `sruja-scan/src/cargo.rs`.

---

## 2. Drift: Analyze the graph for structural issues

**Goal:** Take the **scan graph** and detect **violations**: cycles, orphans, layer violations, god modules. No extra “declared” architecture is required for this step.

### 2.1 Circular dependencies

- We treat the graph as a **directed graph** (nodes = modules/files, edges = dependencies).
- We **find cycles** (e.g. A → B → C → A) using a DFS-based cycle detection.
- Each cycle is reported as a **violation** (red flag).

**Where:** `sruja-diff/src/drift.rs` (e.g. `find_circular_dependencies`).

### 2.2 Orphan modules

- We find **modules with no incoming and no outgoing dependency edges** (in the main dependency graph; we ignore “containment” edges like module → file).
- We **exclude** nodes that look like **tests, examples, tools, or doc** (by path and id), so the count reflects product code.
- Each remaining orphan is reported as a **violation** (possible dead code or missing integration).

**Where:** `sruja-diff/src/drift.rs` (`find_orphan_modules`, `is_likely_doc_or_tool_path`).

### 2.3 Layer violations

- We apply **simple layer rules**: e.g. “frontend” nodes (label contains frontend/ui/web) should not have a **direct** edge to “database” nodes.
- Any edge from such a “frontend” node to a “database” node is a **layer violation** (e.g. “introduce a service layer”).

**Where:** `sruja-diff/src/drift.rs` (`find_layer_violations_advanced`).

### 2.4 God modules

- We count **out-degree** per node (how many dependencies it has).
- Nodes with **more than a threshold** (e.g. 10) dependencies are “god modules.” We **exclude** nodes under test/example/tools/doc paths.
- Each is reported as a **violation** (refactor suggestion).

**Where:** `sruja-diff/src/drift.rs` (`find_god_modules`, same path filter as orphans).

### 2.5 Drift report

- We collect all violations (with kind, severity, message, location, suggestion, source refs).
- We compute **counts** (e.g. number of cycles, orphans, layer violations, god modules) and a **health score** from those counts (see below).
- The **DriftReport** is what the CLI prints (findings, score, suggestions) and can be used for JSON output.

**Where:** `sruja-diff/src/drift.rs` (`detect_architectural_drift` / `detect_architectural_drift_with_config`).

---

## 3. Health score: One number from violations

**Goal:** Turn the list of **violations** into a single **0–100** score so you get a quick signal (with the understanding that the real value is in the findings, not the number alone).

- We **count** violations by kind: cycles, layer violations, orphans, god modules, and “other.”
- We **subtract** from 100 using fixed rules, e.g.:
  - Cycles: penalty per cycle (capped).
  - Layer violations: penalty per violation (capped).
  - Orphans: penalty that scales with count (capped).
  - God modules: penalty that scales with count (capped).
- We apply a **floor** (e.g. 50) so the score never goes below that.
- Result: **health_score** in the drift report and in CLI output.

**Where:** `sruja-diff/src/health.rs` (`calculate_health_score_from_violations`).

---

## 4. How the CLI uses this (quickstart / scan / drift)

- **`sruja scan -r <repo> -o out.json`**  
  Runs **Step 1** only: walk → parse → build graph → write JSON. No drift, no score.

- **`sruja drift -r <repo>`**  
  Runs **Step 1** (scan) then **Step 2** (drift on that graph) and **Step 3** (health score). Prints report (and optionally JSON). Does **not** compare to a baseline `.sruja` unless you pass `-a architecture.sruja`.

- **`sruja quickstart -r <repo>`**  
  Same as drift for the structural part: **Step 1 → Step 2 → Step 3**, then prints a **summary** (inventory, top findings, score, actionable fixes, next steps). No baseline file required.

- **`sruja drift -r <repo> -a architecture.sruja`**  
  Runs scan and drift, then **additionally** compares the **scan graph** to the **declared architecture** (from the `.sruja` file): proposed vs actual components/edges, missing/unexpected nodes, and suggestions. The same health score logic can be used in that comparison path.

---

## 5. Short summary

| Step | What we do |
|------|------------|
| **Scan** | Walk repo → detect language → parse source (Tree-sitter) → extract imports/deps → build graph (nodes + edges). |
| **Drift** | On that graph: find cycles, orphans (excluding test/example/tools/doc), layer violations (e.g. frontend→DB), god modules → produce a list of violations. |
| **Score** | From violation counts: subtract penalties (capped) from 100, apply floor → health score. |

So when we **analyze a repo**, we are: **(1) inferring a dependency graph from source code**, **(2) checking that graph for structural issues**, and **(3) summarizing that into a single score and a list of findings.** We do **not** run the code or tests; we only **parse** supported source files and **analyze** the resulting graph.

---

## 6. Is this really sufficient?

**Short answer:** Without semantic meaning, real layering, runtime/deployment, data flow, more languages, design quality, and trend — the current analysis is **not** sufficient for real architecture work. For anyone who needs those, it can rightly feel **useless**.

### What it *is* sufficient for

- **Structural snapshot:** “What does the dependency graph look like? Where are the cycles and orphans?”
- **Drift vs declared design:** “Does the code match our `.sruja` (or ADR)?” when you have a baseline.
- **Quick health signal:** A single number and a short list of findings to triage.
- **Supported languages:** Go, JS, TS, Python, Rust. For those, the graph is derived from real imports.

So for **“see structure, find obvious problems, compare to what we said”** it is sufficient.

### What it is *not* sufficient for

| Gap | Why it matters |
|-----|-----------------|
| **No semantic meaning** | We see “A imports B,” not “A is the payment service” or “B is the audit logger.” We can’t reason about domain or responsibility. |
| **Crude layering** | “Frontend” and “database” are inferred from labels/paths. We don’t model real rules like “domain must not depend on infra” or “adapters depend on domain.” |
| **No runtime/deployment** | We don’t see processes, containers, or network boundaries. “Which service talks to which over the wire?” is outside this. |
| **No data flow** | We see dependency edges, not “this data flows from A to B” or read vs write. Data architecture isn’t modeled. |
| **Limited languages** | No C, C++, Java, etc. Repos that are mostly unsupported get a partial or empty graph. |
| **Heuristics, not truth** | Node kinds (service, database) and layer rules are heuristics. They can be wrong. Orphans can be entry points; “god” modules can be facades. |
| **Single snapshot** | We don’t know trend (getting better or worse) unless you run again or use timeline/drift-diff. |
| **No quality of design** | We don’t measure test coverage, complexity, or whether dependencies are stable vs volatile. |

So for **“understand the whole architecture, domain, runtime, and data flow, and judge quality”** it is **not** sufficient by itself.

### How to use it

- **Do:** Use it for structural checks, drift vs declared architecture, and a quick triage signal. Treat the score as one input, not the only truth.
- **Don’t:** Rely on it alone for “is this architecture good?” or for domain/runtime design. Combine with docs, ADRs, and human judgment.
- **To be useful for real architecture:** the project would need semantic/domain awareness, real layer rules, runtime or data-flow inputs, more languages, quality/trend — or to be positioned as a building block other tools compose with.
