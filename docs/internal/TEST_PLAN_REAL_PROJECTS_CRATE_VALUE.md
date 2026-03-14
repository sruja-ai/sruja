# Test plan: proving developer value on real projects and crate contribution to architecture maintenance

**Purpose:** Prove that each crate matters because it improves a real developer workflow on a real repo. Crate value is only "proven" when a developer can answer a question, take action, or gate a change with the output.

**Related:** [CRATE_VALUE_AUDIT.md](CRATE_VALUE_AUDIT.md), [VALUE_ON_REAL_PROJECTS.md](../../evaluation/real-world-test/VALUE_ON_REAL_PROJECTS.md), [TEST_ON_REAL_PROJECTS.md](../../evaluation/real-world-test/TEST_ON_REAL_PROJECTS.md), [run_demo.sh](../../evaluation/real-world-test/run_demo.sh)

---

## 1. Developer value we need to prove

We should lead with developer workflows, not crate count.

| Workflow | Developer question | Primary command(s) | Value to prove |
|----------|--------------------|--------------------|----------------|
| **First value** | "What is the shape of this repo and what should I look at first?" | `sruja quickstart -r <repo>`, `sruja drift -r <repo>` | Zero-setup architecture inventory, health, and actionable findings with evidence. |
| **Baseline and docs** | "Can I create or validate architecture-as-code fast?" | `sruja quickstart -r <repo> --generate-baseline`, `sruja lint`, `sruja export` | Developers can create, validate, and publish a baseline without hand-drawing diagrams first. |
| **Docs vs code** | "Does our declared architecture still match the code?" | `sruja drift -r <repo> -a architecture.sruja`, `sruja intent check -r <repo> -i <dir>` | Gaps between declared and actual structure are concrete and actionable. |
| **CI / PR gate** | "Should this change pass architecture checks?" | `sruja drift --fail-on ...`, `sruja drift-pr -b <base> -H <head>`, `sruja compliance -f json` | The tool can fail correctly, report machine-readable results, and focus on new violations. |
| **AI and editor context** | "Can I feed architecture context into Cursor/Copilot or edit .sruja with good DX?" | `sruja context -r <repo> -f <format>`, extension lint/export | Architecture data is reusable by AI tools and available in-editor without extra friction. |
| **Hotspot explanation** | "Why is this repo hard to change, and where should we refactor first?" | `sruja analyze -r <repo> -f json`, `sruja why "..." -r <repo>` | Sruja identifies hotspots and can explain answers with evidence, not only a score. |

**Priority rule:** P0 value is `quickstart`, `drift`, baseline generation/comparison, `analyze`, `context`, and a CI-safe gate. P1 value is `intent check`, `why`, and extension-specific flows.

---

## 2. Architecture maintenance dimensions

We still map work to architecture-maintenance dimensions, but each dimension must produce a developer-facing outcome.

| Dimension | Goal | Developer-facing outcome | Example artifact |
|-----------|------|--------------------------|------------------|
| **Document** | Keep architecture-as-code valid and exportable | `.sruja` is lintable and can become diagrams/docs | lint JSON, Mermaid, Markdown |
| **Discover** | Infer structure from code without prior docs | A developer gets inventory and evidence from a repo quickly | quickstart JSON, scan graph JSON |
| **Validate** | Catch structural problems | A developer sees cycles, orphans, layer problems, hotspots | drift report, fail-on exit |
| **Compare** | Align code with declared architecture or ADRs | A developer can see what docs forgot or got wrong | drift vs baseline, intent report |
| **Enforce** | Provide a single machine-readable gate | CI can fail or pass on architectural regressions | compliance JSON, drift-pr JSON |
| **Explain** | Help teams understand hotspots and decisions | A developer gets evidence-backed rationale or recommendations | analyze JSON, why output |

---

## 3. Crate to workflow mapping

| Crate | Primary user-facing value | Best proof path |
|-------|---------------------------|-----------------|
| **sruja-types** | Stable node/edge kinds across scan, graph, diff, and reports | Raw scan graph has consistent `nodes` / `edges` kinds; downstream commands consume it without translation bugs |
| **sruja-diagnostics** | Consistent parse/lint/export error shape | `sruja lint <file> --format json` returns codes, severity, and location |
| **sruja-language** | Every `.sruja` workflow starts here | lint, export, drift with `-a`, and intent loading all parse the same file successfully |
| **sruja-engine** | Rule-based DSL validation | lint catches invalid references or structural rule violations |
| **sruja-export** | Diagrams and docs from `.sruja` | `sruja export json|mermaid|markdown <file>` produces reusable artifacts from declared architecture |
| **sruja-scan** | Real code becomes graph data | quickstart, drift, analyze, why, context, intent, and compliance all scan the repo |
| **sruja-diff** | Structural drift and declared-vs-actual comparison | `sruja drift`, `sruja drift -a ...`, `sruja drift-pr` |
| **sruja-graph** | Explanation and hotspot analysis | `sruja analyze`, `sruja why`, and compliance policy evaluation |
| **sruja-intent** | Intent vs reality comparison | `sruja intent check`, `sruja compliance -i ...` |
| **sruja-report** | Canonical compliance JSON for CI/tools | `sruja compliance -f json` emits `status`, `health_score`, `drift_entries`, `policy_violations`, `remediation_checklist` |
| **sruja-cli** | All value is delivered through one binary | Every scenario below runs through the CLI |
| **sruja-lsp** | In-editor `.sruja` diagnostics and completions | VS Code extension shows diagnostics without a manual CLI loop |
| **sruja-wasm** | Extension lint/export without a local CLI binary | Extension works when `sruja.lsp.path` is unset and bundled WASM is used |

---

## 4. Real-project scenario matrix

Use real repos from `evaluation/real-world-test/test-repos/`, this repo, or any supported-language repo. Build once with `make build`, then use `./target/release/sruja` or ensure `sruja` is on `PATH`.

### P0 scenarios: must prove core developer value

| ID | Workflow | Command(s) | Expected outcome | Evidence to save | Crates exercised |
|----|----------|------------|------------------|------------------|------------------|
| **TC-CORE-1** | First-value architecture snapshot | `sruja quickstart -r <repo> -f json` | JSON includes `health_score`, `inventory`, `top_findings`, and `actionable_fixes` with file evidence | quickstart JSON | sruja-scan, sruja-diff, sruja-cli |
| **TC-CORE-2** | Structural drift and fail gate | `sruja drift -r <repo> -f json` and `sruja drift -r <repo> --fail-on cycles` | Drift report lists violations/suggestions; exit code flips when requested violation exists | drift JSON plus exit code | sruja-scan, sruja-diff, sruja-cli |
| **TC-CORE-3** | Generate a baseline, then compare docs vs code | `sruja quickstart -r <repo> --generate-baseline`; `sruja lint <repo>/architecture.sruja --format json`; `sruja drift -r <repo> -a <repo>/architecture.sruja -f json` | Baseline file is created, lintable, and useful for declared-vs-actual drift | generated `architecture.sruja`, lint JSON, drift-vs-baseline JSON | sruja-scan, sruja-diff, sruja-language, sruja-engine, sruja-diagnostics, sruja-cli |
| **TC-CORE-4** | Refactor hotspot analysis | `sruja analyze -r <repo> -f json` | Output includes `health_score`, `architecture_completion_score`, and at least one concrete recommendation or hotspot | analyze JSON | sruja-scan, sruja-graph, sruja-cli |
| **TC-CORE-5** | AI-ready architecture context | `sruja context -r <repo> -f json` and `sruja context -r <repo> -f cursor-rules -o /tmp/out.cursorrules` | Context includes summary, layers, boundaries, and reusable output for AI tools | context JSON or generated rules file | sruja-scan, sruja-cli |
| **TC-CORE-6** | CI-safe compliance gate | `sruja compliance -r <repo> -f json > /tmp/compliance.json` | JSON output has canonical report shape; non-zero exit is expected when repo is non-compliant | compliance JSON plus exit code | sruja-scan, sruja-diff, sruja-intent, sruja-graph, sruja-report, sruja-cli |
| **TC-CORE-7** | PR-scoped regression detection | `sruja drift-pr -r <repo> -b <base-ref> -H <head-ref> -f json` | Report isolates new violations between refs instead of listing the full repo history | drift-pr JSON | sruja-scan, sruja-diff, sruja-cli |

### P1 scenarios: deeper proof and extension proof

| ID | Workflow | Command(s) | Expected outcome | Evidence to save | Crates exercised |
|----|----------|------------|------------------|------------------|------------------|
| **TC-SUP-1** | Raw graph export | `sruja scan <repo> --output /tmp/graph.json` | Graph JSON has `nodes` and `edges`; downstream tools can reuse it | scan graph JSON | sruja-scan, sruja-types, sruja-cli |
| **TC-SUP-2** | DSL validation and export | `sruja lint book/valid-examples/pattern-microservices.sruja --format json`; `sruja export json <file>`; `sruja export mermaid <file>`; `sruja export markdown <file>` | `.sruja` file is parseable, lintable, and exportable to multiple formats | lint JSON, exported Mermaid/Markdown/JSON | sruja-language, sruja-engine, sruja-diagnostics, sruja-export, sruja-cli |
| **TC-SUP-3** | Intent check with ADRs or declared architecture | `sruja intent check -r <repo> -i <intent_dir> -f json` | Report shows drift score and missing/undocumented components or relationships | intent JSON | sruja-intent, sruja-language, sruja-scan, sruja-cli |
| **TC-SUP-4** | Evidence-backed "why" answer | `sruja why "Why do we use <technology>?" -r <repo>` | Answer includes confidence plus graph/file evidence | saved why output | sruja-scan, sruja-graph, sruja-cli |
| **TC-SUP-5** | LSP diagnostics in editor | Open a `.sruja` file in VS Code, introduce an error, confirm diagnostics | Editor reports location-aware validation without running commands manually | screenshot or short note | sruja-lsp, sruja-language, sruja-engine, sruja-diagnostics |
| **TC-SUP-6** | WASM lint/export without CLI | In the extension, leave `sruja.lsp.path` unset and use lint or Mermaid export | Extension still validates and exports via WASM | screenshot or short note | sruja-wasm, sruja-export, sruja-language, sruja-engine |

---

## 5. Recommended repo mix

| Repo type | Suggested repo | Why it helps |
|-----------|----------------|--------------|
| **Fast smoke test** | `evaluation/real-world-test/test-repos/express` | Small, quick, good for `quickstart`, `drift`, and baseline comparison |
| **Large real system** | `evaluation/real-world-test/test-repos/gitea` | Proves scale, hotspot detection, and report usefulness on a large codebase |
| **Intent-rich repo** | `.` (this repo) or any repo with `docs/architecture` / `docs/adr` | Best for `intent check`, `compliance`, and `why` with stronger context |
| **PR history test** | Any cloned git repo with multiple refs | Needed for `drift-pr` and commit-based proof |
| **Admin / ecommerce realism** | `react-admin`, `saleor` from the complex set | Proves value on non-framework product repos, not only toy or framework repos |

---

## 6. Known proof gaps and how to improve them

| Gap | Why it matters to developers | Improvement |
|-----|------------------------------|-------------|
| **Outdated command examples break trust immediately** | If docs say `sruja context export` or `sruja scan -r -o`, users hit failures before they see value | Keep this plan aligned with `sruja --help`; add a smoke test that validates every documented command shape |
| **The old plan was too crate-first** | Developers buy workflows, not internal module count | Lead with P0 workflows and keep crate mapping as traceability, not the headline |
| **We do not measure actionability or false positives yet** | A report can be "correct" but still not help a developer make a decision | For each repo, record the top 3 findings as `useful`, `expected`, or `noisy`, with one-line reviewer notes |
| **CI / PR proof is weaker than local proof** | Real developer value is catching new regressions, not re-listing the whole repo state | Require at least one `drift-pr` run on a repo with real refs and save base/head/new violation evidence |
| **Context-export ownership is muddy** | The user-facing `sruja context` flow currently proves CLI value more than `sruja-export` value | Either route `sruja context` through `sruja-export` or keep `sruja-export` proof limited to `sruja export ...` and update crate audits accordingly |
| **`why` quality depends on intent richness** | On code-only repos, answers may be generic even when the graph path works | Run `why` on at least one repo with ADRs or an intent dir; otherwise mark the result as graph-path proof only |
| **Extension proof is still mostly manual** | Manual smoke tests are easy to skip and easy to regress | Keep manual LSP/WASM checks, but also rely on `wasm-pack test --node` and one extension smoke test in CI where possible |
| **`analyze` needs outcome proof, not just score proof** | A score alone does not tell a team what to refactor | Require each analyze run to produce at least one concrete hotspot or recommendation with evidence |
| **Unsupported languages and dynamic imports can skew results** | False-positive orphans and missing edges reduce trust | Include repo diversity in the matrix and capture scanner limitations next to each run result |

---

## 7. Success criteria for "value proven"

- **First value:** On at least one real repo, `sruja quickstart -r <repo>` reaches actionable output in about 2 minutes from a built CLI.
- **Actionable evidence:** Across the tested repos, we capture at least 3 findings with file evidence that a reviewer judges useful or worth checking.
- **Docs vs code:** At least one run shows a concrete mismatch between declared architecture and scanned reality via `drift -a` or `intent check`.
- **CI / PR readiness:** At least one machine-readable gate (`drift --fail-on ...`, `drift-pr`, or `compliance`) returns the correct non-zero exit when violations exist.
- **AI / editor readiness:** At least one context export artifact and one editor-or-WASM validation flow succeed.
- **Per crate:** Every crate in Section 3 is exercised by at least one passing scenario. `sruja-types` remains indirect via scan output consistency.
- **Known limits recorded:** Every repo run stores a short note about scanner blind spots, false positives, or missing intent so results stay credible.

---

## 8. Execution checklist

**One-time setup**

- [ ] Build the CLI: `make build`
- [ ] Optional test repos: `cd evaluation/real-world-test && ./setup_repos.sh`
- [ ] Pick one small repo, one larger repo, and one repo with intent/ADR material if available

**Minimum run order**

1. `sruja quickstart -r <repo> -f json`
2. `sruja drift -r <repo> -f json`
3. `sruja drift -r <repo> --fail-on cycles`
4. `sruja quickstart -r <repo> --generate-baseline`
5. `sruja lint <repo>/architecture.sruja --format json`
6. `sruja drift -r <repo> -a <repo>/architecture.sruja -f json`
7. `sruja analyze -r <repo> -f json`
8. `sruja context -r <repo> -f json`
9. `sruja context -r <repo> -f cursor-rules -o /tmp/out.cursorrules`
10. `sruja intent check -r <repo> -i <intent_dir> -f json`
11. `sruja compliance -r <repo> -f json > /tmp/compliance.json`
12. `sruja drift-pr -r <repo> -b <base-ref> -H <head-ref> -f json`
13. `sruja why "Why do we use <technology>?" -r <repo>`
14. Optional extension proof: LSP and WASM smoke tests

**Important notes**

- `sruja scan` uses positional repo path plus `--output`, for example: `sruja scan <repo> --output /tmp/graph.json`
- `sruja context` is a single command, not `sruja context export`
- `sruja compliance` exits with code `1` when the repo is non-compliant; that non-zero exit is part of the proof
- `sruja drift-pr` requires a real git repo and refs

**Fast path**

- `cd evaluation/real-world-test && ./run_demo.sh`
- `cd evaluation/real-world-test && ./run_demo.sh --baseline`

---

## 9. Recommended automation

Create a small smoke script under `evaluation/real-world-test/` that:

1. Validates documented command shapes against `sruja --help`
2. Runs `quickstart`, `drift`, `analyze`, `context`, and `compliance` on one fast repo
3. Optionally runs `drift-pr` when git history is available
4. Stores JSON outputs plus a short reviewer note in `run_results/`
5. Fails if a documented P0 workflow no longer runs or no longer returns the expected output shape

That automation is more valuable than a crate-only checklist because it protects the workflows developers actually rely on.
