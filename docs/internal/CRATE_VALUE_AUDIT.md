# Crate value audit: practical value of each crate

**Context:** After removing `sruja-semantic` (low perceived value), we need to justify each remaining crate with **concrete user-facing value**—not “fancy tools” that exist but don’t deliver practical outcomes.

**Method:** For each crate: (1) what it does, (2) who consumes it, (3) which CLI/extension/user flows depend on it, (4) verdict (essential / useful / thin or at-risk).

---

## Summary table

| Crate | Primary consumer(s) | User-facing value | Verdict |
|-------|--------------------|-------------------|---------|
| **sruja-types** | sruja-scan, sruja-graph | Single source of truth for NodeKind/EdgeKind in scan/graph/diff | **Essential** |
| **sruja-diagnostics** | language, engine, export, cli, lsp | All parse/lint/export errors and formatting | **Essential** |
| **sruja-language** | engine, export, cli, lsp, intent, diff, wasm | Parse DSL → AST; every .sruja path | **Essential** |
| **sruja-engine** | cli, lsp, wasm | Validation rules; `sruja lint` | **Essential** |
| **sruja-export** | cli, lsp, wasm | JSON/Mermaid/Markdown/DOT/context export | **Essential** |
| **sruja-scan** | cli, diff, intent, graph | `scan_repo()` → Graph; all repo-based commands | **Essential** |
| **sruja-diff** | cli | Quickstart, drift, drift-pr, compliance (structural) | **Essential** |
| **sruja-lsp** | extension | LSP server for VS Code (diagnostics, completions) | **Essential** (if we ship extension) |
| **sruja-wasm** | extension (optional) | Lint/export in browser without CLI binary | **Useful** |
| **sruja-cli** | User | The binary; orchestrates everything | **Essential** |
| **sruja-graph** | cli | `why`, analyze (complexity), compliance (KG), merge_scan | **Useful** (why/analyze); **Review** (complexity depth) |
| **sruja-intent** | cli | `sruja intent check`, compliance (intent drift) | **Useful** |
| **sruja-report** | cli | ComplianceReport DTO for `sruja compliance` | **Thin** (DTO only) |

---

## 1. sruja-types

- **What:** Shared enums and types: `NodeKind`, `EdgeKind`, `Severity`; used as the single source of truth for graph node/edge kinds (scan → graph → diff/report).
- **Consumers:** sruja-scan, sruja-graph (re-export).
- **User value:** No direct command; prevents drift between scan output and graph/diff/report interpretations. Without it, we’d duplicate or mismatch kinds across crates.
- **Verdict:** **Essential.** Small, no redundancy.

---

## 2. sruja-diagnostics

- **What:** Diagnostic type, severity, codes, `format_diagnostic()`; used by language (parse errors), engine (rule violations), export (export errors), CLI (lint/validate output), LSP (publishDiagnostics).
- **Consumers:** sruja-language, sruja-engine, sruja-export, sruja-cli, sruja-lsp.
- **User value:** Every `sruja lint`, LSP diagnostic, and validation error goes through this. Users and IDEs see consistent codes and messages.
- **Verdict:** **Essential.**

---

## 3. sruja-language

- **What:** Parser, AST (`Program`), `collect_elements()`; parses .sruja files and exposes structured elements/relations.
- **Consumers:** sruja-engine, sruja-export, sruja-cli, sruja-lsp, sruja-intent, sruja-diff, sruja-wasm.
- **User value:** Every command that reads .sruja (lint, export, validate, diff, intent, drift with `-a`, compile, list, tree, explain) depends on it. Core of “architecture as code.”
- **Verdict:** **Essential.**

---

## 4. sruja-engine

- **What:** Validator and validation rules (e.g. define-before-use, descriptions, no orphans).
- **Consumers:** sruja-cli (lint, validate), sruja-lsp (validation on change), sruja-wasm (in-browser lint).
- **User value:** `sruja lint` and VS Code diagnostics; ensures DSL quality and consistency.
- **Verdict:** **Essential.**

---

## 5. sruja-export

- **What:** Export of AST to JSON, Mermaid, Markdown, DOT; context exporter for AI (cursor-rules, etc.).
- **Consumers:** sruja-cli (export, context export), sruja-lsp (e.g. export for preview), sruja-wasm (export in browser).
- **User value:** `sruja export json|mermaid|markdown|dot`, `sruja context -r .`; extension preview/export. Enables diagrams, docs, and AI context.
- **Verdict:** **Essential.**

---

## 6. sruja-scan

- **What:** `scan_repo()` (Tree-sitter + manifest fallback); produces `Graph` (nodes, edges) from source and/or package manifests.
- **Consumers:** sruja-cli (scan, quickstart, drift, why, analyze, discover, context, compliance, smart_coverage), sruja-diff (actual graph for drift), sruja-intent (reality for intent check), sruja-graph (merge_scan_into_graph).
- **User value:** Every repo-scoped command: `sruja scan`, `sruja quickstart -r .`, `sruja drift -r .`, `sruja why -r .`, `sruja drift -r . -a architecture.sruja`, `sruja intent check -r .`, `sruja compliance -r .`, `sruja context -r .`, `sruja discover`, `sruja smart-coverage`. The only way to get “reality” from code.
- **Verdict:** **Essential.**

---

## 7. sruja-diff

- **What:** `detect_architectural_drift()` (cycles, orphans, layer violations, god modules), `compare_graphs()`, `program_to_graph()`; structural drift and DSL-vs-scan comparison.
- **Consumers:** sruja-cli only (commands/scan.rs: quickstart, drift, drift_pr; compliance: structural violations).
- **User value:**
  - **Quickstart:** `sruja quickstart -r .` → scan + drift → inventory, health score, top violations. First command many users run.
  - **Drift:** `sruja drift -r .` (structural only) or `sruja drift -r . -a arch.sruja` (compare to declared).
  - **Drift PR:** `sruja drift-pr` for PR-scoped new violations.
  - **Compliance:** Structural part of `sruja compliance` (cycles, layers, etc.).
- **Verdict:** **Essential.** Core of “does code match expectations?”

---

## 8. sruja-lsp

- **What:** LSP server (diagnostics, completions, etc.) using language + engine + export + diagnostics.
- **Consumers:** VS Code extension (spawns LSP or uses CLI).
- **User value:** In-editor validation, hover, completions for .sruja. If we ship the extension, LSP is required for good DX.
- **Verdict:** **Essential** if we keep the extension; otherwise optional.

---

## 9. sruja-wasm

- **What:** WASM bindings: parse, lint, export to JSON/Mermaid/Markdown (and incremental parse API).
- **Consumers:** Extension (wasm.ts: getDiagnosticsFromWasm, getMermaidFromWasm, exportMarkdownFromWasm, getElementsFromWasm, getDocumentSymbolsFromWasm). Book can use for live snippets.
- **User value:** Extension can work without a pre-installed `sruja` binary (e.g. in restricted or browser-like environments). In-editor preview and export without CLI.
- **Verdict:** **Useful.** Improves extension portability; not required if we require CLI for all features.

---

## 10. sruja-graph

- **What:** `KnowledgeGraph`, `merge_scan_into_graph()`, `query()` (deterministic why/what/how), analyzers: `SccAnalyzer`, `TreewidthAnalyzer`, `CentralityAnalyzer`, `CouplingAnalyzer`; policy/decision types.
- **Consumers:** sruja-cli: scan.rs (why, merge), analyze.rs (complexity, analyze), compliance.rs (KnowledgeGraph + policies for compliance), graph_store.rs (load/save graph).
- **User value:**
  - **`sruja why "question" -r .`:** Deterministic Q&A from graph (tech, decisions, dependencies). Evidence-based; no LLM. Documented and used in docs/eval.
  - **`sruja drift -r . -a architecture.sruja`:** View-based report (CTO, SRE, etc.) using structural + graph analyzers (SCC, treewidth, centrality, coupling). Delivers “complexity hotspots,” “zone of pain,” refactor suggestions.
  - **`sruja complexity -r .`:** Explicit structural complexity (treewidth, SCC, centrality, coupling). More “academic” but gives concrete metrics.
  - **Compliance:** KG holds policies; compliance command merges scan into KG and checks policies (plus structural + intent).
- **Risk:** “Analyze” and “complexity” are powerful but dense; value depends on whether teams actually act on SCC/treewidth/coupling. **Recommendation:** Keep for `why` + compliance; consider simplifying or making analyze views optional so the main path stays quickstart/drift.
- **Verdict:** **Useful.** `why` and compliance have clear value; analyze/complexity are “prove value or simplify.”

---

## 11. sruja-intent

- **What:** Load intent from .sruja + ADRs (`IntentIntelligence::load_from_directory`), compare to scan graph (`DriftDetector::detect`), produce drift report (undocumented/missing components and relationships, boundary/policy violations).
- **Consumers:** sruja-cli: commands/intent.rs (`sruja intent check`), commands/compliance.rs (intent drift entries in ComplianceReport).
- **User value:**
  - **`sruja intent check -r . -i <dir>`:** “Does the codebase match our declared architecture and ADRs?” Single command for intent vs reality.
  - **Compliance:** Intent drift (and boundary violations) feed into `sruja compliance` status and remediation checklist.
- **Verdict:** **Useful.** Fills a real need (docs vs code); structural only (no semantic matching yet). Keep; consider semantic similarity later if needed.

---

## 12. sruja-report

- **What:** DTOs: `ComplianceReport`, `DriftEntry`, `PolicyViolationEntry`; also `ComprehensiveReport`, `build_recommendations`, layer sections (Structural, Semantic, Intent, Runtime).
- **Consumers:** sruja-cli: commands/compliance.rs uses **only** `ComplianceReport`, `DriftEntry`, `PolicyViolationEntry`. **ComprehensiveReport / build_recommendations / SemanticSection** are **not used** by the CLI (only by sruja-report’s own tests).
- **User value:** `sruja compliance -r . -f json` emits a canonical JSON shape (status, health_score, structural_violations, drift_entries, policy_violations, remediation_checklist). Useful for CI and tooling.
- **Risk:** Half of the crate (comprehensive, semantic layer) is dead code from CLI’s perspective after sruja-semantic removal. SemanticSection and build_recommendations could be removed or kept for a future “full report” API.
- **Verdict:** **Thin.** Compliance DTOs have practical value (CI, scripting). Consider: (a) move ComplianceReport/DriftEntry/PolicyViolationEntry into CLI and delete sruja-report, or (b) keep sruja-report as the single report schema and remove unused comprehensive/semantic types to avoid “fancy unused” surface.

---

## Recommendations

1. **Keep as-is (essential):** sruja-types, sruja-diagnostics, sruja-language, sruja-engine, sruja-export, sruja-scan, sruja-diff, sruja-cli, sruja-lsp (if extension is shipped).
2. **Keep, prove or simplify:** sruja-graph — keep for `why` and compliance; validate that `sruja analyze` / `sruja complexity` are used in practice; if not, trim or make optional.
3. **Keep:** sruja-intent (clear intent-vs-reality value), sruja-wasm (extension value).
4. **Simplify:** sruja-report — kept as minimal compliance schema crate.
5. **Document:** Add a short “when to use which command” (e.g. quickstart vs drift vs intent check vs compliance) in README or book so value is obvious to users.

---

## Commands that deliver immediate practical value (evidence)

These are the flows that appear in docs, eval, and README as primary user entry points:

- **`sruja quickstart -r .`** — No config; inventory + health + top issues. (scan + diff)
- **`sruja drift -r .`** — Structural drift; optional `-a arch.sruja` for declared vs actual. (scan + diff, optional language/diff)
- **`sruja lint <file>`** — Validate .sruja. (language + engine + diagnostics)
- **`sruja export <format> <file>`** — Diagrams and docs. (language + export)
- **`sruja intent check -r . -i <dir>`** — Intent vs reality. (scan + intent + language)
- **`sruja compliance -r .`** — Single compliance status for CI. (scan + diff + intent + graph + report)
- **`sruja why "question" -r .`** — Deterministic evidence-based answers. (scan + graph)

Commands that are “deeper” and need usage evidence: `sruja analyze`, `sruja complexity`, `sruja smart-coverage`. If adoption is low, consider folding key ideas into quickstart/drift or making them opt-in.
