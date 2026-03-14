# Test plan: proving value on real projects and crate contribution to architecture maintenance

**Purpose:** Demonstrate that each crate delivers practical value for **architecture maintenance** when run on real (or real-like) projects. This plan ties test scenarios to crates and to measurable outcomes.

**Related:** [CRATE_VALUE_AUDIT.md](CRATE_VALUE_AUDIT.md) (what each crate does), [evaluation/real-world-test/](../evaluation/real-world-test/) (scripts and test-repos).

---

## 1. Architecture maintenance dimensions

We define **architecture maintenance** as:

| Dimension | Goal | Example question |
|-----------|------|-------------------|
| **Document** | Keep architecture as code valid and exportable | Is our .sruja valid? Can we generate diagrams? |
| **Discover** | Infer structure from code without prior docs | What components and dependencies exist? |
| **Validate** | Catch structural problems (cycles, orphans, layers) | Are there cycles or god modules? |
| **Compare** | Align code with declared intent | Does code match our architecture file / ADRs? |
| **Enforce** | Single gate for CI (structural + intent + policy) | Should this PR pass architecture checks? |
| **Explain** | Answer “why” from graph and decisions | Why do we use technology X? |

Each test scenario below maps to one or more dimensions and to the crates that enable them.

---

## 2. Crate → maintenance dimension → test scenario

| Crate | Maintenance dimension(s) | Primary test scenario(s) |
|-------|---------------------------|---------------------------|
| **sruja-types** | (infra: consistent kinds across scan/graph/diff) | Implicit in all scan/drift/graph flows; no standalone test. |
| **sruja-diagnostics** | Document | TC-DOC-1 (lint errors have codes/locations). |
| **sruja-language** | Document | TC-DOC-1, TC-DOC-2, TC-CMP-1, TC-INT-1. |
| **sruja-engine** | Document | TC-DOC-1 (lint rules). |
| **sruja-export** | Document | TC-DOC-2 (export formats), TC-DOC-3 (context for AI). |
| **sruja-scan** | Discover | TC-DIS-1, TC-DIS-2; feeds all drift/intent/compliance/why. |
| **sruja-diff** | Validate, Compare | TC-VAL-1, TC-VAL-2, TC-CMP-1. |
| **sruja-graph** | Explain, Enforce | TC-EXP-1, TC-ENF-1 (compliance KG). |
| **sruja-intent** | Compare, Enforce | TC-INT-1, TC-ENF-1. |
| **sruja-report** | Enforce | TC-ENF-1 (ComplianceReport shape). |
| **sruja-cli** | All | Orchestrates every scenario. |
| **sruja-lsp** | Document | TC-DOC-4 (in-editor validation). |
| **sruja-wasm** | Document | TC-DOC-5 (extension lint/export without CLI). |

---

## 3. Test scenarios (real projects)

Use **test-repos** from `evaluation/real-world-test/` (e.g. express, fastapi, gitea) or any real repo. Build CLI: `make build` from repo root; set `SRUJA` to `./target/release/sruja` or ensure `sruja` is on PATH.

---

### TC-DOC: Document (DSL quality, export, context)

**Crates exercised:** sruja-language, sruja-diagnostics, sruja-engine, sruja-export, sruja-lsp, sruja-wasm.

| ID | Scenario | Steps | Expected outcome | Success criteria |
|----|----------|--------|-------------------|------------------|
| **TC-DOC-1** | Lint catches errors and reports codes | 1. Use a .sruja file (e.g. `book/valid-examples/pattern-microservices.sruja`). 2. Introduce an error (e.g. reference undefined component). 3. Run `sruja lint <file>`. | Diagnostics with severity and code (e.g. undefined reference); exit non-zero for errors. | At least one rule from sruja-engine fires; output uses sruja-diagnostics format. |
| **TC-DOC-2** | Export to multiple formats | 1. Run `sruja export json <file>`, `sruja export mermaid <file>`, `sruja export markdown <file>`. | Valid JSON; valid Mermaid; readable Markdown. | All three exports succeed; Mermaid renders in a viewer. |
| **TC-DOC-3** | Context export for AI | 1. Run `sruja context export -r <repo> -f cursor-rules -o /tmp/out.cursorrules`. | File contains structured architecture context (components, deps, technologies). | File exists and is consumable by cursor-rules or similar. |
| **TC-DOC-4** | LSP diagnostics in editor | 1. Open a .sruja file in VS Code with Sruja extension. 2. Break syntax or reference. | Underline/panel shows diagnostic with message and location. | LSP (sruja-lsp) provides diagnostics; no CLI required for this check. |
| **TC-DOC-5** | WASM lint/export without CLI | 1. In extension, use “export to Mermaid” or lint when CLI path is not set but WASM is present. | Export or lint works via WASM. | sruja-wasm used; no `sruja` binary on PATH required. |

---

### TC-DIS: Discover (infer structure from code)

**Crates exercised:** sruja-scan (and sruja-types via scan output).

| ID | Scenario | Steps | Expected outcome | Success criteria |
|----|----------|--------|-------------------|------------------|
| **TC-DIS-1** | Quickstart on real repo | 1. Run `sruja quickstart -r <repo>` (e.g. `evaluation/real-world-test/test-repos/express`). | Inventory (component counts), health score, top findings (e.g. orphans, god modules) with file paths. | Scan completes; output includes numbers and at least one finding with path. |
| **TC-DIS-2** | Scan produces graph | 1. Run `sruja scan -r <repo> -o /tmp/graph.json`. 2. Inspect JSON. | Nodes and edges; node ids/labels and paths where available. | JSON has `nodes` and `edges`; node kinds align with sruja-types. |

**Script:** `evaluation/real-world-test/run_demo.sh` (quickstart + drift).

---

### TC-VAL: Validate (structural drift)

**Crates exercised:** sruja-scan, sruja-diff.

| ID | Scenario | Steps | Expected outcome | Success criteria |
|----|----------|--------|-------------------|------------------|
| **TC-VAL-1** | Drift without baseline | 1. Run `sruja drift -r <repo>`. | List of violations: cycles, orphans, layer violations, god modules; each with evidence (paths where applicable). | At least one violation type reported; exit code and output usable for CI. |
| **TC-VAL-2** | Drift with fail_on | 1. Run `sruja drift -r <repo> --fail-on cycles`. | Exit 1 if cycles exist; 0 otherwise. | Exit code reflects presence of cycles. |

**Script:** `evaluation/real-world-test/run_demo.sh` (drift phase).

---

### TC-CMP: Compare (code vs declared architecture)

**Crates exercised:** sruja-language, sruja-scan, sruja-diff.

| ID | Scenario | Steps | Expected outcome | Success criteria |
|----|----------|--------|-------------------|------------------|
| **TC-CMP-1** | Drift vs architecture file | 1. Have a repo and a .sruja file (e.g. generated or from `test-repos/express/architecture.sruja`). 2. Run `sruja drift -r <repo> -a <arch.sruja>`. | Diff summary (e.g. proposed vs actual counts); list of unconnected DSL components; concrete suggestions. | Output shows gap between declared and scanned; suggestions are actionable. |

**Script:** `evaluation/real-world-test/run_demo.sh --baseline`.

---

### TC-INT: Intent (declared intent vs reality)

**Crates exercised:** sruja-intent, sruja-language, sruja-scan.

| ID | Scenario | Steps | Expected outcome | Success criteria |
|----|----------|--------|-------------------|------------------|
| **TC-INT-1** | Intent check with .sruja and optional ADRs | 1. Repo with `docs/architecture/` (or custom `-i` dir) containing at least one .sruja file (and optionally `adr/decisions/*.md`). 2. Run `sruja intent check -r <repo> -i <dir>`. | Drift score, health, list of violations (undocumented/missing components or relationships, boundary violations); suggestions. | Intent model loaded from .sruja (and ADRs if present); report compares declared vs scan. |

**Note:** If repo has no intent dir, use a directory that contains a .sruja file and optionally ADRs.

---

### TC-ENF: Enforce (compliance gate)

**Crates exercised:** sruja-scan, sruja-diff, sruja-intent, sruja-graph, sruja-report.

| ID | Scenario | Steps | Expected outcome | Success criteria |
|----|----------|--------|-------------------|------------------|
| **TC-ENF-1** | Compliance report (structural + intent) | 1. Run `sruja compliance -r <repo> -a <arch.sruja> -i <intent_dir> -f json` (or omit `-a`/`-i` if not available). | JSON: `status` (compliant/non_compliant), `health_score`, `structural_violations`, `drift_entries`, `policy_violations`, `remediation_checklist`. | ComplianceReport shape (sruja-report); status reflects structural and intent drift. |

---

### TC-EXP: Explain (why / evidence from graph)

**Crates exercised:** sruja-scan, sruja-graph.

| ID | Scenario | Steps | Expected outcome | Success criteria |
|----|----------|--------|-------------------|------------------|
| **TC-EXP-1** | Why question with evidence | 1. Run `sruja why "Why do we use Express?" -r <repo>` (or a technology present in the repo). | Answer plus confidence; evidence (graph nodes/edges or file refs). | Deterministic answer; evidence lines reference repo or graph. |

**Note:** Value is higher when repo has ADRs/decisions in the graph; otherwise answers may be generic. Still proves sruja-graph query path.

---

## 4. Per-crate proof table

Use this to record that each crate was exercised and how.

| Crate | How we prove value | Test IDs |
|-------|--------------------|----------|
| **sruja-types** | Consistent NodeKind/EdgeKind in scan output and drift/graph; no duplicate enums. | TC-DIS-2 (and all scan-based flows). |
| **sruja-diagnostics** | Lint and validation output have codes, severity, locations. | TC-DOC-1. |
| **sruja-language** | Every .sruja read (lint, export, drift -a, intent) parses and yields elements. | TC-DOC-1, TC-DOC-2, TC-CMP-1, TC-INT-1. |
| **sruja-engine** | Lint reports rule violations (e.g. missing description, undefined ref). | TC-DOC-1. |
| **sruja-export** | Export to JSON/Mermaid/Markdown and context export succeed. | TC-DOC-2, TC-DOC-3. |
| **sruja-scan** | quickstart, drift, intent check, compliance, why all get a graph from repo. | TC-DIS-1, TC-DIS-2, TC-VAL-1, TC-CMP-1, TC-INT-1, TC-ENF-1, TC-EXP-1. |
| **sruja-diff** | Drift and quickstart show violations; drift -a shows declared vs actual. | TC-VAL-1, TC-VAL-2, TC-CMP-1, TC-ENF-1. |
| **sruja-graph** | `why` returns answer + evidence; compliance uses KG for policy/merge. | TC-EXP-1, TC-ENF-1. |
| **sruja-intent** | Intent check and compliance report include intent drift (undocumented/missing, boundaries). | TC-INT-1, TC-ENF-1. |
| **sruja-report** | Compliance JSON has ComplianceReport shape and remediation checklist. | TC-ENF-1. |
| **sruja-cli** | All scenarios run via CLI subcommands. | All. |
| **sruja-lsp** | Editor shows diagnostics for .sruja without running CLI. | TC-DOC-4. |
| **sruja-wasm** | Extension lint/export works without CLI when WASM is bundled. | TC-DOC-5. |

---

## 5. Execution checklist (real projects)

**One-time setup**

- [ ] Build: `make build` (or install sruja CLI).
- [ ] (Optional) Clone test-repos: `cd evaluation/real-world-test && ./setup_repos.sh`.
- [ ] Pick at least one “real” repo: e.g. `test-repos/express`, `test-repos/gitea`, or your own.

**Run scenarios (minimum to prove all crates)**

1. **Document:** Lint a .sruja file; export to JSON and Mermaid; run context export on a repo. (TC-DOC-1, TC-DOC-2, TC-DOC-3)
2. **Discover:** `sruja quickstart -r <repo>`; `sruja scan -r <repo> -o /tmp/g.json`. (TC-DIS-1, TC-DIS-2)
3. **Validate:** `sruja drift -r <repo>`; optionally `--fail-on cycles`. (TC-VAL-1, TC-VAL-2)
4. **Compare:** `sruja drift -r <repo> -a <arch.sruja>`. (TC-CMP-1)
5. **Intent:** `sruja intent check -r <repo> -i <dir_with_sruja>`. (TC-INT-1)
6. **Enforce:** `sruja compliance -r <repo> -f json` (optionally with `-a` and `-i`). (TC-ENF-1)
7. **Explain:** `sruja why "Why <technology>?" -r <repo>`. (TC-EXP-1)
8. **LSP/WASM:** Open .sruja in VS Code; trigger lint/export with and without CLI. (TC-DOC-4, TC-DOC-5)

**Fast path (single script)**

- Run `evaluation/real-world-test/run_demo.sh` for quickstart + drift.
- Run `evaluation/real-world-test/run_demo.sh --baseline` to add drift vs architecture.
- Manually run intent check, compliance, and why on the same repo to cover remaining crates.

---

## 6. Success criteria for “value proven”

- **Per dimension:** At least one scenario for Document, Discover, Validate, Compare, Enforce, Explain passes on at least one real repo.
- **Per crate:** Every crate in the table in §4 is exercised by at least one passing scenario (except sruja-types, which is validated indirectly via scan/diff/graph consistency).
- **Artifacts:** Optional: store per-repo or per-run outputs (e.g. quickstart summary, drift report, compliance JSON) in `run_results/` with a short note (e.g. “TC-DIS-1, TC-VAL-1 passed on express”).

---

## 7. Optional: automation sketch

A script could:

1. Run `sruja quickstart -r <repo>` and assert non-empty output and exit 0.
2. Run `sruja drift -r <repo>` and assert violation list or “no violations” and parseable output.
3. Run `sruja drift -r <repo> -a <arch.sruja>` when a baseline exists and assert diff output.
4. Run `sruja intent check -r <repo> -i <dir>` when an intent dir exists and assert report with drift_score and violations.
5. Run `sruja compliance -r <repo> -f json` and assert JSON with `status` and `remediation_checklist`.
6. Run `sruja why "Why X?" -r <repo>` and assert answer and evidence lines.

This would live in `evaluation/real-world-test/` or `scripts/testing/` and reuse existing helpers (e.g. `lib.sh`, `find_sruja`).
