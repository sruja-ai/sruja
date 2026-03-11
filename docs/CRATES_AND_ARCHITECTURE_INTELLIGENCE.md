# Crates: Uses and Role in the Architecture Intelligence Demo

This document lists each workspace crate, what it is used for, and how it fits into the **Architecture Intelligence** demo flow: **intent → scan → drift → analyze → AI ask**.

---

## Demo flow (reference)

| Step | Demo command / action | Purpose |
|------|------------------------|---------|
| 1 | Show `architecture.sruja` | The “rulebook” (declared intent) |
| 2 | `sruja scan --output sruja.graph.json` | Build dependency graph from code |
| 3 | `sruja drift -a architecture.sruja` | Compare code graph vs. baseline |
| 4 | `sruja analyze --view cto -t traces.json` | CTO report (structural + optional traces) |
| 5 | `sruja ai ask "..."` or `sruja why "..."` | Answer questions with evidence (LLM or deterministic) |

---

## Core (DSL and tooling)

### sruja-types

**Uses:** Shared type definitions used across crates: `NodeKind`, `EdgeKind`, `NodeId`, `DecisionId`, `PolicyId`, `RequirementId`, `Severity`, and related (de)serialization.

**In the demo:** Not called directly by the demo script. Used by **sruja-scan** (node/edge kinds), **sruja-graph** (re-exports for knowledge graph), **sruja-diff**, **sruja-cli** (views/reports). Everything that represents “component kind” or “relationship kind” ultimately goes through these types.

---

### sruja-diagnostics

**Uses:** Structured diagnostics for the Sruja DSL: `Diagnostic`, `Severity`, `SourceLocation`, `ErrorReporter`, `format_diagnostic`. Used for parse/validation errors and editor feedback.

**In the demo:** Not invoked by the demo steps. Used by **sruja-language** (parser errors), **sruja-engine** (validation), **sruja-cli** (e.g. `lint`, `export`), and **sruja-lsp**. Relevant when editing or validating `.sruja` files (e.g. the rulebook), not when running scan/drift/analyze/ask.

---

### sruja-language

**Uses:** Sruja DSL parsing and AST: `Parser`, `Program`, AST types, `collect_elements` (and related traversal). Parses `.sruja` source into a structured program.

**In the demo:**  
- **Step 1 (intent):** The rulebook `architecture.sruja` is shown as text; parsing is not required for display.  
- **Step 3 (drift):** When you run `sruja drift -a architecture.sruja`, the CLI parses `architecture.sruja` with **sruja-language**, then uses **sruja-diff** to turn that AST into a “proposed” graph and compare it to the scanned graph. So the **intent baseline** for drift is the parsed `.sruja` file.

Also used by: lint, export, tree, fmt, and by **sruja-diff**’s `program_to_graph` (convert DSL → graph for comparison).

---

### sruja-engine

**Uses:** Validation rules for Sruja architectures: cycles, orphans, unique IDs, layer rules, governance, SLO/scenario checks, etc. `Validator`, `Rule`, and rule implementations.

**In the demo:** Not directly invoked by the demo script. Used by **sruja-cli** for `lint` and `validate` on `.sruja` files. If you run `sruja lint demo/architecture.sruja` before or after the demo, **sruja-engine** runs. Drift step uses **sruja-diff** for structural drift (cycles, orphans, god modules), not the DSL validator.

---

### sruja-export

**Uses:** Export Sruja AST to other formats: JSON, Markdown, Mermaid, DSL printer, context export. Used when you have a `.sruja` model and want docs or diagrams.

**In the demo:** Not used in the five demo steps. Used by **sruja-cli** for `export`, and by **sruja-wasm** (browser/Node) for in-editor export. The demo focuses on scan/drift/analyze/ask, not on exporting the rulebook.

---

## Scanning and graph

### sruja-scan

**Uses:** Infer an architecture graph from a repo: Tree-sitter–based parsing of source (Rust, JS/TS, Python, Go, Java, etc.), extraction of modules and dependencies, optional Cargo/npm manifest scanning. Produces a `Graph` (nodes, edges, evidence).

**In the demo:**  
- **Step 2 (scan):** `sruja scan --output sruja.graph.json` is implemented with **sruja-scan** (`scan_repo`). It writes the inferred graph to disk.  
- **Step 3 (drift):** The same graph (from scan or from the saved file) is the “actual” graph compared to the baseline.  
- **Step 4 (analyze):** Analyze starts by scanning the repo (or loading a graph) via **sruja-scan**.  
- **Step 5 (why / ai ask):** The graph fed to “why” or “ai ask” comes from **sruja-scan** (or from the saved `sruja.graph.json`).

So **sruja-scan** is the “reality” side: it turns code into the graph used everywhere in the demo after step 1.

---

### sruja-graph

**Uses:** Architecture knowledge graph: `KnowledgeGraph` (nodes, edges, decisions, policies), `merge_scan_into_graph`, and analyses: centrality, coupling, SCC, treewidth. Also graph query for “why” (evidence-based answers).

**In the demo:**  
- **Step 2 (scan):** Optional: CLI can merge scan result into a `KnowledgeGraph` for later use; the demo mainly writes the scan graph to JSON.  
- **Step 4 (analyze):** **sruja-graph** runs structural analyses (SCC, treewidth, centrality, coupling) and feeds into the CTO view and recommendations.  
- **Step 5 (why / ai ask):** The “why” command loads the scan graph (from file or by re-scanning), merges it into a **sruja-graph** `KnowledgeGraph`, and runs `query(question)` to get an answer and evidence. The “ai ask” path uses the same graph to build context for the LLM.

So **sruja-graph** is the central place for “query the architecture” and for advanced structural metrics in the demo.

---

### sruja-diff

**Uses:** Compare two architecture graphs and detect structural drift: cycles, orphans, layer violations, god modules. Also `program_to_graph` to convert a parsed DSL program (from **sruja-language**) into a graph. Produces `DriftReport`, `Violation`, severity, source refs.

**In the demo:**  
- **Step 3 (drift):** `sruja drift -a architecture.sruja` does: (1) **sruja-scan** for “actual” graph, (2) **sruja-language** parse of `architecture.sruja`, (3) **sruja-diff** `program_to_graph` to get “proposed” graph, (4) **sruja-diff** `compare_graphs(actual, proposed)` to get violations. So **sruja-diff** implements “code vs. baseline” for the demo.  
- **Step 4 (analyze):** **sruja-diff** `detect_architectural_drift` is used for scan-only drift (no baseline) and feeds into the structural part of the report (e.g. god modules, orphans).

**sruja-diff** is the “compare” and “structural drift” crate for the demo.

---

## Intent and reports

### sruja-intent

**Uses:** Intent vs. reality: load declared intent from ADRs (MADR/Nygard, etc.) and from `.sruja` files, build an `IntentModel`, and compare to implementation (boundary drift, undocumented/missing components, policy violations). Used by `sruja intent check` and by the intent layer of `sruja analyze` when an intent path is provided.

**In the demo:**  
- **Step 1 (intent):** The demo only *displays* `architecture.sruja` as the rulebook; it does not run **sruja-intent** in this step.  
- **Step 3 (drift):** The “drift vs. baseline” path uses **sruja-language** + **sruja-diff** (DSL → graph, then compare). It does **not** use **sruja-intent**’s ADR/IntentModel pipeline. So in the current demo, **sruja-intent** is not on the critical path.  
- **Step 4 (analyze):** If you run `sruja analyze -i <dir>` (intent directory), **sruja-intent** loads ADRs and `.sruja` as intent and contributes the “intent” section of the report. The demo’s `analyze --view cto -t traces.json` does not pass `-i`, so **sruja-intent** is optional for the demo as scripted.

**Summary:** **sruja-intent** is used for “intent check” and for the intent layer of analyze when configured; the demo’s drift step uses **sruja-diff** + **sruja-language** for baseline comparison.

---

### sruja-report

**Uses:** Canonical report schema for architecture intelligence: `ComprehensiveReport`, structural/semantic/intent/runtime sections, `Recommendation`, priorities, etc. DTO-only; filled by CLI from **sruja-diff**, **sruja-semantic**, **sruja-intent**, and runtime data.

**In the demo:**  
- **Step 4 (analyze):** **sruja-cli** builds a `ComprehensiveReport` (or equivalent) using **sruja-report** types and fills it from **sruja-scan**, **sruja-diff**, **sruja-graph**, and optionally **sruja-semantic** / **sruja-intent** / traces. So **sruja-report** is the shared “report shape” for the analyze step.

---

## Semantic and AI

### sruja-semantic

**Uses:** Semantic analysis on top of the structure: vocabulary extraction, domain clustering, bounded contexts, semantic coupling, optional embedding-based similarity. Can use a stub provider (no API key) or a real embedding provider.

**In the demo:**  
- **Step 4 (analyze):** When `sruja analyze` runs, it can run **sruja-semantic** (e.g. with `StubEmbeddingProvider`) to add semantic layer to the report. The demo’s analyze command uses it if the code path includes semantic analysis.  
- **Step 5:** Selection/scoring code in **sruja-cli** may use **sruja-semantic** for relevance (e.g. evidence selection); the main “answer” for “why” is from **sruja-graph** query.

**sruja-semantic** is optional in the demo; it enriches analyze and possibly answer quality.

---

### sruja-cli

**Uses:** Command-line interface: subcommands for lint, export, scan, drift, quickstart, why, analyze, context, intent, ai (explain, ask, feedback), timeline, etc. Orchestrates **sruja-scan**, **sruja-diff**, **sruja-graph**, **sruja-intent**, **sruja-report**, **sruja-semantic**, **sruja-language**, **sruja-engine**, **sruja-export**, and internal AI/memory logic.

**In the demo:**  
- **Step 1:** Script shows `architecture.sruja` (no crate call).  
- **Step 2:** `scan` → **sruja-scan** `scan_repo`, write JSON.  
- **Step 3:** `drift -a architecture.sruja` → **sruja-scan** + **sruja-language** + **sruja-diff** (program_to_graph + compare_graphs).  
- **Step 4:** `analyze --view cto -t traces.json` → **sruja-scan**, **sruja-diff**, **sruja-graph**, **sruja-report**, optional **sruja-semantic** and traces.  
- **Step 5:** `ai ask` or `why` → load/scan graph, **sruja-graph** merge + query, and (for `ai ask`) LLM call with context from the graph.

**sruja-cli** is the single entry point the demo uses for all commands.

---

## LSP, WASM (outside the demo script)

Integration with editors is via **skills + CLI** (e.g. Sruja skill in Cursor/Copilot); no MCP server. The demo runs the CLI only.

### sruja-lsp

**Uses:** Language server for the Sruja DSL: diagnostics, hover, completion, etc., for `.sruja` files in editors (e.g. VS Code extension).

**In the demo:** Not used. The demo does not open an editor or run LSP. LSP is for editing the rulebook or other `.sruja` files.

---

### sruja-wasm

**Uses:** WASM build of Sruja for browser/Node: parse DSL, export to JSON/Markdown/Mermaid, validate. Used by the book and the VS Code extension for in-browser or in-editor validation/export.

**In the demo:** Not used. The demo uses the native CLI binary; no WASM is loaded.

---

## Summary: crates in the Architecture Intelligence demo

| Crate | Primary use | In demo (step) |
|-------|-------------|----------------|
| **sruja-types** | Shared node/edge/ID types | Indirect (used by scan, graph, diff, views) |
| **sruja-diagnostics** | DSL diagnostics | No (only if you lint the rulebook) |
| **sruja-language** | Parse `.sruja` | Yes (3: parse baseline for drift) |
| **sruja-engine** | Validate `.sruja` | No (only for lint/validate) |
| **sruja-export** | Export AST to JSON/MD/Mermaid | No |
| **sruja-scan** | Infer graph from repo | Yes (2, 3, 4, 5) |
| **sruja-graph** | Knowledge graph, merge, query, analyses | Yes (4, 5) |
| **sruja-diff** | Drift detection, compare graphs, program_to_graph | Yes (3, 4) |
| **sruja-intent** | ADR + intent model, intent vs. reality | Optional (4 if `-i` used; not in scripted demo) |
| **sruja-report** | Report schema | Yes (4: analyze report) |
| **sruja-semantic** | Semantic analysis | Optional (4, and possibly 5) |
| **sruja-cli** | All commands | Yes (orchestrates 2–5) |
| **sruja-lsp** | LSP for `.sruja` | No |
| **sruja-wasm** | WASM parse/export/validate | No |

**Critical path for the scripted demo:** **sruja-cli** → **sruja-scan** (step 2) → **sruja-language** + **sruja-diff** (step 3) → **sruja-scan** + **sruja-diff** + **sruja-graph** + **sruja-report** (step 4) → **sruja-graph** + **sruja-scan** (step 5).

For how **representation** (essential architecture), **drift**, **policies**, and **compliance** fit together, see [REPRESENTATION_DRIFT_POLICY_COMPLIANCE.md](REPRESENTATION_DRIFT_POLICY_COMPLIANCE.md).
