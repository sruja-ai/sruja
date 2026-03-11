# Plan: Next Steps of Improvements

Prioritized plan for the next phase of Sruja improvements, based on [SRUJA_SKILL_IMPROVEMENTS.md](SRUJA_SKILL_IMPROVEMENTS.md), [ARCHITECTURE_ANALYSIS_IMPROVEMENTS.md](../evaluation/results/comparison_express_20260310_0915/ARCHITECTURE_ANALYSIS_IMPROVEMENTS.md), and current state.

---

## Phase 1: Quick wins (1–2 days)

| # | Task | Why | Owner / note |
|---|------|-----|--------------|
| 1.1 | **Fix Mermaid link count in summarize script** | Comparison reports "0 links" for Mermaid while the file has many `-->`; metrics are misleading. | `scripts/summarize_comparison.sh`: adjust `grep` pattern or parse Mermaid for arrow lines; or document "Links: approximate for Mermaid". |
| 1.2 | ~~**Add CHANGELOG to agent skill**~~ | ~~Users and evaluators can tie behavior to a version~~ | **Done:** `skills/sruja-architecture-agent/CHANGELOG.md` added; version 1.2.0 in SKILL. |
| 1.3 | **Link "super prompt" from README / INSTALL_AS_SKILL** | One place for the copy-paste prompt that gives best results. | Add a short "Recommended prompt" section in `docs/INSTALL_AS_SKILL.md` and/or main README pointing to the prompt in the agent SKILL. |

---

## Phase 2: Skill and REFERENCE content (2–4 days)

| # | Task | Why | Owner / note |
|---|------|-----|--------------|
| 2.1 | **Align all REFERENCE examples to canonical DSL** | Some examples may still use block form without IDs, or `metadata { }` / `external_system` if not in language spec. Ensures "skill example = lintable". | Audit `skills/sruja-architecture-agent/REFERENCE.md` (and any example .sruja in skills); replace with assignment form `Id = kind "Label" { }`, and replace unsupported constructs with documented ones. |
| 2.2 | **Strengthen "fix until lint passes" in agent flow** | Last run had one cycle left unfixed; agent returned without re-running lint after a fix. | In SKILL or REFERENCE: add an explicit loop: "Run lint → if errors, apply fixes from lint→fix table → re-run lint. Repeat until pass. Do not present until pass." Optionally add one example: "If E204 circular dependency: remove one edge in the cycle (e.g. NodeHTTPServer -> Application), re-run lint." |
| 2.3 | **Expand per-language hints (optional)** | Improves accuracy of `technology` and entry-point detection for more stacks. | REFERENCE already has a table (Express, FastAPI, Django, Spring Boot, Next.js). Add 1–2 more if high value (e.g. Go Gin, NestJS, Rails) and ensure entry points and technology strings are precise. |

---

## Phase 3: Testing and comparison (2–3 days)

| # | Task | Why | Owner / note |
|---|------|-----|--------------|
| 3.1 | **Run comparison on 1–2 more repos** | Validate skill improvements on different stacks (e.g. FastAPI, a small app). | Run `./scripts/run_comparison_test.sh fastapi https://github.com/tiangolo/fastapi` (and optionally one app); run `summarize_comparison.sh`; note lint pass/fail and scope. |
| 3.2 | **Record "lint pass rate" and scope in comparison runs** | Measure whether "mandatory lint" and cycle fix are consistently applied. | In `run_comparison_test.sh` or a small wrapper: capture lint exit code and log "Sruja lint: pass/fail"; in README or summary, record last N runs (e.g. express pass after cycle fix, fastapi pass/fail). |
| 3.3 | **Optional: automate comparison in CI (cron or weekly)** | Keeps Mermaid vs Sruja comparison and skill quality regression visible. | If agent can run in CI (e.g. with `agent --trust` and a timeout), add a job that runs comparison on express (or a small fixture), saves artifacts, and fails if Sruja lint fails. Otherwise document "run locally" and keep a manual log. |

---

## Phase 4: Docs and discoverability (1–2 days)

| # | Task | Why | Owner / note |
|---|------|-----|--------------|
| 4.1 | **Single "Getting started with the skill" page** | One path: install skill → run one prompt → validate → optional drift. | New or extended doc (e.g. `docs/GETTING_STARTED_SKILL.md` or section in INSTALL_AS_SKILL): install, paste super prompt, run `sruja lint`, optional `sruja drift -a architecture.sruja -r .`. |
| 4.2 | **Index of evaluation and comparison artifacts** | So "what did we test and what did we learn?" is easy to find. | In `evaluation/README.md` or `evaluation/real-world-test/README.md`: list comparison runs (with links to `comparison_*` dirs), link to IS_SRUJA_HELPFUL.md and ARCHITECTURE_ANALYSIS_IMPROVEMENTS.md, and to NEXT_STEPS_IMPROVEMENTS.md. |
| 4.3 | **DEVELOPMENT.md: add "Skills and evaluation"** | Developers see where skill and comparison work lives. | Add a short subsection: skill source in `skills/`, comparison script `scripts/run_comparison_test.sh`, evaluation results in `evaluation/results/`, improvement plan in `docs/NEXT_STEPS_IMPROVEMENTS.md`. |

---

## Phase 6: Sruja code (Rust crates)

| # | Task | Why | Where |
|---|------|-----|--------|
| 6.1 | ~~**Fix git_check panic**~~ | ~~`git_check.unwrap()` can panic~~ | **Done:** use `.ok().and_then(|o| o.status.success().then_some(())).is_some()`. |
| 6.2 | ~~**Harden worktree path handling**~~ | ~~`worktree_dir.to_str().unwrap()` panics on non-UTF8~~ | **Done:** use `.arg(worktree_dir.as_path())` so paths stay as `OsStr`. |
| 6.3 | ~~**Add e2e test for `sruja lint`**~~ | ~~Lint is the main entry point~~ | **Done:** `crates/sruja-cli/tests/lint_e2e.rs` (3 tests: valid succeeds, invalid fails, nonexistent fails). |
| 6.4 | **Replace library unwraps in algorithms** | sruja-diff (source_ref, drift) and sruja-graph (scc) use `unwrap()` on invariants; use safe access or debug_assert. | `sruja-diff/src/source_ref.rs`, `sruja-diff/src/drift.rs`, `sruja-graph/src/scc.rs` |
| 6.5 | ~~**Share e2e test helper**~~ | ~~Duplicated run_sruja/create_test_repo/write_file~~ | **Done:** `crates/sruja-cli/tests/common.rs`; analyze_e2e, drift_e2e, quickstart_e2e, lint_e2e use it. |
| 6.6 | **Resolve or document cycle-rule / causal_loop TODOs** | cycle.rs tests and markdown_export ignored test; document or fix. | `sruja-engine/src/rules/cycle.rs`, `sruja-export/tests/markdown_export.rs` |
| 6.7 | **Doc comments for main public APIs** | sruja-scan, sruja-diff, sruja-intent, sruja-graph lack docs on key entry points. | Various `lib.rs` and main modules |
| 6.8 | **Dead code** | `ConfigError::NotFound` has `#[allow(dead_code)]`; use or remove. | `crates/sruja-cli/src/config.rs` |

---

## Phase 5: Product and tooling (backlog)

| # | Task | Why | Owner / note |
|---|------|-----|--------------|
| 5.1 | **Linter: optional "suggest cycle fix"** | When E204 is reported, suggest which edge to remove (e.g. by direction or name). | In sruja-engine or CLI: for cycle diagnostic, optionally emit a hint like "Consider removing relationship NodeHTTPServer -> Application to break cycle." |
| 5.2 | ~~**Export: Sruja → Mermaid**~~ | Lets users get a diagram from a .sruja file for comparison or presentation. | **Done:** `sruja export mermaid <file>` exists; documented in [GETTING_STARTED_SKILL.md](GETTING_STARTED_SKILL.md) and [evaluation/README.md](../evaluation/README.md#same-content-two-formats-sruja--mermaid). |
| 5.3 | **Skill version in generated files (optional)** | Trace which skill version produced a file. | Optional comment in generated .sruja, e.g. `// Generated with sruja-architecture-agent (skill version 1.1.0)`. Low priority. |

---

## Summary table

| Phase | Focus | Rough effort |
|-------|--------|--------------|
| **1** | Quick wins: Mermaid metrics, CHANGELOG, super-prompt link | 1–2 days |
| **2** | Skill/REFERENCE: canonical examples, lint-until-pass, language hints | 2–4 days |
| **3** | Testing: more repos, lint pass rate, optional CI | 2–3 days |
| **4** | Docs: getting started, evaluation index, DEVELOPMENT.md | 1–2 days |
| **5** | Backlog: linter hints, export, versioning | As capacity |
| **6** | Sruja code: panic fixes, e2e lint, shared test helper, docs | 2–3 days |

---

## Success criteria (next 2–4 weeks)

- [x] Mermaid link count fixed or documented in comparison summary.
- [x] Agent skill has CHANGELOG and version bump for recent changes.
- [ ] At least one more comparison run (e.g. FastAPI) with results and lint status recorded. *(Run: `./scripts/run_comparison_test.sh fastapi https://github.com/tiangolo/fastapi`; then `./scripts/summarize_comparison.sh evaluation/results/comparison_fastapi_<timestamp>`.)*
- [x] REFERENCE examples use only canonical DSL (assignment form; no unsupported constructs).
- [x] "Fix until lint passes" is explicit (with cycle-fix example) in skill/REFERENCE.
- [x] README or INSTALL_AS_SKILL links the recommended "super prompt" for the skill.
- [x] DEVELOPMENT.md or evaluation README points to skills, comparison script, and this plan.
- [x] **Code (Phase 6):** git_check and worktree path panics fixed in `scan.rs`. Remaining: lint e2e, shared e2e helper, library unwraps, docs.

Use this plan as a living doc: complete items, add new ones from feedback, and adjust phases as priorities change.
