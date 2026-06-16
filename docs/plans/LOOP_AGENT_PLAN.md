# Loop Agent Plan — Sruja as a CLI-first autonomous coding agent

> **Status:** In progress (Phases 0–2, 5 shipped). **Supersedes** the "Sruja is a
> passive harness, NOT an orchestrator agent" stance in `AGENTS.md` and ~12 docs.
> The docs reversal (Phase 5) is complete.

## 1. Decision

Sruja becomes a **standalone, CLI-first autonomous coding agent**. It owns the
full observe → act → verify → critique → replan loop and runs until a verifiable
condition is met or a budget is exhausted.

- **CLI first.** `sruja agent run "<goal>"` drives the whole loop.
- **Provider-agnostic.** Built on the existing `LlmClient` trait; concrete
  providers chosen later. No editor/Cursor dependency.
- **GitHub integration later** (issue intake, PR creation, scheduled runs) — out
  of scope for the initial build.
- **Builds on `sruja-agent`**, not greenfield. The cognition pipeline
  (`comprehend → plan → execute → critique → reflect`), `LlmClient`,
  `ToolRegistry`, `AgenticMemory`, DLC, pair/multi/matts already exist. The work
  is **close the gaps + wire a runner + un-harness the docs**.

This is the *loop engineering* thesis: the agent that writes the code is **not**
the one that grades it. Sruja's deterministic layer (`verify-task`, drift, lint,
intent) is the independent grader; `sruja-agent` is the actor. Both in one tool.

## 2. Why consolidate first

Two parallel half-built "agents" exist today, both incomplete:

1. **`sruja-agent` library** — full cognition incl. `Critique` (the independent
   reviewer) + `reflect`, but **linear**. `Critique.approved` is computed and
   discarded (`cognition/mod.rs:680-684`). `DlcConfig.max_iterations`
   (`dlc/mod.rs:179`, "for retry loops") is **dead code**.
2. **CLI `agent_run`** (`crates/sruja-cli/src/commands/agent_run.rs`) — observe →
   plan → **verify only**; the ACT phase is deliberately skipped
   (`agent_run.rs:1024-1026`); no critique, no reflect.

Plus fragmentation: risk lives in 5 places (`context` `TaskRisk`, display-only
in `focus`, `RiskLevel` in `critique`, `RiskItem` in `what_if`, `RiskSummary` in
`confidence`); evidence in two dirs (`.sruja/runs/<id>/` vs
`.sruja/agent/runs/<id>/`) with two schemas.

**Adding loop features on top of that multiplies the mess.** Consolidation *is*
the first loop work: closing the critique→replan edge creates the loop spine;
unifying risk enables the "is-this-agent-ready" gate; comparable per-iteration
evidence enables convergence detection.

## 3. Best-practice alignment (the spec we build to)

| Practice (SWE-agent/LangGraph) | sruja has | gap this plan closes |
|---|---|---|
| Multi-agent + dynamic routing | `ModelMapping`/`TaskTier`/`ModelRouter`, `multi/`,`pair/`,`matts/`, `Critique` | orchestrator→specialist dispatch (Phase 2) |
| Layered context (LSP/AST, skeletons, scratchpad) | repomap, `focus`, blast-radius, MCP ladder, `context_events.jsonl` | agent-callable LSP/AST + diff-edit tools, explicit scratchpad (Phase 3) |
| Typed tools + timeouts + error contracts | `Tool` trait, `FileGuard`, typed `VerifyTaskOutput`, allowlists | typed shell returns, hard timeouts (Phase 3) |
| ReAct loop + TDD + diff editing | TDD pipeline + `FileGuard` alternation, `verify`, `Critique` | **loop spine + SEARCH/REPLACE editing** (Phase 0, 3) |
| Guardrails + sandbox | `.srujaignore`, boundaries, drift, `spend_cap_usd`, worktree `SandboxConfig` | team-guidelines injection, mandatory sandbox for ACT (Phase 4) |

## 4. Phased delivery

### Phase 0 — Loop spine (slice 0, this session) ✅
**Close the critique→replan loop in `sruja-agent`.** Provider-agnostic, mock-tested.
- New types in `cognition/mod.rs`: `LoopConfig`, `LoopIteration` (per-iteration
  evidence), `LoopTermination` (`Approved`/`MaxIterations`/`NoReplan`/`Aborted`),
  `LoopResult`.
- `Agent::replan(goal, comprehension, critique)` — plans again with the prior
  critique's `issues`/`suggestions` injected.
- `Agent::run_loop(goal, &LoopConfig)` — comprehend once, then iterate
  (re)plan→execute→critique; terminate on approval, budget, or hard error.
- Mock-LLM tests: converges on iteration 2; exhausts budget; honors `stop_on_approval=false`.
- **Exit criteria:** `cargo test -p sruja-agent` green; `Agent::run` unchanged
  (backward compatible).

### Phase 1 — Real ACT phase ✅ (library done)
Make execution actually mutate via tools (file edit, shell), not leave ACT to a
host. Enforce `FileGuard` scope; convert the CLI `agent_run` ACT-skip into a
call into `sruja-agent`. Provider-agnostic (tools, not model, do the mutation).

**Done (library):**
- Fixed `Agent::execute` double-LLM-call bug: was `complete_tiered` (one-shot,
  tool calls silently ignored) then `run_tool_loop` on the **default** model with
  the first response fed back as a user message. Now a **single tiered
  `run_tool_loop`** — tier routing preserved through the tool loop, no wasted
  re-prompt. Extracted `Agent::model_for_tier` helper.
- Added `ToolRegistry::with_builtin(root, shell_allowlist)` — registers the 6
  built-in tools (file_read, file_write, file_edit, glob, grep, shell) rooted at
  a workspace. This is what a standalone agent needs to mutate.
- Integration test `run_loop_actually_mutates_files_via_tools`: a scripted LLM
  issues a `file_write` tool call during execute; the test asserts the file
  exists on disk after `run_loop`. Proves the full loop
  (comprehend→plan→**execute via tools**→critique→approve) mutates the FS
  through the `FileGuard`.

**Remaining (CLI wiring = slice 2):** convert `agent_run.rs` ACT-skip into a
call into `sruja-agent::run_loop`.

### Phase 2 — CLI runner (slice 2, this session) ✅
`sruja agent loop` drives the full closed loop against a real workspace.

**Done:**
- New CLI subcommand `sruja agent loop --goal "..."` with flags:
  `--max-iterations`, `--no-tdd`, `--dry-run`, `--model`, `--base-url`,
  `--format json|text`.
- `.sruja/loop.toml` manifest support: `max_iterations`, `tdd`,
  `review_every_change`, `dry_run`, `shell_allowlist`, `model`, `base_url`,
  and `[models]` section for per-tier model routing. CLI flags override
  manifest, manifest overrides defaults.
- Wires `OpenAiClient::from_env()` → `ToolRegistry::with_builtin(repo, …)` →
  `Agent::builder().build()` → `run_loop(goal, &LoopConfig)`.
- Human-readable output: per-iteration PASS/FAIL with critique score/issues,
  total token usage, convergence status. JSON output via `--format json`.
- API key resolution: `OPENAI_API_KEY` or `SRUJA_ENRICH_API_KEY`.
- Added `AgentError` and `ModelMapping` to `sruja_agent` root re-exports.
- All tests pass: 84 agent + 187 CLI + 7 doc-tests. Clippy clean.

### Phase 2b — Specialist decomposition
Split the monolithic `Agent::run` into roles routed by `TaskTier`:
orchestrator (premium, plan-only), coder (mid), reviewer (review model =
`Critique`), terminal (cheap). Parallel specialist sub-agents via existing
`multi/`. This activates practice #1 without a new orchestration framework.

### Phase 3 — Production tooling
Diff-based editing (`SEARCH/REPLACE` / git-patch tool), typed shell returns
(`{pass, fail, error_line, stack_trace}`), hard timeouts, agent-callable
LSP/AST (go-to-definition, find-refs), explicit scratchpad artifact updated
per iteration.

### Phase 4 — Guardrails & convergence ✅
**Spend cap:** `LoopConfig.spend_cap_usd` checked after each iteration via
`Usage::estimated_cost_usd()`. Also catches `LlmError::BudgetExceeded` from the
`ModelRouter` and terminates gracefully with `LoopTermination::SpendCapExceeded`
instead of propagating the error.

**Oscillation detection:** `LoopConfig.detect_oscillation` (default: true)
hashes critique-issue signatures per iteration. If the same signature repeats,
the loop terminates with `LoopTermination::Oscillation` — preventing flailing.

**CLI flags:** `--spend-cap <USD>` and `--no-oscillation-detection` wired into
`sruja agent loop`. Both overridable in `.sruja/loop.toml`.

**Tests:** `run_loop_terminates_on_spend_cap`, `run_loop_detects_oscillation`,
`usage_estimated_cost_is_nonzero`, `critique_signature_normalises_order`.

Remaining Phase 4 work: team-guidelines auto-injection, mandatory sandbox
(worktree today; Docker/cloud later), per-iteration evidence comparison across
runs (`verification_hash`/`content_hash` reader).

### Phase 5 — Docs reversal ✅
Rewrote the "harness, not agent" / "no `--autonomous` mode" stance across all
docs. The loop-agent stance is now the primary narrative. What is **preserved**:
the *independent grader* principle (Sruja's deterministic layer still grades
its own actor deterministically).

**Files updated (18):**
- `AGENTS.md` — section headers, body, Do/Do-not list
- `docs/GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md` — title, intro, boundary
  table, correction table, "what Sruja does not do", orchestration section
- `docs/HOST_AGENT_INTEGRATION.md` — intro, trust model, boundary table, ACT
  step, references
- `docs/ENTERPRISE_ADOPTION.md` — opening paragraph
- `docs/FEATURE_TIERS.md` — MCP position, agent ops, fold table, loop section
- `docs/GETTING_STARTED_SKILL.md` — subtitle, prerequisites, playbook scope
- `docs/PRIVACY_AND_RETENTION.md` — opening paragraph
- `docs/mcp_setup.md` — intro paragraph
- `docs/CONTEXT_ENGINEERING.md` — MCP section
- `docs/architecture/domain-schema.md` — see-also link
- `docs/architecture/README.md` — domain-schema reference
- `docs/README.md` — table entry
- `docs/examples/host-gates/README.md` — intro
- `docs/plans/AGENT_DELIVERY_PLAN.md` — north star, non-goals, acceptance
- `docs/plans/AI_DLC_ACE_SYNTHESIS_PLAN.md` — intro, will-not-ship list
- `book/src/getting-started.md` — FAQ
- `book/src/docs/intro.md` — design principles
- `book/src/courses/agentic-ai/` — course-overview, module-overview, lesson-4
- `book/src/courses/production-architecture/module-4-evolution/lesson-3.md`
- `.cursor/commands/sruja-reflect-on-run.md` — title, intro

Remaining Phase 5 work: wire `focus` grounding into `agent loop` and integrate
`verify-task` as the exit gate (deferred to Phase 3/4).

### Phase 6 (later) — GitHub integration
Issue intake ("agent-ready" task detection), PR creation, scheduled/unattended
runs via GitHub Actions closed-loop template.

## 5. Stance reconciliation (for review)

The old stance opposed Sruja *owning the LLM loop and running unattended*.
This plan reverses that. What is **preserved**:

- **Independent grading.** The deterministic layer (`verify-task`, drift, lint,
  intent) remains the authoritative reviewer — the actor never grades itself.
- **Determinism where it matters.** Gates are still blake3-hashable, replayable,
  machine-readable PASS/FAIL.
- **Closed-loop by default.** Autonomous runs are bounded by a loop manifest
  (goal, scope, budget, exit gate, kill conditions) — not open-ended.

## 6. Risks
- **Scope creep into "compete with Claude Code."** Mitigation: CLI-first,
  provider-agnostic, GitHub later; one phase at a time.
- **Two agent paths diverging again.** Mitigation: Phase 1 makes CLI
  `agent_run` a thin shell over `sruja-agent::run_loop`.
- **Cost/runaway.** Mitigation: `spend_cap_usd` (exists) + `max_iterations`
  (Phase 0) + convergence kill (Phase 4).
